use thiserror::Error;

#[derive(Debug, Error)]
pub enum ServiceError {
    #[error("not found: {0}")]
    NotFound(String),
    #[error("validation: {0}")]
    Validation(String),
    #[error("database: {0}")]
    Database(#[from] rusqlite::Error),
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
