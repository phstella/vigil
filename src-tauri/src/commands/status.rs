//! Workspace status command wrapper.
//!
//! Aggregates git branch/sync state, index metrics, and app version into
//! a single `WorkspaceStatus` response for the footer status bar.

use std::time::{SystemTime, UNIX_EPOCH};

use tauri::State;

use crate::core::git::status as git_status;
use crate::core::index::metrics;
use crate::models::error::VigilError;
use crate::models::status::{SyncState, WorkspaceStatus};
use crate::state::AppState;

/// Application version from Cargo.toml, embedded at compile time.
const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Query the current workspace status.
///
/// Aggregates:
/// - Git branch name and sync state (from the workspace root)
/// - Notes, tags, and files counts (from file and tag indices)
/// - Application version (from Cargo.toml)
/// - Current timestamp as the last index update marker
///
/// Returns default/zero values for fields when no workspace is open or
/// indices are not yet built.
#[tauri::command]
pub async fn workspace_status(
    state: State<'_, AppState>,
) -> Result<WorkspaceStatus, VigilError> {
    // Determine git info from the workspace root.
    let (branch, sync_state) = match state.workspace() {
        Some(ws) => {
            let root = ws.root().to_path_buf();
            let branch = git_status::current_branch(&root);
            let sync = git_status::sync_state(&root);
            (branch, sync)
        }
        None => (None, SyncState::Unknown),
    };

    // Collect index metrics.
    let file_index = state.index();
    let tag_index = state.tag_index();
    let workspace_metrics = metrics::collect_metrics(
        file_index.as_ref(),
        tag_index.as_ref(),
    );

    let last_index_update_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0);

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
