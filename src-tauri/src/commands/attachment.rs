//! 图片附件 Tauri Command。

use tauri::State;
use tauri_plugin_dialog::DialogExt;

use crate::error::ServiceError;
use crate::models::attachment::{Attachment, OwnerType};
use crate::services::attachment_service;
use crate::AppState;

fn map_err(err: ServiceError) -> String {
    err.to_string()
}

#[tauri::command]
#[specta::specta]
pub fn list_attachments(
    state: State<'_, AppState>,
    owner_type: OwnerType,
    owner_id: i64,
) -> Result<Vec<Attachment>, String> {
    let conn = state.db.lock().map_err(|_| "database lock poisoned".to_string())?;
    attachment_service::list_by_owner(&conn, &state.data_dir, owner_type, owner_id)
        .map_err(map_err)
}

#[tauri::command]
#[specta::specta]
pub fn add_attachment_from_file(
    state: State<'_, AppState>,
    owner_type: OwnerType,
    owner_id: i64,
    source_path: String,
) -> Result<Attachment, String> {
    let conn = state.db.lock().map_err(|_| "database lock poisoned".to_string())?;
    let path = std::path::Path::new(&source_path);
    let original_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .map(String::from);
    attachment_service::add_from_file(
        &conn,
        &state.data_dir,
        owner_type,
        owner_id,
        path,
        original_name,
    )
    .map_err(map_err)
}

#[tauri::command]
#[specta::specta]
pub fn add_attachment_from_bytes(
    state: State<'_, AppState>,
    owner_type: OwnerType,
    owner_id: i64,
    file_name: String,
    mime_type: String,
    data: Vec<u8>,
) -> Result<Attachment, String> {
    let conn = state.db.lock().map_err(|_| "database lock poisoned".to_string())?;
    attachment_service::add_from_bytes(
        &conn,
        &state.data_dir,
        owner_type,
        owner_id,
        &file_name,
        &mime_type,
        &data,
    )
    .map_err(map_err)
}

#[tauri::command]
#[specta::specta]
pub fn delete_attachment(state: State<'_, AppState>, id: i64) -> Result<(), String> {
    let conn = state.db.lock().map_err(|_| "database lock poisoned".to_string())?;
    attachment_service::delete_one(&conn, &state.data_dir, id).map_err(map_err)
}

/// 打开图片文件选择器，返回选中路径（未选则 None）。
#[tauri::command]
#[specta::specta]
pub async fn pick_attachment_file(app: tauri::AppHandle) -> Result<Option<String>, String> {
    let path = app
        .dialog()
        .file()
        .add_filter("图片", &["png", "jpg", "jpeg", "gif", "webp", "bmp"])
        .blocking_pick_file();
    Ok(path.map(|p| p.to_string()))
}
