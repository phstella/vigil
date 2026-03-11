//! Error envelope for Vigil IPC.
//!
//! Every `#[tauri::command]` returns `Result<T, VigilError>`. Tauri serializes
//! the error branch into a JSON string via `Into<tauri::ipc::InvokeError>`.

use serde::Serialize;

/// Typed error codes that map 1:1 to the IPC contract error table.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrorCode {
    WorkspaceNotOpen,
    PathOutsideWorkspace,
    FileNotFound,
    FileAlreadyExists,
    PermissionDenied,
    InvalidArgument,
    BinaryFile,
    StaleEtag,
    IndexUnavailable,
    GitUnavailable,
    PluginError,
    InternalError,
}

/// Top-level error type for all Vigil backend operations.
///
/// Uses `thiserror` for ergonomic `Display` + `Error` derivation and carries
/// a structured [`ErrorCode`] so the frontend can branch on machine-readable
/// codes without parsing message strings.
#[derive(Debug, thiserror::Error)]
pub enum VigilError {
    #[error("No workspace is currently open")]
    WorkspaceNotOpen,

    #[error("Path escapes workspace root: {path}")]
    PathOutsideWorkspace { path: String },

    #[error("File not found: {path}")]
    FileNotFound { path: String },

    #[error("File already exists: {path}")]
    FileAlreadyExists { path: String },

    #[error("Permission denied: {path}")]
    PermissionDenied { path: String },

    #[error("Invalid argument: {reason}")]
    InvalidArgument { reason: String },

    #[error("Binary file not supported: {path}")]
    BinaryFile { path: String },

    #[error("Stale etag – file was modified externally")]
    StaleEtag,

    #[error("File index is not yet ready")]
    IndexUnavailable,

    #[error("Git operation failed: {reason}")]
    GitUnavailable { reason: String },

    #[error("Plugin error: {reason}")]
    PluginError { reason: String },

    #[error("Internal error: {reason}")]
    InternalError { reason: String },
}

impl VigilError {
    /// Machine-readable error code for frontend branching.
    pub fn code(&self) -> ErrorCode {
        match self {
            Self::WorkspaceNotOpen => ErrorCode::WorkspaceNotOpen,
            Self::PathOutsideWorkspace { .. } => ErrorCode::PathOutsideWorkspace,
            Self::FileNotFound { .. } => ErrorCode::FileNotFound,
            Self::FileAlreadyExists { .. } => ErrorCode::FileAlreadyExists,
            Self::PermissionDenied { .. } => ErrorCode::PermissionDenied,
            Self::InvalidArgument { .. } => ErrorCode::InvalidArgument,
            Self::BinaryFile { .. } => ErrorCode::BinaryFile,
            Self::StaleEtag => ErrorCode::StaleEtag,
            Self::IndexUnavailable => ErrorCode::IndexUnavailable,
            Self::GitUnavailable { .. } => ErrorCode::GitUnavailable,
            Self::PluginError { .. } => ErrorCode::PluginError,
            Self::InternalError { .. } => ErrorCode::InternalError,
        }
    }
}

/// Serializable error envelope sent to the frontend.
///
/// Matches the IPC contract error shape:
/// ```json
/// { "code": "ERROR_CODE", "message": "Human-readable description", "details": {} }
/// ```
#[derive(Debug, Clone, Serialize)]
pub struct ErrorEnvelope {
    pub code: ErrorCode,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

impl From<VigilError> for ErrorEnvelope {
    fn from(err: VigilError) -> Self {
        Self {
            code: err.code(),
            message: err.to_string(),
            details: None,
        }
    }
}

/// Convert `VigilError` into Tauri's IPC error type so `#[tauri::command]`
/// functions can use `Result<T, VigilError>` directly.
impl From<VigilError> for tauri::ipc::InvokeError {
    fn from(err: VigilError) -> Self {
        let envelope = ErrorEnvelope::from(err);
        // Tauri serializes this JSON value and delivers it to the frontend
        // reject handler.
        let value = serde_json::to_value(envelope).unwrap_or_else(|e| {
            serde_json::json!({
                "code": "INTERNAL_ERROR",
                "message": format!("Failed to serialize error: {e}"),
            })
        });
        tauri::ipc::InvokeError::from(value)
    }
}

// ---------------------------------------------------------------------------
// Convenience conversions from common library errors
// ---------------------------------------------------------------------------

impl From<std::io::Error> for VigilError {
    fn from(err: std::io::Error) -> Self {
        match err.kind() {
            std::io::ErrorKind::NotFound => Self::FileNotFound {
                path: err.to_string(),
            },
            std::io::ErrorKind::PermissionDenied => Self::PermissionDenied {
                path: err.to_string(),
            },
            _ => Self::InternalError {
                reason: err.to_string(),
            },
        }
    }
}

impl From<git2::Error> for VigilError {
    fn from(err: git2::Error) -> Self {
        Self::GitUnavailable {
            reason: err.message().to_owned(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_code_serializes_screaming_snake() {
        let json = serde_json::to_string(&ErrorCode::WorkspaceNotOpen).unwrap();
        assert_eq!(json, "\"WORKSPACE_NOT_OPEN\"");

        let json = serde_json::to_string(&ErrorCode::PathOutsideWorkspace).unwrap();
        assert_eq!(json, "\"PATH_OUTSIDE_WORKSPACE\"");

        let json = serde_json::to_string(&ErrorCode::StaleEtag).unwrap();
        assert_eq!(json, "\"STALE_ETAG\"");
    }

    #[test]
    fn error_envelope_from_vigil_error() {
        let err = VigilError::FileNotFound {
            path: "notes/hello.md".into(),
        };
        let envelope = ErrorEnvelope::from(err);
        assert_eq!(envelope.code, ErrorCode::FileNotFound);
        assert!(envelope.message.contains("notes/hello.md"));
        assert!(envelope.details.is_none());
    }

    #[test]
    fn io_error_converts_to_vigil_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "gone");
        let vigil_err = VigilError::from(io_err);
        assert_eq!(vigil_err.code(), ErrorCode::FileNotFound);

        let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "nope");
        let vigil_err = VigilError::from(io_err);
        assert_eq!(vigil_err.code(), ErrorCode::PermissionDenied);

        let io_err = std::io::Error::other("boom");
        let vigil_err = VigilError::from(io_err);
        assert_eq!(vigil_err.code(), ErrorCode::InternalError);
    }
}
