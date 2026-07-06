pub mod devices;
pub mod jobs;

use rusqlite::{Connection, OptionalExtension, Result, params};
use std::path::PathBuf;

const APP_DIR_NAME: &str = "OxideNMS";
const DB_FILE_NAME: &str = "oxidenms.db";
const LEGACY_DB_FILE_NAME: &str = "app.db";
const VAULT_CHECK_KEY: &str = "master_check";
const VAULT_CHECK_VALUE: &str = "oxidenms-vault-check-v1";

/// Runtime veritabaninin OS veri klasorundeki hedef yolu.
pub fn database_path() -> PathBuf {
    let base = dirs::data_dir()
        .or_else(dirs::config_dir)
        .unwrap_or_else(|| PathBuf::from("."))
        .join(APP_DIR_NAME);

    if let Err(e) = std::fs::create_dir_all(&base) {
        eprintln!("OxideNMS veri klasoru olusturulamadi ({e}); app.db kullaniliyor.");
        return PathBuf::from(LEGACY_DB_FILE_NAME);
    }

    let target = base.join(DB_FILE_NAME);
    let legacy = PathBuf::from(LEGACY_DB_FILE_NAME);
    if !target.exists()
        && legacy.exists()
        && let Err(e) = std::fs::copy(&legacy, &target)
    {
        eprintln!("Eski app.db yeni veri klasorune kopyalanamadi: {e}");
    }

    target
}

/// Veritabani baglantisini dondurur ve gerekli tablolari olusturur.
pub fn get_connection() -> Result<Connection> {
    let conn = Connection::open(database_path())?;
    initialize_schema(&conn)?;
    Ok(conn)
}

/// Veritabani semasini olusturur veya eksik alanlari tamamlar.
pub fn initialize_schema(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS devices (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            ip_address TEXT NOT NULL,
            username TEXT,
            encrypted_credentials TEXT,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )?;

    let _ = conn.execute("ALTER TABLE devices ADD COLUMN username TEXT", []);
    let _ = conn.execute(
        "ALTER TABLE devices ADD COLUMN encrypted_credentials TEXT",
        [],
    );
    let _ = conn.execute("ALTER TABLE devices ADD COLUMN platform TEXT", []);
    let _ = conn.execute("ALTER TABLE devices ADD COLUMN model TEXT", []);
    let _ = conn.execute("ALTER TABLE devices ADD COLUMN serial TEXT", []);
    let _ = conn.execute("ALTER TABLE devices ADD COLUMN ios_version TEXT", []);
    let _ = conn.execute("ALTER TABLE devices ADD COLUMN site TEXT", []);
    let _ = conn.execute("ALTER TABLE devices ADD COLUMN role TEXT", []);
    let _ = conn.execute("ALTER TABLE devices ADD COLUMN tags TEXT", []);
    let _ = conn.execute("ALTER TABLE devices ADD COLUMN last_seen DATETIME", []);

    conn.execute(
        "CREATE TABLE IF NOT EXISTS config_gecmisi (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            device_id INTEGER NOT NULL,
            config_text TEXT NOT NULL,
            recorded_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY(device_id) REFERENCES devices(id)
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS vault_metadata (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS audit_log (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            occurred_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            action TEXT NOT NULL,
            target TEXT NOT NULL,
            status TEXT NOT NULL,
            details TEXT
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS operation_jobs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            kind TEXT NOT NULL,
            target TEXT NOT NULL,
            status TEXT NOT NULL,
            queued_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            started_at DATETIME,
            finished_at DATETIME,
            attempts INTEGER NOT NULL DEFAULT 0,
            max_attempts INTEGER NOT NULL DEFAULT 1,
            details TEXT,
            last_error TEXT
        )",
        [],
    )?;

    Ok(())
}

/// Master password'u dogrular; ilk calistirmada vault check kaydini olusturur.
pub fn verify_or_initialize_vault(master_pass: &str) -> std::result::Result<(), String> {
    if master_pass.trim().is_empty() {
        return Err("Master password bos olamaz.".to_string());
    }

    let conn = get_connection().map_err(|e| format!("Veritabani acilamadi: {e}"))?;
    let existing: Option<String> = conn
        .query_row(
            "SELECT value FROM vault_metadata WHERE key = ?1",
            [VAULT_CHECK_KEY],
            |row| row.get(0),
        )
        .optional()
        .map_err(|e| format!("Vault metadata okunamadi: {e}"))?;

    if let Some(encrypted_check) = existing {
        let plain = crate::crypto::decrypt_credential(&encrypted_check, master_pass)?;
        if plain == VAULT_CHECK_VALUE {
            Ok(())
        } else {
            Err("Master password dogrulanamadi.".to_string())
        }
    } else {
        let encrypted_check = crate::crypto::encrypt_credential(VAULT_CHECK_VALUE, master_pass)?;
        conn.execute(
            "INSERT INTO vault_metadata (key, value) VALUES (?1, ?2)",
            params![VAULT_CHECK_KEY, encrypted_check],
        )
        .map_err(|e| format!("Vault metadata yazilamadi: {e}"))?;
        record_audit(
            "vault.initialize",
            "local",
            "success",
            "Master vault initialized",
        );
        Ok(())
    }
}

/// Operasyon audit kaydi yazar. Audit hatasi ana operasyonu bozmaz.
pub fn record_audit(action: &str, target: &str, status: &str, details: &str) {
    if let Ok(conn) = get_connection() {
        let _ = conn.execute(
            "INSERT INTO audit_log (action, target, status, details) VALUES (?1, ?2, ?3, ?4)",
            params![action, target, status, details],
        );
    }
}
