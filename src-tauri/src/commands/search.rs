//! Fuzzy-find and full-text search command wrappers.

use tauri::State;

use crate::core::content::ContentSearcher;
use crate::core::search::FuzzyFinder;
use crate::models::error::VigilError;
use crate::models::search::{FuzzyFindResponse, SearchContentResponse};
use crate::state::AppState;

/// Default result limit when the caller does not specify one.
const DEFAULT_LIMIT: u32 = 50;

/// Maximum result limit (per IPC contract).
const MAX_LIMIT: u32 = 200;

/// Search workspace file index by fuzzy filename matching.
///
/// Used by `Ctrl+P` omnibar in filename mode.
///
/// - `query`: user input string
/// - `limit`: maximum results (clamped to 200; defaults to 50)
#[tauri::command]
pub async fn fuzzy_find(
    query: String,
    limit: Option<u32>,
    state: State<'_, AppState>,
) -> Result<FuzzyFindResponse, VigilError> {
    let index = state.index().ok_or(VigilError::IndexUnavailable)?;

    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let finder = FuzzyFinder::new(&index);
    let matches = finder.fuzzy_find(&query, limit);

    Ok(FuzzyFindResponse { matches })
}

/// Search file contents for lines matching a phrase or snippet.
///
/// Used by `Ctrl+Shift+F` omnibar in content search mode.
///
/// - `query`: search string (case-insensitive substring match)
/// - `limit`: maximum results (clamped to 200; defaults to 50)
#[tauri::command]
pub async fn search_content(
    query: String,
    limit: Option<u32>,
    state: State<'_, AppState>,
) -> Result<SearchContentResponse, VigilError> {
    let index = state.index().ok_or(VigilError::IndexUnavailable)?;
    let workspace_root = index.root().to_path_buf();

    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let searcher = ContentSearcher::new(&index);
    let matches = searcher.search_content(&query, &workspace_root, limit);

    Ok(SearchContentResponse { matches })
}
