use crate::crypto;
use crate::db;
use crate::gui::tools::{ToolEvent, ToolScreen};
use crate::i18n::{Language, Message, t};
use eframe::egui;
use ssh;
use std::sync::{Arc, Mutex};
use std::thread;

#[derive(Clone)]
struct DeviceData {
    name: String,
    ip: String,
    user: String,
    enc_cred: String,
    selected: bool,
    status: Option<String>,
}

pub struct BulkDeployTool {
    master_pass: String,
    unlocked: bool,
    devices: Arc<Mutex<Vec<DeviceData>>>,
    commands: String,
    is_deploying: bool,
}

impl Default for BulkDeployTool {
    fn default() -> Self {
        Self {
            master_pass: String::new(),
            unlocked: false,
            devices: Arc::new(Mutex::new(Vec::new())),
            commands: String::new(),
            is_deploying: false,
        }
    }
}

impl BulkDeployTool {
    pub fn new() -> Self {
        Self::default()
    }

    fn fetch_devices(&mut self) {
        let mut list = Vec::new();
        if let Ok(conn) = db::get_connection()
            && let Ok(mut stmt) = conn.prepare("SELECT name, ip_address, username, encrypted_credentials FROM devices ORDER BY name") {
                let dev_iter = stmt.query_map([], |row| {
                    Ok(DeviceData {
                        name: row.get(0)?,
                        ip: row.get(1)?,
                        user: row.get::<_, Option<String>>(2)?.unwrap_or_default(),
                        enc_cred: row.get::<_, Option<String>>(3)?.unwrap_or_default(),
                        selected: false,
                        status: None,
                    })
                });
                if let Ok(iter) = dev_iter {
                    for dev in iter.flatten() {
                        list.push(dev);
                    }
                }
            }
        if let Ok(mut lock) = self.devices.lock() {
            *lock = list;
        }
    }

    fn deploy_commands(&mut self, ctx: egui::Context) {
        self.is_deploying = true;
        let cmds = self.commands.clone();
        let m_pass = self.master_pass.clone();

        let devices_arc = self.devices.clone();

        let devices_len = {
            let lock = devices_arc.lock().unwrap();
            lock.len()
        };

        for i in 0..devices_len {
            let (ip, user, enc_cred, selected) = {
                let lock = devices_arc.lock().unwrap();
                let dev = &lock[i];
                (
                    dev.ip.clone(),
                    dev.user.clone(),
                    dev.enc_cred.clone(),
                    dev.selected,
                )
            };

            if !selected {
                continue;
            }

            {
                let mut lock = devices_arc.lock().unwrap();
                lock[i].status = Some("Gönderiliyor...".to_string());
            }

            let cmds_thread = cmds.clone();
            let pass = m_pass.clone();
            let devs_clone = devices_arc.clone();
            let bg_ctx = ctx.clone();

            thread::spawn(move || {
                let result = match crypto::decrypt_credential(&enc_cred, &pass) {
                    Ok(plain_pass) => {
                        let addr = format!("{}:22", ip);
                        let session = ssh::create_session()
                            .username(&user)
                            .password(&plain_pass)
                            .connect(&addr);

                        match session {
                            Ok(sess) => {
                                let mut local_sess = sess.run_local();
                                match local_sess.open_exec() {
                                    Ok(exec) => match exec.send_command(&cmds_thread) {
                                        Ok(_) => "BAŞARILI".to_string(),
                                        Err(e) => format!("Cmd Err: {:?}", e),
                                    },
                                    Err(e) => format!("Exec Err: {:?}", e),
                                }
                            }
                            Err(_e) => "SSH Bağlantı Hatası".to_string(),
                        }
                    }
                    Err(_) => "Şifre Çözme Hatası".to_string(),
                };

                if let Ok(mut lock) = devs_clone.lock()
                    && let Some(d) = lock.get_mut(i)
                {
                    d.status = Some(result);
                }
                bg_ctx.request_repaint();
            });
        }
    }
}

impl ToolScreen for BulkDeployTool {
    fn id(&self) -> &'static str {
        "bulk_deploy"
    }

    fn icon(&self) -> &'static str {
        "🚀"
    }

    fn name(&self, dil: Language) -> &'static str {
        t(dil, Message::BulkDeploy)
    }

    fn draw(&mut self, ui: &mut egui::Ui, dil: Language) -> Option<ToolEvent> {
        ui.heading(t(dil, Message::BulkDeploy));
        ui.add_space(10.0);

        if !self.unlocked {
            ui.label(t(dil, Message::EnterMasterPassword));
            ui.horizontal(|ui| {
                ui.add(egui::TextEdit::singleline(&mut self.master_pass).password(true));
                if ui.button(t(dil, Message::Unlock)).clicked() && !self.master_pass.is_empty() {
                    self.unlocked = true;
                    self.fetch_devices();
                }
            });
            return None;
        }

        ui.label(t(dil, Message::DeployCommands));
        ui.add(egui::TextEdit::multiline(&mut self.commands).desired_rows(5));
        ui.add_space(10.0);

        if ui.button(t(dil, Message::DeployCommands)).clicked() {
            self.deploy_commands(ui.ctx().clone());
        }

        ui.add_space(10.0);
        ui.label(egui::RichText::new(t(dil, Message::SelectDevices)).strong());

        if let Ok(mut devs) = self.devices.lock() {
            egui::ScrollArea::vertical().show(ui, |ui| {
                egui::Grid::new("bulk_grid")
                    .striped(true)
                    .spacing([20.0, 8.0])
                    .show(ui, |ui| {
                        ui.label("Seç");
                        ui.label(t(dil, Message::DeviceName));
                        ui.label(t(dil, Message::IPAddress));
                        ui.label("Durum");
                        ui.end_row();

                        for dev in devs.iter_mut() {
                            ui.checkbox(&mut dev.selected, "");
                            ui.label(&dev.name);
                            ui.label(&dev.ip);
                            ui.label(dev.status.as_deref().unwrap_or("-"));
                            ui.end_row();
                        }
                    });
            });
        }

        None
    }
}
