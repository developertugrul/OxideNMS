//! GUI araçları.
//!
//! Her araç (Subnet, Ayarlar, ileride VLAN planlayıcı...) burada bir struct
//! olur ve ortak `ToolScreen` sözleşmesini (trait) uygular. Böylece GUI kabuğu
//! araçların ne olduğunu bilmeden hepsini aynı şekilde listeler ve çizer.
//! Yeni araç eklemek = yeni struct + listeye kayıt.

use eframe::egui;

use crate::i18n::Language;

pub mod settings_mod;
pub mod security;
pub mod subnet;
pub mod diff;
pub mod ssh;
pub mod vlan;
pub mod topology;
pub mod device_manager;
pub mod bulk_deploy;
pub mod backup;
pub mod snmp_map;

pub use settings_mod::SettingsTool;
pub use security::SecurityTool;
pub use subnet::SubnetTool;
pub use diff::DiffTool;
pub use ssh::SshTool;
pub use vlan::VlanTool;
pub use topology::TopologyTool;
pub use device_manager::DeviceManagerTool;
pub use bulk_deploy::BulkDeployTool;
pub use backup::BackupTool;
pub use snmp_map::SnmpMapTool;

/// Bir aracın çizim sırasında kabuğa bildirebileceği olay.
/// (Örn. Ayarlar aracı dil değiştirdiğinde uygulama genelinde uygulanmalı.)
pub enum ToolEvent {
    /// Kullanıcı yeni bir dil seçti.
    LanguageSelected(Language),
    /// Başka bir araca (Diff aracı) veri gönder ve oraya geç.
    SwitchToDiff { old_config: String, new_config: String },
}

/// Bir aracın uyması gereken sözleşme (OOP'deki "interface").
///
/// GUI kabuğu bu trait üzerinden çalışır; somut aracın ne olduğunu bilmez
/// (buna "dynamic dispatch" denir: `Box<dyn ToolScreen>`).
pub trait ToolScreen {
    /// Aracın sistem içindeki benzersiz kimliği (örn. "diff", "ssh")
    fn id(&self) -> &'static str { "unknown" }

    /// Araç için menü ikonu
    fn icon(&self) -> &'static str { "🔧" }

    /// Sol panelde ve başlıkta görünecek name (seçili dile göre).
    fn name(&self, dil: Language) -> &'static str;

    /// Orta paneli çizer. Gerekirse kabuğa bir olay döndürür.
    fn draw(&mut self, ui: &mut egui::Ui, dil: Language) -> Option<ToolEvent>;

    /// Başka bir araçtan gelen veriyi kabul et (varsayılan: yoksay).
    fn receive_data(&mut self, _old: String, _new: String) {}
}
