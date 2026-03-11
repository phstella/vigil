//! File read/write/rename/delete command wrappers.

use tauri::State;

use crate::models::error::VigilError;
use crate::models::files::{
    CreateNoteRequest, CreateNoteResponse, DeleteFileResponse, ListDirResponse, ReadFileResponse,
    RenameFileResponse, WriteFileRequest, WriteFileResponse,
};
use crate::state::AppState;

/// Helper to get the workspace or return `WorkspaceNotOpen`.
fn require_workspace(state: &AppState) -> Result<crate::core::fs::WorkspaceFs, VigilError> {
    state.workspace().ok_or(VigilError::WorkspaceNotOpen)
}

/// List entries in a workspace directory.
///
/// Pass an empty string for the workspace root.
#[tauri::command]
pub async fn list_dir(
    path: String,
    state: State<'_, AppState>,
) -> Result<ListDirResponse, VigilError> {
    let ws = require_workspace(&state)?;
    ws.list_dir(&path)
}

/// Read a text file's content and metadata.
#[tauri::command]
pub async fn read_file(
    path: String,
    state: State<'_, AppState>,
) -> Result<ReadFileResponse, VigilError> {
    let ws = require_workspace(&state)?;
    ws.read_file(&path)
}

/// Write content to a file, optionally with optimistic concurrency check.
#[tauri::command]
pub async fn write_file(
    request: WriteFileRequest,
    state: State<'_, AppState>,
) -> Result<WriteFileResponse, VigilError> {
    let ws = require_workspace(&state)?;
    ws.write_file(&request)
}

/// Create a new markdown note file.
#[tauri::command]
pub async fn create_note(
    request: CreateNoteRequest,
    state: State<'_, AppState>,
) -> Result<CreateNoteResponse, VigilError> {
    let ws = require_workspace(&state)?;
    ws.create_note(&request)
}

/// Rename or move a file within the workspace.
#[tauri::command]
pub async fn rename_file(
    old_path: String,
    new_path: String,
    state: State<'_, AppState>,
) -> Result<RenameFileResponse, VigilError> {
    let ws = require_workspace(&state)?;
    ws.rename_file(&old_path, &new_path)
}

/// Delete a file or empty directory within the workspace.
#[tauri::command]
pub async fn delete_file(
    path: String,
    state: State<'_, AppState>,
) -> Result<DeleteFileResponse, VigilError> {
    let ws = require_workspace(&state)?;
    ws.delete_file(&path)
}
