use tauri::State;

use crate::error::ServiceError;
use crate::models::work_order::{WorkOrder, WorkOrderInput};
use crate::services::work_order_service;
use crate::AppState;

fn map_err(err: ServiceError) -> String {
    err.to_string()
}

#[tauri::command]
pub fn list_work_orders(
    state: State<'_, AppState>,
    statuses: Vec<String>,
    include_completed: bool,
) -> Result<Vec<WorkOrder>, String> {
    let conn = state.db.lock().map_err(|_| "database lock poisoned".to_string())?;
    work_order_service::find_by_statuses(&conn, &statuses, include_completed).map_err(map_err)
}

#[tauri::command]
pub fn get_work_order(state: State<'_, AppState>, id: i64) -> Result<WorkOrder, String> {
    let conn = state.db.lock().map_err(|_| "database lock poisoned".to_string())?;
    work_order_service::get_required(&conn, id).map_err(map_err)
}

#[tauri::command]
pub fn create_work_order(
    state: State<'_, AppState>,
    input: WorkOrderInput,
) -> Result<WorkOrder, String> {
    let conn = state.db.lock().map_err(|_| "database lock poisoned".to_string())?;
    work_order_service::create(&conn, input).map_err(map_err)
}

#[tauri::command]
pub fn update_work_order(
    state: State<'_, AppState>,
    id: i64,
    input: WorkOrderInput,
) -> Result<WorkOrder, String> {
    let conn = state.db.lock().map_err(|_| "database lock poisoned".to_string())?;
    work_order_service::update(&conn, id, input).map_err(map_err)
}

#[tauri::command]
pub fn delete_work_order(state: State<'_, AppState>, id: i64) -> Result<(), String> {
    let conn = state.db.lock().map_err(|_| "database lock poisoned".to_string())?;
    work_order_service::delete(&conn, id).map_err(map_err)
}

#[tauri::command]
pub fn update_priorities(
    state: State<'_, AppState>,
    ordered_ids: Vec<i64>,
) -> Result<(), String> {
    let conn = state.db.lock().map_err(|_| "database lock poisoned".to_string())?;
    work_order_service::update_priorities(&conn, &ordered_ids).map_err(map_err)
}

#[tauri::command]
pub fn is_work_order_overdue(work_order: WorkOrder) -> bool {
    work_order_service::is_overdue(&work_order)
}
