//! Filo Uyum (Compliance) aracı — GUI ekranı.
//!
//! Envanterdeki her cihazın DB'de saklı SON config'ini güvenlik denetiminden
//! geçirir, cihaz başına skor ve filo geneli uyum duruşu gösterir. Denetim
//! mantığı `network::compliance` domaininde; bu ekran sadece okur ve çizer.

use eframe::egui;
use egui_plot::{Line, Plot, PlotPoints};

use super::{ToolEvent, ToolScreen};
use crate::db;
use crate::i18n::{Language, text};
use crate::network::compliance::{self, FleetPosture};

/// Tabloda gösterilecek, skora göre sıralanmış cihaz satırı.
struct DisplayRow {
    name: String,
    ip: String,
    score: u8,
    grade: &'static str,
    critical: usize,
    warning: usize,
    info: usize,
}

pub struct ComplianceTool {
    fleet: Option<FleetPosture>,
    rows: Vec<DisplayRow>,
    history: Vec<db::compliance::ScorePoint>,
    status: String,
}

impl Default for ComplianceTool {
    fn default() -> Self {
        let mut tool = Self {
            fleet: None,
            rows: Vec::new(),
            history: Vec::new(),
            status: String::new(),
        };
        tool.refresh();
        tool
    }
}

impl ComplianceTool {
    /// DB'den cihazları + son config'lerini okuyup filo posture hesaplar.
    fn refresh(&mut self) {
        self.fleet = None;
        self.rows.clear();

        let Ok(conn) = db::get_connection() else {
            self.status = "Database connection failed.".to_string();
            return;
        };

        let devices = match db::devices::all_devices(&conn) {
            Ok(d) => d,
            Err(e) => {
                self.status = format!("Device read failed: {e}");
                return;
            }
        };

        // Sadece en az bir config'i olan cihazlar denetlenir.
        let mut meta: Vec<(String, String)> = Vec::new(); // (name, ip)
        let mut configs: Vec<(String, String)> = Vec::new(); // (name, config)
        for d in &devices {
            if let Ok(history) = db::devices::get_config_history(&conn, d.id)
                && let Some(latest) = history.first()
            {
                meta.push((d.name.clone(), d.ip_address.clone()));
                configs.push((d.name.clone(), latest.config_text.clone()));
            }
        }

        let fleet = compliance::fleet_posture(&configs);

        // Görüntü satırları: fleet.devices ile meta aynı sırada; birleştir + sırala.
        for (i, dp) in fleet.devices.iter().enumerate() {
            let ip = meta.get(i).map(|m| m.1.clone()).unwrap_or_default();
            self.rows.push(DisplayRow {
                name: dp.device.clone(),
                ip,
                score: dp.summary.score,
                grade: dp.summary.grade,
                critical: dp.summary.critical,
                warning: dp.summary.warning,
                info: dp.summary.info,
            });
        }
        self.rows.sort_by_key(|r| r.score); // en riskli (düşük skor) başta

        self.history = db::compliance::fleet_history(&conn, 60).unwrap_or_default();

        self.status = if configs.is_empty() {
            "No stored device configurations found. Use Backup to store configs first.".to_string()
        } else {
            format!("{} device(s) audited.", configs.len())
        };
        self.fleet = Some(fleet);
    }

    /// Mevcut filo posture'unu geçmişe (DB) kaydeder ve trend'i günceller.
    fn record_snapshot(&mut self) {
        let Some(fleet) = &self.fleet else {
            return;
        };
        let Ok(conn) = db::get_connection() else {
            self.status = "Database connection failed.".to_string();
            return;
        };
        if let Err(e) = db::compliance::record_run(&conn, fleet) {
            self.status = format!("Snapshot failed: {e}");
            return;
        }
        self.history = db::compliance::fleet_history(&conn, 60).unwrap_or_default();
        self.status = format!("Snapshot recorded ({} points).", self.history.len());
    }
}

impl ToolScreen for ComplianceTool {
    fn id(&self) -> &'static str {
        "compliance"
    }

    fn icon(&self) -> &'static str {
        "📋"
    }

    fn name(&self, dil: Language) -> &'static str {
        text(dil, "Compliance", "Uyum")
    }

    fn draw(&mut self, ui: &mut egui::Ui, dil: Language) -> Option<ToolEvent> {
        ui.add_space(6.0);
        ui.heading(text(dil, "Fleet Compliance", "Filo Uyum Durumu"));
        ui.label(text(
            dil,
            "Security posture across all devices, based on their latest stored configuration.",
            "Tüm cihazların, saklı son config'lerine göre güvenlik duruşu.",
        ));
        ui.add_space(8.0);

        ui.horizontal(|ui| {
            if ui.button(text(dil, "Refresh", "Yenile")).clicked() {
                self.refresh();
            }
            let has_devices = self.fleet.as_ref().is_some_and(|f| f.device_count() > 0);
            if has_devices
                && ui
                    .button(text(dil, "Record snapshot", "Snapshot kaydet"))
                    .clicked()
            {
                self.record_snapshot();
            }
            if has_devices
                && ui
                    .button(text(dil, "Copy fleet report", "Filo raporunu kopyala"))
                    .clicked()
                && let Some(f) = &self.fleet
            {
                ui.ctx().copy_text(crate::report::fleet_markdown_report(f));
            }
        });

        if !self.status.is_empty() {
            ui.add_space(4.0);
            ui.label(egui::RichText::new(&self.status).weak());
        }

        ui.add_space(10.0);

        let Some(fleet) = &self.fleet else {
            return None;
        };
        if fleet.device_count() == 0 {
            return None;
        }

        // Filo özet kartı.
        fleet_karti(ui, dil, fleet);
        ui.add_space(10.0);

        // Duruş trendi: zaman içinde filo skoru (kaydedilmiş snapshot'lar).
        if self.history.len() >= 2 {
            ui.label(egui::RichText::new(text(dil, "Posture trend", "Duruş trendi")).strong());
            let pts: Vec<[f64; 2]> = self
                .history
                .iter()
                .enumerate()
                .map(|(i, p)| [i as f64, p.score as f64])
                .collect();
            Plot::new("compliance_trend")
                .height(150.0)
                .include_y(0.0)
                .include_y(105.0)
                .show(ui, |plot_ui| {
                    plot_ui.line(
                        Line::new(PlotPoints::from(pts))
                            .name(text(dil, "Fleet score", "Filo skoru")),
                    );
                });
        } else {
            ui.label(
                egui::RichText::new(text(
                    dil,
                    "Record snapshots over time to see the posture trend.",
                    "Trendi görmek için zamanla snapshot kaydedin.",
                ))
                .small()
                .weak(),
            );
        }

        ui.add_space(8.0);
        ui.separator();
        ui.add_space(8.0);

        // Cihaz başına skor tablosu.
        egui::ScrollArea::vertical().show(ui, |ui| {
            egui::Grid::new("compliance_grid")
                .num_columns(7)
                .striped(true)
                .spacing([16.0, 6.0])
                .show(ui, |ui| {
                    ui.label(egui::RichText::new(text(dil, "Device", "Cihaz")).strong());
                    ui.label(egui::RichText::new("IP").strong());
                    ui.label(egui::RichText::new(text(dil, "Score", "Skor")).strong());
                    ui.label(egui::RichText::new(text(dil, "Grade", "Not")).strong());
                    ui.label(egui::RichText::new(text(dil, "Crit", "Kritik")).strong());
                    ui.label(egui::RichText::new(text(dil, "Warn", "Uyarı")).strong());
                    ui.label(egui::RichText::new(text(dil, "Info", "Bilgi")).strong());
                    ui.end_row();

                    for r in &self.rows {
                        ui.label(&r.name);
                        ui.label(egui::RichText::new(&r.ip).monospace().weak());
                        ui.label(
                            egui::RichText::new(format!("{}", r.score))
                                .strong()
                                .color(grade_color(r.grade)),
                        );
                        ui.label(egui::RichText::new(r.grade).color(grade_color(r.grade)));
                        ui.label(renkli_sayi(r.critical, egui::Color32::from_rgb(220, 80, 80)));
                        ui.label(renkli_sayi(r.warning, egui::Color32::from_rgb(220, 150, 60)));
                        ui.label(renkli_sayi(r.info, egui::Color32::from_rgb(150, 150, 150)));
                        ui.end_row();
                    }
                });
        });

        None
    }
}

/// 0 ise soluk, değilse verilen renkte sayı.
fn renkli_sayi(n: usize, renk: egui::Color32) -> egui::RichText {
    if n == 0 {
        egui::RichText::new("0").weak()
    } else {
        egui::RichText::new(n.to_string()).color(renk).strong()
    }
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

/// Filo geneli özet kartı.
fn fleet_karti(ui: &mut egui::Ui, dil: Language, f: &FleetPosture) {
    egui::Frame::group(ui.style()).show(ui, |ui| {
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new(text(dil, "Fleet score:", "Filo skoru:")).strong());
            ui.label(
                egui::RichText::new(format!("{}/100  ({})", f.average_score, f.grade()))
                    .size(20.0)
                    .strong()
                    .color(grade_color(f.grade())),
            );
            ui.separator();
            ui.label(format!("{}: {}", text(dil, "Devices", "Cihaz"), f.device_count()));
            ui.separator();
            ui.label(
                egui::RichText::new(format!("● {}", f.total_critical))
                    .color(egui::Color32::from_rgb(220, 80, 80)),
            );
            ui.label(
                egui::RichText::new(format!("● {}", f.total_warning))
                    .color(egui::Color32::from_rgb(220, 150, 60)),
            );
            ui.label(
                egui::RichText::new(format!("● {}", f.total_info))
                    .color(egui::Color32::from_rgb(150, 150, 150)),
            );
        });
    });
}
