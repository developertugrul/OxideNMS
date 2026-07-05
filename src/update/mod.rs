//! Surum kontrolu / zorunlu guncelleme mantigi.
//!
//! Bu modul saf "servis" mantigidir: internetteki bir manifest dosyasini
//! indirir, uygulamanin kendi surumuyle karsilastirir ve "ne yapilmali"
//! bilgisini dondurur. Ekrana bir sey cizmez — o is GUI'nin.
//!
//! Zorunlu guncellemenin sirri: uygulamanin surumu manifest'teki
//! `minimum_version`'dan kucukse, kullanici uygulamayi kullanamaz.

use semver::Version;
use serde::Deserialize;

use crate::error::UpdateError;

/// Manifest degeri gelmezse kullanilacak varsayilan URL.
///
/// NOT: Burayi kendi yayin adresinle degistir. Test icin `CISCO_MANIFEST_URL`
/// ortam degiskeni bunu gecersiz kilar (asagidaki `manifest_url`'e bak).
pub const DEFAULT_MANIFEST_URL: &str = "https://raw.githubusercontent.com/developertugrul/OxideNMS/main/latest.json";

/// Sunucudaki surum bilgisi dosyasinin (JSON) yapisi.
///
/// Ornek:
/// ```json
/// {
///   "latest_version": "1.3.0",
///   "minimum_version": "1.3.0",
///   "download_url": "https://.../cisco-setup.exe",
///   "notes": "Critical security guncellemesi"
/// }
/// ```
#[derive(Debug, Clone, Deserialize)]
pub struct VersionManifest {
    /// Yayinlanan en son surum.
    pub latest_version: String,
    /// Bundan eski surumler ZORUNLU olarak guncellenmeli.
    pub minimum_version: String,
    /// Kullanicinin yonlendirilecegi indirme adresi.
    pub download_url: String,
    /// Surum notu (istege bagli).
    #[serde(default)]
    pub notes: Option<String>,
}

/// Kontrol sonucu: uygulama ne yapmali?
#[derive(Debug, Clone)]
pub enum UpdateStatus {
    /// Guncel; normal calis.
    UpToDate,
    /// Daha yeni surum var ama zorunlu degil (kullanici devam edebilir).
    Optional(VersionManifest),
    /// Surum cok eski; kullanici GUNCELLEMEDEN uygulamayi kullanamaz.
    Mandatory(VersionManifest),
}

/// Manifest'i indirir, `current` surumle karsilastirir ve durumu dondurur.
///
/// `current` genelde `env!("CARGO_PKG_VERSION")` ile verilir.
pub fn check(url: &str, current: &str) -> Result<UpdateStatus, UpdateError> {
    let current = Version::parse(current)?;

    // 1) Manifest'i indir (kisa zaman asimlariyla; UI'yi kilitlememek icin
    //    bu fonksiyon zaten arka planda bir thread'de cagrilacak).
    let body = ureq::get(url)
        .timeout(std::time::Duration::from_secs(8))
        .call()?
        .into_string()?;

    // 2) JSON'u coz.
    let manifest: VersionManifest = serde_json::from_str(&body)?;

    // 3) Karari saf mantiga birak (I/O'suz, test edilebilir).
    decide(&current.to_string(), manifest)
}

/// Surum kararini verir: I/O icermez, bu yuzden kolayca test edilir.
/// `current` uygulamanin kendi surumu, `manifest` sunucudan gelen bilgi.
pub fn decide(current: &str, manifest: VersionManifest) -> Result<UpdateStatus, UpdateError> {
    let current = Version::parse(current)?;
    let minimum = Version::parse(&manifest.minimum_version)?;
    let latest = Version::parse(&manifest.latest_version)?;

    let durum = if current < minimum {
        UpdateStatus::Mandatory(manifest)
    } else if current < latest {
        UpdateStatus::Optional(manifest)
    } else {
        UpdateStatus::UpToDate
    };

    Ok(durum)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test icin hazir bir manifest uretir.
    fn manifest(minimum: &str, latest: &str) -> VersionManifest {
        VersionManifest {
            latest_version: latest.to_string(),
            minimum_version: minimum.to_string(),
            download_url: "https://ornek.com/indir".to_string(),
            notes: None,
        }
    }

    #[test]
    fn eski_surum_zorunlu_guncelleme() {
        // Uygulama 0.1.0, minimum 1.0.0 -> ZORUNLU.
        let result = decide("0.1.0", manifest("1.0.0", "1.0.0")).unwrap();
        assert!(matches!(result, UpdateStatus::Mandatory(_)));
    }

    #[test]
    fn yeni_var_ama_zorunlu_degil() {
        // Uygulama 1.0.0, minimum 1.0.0, en son 1.2.0 -> ISTEGE BAGLI.
        let result = decide("1.0.0", manifest("1.0.0", "1.2.0")).unwrap();
        assert!(matches!(result, UpdateStatus::Optional(_)));
    }

    #[test]
    fn guncel_surum() {
        // Uygulama en son surumde -> GUNCEL.
        let result = decide("1.2.0", manifest("1.0.0", "1.2.0")).unwrap();
        assert!(matches!(result, UpdateStatus::UpToDate));
    }

    #[test]
    fn gecersiz_surum_hata_dondurur() {
        assert!(decide("abc", manifest("1.0.0", "1.0.0")).is_err());
    }
}
