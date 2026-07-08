//! Ag domaini: subnet, ip ve ileride config/topoloji gibi saf ag mantigi.
//!
//! Bu katman arayuzden bagimsizdir. Ekrana bir sey yazmaz, dosya okumaz;
//! sadece "verilen ag verisinden dogru sonucu uretir".

pub mod compliance;
pub mod diff;
pub mod discovery;
pub mod security;
pub mod subnet;

// Sik kullanilan tipi disari kolay erisilsin diye yeniden ihrac ediyoruz.
pub use subnet::Subnet;
