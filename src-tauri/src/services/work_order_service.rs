//! 工单 CRUD、筛选排序与逾期判定等业务逻辑。

use std::collections::HashMap;

use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension};

use crate::db::datetime::{
    format_datetime, read_datetime_column, read_optional_datetime_column,
};
use crate::error::ServiceError;
use crate::models::status_config::StatusConfig;
use crate::models::work_order::{WorkOrder, WorkOrderInput};
use crate::services::status_config_service;

fn read_extra_fields(row: &rusqlite::Row<'_>) -> Result<Option<HashMap<String, String>>, rusqlite::Error> {
    let raw: Option<String> = row.get("extra_fields")?;
    let Some(text) = raw.filter(|s| !s.trim().is_empty()) else {
        return Ok(None);
    };
    let map: HashMap<String, String> = serde_json::from_str(&text).unwrap_or_default();
    if map.is_empty() {
        Ok(None)
    } else {
        Ok(Some(map))
    }
}

fn row_to_work_order(row: &rusqlite::Row<'_>) -> Result<WorkOrder, rusqlite::Error> {
    Ok(WorkOrder {
        id: Some(row.get("id")?),
        title: row.get("title")?,
        description: row.get("description")?,
        status: row.get("status")?,
        priority: row.get("priority")?,
        extra_fields: read_extra_fields(row)?,
        due_date: read_optional_datetime_column(row, "due_date")?,
        created_at: read_datetime_column(row, "created_at")?,
        updated_at: read_datetime_column(row, "updated_at")?,
    })
}

fn validate_title(title: &str) -> Result<(), ServiceError> {
    if title.trim().is_empty() {
        return Err(ServiceError::Validation("Title is required".into()));
    }
    Ok(())
}

fn normalize_extra_fields(
    extra_fields: Option<HashMap<String, String>>,
) -> Option<HashMap<String, String>> {
    let mut map = extra_fields.unwrap_or_default();
    map.retain(|_, v| !v.trim().is_empty());
    map = map
        .into_iter()
        .map(|(k, v)| (k, v.trim().to_string()))
        .collect();
    if map.is_empty() {
        None
    } else {
        Some(map)
    }
}

fn extra_fields_to_json(extra_fields: &Option<HashMap<String, String>>) -> Option<String> {
    extra_fields
        .as_ref()
        .filter(|m| !m.is_empty())
        .and_then(|m| serde_json::to_string(m).ok())
}

fn next_priority(conn: &Connection) -> Result<i32, ServiceError> {
    let max: Option<i32> = conn
        .query_row(
            "SELECT priority FROM work_order ORDER BY priority DESC LIMIT 1",
            [],
            |row| row.get(0),
        )
        .optional()?;
    Ok(max.map(|p| p + 1).unwrap_or(0))
}

fn validate_input(config: &StatusConfig, input: &WorkOrderInput) -> Result<(), ServiceError> {
    validate_title(&input.title)?;
    let fields = normalize_extra_fields(input.extra_fields.clone()).unwrap_or_default();
    status_config_service::validate_extra_fields(config, &input.status, &fields)?;
    Ok(())
}

const WORK_ORDER_SELECT: &str = "SELECT id, title, description, status, priority, extra_fields, due_date, created_at, updated_at FROM work_order";

/// 按 id 获取工单，不存在返回 [`ServiceError::NotFound`]。
pub fn get_required(conn: &Connection, id: i64) -> Result<WorkOrder, ServiceError> {
    conn.query_row(
        &format!("{WORK_ORDER_SELECT} WHERE id = ?1"),
        params![id],
        row_to_work_order,
    )
    .optional()?
    .ok_or_else(|| ServiceError::NotFound(format!("Work order not found: {id}")))
}

/// 创建工单并分配递增 priority。
pub fn create(
    conn: &Connection,
    input: WorkOrderInput,
    config: &StatusConfig,
) -> Result<WorkOrder, ServiceError> {
    validate_input(config, &input)?;
    let now = Utc::now().naive_utc();
    let priority = next_priority(conn)?;
    let extra_fields = normalize_extra_fields(input.extra_fields);
    conn.execute(
        "INSERT INTO work_order (title, description, status, priority, extra_fields, due_date, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            input.title.trim(),
            input.description,
            input.status,
            priority,
            extra_fields_to_json(&extra_fields),
            input.due_date.map(format_datetime),
            format_datetime(now),
            format_datetime(now),
        ],
    )?;
    let id = conn.last_insert_rowid();
    get_required(conn, id)
}

/// 更新工单字段。
pub fn update(
    conn: &Connection,
    id: i64,
    input: WorkOrderInput,
    config: &StatusConfig,
) -> Result<WorkOrder, ServiceError> {
    get_required(conn, id)?;
    validate_input(config, &input)?;
    let now = Utc::now().naive_utc();
    let extra_fields = normalize_extra_fields(input.extra_fields);
    conn.execute(
        "UPDATE work_order SET title = ?1, description = ?2, status = ?3, extra_fields = ?4, due_date = ?5, updated_at = ?6 WHERE id = ?7",
        params![
            input.title.trim(),
            input.description,
            input.status,
            extra_fields_to_json(&extra_fields),
            input.due_date.map(format_datetime),
            format_datetime(now),
            id,
        ],
    )?;
    get_required(conn, id)
}

/// 删除工单及其全部进度日志。
pub fn delete(conn: &Connection, id: i64) -> Result<(), ServiceError> {
    get_required(conn, id)?;
    conn.execute(
        "DELETE FROM progress_log WHERE work_order_id = ?1",
        params![id],
    )?;
    conn.execute("DELETE FROM work_order WHERE id = ?1", params![id])?;
    Ok(())
}

/// 按状态筛选工单；`statuses` 为空表示全部。
/// `query` 非空时在标题、描述、legacy 等待列与 extra_fields 中模糊匹配。
pub fn find_by_statuses(
    conn: &Connection,
    statuses: &[String],
    query: Option<&str>,
) -> Result<Vec<WorkOrder>, ServiceError> {
    let mut sql = WORK_ORDER_SELECT.to_string();
    let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

    if !statuses.is_empty() {
        let placeholders: Vec<String> = statuses
            .iter()
            .enumerate()
            .map(|(i, _)| format!("?{}", i + 1))
            .collect();
        sql.push_str(&format!(" WHERE status IN ({})", placeholders.join(", ")));
        for s in statuses {
            params.push(Box::new(s.clone()));
        }
    }

    if let Some(q) = query.map(str::trim).filter(|s| !s.is_empty()) {
        let pattern = format!("%{q}%");
        if statuses.is_empty() {
            sql.push_str(" WHERE ");
        } else {
            sql.push_str(" AND ");
        }
        let base = params.len();
        sql.push_str(&format!(
            "(title LIKE ?{} OR description LIKE ?{} OR waiting_for LIKE ?{} OR waiting_reason LIKE ?{} OR extra_fields LIKE ?{})",
            base + 1,
            base + 2,
            base + 3,
            base + 4,
            base + 5,
        ));
        for _ in 0..5 {
            params.push(Box::new(pattern.clone()));
        }
    }

    sql.push_str(" ORDER BY priority ASC, updated_at DESC");
    let param_refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(param_refs.as_slice(), row_to_work_order)?;
    let mut result = Vec::new();
    for row in rows {
        result.push(row?);
    }
    Ok(result)
}

/// 按 id 顺序批量重写 priority（0 起递增）。
pub fn update_priorities(conn: &Connection, ordered_ids: &[i64]) -> Result<(), ServiceError> {
    if ordered_ids.is_empty() {
        return Ok(());
    }
    for (i, id) in ordered_ids.iter().enumerate() {
        get_required(conn, *id)?;
        conn.execute(
            "UPDATE work_order SET priority = ?1 WHERE id = ?2",
            params![i as i32, id],
        )?;
    }
    Ok(())
}

/// 判断工单是否逾期（有截止日期且已过期）。
pub fn is_overdue(work_order: &WorkOrder) -> bool {
    if work_order.due_date.is_none() {
        return false;
    }
    work_order.due_date.unwrap() < Utc::now().naive_utc()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::connection::open_connection;
    use chrono::Duration;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_db() -> (Connection, std::path::PathBuf) {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("workorder-svc-{nanos}"));
        let conn = open_connection(&dir).unwrap();
        (conn, dir)
    }

    fn config() -> StatusConfig {
        StatusConfig::default_config()
    }

    fn input(title: &str) -> WorkOrderInput {
        WorkOrderInput {
            title: title.to_string(),
            description: None,
            status: "NOT_STARTED".into(),
            extra_fields: None,
            due_date: None,
        }
    }

    #[test]
    fn create_and_find_by_statuses() {
        let (conn, dir) = temp_db();
        let created = create(&conn, input("Task A"), &config()).unwrap();
        assert!(created.id.is_some());
        let list = find_by_statuses(&conn, &[], None).unwrap();
        assert_eq!(list.len(), 1);
        drop(conn);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn filter_by_status() {
        let (conn, dir) = temp_db();
        create(&conn, input("Open"), &config()).unwrap();
        let done = create(&conn, input("Done"), &config()).unwrap();
        update(
            &conn,
            done.id.unwrap(),
            WorkOrderInput {
                status: "COMPLETED".into(),
                ..input("Done")
            },
            &config(),
        )
        .unwrap();
        let all = find_by_statuses(&conn, &[], None).unwrap();
        assert_eq!(all.len(), 2);
        let only_done = find_by_statuses(&conn, &["COMPLETED".into()], None).unwrap();
        assert_eq!(only_done.len(), 1);
        drop(conn);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn update_priorities_reorders() {
        let (conn, dir) = temp_db();
        let first = create(&conn, input("First"), &config()).unwrap();
        let second = create(&conn, input("Second"), &config()).unwrap();
        update_priorities(&conn, &[second.id.unwrap(), first.id.unwrap()]).unwrap();
        let list = find_by_statuses(&conn, &[], None).unwrap();
        assert_eq!(list[0].id, second.id);
        drop(conn);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn is_overdue_when_past_due() {
        let (conn, dir) = temp_db();
        let wo = create(
            &conn,
            WorkOrderInput {
                due_date: Some(Utc::now().naive_utc() - Duration::hours(1)),
                ..input("Overdue")
            },
            &config(),
        )
        .unwrap();
        assert!(is_overdue(&wo));
        let completed = update(
            &conn,
            wo.id.unwrap(),
            WorkOrderInput {
                status: "COMPLETED".into(),
                due_date: wo.due_date,
                ..input("Overdue")
            },
            &config(),
        )
        .unwrap();
        assert!(is_overdue(&completed));
        drop(conn);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn required_extra_fields_enforced() {
        let (conn, dir) = temp_db();
        let err = create(
            &conn,
            WorkOrderInput {
                status: "WAITING_REPLY".into(),
                ..input("Waiting")
            },
            &config(),
        )
        .unwrap_err();
        assert!(matches!(err, ServiceError::Validation(_)));
        drop(conn);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn migrate_waiting_columns_to_extra_fields() {
        let (conn, dir) = temp_db();
        conn.execute(
            "INSERT INTO work_order (title, description, status, priority, waiting_for, waiting_reason, due_date, created_at, updated_at)
             VALUES ('Legacy', NULL, 'WAITING_REPLY', 0, '运维组', '排期', NULL, datetime('now'), datetime('now'))",
            [],
        )
        .unwrap();
        crate::db::migrate::migrate_extra_fields(&conn).unwrap();
        let list = find_by_statuses(&conn, &[], None).unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(
            list[0].extra_fields.as_ref().and_then(|m| m.get("waitingFor")),
            Some(&"运维组".to_string())
        );
        drop(conn);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn search_by_text_fields() {
        let (conn, dir) = temp_db();
        create(
            &conn,
            WorkOrderInput {
                title: "网络故障".into(),
                description: Some("交换机端口异常".into()),
                ..input("网络故障")
            },
            &config(),
        )
        .unwrap();
        create(
            &conn,
            WorkOrderInput {
                title: "其他任务".into(),
                ..input("其他任务")
            },
            &config(),
        )
        .unwrap();
        let mut fields = HashMap::new();
        fields.insert("waitingFor".into(), "运维组".into());
        fields.insert("waitingReason".into(), "等待排期".into());
        create(
            &conn,
            WorkOrderInput {
                title: "待确认".into(),
                status: "WAITING_REPLY".into(),
                extra_fields: Some(fields),
                ..input("待确认")
            },
            &config(),
        )
        .unwrap();

        let by_title = find_by_statuses(&conn, &[], Some("网络")).unwrap();
        assert_eq!(by_title.len(), 1);
        let by_waiting = find_by_statuses(&conn, &[], Some("运维组")).unwrap();
        assert_eq!(by_waiting.len(), 1);
        drop(conn);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn title_required() {
        let (conn, dir) = temp_db();
        let err = create(&conn, input(" "), &config()).unwrap_err();
        assert!(matches!(err, ServiceError::Validation(_)));
        drop(conn);
        let _ = std::fs::remove_dir_all(dir);
    }
}
