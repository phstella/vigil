//! Search-related models for fuzzy file matching and content search.

use serde::{Deserialize, Serialize};

use super::files::EntryKind;

/// Request payload for the `fuzzy_find` command.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchQuery {
    /// User input string.
    pub query: String,
    /// Maximum results (clamped to 200).
    pub limit: u32,
}

/// A single fuzzy filename match returned from the omnibar search.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FuzzyMatch {
    /// Workspace-relative file path.
    pub path: String,
    /// Formatted display string for omnibar.
    pub display: String,
    /// Match score (higher is better).
    pub score: f64,
    /// Entry type.
    pub kind: EntryKind,
    /// Character positions that matched in `display`.
    pub matched_indices: Vec<u32>,
}

/// Response payload for the `fuzzy_find` command.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FuzzyFindResponse {
    /// Ranked results, best first.
    pub matches: Vec<FuzzyMatch>,
}

/// A single content search match with snippet context (Epic 4).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentMatch {
    /// Workspace-relative file path.
    pub path: String,
    /// 1-based line number.
    pub line_number: u32,
    /// Start column of match within line.
    pub line_start_col: u32,
    /// End column of match within line.
    pub line_end_col: u32,
    /// Context line(s) around the match.
    pub preview: String,
    /// Relevance score.
    pub score: f64,
}

/// Response payload for the `search_content` command (Epic 4).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchContentResponse {
    /// Ranked snippet matches.
    pub matches: Vec<ContentMatch>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn search_query_roundtrip() {
        let q = SearchQuery {
            query: "hello".into(),
            limit: 50,
        };
        let json = serde_json::to_string(&q).unwrap();
        let deser: SearchQuery = serde_json::from_str(&json).unwrap();
        assert_eq!(deser.query, "hello");
        assert_eq!(deser.limit, 50);
    }

    #[test]
    fn fuzzy_match_roundtrip() {
        let m = FuzzyMatch {
            path: "notes/hello.md".into(),
            display: "hello.md".into(),
            score: 95.5,
            kind: EntryKind::File,
            matched_indices: vec![0, 1, 2, 3, 4],
        };
        let json = serde_json::to_string(&m).unwrap();
        let deser: FuzzyMatch = serde_json::from_str(&json).unwrap();
        assert_eq!(deser.path, "notes/hello.md");
        assert_eq!(deser.matched_indices.len(), 5);
    }

    #[test]
    fn content_match_roundtrip() {
        let m = ContentMatch {
            path: "notes/hello.md".into(),
            line_number: 42,
            line_start_col: 10,
            line_end_col: 15,
            preview: "some context here".into(),
            score: 88.0,
        };
        let json = serde_json::to_string(&m).unwrap();
        let deser: ContentMatch = serde_json::from_str(&json).unwrap();
        assert_eq!(deser.line_number, 42);
    }
}
