use crate::crypto;
use crate::db;
use crate::gui::tools::{ToolEvent, ToolScreen};
use crate::i18n::{Language, text};
use eframe::egui;
use ssh;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

#[derive(Clone)]
struct BackupLog {
    time: String,
    device: String,
    status: String,
}

#[derive(Clone)]
struct BackupDevice {
    id: i32,
    name: String,
    ip: String,
    user: String,
    enc_cred: String,
}

#[derive(Clone)]
struct BackupRunOptions {
    max_attempts: i32,
    timeout_secs: i32,
    retention_keep: i32,
    details: &'static str,
}

pub struct BackupTool {
    master_pass: String,
    unlocked: bool,
    interval_hours: i32,
    max_attempts: i32,
    timeout_secs: i32,
    retention_keep: i32,
    is_running: Arc<Mutex<bool>>,
    is_manual_running: Arc<Mutex<bool>>,
    logs: Arc<Mutex<Vec<BackupLog>>>,
}

impl Default for BackupTool {
    fn default() -> Self {
        Self {
            master_pass: String::new(),
            unlocked: false,
            interval_hours: 2,
            max_attempts: 2,
            timeout_secs: 15,
            retention_keep: 20,
            is_running: Arc::new(Mutex::new(false)),
            is_manual_running: Arc::new(Mutex::new(false)),
            logs: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

impl BackupTool {
    pub fn new() -> Self {
        Self::default()
    }

    fn run_backup(
        id: i32,
        ip: &str,
        user: &str,
        enc_cred: &str,
        master_pass: &str,
        timeout_secs: i32,
        retention_keep: i32,
    ) -> (bool, String) {
        let Ok(plain_pass) = crypto::decrypt_credential(enc_cred, master_pass) else {
            return (false, "Password decrypt failed".to_string());
        };

        let addr = format!("{}:22", ip);
        let timeout = Duration::from_secs(timeout_secs.max(1) as u64);
        let session = ssh::create_session()
            .timeout(Some(timeout))
            .username(user)
            .password(&plain_pass)
            .connect_with_timeout(&addr, Some(timeout));

        let Ok(sess) = session else {
            return (false, "SSH connection failed".to_string());
        };

        let mut local_sess = sess.run_local();
        local_sess.set_timeout(Some(timeout));
        let Ok(exec) = local_sess.open_exec() else {
            return (false, "Exec channel failed".to_string());
        };

        let res: Result<Vec<u8>, _> = exec.send_command("show running-config");
        let Ok(vec) = res else {
            return (false, "Command failed".to_string());
        };

        let config = String::from_utf8_lossy(&vec).into_owned();
        let Ok(conn) = db::get_connection() else {
            return (false, "Database connection failed".to_string());
        };

        match db::devices::save_config(&conn, id as i64, &config) {
            Ok(snapshot_id) => {
                let pruned = db::devices::prune_config_history(
                    &conn,
                    id as i64,
                    retention_keep.max(1) as i64,
                )
                .unwrap_or(0);
                (
                    true,
                    format!("Backup successful; snapshot #{snapshot_id}; pruned {pruned} old"),
                )
            }
            Err(e) => (false, format!("Config save failed: {e}")),
        }
    }

    fn fetch_devices() -> Vec<BackupDevice> {
        let mut devices = Vec::new();
        if let Ok(conn) = db::get_connection()
            && let Ok(mut stmt) = conn.prepare(
                "SELECT id, name, ip_address, username, encrypted_credentials FROM devices",
            )
            && let Ok(iter) = stmt.query_map([], |row| {
                Ok(BackupDevice {
                    id: row.get::<_, i32>(0)?,
                    name: row.get::<_, String>(1)?,
                    ip: row.get::<_, String>(2)?,
                    user: row.get::<_, Option<String>>(3)?.unwrap_or_default(),
                    enc_cred: row.get::<_, Option<String>>(4)?.unwrap_or_default(),
                })
            })
        {
            devices.extend(iter.flatten());
        }

        devices
    }

    fn append_log(logs: &Arc<Mutex<Vec<BackupLog>>>, device: &str, status: &str) {
        if let Ok(mut l) = logs.lock() {
            let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
            l.push(BackupLog {
                time: now,
                device: device.to_string(),
                status: status.to_string(),
            });
        }
    }

    fn execute_device_backup(
        device: BackupDevice,
        master_pass: &str,
        options: &BackupRunOptions,
        logs: &Arc<Mutex<Vec<BackupLog>>>,
        ctx: &egui::Context,
    ) {
        let max_attempts = options.max_attempts.max(1);
        let target = format!("{} ({})", device.name, device.ip);
        let job_id = db::get_connection().ok().and_then(|conn| {
            db::jobs::enqueue(
                &conn,
                "backup.running_config",
                &target,
                max_attempts as i64,
                options.details,
            )
            .ok()
        });

        let mut ok = false;
        let mut final_status = "Backup not attempted".to_string();

        for attempt in 1..=max_attempts {
            if let Some(job_id) = job_id
                && let Ok(conn) = db::get_connection()
            {
                let _ = db::jobs::mark_running(&conn, job_id);
            }

            let (attempt_ok, status_msg) = Self::run_backup(
                device.id,
                &device.ip,
                &device.user,
                &device.enc_cred,
                master_pass,
                options.timeout_secs,
                options.retention_keep,
            );

            final_status = format!("Attempt {attempt}/{max_attempts}: {status_msg}");
            if attempt_ok {
                ok = true;
                break;
            }

            if attempt < max_attempts {
                thread::sleep(Duration::from_secs(2));
            }
        }

        if let Some(job_id) = job_id
            && let Ok(conn) = db::get_connection()
        {
            if ok {
                let _ = db::jobs::mark_succeeded(&conn, job_id, &final_status);
            } else {
                let _ = db::jobs::mark_failed(&conn, job_id, &final_status, "");
            }
        }

        Self::append_log(logs, &device.name, &final_status);
        db::record_audit(
            "backup.running_config",
            &device.ip,
            if ok { "success" } else { "failed" },
            &final_status,
        );
        ctx.request_repaint();
    }

    fn run_backup_cycle(
        ctx: &egui::Context,
        logs: &Arc<Mutex<Vec<BackupLog>>>,
        master_pass: &str,
        options: &BackupRunOptions,
    ) {
        let devices = Self::fetch_devices();
        if devices.is_empty() {
            Self::append_log(logs, "Inventory", "No devices found for backup");
            return;
        }

        for device in devices {
            Self::execute_device_backup(device, master_pass, options, logs, ctx);
        }
    }

    fn recent_jobs() -> Vec<db::jobs::OperationJob> {
        let Ok(conn) = db::get_connection() else {
            return Vec::new();
        };

        db::jobs::recent_by_kind(&conn, "backup.running_config", 20).unwrap_or_default()
    }

    fn start_backup_loop(&mut self, ctx: egui::Context) {
        let is_running = self.is_running.clone();
        let logs = self.logs.clone();
        let m_pass = self.master_pass.clone();
        let interval = self.interval_hours as u64;
        let max_attempts = self.max_attempts;
        let timeout_secs = self.timeout_secs;
        let retention_keep = self.retention_keep;
        let options = BackupRunOptions {
            max_attempts,
            timeout_secs,
            retention_keep,
            details: "Scheduled running-config backup",
        };

        if let Ok(mut lock) = is_running.lock() {
            if *lock {
                return;
            }
            *lock = true;
        }

        thread::spawn(move || {
            loop {
                if let Ok(lock) = is_running.lock()
                    && !*lock
                {
                    break;
                }

                Self::run_backup_cycle(&ctx, &logs, &m_pass, &options);

                thread::sleep(Duration::from_secs(interval * 3600));
            }
        });
    }

    fn start_manual_backup(&mut self, ctx: egui::Context) {
        let is_manual_running = self.is_manual_running.clone();
        let logs = self.logs.clone();
        let master_pass = self.master_pass.clone();
        let max_attempts = self.max_attempts;
        let timeout_secs = self.timeout_secs;
        let retention_keep = self.retention_keep;
        let options = BackupRunOptions {
            max_attempts,
            timeout_secs,
            retention_keep,
            details: "Manual running-config backup",
        };

        if let Ok(mut lock) = is_manual_running.lock() {
            if *lock {
                return;
            }
            *lock = true;
        }

        thread::spawn(move || {
            Self::run_backup_cycle(&ctx, &logs, &master_pass, &options);
            if let Ok(mut lock) = is_manual_running.lock() {
                *lock = false;
            }
            ctx.request_repaint();
        });
    }

    fn stop_backup_loop(&mut self) {
        if let Ok(mut lock) = self.is_running.lock() {
            *lock = false;
        }
    }
}

impl ToolScreen for BackupTool {
    fn id(&self) -> &'static str {
        "auto_backup"
    }

    fn icon(&self) -> &'static str {
        "BAK"
    }

    fn name(&self, _dil: Language) -> &'static str {
        "Auto Backup"
    }

    fn draw(&mut self, ui: &mut egui::Ui, dil: Language) -> Option<ToolEvent> {
        ui.heading(text(dil, "Automatic Backup", "Otomatik Yedekleme"));
        ui.add_space(10.0);

        if !self.unlocked {
            ui.label(text(
                dil,
                "Enter the master password to decrypt device credentials:",
                "Cihaz sifrelerini cozmek icin master password girin:",
            ));
            ui.horizontal(|ui| {
                ui.add(egui::TextEdit::singleline(&mut self.master_pass).password(true));
                if ui.button(text(dil, "Unlock", "Kilidi ac")).clicked()
                    && !self.master_pass.is_empty()
                {
                    match db::verify_or_initialize_vault(&self.master_pass) {
                        Ok(()) => self.unlocked = true,
                        Err(e) => {
                            if let Ok(mut l) = self.logs.lock() {
                                l.push(BackupLog {
                                    time: chrono::Local::now()
                                        .format("%Y-%m-%d %H:%M:%S")
                                        .to_string(),
                                    device: "Vault".to_string(),
                                    status: e,
                                });
                            }
                        }
                    }
                }
            });
            return None;
        }

        ui.label(
            egui::RichText::new(text(
                dil,
                "Vault unlocked. Backup jobs can be started.",
                "Vault acik. Yedekleme gorevleri baslatilabilir.",
            ))
            .color(egui::Color32::GREEN),
        );
        ui.add_space(10.0);

        ui.horizontal(|ui| {
            ui.label(text(dil, "Interval (hours):", "Periyot (saat):"));
            ui.add(
                egui::DragValue::new(&mut self.interval_hours)
                    .speed(1)
                    .range(1..=24),
            );
        });

        ui.horizontal_wrapped(|ui| {
            ui.label(text(dil, "Max attempts:", "Maksimum deneme:"));
            ui.add(
                egui::DragValue::new(&mut self.max_attempts)
                    .speed(1)
                    .range(1..=5),
            );

            ui.label(text(dil, "SSH timeout (sec):", "SSH timeout (sn):"));
            ui.add(
                egui::DragValue::new(&mut self.timeout_secs)
                    .speed(1)
                    .range(3..=120),
            );

            ui.label(text(
                dil,
                "Keep snapshots/device:",
                "Cihaz basina snapshot:",
            ));
            ui.add(
                egui::DragValue::new(&mut self.retention_keep)
                    .speed(1)
                    .range(1..=500),
            );
        });

        ui.add_space(10.0);

        let running = *self.is_running.lock().unwrap();
        let manual_running = *self.is_manual_running.lock().unwrap();
        ui.horizontal(|ui| {
            if ui
                .add_enabled(
                    !manual_running,
                    egui::Button::new(text(dil, "Run backup now", "Simdi backup calistir")),
                )
                .clicked()
            {
                self.start_manual_backup(ui.ctx().clone());
            }

            if manual_running {
                ui.label(
                    egui::RichText::new(text(
                        dil,
                        "Manual backup is running...",
                        "Manuel backup calisiyor...",
                    ))
                    .color(egui::Color32::YELLOW),
                );
            }
        });

        if running {
            ui.label(
                egui::RichText::new(text(
                    dil,
                    "Backup service is running in the background...",
                    "Yedekleme servisi arka planda calisiyor...",
                ))
                .color(egui::Color32::YELLOW),
            );
            if ui.button(text(dil, "Stop", "Durdur")).clicked() {
                self.stop_backup_loop();
            }
        } else if ui.button(text(dil, "Start", "Baslat")).clicked() {
            self.start_backup_loop(ui.ctx().clone());
        }

        ui.add_space(20.0);
        ui.heading(text(dil, "Logs", "Loglar"));
        egui::ScrollArea::vertical().show(ui, |ui| {
            if let Ok(l) = self.logs.lock() {
                for log in l.iter().rev() {
                    ui.label(format!("[{}] {} - {}", log.time, log.device, log.status));
                }
            }
        });

        ui.add_space(18.0);
        ui.heading(text(dil, "Recent backup jobs", "Son backup joblari"));
        let jobs = Self::recent_jobs();
        if jobs.is_empty() {
            ui.label(text(
                dil,
                "No persisted backup jobs yet.",
                "Henuz kalici backup job kaydi yok.",
            ));
        } else {
            egui::ScrollArea::vertical()
                .max_height(220.0)
                .show(ui, |ui| {
                    egui::Grid::new("backup_jobs_grid")
                        .striped(true)
                        .spacing([14.0, 6.0])
                        .show(ui, |ui| {
                            ui.strong("ID");
                            ui.strong(text(dil, "Target", "Hedef"));
                            ui.strong(text(dil, "Status", "Durum"));
                            ui.strong(text(dil, "Attempts", "Deneme"));
                            ui.strong(text(dil, "Queued", "Kuyruk"));
                            ui.strong(text(dil, "Result", "Sonuc"));
                            ui.end_row();

                            for job in jobs {
                                ui.monospace(job.id.to_string());
                                ui.label(job.target);
                                ui.label(job.status);
                                ui.label(format!("{}/{}", job.attempts, job.max_attempts));
                                ui.label(job.queued_at);
                                ui.label(
                                    job.last_error
                                        .or(job.details)
                                        .unwrap_or_else(|| "-".to_string()),
                                );
                                ui.end_row();
                            }
                        });
                });
        }

        None
    }
}
