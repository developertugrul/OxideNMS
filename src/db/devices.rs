use rusqlite::{Connection, Result, params};

#[derive(Debug, Clone)]
pub struct Device {
    pub id: i64,
    pub name: String,
    pub ip_address: String,
    pub created_at: String,
}

#[derive(Debug, Clone)]
pub struct ConfigHistory {
    pub id: i64,
    pub device_id: i64,
    pub config_text: String,
    pub recorded_at: String,
}

/// Yeni bir cihaz ekler.
pub fn add_device(conn: &Connection, name: &str, ip_address: &str) -> Result<i64> {
    conn.execute(
        "INSERT INTO devices (name, ip_address) VALUES (?1, ?2)",
        params![name, ip_address],
    )?;
    Ok(conn.last_insert_rowid())
}

/// IP adresine göre cihaz arar, yoksa ekler ve ID'sini döner.
pub fn get_or_create_device(conn: &Connection, name: &str, ip_address: &str) -> Result<i64> {
    let mut stmt = conn.prepare("SELECT id FROM devices WHERE ip_address = ?1")?;
    let mut rows = stmt.query(params![ip_address])?;
    if let Some(row) = rows.next()? {
        Ok(row.get(0)?)
    } else {
        add_device(conn, name, ip_address)
    }
}

/// Tüm cihazları listeler.
pub fn all_devices(conn: &Connection) -> Result<Vec<Device>> {
    let mut stmt = conn.prepare("SELECT id, name, ip_address, created_at FROM devices")?;
    let cihaz_iter = stmt.query_map([], |row| {
        Ok(Device {
            id: row.get(0)?,
            name: row.get(1)?,
            ip_address: row.get(2)?,
            created_at: row.get(3)?,
        })
    })?;

    let mut devices = Vec::new();
    for c in cihaz_iter {
        devices.push(c?);
    }
    Ok(devices)
}

/// Belirli bir cihaz için yeni bir config kaydeder.
pub fn save_config(conn: &Connection, device_id: i64, config_text: &str) -> Result<i64> {
    conn.execute(
        "INSERT INTO config_gecmisi (device_id, config_text) VALUES (?1, ?2)",
        params![device_id, config_text],
    )?;
    Ok(conn.last_insert_rowid())
}

/// Belirli bir cihaz icin en yeni N config kaydini tutar, eskileri siler.
pub fn prune_config_history(conn: &Connection, device_id: i64, keep_latest: i64) -> Result<usize> {
    if keep_latest <= 0 {
        return Ok(0);
    }

    conn.execute(
        "DELETE FROM config_gecmisi
         WHERE device_id = ?1
           AND id NOT IN (
               SELECT id FROM config_gecmisi
               WHERE device_id = ?1
               ORDER BY id DESC
               LIMIT ?2
           )",
        params![device_id, keep_latest],
    )
}

/// Belirli bir cihaza ait config geçmişini (yeniden eskiye) listeler.
pub fn get_config_history(conn: &Connection, device_id: i64) -> Result<Vec<ConfigHistory>> {
    let mut stmt = conn.prepare(
        "SELECT id, device_id, config_text, recorded_at 
         FROM config_gecmisi 
         WHERE device_id = ?1 
         ORDER BY id DESC",
    )?;

    let config_iter = stmt.query_map(params![device_id], |row| {
        Ok(ConfigHistory {
            id: row.get(0)?,
            device_id: row.get(1)?,
            config_text: row.get(2)?,
            recorded_at: row.get(3)?,
        })
    })?;

    let mut gecmis = Vec::new();
    for c in config_iter {
        gecmis.push(c?);
    }
    Ok(gecmis)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    fn memory_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        crate::db::initialize_schema(&conn).unwrap();
        conn
    }

    #[test]
    fn prune_config_history_keeps_latest_snapshots() {
        let conn = memory_db();
        let device_id = add_device(&conn, "R1", "192.0.2.10").unwrap();

        save_config(&conn, device_id, "version 1").unwrap();
        save_config(&conn, device_id, "version 2").unwrap();
        save_config(&conn, device_id, "version 3").unwrap();

        let deleted = prune_config_history(&conn, device_id, 2).unwrap();
        let history = get_config_history(&conn, device_id).unwrap();

        assert_eq!(deleted, 1);
        assert_eq!(history.len(), 2);
        assert_eq!(history[0].config_text, "version 3");
        assert_eq!(history[1].config_text, "version 2");
    }
}
