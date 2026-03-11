//! Backlink query and link graph command wrappers.

use tauri::State;

use crate::models::error::VigilError;
use crate::models::links::BacklinksResponse;
use crate::state::AppState;

/// Get notes that link to the specified file.
///
/// Queries the in-memory link graph for all backlinks pointing at the given
/// workspace-relative path. Returns an empty list if no notes link to the target.
#[tauri::command]
pub async fn get_backlinks(
    path: String,
    state: State<'_, AppState>,
) -> Result<BacklinksResponse, VigilError> {
    let graph = state.link_graph().ok_or(VigilError::IndexUnavailable)?;

    let backlinks = graph.get_backlinks(&path);

    Ok(BacklinksResponse { backlinks })
}
