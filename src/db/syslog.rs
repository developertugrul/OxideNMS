use rusqlite::{Connection, Result, Row, params};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyslogEvent {
    pub id: i64,
    pub received_at: String,
    pub source_ip: String,
    pub severity: String,
    pub message: String,
    pub raw_message: String,
}

pub fn save_event(
    conn: &Connection,
    source_ip: &str,
    severity: &str,
    message: &str,
    raw_message: &str,
) -> Result<i64> {
    conn.execute(
        "INSERT INTO syslog_events (source_ip, severity, message, raw_message)
         VALUES (?1, ?2, ?3, ?4)",
        params![source_ip, severity, message, raw_message],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn search(
    conn: &Connection,
    source_filter: &str,
    severity_filter: &str,
    text_filter: &str,
    limit: i64,
) -> Result<Vec<SyslogEvent>> {
    let mut stmt = conn.prepare(
        "SELECT id, received_at, source_ip, severity, message, raw_message
         FROM syslog_events
         WHERE (?1 = '' OR source_ip LIKE '%' || ?1 || '%')
           AND (?2 = '' OR severity = ?2)
           AND (
                ?3 = ''
                OR message LIKE '%' || ?3 || '%'
                OR raw_message LIKE '%' || ?3 || '%'
           )
         ORDER BY id DESC
         LIMIT ?4",
    )?;

    let rows = stmt.query_map(
        params![
            source_filter.trim(),
            severity_filter.trim(),
            text_filter.trim(),
            limit.max(1)
        ],
        map_event,
    )?;
    rows.collect()
}

fn map_event(row: &Row<'_>) -> Result<SyslogEvent> {
    Ok(SyslogEvent {
        id: row.get(0)?,
        received_at: row.get(1)?,
        source_ip: row.get(2)?,
        severity: row.get(3)?,
        message: row.get(4)?,
        raw_message: row.get(5)?,
    })
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
    fn syslog_events_are_persisted_and_searchable() {
        let conn = memory_db();

        save_event(
            &conn,
            "192.0.2.10",
            "ERR",
            "%LINK-3-UPDOWN: Interface GigabitEthernet1 changed state to down",
            "<179>%LINK-3-UPDOWN: Interface GigabitEthernet1 changed state to down",
        )
        .unwrap();
        save_event(
            &conn,
            "192.0.2.20",
            "INFO",
            "Configured from console",
            "<190>Configured",
        )
        .unwrap();

        let events = search(&conn, "192.0.2.10", "ERR", "UPDOWN", 10).unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].source_ip, "192.0.2.10");
        assert_eq!(events[0].severity, "ERR");
    }
}
