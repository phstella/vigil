//! Workspace open command wrapper.

use std::time::{SystemTime, UNIX_EPOCH};

use tauri::{AppHandle, State};

use crate::core::fs::WorkspaceFs;
use crate::core::git::GitService;
use crate::core::index::{FileIndex, FileWatcher, TagIndex};
use crate::core::links::LinkGraph;
use crate::events::{git_events, index_events, status_events};
use crate::models::error::VigilError;
use crate::models::status::WorkspaceStatus;
use crate::models::workspace::OpenWorkspaceResponse;
use crate::state::AppState;

/// Application version from Cargo.toml, embedded at compile time.
const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Helper: current unix epoch ms.
fn now_epoch_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

/// Open (or switch to) a workspace directory.
///
/// The path is canonicalized and must point to an existing directory. Previous
/// workspace state is discarded on switch. After opening:
/// 1. Runs a full index scan of the workspace.
/// 2. Starts a recursive file watcher for incremental updates.
/// 3. Emits `vigil://index-ready` when the initial scan completes.
#[tauri::command]
pub async fn open_workspace(
    root_path: String,
    state: State<'_, AppState>,
    app_handle: AppHandle,
) -> Result<OpenWorkspaceResponse, VigilError> {
    // Clear previous workspace state (watcher, index, workspace).
    state.clear_all();

    // Open the workspace filesystem service.
    let (ws, resp) = WorkspaceFs::open(&root_path)?;
    let root = ws.root().to_path_buf();

    state.set_workspace(ws);

    // Build the file index with a full scan.
    let index = FileIndex::new(root.clone());
    let scan_result = index.full_scan();

    // Record index update timestamp.
    let index_ts = now_epoch_ms();
    state.set_last_index_update_ms(index_ts);
    state.set_index(index.clone());

    // Build the link graph from the indexed notes.
    let link_graph = LinkGraph::new();
    link_graph.rebuild(&index);
    state.set_link_graph(link_graph);

    // Build the tag index from the indexed notes.
    let tag_index = TagIndex::new();
    tag_index.rebuild(&index);
    state.set_tag_index(tag_index);

    // Emit index-ready event.
    index_events::emit_index_ready(
        &app_handle,
        scan_result.files_count,
        scan_result.notes_count,
        scan_result.duration_ms,
    );

    // Start the file watcher for incremental updates.
    // We need clones of state arcs for the watcher callback.
    let watcher_index = index;
    let watcher_app_handle = app_handle.clone();
    let watcher_root = root.clone();

    // Capture Arc-wrapped state fields for the watcher callback.
    let w_link_graph = state.link_graph().unwrap_or_default();
    let w_tag_index = state.tag_index().unwrap_or_default();
    // We need a way to update last_index_update_ms from the watcher.
    // The watcher callback cannot access State<AppState>, but we can share
    // the atomic directly.
    let w_last_index_ms = {
        // Access the field through a helper method that returns the Arc.
        state.last_index_update_arc()
    };

    // Determine git info for status events.
    let w_git_root = root.clone();

    match FileWatcher::start(&root, move |events| {
        // Group events by change kind for batch processing.
        use crate::core::index::ChangeKind;
        use std::collections::HashMap;

        let mut by_kind: HashMap<ChangeKind, Vec<std::path::PathBuf>> = HashMap::new();
        for event in events {
            by_kind.entry(event.kind).or_default().push(event.path);
        }

        let mut all_changes = Vec::new();
        for (kind, paths) in &by_kind {
            let changes = watcher_index.handle_event(paths, *kind);
            all_changes.extend(changes);
        }

        if !all_changes.is_empty() {
            // Record index update timestamp.
            let ts = now_epoch_ms();
            w_last_index_ms.store(ts, std::sync::atomic::Ordering::Release);

            // Emit index-updated event.
            index_events::emit_index_updated(&watcher_app_handle, &all_changes);

            // Rebuild dependent state: tag index and link graph.
            w_tag_index.rebuild(&watcher_index);
            w_link_graph.rebuild(&watcher_index);

            // Emit git hunks for changed/created markdown files (non-delete).
            // This is best-effort: non-git workspaces will produce empty hunks.
            let git_svc = GitService::new(watcher_root.clone());
            for change in &all_changes {
                if change.change_kind != ChangeKind::Deleted && change.path.ends_with(".md") {
                    if let Ok(resp) = git_svc.get_hunks(&change.path) {
                        git_events::emit_git_hunks(&watcher_app_handle, &change.path, resp.hunks);
                    }
                }
            }

            // Emit status-updated event.
            let branch = crate::core::git::status::current_branch(&w_git_root);
            let sync = crate::core::git::status::sync_state(&w_git_root);
            let file_index_ref = &watcher_index;
            let metrics = crate::core::index::metrics::collect_metrics(
                Some(file_index_ref),
                Some(&w_tag_index),
            );
            let status = WorkspaceStatus {
                branch,
                sync_state: sync,
                notes_count: metrics.notes_count,
                tags_count: metrics.tags_count,
                files_count: metrics.files_count,
                version: APP_VERSION.to_string(),
                last_index_update_ms: ts,
            };
            status_events::emit_status_updated(&watcher_app_handle, &status);
        }
    }) {
        Ok(watcher) => {
            state.set_watcher(watcher);
        }
        Err(e) => {
            // Watcher failure is not fatal; log and continue.
            eprintln!("Warning: failed to start file watcher: {e}");
        }
    }

    Ok(resp)
}
