//! 进度日志 CRUD。

use std::collections::HashMap;

use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension};

use crate::db::datetime::{format_datetime, read_datetime_column};
use crate::error::ServiceError;
use crate::models::progress_log::{ProgressLog, ProgressLogInput};
use crate::models::status_config::StatusConfig;
use crate::services::status_config_service;
use crate::services::work_order_service::get_required;

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

fn validate_input(config: &StatusConfig, input: &ProgressLogInput) -> Result<(), ServiceError> {
    if input.title.trim().is_empty() {
        return Err(ServiceError::Validation("Progress title is required".into()));
    }
    let fields = normalize_extra_fields(input.extra_fields.clone()).unwrap_or_default();
    status_config_service::validate_extra_fields(config, &input.status, &fields)?;
    Ok(())
}

fn row_to_progress_log(row: &rusqlite::Row<'_>) -> Result<ProgressLog, rusqlite::Error> {
    Ok(ProgressLog {
        id: Some(row.get("id")?),
        work_order_id: row.get("work_order_id")?,
        title: row.get("title")?,
        content: row.get("content")?,
        status: row.get("status")?,
        extra_fields: read_extra_fields(row)?,
        created_at: read_datetime_column(row, "created_at")?,
    })
}

const PROGRESS_LOG_SELECT: &str =
    "SELECT id, work_order_id, title, content, status, extra_fields, created_at FROM progress_log";

fn get_required_log(
    conn: &Connection,
    log_id: i64,
    work_order_id: i64,
) -> Result<ProgressLog, ServiceError> {
    let log = conn
        .query_row(
            &format!("{PROGRESS_LOG_SELECT} WHERE id = ?1"),
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
    let mut stmt = conn.prepare(&format!(
        "{PROGRESS_LOG_SELECT} WHERE work_order_id = ?1 ORDER BY created_at DESC"
    ))?;
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
    config: &StatusConfig,
) -> Result<ProgressLog, ServiceError> {
    validate_input(config, input)?;
    get_required(conn, work_order_id)?;
    let now = Utc::now().naive_utc();
    let content = input
        .content
        .as_deref()
        .map(str::trim)
        .unwrap_or("");
    let extra_fields = normalize_extra_fields(input.extra_fields.clone());
    conn.execute(
        "INSERT INTO progress_log (work_order_id, title, content, status, extra_fields, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            work_order_id,
            input.title.trim(),
            content,
            input.status,
            extra_fields_to_json(&extra_fields),
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
    config: &StatusConfig,
) -> Result<ProgressLog, ServiceError> {
    validate_input(config, input)?;
    get_required_log(conn, log_id, work_order_id)?;
    let content = input
        .content
        .as_deref()
        .map(str::trim)
        .unwrap_or("");
    let extra_fields = normalize_extra_fields(input.extra_fields.clone());
    conn.execute(
        "UPDATE progress_log SET title = ?1, content = ?2, status = ?3, extra_fields = ?4 WHERE id = ?5",
        params![
            input.title.trim(),
            content,
            input.status,
            extra_fields_to_json(&extra_fields),
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
    use crate::models::status_config::{StatusConfig, StatusDefinition, StatusField, StatusFieldType};
    use crate::models::tag_config::TagConfig;
    use crate::models::work_order::WorkOrderInput;
    use crate::services::work_order_service::create;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn config() -> StatusConfig {
        StatusConfig::default_config()
    }

    fn tag_config() -> TagConfig {
        TagConfig::default_config()
    }

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
            status: "IN_PROGRESS".into(),
            extra_fields: None,
        }
    }

    #[test]
    fn add_log_with_empty_content() {
        let (conn, dir) = temp_db();
        let wo = create(
            &conn,
            WorkOrderInput {
                title: "With log".into(),
                description: None,
                status: "NOT_STARTED".into(),
                extra_fields: None,
                due_date: None,
                tags: vec![],
            },
            &config(),
            &tag_config(),
        )
        .unwrap();
        let log = add_log(
            &conn,
            wo.id.unwrap(),
            &ProgressLogInput {
                title: "Title only".into(),
                content: None,
                status: "IN_PROGRESS".into(),
                extra_fields: None,
            },
            &config(),
        )
        .unwrap();
        assert_eq!(log.content.as_deref(), Some(""));
        drop(conn);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn add_update_delete_log() {
        let (conn, dir) = temp_db();
        let wo = create(
            &conn,
            WorkOrderInput {
                title: "With log".into(),
                description: None,
                status: "NOT_STARTED".into(),
                extra_fields: None,
                due_date: None,
                tags: vec![],
            },
            &config(),
            &tag_config(),
        )
        .unwrap();
        let log = add_log(&conn, wo.id.unwrap(), &sample_input("Started investigation"), &config()).unwrap();
        assert_eq!(find_by_work_order_id(&conn, wo.id.unwrap()).unwrap().len(), 1);
        let updated = update_log(
            &conn,
            log.id.unwrap(),
            wo.id.unwrap(),
            &ProgressLogInput {
                title: "Updated title".into(),
                content: Some("Updated note".into()),
                status: "COMPLETED".into(),
                extra_fields: None,
            },
            &config(),
        )
        .unwrap();
        assert_eq!(updated.title, "Updated title");
        assert_eq!(updated.content.as_deref(), Some("Updated note"));
        assert_eq!(updated.status, "COMPLETED");
        delete_log(&conn, log.id.unwrap(), wo.id.unwrap()).unwrap();
        assert!(find_by_work_order_id(&conn, wo.id.unwrap()).unwrap().is_empty());
        drop(conn);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn required_extra_fields_enforced() {
        let (conn, dir) = temp_db();
        let mut status_config = config();
        status_config.statuses.push(StatusDefinition {
            id: "CUSTOM_WAITING".into(),
            label: "自定义待回复".into(),
            order: 10,
            color: "rgba(252, 211, 77, 0.45)".into(),
            fields: vec![StatusField {
                key: "waitingFor".into(),
                label: "等待对象".into(),
                field_type: StatusFieldType::Text,
                required: true,
            }],
        });
        let wo = create(
            &conn,
            WorkOrderInput {
                title: "With log".into(),
                description: None,
                status: "NOT_STARTED".into(),
                extra_fields: None,
                due_date: None,
                tags: vec![],
            },
            &status_config,
            &tag_config(),
        )
        .unwrap();
        let err = add_log(
            &conn,
            wo.id.unwrap(),
            &ProgressLogInput {
                title: "Waiting".into(),
                content: None,
                status: "CUSTOM_WAITING".into(),
                extra_fields: None,
            },
            &status_config,
        )
        .unwrap_err();
        assert!(err.to_string().contains("等待对象"));
        let log = add_log(
            &conn,
            wo.id.unwrap(),
            &ProgressLogInput {
                title: "Waiting".into(),
                content: None,
                status: "CUSTOM_WAITING".into(),
                extra_fields: Some(HashMap::from([("waitingFor".into(), "运维组".into())])),
            },
            &status_config,
        )
        .unwrap();
        assert_eq!(
            log.extra_fields
                .as_ref()
                .and_then(|m| m.get("waitingFor"))
                .map(String::as_str),
            Some("运维组"),
        );
        drop(conn);
        let _ = std::fs::remove_dir_all(dir);
    }
}
