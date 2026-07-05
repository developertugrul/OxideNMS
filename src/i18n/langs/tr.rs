use super::super::Message;

pub fn tr(m: Message) -> &'static str {
    match m {
        Message::AppName => "Cisco Ağ Araçları",
        Message::Tools => "ARAÇLAR",
        Message::PlaceholderVlan => "VLAN Planlayıcı (yakında)",
        Message::PlaceholderConfigDiff => "Config Karşılaştır (yakında)",
        Message::PlaceholderTopology => "Topoloji (yakında)",
        Message::CheckingUpdates => "Güncellemeler denetleniyor...",
        Message::UpdateRequired => "Zorunlu Güncelleme",
        Message::VersionUnsupported => {
            "Bu sürüm artık desteklenmiyor. Lütfen devam etmek için en güncel sürümü kurun."
        }
        Message::YourVersion => "Sizin sürümünüz",
        Message::RequiredVersion => "Gerekli sürüm",
        Message::DownloadUpdate => "Güncellemeyi İndir",
        Message::DownloadInBrowser => "İndirme işlemi tarayıcınızda açılacaktır.",
        Message::UpdateCheckFailed => "Güncelleme denetimi yapılamadı",
        Message::NewVersionAvailable => "Yeni sürüm mevcut: v{0} (sizdeki v{1}).",
        Message::Download => "İndir",
        Message::SubnetName => "Subnet Hesaplayıcı",
        Message::SubnetDescription => {
            "Bir CIDR girin (örn. 192.168.1.10/24). Sonuç anında hesaplanır."
        }
        Message::CidrInput => "CIDR:",
        Message::Input => "Girdi",
        Message::SubnetMask => "Subnet maskesi",
        Message::Network => "Ağ (Network)",
        Message::Broadcast => "Yayın (Broadcast)",
        Message::FirstHost => "İlk host",
        Message::LastHost => "Son host",
        Message::UsableHosts => "Kullanılabilir host",
        Message::ErrorPrefix => "Hata: ",
        Message::SettingsName => "Ayarlar",
        Message::SettingsDescription => {
            "Değişiklikler 'Kaydet' ile diskteki settings.toml dosyasına yazılır."
        }
        Message::ThemeLabel => "Tema",
        Message::ThemeDark => "Koyu",
        Message::ThemeLight => "Açık",
        Message::LanguageLabel => "Dil",
        Message::ManifestUrl => "Güncelleme manifest adresi",
        Message::ManifestNote => {
            "(Manifest değişimi uygulamayı yeniden başlatınca tam etkili olur.)"
        }
        Message::Save => "Kaydet",
        Message::Saved => "Kaydedildi.",
        Message::SaveFailedPrefix => "Kaydedilemedi: ",
        Message::FilePrefix => "Dosya: ",
        Message::DiffName => "Config Karşılaştır (Diff)",
        Message::DiffDescription => {
            "İki cihaz konfigürasyonunu karşılaştırıp değişen satırları tespit edin."
        }
        Message::DiffOldConfig => "Eski Config",
        Message::DiffNewConfig => "Yeni Config",
        Message::DiffCompare => "Karşılaştır",
        Message::DiffSaveToDb => "Yeni Config'i DB'ye Kaydet",
        Message::SecName => "Config Güvenlik Denetimi",
        Message::SecDescription => {
            "Cisco config'ini yapıştır ve 'Denetle'ye bas. Riskli ayarlar listelenir."
        }
        Message::SecAudit => "Denetle",
        Message::SecNoIssues => "Güvenlik sorunu bulunamadı.",
        Message::SecFindings => "{0} bulgu bulundu:",
        Message::SecLine => "Satır",
        Message::SecLevelCritical => "KRİTİK",
        Message::SecLevelWarning => "UYARI",
        Message::SecLevelInfo => "BİLGİ",
        Message::SecTitleTelnetEnabled => "Telnet etkin",
        Message::SecTitleNoEnableSecret => "Zayıf enable parolası",
        Message::SecTitleSnmpPublic => "SNMP community 'public'",
        Message::SecTitleSnmpPrivate => "SNMP community 'private'",
        Message::SecTitleNoPasswordEncryption => "Parola şifreleme kapalı",
        Message::SecTitleHttpServerEnabled => "HTTP sunucusu açık",
        Message::SecTitleWeakPassword => "Zayıf/varsayılan parola",
        Message::SecTitleSnmpRw => "SNMP yazma erişimi (RW)",
        Message::SecTitleSshV1 => "SSH v1 etkin",
        Message::SecTitleType7Password => "Zayıf şifreleme (Type-7)",
        Message::SecTitleLinePasswordless => "Parolasız hat erişimi",
        Message::SecTitleNoLogging => "Loglama kapalı",
        Message::SecTitleNoNtpAuth => "NTP kimlik doğrulama yok",
        Message::SecAdviceTelnetEnabled => {
            "Telnet şifresizdir. Bunun yerine 'transport input ssh' kullanın."
        }
        Message::SecAdviceNoEnableSecret => {
            "'enable password' zayıftır. Güçlü hash için 'enable secret' kullanın."
        }
        Message::SecAdviceSnmpPublic => {
            "Varsayılan 'public' community tahmin edilebilir. Değiştirin veya SNMPv3 kullanın."
        }
        Message::SecAdviceSnmpPrivate => "Varsayılan 'private' community'yi değiştirin.",
        Message::SecAdviceNoPasswordEncryption => {
            "'service password-encryption' ekleyin; düz metin parolaları gizler."
        }
        Message::SecAdviceHttpServerEnabled => {
            "'no ip http server' ile kapatın; gerekiyorsa HTTPS kullanın."
        }
        Message::SecAdviceWeakPassword => {
            "Tahmin edilebilir/varsayılan parolayı güçlü bir parolayla değiştirin."
        }
        Message::SecAdviceSnmpRw => {
            "SNMP yazma yetkisi (RW) güvenlik riskidir. Sadece RO (okuma) kullanın veya SNMPv3'e geçin."
        }
        Message::SecAdviceSshV1 => {
            "SSH sürüm 1 güvensizdir. 'ip ssh version 2' komutuyla güncelleyin."
        }
        Message::SecAdviceType7Password => {
            "Type-7 parolalar kolayca geri döndürülebilir. Güçlü 'secret' algoritmaları kullanın."
        }
        Message::SecAdviceLinePasswordless => {
            "Console, VTY veya AUX hatları parolasız bırakılamaz. Parola ayarlayın veya SSH key kullanın."
        }
        Message::SecAdviceNoLogging => {
            "Sistem olaylarını izlemek için 'logging host <IP>' veya 'logging buffered' etkinleştirilmeli."
        }
        Message::SecAdviceNoNtpAuth => {
            "NTP spoofing saldırılarını önlemek için NTP kimlik doğrulaması (NTP auth) yapılandırın."
        }

        Message::SshName => "SSH Cihaz Bağlantısı",
        Message::SshDescription => "Cihaza bağlanarak konfigürasyon çek",
        Message::SshIp => "IP Adresi:",
        Message::SshPort => "Port:",
        Message::SshUser => "Kullanıcı:",
        Message::SshPass => "Şifre:",
        Message::SshConnect => "Bağlan & Konfigürasyon Çek",
        Message::SshConnectingLabel => "Bağlanılıyor...",
        Message::SshSuccess => "Konfigürasyon başarıyla çekildi!",

        Message::VlanName => "VLAN Planlayıcı",
        Message::VlanDescription => "Departmanlar için Subnet ve VLAN konfigürasyonu oluşturur",
        Message::VlanNetworkPrefix => "Ana Ağ (CIDR):",
        Message::VlanAddDepartment => "Departman Ekle",
        Message::VlanDeptName => "Departman Adı",
        Message::VlanId => "VLAN ID",
        Message::VlanHosts => "Host Sayısı",
        Message::VlanGenerate => "Planla ve Config Üret",
        Message::VlanResult => "Üretilen Konfigürasyon",
        Message::TopologyName => "Config Ayrıştırıcı",
        Message::TopologyDescription => {
            "Running-config dosyasını analiz edip Interface durumlarını listeler"
        }
        Message::TopologyConfigInput => "Running-Config Metni:",
        Message::TopologyParse => "Ayrıştır",
        Message::TopologyInterface => "Arayüz (Interface)",
        Message::TopologyStatus => "Durum",
        Message::TopologyIp => "IP Adresi",
        Message::TopologyVlan => "VLAN / Mod",
        Message::DeviceManager => "Cihaz Yöneticisi",
        Message::AddDevice => "Cihaz Ekle",
        Message::DeviceName => "Cihaz Adı",
        Message::IPAddress => "IP Adresi",
        Message::Username => "Kullanıcı Adı",
        Message::Password => "Parola / SSH Anahtarı",
        Message::SaveDevice => "Cihazı Kaydet",
        Message::DeviceSaved => "Cihaz başarıyla kaydedildi.",
        Message::DeleteDevice => "Sil",
        Message::MasterPassword => "Ana Şifre (Master)",
        Message::EnterMasterPassword => "Şifreli cihaz kasasını açmak için Ana Şifreyi girin",
        Message::Unlock => "Kilidi Aç",
        Message::Unlocked => "Kasa Açık",
        Message::EncryptionError => "Şifreleme/Çözme Hatası",
        Message::BulkDeploy => "Toplu Dağıtım",
        Message::SelectDevices => "Cihazları Seç",
        Message::DeployCommands => "Komutları Gönder",
        Message::DeploySuccess => "Dağıtım tamamlandı",
    }
}
