//! SQLite 日期时间解析、格式化与列读取（兼容 Java 遗留 epoch 毫秒）。

use chrono::{NaiveDateTime, TimeZone, Utc};
use rusqlite::types::Value;
use rusqlite::Row;

use crate::error::ServiceError;

const DATETIME_FMT: &str = "%Y-%m-%dT%H:%M:%S";

/// 解析 ISO 或常见格式的日期时间字符串。
pub fn parse_datetime_str(value: &str) -> Result<NaiveDateTime, ServiceError> {
    NaiveDateTime::parse_from_str(value, DATETIME_FMT)
        .or_else(|_| NaiveDateTime::parse_from_str(value, "%Y-%m-%d %H:%M:%S"))
        .or_else(|_| NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%M:%S%.f"))
        .map_err(|_| ServiceError::Validation(format!("invalid datetime: {value}")))
}

/// 格式化为 `%Y-%m-%dT%H:%M:%S` 字符串写入 SQLite。
pub fn format_datetime(value: NaiveDateTime) -> String {
    value.format(DATETIME_FMT).to_string()
}

/// 从 epoch 毫秒（或秒）转换为 UTC naive 时间。
pub fn from_epoch_millis(raw: i64) -> Result<NaiveDateTime, ServiceError> {
    let millis = if raw.abs() > 1_000_000_000_000 {
        raw
    } else {
        raw * 1000
    };
    Utc.timestamp_millis_opt(millis)
        .single()
        .map(|dt| dt.naive_utc())
        .ok_or_else(|| ServiceError::Validation(format!("invalid epoch millis: {raw}")))
}

/// 从 SQLite `Value` 读取日期时间（支持整数 epoch 与文本）。
pub fn read_datetime_value(value: &Value) -> Result<NaiveDateTime, ServiceError> {
    match value {
        Value::Integer(n) => from_epoch_millis(*n),
        Value::Text(s) => parse_datetime_str(s),
        Value::Real(f) => from_epoch_millis(*f as i64),
        Value::Null => Err(ServiceError::Validation("datetime is null".into())),
        Value::Blob(_) => Err(ServiceError::Validation("datetime blob unsupported".into())),
    }
}

/// 从查询行读取必填日期时间列。
pub fn read_datetime_column(row: &Row<'_>, column: &str) -> Result<NaiveDateTime, rusqlite::Error> {
    let value: Value = row.get(column)?;
    read_datetime_value(&value)
        .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))
}

/// 从查询行读取可选日期时间列，`NULL` 映射为 `None`。
pub fn read_optional_datetime_column(
    row: &Row<'_>,
    column: &str,
) -> Result<Option<NaiveDateTime>, rusqlite::Error> {
    let value: Value = row.get(column)?;
    match value {
        Value::Null => Ok(None),
        other => read_datetime_value(&other)
            .map(Some)
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_java_epoch_millis() {
        let dt = from_epoch_millis(1_783_324_852_840).unwrap();
        assert_eq!(dt.format("%Y-%m-%d").to_string(), "2026-07-06");
    }
}
