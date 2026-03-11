//! Workspace open command wrapper.

use tauri::{AppHandle, State};

use crate::core::fs::WorkspaceFs;
use crate::core::index::{FileIndex, FileWatcher, TagIndex};
use crate::core::links::LinkGraph;
use crate::events::index_events;
use crate::models::error::VigilError;
use crate::models::workspace::OpenWorkspaceResponse;
use crate::state::AppState;

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
    let watcher_index = index;
    let watcher_app_handle = app_handle.clone();

    match FileWatcher::start(&root, move |events| {
        // Group events by change kind for batch processing.
        use crate::core::index::ChangeKind;
        use std::collections::HashMap;

        let mut by_kind: HashMap<ChangeKind, Vec<std::path::PathBuf>> = HashMap::new();
        for event in events {
            by_kind.entry(event.kind).or_default().push(event.path);
        }

        let mut all_changes = Vec::new();
        for (kind, paths) in by_kind {
            let changes = watcher_index.handle_event(&paths, kind);
            all_changes.extend(changes);
        }

        if !all_changes.is_empty() {
            index_events::emit_index_updated(&watcher_app_handle, &all_changes);
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
