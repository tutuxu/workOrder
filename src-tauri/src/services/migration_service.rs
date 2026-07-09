//! 数据目录统一迁移（v1.0 → 当前版本及后续升级）。

use std::path::{Path, PathBuf};

use rusqlite::Connection;

use crate::db::connection::open_connection;
use crate::error::ServiceError;
use crate::services::{status_config_service, tag_config_service};

/// 一次迁移执行的结果摘要。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MigrationSummary {
    pub data_dir: PathBuf,
    /// 迁移后含 extra_fields 数据的工单行数
    pub extra_fields_rows: i64,
    /// 本次是否新创建了 status_config.json（v1.0 升级时为 true）
    pub status_config_created: bool,
    /// 本次是否新创建了 tag_config.json
    pub tag_config_created: bool,
}

/// 执行全部数据迁移：DB schema/列升级 + 默认状态配置补全。
/// 与正常启动时 `open_connection` + `ensure_default_config` 行为一致，可重复执行（幂等）。
pub fn run_data_migrations(data_dir: &Path) -> Result<MigrationSummary, ServiceError> {
    let status_config_existed = data_dir.join(status_config_service::CONFIG_FILE_NAME).exists();
    let tag_config_existed = data_dir.join(tag_config_service::CONFIG_FILE_NAME).exists();
    let conn = open_connection(data_dir)?;
    let extra_fields_rows = count_extra_fields_rows(&conn)?;
    status_config_service::ensure_default_config(data_dir)?;
    tag_config_service::ensure_default_config(data_dir)?;
    Ok(MigrationSummary {
        data_dir: data_dir.to_path_buf(),
        extra_fields_rows,
        status_config_created: !status_config_existed,
        tag_config_created: !tag_config_existed,
    })
}

fn count_extra_fields_rows(conn: &Connection) -> Result<i64, ServiceError> {
    conn.query_row(
        "SELECT COUNT(*) FROM work_order WHERE extra_fields IS NOT NULL AND trim(extra_fields) != ''",
        [],
        |row| row.get(0),
    )
    .map_err(Into::into)
}

impl MigrationSummary {
    pub fn format_report(&self) -> String {
        format!(
            "数据目录: {}\n\
             含扩展字段的工单: {} 条\n\
             状态配置: {}\n\
             标签配置: {}",
            self.data_dir.display(),
            self.extra_fields_rows,
            if self.status_config_created {
                "已生成 status_config.json（从 v1.0 默认四状态迁移）"
            } else {
                "已存在，未覆盖"
            },
            if self.tag_config_created {
                "已生成 tag_config.json（默认标签一、标签二）"
            } else {
                "已存在，未覆盖"
            }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::migrate;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_dir(prefix: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("{prefix}-{nanos}"))
    }

    #[test]
    fn v1_style_data_upgrades() {
        let dir = temp_dir("v1-upgrade");
        std::fs::create_dir_all(&dir).unwrap();
        let conn = open_connection(&dir).unwrap();
        conn.execute(
            "INSERT INTO work_order (title, description, status, priority, waiting_for, waiting_reason, due_date, created_at, updated_at)
             VALUES ('待确认', NULL, 'WAITING_REPLY', 0, '联调方', '等接口', NULL, datetime('now'), datetime('now'))",
            [],
        )
        .unwrap();
        drop(conn);

        let summary = run_data_migrations(&dir).unwrap();
        assert!(summary.status_config_created);
        assert!(summary.extra_fields_rows >= 1);
        assert!(dir.join("status_config.json").exists());

        let conn = open_connection(&dir).unwrap();
        migrate::migrate_extra_fields(&conn).unwrap();
        let json: String = conn
            .query_row(
                "SELECT extra_fields FROM work_order WHERE title = '待确认'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert!(json.contains("waitingFor"));
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn idempotent_second_run() {
        let dir = temp_dir("v1-upgrade-idempotent");
        run_data_migrations(&dir).unwrap();
        let second = run_data_migrations(&dir).unwrap();
        assert!(!second.status_config_created);
        let _ = std::fs::remove_dir_all(dir);
    }
}
