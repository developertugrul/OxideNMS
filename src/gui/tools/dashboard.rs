use crate::db;
use crate::gui::tools::{ToolEvent, ToolScreen};
use crate::i18n::{Language, text};
use crate::network::security::{self, Level};
use eframe::egui;

#[derive(Default)]
pub struct DashboardTool;

#[derive(Default)]
struct DashboardStats {
    devices: i64,
    config_snapshots: i64,
    devices_with_backup: i64,
    audit_events: i64,
    critical_findings: usize,
    warning_findings: usize,
    last_backup: Option<String>,
    db_path: String,
}

impl DashboardTool {
    pub fn new() -> Self {
        Self
    }

    fn load_stats() -> DashboardStats {
        let mut stats = DashboardStats {
            db_path: db::database_path().display().to_string(),
            ..Default::default()
        };

        let Ok(conn) = db::get_connection() else {
            return stats;
        };

        stats.devices = conn
            .query_row("SELECT COUNT(*) FROM devices", [], |row| row.get(0))
            .unwrap_or(0);
        stats.config_snapshots = conn
            .query_row("SELECT COUNT(*) FROM config_gecmisi", [], |row| row.get(0))
            .unwrap_or(0);
        stats.devices_with_backup = conn
            .query_row(
                "SELECT COUNT(DISTINCT device_id) FROM config_gecmisi",
                [],
                |row| row.get(0),
            )
            .unwrap_or(0);
        stats.audit_events = conn
            .query_row("SELECT COUNT(*) FROM audit_log", [], |row| row.get(0))
            .unwrap_or(0);
        stats.last_backup = conn
            .query_row("SELECT MAX(recorded_at) FROM config_gecmisi", [], |row| {
                row.get::<_, Option<String>>(0)
            })
            .unwrap_or(None);

        if let Ok(mut stmt) = conn.prepare(
            "SELECT config_text FROM config_gecmisi
             WHERE id IN (SELECT MAX(id) FROM config_gecmisi GROUP BY device_id)",
        ) && let Ok(rows) = stmt.query_map([], |row| row.get::<_, String>(0))
        {
            for config in rows.flatten() {
                for finding in security::audit(&config) {
                    match finding.level {
                        Level::Critical => stats.critical_findings += 1,
                        Level::Warning => stats.warning_findings += 1,
                        Level::Info => {}
                    }
                }
            }
        }

        stats
    }

    fn metric(ui: &mut egui::Ui, label: &str, value: impl ToString, note: &str) {
        ui.group(|ui| {
            ui.set_min_width(170.0);
            ui.label(egui::RichText::new(label).strong());
            ui.label(egui::RichText::new(value.to_string()).size(24.0).strong());
            ui.label(egui::RichText::new(note).small().weak());
        });
    }
}

impl ToolScreen for DashboardTool {
    fn id(&self) -> &'static str {
        "dashboard"
    }

    fn icon(&self) -> &'static str {
        "DASH"
    }

    fn name(&self, _dil: Language) -> &'static str {
        "Dashboard"
    }

    fn draw(&mut self, ui: &mut egui::Ui, dil: Language) -> Option<ToolEvent> {
        let stats = Self::load_stats();

        ui.heading(text(
            dil,
            "OxideNMS Operations Dashboard",
            "OxideNMS Operasyon Dashboard",
        ));
        ui.label(text(
            dil,
            "Device inventory, configuration backups, audit records, and security findings.",
            "Cihaz envanteri, konfigurasyon yedekleri, audit kayitlari ve guvenlik bulgulari.",
        ));
        ui.add_space(12.0);

        ui.horizontal_wrapped(|ui| {
            Self::metric(
                ui,
                text(dil, "Devices", "Cihaz"),
                stats.devices,
                text(dil, "Total inventory records", "Envanterdeki toplam kayit"),
            );
            Self::metric(
                ui,
                text(dil, "Devices backed up", "Backup alan cihaz"),
                stats.devices_with_backup,
                text(
                    dil,
                    "Devices with config history",
                    "Config gecmisi olan cihaz",
                ),
            );
            Self::metric(
                ui,
                text(dil, "Config snapshots", "Config snapshot"),
                stats.config_snapshots,
                text(dil, "Total backup versions", "Toplam yedek surumu"),
            );
            Self::metric(
                ui,
                text(dil, "Audit events", "Audit event"),
                stats.audit_events,
                text(dil, "Operational trail", "Operasyon izi"),
            );
        });

        ui.add_space(12.0);
        ui.horizontal_wrapped(|ui| {
            Self::metric(
                ui,
                text(dil, "Critical findings", "Kritik finding"),
                stats.critical_findings,
                text(dil, "In latest config versions", "Son config surumlerinde"),
            );
            Self::metric(
                ui,
                text(dil, "Warning findings", "Uyari finding"),
                stats.warning_findings,
                text(dil, "In latest config versions", "Son config surumlerinde"),
            );
            Self::metric(
                ui,
                text(dil, "Last backup", "Son backup"),
                stats.last_backup.as_deref().unwrap_or("-"),
                "config_gecmisi MAX(recorded_at)",
            );
        });

        ui.add_space(16.0);
        ui.separator();
        ui.add_space(10.0);
        ui.label(egui::RichText::new(text(dil, "Operational status", "Operasyon durumu")).strong());
        ui.label(format!(
            "{}: {}",
            text(dil, "Database", "Veritabani"),
            stats.db_path
        ));

        if stats.devices == 0 {
            ui.colored_label(
                egui::Color32::YELLOW,
                text(
                    dil,
                    "No devices yet. Start by adding devices in Device Manager.",
                    "Henuz cihaz yok. Device Manager uzerinden cihaz ekleyerek baslayin.",
                ),
            );
        } else if stats.devices_with_backup < stats.devices {
            ui.colored_label(
                egui::Color32::YELLOW,
                text(
                    dil,
                    "Some devices do not have a configuration backup yet.",
                    "Bazi cihazlarin henuz konfigurasyon yedegi yok.",
                ),
            );
        } else {
            ui.colored_label(
                egui::Color32::GREEN,
                text(
                    dil,
                    "Configuration history exists for all inventory devices.",
                    "Envanterdeki cihazlar icin konfigurasyon gecmisi mevcut.",
                ),
            );
        }

        None
    }
}
