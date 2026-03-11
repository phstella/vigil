//! Event payloads and emitters for git diff/hunk changes.
//!
//! All events use the `vigil://` prefix per the IPC contract.
//! Payloads include `timestamp_ms` and `contract_version` metadata fields.

use std::time::{SystemTime, UNIX_EPOCH};

use serde::Serialize;
use tauri::{AppHandle, Emitter};

use crate::models::git::GitHunk;

/// Event name for git hunk change notifications.
pub const GIT_HUNKS_EVENT: &str = "vigil://git-hunks";

/// Contract version included in all event payloads.
const CONTRACT_VERSION: &str = "v1";

/// Payload for `vigil://git-hunks`.
#[derive(Debug, Clone, Serialize)]
pub struct GitHunksPayload {
    /// Workspace-relative file path.
    pub path: String,
    /// Updated hunk list.
    pub hunks: Vec<GitHunk>,
    /// Unix epoch ms when event was emitted.
    pub timestamp_ms: i64,
    /// Always "v1".
    pub contract_version: String,
}

/// Emit a `vigil://git-hunks` event with updated hunks for a file.
pub fn emit_git_hunks(app_handle: &AppHandle, path: &str, hunks: Vec<GitHunk>) {
    let payload = GitHunksPayload {
        path: path.to_string(),
        hunks,
        timestamp_ms: now_epoch_ms(),
        contract_version: CONTRACT_VERSION.to_string(),
    };

    if let Err(e) = app_handle.emit(GIT_HUNKS_EVENT, &payload) {
        eprintln!("Failed to emit {GIT_HUNKS_EVENT}: {e}");
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
    use crate::models::git::HunkChangeType;

    #[test]
    fn git_hunks_payload_serializes() {
        let payload = GitHunksPayload {
            path: "notes/hello.md".to_string(),
            hunks: vec![GitHunk {
                change_type: HunkChangeType::Added,
                start_line: 5,
                end_line: 10,
                base_start_line: None,
                base_end_line: None,
            }],
            timestamp_ms: 1_700_000_000_000,
            contract_version: "v1".to_string(),
        };
        let json = serde_json::to_string(&payload).unwrap();
        assert!(json.contains("\"contract_version\":\"v1\""));
        assert!(json.contains("\"path\":\"notes/hello.md\""));
        assert!(json.contains("\"change_type\":\"added\""));
        assert!(json.contains("\"start_line\":5"));
    }

    #[test]
    fn git_hunks_payload_empty_hunks() {
        let payload = GitHunksPayload {
            path: "test.md".to_string(),
            hunks: vec![],
            timestamp_ms: 1_700_000_000_000,
            contract_version: "v1".to_string(),
        };
        let json = serde_json::to_string(&payload).unwrap();
        assert!(json.contains("\"hunks\":[]"));
    }
}
