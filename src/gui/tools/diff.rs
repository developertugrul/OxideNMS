use eframe::egui;

use super::{ToolScreen, ToolEvent};
use crate::i18n::{Language, Message, t};
use crate::network::diff::{self, DiffRow, DiffType};
use crate::db;

pub struct DiffTool {
    old_config: String,
    new_config: String,
    result: Option<Vec<DiffRow>>,
    db_mesaj: Option<String>,
    device_name: String,
}

impl Default for DiffTool {
    fn default() -> Self {
        Self {
            old_config: "hostname R1\n!\ninterface GigabitEthernet0/0\n ip address 10.0.0.1 255.255.255.0\n no shutdown\n!".to_owned(),
            new_config: "hostname R1\n!\ninterface GigabitEthernet0/0\n ip address 10.0.0.2 255.255.255.0\n no shutdown\n!\ninterface GigabitEthernet0/1\n no shutdown\n!".to_owned(),
            result: None,
            db_mesaj: None,
            device_name: "Router-1".to_owned(),
        }
    }
}

impl ToolScreen for DiffTool {
    fn id(&self) -> &'static str { "diff" }
    
    fn icon(&self) -> &'static str { "🔄" }
    
    fn name(&self, dil: Language) -> &'static str {
        t(dil, Message::DiffName)
    }

    fn draw(&mut self, ui: &mut egui::Ui, dil: Language) -> Option<ToolEvent> {
        ui.add_space(6.0);
        ui.heading(t(dil, Message::DiffName));
        ui.label(t(dil, Message::DiffDescription));
        ui.add_space(10.0);

        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.label(t(dil, Message::DiffOldConfig));
                ui.add(
                    egui::TextEdit::multiline(&mut self.old_config)
                        .desired_rows(10)
                        .desired_width(ui.available_width() / 2.0 - 5.0)
                        .code_editor(),
                );
            });
            ui.vertical(|ui| {
                ui.label(t(dil, Message::DiffNewConfig));
                ui.add(
                    egui::TextEdit::multiline(&mut self.new_config)
                        .desired_rows(10)
                        .desired_width(ui.available_width())
                        .code_editor(),
                );
            });
        });

        ui.add_space(8.0);
        ui.horizontal(|ui| {
            if ui.button(t(dil, Message::DiffCompare)).clicked() {
                self.result = Some(diff::compare_configs(&self.old_config, &self.new_config));
            }

            ui.add_space(20.0);
            
            ui.label("Device Adı:");
            ui.text_edit_singleline(&mut self.device_name);
            if ui.button(t(dil, Message::DiffSaveToDb)).clicked() {
                match db::get_connection() {
                    Ok(conn) => {
                        let device_id = match db::devices::get_or_create_device(&conn, &self.device_name, "Bilinmiyor") {
                            Ok(id) => id,
                            Err(_) => 1,
                        };
                        match db::devices::save_config(&conn, device_id, &self.new_config) {
                            Ok(_) => self.db_mesaj = Some("Veritabanına Saved!".to_owned()),
                            Err(e) => self.db_mesaj = Some(format!("Kayıt Hatası: {}", e)),
                        }
                    },
                    Err(e) => self.db_mesaj = Some(format!("Bağlantı Hatası: {}", e)),
                }
            }

            if let Some(msg) = &self.db_mesaj {
                ui.label(egui::RichText::new(msg).color(egui::Color32::YELLOW));
            }
        });

        ui.add_space(10.0);
        ui.separator();
        ui.add_space(10.0);

        if let Some(farklar) = &self.result {
            egui::ScrollArea::vertical().show(ui, |ui| {
                for line in farklar {
                    let (renk, on_ek) = match line.tip {
                        DiffType::Inserted => (egui::Color32::from_rgb(100, 255, 100), "+ "),
                        DiffType::Deleted => (egui::Color32::from_rgb(255, 100, 100), "- "),
                        DiffType::Unchanged => (egui::Color32::from_rgb(150, 150, 150), "  "),
                    };

                    let text = format!("{}{}", on_ek, line.text.trim_end_matches('\n'));
                    ui.label(egui::RichText::new(text).color(renk).monospace());
                }
            });
        }

        None
    }

    fn receive_data(&mut self, old: String, new: String) {
        self.old_config = old;
        self.new_config = new;
        self.result = Some(diff::compare_configs(&self.old_config, &self.new_config));
        self.db_mesaj = None;
    }
}


