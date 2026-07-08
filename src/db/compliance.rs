//! Uyum (compliance) skor geçmişi — zaman içindeki güvenlik duruşu trendi.
//!
//! Her denetim çalıştırması bir filo satırı + cihaz başına satırlar olarak
//! saklanır. Trend grafiği filo satırlarını (`scope = 'fleet'`) okur.

use rusqlite::{Connection, Result, params};

use crate::network::compliance::FleetPosture;

/// Trend grafiği için tek bir zaman noktası.
#[derive(Debug, Clone)]
pub struct ScorePoint {
    pub recorded_at: String,
    pub score: i64,
    pub critical: i64,
    pub warning: i64,
    pub info: i64,
}

/// Bir denetim çalıştırmasının sonuçlarını saklar: bir filo toplamı satırı
/// ve her cihaz için bir satır.
pub fn record_run(conn: &Connection, fleet: &FleetPosture) -> Result<()> {
    conn.execute(
        "INSERT INTO compliance_snapshots (scope, device_name, score, critical, warning, info)
         VALUES ('fleet', NULL, ?1, ?2, ?3, ?4)",
        params![
            fleet.average_score as i64,
            fleet.total_critical as i64,
            fleet.total_warning as i64,
            fleet.total_info as i64,
        ],
    )?;

    for d in &fleet.devices {
        conn.execute(
            "INSERT INTO compliance_snapshots (scope, device_name, score, critical, warning, info)
             VALUES ('device', ?1, ?2, ?3, ?4, ?5)",
            params![
                d.device,
                d.summary.score as i64,
                d.summary.critical as i64,
                d.summary.warning as i64,
                d.summary.info as i64,
            ],
        )?;
    }

    Ok(())
}

/// Filo skor geçmişini (en fazla `limit` kayıt) eskiden yeniye döndürür.
pub fn fleet_history(conn: &Connection, limit: i64) -> Result<Vec<ScorePoint>> {
    let mut stmt = conn.prepare(
        "SELECT recorded_at, score, critical, warning, info
         FROM compliance_snapshots
         WHERE scope = 'fleet'
         ORDER BY id DESC
         LIMIT ?1",
    )?;

    let rows = stmt.query_map(params![limit], |row| {
        Ok(ScorePoint {
            recorded_at: row.get(0)?,
            score: row.get(1)?,
            critical: row.get(2)?,
            warning: row.get(3)?,
            info: row.get(4)?,
        })
    })?;

    let mut points: Vec<ScorePoint> = Vec::new();
    for p in rows {
        points.push(p?);
    }
    // Sorgu yeniden eskiye; grafik için eskiden yeniye çevir.
    points.reverse();
    Ok(points)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::network::compliance::fleet_posture;
    use rusqlite::Connection;

    fn memory_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        crate::db::initialize_schema(&conn).unwrap();
        conn
    }

    #[test]
    fn record_and_read_fleet_history() {
        let conn = memory_db();

        let temiz = ("clean".to_string(), "hostname R1\naaa new-model\nservice password-encryption\nenable secret 9 $9$a\nbanner login ^C x ^C\nlogging host 1.1.1.1\n".to_string());
        let riskli = (
            "risky".to_string(),
            "line vty 0 4\n transport input telnet\n login\n".to_string(),
        );

        let f1 = fleet_posture(std::slice::from_ref(&temiz));
        record_run(&conn, &f1).unwrap();

        let f2 = fleet_posture(&[temiz, riskli]);
        record_run(&conn, &f2).unwrap();

        let history = fleet_history(&conn, 10).unwrap();
        assert_eq!(history.len(), 2);
        // İlk kayıt (tek temiz cihaz) ikinciden (riskli eklendi) daha yüksek skorlu.
        assert!(history[0].score >= history[1].score);
    }

    #[test]
    fn empty_history() {
        let conn = memory_db();
        assert!(fleet_history(&conn, 10).unwrap().is_empty());
    }
}
