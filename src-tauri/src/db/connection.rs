//! SQLite 连接与数据目录解析。

use std::fs;
use std::path::{Path, PathBuf};

use rusqlite::Connection;

use crate::error::ServiceError;

const SCHEMA_SQL: &str = include_str!("schema.sql");

/// 解析数据目录：环境变量 > 配置 > 向上查找项目根 data/ > 可执行文件旁 data/
pub fn resolve_data_dir(config_dir: Option<&str>) -> PathBuf {
    if let Ok(dir) = std::env::var("WORKORDER_DATA_DIR") {
        return PathBuf::from(dir);
    }
    if let Some(dir) = config_dir {
        if !dir.is_empty() {
            return PathBuf::from(dir);
        }
    }

    if let Some(dir) = find_data_dir_upward(&std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))) {
        return dir;
    }

    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            if let Some(dir) = find_data_dir_upward(parent) {
                return dir;
            }
            return parent.join("data");
        }
    }

    std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("data")
}

/// 从给定目录向上查找含 workorder.db 或已存在的 data/ 目录。
fn find_data_dir_upward(start: &Path) -> Option<PathBuf> {
    let mut dir = start.to_path_buf();
    for _ in 0..8 {
        let candidate = dir.join("data");
        if candidate.join("workorder.db").exists() || candidate.is_dir() {
            return Some(candidate);
        }
        if !dir.pop() {
            break;
        }
    }
    None
}

/// 打开数据库连接并初始化表结构。
pub fn open_connection(data_dir: &Path) -> Result<Connection, ServiceError> {
    fs::create_dir_all(data_dir)?;
    let db_path = data_dir.join("workorder.db");
    let conn = Connection::open(&db_path)?;
    conn.execute_batch(SCHEMA_SQL)?;
    Ok(conn)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn opens_in_memory_compatible_schema() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(SCHEMA_SQL).unwrap();
    }

    #[test]
    fn opens_temp_file_database() {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("workorder-test-{nanos}"));
        let conn = open_connection(&dir).unwrap();
        drop(conn);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn resolves_project_data_from_src_tauri_cwd() {
        let src_tauri = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let resolved = find_data_dir_upward(&src_tauri).expect("project data dir");
        assert!(resolved.ends_with("data"));
    }
}
