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

/// 将 legacy waiting 列迁移至 extra_fields JSON。
pub fn migrate_extra_fields(conn: &Connection) -> Result<(), ServiceError> {
    if !column_exists(conn, "work_order", "extra_fields")? {
        conn.execute("ALTER TABLE work_order ADD COLUMN extra_fields TEXT", [])?;
    }

    let mut stmt = conn.prepare(
        "SELECT id, waiting_for, waiting_reason, extra_fields FROM work_order",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok((
            row.get::<_, i64>(0)?,
            row.get::<_, Option<String>>(1)?,
            row.get::<_, Option<String>>(2)?,
            row.get::<_, Option<String>>(3)?,
        ))
    })?;

    for row in rows {
        let (id, waiting_for, waiting_reason, extra_fields) = row?;
        if extra_fields.as_deref().is_some_and(|s| !s.trim().is_empty()) {
            continue;
        }
        let wf = waiting_for
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty());
        let wr = waiting_reason
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty());
        if wf.is_none() && wr.is_none() {
            continue;
        }
        let mut map = serde_json::Map::new();
        if let Some(v) = wf {
            map.insert("waitingFor".into(), serde_json::Value::String(v.to_string()));
        }
        if let Some(v) = wr {
            map.insert(
                "waitingReason".into(),
                serde_json::Value::String(v.to_string()),
            );
        }
        let json = serde_json::Value::Object(map).to_string();
        conn.execute(
            "UPDATE work_order SET extra_fields = ?1 WHERE id = ?2",
            rusqlite::params![json, id],
        )?;
    }
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

fn table_exists(conn: &Connection, name: &str) -> Result<bool, ServiceError> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name = ?1",
        [name],
        |row| row.get(0),
    )?;
    Ok(count > 0)
}

pub fn migrate_work_order_tag(conn: &Connection) -> Result<(), ServiceError> {
    if table_exists(conn, "work_order_tag")? {
        return Ok(());
    }
    conn.execute_batch(
        "CREATE TABLE work_order_tag (
            work_order_id INTEGER NOT NULL REFERENCES work_order(id) ON DELETE CASCADE,
            tag_id          TEXT    NOT NULL,
            PRIMARY KEY (work_order_id, tag_id)
        );
        CREATE INDEX idx_work_order_tag_tag_id ON work_order_tag(tag_id);",
    )?;
    Ok(())
}

#[cfg(test)]
mod migrate_tag_tests {
    use crate::db::connection::open_connection;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_dir(prefix: &str) -> std::path::PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("{prefix}-{nanos}"))
    }

    #[test]
    fn creates_work_order_tag_table() {
        let dir = temp_dir("migrate-tag");
        let conn = open_connection(&dir).unwrap();
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='work_order_tag'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
        let _ = std::fs::remove_dir_all(dir);
    }
}

#[cfg(test)]
mod extra_fields_tests {
    use super::*;
    use crate::db::connection::open_connection;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_dir(prefix: &str) -> std::path::PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("{prefix}-{nanos}"))
    }

    #[test]
    fn migrates_waiting_columns() {
        let dir = temp_dir("migrate-extra");
        let conn = open_connection(&dir).unwrap();
        conn.execute(
            "INSERT INTO work_order (title, description, status, priority, waiting_for, waiting_reason, due_date, created_at, updated_at)
             VALUES ('Legacy', NULL, 'WAITING_REPLY', 0, '运维组', '排期', NULL, datetime('now'), datetime('now'))",
            [],
        )
        .unwrap();
        migrate_extra_fields(&conn).unwrap();
        let json: String = conn
            .query_row(
                "SELECT extra_fields FROM work_order WHERE title = 'Legacy'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert!(json.contains("waitingFor"));
        assert!(json.contains("运维组"));
        let _ = std::fs::remove_dir_all(dir);
    }
}
