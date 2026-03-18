//! Event payloads and emitters for workspace file rename/move changes.
//!
//! All events use the `vigil://` prefix per the IPC contract.
//! Payloads include `timestamp_ms` and `contract_version` metadata fields.

use std::time::{SystemTime, UNIX_EPOCH};

use serde::Serialize;
use tauri::{AppHandle, Emitter};

/// Event name for file rename/move notifications.
pub const FS_RENAMED_EVENT: &str = "vigil://fs-renamed";

/// Contract version included in all event payloads.
const CONTRACT_VERSION: &str = "v1";

/// Payload for `vigil://fs-renamed`.
#[derive(Debug, Clone, Serialize)]
pub struct FsRenamedPayload {
    /// Previous workspace-relative path.
    pub old_path: String,
    /// New workspace-relative path.
    pub new_path: String,
    /// Unix epoch ms when event was emitted.
    pub timestamp_ms: i64,
    /// Always "v1".
    pub contract_version: String,
}

/// Emit a `vigil://fs-renamed` event for a successful rename/move operation.
pub fn emit_fs_renamed(app_handle: &AppHandle, old_path: &str, new_path: &str) {
    let payload = FsRenamedPayload {
        old_path: old_path.to_string(),
        new_path: new_path.to_string(),
        timestamp_ms: now_epoch_ms(),
        contract_version: CONTRACT_VERSION.to_string(),
    };

    if let Err(e) = app_handle.emit(FS_RENAMED_EVENT, &payload) {
        eprintln!("Failed to emit {FS_RENAMED_EVENT}: {e}");
    }
}

/// Current time as unix epoch milliseconds.
fn now_epoch_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fs_renamed_payload_serializes() {
        let payload = FsRenamedPayload {
            old_path: "a.md".to_string(),
            new_path: "docs/a.md".to_string(),
            timestamp_ms: 1_700_000_000_000,
            contract_version: "v1".to_string(),
        };
        let json = serde_json::to_string(&payload).unwrap();
        assert!(json.contains("\"contract_version\":\"v1\""));
        assert!(json.contains("\"old_path\":\"a.md\""));
        assert!(json.contains("\"new_path\":\"docs/a.md\""));
    }
}
