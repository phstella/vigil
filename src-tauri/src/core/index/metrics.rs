//! Workspace metrics collected from the file and tag indices.
//!
//! Provides a lightweight struct for the status bar: notes count, tags count,
//! files count, and the last index update timestamp.

use crate::core::index::FileIndex;
use crate::core::index::TagIndex;

/// Aggregated workspace metrics from the file and tag indices.
#[derive(Debug, Clone)]
pub struct WorkspaceMetrics {
    /// Number of markdown note files.
    pub notes_count: u64,
    /// Number of unique tags across all notes.
    pub tags_count: u64,
    /// Total number of indexed files (excluding directories).
    pub files_count: u64,
}

/// Collect workspace metrics from the file index and tag index.
///
/// If both are `Some`, real counts are returned. Otherwise, zero counts
/// are returned for unavailable indices.
pub fn collect_metrics(
    file_index: Option<&FileIndex>,
    tag_index: Option<&TagIndex>,
) -> WorkspaceMetrics {
    let (notes_count, files_count) = match file_index {
        Some(idx) => (idx.get_note_count(), idx.get_file_count()),
        None => (0, 0),
    };

    let tags_count = match tag_index {
        Some(idx) => idx.get_all_tags().len() as u64,
        None => 0,
    };

    WorkspaceMetrics {
        notes_count,
        tags_count,
        files_count,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn collect_metrics_with_none_indices() {
        let metrics = collect_metrics(None, None);
        assert_eq!(metrics.notes_count, 0);
        assert_eq!(metrics.tags_count, 0);
        assert_eq!(metrics.files_count, 0);
    }

    #[test]
    fn collect_metrics_with_populated_index() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("note.md"), "# Hello\n\nSome text #tag1").unwrap();
        fs::write(dir.path().join("other.txt"), "plain text").unwrap();

        let file_index = FileIndex::new(dir.path().to_path_buf());
        file_index.full_scan();

        let tag_index = TagIndex::new();
        tag_index.rebuild(&file_index);

        let metrics = collect_metrics(Some(&file_index), Some(&tag_index));
        assert_eq!(metrics.notes_count, 1);
        assert_eq!(metrics.files_count, 2);
        assert_eq!(metrics.tags_count, 1);
    }

    #[test]
    fn collect_metrics_with_file_index_only() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("note.md"), "# Hello").unwrap();

        let file_index = FileIndex::new(dir.path().to_path_buf());
        file_index.full_scan();

        let metrics = collect_metrics(Some(&file_index), None);
        assert_eq!(metrics.notes_count, 1);
        assert_eq!(metrics.files_count, 1);
        assert_eq!(metrics.tags_count, 0);
    }
}
