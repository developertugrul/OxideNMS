//! Ayarlar aracı (GUI ekranı).
//!
//! `ToolScreen` trait'ini uygulayan özel bir araç: uygulama ayarlarını
//! (`AppSettings`) uygulama içinden düzenleyip diske kaydeder. Theme anında,
//! dil ise seçilir seçilmez (olay ile kabuğa bildirilerek) uygulanır.

use eframe::egui;

use super::{ToolEvent, ToolScreen};
use crate::i18n::{Language, Message, t};
use crate::settings::{AppSettings, Theme};

/// Ayarlar ekranın durumu: üzerinde çalışılan ayar kopyası + son kayıt mesajı.
pub struct SettingsTool {
    ayar: AppSettings,
    durum: Option<String>,
}

impl SettingsTool {
    /// Mevcut ayarlarla bir Ayarlar ekranı oluşturur.
    pub fn new(ayar: AppSettings) -> Self {
        Self { ayar, durum: None }
    }

    /// Seçili temaya karşılık gelen egui görünümü.
    fn gorunum(tema: Theme) -> egui::Visuals {
        match tema {
            Theme::Koyu => egui::Visuals::dark(),
            Theme::Acik => egui::Visuals::light(),
        }
    }
}

impl ToolScreen for SettingsTool {
    fn icon(&self) -> &'static str {
        "⚙"
    }

    fn name(&self, dil: Language) -> &'static str {
        t(dil, Message::SettingsName)
    }

    fn draw(&mut self, ui: &mut egui::Ui, dil: Language) -> Option<ToolEvent> {
        let mut olay = None;

        ui.add_space(6.0);
        ui.heading(t(dil, Message::SettingsName));
        ui.label(t(dil, Message::SettingsDescription));
        ui.add_space(12.0);

        // --- Language ---
        ui.label(egui::RichText::new(t(dil, Message::LanguageLabel)).strong());
        egui::ComboBox::from_id_salt("dil_secici")
            .selected_text(self.ayar.dil.yerel_ad())
            .show_ui(ui, |ui| {
                for &d in Language::hepsi() {
                    if ui
                        .selectable_value(&mut self.ayar.dil, d, d.yerel_ad())
                        .clicked()
                    {
                        // Kabuğa bildir ki uygulama geneli anında bu dile geçsin.
                        olay = Some(ToolEvent::LanguageSelected(d));
                    }
                }
            });

        ui.add_space(12.0);

        // --- Theme ---
        ui.label(egui::RichText::new(t(dil, Message::ThemeLabel)).strong());
        ui.horizontal(|ui| {
            ui.selectable_value(&mut self.ayar.tema, Theme::Koyu, t(dil, Message::ThemeDark));
            ui.selectable_value(
                &mut self.ayar.tema,
                Theme::Acik,
                t(dil, Message::ThemeLight),
            );
        });

        ui.add_space(12.0);

        // --- Manifest URL ---
        // Github release politikasina gore sabittir ve kullanici tarafindan degistirilemez.
        let mut manifest_url = crate::update::DEFAULT_MANIFEST_URL.to_string();
        ui.label(egui::RichText::new(t(dil, Message::ManifestUrl)).strong());
        ui.add(
            egui::TextEdit::singleline(&mut manifest_url)
                .desired_width(420.0)
                .interactive(false)
                .hint_text("https://raw.githubusercontent.com/.../latest.json"),
        );
        ui.label(
            egui::RichText::new(t(dil, Message::ManifestNote))
                .small()
                .weak(),
        );

        ui.add_space(16.0);
        ui.separator();
        ui.add_space(12.0);

        // --- Save ---
        if ui.button(t(dil, Message::Save)).clicked() {
            // Temayı anında uygula.
            ui.ctx().set_visuals(Self::gorunum(self.ayar.tema));
            // Diske yaz.
            self.durum = match self.ayar.save() {
                Ok(()) => Some(t(dil, Message::Saved).to_owned()),
                Err(e) => Some(format!("{}{e}", t(dil, Message::SaveFailedPrefix))),
            };
        }

        if let Some(mesaj) = &self.durum {
            ui.add_space(6.0);
            ui.label(egui::RichText::new(mesaj).weak());
        }

        // Ayar dosyasının yolunu göster.
        if let Some(yol) = AppSettings::dosya_yolu() {
            ui.add_space(12.0);
            ui.label(
                egui::RichText::new(format!("{}{}", t(dil, Message::FilePrefix), yol.display()))
                    .small()
                    .weak(),
            );
        }

        olay
    }
}
