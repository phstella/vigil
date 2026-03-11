//! Workspace and vault models.
//!
//! `WorkspaceRoot` is the top-level container for an opened workspace.
//! `Vault` is the logical namespace (1:1 with workspace in MVP).

use serde::{Deserialize, Serialize};

/// The top-level container representing an opened workspace directory.
///
/// All file operations resolve against `canonical_path` and reject path
/// traversal outside the root.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceRoot {
    /// Stable hex-encoded hash of `canonical_path` (deterministic across sessions).
    pub workspace_id: String,
    /// Absolute canonical path to workspace root.
    pub canonical_path: String,
    /// Unix epoch ms when workspace was opened.
    pub opened_at_ms: i64,
}

/// Response payload for the `open_workspace` command.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenWorkspaceResponse {
    /// Stable hash of canonical root path.
    pub workspace_id: String,
    /// Canonicalized absolute path.
    pub canonical_path: String,
    /// Number of `.md` files found.
    pub notes_count: u64,
    /// Total indexed file count.
    pub files_count: u64,
    /// Unix epoch ms when workspace was opened.
    pub opened_at_ms: i64,
}

/// Logical namespace for a workspace. 1:1 with `WorkspaceRoot` in MVP.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vault {
    /// References `WorkspaceRoot.workspace_id`.
    pub workspace_id: String,
    /// Display name (derived from directory name).
    pub name: String,
    /// Count of `.md` files.
    pub notes_count: u64,
    /// Total indexed file count.
    pub files_count: u64,
    /// Unique tag count across all notes.
    pub tags_count: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn workspace_root_roundtrip() {
        let root = WorkspaceRoot {
            workspace_id: "abc123".into(),
            canonical_path: "/home/user/notes".into(),
            opened_at_ms: 1_700_000_000_000,
        };
        let json = serde_json::to_string(&root).unwrap();
        let deser: WorkspaceRoot = serde_json::from_str(&json).unwrap();
        assert_eq!(deser.workspace_id, "abc123");
        assert_eq!(deser.canonical_path, "/home/user/notes");
        assert_eq!(deser.opened_at_ms, 1_700_000_000_000);
    }

    #[test]
    fn vault_roundtrip() {
        let vault = Vault {
            workspace_id: "abc123".into(),
            name: "My Notes".into(),
            notes_count: 42,
            files_count: 100,
            tags_count: 15,
        };
        let json = serde_json::to_string(&vault).unwrap();
        let deser: Vault = serde_json::from_str(&json).unwrap();
        assert_eq!(deser.name, "My Notes");
        assert_eq!(deser.notes_count, 42);
    }

    #[test]
    fn open_workspace_response_roundtrip() {
        let resp = OpenWorkspaceResponse {
            workspace_id: "xyz789".into(),
            canonical_path: "/tmp/workspace".into(),
            notes_count: 10,
            files_count: 50,
            opened_at_ms: 1_700_000_000_000,
        };
        let json = serde_json::to_string(&resp).unwrap();
        let deser: OpenWorkspaceResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(deser.workspace_id, "xyz789");
        assert_eq!(deser.files_count, 50);
    }
}
