//! Workspace filesystem service.
//!
//! All operations are confined to the workspace root directory. Path traversal
//! outside the root is rejected with `VigilError::PathOutsideWorkspace`.

use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::models::error::VigilError;
use crate::models::files::{
    CreateNoteRequest, CreateNoteResponse, DeleteFileResponse, DirEntry, EntryKind,
    ListDirResponse, ReadFileResponse, RenameFileResponse, WriteFileRequest, WriteFileResponse,
};
use crate::models::workspace::OpenWorkspaceResponse;

/// Maximum file size for `read_file` (10 MB).
const MAX_READ_SIZE: u64 = 10 * 1024 * 1024;

/// Maximum entries returned by `list_dir`.
const MAX_LIST_ENTRIES: usize = 50_000;

/// Size of the binary detection prefix scan.
const BINARY_SCAN_SIZE: usize = 8192;

/// Core workspace filesystem service.
///
/// Holds the canonical root path and provides safe filesystem operations
/// that reject path traversal attempts.
#[derive(Debug, Clone)]
pub struct WorkspaceFs {
    /// Canonicalized absolute workspace root path.
    root: PathBuf,
    /// Stable hex-encoded hash of `root`.
    workspace_id: String,
    /// Unix epoch ms when workspace was opened.
    #[allow(dead_code)]
    opened_at_ms: i64,
}

impl WorkspaceFs {
    // -----------------------------------------------------------------------
    // Construction
    // -----------------------------------------------------------------------

    /// Open a workspace rooted at `root_path`.
    ///
    /// The path is canonicalized and must point to an existing directory.
    pub fn open(root_path: &str) -> Result<(Self, OpenWorkspaceResponse), VigilError> {
        let root_path = root_path.trim();
        if root_path.is_empty() {
            return Err(VigilError::InvalidArgument {
                reason: "root_path must not be empty".into(),
            });
        }

        let raw = PathBuf::from(root_path);
        let canonical = fs::canonicalize(&raw).map_err(|e| match e.kind() {
            std::io::ErrorKind::NotFound => VigilError::InvalidArgument {
                reason: format!("path does not exist: {root_path}"),
            },
            std::io::ErrorKind::PermissionDenied => VigilError::PermissionDenied {
                path: root_path.into(),
            },
            _ => VigilError::InternalError {
                reason: e.to_string(),
            },
        })?;

        if !canonical.is_dir() {
            return Err(VigilError::InvalidArgument {
                reason: format!("path is not a directory: {root_path}"),
            });
        }

        let workspace_id = hex_hash(canonical.to_string_lossy().as_ref());
        let opened_at_ms = now_epoch_ms();

        // Count files and notes with a quick walk.
        let (files_count, notes_count) = count_workspace_entries(&canonical);

        let ws = Self {
            root: canonical.clone(),
            workspace_id: workspace_id.clone(),
            opened_at_ms,
        };

        let resp = OpenWorkspaceResponse {
            workspace_id,
            canonical_path: canonical.to_string_lossy().into_owned(),
            notes_count,
            files_count,
            opened_at_ms,
        };

        Ok((ws, resp))
    }

    /// Return the workspace root path.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Return the workspace ID.
    pub fn workspace_id(&self) -> &str {
        &self.workspace_id
    }

    // -----------------------------------------------------------------------
    // Path confinement
    // -----------------------------------------------------------------------

    /// Resolve a workspace-relative `path` to an absolute path that is
    /// guaranteed to be within the workspace root.
    ///
    /// - Empty path resolves to the workspace root itself.
    /// - The resolved path is canonicalized when the target exists.
    /// - For create operations where the target does not yet exist, the parent
    ///   is canonicalized and the filename is appended.
    fn confined_path(&self, rel_path: &str) -> Result<PathBuf, VigilError> {
        let rel_path = rel_path.trim();

        // Fast-reject obvious traversal patterns before touching the filesystem.
        if rel_path.contains("..") {
            return Err(VigilError::PathOutsideWorkspace {
                path: rel_path.into(),
            });
        }

        if rel_path.is_empty() {
            return Ok(self.root.clone());
        }

        let joined = self.root.join(rel_path);

        // If the target exists, canonicalize and verify.
        if joined.exists() {
            let canonical = fs::canonicalize(&joined).map_err(|e| VigilError::InternalError {
                reason: e.to_string(),
            })?;
            if !canonical.starts_with(&self.root) {
                return Err(VigilError::PathOutsideWorkspace {
                    path: rel_path.into(),
                });
            }
            return Ok(canonical);
        }

        // Target does not exist yet (create/write scenario). Canonicalize the
        // parent and append the filename to check confinement.
        self.confined_path_for_new_entry(rel_path)
    }

    /// Resolve a workspace-relative path for a target that does not exist yet.
    fn confined_path_for_new_entry(&self, rel_path: &str) -> Result<PathBuf, VigilError> {
        let joined = self.root.join(rel_path);

        // Walk up to the nearest existing ancestor and canonicalize it.
        let mut ancestor = joined.as_path();
        let mut suffix_parts: Vec<&std::ffi::OsStr> = Vec::new();

        while let Some(parent) = ancestor.parent() {
            if parent.exists() {
                let canonical_parent =
                    fs::canonicalize(parent).map_err(|e| VigilError::InternalError {
                        reason: e.to_string(),
                    })?;
                if !canonical_parent.starts_with(&self.root) {
                    return Err(VigilError::PathOutsideWorkspace {
                        path: rel_path.into(),
                    });
                }
                // Reconstruct the full target from the canonical ancestor.
                let mut result = canonical_parent;
                // Push ancestor's own filename if it wasn't the parent.
                if let Some(name) = ancestor.file_name() {
                    result.push(name);
                }
                for part in suffix_parts.into_iter().rev() {
                    result.push(part);
                }
                return Ok(result);
            }
            // Push the current component and keep walking up.
            if let Some(name) = ancestor.file_name() {
                suffix_parts.push(name);
            }
            ancestor = parent;
        }

        Err(VigilError::PathOutsideWorkspace {
            path: rel_path.into(),
        })
    }

    // -----------------------------------------------------------------------
    // list_dir
    // -----------------------------------------------------------------------

    /// List entries in a workspace-relative directory.
    ///
    /// Entries are sorted: directories first, then alphabetical
    /// (case-insensitive). Hidden entries (`.git`, `node_modules`, etc.) are
    /// included but flagged.
    pub fn list_dir(&self, rel_path: &str) -> Result<ListDirResponse, VigilError> {
        let dir = self.confined_path(rel_path)?;

        if !dir.is_dir() {
            return Err(VigilError::FileNotFound {
                path: rel_path.into(),
            });
        }

        let mut entries: Vec<DirEntry> = Vec::new();
        let mut truncated = false;

        let read_dir = fs::read_dir(&dir).map_err(VigilError::from)?;

        for entry_result in read_dir {
            if entries.len() >= MAX_LIST_ENTRIES {
                truncated = true;
                break;
            }
            let entry = match entry_result {
                Ok(e) => e,
                Err(_) => continue, // skip unreadable entries
            };

            let metadata = match entry.metadata() {
                Ok(m) => m,
                Err(_) => continue,
            };

            let name = entry.file_name().to_string_lossy().into_owned();

            // Skip .git directory at any level.
            if name == ".git" {
                continue;
            }

            let kind = if metadata.is_dir() {
                EntryKind::Dir
            } else {
                EntryKind::File
            };

            let entry_path = entry.path();
            let workspace_rel = entry_path
                .strip_prefix(&self.root)
                .unwrap_or(&entry_path)
                .to_string_lossy()
                .replace('\\', "/");

            let ext = if kind == EntryKind::File {
                entry_path
                    .extension()
                    .map(|e| e.to_string_lossy().into_owned())
            } else {
                None
            };

            let (size_bytes, modified_at_ms) = if kind == EntryKind::File {
                (Some(metadata.len()), Some(modified_epoch_ms(&metadata)))
            } else {
                (None, None)
            };

            let is_hidden = name.starts_with('.');

            entries.push(DirEntry {
                name,
                path: workspace_rel,
                kind,
                ext,
                size_bytes,
                modified_at_ms,
                is_hidden,
            });
        }

        // Sort: directories first, then case-insensitive alphabetical.
        entries.sort_by(|a, b| {
            let dir_ord = match (&a.kind, &b.kind) {
                (EntryKind::Dir, EntryKind::File) => std::cmp::Ordering::Less,
                (EntryKind::File, EntryKind::Dir) => std::cmp::Ordering::Greater,
                _ => std::cmp::Ordering::Equal,
            };
            dir_ord.then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
        });

        Ok(ListDirResponse { entries, truncated })
    }

    // -----------------------------------------------------------------------
    // read_file
    // -----------------------------------------------------------------------

    /// Read a text file's content and metadata.
    pub fn read_file(&self, rel_path: &str) -> Result<ReadFileResponse, VigilError> {
        let rel_path = rel_path.trim();
        if rel_path.is_empty() {
            return Err(VigilError::InvalidArgument {
                reason: "path must not be empty".into(),
            });
        }

        let abs = self.confined_path(rel_path)?;

        if !abs.is_file() {
            return Err(VigilError::FileNotFound {
                path: rel_path.into(),
            });
        }

        let metadata = fs::metadata(&abs).map_err(VigilError::from)?;
        let size = metadata.len();

        if size > MAX_READ_SIZE {
            return Err(VigilError::InvalidArgument {
                reason: format!(
                    "file is too large ({size} bytes, max {MAX_READ_SIZE}): {rel_path}"
                ),
            });
        }

        let bytes = fs::read(&abs).map_err(VigilError::from)?;

        // Binary detection: check for null bytes in first 8 KB.
        let scan_len = bytes.len().min(BINARY_SCAN_SIZE);
        if bytes[..scan_len].contains(&0) {
            return Err(VigilError::BinaryFile {
                path: rel_path.into(),
            });
        }

        let content = String::from_utf8(bytes).map_err(|_| VigilError::BinaryFile {
            path: rel_path.into(),
        })?;

        let etag = content_etag(content.as_bytes());
        let modified_at_ms = modified_epoch_ms(&metadata);

        Ok(ReadFileResponse {
            content,
            encoding: "utf-8".into(),
            size_bytes: size,
            modified_at_ms,
            etag,
        })
    }

    // -----------------------------------------------------------------------
    // write_file
    // -----------------------------------------------------------------------

    /// Write content to a file with optional optimistic concurrency check.
    pub fn write_file(&self, request: &WriteFileRequest) -> Result<WriteFileResponse, VigilError> {
        let rel_path = request.path.trim();
        if rel_path.is_empty() {
            return Err(VigilError::InvalidArgument {
                reason: "path must not be empty".into(),
            });
        }

        let abs = self.confined_path(rel_path)?;

        // Optimistic concurrency: check etag if provided.
        if let Some(ref expected_etag) = request.etag {
            if abs.is_file() {
                let current_bytes = fs::read(&abs).map_err(VigilError::from)?;
                let current_etag = content_etag(&current_bytes);
                if *expected_etag != current_etag {
                    return Err(VigilError::StaleEtag);
                }
            }
        }

        // Write the file. Parent directories are NOT auto-created for write_file
        // per IPC contract.
        fs::write(&abs, &request.content).map_err(VigilError::from)?;

        let metadata = fs::metadata(&abs).map_err(VigilError::from)?;
        let etag = content_etag(request.content.as_bytes());
        let modified_at_ms = modified_epoch_ms(&metadata);

        Ok(WriteFileResponse {
            size_bytes: metadata.len(),
            modified_at_ms,
            etag,
        })
    }

    // -----------------------------------------------------------------------
    // create_note
    // -----------------------------------------------------------------------

    /// Create a new markdown note file.
    ///
    /// - If the path has no extension, `.md` is appended.
    /// - Parent directories are created as needed.
    pub fn create_note(
        &self,
        request: &CreateNoteRequest,
    ) -> Result<CreateNoteResponse, VigilError> {
        let mut rel_path = request.path.trim().to_string();
        if rel_path.is_empty() {
            return Err(VigilError::InvalidArgument {
                reason: "path must not be empty".into(),
            });
        }

        // Append .md if no extension.
        let p = Path::new(&rel_path);
        if p.extension().is_none() {
            rel_path.push_str(".md");
        }

        let abs = self.confined_path(&rel_path)?;

        if abs.exists() {
            return Err(VigilError::FileAlreadyExists {
                path: rel_path.clone(),
            });
        }

        // Create parent directories as needed.
        if let Some(parent) = abs.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent).map_err(VigilError::from)?;
            }
        }

        // Create empty file.
        fs::write(&abs, "").map_err(VigilError::from)?;

        let metadata = fs::metadata(&abs).map_err(VigilError::from)?;
        let etag = content_etag(b"");
        let modified_at_ms = modified_epoch_ms(&metadata);

        Ok(CreateNoteResponse {
            path: rel_path,
            size_bytes: 0,
            modified_at_ms,
            etag,
        })
    }

    // -----------------------------------------------------------------------
    // rename_file
    // -----------------------------------------------------------------------

    /// Rename or move a file within the workspace.
    pub fn rename_file(
        &self,
        old_path: &str,
        new_path: &str,
    ) -> Result<RenameFileResponse, VigilError> {
        let old_path = old_path.trim();
        let new_path = new_path.trim();

        if old_path.is_empty() || new_path.is_empty() {
            return Err(VigilError::InvalidArgument {
                reason: "old_path and new_path must not be empty".into(),
            });
        }

        let abs_old = self.confined_path(old_path)?;
        let abs_new = self.confined_path(new_path)?;

        if !abs_old.exists() {
            return Err(VigilError::FileNotFound {
                path: old_path.into(),
            });
        }

        if abs_new.exists() {
            return Err(VigilError::FileAlreadyExists {
                path: new_path.into(),
            });
        }

        // Ensure parent of new path exists.
        if let Some(parent) = abs_new.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent).map_err(VigilError::from)?;
            }
        }

        fs::rename(&abs_old, &abs_new).map_err(VigilError::from)?;

        let metadata = fs::metadata(&abs_new).map_err(VigilError::from)?;
        let modified_at_ms = modified_epoch_ms(&metadata);

        Ok(RenameFileResponse {
            path: new_path.into(),
            modified_at_ms,
        })
    }

    // -----------------------------------------------------------------------
    // delete_file
    // -----------------------------------------------------------------------

    /// Delete a file or empty directory within the workspace.
    pub fn delete_file(&self, rel_path: &str) -> Result<DeleteFileResponse, VigilError> {
        let rel_path = rel_path.trim();
        if rel_path.is_empty() {
            return Err(VigilError::InvalidArgument {
                reason: "path must not be empty".into(),
            });
        }

        let abs = self.confined_path(rel_path)?;

        if !abs.exists() {
            return Err(VigilError::FileNotFound {
                path: rel_path.into(),
            });
        }

        if abs.is_dir() {
            // Only delete empty directories.
            let has_children = fs::read_dir(&abs)
                .map_err(VigilError::from)?
                .next()
                .is_some();
            if has_children {
                return Err(VigilError::InvalidArgument {
                    reason: format!("directory is not empty: {rel_path}"),
                });
            }
            fs::remove_dir(&abs).map_err(VigilError::from)?;
        } else {
            fs::remove_file(&abs).map_err(VigilError::from)?;
        }

        Ok(DeleteFileResponse {
            path: rel_path.into(),
        })
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Compute a deterministic hex-encoded hash for a string.
fn hex_hash(s: &str) -> String {
    let mut hasher = DefaultHasher::new();
    s.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

/// Compute an etag from file content bytes using a fast hash.
fn content_etag(bytes: &[u8]) -> String {
    let mut hasher = DefaultHasher::new();
    bytes.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

/// Get the modified time of a file as unix epoch milliseconds.
fn modified_epoch_ms(metadata: &fs::Metadata) -> i64 {
    metadata
        .modified()
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

/// Current time as unix epoch milliseconds.
fn now_epoch_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

/// Walk the workspace root and count total files and `.md` notes.
fn count_workspace_entries(root: &Path) -> (u64, u64) {
    let mut files: u64 = 0;
    let mut notes: u64 = 0;

    for entry in walkdir::WalkDir::new(root)
        .into_iter()
        .filter_entry(|e| {
            // Skip .git directories.
            e.file_name() != ".git"
        })
        .flatten()
    {
        if entry.file_type().is_file() {
            files += 1;
            if let Some(ext) = entry.path().extension() {
                if ext.eq_ignore_ascii_case("md") {
                    notes += 1;
                }
            }
        }
    }

    (files, notes)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    /// Create a temporary workspace directory and return its path.
    fn temp_workspace() -> tempfile::TempDir {
        tempfile::tempdir().expect("failed to create temp dir")
    }

    #[test]
    fn open_workspace_succeeds_for_valid_dir() {
        let dir = temp_workspace();
        let (ws, resp) = WorkspaceFs::open(dir.path().to_str().unwrap()).unwrap();
        assert!(!resp.workspace_id.is_empty());
        assert_eq!(ws.root(), fs::canonicalize(dir.path()).unwrap());
        assert_eq!(resp.files_count, 0);
        assert_eq!(resp.notes_count, 0);
    }

    #[test]
    fn open_workspace_rejects_nonexistent_path() {
        let result = WorkspaceFs::open("/nonexistent/path/that/does/not/exist");
        assert!(result.is_err());
    }

    #[test]
    fn open_workspace_rejects_file_path() {
        let dir = temp_workspace();
        let file = dir.path().join("a.txt");
        fs::write(&file, "hello").unwrap();
        let result = WorkspaceFs::open(file.to_str().unwrap());
        assert!(result.is_err());
    }

    #[test]
    fn open_workspace_counts_notes() {
        let dir = temp_workspace();
        fs::write(dir.path().join("a.md"), "note a").unwrap();
        fs::write(dir.path().join("b.md"), "note b").unwrap();
        fs::write(dir.path().join("c.txt"), "text").unwrap();
        let (_ws, resp) = WorkspaceFs::open(dir.path().to_str().unwrap()).unwrap();
        assert_eq!(resp.files_count, 3);
        assert_eq!(resp.notes_count, 2);
    }

    #[test]
    fn path_traversal_is_rejected() {
        let dir = temp_workspace();
        let (ws, _) = WorkspaceFs::open(dir.path().to_str().unwrap()).unwrap();
        let result = ws.read_file("../../etc/passwd");
        assert!(result.is_err());
        match result.unwrap_err() {
            VigilError::PathOutsideWorkspace { .. } => {}
            other => panic!("expected PathOutsideWorkspace, got: {other:?}"),
        }
    }

    #[test]
    fn list_dir_root() {
        let dir = temp_workspace();
        fs::write(dir.path().join("hello.md"), "hi").unwrap();
        fs::create_dir(dir.path().join("subdir")).unwrap();
        let (ws, _) = WorkspaceFs::open(dir.path().to_str().unwrap()).unwrap();
        let resp = ws.list_dir("").unwrap();
        assert_eq!(resp.entries.len(), 2);
        // Dirs come first.
        assert_eq!(resp.entries[0].kind, EntryKind::Dir);
        assert_eq!(resp.entries[1].kind, EntryKind::File);
        assert!(!resp.truncated);
    }

    #[test]
    fn list_dir_skips_dot_git() {
        let dir = temp_workspace();
        fs::create_dir(dir.path().join(".git")).unwrap();
        fs::write(dir.path().join("a.md"), "").unwrap();
        let (ws, _) = WorkspaceFs::open(dir.path().to_str().unwrap()).unwrap();
        let resp = ws.list_dir("").unwrap();
        assert_eq!(resp.entries.len(), 1);
        assert_eq!(resp.entries[0].name, "a.md");
    }

    #[test]
    fn read_file_returns_content_and_etag() {
        let dir = temp_workspace();
        fs::write(dir.path().join("test.md"), "hello world").unwrap();
        let (ws, _) = WorkspaceFs::open(dir.path().to_str().unwrap()).unwrap();
        let resp = ws.read_file("test.md").unwrap();
        assert_eq!(resp.content, "hello world");
        assert_eq!(resp.encoding, "utf-8");
        assert!(!resp.etag.is_empty());
        assert_eq!(resp.size_bytes, 11);
    }

    #[test]
    fn read_file_rejects_binary() {
        let dir = temp_workspace();
        let mut content = vec![0u8; 100];
        content[50] = 0; // explicit null byte
        fs::write(dir.path().join("binary.bin"), &content).unwrap();
        let (ws, _) = WorkspaceFs::open(dir.path().to_str().unwrap()).unwrap();
        let result = ws.read_file("binary.bin");
        assert!(result.is_err());
        match result.unwrap_err() {
            VigilError::BinaryFile { .. } => {}
            other => panic!("expected BinaryFile, got: {other:?}"),
        }
    }

    #[test]
    fn write_file_creates_and_updates() {
        let dir = temp_workspace();
        fs::write(dir.path().join("existing.md"), "old").unwrap();
        let (ws, _) = WorkspaceFs::open(dir.path().to_str().unwrap()).unwrap();

        // Write to existing file.
        let req = WriteFileRequest {
            path: "existing.md".into(),
            content: "new content".into(),
            etag: None,
        };
        let resp = ws.write_file(&req).unwrap();
        assert_eq!(resp.size_bytes, 11);
        assert!(!resp.etag.is_empty());

        // Verify on disk.
        let on_disk = fs::read_to_string(dir.path().join("existing.md")).unwrap();
        assert_eq!(on_disk, "new content");
    }

    #[test]
    fn write_file_stale_etag_rejected() {
        let dir = temp_workspace();
        fs::write(dir.path().join("test.md"), "original").unwrap();
        let (ws, _) = WorkspaceFs::open(dir.path().to_str().unwrap()).unwrap();

        let req = WriteFileRequest {
            path: "test.md".into(),
            content: "new".into(),
            etag: Some("wrong_etag".into()),
        };
        let result = ws.write_file(&req);
        assert!(result.is_err());
        match result.unwrap_err() {
            VigilError::StaleEtag => {}
            other => panic!("expected StaleEtag, got: {other:?}"),
        }
    }

    #[test]
    fn write_file_with_correct_etag_succeeds() {
        let dir = temp_workspace();
        fs::write(dir.path().join("test.md"), "original").unwrap();
        let (ws, _) = WorkspaceFs::open(dir.path().to_str().unwrap()).unwrap();

        // Get the current etag.
        let read_resp = ws.read_file("test.md").unwrap();

        let req = WriteFileRequest {
            path: "test.md".into(),
            content: "updated".into(),
            etag: Some(read_resp.etag),
        };
        let result = ws.write_file(&req);
        assert!(result.is_ok());
    }

    #[test]
    fn create_note_appends_md_extension() {
        let dir = temp_workspace();
        let (ws, _) = WorkspaceFs::open(dir.path().to_str().unwrap()).unwrap();

        let req = CreateNoteRequest {
            path: "my-note".into(),
        };
        let resp = ws.create_note(&req).unwrap();
        assert_eq!(resp.path, "my-note.md");
        assert!(dir.path().join("my-note.md").exists());
    }

    #[test]
    fn create_note_keeps_existing_extension() {
        let dir = temp_workspace();
        let (ws, _) = WorkspaceFs::open(dir.path().to_str().unwrap()).unwrap();

        let req = CreateNoteRequest {
            path: "my-note.txt".into(),
        };
        let resp = ws.create_note(&req).unwrap();
        assert_eq!(resp.path, "my-note.txt");
    }

    #[test]
    fn create_note_creates_parent_dirs() {
        let dir = temp_workspace();
        let (ws, _) = WorkspaceFs::open(dir.path().to_str().unwrap()).unwrap();

        let req = CreateNoteRequest {
            path: "sub/deep/note".into(),
        };
        let resp = ws.create_note(&req).unwrap();
        assert_eq!(resp.path, "sub/deep/note.md");
        assert!(dir.path().join("sub/deep/note.md").exists());
    }

    #[test]
    fn create_note_rejects_existing() {
        let dir = temp_workspace();
        fs::write(dir.path().join("existing.md"), "hi").unwrap();
        let (ws, _) = WorkspaceFs::open(dir.path().to_str().unwrap()).unwrap();

        let req = CreateNoteRequest {
            path: "existing.md".into(),
        };
        let result = ws.create_note(&req);
        assert!(result.is_err());
        match result.unwrap_err() {
            VigilError::FileAlreadyExists { .. } => {}
            other => panic!("expected FileAlreadyExists, got: {other:?}"),
        }
    }

    #[test]
    fn rename_file_succeeds() {
        let dir = temp_workspace();
        fs::write(dir.path().join("old.md"), "content").unwrap();
        let (ws, _) = WorkspaceFs::open(dir.path().to_str().unwrap()).unwrap();

        let resp = ws.rename_file("old.md", "new.md").unwrap();
        assert_eq!(resp.path, "new.md");
        assert!(!dir.path().join("old.md").exists());
        assert!(dir.path().join("new.md").exists());
    }

    #[test]
    fn rename_file_rejects_existing_target() {
        let dir = temp_workspace();
        fs::write(dir.path().join("a.md"), "a").unwrap();
        fs::write(dir.path().join("b.md"), "b").unwrap();
        let (ws, _) = WorkspaceFs::open(dir.path().to_str().unwrap()).unwrap();

        let result = ws.rename_file("a.md", "b.md");
        assert!(result.is_err());
        match result.unwrap_err() {
            VigilError::FileAlreadyExists { .. } => {}
            other => panic!("expected FileAlreadyExists, got: {other:?}"),
        }
    }

    #[test]
    fn delete_file_removes_file() {
        let dir = temp_workspace();
        fs::write(dir.path().join("doomed.md"), "bye").unwrap();
        let (ws, _) = WorkspaceFs::open(dir.path().to_str().unwrap()).unwrap();

        let resp = ws.delete_file("doomed.md").unwrap();
        assert_eq!(resp.path, "doomed.md");
        assert!(!dir.path().join("doomed.md").exists());
    }

    #[test]
    fn delete_rejects_nonempty_dir() {
        let dir = temp_workspace();
        fs::create_dir(dir.path().join("subdir")).unwrap();
        fs::write(dir.path().join("subdir/file.md"), "").unwrap();
        let (ws, _) = WorkspaceFs::open(dir.path().to_str().unwrap()).unwrap();

        let result = ws.delete_file("subdir");
        assert!(result.is_err());
        match result.unwrap_err() {
            VigilError::InvalidArgument { .. } => {}
            other => panic!("expected InvalidArgument, got: {other:?}"),
        }
    }

    #[test]
    fn delete_empty_dir_succeeds() {
        let dir = temp_workspace();
        fs::create_dir(dir.path().join("emptydir")).unwrap();
        let (ws, _) = WorkspaceFs::open(dir.path().to_str().unwrap()).unwrap();

        let resp = ws.delete_file("emptydir").unwrap();
        assert_eq!(resp.path, "emptydir");
        assert!(!dir.path().join("emptydir").exists());
    }

    #[test]
    fn content_etag_is_deterministic() {
        let e1 = content_etag(b"hello world");
        let e2 = content_etag(b"hello world");
        assert_eq!(e1, e2);
        let e3 = content_etag(b"different");
        assert_ne!(e1, e3);
    }
}
