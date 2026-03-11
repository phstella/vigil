//! Git diff hunk models for gutter decorations.

use serde::{Deserialize, Serialize};

/// Classification of a diff hunk change.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HunkChangeType {
    Added,
    Modified,
    Deleted,
}

/// A line-level diff hunk representing a change against git HEAD.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHunk {
    /// Hunk classification.
    pub change_type: HunkChangeType,
    /// First affected line in working copy (1-based).
    pub start_line: u32,
    /// Last affected line in working copy (1-based).
    pub end_line: u32,
    /// First line in base version (None for additions).
    pub base_start_line: Option<u32>,
    /// Last line in base version (None for additions).
    pub base_end_line: Option<u32>,
}

/// Response payload for the `get_git_hunks` command.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHunksResponse {
    /// List of diff hunks.
    pub hunks: Vec<GitHunk>,
}

/// High-level git status of a single file in the working tree.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GitFileStatus {
    /// File is tracked and unmodified.
    Clean,
    /// File has been modified since HEAD.
    Modified,
    /// File is new (untracked or staged as new).
    New,
    /// File has been deleted.
    Deleted,
    /// File has been renamed.
    Renamed,
    /// File has a merge conflict.
    Conflicted,
    /// Status could not be determined.
    Unknown,
}

/// Git status entry for a single file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitStatusEntry {
    /// Workspace-relative file path.
    pub path: String,
    /// High-level status.
    pub status: GitFileStatus,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hunk_change_type_serializes_lowercase() {
        assert_eq!(
            serde_json::to_string(&HunkChangeType::Added).unwrap(),
            "\"added\""
        );
        assert_eq!(
            serde_json::to_string(&HunkChangeType::Modified).unwrap(),
            "\"modified\""
        );
        assert_eq!(
            serde_json::to_string(&HunkChangeType::Deleted).unwrap(),
            "\"deleted\""
        );
    }

    #[test]
    fn git_hunk_roundtrip() {
        let hunk = GitHunk {
            change_type: HunkChangeType::Added,
            start_line: 10,
            end_line: 15,
            base_start_line: None,
            base_end_line: None,
        };
        let json = serde_json::to_string(&hunk).unwrap();
        let deser: GitHunk = serde_json::from_str(&json).unwrap();
        assert_eq!(deser.change_type, HunkChangeType::Added);
        assert!(deser.base_start_line.is_none());
    }

    #[test]
    fn git_hunk_modified_with_base_lines() {
        let hunk = GitHunk {
            change_type: HunkChangeType::Modified,
            start_line: 20,
            end_line: 25,
            base_start_line: Some(18),
            base_end_line: Some(22),
        };
        let json = serde_json::to_string(&hunk).unwrap();
        let deser: GitHunk = serde_json::from_str(&json).unwrap();
        assert_eq!(deser.base_start_line, Some(18));
        assert_eq!(deser.base_end_line, Some(22));
    }

    #[test]
    fn git_file_status_serializes_lowercase() {
        assert_eq!(
            serde_json::to_string(&GitFileStatus::Modified).unwrap(),
            "\"modified\""
        );
        assert_eq!(
            serde_json::to_string(&GitFileStatus::Conflicted).unwrap(),
            "\"conflicted\""
        );
    }
}
