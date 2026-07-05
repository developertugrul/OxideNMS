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
    fn sertlestirilmis_config_temiz() {
        let cfg = "\
hostname CORE1
service password-encryption
enable secret 5 $1$xY9$abcdef
username netadmin secret 5 $1$zZ8$ghijkl
ip ssh version 2
logging host 192.0.2.5
line vty 0 4
 transport input ssh
 login local
";
        let b = audit(cfg);
        assert!(
            !b.iter().any(|x| x.level == Level::Critical),
            "kritik olmamalı: {b:?}"
        );
        assert!(!kod_var(&b, FindingCode::NoEnableSecret));
        assert!(!kod_var(&b, FindingCode::NoPasswordEncryption));
        assert!(!kod_var(&b, FindingCode::LinePasswordless));
        assert!(!kod_var(&b, FindingCode::NoLogging));
    }
}
