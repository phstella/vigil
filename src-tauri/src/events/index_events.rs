//! Event payloads and emitters for file index changes.
//!
//! All events use the `vigil://` prefix per the IPC contract.
//! Payloads include `timestamp_ms` and `contract_version` metadata fields.

use std::time::{SystemTime, UNIX_EPOCH};

use serde::Serialize;
use tauri::{AppHandle, Emitter};

use crate::core::index::{ChangeKind, IndexChangeRecord};
use crate::models::files::EntryKind;

/// Event name for index update notifications.
pub const INDEX_UPDATED_EVENT: &str = "vigil://index-updated";

/// Event name for index ready notification.
pub const INDEX_READY_EVENT: &str = "vigil://index-ready";

/// Contract version included in all event payloads.
const CONTRACT_VERSION: &str = "v1";

/// A single index change entry for the `vigil://index-updated` event.
#[derive(Debug, Clone, Serialize)]
pub struct IndexChange {
    /// Workspace-relative path.
    pub path: String,
    /// What happened.
    pub change_type: String,
    /// Entry type.
    pub kind: String,
}

/// Payload for `vigil://index-updated`.
#[derive(Debug, Clone, Serialize)]
pub struct IndexUpdatedPayload {
    /// List of changed entries.
    pub changes: Vec<IndexChange>,
    /// Unix epoch ms when event was emitted.
    pub timestamp_ms: i64,
    /// Always "v1".
    pub contract_version: String,
}

/// Payload for `vigil://index-ready`.
#[derive(Debug, Clone, Serialize)]
pub struct IndexReadyPayload {
    /// Total files indexed.
    pub files_count: u64,
    /// Markdown files indexed.
    pub notes_count: u64,
    /// Time taken for initial scan in milliseconds.
    pub duration_ms: u64,
    /// Unix epoch ms when event was emitted.
    pub timestamp_ms: i64,
    /// Always "v1".
    pub contract_version: String,
}

/// Convert an `IndexChangeRecord` to the serializable `IndexChange`.
fn record_to_change(record: &IndexChangeRecord) -> IndexChange {
    IndexChange {
        path: record.path.clone(),
        change_type: match record.change_kind {
            ChangeKind::Created => "created".to_string(),
            ChangeKind::Changed => "changed".to_string(),
            ChangeKind::Deleted => "deleted".to_string(),
        },
        kind: match record.entry_kind {
            EntryKind::File => "file".to_string(),
            EntryKind::Dir => "dir".to_string(),
        },
    }
}

/// Emit a `vigil://index-updated` event with the given change records.
pub fn emit_index_updated(app_handle: &AppHandle, records: &[IndexChangeRecord]) {
    if records.is_empty() {
        return;
    }

    let payload = IndexUpdatedPayload {
        changes: records.iter().map(record_to_change).collect(),
        timestamp_ms: now_epoch_ms(),
        contract_version: CONTRACT_VERSION.to_string(),
    };

    if let Err(e) = app_handle.emit(INDEX_UPDATED_EVENT, &payload) {
        eprintln!("Failed to emit {INDEX_UPDATED_EVENT}: {e}");
    }
}

/// Emit a `vigil://index-ready` event after the initial scan completes.
pub fn emit_index_ready(
    app_handle: &AppHandle,
    files_count: u64,
    notes_count: u64,
    duration_ms: u64,
) {
    let payload = IndexReadyPayload {
        files_count,
        notes_count,
        duration_ms,
        timestamp_ms: now_epoch_ms(),
        contract_version: CONTRACT_VERSION.to_string(),
    };

    if let Err(e) = app_handle.emit(INDEX_READY_EVENT, &payload) {
        eprintln!("Failed to emit {INDEX_READY_EVENT}: {e}");
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
    use crate::core::index::ChangeKind;
    use crate::models::files::EntryKind;

    #[test]
    fn record_to_change_created() {
        let record = IndexChangeRecord {
            path: "notes/hello.md".to_string(),
            change_kind: ChangeKind::Created,
            entry_kind: EntryKind::File,
        };
        let change = record_to_change(&record);
        assert_eq!(change.path, "notes/hello.md");
        assert_eq!(change.change_type, "created");
        assert_eq!(change.kind, "file");
    }

    #[test]
    fn record_to_change_deleted_dir() {
        let record = IndexChangeRecord {
            path: "old-folder".to_string(),
            change_kind: ChangeKind::Deleted,
            entry_kind: EntryKind::Dir,
        };
        let change = record_to_change(&record);
        assert_eq!(change.change_type, "deleted");
        assert_eq!(change.kind, "dir");
    }

    #[test]
    fn index_updated_payload_serializes() {
        let payload = IndexUpdatedPayload {
            changes: vec![IndexChange {
                path: "test.md".to_string(),
                change_type: "changed".to_string(),
                kind: "file".to_string(),
            }],
            timestamp_ms: 1_700_000_000_000,
            contract_version: "v1".to_string(),
        };
        let json = serde_json::to_string(&payload).unwrap();
        assert!(json.contains("\"contract_version\":\"v1\""));
        assert!(json.contains("\"change_type\":\"changed\""));
    }

    #[test]
    fn index_ready_payload_serializes() {
        let payload = IndexReadyPayload {
            files_count: 100,
            notes_count: 42,
            duration_ms: 350,
            timestamp_ms: 1_700_000_000_000,
            contract_version: "v1".to_string(),
        };
        let json = serde_json::to_string(&payload).unwrap();
        assert!(json.contains("\"files_count\":100"));
        assert!(json.contains("\"notes_count\":42"));
        assert!(json.contains("\"duration_ms\":350"));
    }
}
