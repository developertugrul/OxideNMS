//! Cisco konfigürasyon güvenlik denetimi (hardening).
//!
//! Saf mantık: bir Cisco config metni verilir, riskli/zayıf ayarları
//! (`Finding` listesi) döndürür. Ekran/dil/dosya YOK — o işler arayüzün.
//! Kurallar text taramasıyla; "line" blokları için basit blok takibi ile.

/// Bir bulgunun önem derecesi.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Level {
    Critical,
    Warning,
    Info,
}

impl Level {
    /// Sıralama için: küçük sayı = daha önemli (önce gösterilir).
    pub fn sira(&self) -> u8 {
        match self {
            Level::Critical => 0,
            Level::Warning => 1,
            Level::Info => 2,
        }
    }
}

/// Hangi güvenlik kuralının tetiklendiğini belirten code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FindingCode {
    TelnetEnabled,
    NoEnableSecret,
    SnmpPublic,
    SnmpPrivate,
    SnmpRw,
    NoPasswordEncryption,
    HttpServerEnabled,
    WeakPassword,
    SshV1,
    Type7Password,
    LinePasswordless,
    NoLogging,
    NoNtpAuth,
    ExecTimeoutDisabled,
    NoAaaNewModel,
    NoLoginBanner,
    WeakEnableSecretType,
}

/// Tek bir güvenlik bulgusu.
#[derive(Debug, Clone)]
pub struct Finding {
    pub level: Level,
    pub code: FindingCode,
    /// İlgili satır numarası (1'den başlar), varsa.
    pub line: Option<usize>,
    /// Sorunlu satırın metni, varsa.
    pub detail: Option<String>,
}

/// Zayıf/varsayılan sayılan parola değerleri (küçük harf).
const ZAYIF_PAROLALAR: &[&str] = &["cisco", "admin", "password", "123456", "default", "pass"];

/// Bir "line" bloğunun (line con/vty/aux) denetim sırasındaki durumu.
struct HatDurumu {
    header: String,
    line: usize,
    parola: bool,
    login_guvenli: bool,
}

/// Aktif "line" bloğunu kapatır; parolasızsa bulgu ekler.
fn hat_bitir(aktif: &mut Option<HatDurumu>, bulgular: &mut Vec<Finding>) {
    let Some(h) = aktif.take() else {
        return;
    };
    let hl = h.header.to_lowercase();
    let vty = hl.contains("vty");
    let con = hl.contains("con");
    let aux = hl.contains("aux");
    if (vty || con || aux) && !h.parola && !h.login_guvenli {
        // vty/aux uzaktan erişimdir -> kritik; console fiziksel -> uyarı.
        let level = if vty || aux {
            Level::Critical
        } else {
            Level::Warning
        };
        bulgular.push(Finding {
            level,
            code: FindingCode::LinePasswordless,
            line: Some(h.line),
            detail: Some(h.header),
        });
    }
}

/// Bir Cisco config metnini denetler ve bulguları döndürür.
pub fn audit(config: &str) -> Vec<Finding> {
    let mut bulgular = Vec::new();

    // Tüm-config düzeyindeki bayraklar.
    let mut parola_sifreleme = false;
    let mut enable_secret = false;
    let mut enable_password: Option<(usize, String)> = None;
    let mut logging_var = false;
    let mut ntp_server = false;
    let mut ntp_auth = false;
    let mut aaa_new_model = false;
    let mut banner_var = false;

    // Aktif "line" bloğu.
    let mut aktif_hat: Option<HatDurumu> = None;

    for (i, ham) in config.lines().enumerate() {
        let satir_no = i + 1;
        let indentli = ham.starts_with(' ') || ham.starts_with('\t');
        let l = ham.trim();
        let lc = l.to_lowercase();

        // --- "line" blok yönetimi ---
        if lc.starts_with("line ") {
            hat_bitir(&mut aktif_hat, &mut bulgular);
            aktif_hat = Some(HatDurumu {
                header: l.to_string(),
                line: satir_no,
                parola: false,
                login_guvenli: false,
            });
        } else if aktif_hat.is_some() {
            if indentli {
                if let Some(h) = aktif_hat.as_mut() {
                    if lc.starts_with("password") {
                        h.parola = true;
                    }
                    if lc.starts_with("login local") || lc.starts_with("login authentication") {
                        h.login_guvenli = true;
                    }
                }
            } else {
                // İndentsiz satır -> blok bitti (satır kuralları aşağıda yine işler).
                hat_bitir(&mut aktif_hat, &mut bulgular);
            }
        }

        // --- Tüm-config bayrakları ---
        if lc.starts_with("service password-encryption") {
            parola_sifreleme = true;
        }
        if lc.starts_with("enable secret") {
            enable_secret = true;
        }
        if lc.starts_with("enable password") {
            enable_password = Some((satir_no, l.to_string()));
        }
        if !indentli && lc.starts_with("logging ") {
            logging_var = true;
        }
        if lc.contains("ntp server") {
            ntp_server = true;
        }
        if lc.starts_with("ntp authenticate") {
            ntp_auth = true;
        }
        if lc.starts_with("aaa new-model") {
            aaa_new_model = true;
        }
        if lc.starts_with("banner ") {
            banner_var = true;
        }

        // --- Satır bazlı kurallar ---

        // Telnet etkin mi?
        if lc.contains("transport input telnet") || lc.contains("transport input all") {
            ekle(
                &mut bulgular,
                Level::Critical,
                FindingCode::TelnetEnabled,
                satir_no,
                l,
            );
        }

        // SSH sürüm 1.
        if lc.contains("ip ssh version 1") {
            ekle(
                &mut bulgular,
                Level::Critical,
                FindingCode::SshV1,
                satir_no,
                l,
            );
        }

        // SNMP varsayılan community'ler ve yazma erişimi.
        if lc.contains("snmp-server community public") {
            ekle(
                &mut bulgular,
                Level::Critical,
                FindingCode::SnmpPublic,
                satir_no,
                l,
            );
        } else if lc.contains("snmp-server community private") {
            ekle(
                &mut bulgular,
                Level::Warning,
                FindingCode::SnmpPrivate,
                satir_no,
                l,
            );
        }
        if lc.starts_with("snmp-server community") && lc.contains(" rw") {
            ekle(
                &mut bulgular,
                Level::Warning,
                FindingCode::SnmpRw,
                satir_no,
                l,
            );
        }

        // Şifresiz HTTP sunucusu (HTTPS 'secure-server' hariç).
        if lc == "ip http server" {
            ekle(
                &mut bulgular,
                Level::Warning,
                FindingCode::HttpServerEnabled,
                satir_no,
                l,
            );
        }

        // Oturum zaman aşımı kapalı: 'exec-timeout 0 0' -> oturum hiç kapanmaz.
        if lc == "exec-timeout 0 0" || lc == "exec-timeout 0" {
            ekle(
                &mut bulgular,
                Level::Warning,
                FindingCode::ExecTimeoutDisabled,
                satir_no,
                l,
            );
        }

        // Zayıf enable secret tipi: Type-5 (MD5). Type-8/9 önerilir.
        if lc.starts_with("enable secret 5 ") {
            ekle(
                &mut bulgular,
                Level::Info,
                FindingCode::WeakEnableSecretType,
                satir_no,
                l,
            );
        }

        // Type-7 (geri döndürülebilir) parola.
        if (lc.contains("password") || lc.contains("secret")) && lc.contains(" 7 ") {
            ekle(
                &mut bulgular,
                Level::Warning,
                FindingCode::Type7Password,
                satir_no,
                l,
            );
        }

        // Zayıf/varsayılan parola: parola satırındaki son değeri kontrol et.
        if (lc.contains("password") || lc.contains("secret"))
            && let Some(son) = l.split_whitespace().next_back()
            && ZAYIF_PAROLALAR.contains(&son.to_lowercase().as_str())
        {
            ekle(
                &mut bulgular,
                Level::Critical,
                FindingCode::WeakPassword,
                satir_no,
                l,
            );
        }
    }

    // Döngü sonunda kalan bloğu kapat.
    hat_bitir(&mut aktif_hat, &mut bulgular);

    // --- Tüm-config kontrolleri ---

    if !enable_secret && let Some((no, txt)) = enable_password {
        bulgular.push(Finding {
            level: Level::Warning,
            code: FindingCode::NoEnableSecret,
            line: Some(no),
            detail: Some(txt),
        });
    }

    if !parola_sifreleme {
        bulgular.push(bulgu_satirsiz(
            Level::Warning,
            FindingCode::NoPasswordEncryption,
        ));
    }

    if !logging_var {
        bulgular.push(bulgu_satirsiz(Level::Info, FindingCode::NoLogging));
    }

    if ntp_server && !ntp_auth {
        bulgular.push(bulgu_satirsiz(Level::Warning, FindingCode::NoNtpAuth));
    }

    if !aaa_new_model {
        bulgular.push(bulgu_satirsiz(Level::Info, FindingCode::NoAaaNewModel));
    }

    if !banner_var {
        bulgular.push(bulgu_satirsiz(Level::Info, FindingCode::NoLoginBanner));
    }

    bulgular
}

/// Satır bilgili bir bulguyu listeye ekler (kısaltma yardımcısı).
fn ekle(bulgular: &mut Vec<Finding>, level: Level, code: FindingCode, line: usize, detail: &str) {
    bulgular.push(Finding {
        level,
        code,
        line: Some(line),
        detail: Some(detail.to_string()),
    });
}

/// Satırsız (tüm-config) bir bulgu üretir.
fn bulgu_satirsiz(level: Level, code: FindingCode) -> Finding {
    Finding {
        level,
        code,
        line: None,
        detail: None,
    }
}

/// Bulgu kategorisi (rapor gruplaması + çerçeve eşlemesi için).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Category {
    Access,
    Authentication,
    Snmp,
    Services,
    Logging,
}

impl Category {
    /// Tüm kategoriler (rapor gruplaması bu sırayla).
    pub fn all() -> &'static [Category] {
        &[
            Category::Access,
            Category::Authentication,
            Category::Snmp,
            Category::Services,
            Category::Logging,
        ]
    }

    /// Kategori etiketi (İngilizce; rapor/başlık için).
    pub fn label(&self) -> &'static str {
        match self {
            Category::Access => "Access & Remote Management",
            Category::Authentication => "Authentication & Passwords",
            Category::Snmp => "SNMP",
            Category::Services => "Services",
            Category::Logging => "Logging & Time",
        }
    }
}

impl FindingCode {
    /// Bulgunun ait olduğu kategori.
    pub fn category(&self) -> Category {
        match self {
            FindingCode::TelnetEnabled
            | FindingCode::LinePasswordless
            | FindingCode::ExecTimeoutDisabled
            | FindingCode::SshV1
            | FindingCode::NoLoginBanner => Category::Access,
            FindingCode::NoEnableSecret
            | FindingCode::WeakEnableSecretType
            | FindingCode::NoPasswordEncryption
            | FindingCode::WeakPassword
            | FindingCode::Type7Password
            | FindingCode::NoAaaNewModel => Category::Authentication,
            FindingCode::SnmpPublic | FindingCode::SnmpPrivate | FindingCode::SnmpRw => {
                Category::Snmp
            }
            FindingCode::HttpServerEnabled => Category::Services,
            FindingCode::NoLogging | FindingCode::NoNtpAuth => Category::Logging,
        }
    }

    /// İlgili güvenlik çerçevesi referansı (bölüm düzeyinde, gösterge amaçlı).
    pub fn reference(&self) -> &'static str {
        match self {
            FindingCode::TelnetEnabled => "CIS IOS: Access Rules · NIST AC-17",
            FindingCode::SshV1 => "CIS IOS: Access Rules · NIST AC-17",
            FindingCode::LinePasswordless => "CIS IOS: Access Rules · NIST AC-3",
            FindingCode::ExecTimeoutDisabled => "CIS IOS: Access Rules · NIST AC-12",
            FindingCode::NoLoginBanner => "CIS IOS: Banner Rules · NIST AC-8",
            FindingCode::NoEnableSecret => "CIS IOS: Password Rules · NIST IA-5",
            FindingCode::WeakEnableSecretType => "CIS IOS: Password Rules · NIST IA-5",
            FindingCode::NoPasswordEncryption => "CIS IOS: Password Rules · NIST IA-5",
            FindingCode::WeakPassword => "CIS IOS: Password Rules · NIST IA-5",
            FindingCode::Type7Password => "CIS IOS: Password Rules · NIST IA-5",
            FindingCode::NoAaaNewModel => "CIS IOS: Local AAA Rules · NIST IA-2",
            FindingCode::SnmpPublic => "CIS IOS: SNMP Rules · NIST CM-6",
            FindingCode::SnmpPrivate => "CIS IOS: SNMP Rules · NIST CM-6",
            FindingCode::SnmpRw => "CIS IOS: SNMP Rules · NIST CM-6",
            FindingCode::HttpServerEnabled => "CIS IOS: Services · NIST CM-7",
            FindingCode::NoLogging => "CIS IOS: Logging Rules · NIST AU-2",
            FindingCode::NoNtpAuth => "CIS IOS: NTP Rules · NIST AU-8",
        }
    }

    /// Örnek düzeltme komutu (Cisco IOS).
    pub fn remediation(&self) -> &'static str {
        match self {
            FindingCode::TelnetEnabled => "line vty 0 4 ; transport input ssh",
            FindingCode::SshV1 => "ip ssh version 2",
            FindingCode::LinePasswordless => "line vty 0 4 ; login local",
            FindingCode::ExecTimeoutDisabled => "line vty 0 4 ; exec-timeout 10 0",
            FindingCode::NoLoginBanner => "banner login ^C Authorized access only ^C",
            FindingCode::NoEnableSecret => "enable secret <strong-password>",
            FindingCode::WeakEnableSecretType => "enable algorithm-type scrypt secret <password>",
            FindingCode::NoPasswordEncryption => "service password-encryption",
            FindingCode::WeakPassword => "use a long, unique, non-default password",
            FindingCode::Type7Password => "replace type-7 with 'secret' (type 8/9)",
            FindingCode::NoAaaNewModel => "aaa new-model",
            FindingCode::HttpServerEnabled => "no ip http server ; ip http secure-server",
            FindingCode::SnmpPublic => "no snmp-server community public ; use SNMPv3",
            FindingCode::SnmpPrivate => "no snmp-server community private ; use SNMPv3",
            FindingCode::SnmpRw => "avoid RW communities; use SNMPv3 with views",
            FindingCode::NoLogging => "logging host <syslog-ip>",
            FindingCode::NoNtpAuth => "ntp authenticate ; ntp authentication-key 1 md5 <key>",
        }
    }
}

/// Denetim özeti: skor, not ve seviye sayıları (profesyonel rapor başlığı).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuditSummary {
    /// Güvenlik duruşu skoru (0-100).
    pub score: u8,
    /// Harf notu (A-F).
    pub grade: &'static str,
    pub critical: usize,
    pub warning: usize,
    pub info: usize,
    pub total: usize,
}

/// Bir skora (0-100) karşılık gelen harf notu.
pub fn grade_for(score: u8) -> &'static str {
    match score {
        90..=100 => "A",
        80..=89 => "B",
        70..=79 => "C",
        60..=69 => "D",
        _ => "F",
    }
}

/// Bulgulardan bir güvenlik duruşu özeti üretir.
///
/// Skor 100'den başlar; her bulgu için ceza düşülür
/// (Kritik -15, Uyarı -7, Bilgi -2), 0'da taban yapar.
pub fn summarize(findings: &[Finding]) -> AuditSummary {
    let critical = findings
        .iter()
        .filter(|f| f.level == Level::Critical)
        .count();
    let warning = findings
        .iter()
        .filter(|f| f.level == Level::Warning)
        .count();
    let info = findings.iter().filter(|f| f.level == Level::Info).count();

    let penalty = (critical * 15 + warning * 7 + info * 2) as u32;
    let score = 100u32.saturating_sub(penalty) as u8;

    AuditSummary {
        score,
        grade: grade_for(score),
        critical,
        warning,
        info,
        total: findings.len(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kod_var(bulgular: &[Finding], code: FindingCode) -> bool {
        bulgular.iter().any(|b| b.code == code)
    }

    #[test]
    fn telnet_yakalanir() {
        let cfg = "line vty 0 4\n transport input telnet\n password gizli\n";
        assert!(kod_var(&audit(cfg), FindingCode::TelnetEnabled));
    }

    #[test]
    fn enable_password_secret_yoksa_uyarir() {
        let cfg = "enable password gizli123\n";
        assert!(kod_var(&audit(cfg), FindingCode::NoEnableSecret));
    }

    #[test]
    fn snmp_public_kritik() {
        assert!(kod_var(
            &audit("snmp-server community public RO\n"),
            FindingCode::SnmpPublic
        ));
    }

    #[test]
    fn snmp_rw_uyari() {
        assert!(kod_var(
            &audit("snmp-server community gizli123 RW\n"),
            FindingCode::SnmpRw
        ));
    }

    #[test]
    fn zayif_parola_yakalanir() {
        assert!(kod_var(
            &audit("username admin password cisco\n"),
            FindingCode::WeakPassword
        ));
    }

    #[test]
    fn ssh_v1_kritik() {
        assert!(kod_var(&audit("ip ssh version 1\n"), FindingCode::SshV1));
    }

    #[test]
    fn tip7_parola_uyari() {
        assert!(kod_var(
            &audit("enable password 7 08701E1D0A18\n"),
            FindingCode::Type7Password
        ));
    }

    #[test]
    fn parolasiz_vty_kritik() {
        // 'login' var ama 'password'/'login local' yok -> parolasız.
        let cfg = "line vty 0 4\n transport input ssh\n login\n!\n";
        assert!(kod_var(&audit(cfg), FindingCode::LinePasswordless));
    }

    #[test]
    fn parolali_vty_temiz() {
        let cfg = "line vty 0 4\n password guclU123\n login\n!\n";
        assert!(!kod_var(&audit(cfg), FindingCode::LinePasswordless));
    }

    #[test]
    fn ntp_auth_yoksa_uyari() {
        assert!(kod_var(
            &audit("ntp server 192.0.2.1\n"),
            FindingCode::NoNtpAuth
        ));
    }

    #[test]
    fn logging_yoksa_bilgi() {
        assert!(kod_var(&audit("hostname R1\n"), FindingCode::NoLogging));
    }

    #[test]
    fn exec_timeout_kapali_uyari() {
        let cfg = "line vty 0 4\n exec-timeout 0 0\n password guclU123\n";
        assert!(kod_var(&audit(cfg), FindingCode::ExecTimeoutDisabled));
    }

    #[test]
    fn aaa_yoksa_bilgi() {
        assert!(kod_var(&audit("hostname R1\n"), FindingCode::NoAaaNewModel));
    }

    #[test]
    fn banner_yoksa_bilgi() {
        assert!(kod_var(&audit("hostname R1\n"), FindingCode::NoLoginBanner));
    }

    #[test]
    fn banner_varsa_bilgi_yok() {
        let cfg = "banner motd ^C Yetkisiz erisim yasaktir ^C\n";
        assert!(!kod_var(&audit(cfg), FindingCode::NoLoginBanner));
    }

    #[test]
    fn zayif_enable_secret_tipi_bilgi() {
        assert!(kod_var(
            &audit("enable secret 5 $1$xY9$abcdef\n"),
            FindingCode::WeakEnableSecretType
        ));
    }

    #[test]
    fn guclu_enable_secret_tipi_temiz() {
        assert!(!kod_var(
            &audit("enable secret 9 $9$abcdef\n"),
            FindingCode::WeakEnableSecretType
        ));
    }

    #[test]
    fn sertlestirilmis_config_temiz() {
        let cfg = "\
hostname CORE1
aaa new-model
service password-encryption
enable secret 9 $9$xY9$abcdef
username netadmin secret 9 $9$zZ8$ghijkl
ip ssh version 2
logging host 192.0.2.5
ntp authenticate
banner login ^C Authorized access only ^C
line vty 0 4
 exec-timeout 10 0
 transport input ssh
 login local
";
        let b = audit(cfg);
        assert!(
            !b.iter().any(|x| x.level == Level::Critical),
            "kritik olmamalı: {b:?}"
        );
        // Hiçbir bulgu olmamalı: tam sertleştirilmiş.
        assert!(b.is_empty(), "temiz config'te bulgu olmamalı: {b:?}");
    }

    #[test]
    fn temiz_config_skor_100_grade_a() {
        let s = summarize(&[]);
        assert_eq!(s.score, 100);
        assert_eq!(s.grade, "A");
        assert_eq!(s.total, 0);
    }

    #[test]
    fn tek_kritik_ceza_dusurur() {
        let f = vec![Finding {
            level: Level::Critical,
            code: FindingCode::TelnetEnabled,
            line: None,
            detail: None,
        }];
        let s = summarize(&f);
        assert_eq!(s.score, 85); // 100 - 15
        assert_eq!(s.grade, "B");
        assert_eq!(s.critical, 1);
    }

    #[test]
    fn cok_bulgu_taban_sifir() {
        // 7 kritik * 15 = 105 -> taban 0, F.
        let f: Vec<Finding> = (0..7)
            .map(|_| Finding {
                level: Level::Critical,
                code: FindingCode::TelnetEnabled,
                line: None,
                detail: None,
            })
            .collect();
        let s = summarize(&f);
        assert_eq!(s.score, 0);
        assert_eq!(s.grade, "F");
    }

    #[test]
    fn kategori_eslemesi_dogru() {
        assert_eq!(FindingCode::TelnetEnabled.category(), Category::Access);
        assert_eq!(FindingCode::SnmpPublic.category(), Category::Snmp);
        assert_eq!(FindingCode::NoLogging.category(), Category::Logging);
        assert_eq!(FindingCode::WeakPassword.category(), Category::Authentication);
        assert_eq!(FindingCode::HttpServerEnabled.category(), Category::Services);
    }
}
