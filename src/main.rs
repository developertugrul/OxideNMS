//! Uygulamanin giris noktasi.
//!
//! Bilerek cok ince: butun is `cisco` kutuphanesinde. Bu dosya sadece
//! masaustu arayuzu (GUI) baslatir. CLI de kutuphanede duruyor; ileride
//! istersek argumanlara gore CLI/GUI secimi buraya eklenebilir.

fn main() -> eframe::Result {
    oxidenms::gui::run()
}
