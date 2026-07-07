//! ZIP 备份与恢复。

use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use chrono::Local;
use rusqlite::Connection;
use zip::write::SimpleFileOptions;
use zip::{ZipArchive, ZipWriter};

use crate::error::ServiceError;
use crate::settings::migration::{copy_data_dir, verify_migration};

pub const BACKUP_FORMAT_VERSION: u32 = 1;
const MANIFEST_NAME: &str = "manifest.json";

fn map_zip_err(err: zip::result::ZipError) -> ServiceError {
    ServiceError::Validation(format!("备份文件处理失败：{err}"))
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct BackupManifest {
    pub format_version: u32,
    pub app_version: String,
    pub exported_at: String,
}

pub fn checkpoint_connection(conn: &Connection) -> Result<(), ServiceError> {
    conn.execute_batch("PRAGMA wal_checkpoint(FULL);")?;
    Ok(())
}

pub fn default_backup_filename() -> String {
    Local::now()
        .format("workorder-backup-%Y%m%d-%H%M%S.zip")
        .to_string()
}

pub fn temp_restore_dir() -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("workorder-restore-{nanos}"))
}

pub fn create_backup_zip(data_dir: &Path, save_path: &Path) -> Result<(), ServiceError> {
    let manifest = BackupManifest {
        format_version: BACKUP_FORMAT_VERSION,
        app_version: env!("CARGO_PKG_VERSION").to_string(),
        exported_at: Local::now().format("%+").to_string(),
    };

    let file = File::create(save_path)?;
    let mut zip = ZipWriter::new(file);
    let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    let manifest_json = serde_json::to_string_pretty(&manifest)
        .map_err(|e| ServiceError::Validation(format!("serialize manifest: {e}")))?;
    zip.start_file(MANIFEST_NAME, options)
        .map_err(map_zip_err)?;
    zip.write_all(manifest_json.as_bytes())?;

    for entry in fs::read_dir(data_dir)? {
        let entry = entry?;
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if name_str.starts_with('.') {
            continue;
        }
        let path = entry.path();
        if path.is_file() {
            add_file_to_zip(&mut zip, &path, &name_str, options)?;
        } else if path.is_dir() {
            add_dir_to_zip(&mut zip, &path, &name_str, options)?;
        }
    }

    zip.finish().map_err(map_zip_err)?;
    Ok(())
}

fn add_file_to_zip(
    zip: &mut ZipWriter<File>,
    path: &Path,
    name_in_zip: &str,
    options: SimpleFileOptions,
) -> Result<(), ServiceError> {
    zip.start_file(name_in_zip, options)
        .map_err(map_zip_err)?;
    let mut file = File::open(path)?;
    std::io::copy(&mut file, zip)?;
    Ok(())
}

fn add_dir_to_zip(
    zip: &mut ZipWriter<File>,
    dir: &Path,
    prefix: &str,
    options: SimpleFileOptions,
) -> Result<(), ServiceError> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let name = entry.file_name();
        let rel = format!("{prefix}/{}", name.to_string_lossy());
        let path = entry.path();
        if path.is_file() {
            add_file_to_zip(zip, &path, &rel, options)?;
        } else if path.is_dir() {
            add_dir_to_zip(zip, &path, &rel, options)?;
        }
    }
    Ok(())
}

pub fn extract_backup_zip(zip_path: &Path, dest_dir: &Path) -> Result<(), ServiceError> {
    fs::create_dir_all(dest_dir)?;
    let file = File::open(zip_path)?;
    let mut archive = ZipArchive::new(file)
        .map_err(|e| ServiceError::Validation(format!("无效的备份文件：{e}")))?;

    for i in 0..archive.len() {
        let mut entry = archive
            .by_index(i)
            .map_err(|e| ServiceError::Validation(format!("无效的备份文件：{e}")))?;
        let Some(relative) = entry.enclosed_name() else {
            return Err(ServiceError::Validation(
                "无效的备份文件：路径非法".into(),
            ));
        };
        let outpath = dest_dir.join(relative);
        if entry.is_dir() {
            fs::create_dir_all(&outpath)?;
        } else {
            if let Some(parent) = outpath.parent() {
                fs::create_dir_all(parent)?;
            }
            let mut outfile = File::create(&outpath)?;
            std::io::copy(&mut entry, &mut outfile)?;
        }
    }
    Ok(())
}

pub fn validate_backup_contents(backup_dir: &Path) -> Result<(), ServiceError> {
    let manifest_path = backup_dir.join(MANIFEST_NAME);
    if !manifest_path.exists() {
        return Err(ServiceError::Validation(
            "无效的备份文件：缺少 manifest".into(),
        ));
    }
    let content = fs::read_to_string(&manifest_path)?;
    let manifest: BackupManifest = serde_json::from_str(&content)
        .map_err(|e| ServiceError::Validation(format!("无效的备份文件：{e}")))?;

    if manifest.format_version > BACKUP_FORMAT_VERSION {
        return Err(ServiceError::Validation(
            "备份版本过新，请升级应用".into(),
        ));
    }

    let db_path = backup_dir.join("workorder.db");
    if !db_path.exists() {
        return Err(ServiceError::Validation(
            "无效的备份文件：缺少 workorder.db".into(),
        ));
    }

    verify_migration(backup_dir, backup_dir)?;
    Ok(())
}

pub fn clear_data_dir(data_dir: &Path) -> Result<(), ServiceError> {
    if !data_dir.exists() {
        fs::create_dir_all(data_dir)?;
        return Ok(());
    }
    for entry in fs::read_dir(data_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            fs::remove_dir_all(&path)?;
        } else {
            fs::remove_file(&path)?;
        }
    }
    Ok(())
}

pub fn apply_restore(backup_dir: &Path, data_dir: &Path) -> Result<(), ServiceError> {
    validate_backup_contents(backup_dir)?;
    clear_data_dir(data_dir)?;
    copy_data_dir(backup_dir, data_dir)?;
    verify_migration(backup_dir, data_dir)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::connection::open_connection;

    fn temp_dir(prefix: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("{prefix}-{nanos}"))
    }

    #[test]
    fn backup_roundtrip() {
        let data_dir = temp_dir("backup-src");
        let zip_path = temp_dir("backup.zip");
        let restore_dir = temp_dir("backup-restore");

        open_connection(&data_dir).unwrap();
        create_backup_zip(&data_dir, &zip_path).unwrap();
        extract_backup_zip(&zip_path, &restore_dir).unwrap();
        validate_backup_contents(&restore_dir).unwrap();

        let _ = fs::remove_dir_all(&data_dir);
        let _ = fs::remove_file(&zip_path);
        let _ = fs::remove_dir_all(&restore_dir);
    }

    #[test]
    fn rejects_missing_manifest() {
        let dir = temp_dir("backup-no-manifest");
        fs::create_dir_all(&dir).unwrap();
        open_connection(&dir).unwrap();
        assert!(validate_backup_contents(&dir).is_err());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn apply_restore_replaces_data() {
        let data_dir = temp_dir("backup-data");
        let backup_dir = temp_dir("backup-staging");
        open_connection(&data_dir).unwrap();
        copy_data_dir(&data_dir, &backup_dir).unwrap();

        fs::write(data_dir.join("marker.txt"), b"old").unwrap();
        apply_restore(&backup_dir, &data_dir).unwrap();
        assert!(data_dir.join("workorder.db").exists());
        assert!(!data_dir.join("marker.txt").exists());

        let _ = fs::remove_dir_all(&data_dir);
        let _ = fs::remove_dir_all(&backup_dir);
    }
}
