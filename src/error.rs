//! Uygulamanin merkezi hata tipleri.
//!
//! Her modul kendi hatasini burada tanimlar. `thiserror` sayesinde
//! her hataya insan-okunur bir mesaj bagliyoruz; bu mesajlar arayuzde
//! (CLI, ileride TUI/GUI) dogrudan gosterilebilir.

use thiserror::Error;

/// Ag domainiyle ilgili hatalar (subnet, ip, vs.).
#[derive(Debug, Error)]
pub enum NetworkError {
    #[error("Input 'ip/prefix' biciminde olmali, orn: 192.168.1.10/24")]
    InvalidFormat,

    #[error("Gecersiz IP adresi: {0}")]
    InvalidIp(String),

    #[error("Gecersiz prefix: {0}")]
    InvalidPrefix(String),

    #[error("Prefix 0 ile 32 arasinda olmali, verilen: {0}")]
    PrefixOutOfRange(u8),
}

/// Surum kontrolu / guncelleme ile ilgili hatalar.
#[derive(Debug, Error)]
pub enum UpdateError {
    #[error("Surum bilgisine ulasilamadi (ag hatasi): {0}")]
    Http(String),

    #[error("Surum bilgisi okunamadi: {0}")]
    Io(String),

    #[error("Surum bilgisi cozumlenemedi (bozuk JSON): {0}")]
    Parse(String),

    #[error("Gecersiz surum numarasi: {0}")]
    Version(String),
}

impl From<ureq::Error> for UpdateError {
    fn from(value: ureq::Error) -> Self {
        Self::Http(value.to_string())
    }
}

impl From<std::io::Error> for UpdateError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value.to_string())
    }
}

impl From<serde_json::Error> for UpdateError {
    fn from(value: serde_json::Error) -> Self {
        Self::Parse(value.to_string())
    }
}

impl From<semver::Error> for UpdateError {
    fn from(value: semver::Error) -> Self {
        Self::Version(value.to_string())
    }
}
