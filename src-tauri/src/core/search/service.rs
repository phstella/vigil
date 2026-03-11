//! Fuzzy file finder service for the omnibar filename mode.
//!
//! `FuzzyFinder` takes a reference to a [`FileIndex`] and scores each indexed
//! file's workspace-relative path against a user query using `nucleo-matcher`.
//! Results are returned sorted by score descending, capped to the requested
//! limit.

use nucleo_matcher::pattern::{AtomKind, CaseMatching, Normalization, Pattern};
use nucleo_matcher::{Config, Matcher, Utf32Str};

use crate::core::index::FileIndex;
use crate::models::files::EntryKind;
use crate::models::search::FuzzyMatch;

/// Maximum number of results the finder will ever return.
const MAX_RESULTS: usize = 200;

/// Fuzzy file finder backed by `nucleo-matcher`.
///
/// Designed to be created on-demand per query (stateless) since it only
/// borrows the [`FileIndex`] for the duration of a single `fuzzy_find` call.
pub struct FuzzyFinder<'a> {
    index: &'a FileIndex,
}

impl<'a> FuzzyFinder<'a> {
    /// Create a new finder that reads from the given file index.
    pub fn new(index: &'a FileIndex) -> Self {
        Self { index }
    }

    /// Run a fuzzy search over all indexed files and return the top `limit`
    /// matches sorted by score descending.
    ///
    /// When `query` is empty, returns the most-recently-modified files up to
    /// `limit` (providing a useful default for the omnibar).
    pub fn fuzzy_find(&self, query: &str, limit: usize) -> Vec<FuzzyMatch> {
        let limit = limit.min(MAX_RESULTS);
        let files = self.index.get_all_files();

        // Empty query: return most-recently-modified files (only actual files,
        // not directories).
        if query.is_empty() {
            return self.recent_files(&files, limit);
        }

        // Configure matcher for path matching.
        let config = Config::DEFAULT.match_paths();
        let mut matcher = Matcher::new(config);

        // Parse the query into a nucleo Pattern (handles multi-word queries,
        // case-smart matching, etc.).
        let pattern = Pattern::new(
            query,
            CaseMatching::Smart,
            Normalization::Smart,
            AtomKind::Fuzzy,
        );

        let mut scored: Vec<FuzzyMatch> = Vec::with_capacity(files.len().min(limit * 4));
        let mut haystack_buf = Vec::new();
        let mut indices_buf: Vec<u32> = Vec::new();

        for entry in &files {
            // Only match actual files, not directories.
            if entry.kind != EntryKind::File {
                continue;
            }

            let haystack = Utf32Str::new(&entry.path, &mut haystack_buf);

            indices_buf.clear();
            if let Some(score) = pattern.indices(haystack, &mut matcher, &mut indices_buf) {
                scored.push(FuzzyMatch {
                    path: entry.path.clone(),
                    display: entry.path.clone(),
                    score: f64::from(score),
                    kind: entry.kind,
                    matched_indices: indices_buf.clone(),
                });
            }
        }

        // Sort by score descending, then by path ascending for stability.
        scored.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.path.cmp(&b.path))
        });

        scored.truncate(limit);
        scored
    }

    /// Return the most-recently-modified files, sorted by `modified_at_ms`
    /// descending. Used when the query is empty.
    fn recent_files(
        &self,
        files: &[crate::models::files::FileEntry],
        limit: usize,
    ) -> Vec<FuzzyMatch> {
        let mut entries: Vec<_> = files
            .iter()
            .filter(|e| e.kind == EntryKind::File)
            .collect();

        entries.sort_by(|a, b| b.modified_at_ms.cmp(&a.modified_at_ms));
        entries.truncate(limit);

        entries
            .into_iter()
            .map(|e| FuzzyMatch {
                path: e.path.clone(),
                display: e.path.clone(),
                score: 0.0,
                kind: e.kind,
                matched_indices: Vec::new(),
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn temp_workspace() -> tempfile::TempDir {
        tempfile::tempdir().expect("failed to create temp dir")
    }

    fn build_index(dir: &std::path::Path) -> FileIndex {
        let index = FileIndex::new(dir.to_path_buf());
        index.full_scan();
        index
    }

    #[test]
    fn empty_query_returns_recent_files() {
        let dir = temp_workspace();
        fs::write(dir.path().join("a.md"), "# A").unwrap();
        fs::write(dir.path().join("b.md"), "# B").unwrap();
        fs::write(dir.path().join("c.txt"), "text").unwrap();

        let index = build_index(dir.path());
        let finder = FuzzyFinder::new(&index);
        let results = finder.fuzzy_find("", 10);

        assert_eq!(results.len(), 3);
        // All scores should be 0 (no matching performed).
        assert!(results.iter().all(|m| m.score == 0.0));
        // All should be files.
        assert!(results.iter().all(|m| m.kind == EntryKind::File));
    }

    #[test]
    fn fuzzy_match_returns_scored_results() {
        let dir = temp_workspace();
        fs::write(dir.path().join("hello.md"), "# Hello").unwrap();
        fs::write(dir.path().join("help.txt"), "help").unwrap();
        fs::write(dir.path().join("world.md"), "# World").unwrap();

        let index = build_index(dir.path());
        let finder = FuzzyFinder::new(&index);
        let results = finder.fuzzy_find("hel", 10);

        // Should match hello.md and help.txt but not world.md.
        assert!(results.len() >= 2);
        let paths: Vec<&str> = results.iter().map(|m| m.path.as_str()).collect();
        assert!(paths.contains(&"hello.md"));
        assert!(paths.contains(&"help.txt"));
        assert!(!paths.contains(&"world.md"));

        // All scores should be positive.
        assert!(results.iter().all(|m| m.score > 0.0));
        // Matched indices should be non-empty.
        assert!(results.iter().all(|m| !m.matched_indices.is_empty()));
    }

    #[test]
    fn limit_caps_results() {
        let dir = temp_workspace();
        for i in 0..20 {
            fs::write(
                dir.path().join(format!("note-{i}.md")),
                format!("# Note {i}"),
            )
            .unwrap();
        }

        let index = build_index(dir.path());
        let finder = FuzzyFinder::new(&index);
        let results = finder.fuzzy_find("note", 5);

        assert_eq!(results.len(), 5);
    }

    #[test]
    fn results_sorted_by_score_descending() {
        let dir = temp_workspace();
        fs::write(dir.path().join("abc.md"), "# ABC").unwrap();
        fs::write(dir.path().join("a_b_c.md"), "text").unwrap();
        fs::write(dir.path().join("axbxcx.md"), "text").unwrap();

        let index = build_index(dir.path());
        let finder = FuzzyFinder::new(&index);
        let results = finder.fuzzy_find("abc", 10);

        for window in results.windows(2) {
            assert!(window[0].score >= window[1].score);
        }
    }

    #[test]
    fn directories_are_excluded() {
        let dir = temp_workspace();
        fs::create_dir(dir.path().join("subdir")).unwrap();
        fs::write(dir.path().join("subdir/file.md"), "# File").unwrap();

        let index = build_index(dir.path());
        let finder = FuzzyFinder::new(&index);
        let results = finder.fuzzy_find("sub", 10);

        // Only the file should match, not the directory entry.
        assert!(results.iter().all(|m| m.kind == EntryKind::File));
    }

    #[test]
    fn max_limit_is_clamped() {
        let dir = temp_workspace();
        for i in 0..5 {
            fs::write(
                dir.path().join(format!("f{i}.md")),
                format!("# F{i}"),
            )
            .unwrap();
        }

        let index = build_index(dir.path());
        let finder = FuzzyFinder::new(&index);
        // Request more than MAX_RESULTS, should be clamped.
        let results = finder.fuzzy_find("f", 999);
        assert!(results.len() <= MAX_RESULTS);
    }

    #[test]
    fn path_matching_works_with_slashes() {
        let dir = temp_workspace();
        fs::create_dir_all(dir.path().join("notes/daily")).unwrap();
        fs::write(
            dir.path().join("notes/daily/today.md"),
            "# Today",
        )
        .unwrap();
        fs::write(dir.path().join("readme.md"), "# Readme").unwrap();

        let index = build_index(dir.path());
        let finder = FuzzyFinder::new(&index);
        let results = finder.fuzzy_find("daily/today", 10);

        assert!(!results.is_empty());
        assert_eq!(results[0].path, "notes/daily/today.md");
    }
}
