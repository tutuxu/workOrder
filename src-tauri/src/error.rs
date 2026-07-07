//! 后端统一错误类型，Command 层序列化为 JSON 字符串返回前端。

use thiserror::Error;

/// Service 层错误，Command 层映射为 `String` 返回给前端。
#[derive(Debug, Error)]
pub enum ServiceError {
    /// 资源不存在（工单、进度日志等）。
    #[error("not found: {0}")]
    NotFound(String),
    /// 业务校验失败（空标题、空进度内容、归属不匹配等）。
    #[error("validation: {0}")]
    Validation(String),
    /// SQLite 读写错误。
    #[error("database: {0}")]
    Database(#[from] rusqlite::Error),
    /// 文件系统错误（创建 data 目录等）。
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
}

impl serde::Serialize for ServiceError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let (code, message) = match self {
            ServiceError::NotFound(msg) => ("NOT_FOUND", msg.clone()),
            ServiceError::Validation(msg) => ("VALIDATION", msg.clone()),
            ServiceError::Database(err) => ("DATABASE", err.to_string()),
            ServiceError::Io(err) => ("IO", err.to_string()),
        };
        serializer.serialize_str(&format!("{{\"code\":\"{code}\",\"message\":{}}}", serde_json::to_string(&message).unwrap_or_default()))
    }
}
