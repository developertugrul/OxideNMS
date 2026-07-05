use super::{ToolEvent, ToolScreen};
use crate::i18n::{Language, Message, t};
use eframe::egui;

#[derive(Default, Clone)]
pub struct Interface {
    name: String,
    ip: String,
    vlan: String,
    mode: String,
    shutdown: bool,
}

pub struct TopologyTool {
    config_input: String,
    interfaces: Vec<Interface>,
}

impl Default for TopologyTool {
    fn default() -> Self {
        Self {
            config_input: String::new(),
            interfaces: Vec::new(),
        }
    }
}

impl ToolScreen for TopologyTool {
    fn id(&self) -> &'static str {
        "topology"
    }

    fn icon(&self) -> &'static str {
        "🕸️"
    }

    fn name(&self, dil: Language) -> &'static str {
        t(dil, Message::TopologyName)
    }

    fn draw(&mut self, ui: &mut egui::Ui, dil: Language) -> Option<ToolEvent> {
        ui.add_space(6.0);
        ui.heading(t(dil, Message::TopologyName));
        ui.label(t(dil, Message::TopologyDescription));
        ui.add_space(10.0);

        ui.label(t(dil, Message::TopologyConfigInput));

        ui.add(
            egui::TextEdit::multiline(&mut self.config_input)
                .desired_width(f32::INFINITY)
                .desired_rows(6)
                .code_editor(),
        );

        ui.add_space(8.0);

        if ui.button(t(dil, Message::TopologyParse)).clicked() {
            self.parse_config();
        }

        ui.add_space(12.0);
        ui.separator();
        ui.add_space(12.0);

        if !self.interfaces.is_empty() {
            egui::ScrollArea::vertical().show(ui, |ui| {
                egui::Grid::new("topology_grid")
                    .num_columns(4)
                    .spacing([20.0, 8.0])
                    .striped(true)
                    .show(ui, |ui| {
                        // Headers
                        ui.label(egui::RichText::new(t(dil, Message::TopologyInterface)).strong());
                        ui.label(egui::RichText::new(t(dil, Message::TopologyStatus)).strong());
                        ui.label(egui::RichText::new(t(dil, Message::TopologyIp)).strong());
                        ui.label(egui::RichText::new(t(dil, Message::TopologyVlan)).strong());
                        ui.end_row();

                        // Rows
                        for intf in &self.interfaces {
                            ui.label(egui::RichText::new(&intf.name).monospace());

                            if intf.shutdown {
                                ui.colored_label(
                                    egui::Color32::from_rgb(220, 80, 80),
                                    "DOWN (admin)",
                                );
                            } else {
                                ui.colored_label(egui::Color32::from_rgb(80, 220, 80), "UP");
                            }

                            let ip_display = if intf.ip.is_empty() { "-" } else { &intf.ip };
                            ui.label(egui::RichText::new(ip_display).monospace());

                            let vlan_display = if !intf.vlan.is_empty() {
                                format!("VLAN {}", intf.vlan)
                            } else if !intf.mode.is_empty() {
                                intf.mode.clone()
                            } else {
                                "-".to_owned()
                            };
                            ui.label(vlan_display);

                            ui.end_row();
                        }
                    });
            });
        }

        None
    }
}

impl TopologyTool {
    fn parse_config(&mut self) {
        self.interfaces.clear();
        let mut current_intf: Option<Interface> = None;

        for line in self.config_input.lines() {
            let trimmed_end = line.trim_end();
            if trimmed_end.starts_with("interface ") {
                if let Some(intf) = current_intf.take() {
                    self.interfaces.push(intf);
                }
                current_intf = Some(Interface {
                    name: trimmed_end.replace("interface ", ""),
                    ..Default::default()
                });
            } else if trimmed_end.starts_with(' ') && current_intf.is_some() {
                let l = trimmed_end.trim();
                let intf = current_intf.as_mut().unwrap();
                if l == "shutdown" {
                    intf.shutdown = true;
                } else if l.starts_with("ip address ") {
                    intf.ip = l.replace("ip address ", "");
                } else if l.starts_with("switchport access vlan ") {
                    intf.vlan = l.replace("switchport access vlan ", "");
                } else if l.starts_with("switchport mode ") {
                    intf.mode = l.replace("switchport mode ", "");
                }
            } else {
                if let Some(intf) = current_intf.take() {
                    self.interfaces.push(intf);
                }
            }
        }

        if let Some(intf) = current_intf.take() {
            self.interfaces.push(intf);
        }
    }
}
