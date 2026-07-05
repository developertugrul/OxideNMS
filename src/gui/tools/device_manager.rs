use crate::crypto;
use crate::db;
use crate::gui::tools::{ToolEvent, ToolScreen};
use crate::i18n::{Language, Message, t};
use eframe::egui;

#[derive(Default)]
pub struct DeviceManagerTool {
    master_pass: String,
    unlocked: bool,
    status_msg: String,
    form: DeviceForm,
}

#[derive(Default)]
struct DeviceForm {
    name: String,
    ip: String,
    user: String,
    pass: String,
    platform: String,
    model: String,
    serial: String,
    ios_version: String,
    site: String,
    role: String,
    tags: String,
}

struct DeviceListRow {
    id: i32,
    name: String,
    ip: String,
    user: String,
    platform: String,
    model: String,
    ios_version: String,
    site: String,
    role: String,
    tags: String,
}

impl DeviceManagerTool {
    pub fn new() -> Self {
        Self::default()
    }

    fn fetch_devices(&self) -> Vec<DeviceListRow> {
        let mut list = Vec::new();
        if let Ok(conn) = db::get_connection()
            && let Ok(mut stmt) = conn.prepare(
                "SELECT id, name, ip_address, username, platform, model, ios_version, site, role, tags
                 FROM devices ORDER BY id DESC",
            )
        {
            let dev_iter = stmt.query_map([], |row| {
                Ok(DeviceListRow {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    ip: row.get(2)?,
                    user: row.get::<_, Option<String>>(3)?.unwrap_or_default(),
                    platform: row.get::<_, Option<String>>(4)?.unwrap_or_default(),
                    model: row.get::<_, Option<String>>(5)?.unwrap_or_default(),
                    ios_version: row.get::<_, Option<String>>(6)?.unwrap_or_default(),
                    site: row.get::<_, Option<String>>(7)?.unwrap_or_default(),
                    role: row.get::<_, Option<String>>(8)?.unwrap_or_default(),
                    tags: row.get::<_, Option<String>>(9)?.unwrap_or_default(),
                })
            });
            if let Ok(iter) = dev_iter {
                list.extend(iter.flatten());
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
                db::record_audit(
                    "device.delete",
                    &id.to_string(),
                    "success",
                    "Device removed",
                );
                self.status_msg = "Cihaz silindi.".to_string();
            }
        }
    }

    fn save_device(&mut self, dil: Language) {
        if self.form.name.is_empty() || self.form.ip.is_empty() {
            self.status_msg = "Cihaz adi ve IP zorunludur.".to_string();
            return;
        }

        let enc_pass = match crypto::encrypt_credential(&self.form.pass, &self.master_pass) {
            Ok(value) => value,
            Err(e) => {
                self.status_msg = e;
                return;
            }
        };

        if let Ok(conn) = db::get_connection() {
            let result = conn.execute(
                "INSERT INTO devices (
                    name, ip_address, username, encrypted_credentials,
                    platform, model, serial, ios_version, site, role, tags
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
                [
                    &self.form.name,
                    &self.form.ip,
                    &self.form.user,
                    &enc_pass,
                    &self.form.platform,
                    &self.form.model,
                    &self.form.serial,
                    &self.form.ios_version,
                    &self.form.site,
                    &self.form.role,
                    &self.form.tags,
                ],
            );

            if result.is_ok() {
                db::record_audit("device.create", &self.form.ip, "success", &self.form.name);
                self.status_msg = t(dil, Message::DeviceSaved).to_string();
                self.form = DeviceForm::default();
            } else if let Err(e) = result {
                self.status_msg = format!("Kayit hatasi: {e}");
            }
        }
    }
}

impl ToolScreen for DeviceManagerTool {
    fn id(&self) -> &'static str {
        "device_manager"
    }

    fn icon(&self) -> &'static str {
        "🔐"
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
                if ui.button(t(dil, Message::Unlock)).clicked() && !self.master_pass.is_empty() {
                    match db::verify_or_initialize_vault(&self.master_pass) {
                        Ok(()) => {
                            self.unlocked = true;
                            self.status_msg = "Vault dogrulandi.".to_string();
                        }
                        Err(e) => self.status_msg = e,
                    }
                }
            });
            if !self.status_msg.is_empty() {
                ui.colored_label(egui::Color32::YELLOW, &self.status_msg);
            }
            return None;
        }

        ui.label(egui::RichText::new(t(dil, Message::Unlocked)).color(egui::Color32::GREEN));
        ui.add_space(12.0);

        ui.group(|ui| {
            ui.label(egui::RichText::new(t(dil, Message::AddDevice)).strong());
            egui::Grid::new("device_form")
                .num_columns(4)
                .spacing([12.0, 8.0])
                .show(ui, |ui| {
                    ui.label(t(dil, Message::DeviceName));
                    ui.text_edit_singleline(&mut self.form.name);
                    ui.label(t(dil, Message::IPAddress));
                    ui.text_edit_singleline(&mut self.form.ip);
                    ui.end_row();

                    ui.label(t(dil, Message::Username));
                    ui.text_edit_singleline(&mut self.form.user);
                    ui.label(t(dil, Message::Password));
                    ui.add(egui::TextEdit::singleline(&mut self.form.pass).password(true));
                    ui.end_row();

                    ui.label("Platform");
                    ui.text_edit_singleline(&mut self.form.platform);
                    ui.label("Model");
                    ui.text_edit_singleline(&mut self.form.model);
                    ui.end_row();

                    ui.label("Serial");
                    ui.text_edit_singleline(&mut self.form.serial);
                    ui.label("IOS version");
                    ui.text_edit_singleline(&mut self.form.ios_version);
                    ui.end_row();

                    ui.label("Site");
                    ui.text_edit_singleline(&mut self.form.site);
                    ui.label("Role");
                    ui.text_edit_singleline(&mut self.form.role);
                    ui.end_row();

                    ui.label("Tags");
                    ui.text_edit_singleline(&mut self.form.tags);
                    ui.end_row();
                });

            if ui.button(t(dil, Message::SaveDevice)).clicked() {
                self.save_device(dil);
            }
        });

        if !self.status_msg.is_empty() {
            ui.add_space(8.0);
            ui.label(
                egui::RichText::new(&self.status_msg).color(egui::Color32::from_rgb(100, 200, 100)),
            );
        }

        ui.add_space(14.0);
        let devices = self.fetch_devices();
        egui::ScrollArea::both().show(ui, |ui| {
            egui::Grid::new("device_grid")
                .striped(true)
                .spacing([16.0, 8.0])
                .show(ui, |ui| {
                    for header in [
                        "Name", "IP", "User", "Platform", "Model", "IOS", "Site", "Role", "Tags",
                        "",
                    ] {
                        ui.label(egui::RichText::new(header).strong());
                    }
                    ui.end_row();

                    let mut delete_id = None;
                    for device in devices {
                        ui.label(device.name);
                        ui.label(device.ip);
                        ui.label(device.user);
                        ui.label(device.platform);
                        ui.label(device.model);
                        ui.label(device.ios_version);
                        ui.label(device.site);
                        ui.label(device.role);
                        ui.label(device.tags);
                        if ui.button(t(dil, Message::DeleteDevice)).clicked() {
                            delete_id = Some(device.id);
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
