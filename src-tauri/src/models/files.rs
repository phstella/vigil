//! File and directory entry models, note metadata, and request types for
//! file CRUD operations.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// The kind of a filesystem entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EntryKind {
    File,
    Dir,
}

/// Full file entry held in the in-memory index.
///
/// `absolute_path` is skipped during serialization so it is never sent over IPC.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    /// Workspace-relative path (forward slashes).
    pub path: String,

    /// Full filesystem path (backend only, never sent to frontend).
    #[serde(skip)]
    pub absolute_path: PathBuf,

    /// File or directory name.
    pub name: String,

    /// Entry type.
    pub kind: EntryKind,

    /// File extension without leading dot, or `None`.
    pub ext: Option<String>,

    /// File size in bytes (0 for directories).
    pub size_bytes: u64,

    /// Last modification unix epoch ms.
    pub modified_at_ms: i64,

    /// Starts with `.` or OS hidden attribute.
    pub is_hidden: bool,

    /// Detected by null-byte scan of first 8 KB.
    pub is_binary: bool,
}

/// Subset of [`FileEntry`] sent to the frontend for directory listings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirEntry {
    /// File or directory name.
    pub name: String,
    /// Workspace-relative path.
    pub path: String,
    /// Entry type.
    pub kind: EntryKind,
    /// File extension without dot, or `None`.
    pub ext: Option<String>,
    /// File size (None for directories).
    pub size_bytes: Option<u64>,
    /// Last modification unix epoch ms (None for directories).
    pub modified_at_ms: Option<i64>,
    /// Hidden file flag.
    pub is_hidden: bool,
}

/// Response payload for the `list_dir` command.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListDirResponse {
    /// Sorted directory entries.
    pub entries: Vec<DirEntry>,
    /// `true` if entry count exceeded the 50,000 limit.
    pub truncated: bool,
}

/// Response payload for the `read_file` command.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadFileResponse {
    /// UTF-8 text content.
    pub content: String,
    /// Detected encoding (always `"utf-8"` for MVP).
    pub encoding: String,
    /// File size in bytes.
    pub size_bytes: u64,
    /// Last modification unix epoch ms.
    pub modified_at_ms: i64,
    /// Content hash for optimistic concurrency.
    pub etag: String,
}

/// Request payload for the `write_file` command.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WriteFileRequest {
    /// Workspace-relative file path.
    pub path: String,
    /// New UTF-8 file content.
    pub content: String,
    /// Expected etag; if set, write fails on mismatch.
    pub etag: Option<String>,
}

/// Response payload for the `write_file` command.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WriteFileResponse {
    /// New file size.
    pub size_bytes: u64,
    /// New modification timestamp.
    pub modified_at_ms: i64,
    /// Updated content hash.
    pub etag: String,
}

/// Request payload for the `create_note` command.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateNoteRequest {
    /// Workspace-relative path for the new note.
    pub path: String,
}

/// Response payload for the `create_note` command.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateNoteResponse {
    /// Actual workspace-relative path (may have `.md` appended).
    pub path: String,
    /// File size (0 for empty).
    pub size_bytes: u64,
    /// Creation timestamp.
    pub modified_at_ms: i64,
    /// Content hash.
    pub etag: String,
}

/// Response payload for the `rename_file` command.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenameFileResponse {
    /// New workspace-relative path.
    pub path: String,
    /// Timestamp after rename.
    pub modified_at_ms: i64,
}

/// Response payload for the `delete_file` command.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteFileResponse {
    /// Deleted workspace-relative path.
    pub path: String,
}

/// Extracted metadata from a markdown note during indexing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteMetadata {
    /// Workspace-relative path.
    pub path: String,
    /// First `# heading`, frontmatter title, or filename without extension.
    pub title: String,
    /// Tags extracted from frontmatter or inline `#tag` syntax.
    pub tags: Vec<String>,
    /// Workspace-relative paths this note links to.
    pub links_out: Vec<String>,
    /// Number of backlinks (computed from link graph).
    pub links_in_count: u64,
    /// Approximate word count of body text.
    pub word_count: u64,
    /// Whether YAML frontmatter block is present.
    pub has_frontmatter: bool,
    /// Last modification timestamp.
    pub modified_at_ms: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entry_kind_serializes_lowercase() {
        assert_eq!(serde_json::to_string(&EntryKind::File).unwrap(), "\"file\"");
        assert_eq!(serde_json::to_string(&EntryKind::Dir).unwrap(), "\"dir\"");
    }

    #[test]
    fn file_entry_absolute_path_is_skipped() {
        let entry = FileEntry {
            path: "notes/hello.md".into(),
            absolute_path: PathBuf::from("/home/user/notes/hello.md"),
            name: "hello.md".into(),
            kind: EntryKind::File,
            ext: Some("md".into()),
            size_bytes: 1024,
            modified_at_ms: 1_700_000_000_000,
            is_hidden: false,
            is_binary: false,
        };
        let json = serde_json::to_string(&entry).unwrap();
        assert!(!json.contains("absolute_path"));
        assert!(!json.contains("/home/user"));
    }

    #[test]
    fn dir_entry_roundtrip() {
        let entry = DirEntry {
            name: "drafts".into(),
            path: "notes/drafts".into(),
            kind: EntryKind::Dir,
            ext: None,
            size_bytes: None,
            modified_at_ms: None,
            is_hidden: false,
        };
        let json = serde_json::to_string(&entry).unwrap();
        let deser: DirEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(deser.kind, EntryKind::Dir);
        assert!(deser.size_bytes.is_none());
    }

    #[test]
    fn note_metadata_roundtrip() {
        let meta = NoteMetadata {
            path: "journal/2024-01.md".into(),
            title: "January 2024".into(),
            tags: vec!["journal".into(), "monthly".into()],
            links_out: vec!["projects/vigil.md".into()],
            links_in_count: 3,
            word_count: 500,
            has_frontmatter: true,
            modified_at_ms: 1_700_000_000_000,
        };
        let json = serde_json::to_string(&meta).unwrap();
        let deser: NoteMetadata = serde_json::from_str(&json).unwrap();
        assert_eq!(deser.tags.len(), 2);
        assert_eq!(deser.links_out.len(), 1);
    }

    #[test]
    fn write_file_request_optional_etag() {
        let req_json = r#"{"path":"a.md","content":"hello","etag":null}"#;
        let req: WriteFileRequest = serde_json::from_str(req_json).unwrap();
        assert!(req.etag.is_none());

        let req_json = r#"{"path":"a.md","content":"hello","etag":"abc123"}"#;
        let req: WriteFileRequest = serde_json::from_str(req_json).unwrap();
        assert_eq!(req.etag.as_deref(), Some("abc123"));
    }
}
