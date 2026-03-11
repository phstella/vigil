//! Workspace status command wrapper.
//!
//! Aggregates git branch/sync state, index metrics, and app version into
//! a single `WorkspaceStatus` response for the footer status bar.

use tauri::State;

use crate::core::git::status as git_status;
use crate::core::index::metrics;
use crate::models::error::VigilError;
use crate::models::status::WorkspaceStatus;
use crate::state::AppState;

/// Application version from Cargo.toml, embedded at compile time.
const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Query the current workspace status.
///
/// Aggregates:
/// - Git branch name and sync state (from the workspace root)
/// - Notes, tags, and files counts (from file and tag indices)
/// - Application version (from Cargo.toml)
/// - Tracked last index update timestamp
///
/// Returns `WORKSPACE_NOT_OPEN` when no workspace is active, per IPC contract.
#[tauri::command]
pub async fn workspace_status(state: State<'_, AppState>) -> Result<WorkspaceStatus, VigilError> {
    // Require an open workspace per IPC contract.
    let ws = state.workspace().ok_or(VigilError::WorkspaceNotOpen)?;

    let root = ws.root().to_path_buf();
    let branch = git_status::current_branch(&root);
    let sync_state = git_status::sync_state(&root);

    // Collect index metrics.
    let file_index = state.index();
    let tag_index = state.tag_index();
    let workspace_metrics = metrics::collect_metrics(file_index.as_ref(), tag_index.as_ref());

    // Use the tracked index update timestamp, not "now".
    let last_index_update_ms = state.last_index_update_ms();

    Ok(WorkspaceStatus {
        branch,
        sync_state,
        notes_count: workspace_metrics.notes_count,
        tags_count: workspace_metrics.tags_count,
        files_count: workspace_metrics.files_count,
        version: APP_VERSION.to_string(),
        last_index_update_ms,
    })
}
