//! Subnet Hesaplayıcı aracı (GUI ekranı).
//!
//! `ToolScreen` trait'ini uygulayan somut bir araç. Kendi girdi durumunu
//! (`cidr_input`) tutar. HESAPLAMA burada YOK — onu `network::Subnet` domaini
//! yapar; bu dosya sadece ekranı çizer ve domaini çağırır. Metinler `t()`
//! üzerinden seçili dilde gelir.

use eframe::egui;

use super::{ToolEvent, ToolScreen};
use crate::i18n::{Language, Message, t};
use crate::network::Subnet;

/// Subnet Hesaplayıcı ekranın durumu.
pub struct SubnetTool {
    cidr_input: String,
}

impl Default for SubnetTool {
    fn default() -> Self {
        Self {
            cidr_input: "192.168.1.10/24".to_owned(),
        }
    }
}

impl ToolScreen for SubnetTool {
    fn icon(&self) -> &'static str {
        "🖧"
    }

    fn name(&self, dil: Language) -> &'static str {
        t(dil, Message::SubnetName)
    }

    fn draw(&mut self, ui: &mut egui::Ui, dil: Language) -> Option<ToolEvent> {
        ui.add_space(6.0);
        ui.heading(t(dil, Message::SubnetName));
        ui.label(t(dil, Message::SubnetDescription));
        ui.add_space(10.0);

        ui.horizontal(|ui| {
            ui.label(t(dil, Message::CidrInput));
            ui.add(
                egui::TextEdit::singleline(&mut self.cidr_input)
                    .desired_width(220.0)
                    .hint_text("192.168.1.10/24"),
            );
        });

        ui.add_space(12.0);
        ui.separator();
        ui.add_space(12.0);

        match Subnet::parse(&self.cidr_input) {
            Ok(net) => sonuc_tablosu(ui, dil, &net),
            Err(hata) => {
                ui.colored_label(
                    egui::Color32::from_rgb(220, 80, 80),
                    format!("{}{hata}", t(dil, Message::ErrorPrefix)),
                );
            }
        }

        None
    }
}

/// Subnet sonuçlarını hizalı bir tablo (grid) olarak çizer.
fn sonuc_tablosu(ui: &mut egui::Ui, dil: Language, net: &Subnet) {
    let ilk_son = match (net.first_host(), net.last_host()) {
        (Some(ilk), Some(son)) => (ilk.to_string(), son.to_string()),
        _ => ("-".to_owned(), "-".to_owned()),
    };

    egui::Grid::new("subnet_sonuc")
        .num_columns(2)
        .spacing([24.0, 8.0])
        .striped(true)
        .show(ui, |ui| {
            line(
                ui,
                t(dil, Message::Input),
                &format!("{}/{}", net.ip(), net.prefix()),
            );
            line(ui, t(dil, Message::SubnetMask), &net.mask().to_string());
            line(ui, t(dil, Message::Network), &net.network().to_string());
            line(ui, t(dil, Message::Broadcast), &net.broadcast().to_string());
            line(ui, t(dil, Message::FirstHost), &ilk_son.0);
            line(ui, t(dil, Message::LastHost), &ilk_son.1);
            line(
                ui,
                t(dil, Message::UsableHosts),
                &net.usable_hosts().to_string(),
            );
        });
}

/// Grid içinde tek bir "etiket : değer" satırı çizer.
fn line(ui: &mut egui::Ui, etiket: &str, deger: &str) {
    ui.label(egui::RichText::new(etiket).weak());
    ui.label(egui::RichText::new(deger).monospace().strong());
    ui.end_row();
}
