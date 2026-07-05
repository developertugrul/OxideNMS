use std::sync::mpsc::{Receiver, channel};
use std::time::Duration;

use eframe::egui;

use crate::db;
use crate::gui::tools::{ToolEvent, ToolScreen};
use crate::i18n::{Language, text};
use crate::network::discovery::{self, DiscoveryProbe};

enum DiscoveryState {
    Idle,
    Scanning,
    Done(Vec<DiscoveryProbe>),
    Error(String),
}

pub struct DiscoveryTool {
    cidr: String,
    timeout_ms: u64,
    state: DiscoveryState,
    rx: Option<Receiver<Result<Vec<DiscoveryProbe>, String>>>,
    status_msg: String,
}

impl Default for DiscoveryTool {
    fn default() -> Self {
        Self {
            cidr: "192.168.1.0/24".to_string(),
            timeout_ms: 300,
            state: DiscoveryState::Idle,
            rx: None,
            status_msg: String::new(),
        }
    }
}

impl DiscoveryTool {
    fn poll_result(&mut self) {
        if let Some(rx) = &self.rx
            && let Ok(result) = rx.try_recv()
        {
            self.state = match result {
                Ok(probes) => DiscoveryState::Done(probes),
                Err(e) => DiscoveryState::Error(e),
            };
            self.rx = None;
        }
    }

    fn start_scan(&mut self, ctx: egui::Context) {
        let cidr = self.cidr.clone();
        let timeout = Duration::from_millis(self.timeout_ms.clamp(50, 3000));
        let (tx, rx) = channel();
        self.rx = Some(rx);
        self.state = DiscoveryState::Scanning;
        self.status_msg.clear();

        std::thread::spawn(move || {
            let result = discovery::scan_cidr(&cidr, timeout).map_err(|e| e.to_string());
            let _ = tx.send(result);
            ctx.request_repaint();
        });
    }

    fn add_candidate(&mut self, probe: &DiscoveryProbe) {
        let ip = probe.ip.to_string();
        let name = format!("Discovered-{ip}");
        let details = probe.service_summary();

        match db::get_connection() {
            Ok(conn) => {
                let result = conn.execute(
                    "INSERT INTO devices (name, ip_address, platform, tags) VALUES (?1, ?2, ?3, ?4)",
                    [&name, &ip, &details, "discovered"],
                );
                match result {
                    Ok(_) => {
                        db::record_audit("discovery.add_device", &ip, "success", &details);
                        self.status_msg = format!("{ip} added to inventory.");
                    }
                    Err(e) => self.status_msg = format!("Save failed: {e}"),
                }
            }
            Err(e) => self.status_msg = format!("Database failed: {e}"),
        }
    }
}

impl ToolScreen for DiscoveryTool {
    fn id(&self) -> &'static str {
        "discovery"
    }

    fn icon(&self) -> &'static str {
        "DISC"
    }

    fn name(&self, _dil: Language) -> &'static str {
        "Discovery"
    }

    fn draw(&mut self, ui: &mut egui::Ui, dil: Language) -> Option<ToolEvent> {
        self.poll_result();

        ui.heading("Device Discovery");
        ui.label(text(
            dil,
            "Checks TCP/22 and TCP/161 reachability across a CIDR range.",
            "CIDR araliginda TCP/22 ve TCP/161 erisilebilirligini kontrol eder.",
        ));
        ui.add_space(10.0);

        ui.horizontal(|ui| {
            ui.label("CIDR");
            ui.text_edit_singleline(&mut self.cidr);
            ui.label("Timeout ms");
            ui.add(egui::DragValue::new(&mut self.timeout_ms).range(50..=3000));
            let scanning = matches!(self.state, DiscoveryState::Scanning);
            if ui
                .add_enabled(!scanning, egui::Button::new(text(dil, "Scan", "Tara")))
                .clicked()
            {
                self.start_scan(ui.ctx().clone());
            }
            if scanning {
                ui.spinner();
            }
        });

        if !self.status_msg.is_empty() {
            ui.colored_label(egui::Color32::YELLOW, &self.status_msg);
        }

        ui.add_space(12.0);

        match &self.state {
            DiscoveryState::Idle => {
                ui.label(text(dil, "Scan has not started.", "Tarama baslatilmadi."));
            }
            DiscoveryState::Scanning => {
                ui.label(text(
                    dil,
                    "Scan is running. Large subnets are limited to the first 1024 hosts.",
                    "Tarama calisiyor. Buyuk subnetler ilk 1024 host ile sinirlidir.",
                ));
            }
            DiscoveryState::Error(e) => {
                ui.colored_label(egui::Color32::RED, e);
            }
            DiscoveryState::Done(probes) => {
                ui.label(format!(
                    "{} {}",
                    probes.len(),
                    text(dil, "candidates found.", "aday bulundu.")
                ));
                let mut add_idx = None;
                egui::ScrollArea::vertical().show(ui, |ui| {
                    egui::Grid::new("discovery_results")
                        .striped(true)
                        .spacing([20.0, 8.0])
                        .show(ui, |ui| {
                            ui.label(egui::RichText::new("IP").strong());
                            ui.label(egui::RichText::new("SSH").strong());
                            ui.label(egui::RichText::new("SNMP").strong());
                            ui.label(egui::RichText::new("Services").strong());
                            ui.label("");
                            ui.end_row();

                            for (idx, probe) in probes.iter().enumerate() {
                                ui.label(probe.ip.to_string());
                                ui.label(if probe.ssh_open { "open" } else { "-" });
                                ui.label(if probe.snmp_open { "open" } else { "-" });
                                ui.label(probe.service_summary());
                                if ui
                                    .button(text(dil, "Add to inventory", "Envantere ekle"))
                                    .clicked()
                                {
                                    add_idx = Some(idx);
                                }
                                ui.end_row();
                            }
                        });
                });
                if let Some(idx) = add_idx
                    && let Some(probe) = probes.get(idx).cloned()
                {
                    self.add_candidate(&probe);
                }
            }
        }

        None
    }
}
