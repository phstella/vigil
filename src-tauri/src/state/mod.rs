//! Managed application state for Tauri.
//!
//! `AppState` holds the current workspace filesystem service and file index
//! behind read-write locks so Tauri commands can access them safely from any
//! thread.

use std::sync::{Arc, RwLock};

use crate::core::fs::WorkspaceFs;
use crate::core::index::{FileIndex, FileWatcher};
use crate::core::links::LinkGraph;

/// Application state managed by Tauri's `State<>` extractor.
///
/// Commands receive `State<'_, AppState>` and use `workspace()` /
/// `set_workspace()` to access the current workspace.
#[derive(Debug, Default)]
pub struct AppState {
    workspace: Arc<RwLock<Option<WorkspaceFs>>>,
    index: Arc<RwLock<Option<FileIndex>>>,
    /// Bidirectional link graph built from the file index.
    link_graph: Arc<RwLock<Option<LinkGraph>>>,
    /// The file watcher is not `Debug`, so we wrap it separately.
    /// It is held alive as long as a workspace is open.
    watcher: Arc<RwLock<Option<WatcherHolder>>>,
}

/// Wrapper to hold the `FileWatcher` which does not implement `Debug`.
struct WatcherHolder {
    _watcher: FileWatcher,
}

impl std::fmt::Debug for WatcherHolder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WatcherHolder").finish()
    }
}

impl AppState {
    /// Create a new empty `AppState` (no workspace open).
    pub fn new() -> Self {
        Self {
            workspace: Arc::new(RwLock::new(None)),
            index: Arc::new(RwLock::new(None)),
            link_graph: Arc::new(RwLock::new(None)),
            watcher: Arc::new(RwLock::new(None)),
        }
    }

    /// Get a clone of the current workspace filesystem service.
    ///
    /// Returns `None` if no workspace is open.
    pub fn workspace(&self) -> Option<WorkspaceFs> {
        self.workspace
            .read()
            .expect("workspace lock poisoned")
            .clone()
    }

    /// Set (or replace) the current workspace.
    pub fn set_workspace(&self, ws: WorkspaceFs) {
        let mut guard = self.workspace.write().expect("workspace lock poisoned");
        *guard = Some(ws);
    }

    /// Get a clone of the current file index.
    ///
    /// Returns `None` if no workspace is open or index not yet built.
    pub fn index(&self) -> Option<FileIndex> {
        self.index
            .read()
            .expect("index lock poisoned")
            .clone()
    }

    /// Set (or replace) the current file index.
    pub fn set_index(&self, index: FileIndex) {
        let mut guard = self.index.write().expect("index lock poisoned");
        *guard = Some(index);
    }

    /// Get a clone of the current link graph.
    ///
    /// Returns `None` if no workspace is open or the graph has not been built.
    pub fn link_graph(&self) -> Option<LinkGraph> {
        self.link_graph
            .read()
            .expect("link_graph lock poisoned")
            .clone()
    }

    /// Set (or replace) the current link graph.
    pub fn set_link_graph(&self, graph: LinkGraph) {
        let mut guard = self.link_graph.write().expect("link_graph lock poisoned");
        *guard = Some(graph);
    }

    /// Set the file watcher for the current workspace.
    ///
    /// The previous watcher (if any) is dropped, which signals it to stop.
    pub fn set_watcher(&self, watcher: FileWatcher) {
        let mut guard = self.watcher.write().expect("watcher lock poisoned");
        *guard = Some(WatcherHolder { _watcher: watcher });
    }

    /// Stop and remove the current file watcher.
    pub fn clear_watcher(&self) {
        let mut guard = self.watcher.write().expect("watcher lock poisoned");
        *guard = None;
    }

    /// Clear all workspace state (workspace, index, watcher).
    ///
    /// Called when switching workspaces.
    pub fn clear_all(&self) {
        self.clear_watcher();
        {
            let mut guard = self.link_graph.write().expect("link_graph lock poisoned");
            *guard = None;
        }
        {
            let mut guard = self.index.write().expect("index lock poisoned");
            *guard = None;
        }
        {
            let mut guard = self.workspace.write().expect("workspace lock poisoned");
            *guard = None;
        }
    }
}
