use rusqlite::{Connection, Result, Row, params};

pub const STATUS_QUEUED: &str = "queued";
pub const STATUS_RUNNING: &str = "running";
pub const STATUS_SUCCEEDED: &str = "succeeded";
pub const STATUS_FAILED: &str = "failed";
pub const STATUS_SKIPPED: &str = "skipped";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OperationJob {
    pub id: i64,
    pub kind: String,
    pub target: String,
    pub status: String,
    pub queued_at: String,
    pub started_at: Option<String>,
    pub finished_at: Option<String>,
    pub attempts: i64,
    pub max_attempts: i64,
    pub details: Option<String>,
    pub last_error: Option<String>,
}

pub fn enqueue(
    conn: &Connection,
    kind: &str,
    target: &str,
    max_attempts: i64,
    details: &str,
) -> Result<i64> {
    conn.execute(
        "INSERT INTO operation_jobs (kind, target, status, max_attempts, details)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            kind,
            target,
            STATUS_QUEUED,
            max_attempts.max(1),
            empty_to_none(details)
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn mark_running(conn: &Connection, job_id: i64) -> Result<()> {
    conn.execute(
        "UPDATE operation_jobs
         SET status = ?1,
             started_at = COALESCE(started_at, CURRENT_TIMESTAMP),
             attempts = attempts + 1,
             last_error = NULL
         WHERE id = ?2",
        params![STATUS_RUNNING, job_id],
    )?;
    Ok(())
}

pub fn mark_succeeded(conn: &Connection, job_id: i64, details: &str) -> Result<()> {
    conn.execute(
        "UPDATE operation_jobs
         SET status = ?1,
             finished_at = CURRENT_TIMESTAMP,
             details = COALESCE(?2, details),
             last_error = NULL
         WHERE id = ?3",
        params![STATUS_SUCCEEDED, empty_to_none(details), job_id],
    )?;
    Ok(())
}

pub fn mark_failed(conn: &Connection, job_id: i64, error: &str, details: &str) -> Result<()> {
    conn.execute(
        "UPDATE operation_jobs
         SET status = ?1,
             finished_at = CURRENT_TIMESTAMP,
             details = COALESCE(?2, details),
             last_error = ?3
         WHERE id = ?4",
        params![STATUS_FAILED, empty_to_none(details), error, job_id],
    )?;
    Ok(())
}

pub fn mark_skipped(conn: &Connection, job_id: i64, details: &str) -> Result<()> {
    conn.execute(
        "UPDATE operation_jobs
         SET status = ?1,
             finished_at = CURRENT_TIMESTAMP,
             details = COALESCE(?2, details)
         WHERE id = ?3",
        params![STATUS_SKIPPED, empty_to_none(details), job_id],
    )?;
    Ok(())
}

pub fn recent(conn: &Connection, limit: i64) -> Result<Vec<OperationJob>> {
    let mut stmt = conn.prepare(
        "SELECT id, kind, target, status, queued_at, started_at, finished_at,
                attempts, max_attempts, details, last_error
         FROM operation_jobs
         ORDER BY id DESC
         LIMIT ?1",
    )?;

    let rows = stmt.query_map(params![limit.max(1)], map_job)?;
    rows.collect()
}

pub fn recent_by_kind(conn: &Connection, kind: &str, limit: i64) -> Result<Vec<OperationJob>> {
    let mut stmt = conn.prepare(
        "SELECT id, kind, target, status, queued_at, started_at, finished_at,
                attempts, max_attempts, details, last_error
         FROM operation_jobs
         WHERE kind = ?1
         ORDER BY id DESC
         LIMIT ?2",
    )?;

    let rows = stmt.query_map(params![kind, limit.max(1)], map_job)?;
    rows.collect()
}

fn map_job(row: &Row<'_>) -> Result<OperationJob> {
    Ok(OperationJob {
        id: row.get(0)?,
        kind: row.get(1)?,
        target: row.get(2)?,
        status: row.get(3)?,
        queued_at: row.get(4)?,
        started_at: row.get(5)?,
        finished_at: row.get(6)?,
        attempts: row.get(7)?,
        max_attempts: row.get(8)?,
        details: row.get(9)?,
        last_error: row.get(10)?,
    })
}

fn empty_to_none(value: &str) -> Option<&str> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed)
    }
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
    fn operation_job_lifecycle_persists_status() {
        let conn = memory_db();

        let id = enqueue(&conn, "backup.running_config", "192.0.2.10", 2, "R1").unwrap();
        mark_running(&conn, id).unwrap();
        mark_succeeded(&conn, id, "Saved config snapshot").unwrap();

        let jobs = recent(&conn, 10).unwrap();
        assert_eq!(jobs.len(), 1);
        assert_eq!(jobs[0].status, STATUS_SUCCEEDED);
        assert_eq!(jobs[0].attempts, 1);
        assert!(jobs[0].started_at.is_some());
        assert!(jobs[0].finished_at.is_some());
        assert_eq!(jobs[0].details.as_deref(), Some("Saved config snapshot"));
    }

    #[test]
    fn recent_by_kind_filters_operation_jobs() {
        let conn = memory_db();

        let backup_id = enqueue(&conn, "backup.running_config", "192.0.2.10", 1, "").unwrap();
        let deploy_id = enqueue(&conn, "bulk_deploy.execute", "192.0.2.20", 1, "").unwrap();
        mark_failed(&conn, backup_id, "SSH connection failed", "").unwrap();
        mark_skipped(&conn, deploy_id, "Dry run only").unwrap();

        let backups = recent_by_kind(&conn, "backup.running_config", 10).unwrap();
        assert_eq!(backups.len(), 1);
        assert_eq!(backups[0].status, STATUS_FAILED);
        assert_eq!(
            backups[0].last_error.as_deref(),
            Some("SSH connection failed")
        );
    }
}
