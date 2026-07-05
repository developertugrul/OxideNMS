use crate::gui::tools::{ToolEvent, ToolScreen};
use crate::i18n::{Language, text};
use eframe::egui;
use minijinja::Environment;

pub struct TemplateTool {
    template_name: String,
    template_content: String,
    vars: Vec<(String, String)>,
    rendered_output: String,
    error_msg: String,
}

impl Default for TemplateTool {
    fn default() -> Self {
        Self {
            template_name: "VPN_Base".to_string(),
            template_content: "crypto isakmp policy 10
 encr aes
 hash sha
 authentication pre-share
 group 14
 lifetime 86400
crypto isakmp key {{ psk }} address {{ remote_ip }}

crypto ipsec transform-set {{ ts_name }} esp-aes esp-sha-hmac"
                .to_string(),
            vars: vec![
                ("psk".to_string(), "Cisco123!".to_string()),
                ("remote_ip".to_string(), "192.168.1.1".to_string()),
                ("ts_name".to_string(), "TS1".to_string()),
            ],
            rendered_output: String::new(),
            error_msg: String::new(),
        }
    }
}

impl TemplateTool {
    pub fn new() -> Self {
        Self::default()
    }

    fn render_template(&mut self) {
        self.error_msg.clear();
        let mut env = Environment::new();

        if let Err(e) =
            env.add_template_owned(self.template_name.clone(), self.template_content.clone())
        {
            self.error_msg = format!("Template error: {e}");
            return;
        }

        let tmpl = match env.get_template(&self.template_name) {
            Ok(t) => t,
            Err(e) => {
                self.error_msg = format!("Template not found: {e}");
                return;
            }
        };

        let mut map = std::collections::BTreeMap::new();
        for (k, v) in &self.vars {
            map.insert(k.clone(), v.clone());
        }

        match tmpl.render(map) {
            Ok(res) => self.rendered_output = res,
            Err(e) => self.error_msg = format!("Render failed: {e}"),
        }
    }
}

impl ToolScreen for TemplateTool {
    fn id(&self) -> &'static str {
        "template"
    }

    fn icon(&self) -> &'static str {
        "TPL"
    }

    fn name(&self, dil: Language) -> &'static str {
        text(dil, "Template Engine", "Sablon Motoru")
    }

    fn draw(&mut self, ui: &mut egui::Ui, dil: Language) -> Option<ToolEvent> {
        ui.heading(text(
            dil,
            "Configuration Template Engine (Jinja2)",
            "Konfigurasyon Sablon Motoru (Jinja2)",
        ));
        ui.add_space(10.0);

        egui::SidePanel::left("template_left_panel")
            .exact_width(300.0)
            .show_inside(ui, |ui| {
                ui.label(egui::RichText::new(text(dil, "Template code", "Sablon kodu")).strong());
                ui.add(
                    egui::TextEdit::multiline(&mut self.template_content)
                        .font(egui::TextStyle::Monospace)
                        .desired_width(f32::INFINITY)
                        .desired_rows(15),
                );

                ui.add_space(15.0);
                ui.label(egui::RichText::new(text(dil, "Variables", "Degiskenler")).strong());

                let mut to_remove = None;
                for (i, (key, val)) in self.vars.iter_mut().enumerate() {
                    ui.horizontal(|ui| {
                        ui.add(egui::TextEdit::singleline(key).desired_width(100.0));
                        ui.label("=");
                        ui.add(egui::TextEdit::singleline(val).desired_width(120.0));
                        if ui.button("X").clicked() {
                            to_remove = Some(i);
                        }
                    });
                }

                if let Some(idx) = to_remove {
                    self.vars.remove(idx);
                }

                if ui
                    .button(text(dil, "Add variable", "Degisken ekle"))
                    .clicked()
                {
                    self.vars
                        .push(("new_variable".to_string(), "value".to_string()));
                }

                ui.add_space(20.0);

                if ui
                    .button(egui::RichText::new(text(dil, "Render", "Derle")).size(16.0))
                    .clicked()
                {
                    self.render_template();
                }

                if !self.error_msg.is_empty() {
                    ui.label(egui::RichText::new(&self.error_msg).color(egui::Color32::RED));
                }
            });

        egui::CentralPanel::default().show_inside(ui, |ui| {
            ui.label(egui::RichText::new(text(dil, "Output", "Cikti")).strong());
            ui.add_space(5.0);

            egui::ScrollArea::both().show(ui, |ui| {
                ui.add(
                    egui::TextEdit::multiline(&mut self.rendered_output)
                        .font(egui::TextStyle::Monospace)
                        .desired_width(f32::INFINITY)
                        .desired_rows(25)
                        .interactive(true),
                );
            });

            if ui
                .button(text(dil, "Copy to clipboard", "Panoya kopyala"))
                .clicked()
            {
                ui.ctx().copy_text(self.rendered_output.clone());
            }
        });

        None
    }
}
