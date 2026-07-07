//! 图片附件增删查、文件读写与级联清理。

use std::fs;
use std::path::{Path, PathBuf};

use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension};
use uuid::Uuid;

use crate::db::datetime::{format_datetime, read_datetime_column};
use crate::error::ServiceError;
use crate::models::attachment::{Attachment, OwnerType};
use crate::services::progress_log_service;
use crate::services::work_order_service;

const MAX_FILE_SIZE: usize = 20 * 1024 * 1024;

const ALLOWED_MIMES: &[&str] = &[
    "image/jpeg",
    "image/png",
    "image/gif",
    "image/webp",
    "image/bmp",
];

/// 返回 `{data_dir}/attachments/{type}/{id}/`。
pub fn attachments_dir(data_dir: &Path, owner_type: OwnerType, owner_id: i64) -> PathBuf {
    data_dir
        .join("attachments")
        .join(owner_type.dir_name())
        .join(owner_id.to_string())
}

fn detect_mime(header: &[u8]) -> Option<&'static str> {
    if header.starts_with(&[0xFF, 0xD8, 0xFF]) {
        return Some("image/jpeg");
    }
    if header.starts_with(&[0x89, 0x50, 0x4E, 0x47]) {
        return Some("image/png");
    }
    if header.starts_with(b"GIF87a") || header.starts_with(b"GIF89a") {
        return Some("image/gif");
    }
    if header.len() >= 12 && &header[0..4] == b"RIFF" && &header[8..12] == b"WEBP" {
        return Some("image/webp");
    }
    if header.starts_with(&[0x42, 0x4D]) {
        return Some("image/bmp");
    }
    None
}

fn extension_for_mime(mime: &str) -> Option<&'static str> {
    match mime {
        "image/jpeg" => Some(".jpg"),
        "image/png" => Some(".png"),
        "image/gif" => Some(".gif"),
        "image/webp" => Some(".webp"),
        "image/bmp" => Some(".bmp"),
        _ => None,
    }
}

/// MIME 白名单 + 20MB 上限 + 魔数校验。
pub fn validate_image(mime: &str, size: usize, header: &[u8]) -> Result<(), ServiceError> {
    if !ALLOWED_MIMES.contains(&mime) {
        return Err(ServiceError::Validation(
            "仅支持图片文件（JPEG、PNG、GIF、WebP、BMP）".into(),
        ));
    }
    if size > MAX_FILE_SIZE {
        return Err(ServiceError::Validation("图片大小不能超过 20MB".into()));
    }
    let detected = detect_mime(header).ok_or_else(|| {
        ServiceError::Validation("仅支持图片文件（JPEG、PNG、GIF、WebP、BMP）".into())
    })?;
    if detected != mime {
        return Err(ServiceError::Validation(
            "仅支持图片文件（JPEG、PNG、GIF、WebP、BMP）".into(),
        ));
    }
    Ok(())
}

fn ensure_path_in_attachments(data_dir: &Path, path: &Path) -> Result<(), ServiceError> {
    let base = data_dir.join("attachments");
    let canonical_base = dunce::canonicalize(&base).unwrap_or(base);
    let canonical_path = dunce::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    if !canonical_path.starts_with(&canonical_base) {
        return Err(ServiceError::Validation("invalid attachment path".into()));
    }
    Ok(())
}

fn verify_owner_exists(
    conn: &Connection,
    owner_type: OwnerType,
    owner_id: i64,
) -> Result<(), ServiceError> {
    match owner_type {
        OwnerType::WorkOrder => {
            work_order_service::get_required(conn, owner_id)?;
        }
        OwnerType::ProgressLog => {
            conn.query_row(
                "SELECT id FROM progress_log WHERE id = ?1",
                params![owner_id],
                |row| row.get::<_, i64>(0),
            )
            .optional()?
            .ok_or_else(|| ServiceError::NotFound(format!("Progress log not found: {owner_id}")))?;
        }
    }
    Ok(())
}

fn row_to_attachment(
    row: &rusqlite::Row<'_>,
    data_dir: &Path,
    owner_type: OwnerType,
    owner_id: i64,
) -> Result<Attachment, rusqlite::Error> {
    let file_name: String = row.get("file_name")?;
    let dir = attachments_dir(data_dir, owner_type, owner_id);
    let file_path = dir.join(&file_name);
    Ok(Attachment {
        id: Some(row.get("id")?),
        owner_type,
        owner_id,
        file_name,
        original_name: row.get("original_name")?,
        mime_type: row.get("mime_type")?,
        file_size: row.get("file_size")?,
        created_at: read_datetime_column(row, "created_at")?,
        file_path: file_path.to_string_lossy().into_owned(),
    })
}

fn get_by_id(conn: &Connection, data_dir: &Path, id: i64) -> Result<Attachment, ServiceError> {
    conn.query_row(
        "SELECT id, owner_type, owner_id, file_name, original_name, mime_type, file_size, created_at
         FROM attachment WHERE id = ?1",
        params![id],
        |row| {
            let owner_type_str: String = row.get("owner_type")?;
            let owner_type = OwnerType::from_str(&owner_type_str).ok_or_else(|| {
                rusqlite::Error::InvalidColumnType(
                    1,
                    "owner_type".into(),
                    rusqlite::types::Type::Text,
                )
            })?;
            let owner_id: i64 = row.get("owner_id")?;
            row_to_attachment(row, data_dir, owner_type, owner_id)
        },
    )
    .optional()?
    .ok_or_else(|| ServiceError::NotFound(format!("Attachment not found: {id}")))
}

fn remove_empty_dir(dir: &Path) {
    if dir.exists() {
        let _ = fs::remove_dir(dir);
    }
}

/// 按归属查询附件，按 created_at 升序。
pub fn list_by_owner(
    conn: &Connection,
    data_dir: &Path,
    owner_type: OwnerType,
    owner_id: i64,
) -> Result<Vec<Attachment>, ServiceError> {
    let mut stmt = conn.prepare(
        "SELECT id, owner_type, owner_id, file_name, original_name, mime_type, file_size, created_at
         FROM attachment WHERE owner_type = ?1 AND owner_id = ?2 ORDER BY created_at ASC",
    )?;
    let rows = stmt.query_map(
        params![owner_type.as_str(), owner_id],
        |row| row_to_attachment(row, data_dir, owner_type, owner_id),
    )?;
    let mut result = Vec::new();
    for row in rows {
        result.push(row?);
    }
    Ok(result)
}

fn insert_attachment(
    conn: &Connection,
    data_dir: &Path,
    owner_type: OwnerType,
    owner_id: i64,
    file_name: &str,
    original_name: Option<&str>,
    mime_type: &str,
    file_size: i64,
) -> Result<Attachment, ServiceError> {
    let now = Utc::now().naive_utc();
    conn.execute(
        "INSERT INTO attachment (owner_type, owner_id, file_name, original_name, mime_type, file_size, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            owner_type.as_str(),
            owner_id,
            file_name,
            original_name,
            mime_type,
            file_size,
            format_datetime(now),
        ],
    )?;
    let id = conn.last_insert_rowid();
    let dir = attachments_dir(data_dir, owner_type, owner_id);
    Ok(Attachment {
        id: Some(id),
        owner_type,
        owner_id,
        file_name: file_name.to_string(),
        original_name: original_name.map(str::to_string),
        mime_type: mime_type.to_string(),
        file_size,
        created_at: now,
        file_path: dir.join(file_name).to_string_lossy().into_owned(),
    })
}

fn write_validated_image(
    conn: &Connection,
    data_dir: &Path,
    owner_type: OwnerType,
    owner_id: i64,
    data: &[u8],
    mime_type: &str,
    original_name: Option<&str>,
) -> Result<Attachment, ServiceError> {
    verify_owner_exists(conn, owner_type, owner_id)?;
    let header_len = data.len().min(16);
    validate_image(mime_type, data.len(), &data[..header_len])?;

    let ext = extension_for_mime(mime_type).ok_or_else(|| {
        ServiceError::Validation("仅支持图片文件（JPEG、PNG、GIF、WebP、BMP）".into())
    })?;
    let stored_name = format!("{}{}", Uuid::new_v4(), ext);
    let dir = attachments_dir(data_dir, owner_type, owner_id);
    fs::create_dir_all(&dir)?;
    let dest = dir.join(&stored_name);
    ensure_path_in_attachments(data_dir, &dest)?;

    if let Err(e) = fs::write(&dest, data) {
        remove_empty_dir(&dir);
        return Err(e.into());
    }

    match insert_attachment(
        conn,
        data_dir,
        owner_type,
        owner_id,
        &stored_name,
        original_name,
        mime_type,
        data.len() as i64,
    ) {
        Ok(att) => Ok(att),
        Err(e) => {
            let _ = fs::remove_file(&dest);
            remove_empty_dir(&dir);
            Err(e)
        }
    }
}

/// 从本地路径复制图片。
pub fn add_from_file(
    conn: &Connection,
    data_dir: &Path,
    owner_type: OwnerType,
    owner_id: i64,
    source_path: &Path,
    original_name: Option<String>,
) -> Result<Attachment, ServiceError> {
    let data = fs::read(source_path)?;
    let header_len = data.len().min(16);
    let mime = detect_mime(&data[..header_len]).ok_or_else(|| {
        ServiceError::Validation("仅支持图片文件（JPEG、PNG、GIF、WebP、BMP）".into())
    })?;
    write_validated_image(
        conn,
        data_dir,
        owner_type,
        owner_id,
        &data,
        mime,
        original_name.as_deref(),
    )
}

/// 从字节写入图片（剪贴板粘贴）。
pub fn add_from_bytes(
    conn: &Connection,
    data_dir: &Path,
    owner_type: OwnerType,
    owner_id: i64,
    _file_name: &str,
    mime_type: &str,
    data: &[u8],
) -> Result<Attachment, ServiceError> {
    write_validated_image(
        conn,
        data_dir,
        owner_type,
        owner_id,
        data,
        mime_type,
        None,
    )
}

/// 删除单条附件记录及磁盘文件。
pub fn delete_one(conn: &Connection, data_dir: &Path, id: i64) -> Result<(), ServiceError> {
    let att = get_by_id(conn, data_dir, id)?;
    let owner_type = att.owner_type;
    let owner_id = att.owner_id;
    let dir = attachments_dir(data_dir, owner_type, owner_id);
    let file_path = dir.join(&att.file_name);
    ensure_path_in_attachments(data_dir, &file_path)?;

    conn.execute("DELETE FROM attachment WHERE id = ?1", params![id])?;
    if file_path.exists() {
        fs::remove_file(&file_path)?;
    }
    remove_empty_dir(&dir);
    Ok(())
}

/// 删除指定归属的全部附件。
pub fn delete_all_for_owner(
    conn: &Connection,
    data_dir: &Path,
    owner_type: OwnerType,
    owner_id: i64,
) -> Result<(), ServiceError> {
    let items = list_by_owner(conn, data_dir, owner_type, owner_id)?;
    for att in items {
        if let Some(id) = att.id {
            delete_one(conn, data_dir, id)?;
        }
    }
    let dir = attachments_dir(data_dir, owner_type, owner_id);
    if dir.exists() {
        let _ = fs::remove_dir_all(&dir);
    }
    Ok(())
}

/// 删除工单及其全部 progress_log 的附件。
pub fn delete_all_for_work_order(
    conn: &Connection,
    data_dir: &Path,
    work_order_id: i64,
) -> Result<(), ServiceError> {
    delete_all_for_owner(conn, data_dir, OwnerType::WorkOrder, work_order_id)?;

    let logs = progress_log_service::find_by_work_order_id(conn, work_order_id)?;
    for log in logs {
        if let Some(log_id) = log.id {
            delete_all_for_owner(conn, data_dir, OwnerType::ProgressLog, log_id)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::connection::open_connection;
    use crate::models::progress_log::ProgressLogInput;
    use crate::models::work_order::WorkOrderInput;
    use crate::models::work_order_status::WorkOrderStatus;
    use crate::services::progress_log_service;
    use crate::services::work_order_service;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_db() -> (Connection, PathBuf) {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("workorder-att-{nanos}"));
        let conn = open_connection(&dir).unwrap();
        (conn, dir)
    }

    fn sample_png() -> Vec<u8> {
        vec![
            0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48,
            0x44, 0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00,
            0x00, 0x1F, 0x15, 0xC4, 0x89, 0x00, 0x00, 0x00, 0x0A, 0x49, 0x44, 0x41, 0x54, 0x78,
            0x9C, 0x63, 0x00, 0x01, 0x00, 0x00, 0x05, 0x00, 0x01, 0x0D, 0x0A, 0x2D, 0xB4, 0x00,
            0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
        ]
    }

    fn wo_input(title: &str) -> WorkOrderInput {
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
    fn add_list_delete_attachment() {
        let (conn, dir) = temp_db();
        let wo = work_order_service::create(&conn, wo_input("att test")).unwrap();
        let wo_id = wo.id.unwrap();

        add_from_bytes(
            &conn,
            &dir,
            OwnerType::WorkOrder,
            wo_id,
            "test.png",
            "image/png",
            &sample_png(),
        )
        .unwrap();

        let list = list_by_owner(&conn, &dir, OwnerType::WorkOrder, wo_id).unwrap();
        assert_eq!(list.len(), 1);
        let file_path = PathBuf::from(&list[0].file_path);
        assert!(file_path.exists());

        delete_one(&conn, &dir, list[0].id.unwrap()).unwrap();
        let list = list_by_owner(&conn, &dir, OwnerType::WorkOrder, wo_id).unwrap();
        assert!(list.is_empty());
        assert!(!file_path.exists());
    }

    #[test]
    fn rejects_oversized_file() {
        let (conn, dir) = temp_db();
        let wo = work_order_service::create(&conn, wo_input("oversize")).unwrap();
        let wo_id = wo.id.unwrap();
        let mut data = sample_png();
        data.resize(MAX_FILE_SIZE + 1, 0);
        let err = add_from_bytes(
            &conn,
            &dir,
            OwnerType::WorkOrder,
            wo_id,
            "big.png",
            "image/png",
            &data,
        )
        .unwrap_err();
        assert!(matches!(err, ServiceError::Validation(_)));
    }

    #[test]
    fn rejects_non_image() {
        let (conn, dir) = temp_db();
        let wo = work_order_service::create(&conn, wo_input("non-image")).unwrap();
        let wo_id = wo.id.unwrap();
        let err = add_from_bytes(
            &conn,
            &dir,
            OwnerType::WorkOrder,
            wo_id,
            "test.txt",
            "text/plain",
            b"hello world",
        )
        .unwrap_err();
        assert!(matches!(err, ServiceError::Validation(_)));
    }

    #[test]
    fn cascade_delete_work_order() {
        let (conn, dir) = temp_db();
        let wo = work_order_service::create(&conn, wo_input("cascade")).unwrap();
        let wo_id = wo.id.unwrap();

        add_from_bytes(
            &conn,
            &dir,
            OwnerType::WorkOrder,
            wo_id,
            "wo.png",
            "image/png",
            &sample_png(),
        )
        .unwrap();

        let log = progress_log_service::add_log(
            &conn,
            wo_id,
            &ProgressLogInput {
                title: "step".into(),
                content: None,
                status: WorkOrderStatus::InProgress,
            },
        )
        .unwrap();
        let log_id = log.id.unwrap();

        add_from_bytes(
            &conn,
            &dir,
            OwnerType::ProgressLog,
            log_id,
            "log.png",
            "image/png",
            &sample_png(),
        )
        .unwrap();

        delete_all_for_work_order(&conn, &dir, wo_id).unwrap();

        assert!(list_by_owner(&conn, &dir, OwnerType::WorkOrder, wo_id)
            .unwrap()
            .is_empty());
        assert!(list_by_owner(&conn, &dir, OwnerType::ProgressLog, log_id)
            .unwrap()
            .is_empty());
        assert!(!attachments_dir(&dir, OwnerType::WorkOrder, wo_id).exists());
        assert!(!attachments_dir(&dir, OwnerType::ProgressLog, log_id).exists());
    }
}
