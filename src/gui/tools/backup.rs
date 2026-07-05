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

pub struct BackupTool {
    master_pass: String,
    unlocked: bool,
    interval_hours: i32,
    is_running: Arc<Mutex<bool>>,
    logs: Arc<Mutex<Vec<BackupLog>>>,
}

impl Default for BackupTool {
    fn default() -> Self {
        Self {
            master_pass: String::new(),
            unlocked: false,
            interval_hours: 2,
            is_running: Arc::new(Mutex::new(false)),
            logs: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

impl BackupTool {
    pub fn new() -> Self {
        Self::default()
    }

    fn start_backup_loop(&mut self, ctx: egui::Context) {
        let is_running = self.is_running.clone();
        let logs = self.logs.clone();
        let m_pass = self.master_pass.clone();
        let interval = self.interval_hours as u64;

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

                let mut devices = Vec::new();
                if let Ok(conn) = db::get_connection()
                    && let Ok(mut stmt) = conn.prepare(
                        "SELECT id, name, ip_address, username, encrypted_credentials FROM devices",
                    )
                    && let Ok(iter) = stmt.query_map([], |row| {
                        Ok((
                            row.get::<_, i32>(0)?,
                            row.get::<_, String>(1)?,
                            row.get::<_, String>(2)?,
                            row.get::<_, Option<String>>(3)?.unwrap_or_default(),
                            row.get::<_, Option<String>>(4)?.unwrap_or_default(),
                        ))
                    })
                {
                    for dev in iter.flatten() {
                        devices.push(dev);
                    }
                }

                for (id, name, ip, user, enc_cred) in devices {
                    let status_msg =
                        if let Ok(plain_pass) = crypto::decrypt_credential(&enc_cred, &m_pass) {
                            let addr = format!("{}:22", ip);
                            let session = ssh::create_session()
                                .username(&user)
                                .password(&plain_pass)
                                .connect(&addr);

                            match session {
                                Ok(sess) => {
                                    let mut local_sess = sess.run_local();
                                    match local_sess.open_exec() {
                                        Ok(exec) => {
                                            let res: Result<Vec<u8>, _> =
                                                exec.send_command("show running-config");
                                            match res {
                                                Ok(vec) => {
                                                    let config =
                                                        String::from_utf8_lossy(&vec).into_owned();
                                                    if let Ok(conn) = db::get_connection() {
                                                        let _ = db::devices::save_config(
                                                            &conn, id as i64, &config,
                                                        );
                                                    }
                                                    "Backup successful".to_string()
                                                }
                                                Err(_) => "Command failed".to_string(),
                                            }
                                        }
                                        Err(_) => "Exec channel failed".to_string(),
                                    }
                                }
                                Err(_) => "SSH connection failed".to_string(),
                            }
                        } else {
                            "Password decrypt failed".to_string()
                        };

                    if let Ok(mut l) = logs.lock() {
                        let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
                        l.push(BackupLog {
                            time: now,
                            device: name.clone(),
                            status: status_msg.clone(),
                        });
                    }
                    db::record_audit("backup.running_config", &ip, "finished", &status_msg);
                    ctx.request_repaint();
                }

                thread::sleep(Duration::from_secs(interval * 3600));
            }
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

        ui.add_space(10.0);

        let running = *self.is_running.lock().unwrap();
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

        None
    }
}
