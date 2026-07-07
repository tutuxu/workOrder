//! 数据库 schema 增量迁移。

use rusqlite::Connection;

use crate::error::ServiceError;

fn column_exists(conn: &Connection, table: &str, column: &str) -> Result<bool, ServiceError> {
    let mut stmt = conn.prepare(&format!("PRAGMA table_info({table})"))?;
    let rows = stmt.query_map([], |row| row.get::<_, String>(1))?;
    for name in rows {
        if name? == column {
            return Ok(true);
        }
    }
    Ok(false)
}

/// 将旧版 progress_log 表迁移到含 title/status 的结构。
pub fn migrate_progress_log(conn: &Connection) -> Result<(), ServiceError> {
    if !column_exists(conn, "progress_log", "title")? {
        conn.execute(
            "ALTER TABLE progress_log ADD COLUMN title VARCHAR(255)",
            [],
        )?;
    }
    if !column_exists(conn, "progress_log", "status")? {
        conn.execute(
            "ALTER TABLE progress_log ADD COLUMN status VARCHAR(50) DEFAULT 'NOT_STARTED'",
            [],
        )?;
    }

    conn.execute(
        "UPDATE progress_log SET title = substr(content, 1, 255) WHERE title IS NULL OR title = ''",
        [],
    )?;
    conn.execute(
        "UPDATE progress_log SET status = 'IN_PROGRESS' WHERE status IS NULL OR status = ''",
        [],
    )?;

    Ok(())
}

/// 确保 attachment 表存在（存量库升级）。
pub fn migrate_attachment(conn: &Connection) -> Result<(), ServiceError> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS attachment (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            owner_type VARCHAR(50) NOT NULL,
            owner_id INTEGER NOT NULL,
            file_name VARCHAR(255) NOT NULL,
            original_name VARCHAR(255),
            mime_type VARCHAR(100) NOT NULL,
            file_size INTEGER NOT NULL,
            created_at TIMESTAMP NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_attachment_owner ON attachment(owner_type, owner_id);",
    )?;
    Ok(())
}
