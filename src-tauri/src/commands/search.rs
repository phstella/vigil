//! Fuzzy-find and full-text search command wrappers.

use tauri::State;

use crate::core::search::FuzzyFinder;
use crate::models::error::VigilError;
use crate::models::search::FuzzyFindResponse;
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
