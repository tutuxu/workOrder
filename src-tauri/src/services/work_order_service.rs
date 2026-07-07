//! 工单 CRUD、筛选排序与逾期判定等业务逻辑。

use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension};

use crate::db::datetime::{
    format_datetime, read_datetime_column, read_optional_datetime_column,
};
use crate::error::ServiceError;
use crate::models::work_order::{WorkOrder, WorkOrderInput};
use crate::models::work_order_status::WorkOrderStatus;

fn row_to_work_order(row: &rusqlite::Row<'_>) -> Result<WorkOrder, rusqlite::Error> {
    let status_str: String = row.get("status")?;
    let status = WorkOrderStatus::from_db_str(&status_str).unwrap_or(WorkOrderStatus::NotStarted);

    Ok(WorkOrder {
        id: Some(row.get("id")?),
        title: row.get("title")?,
        description: row.get("description")?,
        status,
        priority: row.get("priority")?,
        waiting_for: row.get("waiting_for")?,
        waiting_reason: row.get("waiting_reason")?,
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

fn normalize_text(value: Option<&str>) -> Option<String> {
    value.map(str::trim).filter(|s| !s.is_empty()).map(str::to_string)
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

fn format_waiting_reply_log(waiting_for: Option<&str>, waiting_reason: Option<&str>) -> String {
    let mut content = WorkOrderStatus::WaitingReply.display_name().to_string();
    let wf = normalize_text(waiting_for);
    let wr = normalize_text(waiting_reason);
    if let Some(w) = &wf {
        content.push_str("：等待 ");
        content.push_str(w);
    }
    if let Some(r) = &wr {
        if wf.is_some() {
            content.push('，');
        } else {
            content.push('：');
        }
        content.push_str("原因 ");
        content.push_str(r);
    }
    content
}

fn insert_progress_log(
    conn: &Connection,
    work_order_id: i64,
    content: &str,
) -> Result<(), ServiceError> {
    let now = Utc::now().naive_utc();
    conn.execute(
        "INSERT INTO progress_log (work_order_id, content, created_at) VALUES (?1, ?2, ?3)",
        params![work_order_id, content, format_datetime(now)],
    )?;
    Ok(())
}

fn append_waiting_reply_progress_log(
    conn: &Connection,
    before: Option<&WorkOrder>,
    after: &WorkOrder,
) -> Result<(), ServiceError> {
    if after.status != WorkOrderStatus::WaitingReply {
        return Ok(());
    }
    let entered_waiting = before.is_none_or(|b| b.status != WorkOrderStatus::WaitingReply);
    let waiting_info_changed = before.is_some_and(|b| {
        b.status == WorkOrderStatus::WaitingReply
            && (normalize_text(b.waiting_for.as_deref())
                != normalize_text(after.waiting_for.as_deref())
                || normalize_text(b.waiting_reason.as_deref())
                    != normalize_text(after.waiting_reason.as_deref()))
    });
    if !entered_waiting && !waiting_info_changed {
        return Ok(());
    }
    let id = after.id.ok_or_else(|| ServiceError::Validation("missing work order id".into()))?;
    let content = format_waiting_reply_log(
        after.waiting_for.as_deref(),
        after.waiting_reason.as_deref(),
    );
    insert_progress_log(conn, id, &content)
}

/// 按 id 获取工单，不存在返回 [`ServiceError::NotFound`]。
pub fn get_required(conn: &Connection, id: i64) -> Result<WorkOrder, ServiceError> {
    conn.query_row(
        "SELECT id, title, description, status, priority, waiting_for, waiting_reason, due_date, created_at, updated_at FROM work_order WHERE id = ?1",
        params![id],
        row_to_work_order,
    )
    .optional()?
    .ok_or_else(|| ServiceError::NotFound(format!("Work order not found: {id}")))
}

/// 创建工单并分配递增 priority；进入「待回复」时自动写进度日志。
pub fn create(conn: &Connection, input: WorkOrderInput) -> Result<WorkOrder, ServiceError> {
    validate_title(&input.title)?;
    let now = Utc::now().naive_utc();
    let priority = next_priority(conn)?;
    let status = input.status;
    conn.execute(
        "INSERT INTO work_order (title, description, status, priority, waiting_for, waiting_reason, due_date, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![
            input.title.trim(),
            input.description,
            status.as_db_str(),
            priority,
            input.waiting_for,
            input.waiting_reason,
            input.due_date.map(format_datetime),
            format_datetime(now),
            format_datetime(now),
        ],
    )?;
    let id = conn.last_insert_rowid();
    let saved = get_required(conn, id)?;
    append_waiting_reply_progress_log(conn, None, &saved)?;
    get_required(conn, id)
}

/// 更新工单字段；「待回复」状态变更时自动追加进度日志。
pub fn update(conn: &Connection, id: i64, input: WorkOrderInput) -> Result<WorkOrder, ServiceError> {
    let existing = get_required(conn, id)?;
    let before = existing.clone();
    validate_title(&input.title)?;
    let now = Utc::now().naive_utc();
    conn.execute(
        "UPDATE work_order SET title = ?1, description = ?2, status = ?3, waiting_for = ?4, waiting_reason = ?5, due_date = ?6, updated_at = ?7 WHERE id = ?8",
        params![
            input.title.trim(),
            input.description,
            input.status.as_db_str(),
            input.waiting_for,
            input.waiting_reason,
            input.due_date.map(format_datetime),
            format_datetime(now),
            id,
        ],
    )?;
    let saved = get_required(conn, id)?;
    append_waiting_reply_progress_log(conn, Some(&before), &saved)?;
    Ok(saved)
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

/// 按状态筛选工单；`statuses` 为空表示全部，`include_completed` 控制是否含已完成。
/// `query` 非空时在标题、描述、待回复字段中模糊匹配。
pub fn find_by_statuses(
    conn: &Connection,
    statuses: &[String],
    include_completed: bool,
    query: Option<&str>,
) -> Result<Vec<WorkOrder>, ServiceError> {
    let mut effective: Vec<WorkOrderStatus> = if statuses.is_empty() {
        WorkOrderStatus::all().to_vec()
    } else {
        statuses
            .iter()
            .filter_map(|s| WorkOrderStatus::from_db_str(s))
            .collect()
    };
    if !include_completed {
        let user_selected_completed = statuses
            .iter()
            .any(|s| WorkOrderStatus::from_db_str(s) == Some(WorkOrderStatus::Completed));
        if !user_selected_completed {
            effective.retain(|s| *s != WorkOrderStatus::Completed);
        }
    }
    if effective.is_empty() {
        return Ok(vec![]);
    }
    let placeholders: Vec<String> = effective.iter().enumerate().map(|(i, _)| format!("?{}", i + 1)).collect();
    let mut sql = format!(
        "SELECT id, title, description, status, priority, waiting_for, waiting_reason, due_date, created_at, updated_at
         FROM work_order WHERE status IN ({})",
        placeholders.join(", ")
    );
    let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = effective
        .iter()
        .map(|s| Box::new(s.as_db_str().to_string()) as Box<dyn rusqlite::types::ToSql>)
        .collect();
    if let Some(q) = query.map(str::trim).filter(|s| !s.is_empty()) {
        let base = params.len();
        let pattern = format!("%{q}%");
        sql.push_str(&format!(
            " AND (title LIKE ?{} OR description LIKE ?{} OR waiting_for LIKE ?{} OR waiting_reason LIKE ?{})",
            base + 1,
            base + 2,
            base + 3,
            base + 4,
        ));
        for _ in 0..4 {
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

/// 判断工单是否逾期（有截止日期、未完成且已过期）。
pub fn is_overdue(work_order: &WorkOrder) -> bool {
    if work_order.due_date.is_none() {
        return false;
    }
    if work_order.status == WorkOrderStatus::Completed {
        return false;
    }
    work_order.due_date.unwrap() < Utc::now().naive_utc()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::connection::open_connection;
    use crate::services::progress_log_service;
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

    fn input(title: &str) -> WorkOrderInput {
        WorkOrderInput {
            title: title.to_string(),
            description: None,
            status: WorkOrderStatus::NotStarted,
            waiting_for: None,
            waiting_reason: None,
            due_date: None,
        }
    }

    #[test]
    fn create_and_find_by_statuses() {
        let (conn, dir) = temp_db();
        let created = create(&conn, input("Task A")).unwrap();
        assert!(created.id.is_some());
        let list = find_by_statuses(&conn, &[], false, None).unwrap();
        assert_eq!(list.len(), 1);
        drop(conn);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn filter_hides_completed() {
        let (conn, dir) = temp_db();
        create(&conn, input("Open")).unwrap();
        let done = create(&conn, input("Done")).unwrap();
        update(
            &conn,
            done.id.unwrap(),
            WorkOrderInput {
                status: WorkOrderStatus::Completed,
                ..input("Done")
            },
        )
        .unwrap();
        let without = find_by_statuses(&conn, &[], false, None).unwrap();
        assert_eq!(without.len(), 1);
        assert_eq!(without[0].title, "Open");
        let only_done = find_by_statuses(&conn, &["COMPLETED".to_string()], true, None).unwrap();
        assert_eq!(only_done.len(), 1);
        let only_done_without_toggle =
            find_by_statuses(&conn, &["COMPLETED".to_string()], false, None).unwrap();
        assert_eq!(only_done_without_toggle.len(), 1);
        drop(conn);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn update_priorities_reorders() {
        let (conn, dir) = temp_db();
        let first = create(&conn, input("First")).unwrap();
        let second = create(&conn, input("Second")).unwrap();
        update_priorities(&conn, &[second.id.unwrap(), first.id.unwrap()]).unwrap();
        let list = find_by_statuses(&conn, &[], true, None).unwrap();
        assert_eq!(list[0].id, second.id);
        drop(conn);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn is_overdue_respects_completed() {
        let (conn, dir) = temp_db();
        let mut wo = create(
            &conn,
            WorkOrderInput {
                due_date: Some(Utc::now().naive_utc() - Duration::hours(1)),
                ..input("Overdue")
            },
        )
        .unwrap();
        assert!(is_overdue(&wo));
        wo = update(
            &conn,
            wo.id.unwrap(),
            WorkOrderInput {
                status: WorkOrderStatus::Completed,
                due_date: wo.due_date,
                ..input("Overdue")
            },
        )
        .unwrap();
        assert!(!is_overdue(&wo));
        drop(conn);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn waiting_reply_appends_progress_log() {
        let (conn, dir) = temp_db();
        let created = create(&conn, input("Waiting task")).unwrap();
        update(
            &conn,
            created.id.unwrap(),
            WorkOrderInput {
                status: WorkOrderStatus::WaitingReply,
                waiting_for: Some("联调方".into()),
                waiting_reason: Some("等待接口确认".into()),
                ..input("Waiting task")
            },
        )
        .unwrap();
        let logs = progress_log_service::find_by_work_order_id(&conn, created.id.unwrap()).unwrap();
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].content, "待回复：等待 联调方，原因 等待接口确认");
        drop(conn);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn reads_legacy_java_epoch_timestamps() {
        let (conn, dir) = temp_db();
        conn.execute(
            "INSERT INTO work_order (title, description, status, priority, waiting_for, waiting_reason, due_date, created_at, updated_at)
             VALUES ('Legacy', NULL, 'WAITING_REPLY', 0, NULL, NULL, NULL, 1783324852840, 1783324918907)",
            [],
        )
        .unwrap();
        let list = find_by_statuses(&conn, &[], false, None).unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].title, "Legacy");
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
        )
        .unwrap();
        create(
            &conn,
            WorkOrderInput {
                title: "其他任务".into(),
                ..input("其他任务")
            },
        )
        .unwrap();
        create(
            &conn,
            WorkOrderInput {
                title: "待确认".into(),
                status: WorkOrderStatus::WaitingReply,
                waiting_for: Some("运维组".into()),
                waiting_reason: Some("等待排期".into()),
                ..input("待确认")
            },
        )
        .unwrap();

        let by_title = find_by_statuses(&conn, &[], true, Some("网络")).unwrap();
        assert_eq!(by_title.len(), 1);
        assert_eq!(by_title[0].title, "网络故障");

        let by_description = find_by_statuses(&conn, &[], true, Some("交换机")).unwrap();
        assert_eq!(by_description.len(), 1);

        let by_waiting = find_by_statuses(&conn, &[], true, Some("运维组")).unwrap();
        assert_eq!(by_waiting.len(), 1);

        let by_reason = find_by_statuses(&conn, &[], true, Some("排期")).unwrap();
        assert_eq!(by_reason.len(), 1);

        let no_match = find_by_statuses(&conn, &[], true, Some("不存在的关键词")).unwrap();
        assert!(no_match.is_empty());

        let empty_query = find_by_statuses(&conn, &[], true, Some("  ")).unwrap();
        assert_eq!(empty_query.len(), 3);

        drop(conn);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn title_required() {
        let (conn, dir) = temp_db();
        let err = create(&conn, input(" ")).unwrap_err();
        assert!(matches!(err, ServiceError::Validation(_)));
        drop(conn);
        let _ = std::fs::remove_dir_all(dir);
    }
}
