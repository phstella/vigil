//! Integration tests for the workspace filesystem service.
//!
//! These tests exercise `WorkspaceFs` directly (not via Tauri IPC) to validate
//! path confinement, CRUD operations, and error handling.

use std::fs;

use vigil_lib::core::fs::WorkspaceFs;
use vigil_lib::models::error::VigilError;
use vigil_lib::models::files::{CreateNoteRequest, EntryKind, WriteFileRequest};

/// Create a temporary workspace directory.
fn temp_workspace() -> tempfile::TempDir {
    tempfile::tempdir().expect("failed to create temp dir")
}

// ---------------------------------------------------------------------------
// open_workspace
// ---------------------------------------------------------------------------

#[test]
fn open_workspace_valid_directory() {
    let dir = temp_workspace();
    let (ws, resp) = WorkspaceFs::open(dir.path().to_str().unwrap()).unwrap();
    assert!(!resp.workspace_id.is_empty());
    assert!(!resp.canonical_path.is_empty());
    assert!(resp.opened_at_ms > 0);
    assert_eq!(ws.root(), fs::canonicalize(dir.path()).unwrap());
}

#[test]
fn open_workspace_rejects_empty_path() {
    let result = WorkspaceFs::open("");
    assert!(result.is_err());
    match result.unwrap_err() {
        VigilError::InvalidArgument { .. } => {}
        other => panic!("expected InvalidArgument, got: {other:?}"),
    }
}

#[test]
fn open_workspace_rejects_nonexistent_path() {
    let result = WorkspaceFs::open("/tmp/vigil_test_nonexistent_12345");
    assert!(result.is_err());
}

#[test]
fn open_workspace_rejects_file_as_root() {
    let dir = temp_workspace();
    let file = dir.path().join("not_a_dir.txt");
    fs::write(&file, "hello").unwrap();
    let result = WorkspaceFs::open(file.to_str().unwrap());
    assert!(result.is_err());
    match result.unwrap_err() {
        VigilError::InvalidArgument { .. } => {}
        other => panic!("expected InvalidArgument, got: {other:?}"),
    }
}

#[test]
fn open_workspace_counts_files_and_notes() {
    let dir = temp_workspace();
    fs::write(dir.path().join("a.md"), "").unwrap();
    fs::write(dir.path().join("b.md"), "").unwrap();
    fs::write(dir.path().join("c.txt"), "").unwrap();
    fs::create_dir(dir.path().join("sub")).unwrap();
    fs::write(dir.path().join("sub/d.md"), "").unwrap();

    let (_, resp) = WorkspaceFs::open(dir.path().to_str().unwrap()).unwrap();
    assert_eq!(resp.files_count, 4);
    assert_eq!(resp.notes_count, 3);
}

// ---------------------------------------------------------------------------
// Path traversal
// ---------------------------------------------------------------------------

#[test]
fn path_traversal_with_double_dots_rejected() {
    let dir = temp_workspace();
    let (ws, _) = WorkspaceFs::open(dir.path().to_str().unwrap()).unwrap();

    let paths = ["../escape", "sub/../../escape", "../../etc/passwd"];
    for p in &paths {
        let result = ws.read_file(p);
        assert!(result.is_err(), "expected error for path: {p}");
        match result.unwrap_err() {
            VigilError::PathOutsideWorkspace { .. } => {}
            other => panic!("expected PathOutsideWorkspace for {p}, got: {other:?}"),
        }
    }
}

// ---------------------------------------------------------------------------
// list_dir
// ---------------------------------------------------------------------------

#[test]
fn list_dir_empty_directory() {
    let dir = temp_workspace();
    let (ws, _) = WorkspaceFs::open(dir.path().to_str().unwrap()).unwrap();
    let resp = ws.list_dir("").unwrap();
    assert!(resp.entries.is_empty());
    assert!(!resp.truncated);
}

#[test]
fn list_dir_with_files_and_dirs() {
    let dir = temp_workspace();
    fs::write(dir.path().join("b.md"), "").unwrap();
    fs::write(dir.path().join("a.md"), "").unwrap();
    fs::create_dir(dir.path().join("z_dir")).unwrap();
    fs::create_dir(dir.path().join("a_dir")).unwrap();

    let (ws, _) = WorkspaceFs::open(dir.path().to_str().unwrap()).unwrap();
    let resp = ws.list_dir("").unwrap();

    assert_eq!(resp.entries.len(), 4);

    // Directories first, sorted alphabetically.
    assert_eq!(resp.entries[0].name, "a_dir");
    assert_eq!(resp.entries[0].kind, EntryKind::Dir);
    assert_eq!(resp.entries[1].name, "z_dir");
    assert_eq!(resp.entries[1].kind, EntryKind::Dir);

    // Files sorted alphabetically.
    assert_eq!(resp.entries[2].name, "a.md");
    assert_eq!(resp.entries[2].kind, EntryKind::File);
    assert_eq!(resp.entries[3].name, "b.md");
    assert_eq!(resp.entries[3].kind, EntryKind::File);
}

#[test]
fn list_dir_filters_dot_git() {
    let dir = temp_workspace();
    fs::create_dir(dir.path().join(".git")).unwrap();
    fs::write(dir.path().join("readme.md"), "").unwrap();

    let (ws, _) = WorkspaceFs::open(dir.path().to_str().unwrap()).unwrap();
    let resp = ws.list_dir("").unwrap();
    assert_eq!(resp.entries.len(), 1);
    assert_eq!(resp.entries[0].name, "readme.md");
}

#[test]
fn list_dir_subdirectory() {
    let dir = temp_workspace();
    fs::create_dir(dir.path().join("notes")).unwrap();
    fs::write(dir.path().join("notes/a.md"), "").unwrap();
    fs::write(dir.path().join("notes/b.md"), "").unwrap();

    let (ws, _) = WorkspaceFs::open(dir.path().to_str().unwrap()).unwrap();
    let resp = ws.list_dir("notes").unwrap();
    assert_eq!(resp.entries.len(), 2);
}

#[test]
fn list_dir_marks_hidden_files() {
    let dir = temp_workspace();
    fs::write(dir.path().join(".hidden"), "").unwrap();
    fs::write(dir.path().join("visible.md"), "").unwrap();

    let (ws, _) = WorkspaceFs::open(dir.path().to_str().unwrap()).unwrap();
    let resp = ws.list_dir("").unwrap();

    let hidden = resp.entries.iter().find(|e| e.name == ".hidden").unwrap();
    assert!(hidden.is_hidden);

    let visible = resp
        .entries
        .iter()
        .find(|e| e.name == "visible.md")
        .unwrap();
    assert!(!visible.is_hidden);
}

#[test]
fn list_dir_nonexistent_returns_error() {
    let dir = temp_workspace();
    let (ws, _) = WorkspaceFs::open(dir.path().to_str().unwrap()).unwrap();
    let result = ws.list_dir("does_not_exist");
    assert!(result.is_err());
}

// ---------------------------------------------------------------------------
// read_file
// ---------------------------------------------------------------------------

#[test]
fn read_file_returns_content() {
    let dir = temp_workspace();
    fs::write(dir.path().join("test.md"), "# Hello World").unwrap();

    let (ws, _) = WorkspaceFs::open(dir.path().to_str().unwrap()).unwrap();
    let resp = ws.read_file("test.md").unwrap();
    assert_eq!(resp.content, "# Hello World");
    assert_eq!(resp.encoding, "utf-8");
    assert_eq!(resp.size_bytes, 13);
    assert!(!resp.etag.is_empty());
    assert!(resp.modified_at_ms > 0);
}

#[test]
fn read_file_empty_path_rejected() {
    let dir = temp_workspace();
    let (ws, _) = WorkspaceFs::open(dir.path().to_str().unwrap()).unwrap();
    let result = ws.read_file("");
    assert!(result.is_err());
    match result.unwrap_err() {
        VigilError::InvalidArgument { .. } => {}
        other => panic!("expected InvalidArgument, got: {other:?}"),
    }
}

#[test]
fn read_file_nonexistent_returns_not_found() {
    let dir = temp_workspace();
    let (ws, _) = WorkspaceFs::open(dir.path().to_str().unwrap()).unwrap();
    let result = ws.read_file("nope.md");
    assert!(result.is_err());
    match result.unwrap_err() {
        VigilError::FileNotFound { .. } => {}
        other => panic!("expected FileNotFound, got: {other:?}"),
    }
}

#[test]
fn read_file_binary_rejected() {
    let dir = temp_workspace();
    let content = vec![0u8; 100];
    fs::write(dir.path().join("binary.bin"), &content).unwrap();

    let (ws, _) = WorkspaceFs::open(dir.path().to_str().unwrap()).unwrap();
    let result = ws.read_file("binary.bin");
    assert!(result.is_err());
    match result.unwrap_err() {
        VigilError::BinaryFile { .. } => {}
        other => panic!("expected BinaryFile, got: {other:?}"),
    }
}

#[test]
fn read_file_etag_is_deterministic() {
    let dir = temp_workspace();
    fs::write(dir.path().join("test.md"), "deterministic").unwrap();

    let (ws, _) = WorkspaceFs::open(dir.path().to_str().unwrap()).unwrap();
    let resp1 = ws.read_file("test.md").unwrap();
    let resp2 = ws.read_file("test.md").unwrap();
    assert_eq!(resp1.etag, resp2.etag);
}

// ---------------------------------------------------------------------------
// write_file
// ---------------------------------------------------------------------------

#[test]
fn write_file_creates_new_file() {
    let dir = temp_workspace();
    let (ws, _) = WorkspaceFs::open(dir.path().to_str().unwrap()).unwrap();

    let req = WriteFileRequest {
        path: "new.md".into(),
        content: "fresh content".into(),
        etag: None,
    };
    let resp = ws.write_file(&req).unwrap();
    assert_eq!(resp.size_bytes, 13);
    assert!(!resp.etag.is_empty());

    let on_disk = fs::read_to_string(dir.path().join("new.md")).unwrap();
    assert_eq!(on_disk, "fresh content");
}

#[test]
fn write_file_overwrites_existing() {
    let dir = temp_workspace();
    fs::write(dir.path().join("existing.md"), "old").unwrap();

    let (ws, _) = WorkspaceFs::open(dir.path().to_str().unwrap()).unwrap();
    let req = WriteFileRequest {
        path: "existing.md".into(),
        content: "new".into(),
        etag: None,
    };
    ws.write_file(&req).unwrap();
    assert_eq!(
        fs::read_to_string(dir.path().join("existing.md")).unwrap(),
        "new"
    );
}

#[test]
fn write_file_stale_etag_rejected() {
    let dir = temp_workspace();
    fs::write(dir.path().join("test.md"), "original").unwrap();

    let (ws, _) = WorkspaceFs::open(dir.path().to_str().unwrap()).unwrap();
    let req = WriteFileRequest {
        path: "test.md".into(),
        content: "new".into(),
        etag: Some("bad_etag".into()),
    };
    let result = ws.write_file(&req);
    assert!(result.is_err());
    match result.unwrap_err() {
        VigilError::StaleEtag => {}
        other => panic!("expected StaleEtag, got: {other:?}"),
    }

    // File should be unchanged.
    assert_eq!(
        fs::read_to_string(dir.path().join("test.md")).unwrap(),
        "original"
    );
}

#[test]
fn write_file_correct_etag_succeeds() {
    let dir = temp_workspace();
    fs::write(dir.path().join("test.md"), "original").unwrap();

    let (ws, _) = WorkspaceFs::open(dir.path().to_str().unwrap()).unwrap();
    let etag = ws.read_file("test.md").unwrap().etag;

    let req = WriteFileRequest {
        path: "test.md".into(),
        content: "updated".into(),
        etag: Some(etag),
    };
    let resp = ws.write_file(&req).unwrap();
    assert!(!resp.etag.is_empty());
    assert_eq!(
        fs::read_to_string(dir.path().join("test.md")).unwrap(),
        "updated"
    );
}

// ---------------------------------------------------------------------------
// create_note
// ---------------------------------------------------------------------------

#[test]
fn create_note_basic() {
    let dir = temp_workspace();
    let (ws, _) = WorkspaceFs::open(dir.path().to_str().unwrap()).unwrap();

    let req = CreateNoteRequest {
        path: "journal/2024-01".into(),
    };
    let resp = ws.create_note(&req).unwrap();
    assert_eq!(resp.path, "journal/2024-01.md");
    assert_eq!(resp.size_bytes, 0);
    assert!(!resp.etag.is_empty());
    assert!(dir.path().join("journal/2024-01.md").exists());
}

#[test]
fn create_note_preserves_extension() {
    let dir = temp_workspace();
    let (ws, _) = WorkspaceFs::open(dir.path().to_str().unwrap()).unwrap();

    let req = CreateNoteRequest {
        path: "notes.txt".into(),
    };
    let resp = ws.create_note(&req).unwrap();
    assert_eq!(resp.path, "notes.txt");
}

#[test]
fn create_note_rejects_existing_file() {
    let dir = temp_workspace();
    fs::write(dir.path().join("exists.md"), "hi").unwrap();

    let (ws, _) = WorkspaceFs::open(dir.path().to_str().unwrap()).unwrap();
    let req = CreateNoteRequest {
        path: "exists.md".into(),
    };
    let result = ws.create_note(&req);
    assert!(result.is_err());
    match result.unwrap_err() {
        VigilError::FileAlreadyExists { .. } => {}
        other => panic!("expected FileAlreadyExists, got: {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// rename_file
// ---------------------------------------------------------------------------

#[test]
fn rename_file_basic() {
    let dir = temp_workspace();
    fs::write(dir.path().join("old.md"), "content").unwrap();

    let (ws, _) = WorkspaceFs::open(dir.path().to_str().unwrap()).unwrap();
    let resp = ws.rename_file("old.md", "new.md").unwrap();
    assert_eq!(resp.path, "new.md");
    assert!(!dir.path().join("old.md").exists());
    assert!(dir.path().join("new.md").exists());
    assert_eq!(
        fs::read_to_string(dir.path().join("new.md")).unwrap(),
        "content"
    );
}

#[test]
fn rename_file_to_subdirectory() {
    let dir = temp_workspace();
    fs::write(dir.path().join("note.md"), "content").unwrap();

    let (ws, _) = WorkspaceFs::open(dir.path().to_str().unwrap()).unwrap();
    let resp = ws.rename_file("note.md", "archive/note.md").unwrap();
    assert_eq!(resp.path, "archive/note.md");
    assert!(dir.path().join("archive/note.md").exists());
}

#[test]
fn rename_file_rejects_nonexistent_source() {
    let dir = temp_workspace();
    let (ws, _) = WorkspaceFs::open(dir.path().to_str().unwrap()).unwrap();
    let result = ws.rename_file("ghost.md", "new.md");
    assert!(result.is_err());
    match result.unwrap_err() {
        VigilError::FileNotFound { .. } => {}
        other => panic!("expected FileNotFound, got: {other:?}"),
    }
}

#[test]
fn rename_file_rejects_existing_target() {
    let dir = temp_workspace();
    fs::write(dir.path().join("a.md"), "a").unwrap();
    fs::write(dir.path().join("b.md"), "b").unwrap();

    let (ws, _) = WorkspaceFs::open(dir.path().to_str().unwrap()).unwrap();
    let result = ws.rename_file("a.md", "b.md");
    assert!(result.is_err());
    match result.unwrap_err() {
        VigilError::FileAlreadyExists { .. } => {}
        other => panic!("expected FileAlreadyExists, got: {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// delete_file
// ---------------------------------------------------------------------------

#[test]
fn delete_file_removes_file() {
    let dir = temp_workspace();
    fs::write(dir.path().join("doomed.md"), "bye").unwrap();

    let (ws, _) = WorkspaceFs::open(dir.path().to_str().unwrap()).unwrap();
    let resp = ws.delete_file("doomed.md").unwrap();
    assert_eq!(resp.path, "doomed.md");
    assert!(!dir.path().join("doomed.md").exists());
}

#[test]
fn delete_file_removes_empty_dir() {
    let dir = temp_workspace();
    fs::create_dir(dir.path().join("empty")).unwrap();

    let (ws, _) = WorkspaceFs::open(dir.path().to_str().unwrap()).unwrap();
    let resp = ws.delete_file("empty").unwrap();
    assert_eq!(resp.path, "empty");
    assert!(!dir.path().join("empty").exists());
}

#[test]
fn delete_file_rejects_nonempty_dir() {
    let dir = temp_workspace();
    fs::create_dir(dir.path().join("full")).unwrap();
    fs::write(dir.path().join("full/child.md"), "").unwrap();

    let (ws, _) = WorkspaceFs::open(dir.path().to_str().unwrap()).unwrap();
    let result = ws.delete_file("full");
    assert!(result.is_err());
    match result.unwrap_err() {
        VigilError::InvalidArgument { .. } => {}
        other => panic!("expected InvalidArgument, got: {other:?}"),
    }
}

#[test]
fn delete_file_rejects_nonexistent() {
    let dir = temp_workspace();
    let (ws, _) = WorkspaceFs::open(dir.path().to_str().unwrap()).unwrap();
    let result = ws.delete_file("ghost.md");
    assert!(result.is_err());
    match result.unwrap_err() {
        VigilError::FileNotFound { .. } => {}
        other => panic!("expected FileNotFound, got: {other:?}"),
    }
}

#[test]
fn delete_file_path_traversal_rejected() {
    let dir = temp_workspace();
    let (ws, _) = WorkspaceFs::open(dir.path().to_str().unwrap()).unwrap();
    let result = ws.delete_file("../escape");
    assert!(result.is_err());
    match result.unwrap_err() {
        VigilError::PathOutsideWorkspace { .. } => {}
        other => panic!("expected PathOutsideWorkspace, got: {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// State management (AppState)
// ---------------------------------------------------------------------------

#[test]
fn app_state_no_workspace_initially() {
    let state = vigil_lib::state::AppState::new();
    assert!(state.workspace().is_none());
}

#[test]
fn app_state_set_and_get_workspace() {
    let dir = temp_workspace();
    let (ws, _) = WorkspaceFs::open(dir.path().to_str().unwrap()).unwrap();
    let state = vigil_lib::state::AppState::new();

    state.set_workspace(ws.clone());
    let retrieved = state.workspace().unwrap();
    assert_eq!(retrieved.workspace_id(), ws.workspace_id());
}
