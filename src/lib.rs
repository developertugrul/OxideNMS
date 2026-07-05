//! `cisco` kutuphane koku.
//!
//! Tum modulleri burada bir araya getiriyoruz. Uygulama (main.rs) ve
//! ileride yazacagimiz testler bu kutuphaneyi kullanacak.
//!
//! Katmanlar:
//!   - `error`   : merkezi hata tipleri
//!   - `network` : ag domaini (saf mantik)
//!   - `update`  : surum kontrolu / zorunlu guncelleme
//!   - `cli`     : komut satiri arayuzu
//!   - `gui`     : masaustu grafik arayuz (egui)

pub mod cli;
pub mod crypto;
pub mod db;
pub mod error;
pub mod gui;
pub mod i18n;
pub mod network;
pub mod report;
pub mod settings;
pub mod update;
