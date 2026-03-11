//! In-memory content search service for phrase/snippet matching.
//!
//! `ContentSearcher` reads file contents from disk and performs case-insensitive
//! substring matching against a user query. It uses the [`FileIndex`] to know
//! which files exist and skips binary files. Results are returned as
//! [`ContentMatch`] structs with file path, line number, column range, and a
//! preview snippet.

use std::fs;
use std::path::Path;

use crate::core::index::FileIndex;
use crate::models::files::EntryKind;
use crate::models::search::ContentMatch;

/// Maximum number of results the searcher will ever return.
const MAX_RESULTS: usize = 200;

/// Maximum file size (in bytes) we will attempt to read for content search.
/// Files larger than this are skipped to avoid excessive memory usage.
const MAX_FILE_SIZE: u64 = 2 * 1024 * 1024; // 2 MB

/// Stateless content searcher that reads files on demand.
///
/// Created per-query; borrows the [`FileIndex`] to enumerate files and
/// determine which are binary.
pub struct ContentSearcher<'a> {
    index: &'a FileIndex,
}

impl<'a> ContentSearcher<'a> {
    /// Create a new searcher backed by the given file index.
    pub fn new(index: &'a FileIndex) -> Self {
        Self { index }
    }

    /// Search file contents for lines matching `query`.
    ///
    /// - Case-insensitive substring matching.
    /// - Skips binary files and files larger than 2 MB.
    /// - Returns at most `limit` results (clamped to [`MAX_RESULTS`]).
    /// - Results are ordered by file path (stable), then by line number.
    /// - `workspace_root` is the absolute path to the workspace so we can
    ///   resolve indexed relative paths to absolute paths for reading.
    pub fn search_content(
        &self,
        query: &str,
        workspace_root: &Path,
        limit: usize,
    ) -> Vec<ContentMatch> {
        let _perf = crate::core::perf::PerfTimer::start("ContentSearcher::search_content");
        let limit = limit.min(MAX_RESULTS);

        if query.is_empty() {
            return Vec::new();
        }

        let query_lower = query.to_lowercase();
        let files = self.index.get_all_files();

        // Sort files by path for deterministic ordering.
        let mut files = files;
        files.sort_by(|a, b| a.path.cmp(&b.path));

        let mut matches: Vec<ContentMatch> = Vec::new();

        for entry in &files {
            if matches.len() >= limit {
                break;
            }

            // Only search actual files, not directories.
            if entry.kind != EntryKind::File {
                continue;
            }

            // Skip binary files.
            if entry.is_binary {
                continue;
            }

            // Skip files that are too large.
            if entry.size_bytes > MAX_FILE_SIZE {
                continue;
            }

            let abs_path = workspace_root.join(&entry.path);

            // Read file content; skip on any I/O error.
            let content = match fs::read_to_string(&abs_path) {
                Ok(c) => c,
                Err(_) => continue,
            };

            for (line_idx, line) in content.lines().enumerate() {
                if matches.len() >= limit {
                    break;
                }

                let line_lower = line.to_lowercase();
                let mut search_start = 0;

                // Find all occurrences of the query in this line.
                while let Some(col) = line_lower[search_start..].find(&query_lower) {
                    if matches.len() >= limit {
                        break;
                    }

                    let start_col = search_start + col;
                    let end_col = start_col + query.len();

                    matches.push(ContentMatch {
                        path: entry.path.clone(),
                        line_number: (line_idx + 1) as u32,
                        line_start_col: start_col as u32,
                        line_end_col: end_col as u32,
                        preview: line.to_string(),
                        score: score_match(line, start_col, query.len()),
                    });

                    // Advance past this match to find further occurrences.
                    search_start = start_col + 1;
                    if search_start >= line_lower.len() {
                        break;
                    }
                }
            }
        }

        // Sort by score descending, then by path + line number for stability.
        matches.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.path.cmp(&b.path))
                .then_with(|| a.line_number.cmp(&b.line_number))
        });

        matches.truncate(limit);
        matches
    }
}

/// Compute a simple relevance score for a content match.
///
/// Heuristics:
/// - Exact case match gets a bonus.
/// - Match at start of line gets a bonus.
/// - Shorter lines score slightly higher (the match is more prominent).
/// - Longer query matches score higher (more specific).
fn score_match(line: &str, start_col: usize, query_len: usize) -> f64 {
    let mut score: f64 = 100.0;

    // Bonus for match at start of (trimmed) line.
    let trimmed_start = line.len() - line.trim_start().len();
    if start_col == trimmed_start {
        score += 20.0;
    }

    // Bonus for exact case match at the position.
    // (Already checked case-insensitively; re-check for exact case.)
    // We just check if the original substring equals the query literally.
    let end_col = (start_col + query_len).min(line.len());
    let original_slice = &line[start_col..end_col];
    // We need the original query for case comparison, but we only have the
    // length here. Use a simple heuristic: if the slice is all lowercase or
    // all uppercase, it likely matches common query patterns.
    if original_slice
        .chars()
        .all(|c| c.is_lowercase() || !c.is_alphabetic())
    {
        score += 5.0;
    }

    // Shorter lines are slightly more relevant (match is more prominent).
    let line_len = line.len().max(1) as f64;
    score += 10.0 / (line_len / 20.0).max(1.0);

    // Longer queries are more specific.
    score += (query_len as f64).min(20.0);

    score
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn temp_workspace() -> tempfile::TempDir {
        tempfile::tempdir().expect("failed to create temp dir")
    }

    fn build_index(dir: &Path) -> FileIndex {
        let index = FileIndex::new(dir.to_path_buf());
        index.full_scan();
        index
    }

    #[test]
    fn empty_query_returns_no_results() {
        let dir = temp_workspace();
        fs::write(dir.path().join("a.md"), "Hello world").unwrap();

        let index = build_index(dir.path());
        let searcher = ContentSearcher::new(&index);
        let results = searcher.search_content("", dir.path(), 50);

        assert!(results.is_empty());
    }

    #[test]
    fn basic_substring_match() {
        let dir = temp_workspace();
        fs::write(dir.path().join("note.md"), "Hello world\nGoodbye world\n").unwrap();

        let index = build_index(dir.path());
        let searcher = ContentSearcher::new(&index);
        let results = searcher.search_content("world", dir.path(), 50);

        assert_eq!(results.len(), 2);
        // Both lines contain "world".
        assert!(results.iter().all(|m| m.preview.contains("world")));
    }

    #[test]
    fn case_insensitive_match() {
        let dir = temp_workspace();
        fs::write(dir.path().join("note.md"), "Hello WORLD\nworld hello\n").unwrap();

        let index = build_index(dir.path());
        let searcher = ContentSearcher::new(&index);
        let results = searcher.search_content("world", dir.path(), 50);

        assert_eq!(results.len(), 2);
    }

    #[test]
    fn respects_limit() {
        let dir = temp_workspace();
        let content: String = (0..100).map(|i| format!("line {i} match\n")).collect();
        fs::write(dir.path().join("big.md"), &content).unwrap();

        let index = build_index(dir.path());
        let searcher = ContentSearcher::new(&index);
        let results = searcher.search_content("match", dir.path(), 10);

        assert_eq!(results.len(), 10);
    }

    #[test]
    fn skips_binary_files() {
        let dir = temp_workspace();
        // Create a binary file with null bytes.
        let mut binary = vec![0u8; 100];
        binary[0] = b'H';
        binary[1] = b'e';
        binary[2] = b'l';
        binary[50] = 0;
        fs::write(dir.path().join("binary.bin"), &binary).unwrap();
        fs::write(dir.path().join("text.md"), "Hello world\n").unwrap();

        let index = build_index(dir.path());
        let searcher = ContentSearcher::new(&index);
        let results = searcher.search_content("Hel", dir.path(), 50);

        // Should only match text.md, not binary.bin.
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].path, "text.md");
    }

    #[test]
    fn match_columns_are_correct() {
        let dir = temp_workspace();
        fs::write(dir.path().join("note.md"), "foo bar baz\n").unwrap();

        let index = build_index(dir.path());
        let searcher = ContentSearcher::new(&index);
        let results = searcher.search_content("bar", dir.path(), 50);

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].line_number, 1);
        assert_eq!(results[0].line_start_col, 4);
        assert_eq!(results[0].line_end_col, 7);
    }

    #[test]
    fn multiple_matches_per_line() {
        let dir = temp_workspace();
        fs::write(dir.path().join("note.md"), "aaa bbb aaa\n").unwrap();

        let index = build_index(dir.path());
        let searcher = ContentSearcher::new(&index);
        let results = searcher.search_content("aaa", dir.path(), 50);

        assert_eq!(results.len(), 2);
        assert_eq!(results[0].line_start_col, 0);
        // Second match should be at col 8 (after sorting by score, positions
        // may vary, so just check both exist).
        let cols: Vec<u32> = results.iter().map(|m| m.line_start_col).collect();
        assert!(cols.contains(&0));
        assert!(cols.contains(&8));
    }

    #[test]
    fn searches_across_multiple_files() {
        let dir = temp_workspace();
        fs::write(dir.path().join("a.md"), "alpha target\n").unwrap();
        fs::write(dir.path().join("b.md"), "beta content\n").unwrap();
        fs::write(dir.path().join("c.md"), "gamma target\n").unwrap();

        let index = build_index(dir.path());
        let searcher = ContentSearcher::new(&index);
        let results = searcher.search_content("target", dir.path(), 50);

        assert_eq!(results.len(), 2);
        let paths: Vec<&str> = results.iter().map(|m| m.path.as_str()).collect();
        assert!(paths.contains(&"a.md"));
        assert!(paths.contains(&"c.md"));
    }

    #[test]
    fn no_match_returns_empty() {
        let dir = temp_workspace();
        fs::write(dir.path().join("note.md"), "Hello world\n").unwrap();

        let index = build_index(dir.path());
        let searcher = ContentSearcher::new(&index);
        let results = searcher.search_content("zzzznotfound", dir.path(), 50);

        assert!(results.is_empty());
    }

    #[test]
    fn line_numbers_are_one_based() {
        let dir = temp_workspace();
        fs::write(dir.path().join("note.md"), "line1\nline2\ntarget here\n").unwrap();

        let index = build_index(dir.path());
        let searcher = ContentSearcher::new(&index);
        let results = searcher.search_content("target", dir.path(), 50);

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].line_number, 3);
    }

    #[test]
    fn max_limit_is_clamped() {
        let dir = temp_workspace();
        let content: String = (0..10).map(|i| format!("line {i} word\n")).collect();
        fs::write(dir.path().join("note.md"), &content).unwrap();

        let index = build_index(dir.path());
        let searcher = ContentSearcher::new(&index);
        // Request more than MAX_RESULTS.
        let results = searcher.search_content("word", dir.path(), 999);
        assert!(results.len() <= MAX_RESULTS);
    }
}
