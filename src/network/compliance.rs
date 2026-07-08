//! Filo geneli uyum (compliance) posture.
//!
//! Birden çok cihazın config metnini güvenlik denetiminden geçirip cihaz
//! başına skor ve filo geneli özet üretir. Saf mantık: I/O yok, DB/GUI bilmez.
//! (Config'leri okumak GUI/DB katmanının işi; buraya hazır metin gelir.)

use crate::network::security::{self, AuditSummary};

/// Tek bir cihazın uyum durumu.
#[derive(Debug, Clone)]
pub struct DevicePosture {
    pub device: String,
    pub summary: AuditSummary,
}

/// Filo geneli uyum özeti.
#[derive(Debug, Clone)]
pub struct FleetPosture {
    pub devices: Vec<DevicePosture>,
    /// Cihaz skorlarının ortalaması (0-100).
    pub average_score: u8,
    pub total_critical: usize,
    pub total_warning: usize,
    pub total_info: usize,
}

impl FleetPosture {
    pub fn device_count(&self) -> usize {
        self.devices.len()
    }

    /// Ortalama skora karşılık gelen filo notu.
    pub fn grade(&self) -> &'static str {
        security::grade_for(self.average_score)
    }

    /// En düşük skorlu cihazlar önce (en riskliyi başa alır).
    pub fn sorted_by_risk(&self) -> Vec<&DevicePosture> {
        let mut v: Vec<&DevicePosture> = self.devices.iter().collect();
        v.sort_by_key(|d| d.summary.score);
        v
    }
}

/// (cihaz adı, config metni) listesinden filo posture üretir.
pub fn fleet_posture(configs: &[(String, String)]) -> FleetPosture {
    let mut devices = Vec::new();
    let mut total_critical = 0;
    let mut total_warning = 0;
    let mut total_info = 0;
    let mut score_sum: u32 = 0;

    for (name, config) in configs {
        let findings = security::audit(config);
        let summary = security::summarize(&findings);
        score_sum += summary.score as u32;
        total_critical += summary.critical;
        total_warning += summary.warning;
        total_info += summary.info;
        devices.push(DevicePosture {
            device: name.clone(),
            summary,
        });
    }

    let average_score = if devices.is_empty() {
        100
    } else {
        (score_sum / devices.len() as u32) as u8
    };

    FleetPosture {
        devices,
        average_score,
        total_critical,
        total_warning,
        total_info,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bos_filo_ortalama_100() {
        let f = fleet_posture(&[]);
        assert_eq!(f.average_score, 100);
        assert_eq!(f.device_count(), 0);
        assert_eq!(f.grade(), "A");
    }

    #[test]
    fn karisik_filo_ortalama_ve_toplam() {
        let temiz = "\
hostname CLEAN
aaa new-model
service password-encryption
enable secret 9 $9$abc
ip ssh version 2
logging host 192.0.2.5
banner login ^C hi ^C
line vty 0 4
 exec-timeout 10 0
 transport input ssh
 login local
"
        .to_string();
        let riskli = "line vty 0 4\n transport input telnet\n login\n".to_string();

        let f = fleet_posture(&[
            ("clean-sw".to_string(), temiz),
            ("risky-sw".to_string(), riskli),
        ]);

        assert_eq!(f.device_count(), 2);
        // Temiz cihaz 100, riskli cihaz < 100 -> ortalama < 100.
        assert!(f.average_score < 100);
        assert!(f.total_critical >= 1); // telnet + parolasız vty
        // En riskli cihaz başta.
        assert_eq!(f.sorted_by_risk()[0].device, "risky-sw");
    }
}
