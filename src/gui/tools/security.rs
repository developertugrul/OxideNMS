//! Config Güvenlik Denetimi aracı (GUI ekranı).
//!
//! Kullanıcı Cisco config'ini yapıştırır, "Denetle"ye basar; `network::security`
//! domaini riskli ayarları bulur, bu ekran onları seviyeye göre renkli listeler.
//!
//! Not: Güvenlik metinleri şimdilik TR + EN (diğer langs İngilizce'ye düşer).
//! 17 dile tam çeviri sonraki bir adımda çekirdek i18n'e taşınabilir.

use eframe::egui;

use super::{ToolEvent, ToolScreen};
use crate::i18n::{Language, Message, t};
use crate::network::security::{self, Finding, FindingCode, Level};

/// Örnek (riskli) bir config — kullanıcı ilk açtığında aracı hemen denesin diye.
const ORNEK_CONFIG: &str = "\
hostname LAB-SW1
!
enable password 7 08701E1D0A18
!
username admin password admin
!
ip ssh version 1
!
line vty 0 4
 transport input telnet
 login
!
line aux 0
!
snmp-server community public RW
!
ip http server
!
end
";

pub struct SecurityTool {
    config_input: String,
    bulgular: Vec<Finding>,
    denetlendi: bool,
}

impl Default for SecurityTool {
    fn default() -> Self {
        Self {
            config_input: ORNEK_CONFIG.to_owned(),
            bulgular: Vec::new(),
            denetlendi: false,
        }
    }
}

impl ToolScreen for SecurityTool {
    fn icon(&self) -> &'static str {
        "🔒"
    }

    fn name(&self, dil: Language) -> &'static str {
        t(dil, Message::SecName)
    }

    fn draw(&mut self, ui: &mut egui::Ui, dil: Language) -> Option<ToolEvent> {
        ui.add_space(6.0);
        ui.heading(t(dil, Message::SecName));
        ui.label(t(dil, Message::SecDescription));
        ui.add_space(10.0);

        ui.add(
            egui::TextEdit::multiline(&mut self.config_input)
                .desired_rows(10)
                .desired_width(f32::INFINITY)
                .code_editor()
                .hint_text("hostname R1\nenable password ..."),
        );

        ui.add_space(8.0);
        if ui.button(t(dil, Message::SecAudit)).clicked() {
            self.bulgular = security::audit(&self.config_input);
            self.bulgular.sort_by_key(|b| b.level.sira());
            self.denetlendi = true;
        }

        ui.add_space(10.0);
        ui.separator();
        ui.add_space(10.0);

        if self.denetlendi {
            if self.bulgular.is_empty() {
                ui.colored_label(
                    egui::Color32::from_rgb(90, 190, 90),
                    t(dil, Message::SecNoIssues),
                );
            } else {
                let ozet =
                    t(dil, Message::SecFindings).replace("{0}", &self.bulgular.len().to_string());
                ui.label(egui::RichText::new(ozet).strong());
                ui.add_space(6.0);

                egui::ScrollArea::vertical().show(ui, |ui| {
                    for b in &self.bulgular {
                        self.bulgu_ciz(ui, dil, b);
                        ui.add_space(8.0);
                    }
                });
            }
        }

        None
    }
}

impl SecurityTool {
    /// Tek bir bulguyu renkli kart olarak çizer.
    fn bulgu_ciz(&self, ui: &mut egui::Ui, dil: Language, b: &Finding) {
        let (etiket, renk) = seviye_gorsel(dil, b.level);

        ui.horizontal(|ui| {
            ui.label(egui::RichText::new(etiket).strong().color(renk));
            ui.label(egui::RichText::new(bulgu_baslik(dil, b.code)).strong());
        });

        ui.label(bulgu_oneri(dil, b.code));

        if let (Some(no), Some(detail)) = (b.line, &b.detail) {
            ui.label(
                egui::RichText::new(format!("{} {no}: {detail}", t(dil, Message::SecLine)))
                    .monospace()
                    .weak(),
            );
        }
    }
}

/// Seviyenin ekran etiketi ve rengi.
fn seviye_gorsel(dil: Language, s: Level) -> (&'static str, egui::Color32) {
    match s {
        Level::Critical => (
            t(dil, Message::SecLevelCritical),
            egui::Color32::from_rgb(220, 80, 80),
        ),
        Level::Warning => (
            t(dil, Message::SecLevelWarning),
            egui::Color32::from_rgb(220, 150, 60),
        ),
        Level::Info => (
            t(dil, Message::SecLevelInfo),
            egui::Color32::from_rgb(150, 150, 150),
        ),
    }
}

/// Finding başlığı (TR/EN).
fn bulgu_baslik(dil: Language, code: FindingCode) -> &'static str {
    match code {
        FindingCode::TelnetEnabled => t(dil, Message::SecTitleTelnetEnabled),
        FindingCode::NoEnableSecret => t(dil, Message::SecTitleNoEnableSecret),
        FindingCode::SnmpPublic => t(dil, Message::SecTitleSnmpPublic),
        FindingCode::SnmpPrivate => t(dil, Message::SecTitleSnmpPrivate),
        FindingCode::NoPasswordEncryption => t(dil, Message::SecTitleNoPasswordEncryption),
        FindingCode::HttpServerEnabled => t(dil, Message::SecTitleHttpServerEnabled),
        FindingCode::WeakPassword => t(dil, Message::SecTitleWeakPassword),
        FindingCode::SnmpRw => t(dil, Message::SecTitleSnmpRw),
        FindingCode::SshV1 => t(dil, Message::SecTitleSshV1),
        FindingCode::Type7Password => t(dil, Message::SecTitleType7Password),
        FindingCode::LinePasswordless => t(dil, Message::SecTitleLinePasswordless),
        FindingCode::NoLogging => t(dil, Message::SecTitleNoLogging),
        FindingCode::NoNtpAuth => t(dil, Message::SecTitleNoNtpAuth),
    }
}

/// Finding için öneri (TR/EN).
fn bulgu_oneri(dil: Language, code: FindingCode) -> &'static str {
    match code {
        FindingCode::TelnetEnabled => t(dil, Message::SecAdviceTelnetEnabled),
        FindingCode::NoEnableSecret => t(dil, Message::SecAdviceNoEnableSecret),
        FindingCode::SnmpPublic => t(dil, Message::SecAdviceSnmpPublic),
        FindingCode::SnmpPrivate => t(dil, Message::SecAdviceSnmpPrivate),
        FindingCode::NoPasswordEncryption => t(dil, Message::SecAdviceNoPasswordEncryption),
        FindingCode::HttpServerEnabled => t(dil, Message::SecAdviceHttpServerEnabled),
        FindingCode::WeakPassword => t(dil, Message::SecAdviceWeakPassword),
        FindingCode::SnmpRw => t(dil, Message::SecAdviceSnmpRw),
        FindingCode::SshV1 => t(dil, Message::SecAdviceSshV1),
        FindingCode::Type7Password => t(dil, Message::SecAdviceType7Password),
        FindingCode::LinePasswordless => t(dil, Message::SecAdviceLinePasswordless),
        FindingCode::NoLogging => t(dil, Message::SecAdviceNoLogging),
        FindingCode::NoNtpAuth => t(dil, Message::SecAdviceNoNtpAuth),
    }
}
