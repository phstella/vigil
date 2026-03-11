//! Event payloads and emitters for workspace status changes.
//!
//! All events use the `vigil://` prefix per the IPC contract.
//! Payloads include `timestamp_ms` and `contract_version` metadata fields.

use std::time::{SystemTime, UNIX_EPOCH};

use serde::Serialize;
use tauri::{AppHandle, Emitter};

use crate::models::status::WorkspaceStatus;

/// Event name for workspace status update notifications.
pub const STATUS_UPDATED_EVENT: &str = "vigil://status-updated";

/// Contract version included in all event payloads.
const CONTRACT_VERSION: &str = "v1";

/// Payload for `vigil://status-updated`.
#[derive(Debug, Clone, Serialize)]
pub struct StatusUpdatedPayload {
    /// The updated workspace status.
    pub status: WorkspaceStatus,
    /// Unix epoch ms when event was emitted.
    pub timestamp_ms: i64,
    /// Always "v1".
    pub contract_version: String,
}

/// Emit a `vigil://status-updated` event with the current workspace status.
pub fn emit_status_updated(app_handle: &AppHandle, status: &WorkspaceStatus) {
    let payload = StatusUpdatedPayload {
        status: status.clone(),
        timestamp_ms: now_epoch_ms(),
        contract_version: CONTRACT_VERSION.to_string(),
    };

    if let Err(e) = app_handle.emit(STATUS_UPDATED_EVENT, &payload) {
        eprintln!("Failed to emit {STATUS_UPDATED_EVENT}: {e}");
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
    use crate::models::status::SyncState;

    #[test]
    fn status_updated_payload_serializes() {
        let status = WorkspaceStatus {
            branch: Some("main".into()),
            sync_state: SyncState::Synced,
            notes_count: 42,
            tags_count: 10,
            files_count: 100,
            version: "0.0.1".into(),
            last_index_update_ms: 1_700_000_000_000,
        };
        let payload = StatusUpdatedPayload {
            status,
            timestamp_ms: 1_700_000_000_000,
            contract_version: "v1".to_string(),
        };
        let json = serde_json::to_string(&payload).unwrap();
        assert!(json.contains("\"contract_version\":\"v1\""));
        assert!(json.contains("\"branch\":\"main\""));
        assert!(json.contains("\"sync_state\":\"synced\""));
        assert!(json.contains("\"notes_count\":42"));
    }

    #[test]
    fn status_updated_payload_with_no_branch() {
        let status = WorkspaceStatus {
            branch: None,
            sync_state: SyncState::Unknown,
            notes_count: 0,
            tags_count: 0,
            files_count: 0,
            version: "0.0.1".into(),
            last_index_update_ms: 0,
        };
        let payload = StatusUpdatedPayload {
            status,
            timestamp_ms: 1_700_000_000_000,
            contract_version: "v1".to_string(),
        };
        let json = serde_json::to_string(&payload).unwrap();
        assert!(json.contains("\"branch\":null"));
        assert!(json.contains("\"sync_state\":\"unknown\""));
    }
}
