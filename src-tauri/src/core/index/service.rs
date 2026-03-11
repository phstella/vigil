//! In-memory file index with incremental update support.
//!
//! `FileIndex` maintains a `HashMap<PathBuf, NoteMetadata>` for markdown notes
//! and a `HashMap<PathBuf, FileEntry>` for all workspace files. It supports:
//!
//! - `full_scan(root)`: Walk the directory tree using the `ignore` crate
//!   (respects `.gitignore`), build the initial index.
//! - `handle_event(event)`: Incremental create/modify/delete updates.
//! - Query methods: `get_all_files()`, `get_file_metadata(path)`,
//!   `get_note_count()`, `get_tag_count()`.

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::UNIX_EPOCH;

use ignore::WalkBuilder;
use parking_lot::RwLock;

use crate::models::files::{EntryKind, FileEntry, NoteMetadata};

/// Size of the binary detection prefix scan.
const BINARY_SCAN_SIZE: usize = 8192;

/// Change type for incremental index updates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChangeKind {
    Created,
    Changed,
    Deleted,
}

/// A single index change record, produced by incremental updates.
#[derive(Debug, Clone)]
pub struct IndexChangeRecord {
    /// Workspace-relative path (forward slashes).
    pub path: String,
    /// What happened.
    pub change_kind: ChangeKind,
    /// File or directory.
    pub entry_kind: EntryKind,
}

/// Result of a full scan operation.
#[derive(Debug, Clone)]
pub struct ScanResult {
    pub files_count: u64,
    pub notes_count: u64,
    pub duration_ms: u64,
}

/// Thread-safe in-memory file index.
///
/// All public methods acquire the internal `RwLock` as needed; callers do not
/// need external synchronization.
#[derive(Debug, Clone)]
pub struct FileIndex {
    inner: Arc<RwLock<IndexInner>>,
    root: PathBuf,
}

#[derive(Debug, Default)]
struct IndexInner {
    /// All indexed files keyed by workspace-relative path.
    files: HashMap<String, FileEntry>,
    /// Note metadata for markdown files, keyed by workspace-relative path.
    notes: HashMap<String, NoteMetadata>,
}

impl FileIndex {
    /// Create a new empty index for the given workspace root.
    pub fn new(root: PathBuf) -> Self {
        Self {
            inner: Arc::new(RwLock::new(IndexInner::default())),
            root,
        }
    }

    /// Return the workspace root this index is bound to.
    pub fn root(&self) -> &Path {
        &self.root
    }

    // -----------------------------------------------------------------------
    // Full scan
    // -----------------------------------------------------------------------

    /// Walk the workspace directory tree and build the complete index.
    ///
    /// Uses the `ignore` crate which respects `.gitignore` and `.ignore` files
    /// by default. Returns counts for the `index-ready` event.
    pub fn full_scan(&self) -> ScanResult {
        let start = std::time::Instant::now();
        let root = self.root.clone();

        let mut files: HashMap<String, FileEntry> = HashMap::new();
        let mut notes: HashMap<String, NoteMetadata> = HashMap::new();

        let walker = WalkBuilder::new(&root)
            .hidden(false) // include hidden files (we flag them)
            .git_ignore(true)
            .git_global(true)
            .git_exclude(true)
            .filter_entry(|entry| {
                // Always skip .git directory itself
                entry.file_name() != ".git"
            })
            .build();

        for result in walker {
            let entry = match result {
                Ok(e) => e,
                Err(_) => continue,
            };

            // Skip the root directory itself.
            if entry.path() == root {
                continue;
            }

            let abs_path = entry.path().to_path_buf();
            let rel_path = match abs_path.strip_prefix(&root) {
                Ok(p) => p.to_string_lossy().replace('\\', "/"),
                Err(_) => continue,
            };

            let metadata = match entry.metadata() {
                Ok(m) => m,
                Err(_) => continue,
            };

            let name = entry
                .file_name()
                .to_string_lossy()
                .into_owned();

            let kind = if metadata.is_dir() {
                EntryKind::Dir
            } else {
                EntryKind::File
            };

            let ext = if kind == EntryKind::File {
                abs_path
                    .extension()
                    .map(|e| e.to_string_lossy().into_owned())
            } else {
                None
            };

            let is_hidden = name.starts_with('.');
            let size_bytes = if kind == EntryKind::File {
                metadata.len()
            } else {
                0
            };
            let modified_at_ms = modified_epoch_ms(&metadata);

            let is_binary = if kind == EntryKind::File {
                detect_binary(&abs_path)
            } else {
                false
            };

            let file_entry = FileEntry {
                path: rel_path.clone(),
                absolute_path: abs_path.clone(),
                name: name.clone(),
                kind,
                ext: ext.clone(),
                size_bytes,
                modified_at_ms,
                is_hidden,
                is_binary,
            };

            files.insert(rel_path.clone(), file_entry);

            // Extract note metadata for markdown files.
            if kind == EntryKind::File && !is_binary {
                if let Some(ref e) = ext {
                    if e.eq_ignore_ascii_case("md") {
                        if let Some(meta) = extract_note_metadata(&abs_path, &rel_path) {
                            notes.insert(rel_path, meta);
                        }
                    }
                }
            }
        }

        let files_count = files.values().filter(|f| f.kind == EntryKind::File).count() as u64;
        let notes_count = notes.len() as u64;
        let duration_ms = start.elapsed().as_millis() as u64;

        // Replace the index contents atomically.
        {
            let mut inner = self.inner.write();
            inner.files = files;
            inner.notes = notes;
        }

        ScanResult {
            files_count,
            notes_count,
            duration_ms,
        }
    }

    // -----------------------------------------------------------------------
    // Incremental updates
    // -----------------------------------------------------------------------

    /// Process a filesystem event and update the index accordingly.
    ///
    /// Returns a list of change records suitable for emitting as events.
    /// Paths are absolute; they are resolved relative to the workspace root.
    pub fn handle_event(&self, paths: &[PathBuf], change_kind: ChangeKind) -> Vec<IndexChangeRecord> {
        let mut changes = Vec::new();

        for abs_path in paths {
            let rel_path = match abs_path.strip_prefix(&self.root) {
                Ok(p) => p.to_string_lossy().replace('\\', "/"),
                Err(_) => continue,
            };

            // Skip .git directory contents
            if rel_path.starts_with(".git/") || rel_path == ".git" {
                continue;
            }

            match change_kind {
                ChangeKind::Deleted => {
                    let mut inner = self.inner.write();
                    let entry_kind = inner
                        .files
                        .get(&rel_path)
                        .map(|f| f.kind)
                        .unwrap_or(EntryKind::File);

                    inner.files.remove(&rel_path);
                    inner.notes.remove(&rel_path);

                    changes.push(IndexChangeRecord {
                        path: rel_path,
                        change_kind: ChangeKind::Deleted,
                        entry_kind,
                    });
                }
                ChangeKind::Created | ChangeKind::Changed => {
                    // Re-read metadata from disk.
                    let metadata = match fs::metadata(abs_path) {
                        Ok(m) => m,
                        Err(_) => continue,
                    };

                    let name = abs_path
                        .file_name()
                        .map(|n| n.to_string_lossy().into_owned())
                        .unwrap_or_default();

                    let kind = if metadata.is_dir() {
                        EntryKind::Dir
                    } else {
                        EntryKind::File
                    };

                    let ext = if kind == EntryKind::File {
                        abs_path
                            .extension()
                            .map(|e| e.to_string_lossy().into_owned())
                    } else {
                        None
                    };

                    let is_hidden = name.starts_with('.');
                    let size_bytes = if kind == EntryKind::File {
                        metadata.len()
                    } else {
                        0
                    };
                    let modified_at_ms = modified_epoch_ms(&metadata);

                    let is_binary = if kind == EntryKind::File {
                        detect_binary(abs_path)
                    } else {
                        false
                    };

                    let file_entry = FileEntry {
                        path: rel_path.clone(),
                        absolute_path: abs_path.clone(),
                        name,
                        kind,
                        ext: ext.clone(),
                        size_bytes,
                        modified_at_ms,
                        is_hidden,
                        is_binary,
                    };

                    let mut inner = self.inner.write();
                    inner.files.insert(rel_path.clone(), file_entry);

                    // Update or remove note metadata.
                    if kind == EntryKind::File && !is_binary {
                        if let Some(ref e) = ext {
                            if e.eq_ignore_ascii_case("md") {
                                if let Some(meta) =
                                    extract_note_metadata(abs_path, &rel_path)
                                {
                                    inner.notes.insert(rel_path.clone(), meta);
                                }
                            }
                        }
                    }

                    changes.push(IndexChangeRecord {
                        path: rel_path,
                        change_kind,
                        entry_kind: kind,
                    });
                }
            }
        }

        changes
    }

    // -----------------------------------------------------------------------
    // Queries
    // -----------------------------------------------------------------------

    /// Get a snapshot of all indexed files.
    pub fn get_all_files(&self) -> Vec<FileEntry> {
        let inner = self.inner.read();
        inner.files.values().cloned().collect()
    }

    /// Get metadata for a specific file by workspace-relative path.
    pub fn get_file_entry(&self, rel_path: &str) -> Option<FileEntry> {
        let inner = self.inner.read();
        inner.files.get(rel_path).cloned()
    }

    /// Get note metadata for a specific markdown file.
    pub fn get_note_metadata(&self, rel_path: &str) -> Option<NoteMetadata> {
        let inner = self.inner.read();
        inner.notes.get(rel_path).cloned()
    }

    /// Get all note metadata entries.
    pub fn get_all_notes(&self) -> Vec<NoteMetadata> {
        let inner = self.inner.read();
        inner.notes.values().cloned().collect()
    }

    /// Total number of indexed files (excluding directories).
    pub fn get_file_count(&self) -> u64 {
        let inner = self.inner.read();
        inner
            .files
            .values()
            .filter(|f| f.kind == EntryKind::File)
            .count() as u64
    }

    /// Number of indexed markdown notes.
    pub fn get_note_count(&self) -> u64 {
        let inner = self.inner.read();
        inner.notes.len() as u64
    }

    /// Number of unique tags across all indexed notes.
    pub fn get_tag_count(&self) -> u64 {
        let inner = self.inner.read();
        let mut tags: HashSet<&str> = HashSet::new();
        for meta in inner.notes.values() {
            for tag in &meta.tags {
                tags.insert(tag.as_str());
            }
        }
        tags.len() as u64
    }

    /// Get all unique tags and their counts.
    pub fn get_all_tags(&self) -> HashMap<String, u64> {
        let inner = self.inner.read();
        let mut tag_counts: HashMap<String, u64> = HashMap::new();
        for meta in inner.notes.values() {
            for tag in &meta.tags {
                *tag_counts.entry(tag.clone()).or_insert(0) += 1;
            }
        }
        tag_counts
    }

    /// Check whether the index has been populated (post-scan).
    pub fn is_populated(&self) -> bool {
        let inner = self.inner.read();
        !inner.files.is_empty()
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Get the modified time of a file as unix epoch milliseconds.
fn modified_epoch_ms(metadata: &fs::Metadata) -> i64 {
    metadata
        .modified()
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

/// Detect if a file is binary by scanning for null bytes in the first 8 KB.
fn detect_binary(path: &Path) -> bool {
    let Ok(file) = fs::File::open(path) else {
        return false;
    };

    use std::io::Read;
    let mut buf = [0u8; BINARY_SCAN_SIZE];
    let mut reader = std::io::BufReader::new(file);
    let n = match reader.read(&mut buf) {
        Ok(n) => n,
        Err(_) => return false,
    };

    buf[..n].contains(&0)
}

/// Extract note metadata from a markdown file.
///
/// Parses frontmatter, heading, tags, links, and word count.
fn extract_note_metadata(abs_path: &Path, rel_path: &str) -> Option<NoteMetadata> {
    let content = fs::read_to_string(abs_path).ok()?;
    let metadata = fs::metadata(abs_path).ok()?;
    let modified_at_ms = modified_epoch_ms(&metadata);

    let (has_frontmatter, frontmatter_title, frontmatter_tags, body) =
        parse_frontmatter(&content);

    // Title: frontmatter title > first heading > filename stem
    let title = frontmatter_title
        .or_else(|| extract_first_heading(&content))
        .unwrap_or_else(|| {
            Path::new(rel_path)
                .file_stem()
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_else(|| rel_path.to_string())
        });

    // Tags: combine frontmatter tags and inline #tag patterns.
    // Strip fenced code blocks before extracting inline tags.
    let body_no_code = strip_fenced_code_blocks(&body);
    let mut tags: Vec<String> = frontmatter_tags;
    let inline_tags = extract_inline_tags(&body_no_code);
    for t in inline_tags {
        let normalized = t.to_lowercase();
        if !tags.contains(&normalized) {
            tags.push(normalized);
        }
    }

    // Links: extract [[wikilinks]] and [text](path.md) links.
    let links_out = extract_links(&body, rel_path);

    // Word count: approximate by splitting on whitespace.
    let word_count = body.split_whitespace().count() as u64;

    Some(NoteMetadata {
        path: rel_path.to_string(),
        title,
        tags,
        links_out,
        links_in_count: 0, // computed later from link graph
        word_count,
        has_frontmatter,
        modified_at_ms,
    })
}

/// Parse YAML frontmatter from markdown content.
///
/// Returns (has_frontmatter, title, tags, body_without_frontmatter).
fn parse_frontmatter(content: &str) -> (bool, Option<String>, Vec<String>, String) {
    if !content.starts_with("---") {
        return (false, None, Vec::new(), content.to_string());
    }

    // Find the closing `---`.
    let rest = &content[3..];
    let end_pos = rest.find("\n---");
    let Some(end) = end_pos else {
        return (false, None, Vec::new(), content.to_string());
    };

    let frontmatter = &rest[..end];
    // Body starts after closing --- and newline
    let body_start = 3 + end + 4; // "---" prefix + content + "\n---"
    let body = if body_start < content.len() {
        content[body_start..].trim_start_matches('\n').to_string()
    } else {
        String::new()
    };

    let mut title: Option<String> = None;
    let mut tags: Vec<String> = Vec::new();

    for line in frontmatter.lines() {
        let line = line.trim();

        // Parse title: field
        if let Some(value) = line.strip_prefix("title:") {
            let value = value.trim().trim_matches('"').trim_matches('\'');
            if !value.is_empty() {
                title = Some(value.to_string());
            }
        }

        // Parse tags: [tag1, tag2] or tags: field followed by - items
        if let Some(value) = line.strip_prefix("tags:") {
            let value = value.trim();
            if value.starts_with('[') && value.ends_with(']') {
                // Inline array format: tags: [tag1, tag2]
                let inner = &value[1..value.len() - 1];
                for tag in inner.split(',') {
                    let tag = tag.trim().trim_matches('"').trim_matches('\'').to_lowercase();
                    if !tag.is_empty() {
                        tags.push(tag);
                    }
                }
            }
        }

        // YAML list item format: - tag
        if line.starts_with("- ") && !tags.is_empty() {
            // Only collect if we already saw `tags:` line (hacky but works for simple cases)
            // Actually, we just need to detect we're in a tags list.
            // For MVP, we'll rely on the inline format.
        }
    }

    // Try YAML list format for tags if we didn't get inline format.
    if tags.is_empty() {
        let mut in_tags = false;
        for line in frontmatter.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("tags:") {
                let value = trimmed.strip_prefix("tags:").unwrap().trim();
                if value.is_empty() {
                    in_tags = true;
                    continue;
                }
            }
            if in_tags {
                if let Some(tag_value) = trimmed.strip_prefix("- ") {
                    let tag = tag_value
                        .trim()
                        .trim_matches('"')
                        .trim_matches('\'')
                        .to_lowercase();
                    if !tag.is_empty() {
                        tags.push(tag);
                    }
                } else if !trimmed.is_empty() {
                    // End of list
                    in_tags = false;
                }
            }
        }
    }

    (true, title, tags, body)
}

/// Extract the first ATX heading (`# Title`) from markdown content.
fn extract_first_heading(content: &str) -> Option<String> {
    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(heading) = trimmed.strip_prefix("# ") {
            let heading = heading.trim();
            if !heading.is_empty() {
                return Some(heading.to_string());
            }
        }
    }
    None
}

/// Strip fenced code blocks (``` or ~~~) from markdown body text.
///
/// Returns the body with all fenced code block content replaced by empty lines
/// so that line numbers are preserved for other processing.
fn strip_fenced_code_blocks(body: &str) -> String {
    let mut result = String::with_capacity(body.len());
    let mut in_code_block = false;
    let mut fence_char = ' ';
    let mut fence_len = 0;

    for line in body.lines() {
        let trimmed = line.trim_start();
        if !in_code_block {
            // Check for opening fence: 3+ backticks or tildes
            if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
                in_code_block = true;
                fence_char = trimmed.chars().next().unwrap();
                fence_len = trimmed.chars().take_while(|&c| c == fence_char).count();
                result.push('\n');
                continue;
            }
            result.push_str(line);
            result.push('\n');
        } else {
            // Check for closing fence: same char, at least same length
            if trimmed.starts_with(fence_char)
                && trimmed.chars().take_while(|&c| c == fence_char).count() >= fence_len
                && trimmed
                    .chars()
                    .skip_while(|&c| c == fence_char)
                    .all(|c| c.is_whitespace())
            {
                in_code_block = false;
            }
            result.push('\n');
        }
    }

    result
}

/// Extract inline `#tag` patterns from body text.
///
/// Tags are alphanumeric + hyphens + underscores, 1-64 chars.
fn extract_inline_tags(body: &str) -> Vec<String> {
    let mut tags = Vec::new();
    // Simple regex-free extraction: find #word patterns.
    for word in body.split_whitespace() {
        if let Some(tag) = word.strip_prefix('#') {
            // Must start with alphanumeric
            let tag_clean: String = tag
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '-' || *c == '_')
                .collect();
            if !tag_clean.is_empty()
                && tag_clean.len() <= 64
                && tag_clean.chars().next().is_some_and(|c| c.is_alphanumeric())
            {
                let normalized = tag_clean.to_lowercase();
                if !tags.contains(&normalized) {
                    tags.push(normalized);
                }
            }
        }
    }
    tags
}

/// Extract outgoing links from markdown body text.
///
/// Supports `[[wikilink]]` and `[text](path.md)` syntax.
fn extract_links(body: &str, _source_path: &str) -> Vec<String> {
    let mut links = Vec::new();

    // Extract [[wikilinks]]
    let mut rest = body;
    while let Some(start) = rest.find("[[") {
        let after_open = &rest[start + 2..];
        if let Some(end) = after_open.find("]]") {
            let link_content = &after_open[..end];
            // Handle [[target|display]] - take the target part.
            let target = link_content.split('|').next().unwrap_or("").trim();
            if !target.is_empty() {
                // Normalize: add .md if no extension
                let target_path = if Path::new(target).extension().is_none() {
                    format!("{target}.md")
                } else {
                    target.to_string()
                };
                if !links.contains(&target_path) {
                    links.push(target_path);
                }
            }
            rest = &after_open[end + 2..];
        } else {
            break;
        }
    }

    // Extract [text](path) markdown links - only relative paths
    let mut rest = body;
    while let Some(bracket_start) = rest.find("](") {
        let after_paren = &rest[bracket_start + 2..];
        if let Some(paren_end) = after_paren.find(')') {
            let link_target = &after_paren[..paren_end].trim();
            // Skip absolute URLs and anchors
            if !link_target.starts_with("http://")
                && !link_target.starts_with("https://")
                && !link_target.starts_with('#')
                && !link_target.is_empty()
            {
                let target = link_target.trim_start_matches("./");
                if !links.contains(&target.to_string()) {
                    links.push(target.to_string());
                }
            }
            rest = &after_paren[paren_end + 1..];
        } else {
            break;
        }
    }

    links
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn temp_workspace() -> tempfile::TempDir {
        tempfile::tempdir().expect("failed to create temp dir")
    }

    #[test]
    fn new_index_is_empty() {
        let dir = temp_workspace();
        let index = FileIndex::new(dir.path().to_path_buf());
        assert_eq!(index.get_file_count(), 0);
        assert_eq!(index.get_note_count(), 0);
        assert_eq!(index.get_tag_count(), 0);
        assert!(!index.is_populated());
    }

    #[test]
    fn full_scan_indexes_files() {
        let dir = temp_workspace();
        fs::write(dir.path().join("a.md"), "# Hello\n\nSome text").unwrap();
        fs::write(dir.path().join("b.txt"), "plain text").unwrap();
        fs::create_dir(dir.path().join("sub")).unwrap();
        fs::write(dir.path().join("sub/c.md"), "# Sub note").unwrap();

        let index = FileIndex::new(dir.path().to_path_buf());
        let result = index.full_scan();

        assert_eq!(result.files_count, 3);
        assert_eq!(result.notes_count, 2);
        assert!(result.duration_ms < 5000);

        assert_eq!(index.get_file_count(), 3);
        assert_eq!(index.get_note_count(), 2);
        assert!(index.is_populated());
    }

    #[test]
    fn full_scan_respects_ignore_file() {
        let dir = temp_workspace();
        // The `ignore` crate always respects `.ignore` files (no git repo needed).
        // `.gitignore` requires a git repository context.
        fs::write(dir.path().join(".ignore"), "ignored/\n").unwrap();
        fs::write(dir.path().join("visible.md"), "visible").unwrap();
        fs::create_dir(dir.path().join("ignored")).unwrap();
        fs::write(dir.path().join("ignored/secret.md"), "secret").unwrap();

        let index = FileIndex::new(dir.path().to_path_buf());
        index.full_scan();

        // .ignore itself + visible.md = 2 files; ignored/ should be excluded
        assert!(index.get_file_entry("visible.md").is_some());
        assert!(index.get_file_entry("ignored/secret.md").is_none());
    }

    #[test]
    fn incremental_create_event() {
        let dir = temp_workspace();
        fs::write(dir.path().join("initial.md"), "# Init").unwrap();

        let index = FileIndex::new(dir.path().to_path_buf());
        index.full_scan();
        assert_eq!(index.get_file_count(), 1);

        // Create a new file
        let new_path = dir.path().join("added.md");
        fs::write(&new_path, "# Added").unwrap();

        let changes = index.handle_event(&[new_path], ChangeKind::Created);
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].change_kind, ChangeKind::Created);
        assert_eq!(changes[0].path, "added.md");
        assert_eq!(index.get_file_count(), 2);
        assert_eq!(index.get_note_count(), 2);
    }

    #[test]
    fn incremental_modify_event() {
        let dir = temp_workspace();
        let file_path = dir.path().join("note.md");
        fs::write(&file_path, "# Original").unwrap();

        let index = FileIndex::new(dir.path().to_path_buf());
        index.full_scan();

        let meta = index.get_note_metadata("note.md").unwrap();
        assert_eq!(meta.title, "Original");

        // Modify the file
        fs::write(&file_path, "# Updated Title\n\nNew content").unwrap();
        let changes = index.handle_event(&[file_path], ChangeKind::Changed);
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].change_kind, ChangeKind::Changed);

        let meta = index.get_note_metadata("note.md").unwrap();
        assert_eq!(meta.title, "Updated Title");
    }

    #[test]
    fn incremental_delete_event() {
        let dir = temp_workspace();
        let file_path = dir.path().join("doomed.md");
        fs::write(&file_path, "# Doomed").unwrap();

        let index = FileIndex::new(dir.path().to_path_buf());
        index.full_scan();
        assert_eq!(index.get_file_count(), 1);
        assert_eq!(index.get_note_count(), 1);

        fs::remove_file(&file_path).unwrap();
        let changes = index.handle_event(&[file_path], ChangeKind::Deleted);
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].change_kind, ChangeKind::Deleted);
        assert_eq!(index.get_file_count(), 0);
        assert_eq!(index.get_note_count(), 0);
    }

    #[test]
    fn note_metadata_extraction_with_frontmatter() {
        let dir = temp_workspace();
        let content = "---\ntitle: My Title\ntags: [project, idea]\n---\n\n# Heading\n\nBody text here with #inline-tag.\n\nLink to [[other-note]] and [ref](docs/ref.md).\n";
        fs::write(dir.path().join("note.md"), content).unwrap();

        let index = FileIndex::new(dir.path().to_path_buf());
        index.full_scan();

        let meta = index.get_note_metadata("note.md").unwrap();
        assert_eq!(meta.title, "My Title");
        assert!(meta.has_frontmatter);
        assert!(meta.tags.contains(&"project".to_string()));
        assert!(meta.tags.contains(&"idea".to_string()));
        assert!(meta.tags.contains(&"inline-tag".to_string()));
        assert!(meta.links_out.contains(&"other-note.md".to_string()));
        assert!(meta.links_out.contains(&"docs/ref.md".to_string()));
        assert!(meta.word_count > 0);
    }

    #[test]
    fn note_metadata_uses_heading_when_no_frontmatter() {
        let dir = temp_workspace();
        fs::write(dir.path().join("note.md"), "# My Heading\n\nBody text.\n").unwrap();

        let index = FileIndex::new(dir.path().to_path_buf());
        index.full_scan();

        let meta = index.get_note_metadata("note.md").unwrap();
        assert_eq!(meta.title, "My Heading");
        assert!(!meta.has_frontmatter);
    }

    #[test]
    fn note_metadata_uses_filename_as_fallback_title() {
        let dir = temp_workspace();
        fs::write(dir.path().join("plain.md"), "Just some text.\n").unwrap();

        let index = FileIndex::new(dir.path().to_path_buf());
        index.full_scan();

        let meta = index.get_note_metadata("plain.md").unwrap();
        assert_eq!(meta.title, "plain");
    }

    #[test]
    fn tag_count_is_correct() {
        let dir = temp_workspace();
        fs::write(
            dir.path().join("a.md"),
            "---\ntags: [alpha, beta]\n---\n\nText #gamma.\n",
        )
        .unwrap();
        fs::write(
            dir.path().join("b.md"),
            "---\ntags: [beta, delta]\n---\n\nMore text.\n",
        )
        .unwrap();

        let index = FileIndex::new(dir.path().to_path_buf());
        index.full_scan();

        // Unique tags: alpha, beta, gamma, delta = 4
        assert_eq!(index.get_tag_count(), 4);

        let tags = index.get_all_tags();
        assert_eq!(*tags.get("beta").unwrap(), 2);
        assert_eq!(*tags.get("alpha").unwrap(), 1);
    }

    #[test]
    fn extract_inline_tags_works() {
        let tags = extract_inline_tags("Hello #world and #rust-lang are great #123bad");
        assert!(tags.contains(&"world".to_string()));
        assert!(tags.contains(&"rust-lang".to_string()));
        // 123bad should be excluded because it starts with a digit... actually
        // digits are alphanumeric, so it should be included.
        assert!(tags.contains(&"123bad".to_string()));
    }

    #[test]
    fn extract_links_wikilinks_and_markdown() {
        let body = "See [[note-a]] and [[folder/note-b|display]] and [link](path/to.md).";
        let links = extract_links(body, "source.md");
        assert!(links.contains(&"note-a.md".to_string()));
        assert!(links.contains(&"folder/note-b.md".to_string()));
        assert!(links.contains(&"path/to.md".to_string()));
    }

    #[test]
    fn extract_links_skips_urls() {
        let body = "See [external](https://example.com) and [internal](local.md).";
        let links = extract_links(body, "source.md");
        assert!(!links.iter().any(|l| l.contains("example.com")));
        assert!(links.contains(&"local.md".to_string()));
    }

    #[test]
    fn parse_frontmatter_yaml_list_tags() {
        let content = "---\ntags:\n  - alpha\n  - beta\n---\n\nBody.\n";
        let (has_fm, _title, tags, body) = parse_frontmatter(content);
        assert!(has_fm);
        assert!(tags.contains(&"alpha".to_string()));
        assert!(tags.contains(&"beta".to_string()));
        assert!(body.starts_with("Body"));
    }

    #[test]
    fn strip_fenced_code_blocks_removes_content() {
        let body = "Before\n```\n#code-tag\n```\nAfter #real-tag\n";
        let stripped = strip_fenced_code_blocks(body);
        assert!(!stripped.contains("#code-tag"));
        assert!(stripped.contains("#real-tag"));
        assert!(stripped.contains("Before"));
        assert!(stripped.contains("After"));
    }

    #[test]
    fn strip_fenced_code_blocks_with_tilde() {
        let body = "Text\n~~~python\n#not-a-tag\n~~~\n#yes-a-tag\n";
        let stripped = strip_fenced_code_blocks(body);
        assert!(!stripped.contains("#not-a-tag"));
        assert!(stripped.contains("#yes-a-tag"));
    }

    #[test]
    fn inline_tags_not_extracted_from_code_blocks() {
        let dir = temp_workspace();
        let content = "# Note\n\nReal #valid-tag here.\n\n```\n#code-tag\n```\n";
        fs::write(dir.path().join("note.md"), content).unwrap();

        let index = FileIndex::new(dir.path().to_path_buf());
        index.full_scan();

        let meta = index.get_note_metadata("note.md").unwrap();
        assert!(meta.tags.contains(&"valid-tag".to_string()));
        assert!(
            !meta.tags.contains(&"code-tag".to_string()),
            "tags inside code blocks should not be indexed"
        );
    }

    #[test]
    fn skip_git_directory_in_events() {
        let dir = temp_workspace();
        fs::create_dir(dir.path().join(".git")).unwrap();
        fs::write(dir.path().join(".git/config"), "").unwrap();
        fs::write(dir.path().join("real.md"), "# Real").unwrap();

        let index = FileIndex::new(dir.path().to_path_buf());
        index.full_scan();

        let git_path = dir.path().join(".git/config");
        let changes = index.handle_event(&[git_path], ChangeKind::Changed);
        assert!(changes.is_empty());
    }
}
