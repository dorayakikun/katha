use std::path::PathBuf;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum KathaError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("JSON parse error: {0}")]
    JsonError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Session not found: {0}")]
    SessionNotFound(String),

    #[error("Terminal error: {0}")]
    Terminal(String),

    #[error("Export failed: {0}")]
    ExportError(String),

    #[error("File already exists: {0}")]
    FileExists(PathBuf),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),
}

impl KathaError {
    /// ユーザーフレンドリーなエラーメッセージを取得
    pub fn user_message(&self) -> String {
        match self {
            KathaError::IoError(e) => format!("File operation failed: {}", e),
            KathaError::JsonError(msg) => format!("Failed to parse data: {}", msg),
            KathaError::ConfigError(msg) => format!("Configuration error: {}", msg),
            KathaError::SessionNotFound(id) => format!("Session '{}' not found", id),
            KathaError::Terminal(msg) => format!("Terminal error: {}", msg),
            KathaError::ExportError(msg) => format!("Export failed: {}", msg),
            KathaError::FileExists(path) => {
                format!("File already exists: {}", path.display())
            }
            KathaError::PermissionDenied(msg) => format!("Permission denied: {}", msg),
        }
    }
}
