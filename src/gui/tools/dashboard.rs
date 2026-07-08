use crate::db;
use crate::gui::tools::{ToolEvent, ToolScreen};
use crate::i18n::{Language, text};
use crate::network::compliance;
use eframe::egui;

#[derive(Default)]
pub struct DashboardTool;

struct DashboardStats {
    devices: i64,
    config_snapshots: i64,
    devices_with_backup: i64,
    audit_events: i64,
    critical_findings: usize,
    warning_findings: usize,
    info_findings: usize,
    /// Filo geneli güvenlik duruşu skoru (0-100).
    posture_score: u8,
    posture_grade: &'static str,
    last_backup: Option<String>,
    db_path: String,
}

impl Default for DashboardStats {
    fn default() -> Self {
        Self {
            devices: 0,
            config_snapshots: 0,
            devices_with_backup: 0,
            audit_events: 0,
            critical_findings: 0,
            warning_findings: 0,
            info_findings: 0,
            posture_score: 100,
            posture_grade: "A",
            last_backup: None,
            db_path: String::new(),
        }
    }
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

        // Her cihazın SON config'ini (adıyla) al ve filo posture'unu hesapla.
        // Denetim/skorlama tek kaynaktan: network::compliance.
        let mut configs: Vec<(String, String)> = Vec::new();
        if let Ok(mut stmt) = conn.prepare(
            "SELECT d.name, c.config_text
             FROM config_gecmisi c
             JOIN devices d ON d.id = c.device_id
             WHERE c.id IN (SELECT MAX(id) FROM config_gecmisi GROUP BY device_id)",
        ) && let Ok(rows) =
            stmt.query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)))
        {
            configs.extend(rows.flatten());
        }

        let fleet = compliance::fleet_posture(&configs);
        stats.posture_score = fleet.average_score;
        stats.posture_grade = fleet.grade();
        stats.critical_findings = fleet.total_critical;
        stats.warning_findings = fleet.total_warning;
        stats.info_findings = fleet.total_info;

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

    /// Nota göre renk (A/B yeşil, C sarı, D turuncu, F kırmızı).
    fn grade_color(grade: &str) -> egui::Color32 {
        match grade {
            "A" | "B" => egui::Color32::from_rgb(90, 190, 90),
            "C" => egui::Color32::from_rgb(210, 190, 70),
            "D" => egui::Color32::from_rgb(220, 150, 60),
            _ => egui::Color32::from_rgb(220, 80, 80),
        }
    }

    /// En üstte, filo geneli güvenlik duruşunu öne çıkaran başlık kartı.
    fn posture_card(ui: &mut egui::Ui, dil: Language, stats: &DashboardStats) {
        let renk = Self::grade_color(stats.posture_grade);
        egui::Frame::group(ui.style()).show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.label(
                        egui::RichText::new(text(dil, "Fleet security posture", "Filo güvenlik duruşu"))
                            .strong(),
                    );
                    ui.label(
                        egui::RichText::new(format!("{}/100  ({})", stats.posture_score, stats.posture_grade))
                            .size(34.0)
                            .strong()
                            .color(renk),
                    );
                    ui.label(
                        egui::RichText::new(text(
                            dil,
                            "Average across latest device configurations",
                            "Cihazların son config'lerinin ortalaması",
                        ))
                        .small()
                        .weak(),
                    );
                });
                ui.add_space(24.0);
                ui.separator();
                ui.add_space(24.0);
                ui.vertical(|ui| {
                    ui.label(
                        egui::RichText::new(format!("● {} {}", stats.critical_findings, text(dil, "Critical", "Kritik")))
                            .color(egui::Color32::from_rgb(220, 80, 80))
                            .strong(),
                    );
                    ui.label(
                        egui::RichText::new(format!("● {} {}", stats.warning_findings, text(dil, "Warning", "Uyarı")))
                            .color(egui::Color32::from_rgb(220, 150, 60)),
                    );
                    ui.label(
                        egui::RichText::new(format!("● {} {}", stats.info_findings, text(dil, "Info", "Bilgi")))
                            .color(egui::Color32::from_rgb(150, 150, 150)),
                    );
                });
            });
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

        // Başlık: filo geneli güvenlik duruşu (profesyonel operatör görünümü).
        Self::posture_card(ui, dil, &stats);
        ui.add_space(14.0);

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
                text(dil, "Last backup", "Son backup"),
                stats.last_backup.as_deref().unwrap_or("-"),
                text(dil, "Most recent config snapshot", "En yeni config snapshot"),
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
