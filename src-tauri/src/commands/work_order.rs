//! 工单相关 Tauri Command，前端通过 `invoke` 调用。

use tauri::State;

use crate::error::ServiceError;
use crate::models::tag_config::TagMatchMode;
use crate::models::work_order::{WorkOrder, WorkOrderInput};
use crate::services::attachment_service;
use crate::services::status_config_service;
use crate::services::tag_config_service;
use crate::services::work_order_service;
use crate::AppState;

fn map_err(err: ServiceError) -> String {
    err.to_string()
}

/// 按状态、标签与文本搜索筛选工单列表。
#[tauri::command]
#[specta::specta]
pub fn list_work_orders(
    state: State<'_, AppState>,
    statuses: Vec<String>,
    tags: Vec<String>,
    tag_match_mode: TagMatchMode,
    query: String,
) -> Result<Vec<WorkOrder>, String> {
    let conn = state
        .db
        .lock()
        .map_err(|_| "database lock poisoned".to_string())?;
    let tag_config = tag_config_service::load_config(&state.data_dir).map_err(map_err)?;
    let query_ref = if query.trim().is_empty() {
        None
    } else {
        Some(query.trim())
    };
    work_order_service::find_by_filters(
        &conn,
        &statuses,
        &tags,
        tag_match_mode,
        query_ref,
        &tag_config,
    )
    .map_err(map_err)
}

/// 按 id 获取单条工单，不存在时返回 `NOT_FOUND`。
#[tauri::command]
#[specta::specta]
pub fn get_work_order(state: State<'_, AppState>, id: i64) -> Result<WorkOrder, String> {
    let conn = state
        .db
        .lock()
        .map_err(|_| "database lock poisoned".to_string())?;
    work_order_service::get_required(&conn, id).map_err(map_err)
}

/// 创建工单；title 必填，priority 自动递增。
#[tauri::command]
#[specta::specta]
pub fn create_work_order(
    state: State<'_, AppState>,
    input: WorkOrderInput,
) -> Result<WorkOrder, String> {
    let conn = state
        .db
        .lock()
        .map_err(|_| "database lock poisoned".to_string())?;
    let config = status_config_service::load_config(&state.data_dir).map_err(map_err)?;
    let tag_config = tag_config_service::load_config(&state.data_dir).map_err(map_err)?;
    work_order_service::create(&conn, input, &config, &tag_config).map_err(map_err)
}

/// 更新工单。
#[tauri::command]
#[specta::specta]
pub fn update_work_order(
    state: State<'_, AppState>,
    id: i64,
    input: WorkOrderInput,
) -> Result<WorkOrder, String> {
    let conn = state
        .db
        .lock()
        .map_err(|_| "database lock poisoned".to_string())?;
    let config = status_config_service::load_config(&state.data_dir).map_err(map_err)?;
    let tag_config = tag_config_service::load_config(&state.data_dir).map_err(map_err)?;
    work_order_service::update(&conn, id, input, &config, &tag_config).map_err(map_err)
}

/// 删除工单及其全部进度日志。
#[tauri::command]
#[specta::specta]
pub fn delete_work_order(state: State<'_, AppState>, id: i64) -> Result<(), String> {
    let conn = state
        .db
        .lock()
        .map_err(|_| "database lock poisoned".to_string())?;
    attachment_service::delete_all_for_work_order(&conn, &state.data_dir, id).map_err(map_err)?;
    work_order_service::delete(&conn, id).map_err(map_err)
}

/// 按给定 id 顺序批量更新 priority（用于拖拽排序）。
#[tauri::command]
#[specta::specta]
pub fn update_priorities(
    state: State<'_, AppState>,
    ordered_ids: Vec<i64>,
) -> Result<(), String> {
    let conn = state
        .db
        .lock()
        .map_err(|_| "database lock poisoned".to_string())?;
    work_order_service::update_priorities(&conn, &ordered_ids).map_err(map_err)
}

/// 判断工单是否逾期（有 due_date 且早于当前时间）。
#[tauri::command]
#[specta::specta]
pub fn is_work_order_overdue(work_order: WorkOrder) -> bool {
    work_order_service::is_overdue(&work_order)
}

/// 列出回收站中的工单，按删除时间新到旧。
#[tauri::command]
#[specta::specta]
pub fn list_trashed_work_orders(state: State<'_, AppState>) -> Result<Vec<WorkOrder>, String> {
    let conn = state
        .db
        .lock()
        .map_err(|_| "database lock poisoned".to_string())?;
    work_order_service::list_trashed(&conn).map_err(map_err)
}

/// 将事项移入回收站；已在回收站或不存在的 id 跳过。
#[tauri::command]
#[specta::specta]
pub fn trash_work_orders(state: State<'_, AppState>, ids: Vec<i64>) -> Result<(), String> {
    let mut conn = state
        .db
        .lock()
        .map_err(|_| "database lock poisoned".to_string())?;
    work_order_service::trash_work_orders(&mut conn, &ids).map_err(map_err)
}

/// 从回收站还原事项；不在回收站或不存在的 id 跳过。
#[tauri::command]
#[specta::specta]
pub fn restore_work_orders(state: State<'_, AppState>, ids: Vec<i64>) -> Result<(), String> {
    let mut conn = state
        .db
        .lock()
        .map_err(|_| "database lock poisoned".to_string())?;
    work_order_service::restore_work_orders(&mut conn, &ids).map_err(map_err)
}

/// 彻底删除事项（含附件与进度）；不存在的 id 跳过。
#[tauri::command]
#[specta::specta]
pub fn permanently_delete_work_orders(
    state: State<'_, AppState>,
    ids: Vec<i64>,
) -> Result<(), String> {
    if ids.is_empty() {
        return Ok(());
    }
    let mut conn = state
        .db
        .lock()
        .map_err(|_| "database lock poisoned".to_string())?;
    let tx = conn.transaction().map_err(|e| map_err(e.into()))?;
    for id in ids {
        if work_order_service::get_required(&tx, id).is_err() {
            continue;
        }
        attachment_service::delete_all_for_work_order(&tx, &state.data_dir, id).map_err(map_err)?;
        work_order_service::delete(&tx, id).map_err(map_err)?;
    }
    tx.commit().map_err(|e| map_err(e.into()))?;
    Ok(())
}
