//! GUI araçları.
//!
//! Her araç (Subnet, Ayarlar, ileride VLAN planlayıcı...) burada bir struct
//! olur ve ortak `ToolScreen` sözleşmesini (trait) uygular. Böylece GUI kabuğu
//! araçların ne olduğunu bilmeden hepsini aynı şekilde listeler ve çizer.
//! Yeni araç eklemek = yeni struct + listeye kayıt.

use eframe::egui;

use crate::i18n::Language;

pub mod audit_log;
pub mod backup;
pub mod bulk_deploy;
pub mod compliance;
pub mod config_history;
pub mod dashboard;
pub mod device_manager;
pub mod diff;
pub mod discovery;
pub mod firmware;
pub mod security;
pub mod settings_mod;
pub mod snmp_map;
pub mod ssh;
pub mod subnet;
pub mod syslog;
pub mod template;
pub mod topology;
pub mod vlan;

pub use audit_log::AuditLogTool;
pub use backup::BackupTool;
pub use bulk_deploy::BulkDeployTool;
pub use compliance::ComplianceTool;
pub use config_history::ConfigHistoryTool;
pub use dashboard::DashboardTool;
pub use device_manager::DeviceManagerTool;
pub use diff::DiffTool;
pub use discovery::DiscoveryTool;
pub use firmware::FirmwareTool;
pub use security::SecurityTool;
pub use settings_mod::SettingsTool;
pub use snmp_map::SnmpMapTool;
pub use ssh::SshTool;
pub use subnet::SubnetTool;
pub use syslog::SyslogTool;
pub use template::TemplateTool;
pub use topology::TopologyTool;
pub use vlan::VlanTool;

/// Bir aracın çizim sırasında kabuğa bildirebileceği olay.
/// (Örn. Ayarlar aracı dil değiştirdiğinde uygulama genelinde uygulanmalı.)
pub enum ToolEvent {
    /// Kullanıcı yeni bir dil seçti.
    LanguageSelected(Language),
    /// Başka bir araca (Diff aracı) veri gönder ve oraya geç.
    SwitchToDiff {
        old_config: String,
        new_config: String,
    },
}

/// Bir aracın uyması gereken sözleşme (OOP'deki "interface").
///
/// GUI kabuğu bu trait üzerinden çalışır; somut aracın ne olduğunu bilmez
/// (buna "dynamic dispatch" denir: `Box<dyn ToolScreen>`).
pub trait ToolScreen {
    /// Aracın sistem içindeki benzersiz kimliği (örn. "diff", "ssh")
    fn id(&self) -> &'static str {
        "unknown"
    }

    /// Araç için menü ikonu
    fn icon(&self) -> &'static str {
        "🔧"
    }

    /// Sol panelde ve başlıkta görünecek name (seçili dile göre).
    fn name(&self, dil: Language) -> &'static str;

    /// Orta paneli çizer. Gerekirse kabuğa bir olay döndürür.
    fn draw(&mut self, ui: &mut egui::Ui, dil: Language) -> Option<ToolEvent>;

    /// Başka bir araçtan gelen veriyi kabul et (varsayılan: yoksay).
    fn receive_data(&mut self, _old: String, _new: String) {}
}
