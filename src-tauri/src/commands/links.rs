//! Backlink query and link graph command wrappers.

use tauri::State;

use crate::models::error::VigilError;
use crate::models::links::BacklinksResponse;
use crate::state::AppState;

/// Shared `get_backlinks` command logic, split for direct testing.
pub fn get_backlinks_for_state(
    path: &str,
    state: &AppState,
) -> Result<BacklinksResponse, VigilError> {
    // Match IPC contract precedence: workspace requirement is checked first.
    state.workspace().ok_or(VigilError::WorkspaceNotOpen)?;

    let graph = state.link_graph().ok_or(VigilError::IndexUnavailable)?;
    let backlinks = graph.get_backlinks(path);

    Ok(BacklinksResponse { backlinks })
}

/// Get notes that link to the specified file.
///
/// Queries the in-memory link graph for all backlinks pointing at the given
/// workspace-relative path. Returns an empty list if no notes link to the target.
#[tauri::command]
pub async fn get_backlinks(
    path: String,
    state: State<'_, AppState>,
) -> Result<BacklinksResponse, VigilError> {
    get_backlinks_for_state(&path, &state)
}
