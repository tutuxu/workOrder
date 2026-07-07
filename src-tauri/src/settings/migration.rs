//! 数据目录迁移：递归复制与校验。

use std::fs;
use std::path::{Path, PathBuf};

use rusqlite::Connection;

use crate::error::ServiceError;

/// 规范化路径字符串用于比较（Windows 不区分大小写）。
fn normalize_path(path: &Path) -> PathBuf {
    dunce::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
}

/// 校验目标目录是否可用于迁移。
pub fn validate_target_dir(current: &Path, target: &Path) -> Result<(), ServiceError> {
    let current_norm = normalize_path(current);
    let target_norm = normalize_path(target);

    if current_norm == target_norm {
        return Err(ServiceError::Validation(
            "新目录与当前目录相同".into(),
        ));
    }

    if current_norm.starts_with(&target_norm) || target_norm.starts_with(&current_norm) {
        return Err(ServiceError::Validation(
            "新目录与当前目录存在包含关系，请选择其他位置".into(),
        ));
    }

    if target.exists() {
        let mut entries = fs::read_dir(&target)
            .map_err(|e| ServiceError::Io(e))?
            .peekable();
        if entries.peek().is_none() {
            // empty dir — ok
        } else if target.join("workorder.db").exists() {
            return Err(ServiceError::Validation(
                "目标目录已含 workorder.db，请选择空目录".into(),
            ));
        } else {
            return Err(ServiceError::Validation("目标目录必须为空".into()));
        }
    }

    fs::create_dir_all(target)?;

    // probe writable
    let probe = target.join(".write_probe");
    fs::write(&probe, b"")?;
    fs::remove_file(&probe)?;

    Ok(())
}

/// 递归复制 src 目录下所有条目到 dst（dst 必须已通过 validate_target_dir）。
pub fn copy_data_dir(src: &Path, dst: &Path) -> Result<(), ServiceError> {
    if !src.is_dir() {
        return Err(ServiceError::Validation("源数据目录不存在".into()));
    }
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let name = entry.file_name();
        let from = entry.path();
        let to = dst.join(&name);
        if file_type.is_dir() {
            fs::create_dir_all(&to)?;
            copy_data_dir(&from, &to)?;
        } else {
            fs::copy(&from, &to)?;
        }
    }
    Ok(())
}

/// 复制后校验：workorder.db 存在、大小一致、integrity_check 通过。
pub fn verify_migration(src: &Path, dst: &Path) -> Result<(), ServiceError> {
    let src_db = src.join("workorder.db");
    let dst_db = dst.join("workorder.db");

    if !dst_db.exists() {
        return Err(ServiceError::Validation(
            "迁移后的数据库校验失败：workorder.db 不存在".into(),
        ));
    }

    let src_meta = fs::metadata(&src_db)?;
    let dst_meta = fs::metadata(&dst_db)?;
    if src_meta.len() != dst_meta.len() {
        return Err(ServiceError::Validation(
            "迁移后的数据库校验失败：文件大小不一致".into(),
        ));
    }

    let conn = Connection::open(&dst_db)?;
    let result: String = conn.query_row("PRAGMA integrity_check", [], |row| row.get(0))?;
    if result != "ok" {
        return Err(ServiceError::Validation(format!(
            "迁移后的数据库校验失败：{result}"
        )));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::connection::open_connection;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_dir(prefix: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("{prefix}-{nanos}"))
    }

    #[test]
    fn rejects_same_path() {
        let dir = temp_dir("migration-same");
        fs::create_dir_all(&dir).unwrap();
        assert!(validate_target_dir(&dir, &dir).is_err());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn rejects_subdirectory_target() {
        let parent = temp_dir("migration-parent");
        open_connection(&parent).unwrap();
        let subdir = parent.join("subdir");
        let err = validate_target_dir(&parent, &subdir).unwrap_err();
        assert!(matches!(err, ServiceError::Validation(_)));
        let _ = fs::remove_dir_all(&parent);
    }

    #[test]
    fn rejects_nonempty_without_db() {
        let src = temp_dir("migration-src-ne");
        let dst = temp_dir("migration-dst-ne");
        fs::create_dir_all(&src).unwrap();
        fs::create_dir_all(&dst).unwrap();
        fs::write(dst.join("random.txt"), b"data").unwrap();
        let err = validate_target_dir(&src, &dst).unwrap_err();
        match err {
            ServiceError::Validation(msg) => assert_eq!(msg, "目标目录必须为空"),
            other => panic!("expected Validation, got {other:?}"),
        }
        let _ = fs::remove_dir_all(&src);
        let _ = fs::remove_dir_all(&dst);
    }

    #[test]
    fn rejects_nonempty_target_with_db() {
        let src = temp_dir("migration-src");
        let dst = temp_dir("migration-dst");
        open_connection(&src).unwrap();
        fs::create_dir_all(&dst).unwrap();
        fs::copy(src.join("workorder.db"), dst.join("workorder.db")).unwrap();
        assert!(validate_target_dir(&src, &dst).is_err());
        let _ = fs::remove_dir_all(&src);
        let _ = fs::remove_dir_all(&dst);
    }

    #[test]
    fn copy_and_verify_succeeds() {
        let src = temp_dir("migration-copy-src");
        let dst = temp_dir("migration-copy-dst");
        open_connection(&src).unwrap();
        validate_target_dir(&src, &dst).unwrap();
        copy_data_dir(&src, &dst).unwrap();
        verify_migration(&src, &dst).unwrap();
        let _ = fs::remove_dir_all(&src);
        let _ = fs::remove_dir_all(&dst);
    }
}
