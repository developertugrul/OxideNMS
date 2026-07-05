use crate::gui::tools::{ToolEvent, ToolScreen};
use crate::i18n::Language;
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

        if let Err(e) = env.add_template(&self.template_name, &self.template_content) {
            self.error_msg = format!("Şablon Hatası: {}", e);
            return;
        }

        let tmpl = match env.get_template(&self.template_name) {
            Ok(t) => t,
            Err(e) => {
                self.error_msg = format!("Şablon Bulunamadı: {}", e);
                return;
            }
        };

        // Değişkenleri dinamik olarak context'e ekle
        let _ctx = minijinja::Value::from(std::collections::BTreeMap::<String, String>::new());
        for (_k, _v) in &self.vars {
            // Basit bir şekilde manuel JSON/Map oluşturuyoruz
        }

        // Minijinja'nın `context!` makrosu sabit anahtarlar ister. Dinamik dict oluşturmak için serde veya value::Map kullanabiliriz.
        // Rust'ta en kolay yol `BTreeMap`'i `Value`'ya çevirmektir.
        let mut map = std::collections::BTreeMap::new();
        for (k, v) in &self.vars {
            map.insert(k.clone(), v.clone());
        }

        match tmpl.render(map) {
            Ok(res) => self.rendered_output = res,
            Err(e) => self.error_msg = format!("Render Hatası: {}", e),
        }
    }
}

impl ToolScreen for TemplateTool {
    fn id(&self) -> &'static str {
        "template"
    }

    fn icon(&self) -> &'static str {
        "📝"
    }

    fn name(&self, _dil: Language) -> &'static str {
        "Şablon Motoru"
    }

    fn draw(&mut self, ui: &mut egui::Ui, _dil: Language) -> Option<ToolEvent> {
        ui.heading("Konfigürasyon Şablon Motoru (Jinja2)");
        ui.add_space(10.0);

        egui::SidePanel::left("template_left_panel")
            .exact_width(300.0)
            .show_inside(ui, |ui| {
                ui.label(egui::RichText::new("Şablon Kodu (Template)").strong());
                ui.add(
                    egui::TextEdit::multiline(&mut self.template_content)
                        .font(egui::TextStyle::Monospace)
                        .desired_width(f32::INFINITY)
                        .desired_rows(15),
                );

                ui.add_space(15.0);
                ui.label(egui::RichText::new("Değişkenler (Variables)").strong());

                let mut to_remove = None;
                for (i, (key, val)) in self.vars.iter_mut().enumerate() {
                    ui.horizontal(|ui| {
                        ui.add(egui::TextEdit::singleline(key).desired_width(100.0));
                        ui.label("=");
                        ui.add(egui::TextEdit::singleline(val).desired_width(120.0));
                        if ui.button("❌").clicked() {
                            to_remove = Some(i);
                        }
                    });
                }

                if let Some(idx) = to_remove {
                    self.vars.remove(idx);
                }

                if ui.button("➕ Değişken Ekle").clicked() {
                    self.vars
                        .push(("yeni_degisken".to_string(), "deger".to_string()));
                }

                ui.add_space(20.0);

                if ui
                    .button(egui::RichText::new("⚙️ Derle (Render)").size(16.0))
                    .clicked()
                {
                    self.render_template();
                }

                if !self.error_msg.is_empty() {
                    ui.label(egui::RichText::new(&self.error_msg).color(egui::Color32::RED));
                }
            });

        egui::CentralPanel::default().show_inside(ui, |ui| {
            ui.label(egui::RichText::new("Çıktı (Rendered Config)").strong());
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

            if ui.button("📋 Panoya Kopyala").clicked() {
                ui.ctx().copy_text(self.rendered_output.clone());
            }
        });

        None
    }
}
