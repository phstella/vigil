//! Workspace status model for the footer status bar.

use serde::{Deserialize, Serialize};

/// Sync state between local branch and upstream tracking branch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SyncState {
    Synced,
    Ahead,
    Behind,
    Diverged,
    Unknown,
}

/// Composite status for the footer status bar.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceStatus {
    /// Current git branch name, or None if not a git repo.
    pub branch: Option<String>,
    /// Sync state relative to upstream.
    pub sync_state: SyncState,
    /// Number of `.md` files.
    pub notes_count: u64,
    /// Number of unique tags.
    pub tags_count: u64,
    /// Total indexed file count.
    pub files_count: u64,
    /// Application version string (from `Cargo.toml`).
    pub version: String,
    /// Timestamp of last index refresh.
    pub last_index_update_ms: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sync_state_serializes_lowercase() {
        assert_eq!(
            serde_json::to_string(&SyncState::Synced).unwrap(),
            "\"synced\""
        );
        assert_eq!(
            serde_json::to_string(&SyncState::Ahead).unwrap(),
            "\"ahead\""
        );
        assert_eq!(
            serde_json::to_string(&SyncState::Behind).unwrap(),
            "\"behind\""
        );
        assert_eq!(
            serde_json::to_string(&SyncState::Diverged).unwrap(),
            "\"diverged\""
        );
        assert_eq!(
            serde_json::to_string(&SyncState::Unknown).unwrap(),
            "\"unknown\""
        );
    }

    #[test]
    fn workspace_status_roundtrip() {
        let status = WorkspaceStatus {
            branch: Some("main".into()),
            sync_state: SyncState::Ahead,
            notes_count: 42,
            tags_count: 15,
            files_count: 100,
            version: "0.0.1".into(),
            last_index_update_ms: 1_700_000_000_000,
        };
        let json = serde_json::to_string(&status).unwrap();
        let deser: WorkspaceStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(deser.branch.as_deref(), Some("main"));
        assert_eq!(deser.sync_state, SyncState::Ahead);
        assert_eq!(deser.notes_count, 42);
    }

    #[test]
    fn workspace_status_no_branch() {
        let status = WorkspaceStatus {
            branch: None,
            sync_state: SyncState::Unknown,
            notes_count: 0,
            tags_count: 0,
            files_count: 0,
            version: "0.0.1".into(),
            last_index_update_ms: 0,
        };
        let json = serde_json::to_string(&status).unwrap();
        let deser: WorkspaceStatus = serde_json::from_str(&json).unwrap();
        assert!(deser.branch.is_none());
        assert_eq!(deser.sync_state, SyncState::Unknown);
    }
}
