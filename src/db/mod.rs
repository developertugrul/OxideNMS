pub mod devices;

use rusqlite::{Connection, Result};
use std::path::PathBuf;

/// Veritabanı bağlantısını döndürür ve gerekli tabloları oluşturur.
pub fn get_connection() -> Result<Connection> {
    // Şimdilik uygulamanın çalıştığı dizinde app.db kullanalım.
    let path = PathBuf::from("app.db");
    let conn = Connection::open(path)?;

    // Tabloları oluştur
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

    // Geçmiş veritabanı sürümleri için basit migrasyon
    let _ = conn.execute("ALTER TABLE devices ADD COLUMN username TEXT", []);
    let _ = conn.execute("ALTER TABLE devices ADD COLUMN encrypted_credentials TEXT", []);

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

    Ok(conn)
}
