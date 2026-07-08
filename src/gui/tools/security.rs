//! Config Güvenlik Denetimi aracı (GUI ekranı).
//!
//! Kullanıcı Cisco config'ini yapıştırır, "Denetle"ye basar; `network::security`
//! domaini riskli ayarları bulur, bu ekran onları seviyeye göre renkli listeler.
//!
//! Not: Güvenlik metinleri şimdilik TR + EN (diğer langs İngilizce'ye düşer).
//! 17 dile tam çeviri sonraki bir adımda çekirdek i18n'e taşınabilir.

use eframe::egui;

use super::{ToolEvent, ToolScreen};
use crate::i18n::{Language, Message, t, text};
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
 exec-timeout 0 0
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
                // Güvenlik duruşu skoru (profesyonel özet kartı).
                skor_karti(ui, dil, &security::summarize(&self.bulgular));
                ui.add_space(6.0);

                let ozet =
                    t(dil, Message::SecFindings).replace("{0}", &self.bulgular.len().to_string());
                ui.label(egui::RichText::new(ozet).strong());
                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    if ui.button("Markdown raporu kopyala").clicked() {
                        ui.ctx().copy_text(crate::report::security_markdown_report(
                            "pasted-config",
                            &self.bulgular,
                        ));
                    }
                    if ui.button("CSV raporu kopyala").clicked() {
                        ui.ctx()
                            .copy_text(crate::report::security_csv_report(&self.bulgular));
                    }
                });
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

        // Çerçeve referansı ve örnek düzeltme komutu (profesyonel dokunuş).
        ui.label(
            egui::RichText::new(format!("↳ {}", b.code.reference()))
                .small()
                .weak(),
        );
        ui.label(
            egui::RichText::new(format!("$ {}", b.code.remediation()))
                .monospace()
                .small()
                .color(egui::Color32::from_rgb(120, 170, 120)),
        );
    }
}

/// Güvenlik duruşu skor kartı: renkli skor/not + seviye sayıları.
fn skor_karti(ui: &mut egui::Ui, dil: Language, s: &security::AuditSummary) {
    let renk = match s.grade {
        "A" | "B" => egui::Color32::from_rgb(90, 190, 90),
        "C" => egui::Color32::from_rgb(210, 190, 70),
        "D" => egui::Color32::from_rgb(220, 150, 60),
        _ => egui::Color32::from_rgb(220, 80, 80),
    };
    egui::Frame::group(ui.style()).show(ui, |ui| {
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new(text(dil, "Security posture:", "Güvenlik duruşu:")).strong(),
            );
            ui.label(
                egui::RichText::new(format!("{}/100  ({})", s.score, s.grade))
                    .size(20.0)
                    .strong()
                    .color(renk),
            );
            ui.separator();
            ui.label(
                egui::RichText::new(format!("● {}", s.critical))
                    .color(egui::Color32::from_rgb(220, 80, 80)),
            );
            ui.label(
                egui::RichText::new(format!("● {}", s.warning))
                    .color(egui::Color32::from_rgb(220, 150, 60)),
            );
            ui.label(
                egui::RichText::new(format!("● {}", s.info))
                    .color(egui::Color32::from_rgb(150, 150, 150)),
            );
        });
    });
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
        // Yeni kurallar (şimdilik EN/TR; ileride tam i18n'e taşınabilir).
        FindingCode::ExecTimeoutDisabled => text(
            dil,
            "Session timeout disabled",
            "Oturum zaman aşımı kapalı",
        ),
        FindingCode::NoAaaNewModel => text(dil, "AAA not enabled", "AAA etkin değil"),
        FindingCode::NoLoginBanner => text(dil, "No login banner", "Giriş banner'ı yok"),
        FindingCode::WeakEnableSecretType => {
            text(dil, "Weak enable secret type", "Zayıf enable secret tipi")
        }
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
        // Yeni kurallar (şimdilik EN/TR).
        FindingCode::ExecTimeoutDisabled => text(
            dil,
            "'exec-timeout 0 0' keeps sessions open forever. Set a timeout, e.g. 'exec-timeout 10 0'.",
            "'exec-timeout 0 0' oturumu hiç kapatmaz. Bir zaman aşımı verin, örn: 'exec-timeout 10 0'.",
        ),
        FindingCode::NoAaaNewModel => text(
            dil,
            "No 'aaa new-model'. Enable AAA for centralized authentication and accounting.",
            "'aaa new-model' yok. Merkezi kimlik doğrulama/kayıt için AAA'yı etkinleştirin.",
        ),
        FindingCode::NoLoginBanner => text(
            dil,
            "No banner configured. Add a legal 'banner login/motd' warning against unauthorized access.",
            "Banner yok. Yetkisiz erişime karşı yasal bir 'banner login/motd' uyarısı ekleyin.",
        ),
        FindingCode::WeakEnableSecretType => text(
            dil,
            "'enable secret 5' is MD5. Prefer type 8 (PBKDF2) or type 9 (scrypt).",
            "'enable secret 5' MD5'tir. Tip 8 (PBKDF2) veya tip 9 (scrypt) tercih edin.",
        ),
    }
}
