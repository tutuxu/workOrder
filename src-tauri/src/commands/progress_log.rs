//! 进度日志相关 Tauri Command。

use tauri::State;

use crate::error::ServiceError;
use crate::models::attachment::OwnerType;
use crate::models::progress_log::{ProgressLog, ProgressLogInput};
use crate::services::attachment_service;
use crate::services::progress_log_service;
use crate::services::status_config_service;
use crate::AppState;

fn map_err(err: ServiceError) -> String {
    err.to_string()
}

/// 列出指定工单下的全部进度日志，按 created_at 降序。
#[tauri::command]
#[specta::specta]
pub fn list_progress_logs(
    state: State<'_, AppState>,
    work_order_id: i64,
) -> Result<Vec<ProgressLog>, String> {
    let conn = state.db.lock().map_err(|_| "database lock poisoned".to_string())?;
    progress_log_service::find_by_work_order_id(&conn, work_order_id).map_err(map_err)
}

/// 为工单添加进度日志，title 不能为空。
#[tauri::command]
#[specta::specta]
pub fn add_progress_log(
    state: State<'_, AppState>,
    work_order_id: i64,
    input: ProgressLogInput,
) -> Result<ProgressLog, String> {
    let conn = state.db.lock().map_err(|_| "database lock poisoned".to_string())?;
    let config = status_config_service::load_config(&state.data_dir).map_err(map_err)?;
    progress_log_service::add_log(&conn, work_order_id, &input, &config).map_err(map_err)
}

/// 更新进度日志；log 必须属于指定工单。
#[tauri::command]
#[specta::specta]
pub fn update_progress_log(
    state: State<'_, AppState>,
    log_id: i64,
    work_order_id: i64,
    input: ProgressLogInput,
) -> Result<ProgressLog, String> {
    let conn = state.db.lock().map_err(|_| "database lock poisoned".to_string())?;
    let config = status_config_service::load_config(&state.data_dir).map_err(map_err)?;
    progress_log_service::update_log(&conn, log_id, work_order_id, &input, &config).map_err(map_err)
}

/// 删除进度日志；log 必须属于指定工单。
#[tauri::command]
#[specta::specta]
pub fn delete_progress_log(
    state: State<'_, AppState>,
    log_id: i64,
    work_order_id: i64,
) -> Result<(), String> {
    let conn = state.db.lock().map_err(|_| "database lock poisoned".to_string())?;
    attachment_service::delete_all_for_owner(
        &conn,
        &state.data_dir,
        OwnerType::ProgressLog,
        log_id,
    )
    .map_err(map_err)?;
    progress_log_service::delete_log(&conn, log_id, work_order_id).map_err(map_err)
}
