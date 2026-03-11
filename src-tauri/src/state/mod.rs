//! Managed application state for Tauri.
//!
//! `AppState` holds the current workspace filesystem service behind a
//! read-write lock so Tauri commands can access it safely from any thread.

use std::sync::{Arc, RwLock};

use crate::core::fs::WorkspaceFs;

/// Application state managed by Tauri's `State<>` extractor.
///
/// Commands receive `State<'_, AppState>` and use `workspace()` /
/// `set_workspace()` to access the current workspace.
#[derive(Debug, Default)]
pub struct AppState {
    workspace: Arc<RwLock<Option<WorkspaceFs>>>,
}

impl AppState {
    /// Create a new empty `AppState` (no workspace open).
    pub fn new() -> Self {
        Self {
            workspace: Arc::new(RwLock::new(None)),
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
}
