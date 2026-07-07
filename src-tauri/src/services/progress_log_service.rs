//! 进度日志 CRUD。

use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension};

use crate::db::datetime::{format_datetime, read_datetime_column};
use crate::error::ServiceError;
use crate::models::progress_log::{ProgressLog, ProgressLogInput};
use crate::models::work_order_status::WorkOrderStatus;
use crate::services::work_order_service::get_required;

fn row_to_progress_log(row: &rusqlite::Row<'_>) -> Result<ProgressLog, rusqlite::Error> {
    let status_str: String = row.get("status")?;
    let status = WorkOrderStatus::from_db_str(&status_str).unwrap_or(WorkOrderStatus::NotStarted);
    Ok(ProgressLog {
        id: Some(row.get("id")?),
        work_order_id: row.get("work_order_id")?,
        title: row.get("title")?,
        content: row.get("content")?,
        status,
        created_at: read_datetime_column(row, "created_at")?,
    })
}

fn get_required_log(
    conn: &Connection,
    log_id: i64,
    work_order_id: i64,
) -> Result<ProgressLog, ServiceError> {
    let log = conn
        .query_row(
            "SELECT id, work_order_id, title, content, status, created_at FROM progress_log WHERE id = ?1",
            params![log_id],
            row_to_progress_log,
        )
        .optional()?
        .ok_or_else(|| ServiceError::NotFound(format!("Progress log not found: {log_id}")))?;
    if log.work_order_id != work_order_id {
        return Err(ServiceError::Validation(format!(
            "Progress log does not belong to work order: {work_order_id}"
        )));
    }
    Ok(log)
}

/// 列出工单下全部进度日志，按 created_at 降序。
pub fn find_by_work_order_id(
    conn: &Connection,
    work_order_id: i64,
) -> Result<Vec<ProgressLog>, ServiceError> {
    let mut stmt = conn.prepare(
        "SELECT id, work_order_id, title, content, status, created_at FROM progress_log WHERE work_order_id = ?1 ORDER BY created_at DESC",
    )?;
    let rows = stmt.query_map(params![work_order_id], row_to_progress_log)?;
    let mut result = Vec::new();
    for row in rows {
        result.push(row?);
    }
    Ok(result)
}

/// 添加工单进度日志，title 不能为空。
pub fn add_log(
    conn: &Connection,
    work_order_id: i64,
    input: &ProgressLogInput,
) -> Result<ProgressLog, ServiceError> {
    if input.title.trim().is_empty() {
        return Err(ServiceError::Validation("Progress title is required".into()));
    }
    get_required(conn, work_order_id)?;
    let now = Utc::now().naive_utc();
    let content = input
        .content
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty());
    conn.execute(
        "INSERT INTO progress_log (work_order_id, title, content, status, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            work_order_id,
            input.title.trim(),
            content,
            input.status.as_db_str(),
            format_datetime(now),
        ],
    )?;
    let id = conn.last_insert_rowid();
    get_required_log(conn, id, work_order_id)
}

/// 更新进度日志，并校验归属工单。
pub fn update_log(
    conn: &Connection,
    log_id: i64,
    work_order_id: i64,
    input: &ProgressLogInput,
) -> Result<ProgressLog, ServiceError> {
    if input.title.trim().is_empty() {
        return Err(ServiceError::Validation("Progress title is required".into()));
    }
    get_required_log(conn, log_id, work_order_id)?;
    let content = input
        .content
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty());
    conn.execute(
        "UPDATE progress_log SET title = ?1, content = ?2, status = ?3 WHERE id = ?4",
        params![
            input.title.trim(),
            content,
            input.status.as_db_str(),
            log_id,
        ],
    )?;
    get_required_log(conn, log_id, work_order_id)
}

/// 删除进度日志，并校验归属工单。
pub fn delete_log(
    conn: &Connection,
    log_id: i64,
    work_order_id: i64,
) -> Result<(), ServiceError> {
    get_required_log(conn, log_id, work_order_id)?;
    conn.execute("DELETE FROM progress_log WHERE id = ?1", params![log_id])?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::connection::open_connection;
    use crate::models::work_order::WorkOrderInput;
    use crate::models::work_order_status::WorkOrderStatus;
    use crate::services::work_order_service::create;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_db() -> (Connection, std::path::PathBuf) {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("workorder-plog-{nanos}"));
        let conn = open_connection(&dir).unwrap();
        (conn, dir)
    }

    fn sample_input(title: &str) -> ProgressLogInput {
        ProgressLogInput {
            title: title.into(),
            content: Some("Detailed notes".into()),
            status: WorkOrderStatus::InProgress,
        }
    }

    #[test]
    fn add_update_delete_log() {
        let (conn, dir) = temp_db();
        let wo = create(
            &conn,
            WorkOrderInput {
                title: "With log".into(),
                description: None,
                status: WorkOrderStatus::NotStarted,
                waiting_for: None,
                waiting_reason: None,
                due_date: None,
            },
        )
        .unwrap();
        let log = add_log(&conn, wo.id.unwrap(), &sample_input("Started investigation")).unwrap();
        assert_eq!(find_by_work_order_id(&conn, wo.id.unwrap()).unwrap().len(), 1);
        let updated = update_log(
            &conn,
            log.id.unwrap(),
            wo.id.unwrap(),
            &ProgressLogInput {
                title: "Updated title".into(),
                content: Some("Updated note".into()),
                status: WorkOrderStatus::Completed,
            },
        )
        .unwrap();
        assert_eq!(updated.title, "Updated title");
        assert_eq!(updated.content.as_deref(), Some("Updated note"));
        assert_eq!(updated.status, WorkOrderStatus::Completed);
        delete_log(&conn, log.id.unwrap(), wo.id.unwrap()).unwrap();
        assert!(find_by_work_order_id(&conn, wo.id.unwrap()).unwrap().is_empty());
        drop(conn);
        let _ = std::fs::remove_dir_all(dir);
    }
}
