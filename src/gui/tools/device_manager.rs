use crate::crypto;
use crate::db;
use crate::gui::tools::{ToolEvent, ToolScreen};
use crate::i18n::{Language, Message, t};
use eframe::egui;

#[derive(Default)]
pub struct DeviceManagerTool {
    master_pass: String,
    unlocked: bool,

    // Add form
    new_name: String,
    new_ip: String,
    new_user: String,
    new_pass: String,

    status_msg: String,
}

impl DeviceManagerTool {
    pub fn new() -> Self {
        Self::default()
    }

    fn fetch_devices(&self) -> Vec<(i32, String, String, String)> {
        let mut list = Vec::new();
        if let Ok(conn) = db::get_connection() {
            if let Ok(mut stmt) =
                conn.prepare("SELECT id, name, ip_address, username FROM devices ORDER BY id DESC")
            {
                let dev_iter = stmt.query_map([], |row| {
                    Ok((
                        row.get(0)?,
                        row.get(1)?,
                        row.get(2)?,
                        row.get::<_, Option<String>>(3)?.unwrap_or_default(),
                    ))
                });
                if let Ok(iter) = dev_iter {
                    for dev in iter.flatten() {
                        list.push(dev);
                    }
                }
            }
        }
        list
    }

    fn delete_device(&mut self, id: i32) {
        if let Ok(conn) = db::get_connection() {
            let _ = conn.execute("DELETE FROM config_gecmisi WHERE device_id = ?1", [id]);
            if conn
                .execute("DELETE FROM devices WHERE id = ?1", [id])
                .is_ok()
            {
                self.status_msg = "Cihaz silindi.".to_string();
            }
        }
    }
}

impl ToolScreen for DeviceManagerTool {
    fn id(&self) -> &'static str {
        "device_manager"
    }

    fn icon(&self) -> &'static str {
        "🔒"
    }

    fn name(&self, dil: Language) -> &'static str {
        t(dil, Message::DeviceManager)
    }

    fn draw(&mut self, ui: &mut egui::Ui, dil: Language) -> Option<ToolEvent> {
        ui.heading(t(dil, Message::DeviceManager));
        ui.add_space(10.0);

        if !self.unlocked {
            ui.label(t(dil, Message::EnterMasterPassword));
            ui.horizontal(|ui| {
                ui.add(egui::TextEdit::singleline(&mut self.master_pass).password(true));
                if ui.button(t(dil, Message::Unlock)).clicked() {
                    if !self.master_pass.is_empty() {
                        self.unlocked = true;
                    }
                }
            });
            return None;
        }

        ui.label(egui::RichText::new(t(dil, Message::Unlocked)).color(egui::Color32::GREEN));
        ui.add_space(20.0);

        // Add Device Form
        ui.group(|ui| {
            ui.label(egui::RichText::new(t(dil, Message::AddDevice)).strong());
            ui.horizontal(|ui| {
                ui.label(t(dil, Message::DeviceName));
                ui.text_edit_singleline(&mut self.new_name);
            });
            ui.horizontal(|ui| {
                ui.label(t(dil, Message::IPAddress));
                ui.text_edit_singleline(&mut self.new_ip);
            });
            ui.horizontal(|ui| {
                ui.label(t(dil, Message::Username));
                ui.text_edit_singleline(&mut self.new_user);
            });
            ui.horizontal(|ui| {
                ui.label(t(dil, Message::Password));
                ui.add(egui::TextEdit::singleline(&mut self.new_pass).password(true));
            });

            if ui.button(t(dil, Message::SaveDevice)).clicked() {
                if !self.new_name.is_empty() && !self.new_ip.is_empty() {
                    let enc_pass = crypto::encrypt_credential(&self.new_pass, &self.master_pass)
                        .unwrap_or_default();

                    if let Ok(conn) = db::get_connection() {
                        let _ = conn.execute(
                            "INSERT INTO devices (name, ip_address, username, encrypted_credentials) VALUES (?1, ?2, ?3, ?4)",
                            [&self.new_name, &self.new_ip, &self.new_user, &enc_pass],
                        );
                        self.status_msg = t(dil, Message::DeviceSaved).to_string();
                        self.new_name.clear();
                        self.new_ip.clear();
                        self.new_user.clear();
                        self.new_pass.clear();
                    }
                }
            }
        });

        if !self.status_msg.is_empty() {
            ui.add_space(8.0);
            ui.label(
                egui::RichText::new(&self.status_msg).color(egui::Color32::from_rgb(100, 200, 100)),
            );
        }

        ui.add_space(20.0);

        // List Devices
        let devices = self.fetch_devices();
        egui::ScrollArea::vertical().show(ui, |ui| {
            egui::Grid::new("device_grid")
                .striped(true)
                .spacing([20.0, 8.0])
                .show(ui, |ui| {
                    ui.label(egui::RichText::new(t(dil, Message::DeviceName)).strong());
                    ui.label(egui::RichText::new(t(dil, Message::IPAddress)).strong());
                    ui.label(egui::RichText::new(t(dil, Message::Username)).strong());
                    ui.label("");
                    ui.end_row();

                    let mut delete_id = None;
                    for (id, name, ip, user) in devices {
                        ui.label(name);
                        ui.label(ip);
                        ui.label(user);
                        if ui.button(t(dil, Message::DeleteDevice)).clicked() {
                            delete_id = Some(id);
                        }
                        ui.end_row();
                    }

                    if let Some(id) = delete_id {
                        self.delete_device(id);
                    }
                });
        });

        None
    }
}
