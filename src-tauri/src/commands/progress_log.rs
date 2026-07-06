use tauri::State;

use crate::error::ServiceError;
use crate::models::progress_log::ProgressLog;
use crate::services::progress_log_service;
use crate::AppState;

fn map_err(err: ServiceError) -> String {
    err.to_string()
}

#[tauri::command]
pub fn list_progress_logs(
    state: State<'_, AppState>,
    work_order_id: i64,
) -> Result<Vec<ProgressLog>, String> {
    let conn = state.db.lock().map_err(|_| "database lock poisoned".to_string())?;
    progress_log_service::find_by_work_order_id(&conn, work_order_id).map_err(map_err)
}

#[tauri::command]
pub fn add_progress_log(
    state: State<'_, AppState>,
    work_order_id: i64,
    content: String,
) -> Result<ProgressLog, String> {
    let conn = state.db.lock().map_err(|_| "database lock poisoned".to_string())?;
    progress_log_service::add_log(&conn, work_order_id, &content).map_err(map_err)
}

#[tauri::command]
pub fn update_progress_log(
    state: State<'_, AppState>,
    log_id: i64,
    work_order_id: i64,
    content: String,
) -> Result<ProgressLog, String> {
    let conn = state.db.lock().map_err(|_| "database lock poisoned".to_string())?;
    progress_log_service::update_log(&conn, log_id, work_order_id, &content).map_err(map_err)
}

#[tauri::command]
pub fn delete_progress_log(
    state: State<'_, AppState>,
    log_id: i64,
    work_order_id: i64,
) -> Result<(), String> {
    let conn = state.db.lock().map_err(|_| "database lock poisoned".to_string())?;
    progress_log_service::delete_log(&conn, log_id, work_order_id).map_err(map_err)
}
