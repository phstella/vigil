//! Git status, diff, blame command wrappers.

use tauri::State;

use crate::core::git::GitService;
use crate::models::error::VigilError;
use crate::models::git::{GitHunksResponse, GitStatusEntry};
use crate::state::AppState;

/// Helper to build a `GitService` from the current workspace.
fn require_git_service(state: &AppState) -> Result<GitService, VigilError> {
    let ws = state.workspace().ok_or(VigilError::WorkspaceNotOpen)?;
    Ok(GitService::new(ws.root().to_path_buf()))
}

/// Get line-level diff hunks for a file against git HEAD.
///
/// Returns an empty hunk list if the workspace is not a git repository
/// or the file is untracked.
#[tauri::command]
pub async fn get_git_hunks(
    path: String,
    state: State<'_, AppState>,
) -> Result<GitHunksResponse, VigilError> {
    let path = path.trim().to_string();
    if path.is_empty() {
        return Err(VigilError::InvalidArgument {
            reason: "path must not be empty".into(),
        });
    }
    let svc = require_git_service(&state)?;
    svc.get_hunks(&path)
}

/// Get git status of all files in the workspace.
///
/// Returns an empty list if the workspace is not a git repository.
#[tauri::command]
pub async fn get_git_status(state: State<'_, AppState>) -> Result<Vec<GitStatusEntry>, VigilError> {
    let svc = require_git_service(&state)?;
    svc.get_status()
}
