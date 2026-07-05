//! Uygulama ayarlari — `settings.toml`.
//!
//! Kullanici tercihlerini OS'un standart config klasorunde tutar
//! (Windows: %APPDATA%\cisco\settings.toml).
//!
//! Update manifest URL'i bilerek burada saklanmaz. Kullanici ayari veya ortam
//! degiskeni ile degistirilemeyen sabit release politikasinin parcasidir.
//!
//! Tasarim: yukleme ASLA panik atmaz. Dosya yoksa varsayilan olusturulup
//! diske yazilir; bozuksa varsayilana dusulur ve uyari basilir.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::i18n::Language;
/// Uygulama arayuz temasi.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Theme {
    #[default]
    Koyu,
    Acik,
}

/// Kullanicinin duzenleyebilecegi tum uygulama ayarlari.
///
/// `#[serde(default)]`: dosyada eksik alan olursa o alan varsayilanina duser
/// (eski ayar dosyalari yeni alanlar eklendiginde de calismaya devam eder).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AppSettings {
    /// Arayuz temasi.
    pub tema: Theme,
    /// Arayuz dili.
    pub dil: Language,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            tema: Theme::Koyu,
            dil: Language::default(),
        }
    }
}

impl AppSettings {
    /// Ayar dosyasinin tam yolu: `<config_dir>/cisco/settings.toml`.
    pub fn dosya_yolu() -> Option<PathBuf> {
        dirs::config_dir().map(|d| d.join("cisco").join("settings.toml"))
    }

    /// Ayarlari diskten yukler.
    /// - Dosya yoksa: varsayilani olusturup diske yazar.
    /// - Dosya bozuksa: varsayilana duser (uyari basar).
    ///
    /// Her durumda gecerli bir `AppSettings` doner (panik yok).
    pub fn load() -> Self {
        let Some(yol) = Self::dosya_yolu() else {
            return Self::default();
        };

        match std::fs::read_to_string(&yol) {
            Ok(icerik) => match toml::from_str(&icerik) {
                Ok(ayar) => ayar,
                Err(e) => {
                    eprintln!("Ayar dosyasi cozumlenemedi ({e}); varsayilan kullaniliyor.");
                    Self::default()
                }
            },
            Err(_) => {
                // Dosya yok: varsayilani diske yaz ki kullanici duzenleyebilsin.
                let ayar = Self::default();
                if let Err(e) = ayar.save() {
                    eprintln!("Ayar dosyasi olusturulamadi: {e}");
                }
                ayar
            }
        }
    }

    /// Ayarlari diske yazar (klasoru gerekirse olusturur).
    pub fn save(&self) -> std::io::Result<()> {
        let Some(yol) = Self::dosya_yolu() else {
            return Ok(());
        };
        if let Some(ust_klasor) = yol.parent() {
            std::fs::create_dir_all(ust_klasor)?;
        }
        let icerik = toml::to_string_pretty(self).map_err(std::io::Error::other)?;
        std::fs::write(&yol, icerik)
    }

    /// Kullanilacak manifest URL'i sabittir ve kullanici tarafindan degistirilemez.
    pub fn effective_manifest_url(&self) -> &'static str {
        crate::update::DEFAULT_MANIFEST_URL
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn varsayilan_toml_round_trip() {
        let a = AppSettings::default();
        let text = toml::to_string_pretty(&a).unwrap();
        let b: AppSettings = toml::from_str(&text).unwrap();
        assert_eq!(a.tema, b.tema);
    }

    #[test]
    fn eksik_alan_varsayilana_duser() {
        // Bos toml -> tum alanlar varsayilan olmali.
        let b: AppSettings = toml::from_str("").unwrap();
        assert_eq!(b.tema, Theme::Koyu);
        assert_eq!(
            b.effective_manifest_url(),
            crate::update::DEFAULT_MANIFEST_URL
        );
    }

    #[test]
    fn manifest_url_kullanici_ayarindan_okunmaz() {
        let b: AppSettings = toml::from_str(
            r#"
manifest_url = "https://attacker.invalid/latest.json"
tema = "Acik"
"#,
        )
        .unwrap();
        assert_eq!(
            b.effective_manifest_url(),
            crate::update::DEFAULT_MANIFEST_URL
        );
    }
}
