//! Workspace open command wrapper.

use tauri::State;

use crate::core::fs::WorkspaceFs;
use crate::models::error::VigilError;
use crate::models::workspace::OpenWorkspaceResponse;
use crate::state::AppState;

/// Open (or switch to) a workspace directory.
///
/// The path is canonicalized and must point to an existing directory. Previous
/// workspace state is discarded on switch.
#[tauri::command]
pub async fn open_workspace(
    root_path: String,
    state: State<'_, AppState>,
) -> Result<OpenWorkspaceResponse, VigilError> {
    let (ws, resp) = WorkspaceFs::open(&root_path)?;
    state.set_workspace(ws);
    Ok(resp)
}
