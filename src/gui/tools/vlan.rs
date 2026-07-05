use super::{ToolEvent, ToolScreen};
use crate::i18n::{Language, Message, t};
use eframe::egui;
use std::net::Ipv4Addr;

pub struct Department {
    name: String,
    vlan_id: String,
    hosts: String,
}

pub struct VlanTool {
    network_prefix: String,
    departments: Vec<Department>,
    result_config: String,
}

impl Default for VlanTool {
    fn default() -> Self {
        Self {
            network_prefix: "10.0.0.0/16".to_owned(),
            departments: vec![Department {
                name: "IT".to_owned(),
                vlan_id: "10".to_owned(),
                hosts: "50".to_owned(),
            }],
            result_config: String::new(),
        }
    }
}

impl ToolScreen for VlanTool {
    fn id(&self) -> &'static str {
        "vlan"
    }

    fn icon(&self) -> &'static str {
        "🏢"
    }

    fn name(&self, dil: Language) -> &'static str {
        t(dil, Message::VlanName)
    }

    fn draw(&mut self, ui: &mut egui::Ui, dil: Language) -> Option<ToolEvent> {
        ui.add_space(6.0);
        ui.heading(t(dil, Message::VlanName));
        ui.label(t(dil, Message::VlanDescription));
        ui.add_space(10.0);

        ui.horizontal(|ui| {
            ui.label(t(dil, Message::VlanNetworkPrefix));
            ui.text_edit_singleline(&mut self.network_prefix);
        });

        ui.add_space(10.0);

        // Departments Table
        let mut index_to_remove = None;
        egui::Grid::new("vlan_departments")
            .num_columns(4)
            .spacing([10.0, 8.0])
            .show(ui, |ui| {
                ui.label(t(dil, Message::VlanDeptName));
                ui.label(t(dil, Message::VlanId));
                ui.label(t(dil, Message::VlanHosts));
                ui.label("");
                ui.end_row();

                for (i, dept) in self.departments.iter_mut().enumerate() {
                    ui.text_edit_singleline(&mut dept.name);
                    ui.add(egui::TextEdit::singleline(&mut dept.vlan_id).desired_width(50.0));
                    ui.add(egui::TextEdit::singleline(&mut dept.hosts).desired_width(80.0));
                    if ui.button("❌").clicked() {
                        index_to_remove = Some(i);
                    }
                    ui.end_row();
                }
            });

        if let Some(idx) = index_to_remove {
            self.departments.remove(idx);
        }

        ui.add_space(8.0);

        ui.horizontal(|ui| {
            if ui.button(t(dil, Message::VlanAddDepartment)).clicked() {
                self.departments.push(Department {
                    name: String::new(),
                    vlan_id: String::new(),
                    hosts: String::new(),
                });
            }

            if ui.button(t(dil, Message::VlanGenerate)).clicked() {
                self.generate_config();
            }
        });

        ui.add_space(12.0);
        ui.separator();
        ui.add_space(12.0);

        if !self.result_config.is_empty() {
            ui.label(t(dil, Message::VlanResult));
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.add(
                    egui::TextEdit::multiline(&mut self.result_config.as_str())
                        .desired_width(f32::INFINITY)
                        .desired_rows(15)
                        .code_editor(),
                );
            });
        }

        None
    }
}

impl VlanTool {
    fn generate_config(&mut self) {
        use crate::network::Subnet;

        self.result_config.clear();

        let root_net = match Subnet::parse(&self.network_prefix) {
            Ok(net) => net,
            Err(_) => {
                self.result_config = "Geçersiz Ana Ağ CIDR formatı!".to_owned();
                return;
            }
        };

        // Parse and sort departments by host size descending
        let mut parsed_depts = Vec::new();
        for d in &self.departments {
            let hosts = d.hosts.parse::<u32>().unwrap_or(0);
            parsed_depts.push((d.name.clone(), d.vlan_id.clone(), hosts));
        }

        parsed_depts.sort_by(|a, b| b.2.cmp(&a.2));

        let mut current_ip = u32::from(root_net.network());
        let mut config_out = String::new();

        for (name, vlan_id, hosts) in parsed_depts {
            if hosts == 0 || name.is_empty() || vlan_id.is_empty() {
                continue;
            }

            // Calculate required prefix
            let mut prefix = 32;
            while (2u32.pow(32 - prefix) as i64) - 2 < hosts as i64 {
                if prefix == 0 {
                    break;
                }
                prefix -= 1;
            }

            // Align current_ip to the subnet size boundary
            let step = 2u32.pow(32 - prefix);
            if current_ip % step != 0 {
                current_ip += step - (current_ip % step);
            }

            let subnet = Subnet::new(Ipv4Addr::from(current_ip), prefix as u8).unwrap();
            let first_host = subnet.first_host().unwrap();
            let mask = subnet.mask();

            config_out.push_str(&format!("! Department: {}\n", name));
            config_out.push_str(&format!("vlan {}\n", vlan_id));
            config_out.push_str(&format!(" name {}\n", name));
            config_out.push_str(&format!("interface Vlan{}\n", vlan_id));
            config_out.push_str(&format!(" description {}\n", name));
            config_out.push_str(&format!(" ip address {} {}\n", first_host, mask));
            config_out.push_str(" no shutdown\n\n");

            current_ip += step;
        }

        self.result_config = config_out;
    }
}
