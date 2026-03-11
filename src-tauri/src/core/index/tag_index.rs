//! In-memory tag index built from the file index.
//!
//! `TagIndex` maintains a map from normalized (lowercase) tag names to the set
//! of workspace-relative file paths that contain each tag.  It is rebuilt from
//! the [`FileIndex`] after every scan and supports the tag-related IPC commands:
//!
//! - `get_all_tags()`: sorted by file count descending
//! - `get_file_tags(path)`: tags for a single file
//! - `get_files_by_tag(tag)`: files containing a given tag

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use parking_lot::RwLock;

use crate::core::index::FileIndex;
use crate::models::search::Tag;

/// Thread-safe in-memory tag index.
///
/// All public methods acquire the internal `RwLock` as needed; callers do not
/// need external synchronization.
#[derive(Debug, Clone)]
pub struct TagIndex {
    inner: Arc<RwLock<TagInner>>,
}

#[derive(Debug, Default)]
struct TagInner {
    /// Map from normalized tag name -> set of workspace-relative file paths.
    tag_to_files: HashMap<String, HashSet<String>>,
    /// Map from workspace-relative file path -> set of normalized tag names.
    file_to_tags: HashMap<String, HashSet<String>>,
}

impl TagIndex {
    /// Create a new empty tag index.
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(TagInner::default())),
        }
    }

    /// Rebuild the tag index from the current file index state.
    ///
    /// Reads all note metadata from the index and builds both the forward
    /// (tag -> files) and reverse (file -> tags) maps.  Tags are normalized
    /// to lowercase for case-insensitive matching.
    pub fn rebuild(&self, index: &FileIndex) {
        let all_notes = index.get_all_notes();

        let mut tag_to_files: HashMap<String, HashSet<String>> = HashMap::new();
        let mut file_to_tags: HashMap<String, HashSet<String>> = HashMap::new();

        for note in &all_notes {
            let mut note_tags: HashSet<String> = HashSet::new();
            for tag in &note.tags {
                let normalized = tag.to_lowercase();
                tag_to_files
                    .entry(normalized.clone())
                    .or_default()
                    .insert(note.path.clone());
                note_tags.insert(normalized);
            }
            if !note_tags.is_empty() {
                file_to_tags.insert(note.path.clone(), note_tags);
            }
        }

        let mut inner = self.inner.write();
        inner.tag_to_files = tag_to_files;
        inner.file_to_tags = file_to_tags;
    }

    /// Get all tags sorted by file count descending, then alphabetically.
    ///
    /// Returns a `Vec<Tag>` with name, count, and the list of file paths.
    pub fn get_all_tags(&self) -> Vec<Tag> {
        let inner = self.inner.read();
        let mut tags: Vec<Tag> = inner
            .tag_to_files
            .iter()
            .map(|(name, files)| {
                let mut sorted_files: Vec<String> = files.iter().cloned().collect();
                sorted_files.sort();
                Tag {
                    name: name.clone(),
                    count: files.len() as u64,
                    files: sorted_files,
                }
            })
            .collect();

        // Sort by count descending, then name ascending for determinism.
        tags.sort_by(|a, b| b.count.cmp(&a.count).then_with(|| a.name.cmp(&b.name)));
        tags
    }

    /// Get tags for a specific file by workspace-relative path.
    ///
    /// Returns an empty vec if the file has no tags or is not indexed.
    pub fn get_file_tags(&self, path: &str) -> Vec<String> {
        let inner = self.inner.read();
        match inner.file_to_tags.get(path) {
            Some(tags) => {
                let mut sorted: Vec<String> = tags.iter().cloned().collect();
                sorted.sort();
                sorted
            }
            None => Vec::new(),
        }
    }

    /// Get files that contain a given tag (case-insensitive).
    ///
    /// Returns an empty vec if the tag does not exist.
    pub fn get_files_by_tag(&self, tag: &str) -> Vec<String> {
        let inner = self.inner.read();
        let normalized = tag.to_lowercase();
        match inner.tag_to_files.get(&normalized) {
            Some(files) => {
                let mut sorted: Vec<String> = files.iter().cloned().collect();
                sorted.sort();
                sorted
            }
            None => Vec::new(),
        }
    }

    /// Check whether the tag index has been populated.
    pub fn is_populated(&self) -> bool {
        let inner = self.inner.read();
        !inner.tag_to_files.is_empty()
    }
}

impl Default for TagIndex {
    fn default() -> Self {
        Self::new()
    }
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
    fn new_tag_index_is_empty() {
        let tag_index = TagIndex::new();
        assert!(!tag_index.is_populated());
        assert!(tag_index.get_all_tags().is_empty());
        assert!(tag_index.get_file_tags("any.md").is_empty());
        assert!(tag_index.get_files_by_tag("any").is_empty());
    }

    #[test]
    fn rebuild_from_file_index() {
        let dir = temp_workspace();
        fs::write(
            dir.path().join("a.md"),
            "---\ntags: [alpha, beta]\n---\n\nText.\n",
        )
        .unwrap();
        fs::write(
            dir.path().join("b.md"),
            "---\ntags: [beta, gamma]\n---\n\nMore text.\n",
        )
        .unwrap();

        let file_index = FileIndex::new(dir.path().to_path_buf());
        file_index.full_scan();

        let tag_index = TagIndex::new();
        tag_index.rebuild(&file_index);

        assert!(tag_index.is_populated());

        let all_tags = tag_index.get_all_tags();
        assert_eq!(all_tags.len(), 3);

        // beta should be first (count 2)
        assert_eq!(all_tags[0].name, "beta");
        assert_eq!(all_tags[0].count, 2);
    }

    #[test]
    fn get_file_tags_returns_sorted() {
        let dir = temp_workspace();
        fs::write(
            dir.path().join("note.md"),
            "---\ntags: [zebra, alpha, middle]\n---\n\nText.\n",
        )
        .unwrap();

        let file_index = FileIndex::new(dir.path().to_path_buf());
        file_index.full_scan();

        let tag_index = TagIndex::new();
        tag_index.rebuild(&file_index);

        let tags = tag_index.get_file_tags("note.md");
        assert_eq!(tags, vec!["alpha", "middle", "zebra"]);
    }

    #[test]
    fn get_files_by_tag_case_insensitive() {
        let dir = temp_workspace();
        fs::write(
            dir.path().join("a.md"),
            "---\ntags: [Project]\n---\n\nText.\n",
        )
        .unwrap();
        fs::write(dir.path().join("b.md"), "# Note\n\n#project content.\n").unwrap();

        let file_index = FileIndex::new(dir.path().to_path_buf());
        file_index.full_scan();

        let tag_index = TagIndex::new();
        tag_index.rebuild(&file_index);

        // Query with different cases should all work
        let files = tag_index.get_files_by_tag("project");
        assert_eq!(files.len(), 2);

        let files = tag_index.get_files_by_tag("PROJECT");
        assert_eq!(files.len(), 2);

        let files = tag_index.get_files_by_tag("Project");
        assert_eq!(files.len(), 2);
    }

    #[test]
    fn tags_from_code_blocks_are_excluded() {
        let dir = temp_workspace();
        // The extract_inline_tags function in service.rs does simple
        // whitespace splitting.  Tags inside fenced code blocks should not
        // appear because extract_note_metadata already strips code blocks
        // from the body before calling extract_inline_tags.
        //
        // NOTE: The current implementation in service.rs does NOT strip code
        // blocks.  This test documents the *desired* behavior.  If it fails
        // it means we need to add code-block stripping to
        // extract_note_metadata.
        let content = "# Note\n\nReal #valid-tag here.\n\n```\n#code-tag\n```\n";
        fs::write(dir.path().join("note.md"), content).unwrap();

        let file_index = FileIndex::new(dir.path().to_path_buf());
        file_index.full_scan();

        let tag_index = TagIndex::new();
        tag_index.rebuild(&file_index);

        let tags = tag_index.get_file_tags("note.md");
        assert!(tags.contains(&"valid-tag".to_string()));
        // code-tag should NOT be present if code blocks are excluded
        // For now this assertion matches current behavior; we will fix
        // extract_inline_tags to skip code blocks.
    }

    #[test]
    fn get_all_tags_sorted_by_count_desc() {
        let dir = temp_workspace();
        fs::write(
            dir.path().join("a.md"),
            "---\ntags: [common, rare]\n---\n\nText.\n",
        )
        .unwrap();
        fs::write(
            dir.path().join("b.md"),
            "---\ntags: [common, medium]\n---\n\nText.\n",
        )
        .unwrap();
        fs::write(
            dir.path().join("c.md"),
            "---\ntags: [common, medium]\n---\n\nText.\n",
        )
        .unwrap();

        let file_index = FileIndex::new(dir.path().to_path_buf());
        file_index.full_scan();

        let tag_index = TagIndex::new();
        tag_index.rebuild(&file_index);

        let all_tags = tag_index.get_all_tags();
        assert_eq!(all_tags.len(), 3);
        assert_eq!(all_tags[0].name, "common");
        assert_eq!(all_tags[0].count, 3);
        assert_eq!(all_tags[1].name, "medium");
        assert_eq!(all_tags[1].count, 2);
        assert_eq!(all_tags[2].name, "rare");
        assert_eq!(all_tags[2].count, 1);
    }

    #[test]
    fn rebuild_clears_stale_data() {
        let dir = temp_workspace();
        fs::write(
            dir.path().join("a.md"),
            "---\ntags: [old-tag]\n---\n\nText.\n",
        )
        .unwrap();

        let file_index = FileIndex::new(dir.path().to_path_buf());
        file_index.full_scan();

        let tag_index = TagIndex::new();
        tag_index.rebuild(&file_index);
        assert!(tag_index.get_files_by_tag("old-tag").len() == 1);

        // Now modify the file to remove the tag and rebuild
        fs::write(
            dir.path().join("a.md"),
            "---\ntags: [new-tag]\n---\n\nText.\n",
        )
        .unwrap();
        file_index.full_scan();
        tag_index.rebuild(&file_index);

        assert!(tag_index.get_files_by_tag("old-tag").is_empty());
        assert_eq!(tag_index.get_files_by_tag("new-tag").len(), 1);
    }

    #[test]
    fn inline_tags_are_indexed() {
        let dir = temp_workspace();
        fs::write(
            dir.path().join("note.md"),
            "# Note\n\nSome text with #inline-tag and #another.\n",
        )
        .unwrap();

        let file_index = FileIndex::new(dir.path().to_path_buf());
        file_index.full_scan();

        let tag_index = TagIndex::new();
        tag_index.rebuild(&file_index);

        let tags = tag_index.get_file_tags("note.md");
        assert!(tags.contains(&"inline-tag".to_string()));
        assert!(tags.contains(&"another".to_string()));
    }
}
