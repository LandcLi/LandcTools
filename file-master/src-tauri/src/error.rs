//! 统一错误类型：所有 command 返回 `Result<T, AppError>`，
//! AppError 序列化为字符串，前端可直接展示。

use serde::{Serialize, Serializer};

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("{0}")]
    Io(String),

    #[error("{0}")]
    Invalid(String),

    #[error("路径不存在：{0}")]
    NotFound(String),

    #[error("权限不足：{0}")]
    Denied(String),

    #[error("{0}")]
    Unsupported(String),

    #[error("{0}")]
    Internal(String),
}

impl Serialize for AppError {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self {
        match e.kind() {
            std::io::ErrorKind::NotFound => AppError::NotFound(e.to_string()),
            std::io::ErrorKind::PermissionDenied => AppError::Denied(e.to_string()),
            _ => AppError::Io(e.to_string()),
        }
    }
}

impl From<anyhow::Error> for AppError {
    fn from(e: anyhow::Error) -> Self {
        AppError::Internal(format!("{e:#}"))
    }
}

pub type AppResult<T> = Result<T, AppError>;
