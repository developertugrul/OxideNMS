//! Uygulama ayarlari — `settings.toml`.
//!
//! Sabit degerleri (manifest URL, tema) koddan cikarip kullanicinin
//! duzenleyebilecegi bir dosyaya aliyoruz. Dosya, OS'un standart config
//! klasorunde durur (Windows: %APPDATA%\cisco\settings.toml).
//!
//! Tasarim: yukleme ASLA panik atmaz. Dosya yoksa varsayilan olusturulup
//! diske yazilir; bozuksa varsayilana dusulur ve uyari basilir.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::i18n::Language;
use crate::update::DEFAULT_MANIFEST_URL;

/// Uygulama arayuz temasi.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Theme {
    Koyu,
    Acik,
}

impl Default for Theme {
    fn default() -> Self {
        Theme::Koyu
    }
}

/// Kullanicinin duzenleyebilecegi tum uygulama ayarlari.
///
/// `#[serde(default)]`: dosyada eksik alan olursa o alan varsayilanina duser
/// (eski ayar dosyalari yeni alanlar eklendiginde de calismaya devam eder).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AppSettings {
    /// Surum kontrolu icin manifest adresi.
    pub manifest_url: String,
    /// Arayuz temasi.
    pub tema: Theme,
    /// Arayuz dili.
    pub dil: Language,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            manifest_url: DEFAULT_MANIFEST_URL.to_string(),
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
        let icerik = toml::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        std::fs::write(&yol, icerik)
    }

    /// Kullanilacak manifest URL'i: `CISCO_MANIFEST_URL` ortam degiskeni varsa
    /// onu, yoksa ayar dosyasindakini dondurur. (Ortam degiskeni test icin.)
    pub fn effective_manifest_url(&self) -> String {
        std::env::var("CISCO_MANIFEST_URL").unwrap_or_else(|_| self.manifest_url.clone())
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
        assert_eq!(a.manifest_url, b.manifest_url);
        assert_eq!(a.tema, b.tema);
    }

    #[test]
    fn eksik_alan_varsayilana_duser() {
        // Bos toml -> tum alanlar varsayilan olmali.
        let b: AppSettings = toml::from_str("").unwrap();
        assert_eq!(b.tema, Theme::Koyu);
        assert_eq!(b.manifest_url, DEFAULT_MANIFEST_URL);
    }
}
