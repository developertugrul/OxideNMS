use eframe::egui;

use super::{ToolEvent, ToolScreen};
use crate::db;
use crate::i18n::Language;

#[derive(Clone, Default)]
struct DeviceSummary {
    id: i64,
    name: String,
    ip_address: String,
    backup_count: i64,
    last_backup: String,
}

#[derive(Clone, Default)]
struct ConfigSnapshot {
    id: i64,
    recorded_at: String,
    bytes: usize,
    config_text: String,
}

pub struct ConfigHistoryTool {
    devices: Vec<DeviceSummary>,
    snapshots: Vec<ConfigSnapshot>,
    selected_device: Option<i64>,
    status: String,
}

impl Default for ConfigHistoryTool {
    fn default() -> Self {
        let mut tool = Self {
            devices: Vec::new(),
            snapshots: Vec::new(),
            selected_device: None,
            status: String::new(),
        };
        tool.refresh_devices();
        tool
    }
}

impl ConfigHistoryTool {
    fn refresh_devices(&mut self) {
        self.devices.clear();

        let Ok(conn) = db::get_connection() else {
            self.status = "Database connection failed".to_string();
            return;
        };

        let sql = "
            SELECT
                d.id,
                d.name,
                d.ip_address,
                COUNT(c.id) AS backup_count,
                COALESCE(MAX(c.recorded_at), '') AS last_backup
            FROM devices d
            LEFT JOIN config_gecmisi c ON c.device_id = d.id
            GROUP BY d.id, d.name, d.ip_address
            ORDER BY last_backup DESC, d.name ASC
        ";

        let Ok(mut stmt) = conn.prepare(sql) else {
            self.status = "Device inventory query failed".to_string();
            return;
        };

        let rows = stmt.query_map([], |row| {
            Ok(DeviceSummary {
                id: row.get(0)?,
                name: row.get(1)?,
                ip_address: row.get(2)?,
                backup_count: row.get(3)?,
                last_backup: row.get(4)?,
            })
        });

        match rows {
            Ok(iter) => {
                self.devices = iter.flatten().collect();
                if self.selected_device.is_none() {
                    self.selected_device = self.devices.first().map(|device| device.id);
                }
                self.refresh_snapshots();
                self.status = format!("{} devices loaded", self.devices.len());
            }
            Err(e) => self.status = format!("Device inventory read failed: {e}"),
        }
    }

    fn refresh_snapshots(&mut self) {
        self.snapshots.clear();

        let Some(device_id) = self.selected_device else {
            return;
        };

        let Ok(conn) = db::get_connection() else {
            self.status = "Database connection failed".to_string();
            return;
        };

        let Ok(mut stmt) = conn.prepare(
            "
            SELECT id, recorded_at, config_text
            FROM config_gecmisi
            WHERE device_id = ?1
            ORDER BY id DESC
            ",
        ) else {
            self.status = "Backup history query failed".to_string();
            return;
        };

        let rows = stmt.query_map([device_id], |row| {
            let config_text: String = row.get(2)?;
            Ok(ConfigSnapshot {
                id: row.get(0)?,
                recorded_at: row.get(1)?,
                bytes: config_text.len(),
                config_text,
            })
        });

        match rows {
            Ok(iter) => {
                self.snapshots = iter.flatten().collect();
                self.status = format!("{} snapshots loaded", self.snapshots.len());
            }
            Err(e) => self.status = format!("Backup history read failed: {e}"),
        }
    }

    fn selected_device_name(&self) -> String {
        self.selected_device
            .and_then(|id| self.devices.iter().find(|device| device.id == id))
            .map(|device| format!("{} ({})", device.name, device.ip_address))
            .unwrap_or_else(|| "No device selected".to_string())
    }
}

impl ToolScreen for ConfigHistoryTool {
    fn id(&self) -> &'static str {
        "config_history"
    }

    fn icon(&self) -> &'static str {
        "CFG"
    }

    fn name(&self, _dil: Language) -> &'static str {
        "Config History"
    }

    fn draw(&mut self, ui: &mut egui::Ui, _dil: Language) -> Option<ToolEvent> {
        ui.heading("Configuration Lifecycle");
        ui.label("Review saved running-config snapshots and compare recent changes.");
        ui.add_space(10.0);

        let mut event = None;
        ui.horizontal(|ui| {
            if ui.button("Refresh").clicked() {
                self.refresh_devices();
            }

            let can_compare = self.snapshots.len() >= 2;
            if ui
                .add_enabled(can_compare, egui::Button::new("Compare latest two"))
                .clicked()
            {
                let newest = self.snapshots[0].clone();
                let previous = self.snapshots[1].clone();
                db::record_audit(
                    "config.compare",
                    &self.selected_device_name(),
                    "started",
                    "Latest two snapshots sent to diff",
                );
                event = Some(ToolEvent::SwitchToDiff {
                    old_config: previous.config_text,
                    new_config: newest.config_text,
                });
            }

            ui.label(egui::RichText::new(&self.status).weak());
        });
        if event.is_some() {
            return event;
        }

        ui.add_space(8.0);
        ui.columns(2, |columns| {
            columns[0].vertical(|ui| {
                ui.label(egui::RichText::new("Devices").strong());
                ui.separator();

                egui::ScrollArea::vertical()
                    .id_salt("config_history_devices")
                    .max_height(420.0)
                    .show(ui, |ui| {
                        for device in self.devices.clone() {
                            let selected = Some(device.id) == self.selected_device;
                            let last_backup = if device.last_backup.is_empty() {
                                "never".to_string()
                            } else {
                                device.last_backup.clone()
                            };
                            let label = format!(
                                "{}\n{} | backups: {} | last: {}",
                                device.name, device.ip_address, device.backup_count, last_backup
                            );

                            if ui.selectable_label(selected, label).clicked() {
                                self.selected_device = Some(device.id);
                                self.refresh_snapshots();
                            }
                            ui.add_space(4.0);
                        }
                    });
            });

            columns[1].vertical(|ui| {
                ui.label(egui::RichText::new(self.selected_device_name()).strong());
                ui.separator();

                egui::Grid::new("config_history_snapshots")
                    .striped(true)
                    .num_columns(4)
                    .spacing([12.0, 6.0])
                    .show(ui, |ui| {
                        ui.strong("ID");
                        ui.strong("Recorded");
                        ui.strong("Size");
                        ui.strong("Action");
                        ui.end_row();

                        for snapshot in &self.snapshots {
                            ui.monospace(snapshot.id.to_string());
                            ui.label(&snapshot.recorded_at);
                            ui.label(format!("{} bytes", snapshot.bytes));
                            if ui.button("Copy").clicked() {
                                ui.ctx().copy_text(snapshot.config_text.clone());
                                self.status = format!("Snapshot {} copied", snapshot.id);
                            }
                            ui.end_row();
                        }
                    });

                if self.snapshots.is_empty() {
                    ui.add_space(12.0);
                    ui.label(egui::RichText::new("No saved configuration snapshots.").weak());
                }
            });
        });

        None
    }
}
