//! Uluslararasılaştırma (i18n) — çoklu dil desteği.
//!
//! Mimari:
//!   - `Language`     : desteklenen langs (17). Ayarlarda saklanır, seçici gösterir.
//!   - `Message`   : çevrilebilir metinlerin tip-güvenli anahtarları.
//!   - `t()`     : (dil, mesaj) -> o dildeki text. Language dağıtımı burada.
//!   - `langs`  : her dilin text tablosu (veri).
//!
//! Arayüz katmanı asla sabit text yazmaz; her zaman `t(dil, Message::X)` çağırır.
//! Parametreli metinler `{0}`, `{1}` yer tutucuları içerir; çağıran taraf
//! `.replace("{0}", ...)` ile doldurur (kelime sırası dile göre korunur).

use serde::{Deserialize, Serialize};

pub mod langs;

/// Desteklenen langs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Language {
    #[default]
    Turkish,
    English,
    German,
    Russian,
    Spanish,
    Kazakh,
    Azerbaijani,
    Kyrgyz,
    Turkmen,
    Hindi,
    Chinese,
    Uzbek,
    Italian,
    Greek,
    Hungarian,
    Bulgarian,
    Portuguese,
}

impl Language {
    /// Language seçicide gösterilecek yerel name (endonim).
    pub fn yerel_ad(&self) -> &'static str {
        match self {
            Language::Turkish => "Türkçe",
            Language::English => "English",
            Language::German => "Deutsch",
            Language::Russian => "Русский",
            Language::Spanish => "Español",
            Language::Kazakh => "Қазақша",
            Language::Azerbaijani => "Azərbaycan",
            Language::Kyrgyz => "Кыргызча",
            Language::Turkmen => "Türkmençe",
            Language::Hindi => "हिन्दी",
            Language::Chinese => "中文",
            Language::Uzbek => "Oʻzbekcha",
            Language::Italian => "Italiano",
            Language::Greek => "Ελληνικά",
            Language::Hungarian => "Magyar",
            Language::Bulgarian => "Български",
            Language::Portuguese => "Português",
        }
    }

    /// Tüm langs — dil seçici bu sırayla listeler.
    pub fn hepsi() -> &'static [Language] {
        &[
            Language::Turkish,
            Language::English,
            Language::German,
            Language::Russian,
            Language::Spanish,
            Language::Kazakh,
            Language::Azerbaijani,
            Language::Kyrgyz,
            Language::Turkmen,
            Language::Hindi,
            Language::Chinese,
            Language::Uzbek,
            Language::Italian,
            Language::Greek,
            Language::Hungarian,
            Language::Bulgarian,
            Language::Portuguese,
        ]
    }
}

/// Çevrilebilir metinlerin anahtarları. Yeni bir UI metni = buraya bir anahtar
/// (ve `langs.rs`'teki her dile bir kol — derleyici zorlar).
#[derive(Debug, Clone, Copy)]
pub enum Message {
    // Genel / kabuk
    AppName,
    Tools,
    PlaceholderVlan,
    PlaceholderConfigDiff,
    PlaceholderTopology,
    // Güncelleme ekranları
    CheckingUpdates,
    UpdateRequired,
    VersionUnsupported,
    YourVersion,
    RequiredVersion,
    DownloadUpdate,
    DownloadInBrowser,
    UpdateCheckFailed,
    NewVersionAvailable,
    DeviceManager,
    AddDevice,
    DeviceName,
    IPAddress,
    Username,
    Password,
    SaveDevice,
    DeviceSaved,
    DeleteDevice,
    MasterPassword,
    EnterMasterPassword,
    Unlock,
    Unlocked,
    EncryptionError,
    BulkDeploy,
    SelectDevices,
    DeployCommands,
    DeploySuccess,
    Download,
    // Subnet aracı
    SubnetName,
    SubnetDescription,
    CidrInput,
    Input,
    SubnetMask,
    Network,
    Broadcast,
    FirstHost,
    LastHost,
    UsableHosts,
    ErrorPrefix,
    // Ayarlar aracı
    SettingsName,
    SettingsDescription,
    ThemeLabel,
    ThemeDark,
    ThemeLight,
    LanguageLabel,
    ManifestUrl,
    ManifestNote,
    Save,
    Saved,
    SaveFailedPrefix,
    FilePrefix,

    // Diff aracı
    DiffName,
    DiffDescription,
    DiffOldConfig,
    DiffNewConfig,
    DiffCompare,
    DiffSaveToDb,

    // Security aracı
    SecName,
    SecDescription,
    SecAudit,
    SecNoIssues,
    SecFindings,
    SecLine,
    SecLevelCritical,
    SecLevelWarning,
    SecLevelInfo,

    // Security Findings (Titles)
    SecTitleTelnetEnabled,
    SecTitleNoEnableSecret,
    SecTitleSnmpPublic,
    SecTitleSnmpPrivate,
    SecTitleNoPasswordEncryption,
    SecTitleHttpServerEnabled,
    SecTitleWeakPassword,
    SecTitleSnmpRw,
    SecTitleSshV1,
    SecTitleType7Password,
    SecTitleLinePasswordless,
    SecTitleNoLogging,
    SecTitleNoNtpAuth,

    // Security Findings (Advice)
    SecAdviceTelnetEnabled,
    SecAdviceNoEnableSecret,
    SecAdviceSnmpPublic,
    SecAdviceSnmpPrivate,
    SecAdviceNoPasswordEncryption,
    SecAdviceHttpServerEnabled,
    SecAdviceWeakPassword,
    SecAdviceSnmpRw,
    SecAdviceSshV1,
    SecAdviceType7Password,
    SecAdviceLinePasswordless,
    SecAdviceNoLogging,
    SecAdviceNoNtpAuth,

    // SSH Tool
    SshName,
    SshDescription,
    SshIp,
    SshPort,
    SshUser,
    SshPass,
    SshConnect,
    SshConnectingLabel,
    SshSuccess,

    // Vlan Tool
    VlanName,
    VlanDescription,
    VlanNetworkPrefix,
    VlanAddDepartment,
    VlanDeptName,
    VlanId,
    VlanHosts,
    VlanGenerate,
    VlanResult,

    // Topology Tool
    TopologyName,
    TopologyDescription,
    TopologyConfigInput,
    TopologyParse,
    TopologyInterface,
    TopologyStatus,
    TopologyIp,
    TopologyVlan,
}

/// Bir mesajın verilen dildeki karşılığı.
pub fn t(dil: Language, mesaj: Message) -> &'static str {
    use langs::*;
    match dil {
        Language::Turkish => tr(mesaj),
        Language::English => en(mesaj),
        Language::German => de(mesaj),
        Language::Russian => ru(mesaj),
        Language::Spanish => es(mesaj),
        Language::Kazakh => kk(mesaj),
        Language::Azerbaijani => az(mesaj),
        Language::Kyrgyz => ky(mesaj),
        Language::Turkmen => tk(mesaj),
        Language::Hindi => hi(mesaj),
        Language::Chinese => zh(mesaj),
        Language::Uzbek => uz(mesaj),
        Language::Italian => it(mesaj),
        Language::Greek => el(mesaj),
        Language::Hungarian => hu(mesaj),
        Language::Bulgarian => bg(mesaj),
        Language::Portuguese => pt(mesaj),
    }
}
