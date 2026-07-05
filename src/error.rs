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
    Http(#[from] ureq::Error),

    #[error("Surum bilgisi okunamadi: {0}")]
    Io(#[from] std::io::Error),

    #[error("Surum bilgisi cozumlenemedi (bozuk JSON): {0}")]
    Parse(#[from] serde_json::Error),

    #[error("Gecersiz surum numarasi: {0}")]
    Version(#[from] semver::Error),
}
