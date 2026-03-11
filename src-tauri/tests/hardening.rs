//! Epic 1.11 hardening tests.
//!
//! Validates the behavioral gaps closed in the hardening pass:
//! - workspace_status errors when no workspace is open
//! - last_index_update_ms tracks real index refresh time
//! - Tag/link/status coherence after file changes without workspace reopen
//! - Stale note metadata removed when file stops being valid markdown
//! - Git hunk event flow for modified files

use std::fs;
use std::path::Path;
use std::sync::atomic::Ordering;

use vigil_lib::core::fs::WorkspaceFs;
use vigil_lib::core::git::GitService;
use vigil_lib::core::index::{ChangeKind, FileIndex, TagIndex};
use vigil_lib::core::links::LinkGraph;
use vigil_lib::models::error::{ErrorEnvelope, VigilError};
use vigil_lib::models::files::WriteFileRequest;
use vigil_lib::models::links::BacklinksResponse;
use vigil_lib::state::AppState;

fn temp_workspace() -> tempfile::TempDir {
    tempfile::tempdir().expect("failed to create temp dir")
}

// ---------------------------------------------------------------------------
// Gap 3: workspace_status must return WORKSPACE_NOT_OPEN when no workspace
// ---------------------------------------------------------------------------
// These tests call extracted command-logic helpers directly (not Tauri runtime
// integration) so they stay fast while validating command boundary behavior.

#[test]
fn workspace_status_command_logic_requires_open_workspace() {
    let state = AppState::new();
    let result = vigil_lib::commands::status::workspace_status_for_state(&state);
    assert!(matches!(result, Err(VigilError::WorkspaceNotOpen)));
}

#[test]
fn get_backlinks_command_logic_checks_workspace_before_index() {
    let state = AppState::new();
    // Ensure the link graph exists; missing workspace must still take precedence.
    state.set_link_graph(LinkGraph::new());
    let result = vigil_lib::commands::links::get_backlinks_for_state("target.md", &state);
    assert!(matches!(result, Err(VigilError::WorkspaceNotOpen)));
}

// ---------------------------------------------------------------------------
// Gap 4: last_index_update_ms tracks real index time
// ---------------------------------------------------------------------------

#[test]
fn last_index_update_ms_starts_at_zero() {
    let state = AppState::new();
    assert_eq!(state.last_index_update_ms(), 0);
}

#[test]
fn last_index_update_ms_updates_on_set() {
    let state = AppState::new();
    let ts = 1_700_000_000_000i64;
    state.set_last_index_update_ms(ts);
    assert_eq!(state.last_index_update_ms(), ts);
}

#[test]
fn last_index_update_ms_clears_on_clear_all() {
    let state = AppState::new();
    state.set_last_index_update_ms(1_700_000_000_000);
    state.clear_all();
    assert_eq!(state.last_index_update_ms(), 0);
}

#[test]
fn last_index_update_arc_shares_same_value() {
    let state = AppState::new();
    let arc = state.last_index_update_arc();

    state.set_last_index_update_ms(42);
    assert_eq!(arc.load(Ordering::Acquire), 42);

    arc.store(99, Ordering::Release);
    assert_eq!(state.last_index_update_ms(), 99);
}

// ---------------------------------------------------------------------------
// Gap 1: Tag index coherence after incremental file changes
// ---------------------------------------------------------------------------

#[test]
fn tags_correct_after_write_without_reopen() {
    let dir = temp_workspace();
    fs::write(
        dir.path().join("a.md"),
        "---\ntags: [alpha, beta]\n---\n\nText.\n",
    )
    .unwrap();

    let index = FileIndex::new(dir.path().to_path_buf());
    index.full_scan();

    let tag_index = TagIndex::new();
    tag_index.rebuild(&index);

    assert_eq!(tag_index.get_files_by_tag("alpha").len(), 1);
    assert_eq!(tag_index.get_files_by_tag("beta").len(), 1);

    // Modify the file: remove "alpha", add "gamma"
    fs::write(
        dir.path().join("a.md"),
        "---\ntags: [gamma, beta]\n---\n\nUpdated.\n",
    )
    .unwrap();

    let path = dir.path().join("a.md");
    index.handle_event(&[path], ChangeKind::Changed);
    tag_index.rebuild(&index);

    // alpha should be gone, gamma should appear
    assert!(
        tag_index.get_files_by_tag("alpha").is_empty(),
        "alpha should be removed after edit"
    );
    assert_eq!(
        tag_index.get_files_by_tag("gamma").len(),
        1,
        "gamma should appear after edit"
    );
    assert_eq!(
        tag_index.get_files_by_tag("beta").len(),
        1,
        "beta should persist"
    );
}

#[test]
fn tags_correct_after_delete_without_reopen() {
    let dir = temp_workspace();
    fs::write(
        dir.path().join("a.md"),
        "---\ntags: [only-here]\n---\n\nText.\n",
    )
    .unwrap();
    fs::write(dir.path().join("b.md"), "# B\n").unwrap();

    let index = FileIndex::new(dir.path().to_path_buf());
    index.full_scan();

    let tag_index = TagIndex::new();
    tag_index.rebuild(&index);

    assert_eq!(tag_index.get_files_by_tag("only-here").len(), 1);

    // Delete the file
    let path = dir.path().join("a.md");
    fs::remove_file(&path).unwrap();
    index.handle_event(&[path], ChangeKind::Deleted);
    tag_index.rebuild(&index);

    assert!(
        tag_index.get_files_by_tag("only-here").is_empty(),
        "tag from deleted file should be gone"
    );
}

// ---------------------------------------------------------------------------
// Gap 1: Link graph coherence after incremental changes
// ---------------------------------------------------------------------------

#[test]
fn backlinks_correct_after_write_without_reopen() {
    let dir = temp_workspace();
    fs::write(dir.path().join("a.md"), "# A\n\nLinks to [[b]].\n").unwrap();
    fs::write(dir.path().join("b.md"), "# B\n").unwrap();

    let index = FileIndex::new(dir.path().to_path_buf());
    index.full_scan();

    let graph = LinkGraph::new();
    graph.rebuild(&index);

    assert_eq!(graph.get_backlinks("b.md").len(), 1);

    // Modify a.md to remove the link to b and add link to c
    fs::write(dir.path().join("a.md"), "# A\n\nNow links to [[c]].\n").unwrap();

    let path = dir.path().join("a.md");
    index.handle_event(&[path], ChangeKind::Changed);
    graph.rebuild(&index);

    assert!(
        graph.get_backlinks("b.md").is_empty(),
        "backlink to b.md should be gone after edit"
    );
    assert_eq!(
        graph.get_backlinks("c.md").len(),
        1,
        "backlink to c.md should appear after edit"
    );
}

#[test]
fn backlinks_correct_after_delete_without_reopen() {
    let dir = temp_workspace();
    fs::write(
        dir.path().join("source.md"),
        "# Source\n\nSee [[target]].\n",
    )
    .unwrap();
    fs::write(dir.path().join("target.md"), "# Target\n").unwrap();

    let index = FileIndex::new(dir.path().to_path_buf());
    index.full_scan();

    let graph = LinkGraph::new();
    graph.rebuild(&index);

    assert_eq!(graph.get_backlinks("target.md").len(), 1);

    // Delete source.md
    let path = dir.path().join("source.md");
    fs::remove_file(&path).unwrap();
    index.handle_event(&[path], ChangeKind::Deleted);
    graph.rebuild(&index);

    assert!(
        graph.get_backlinks("target.md").is_empty(),
        "backlink from deleted source should be gone"
    );
}

#[test]
fn backlinks_correct_after_rename_without_reopen() {
    let dir = temp_workspace();

    let (ws, _) = WorkspaceFs::open(dir.path().to_str().unwrap()).unwrap();
    fs::write(dir.path().join("a.md"), "# A\n\nLinks to [[b]].\n").unwrap();
    fs::write(dir.path().join("b.md"), "# B\n").unwrap();

    let index = FileIndex::new(dir.path().to_path_buf());
    index.full_scan();

    let graph = LinkGraph::new();
    graph.rebuild(&index);

    assert_eq!(graph.get_backlinks("b.md").len(), 1);
    assert_eq!(graph.get_backlinks("b.md")[0].source_path, "a.md");

    // Rename a.md to renamed.md
    ws.rename_file("a.md", "renamed.md").unwrap();
    let old_path = dir.path().join("a.md");
    let new_path = dir.path().join("renamed.md");
    index.handle_event(&[old_path], ChangeKind::Deleted);
    index.handle_event(&[new_path], ChangeKind::Created);
    graph.rebuild(&index);

    // b.md should now have a backlink from renamed.md, not a.md
    let backlinks = graph.get_backlinks("b.md");
    assert_eq!(
        backlinks.len(),
        1,
        "should still have 1 backlink after rename"
    );
    assert_eq!(
        backlinks[0].source_path, "renamed.md",
        "backlink source should be updated to renamed.md"
    );
}

// ---------------------------------------------------------------------------
// Gap 6: Stale note metadata removed when file stops being valid markdown
// ---------------------------------------------------------------------------

#[test]
fn stale_note_removed_when_md_becomes_binary() {
    let dir = temp_workspace();
    fs::write(dir.path().join("note.md"), "# Hello\n\nText.\n").unwrap();

    let index = FileIndex::new(dir.path().to_path_buf());
    index.full_scan();

    assert!(index.get_note_metadata("note.md").is_some());

    // Overwrite with binary content (null bytes)
    let binary_content: Vec<u8> = vec![0u8; 100];
    fs::write(dir.path().join("note.md"), &binary_content).unwrap();

    let path = dir.path().join("note.md");
    index.handle_event(&[path], ChangeKind::Changed);

    assert!(
        index.get_note_metadata("note.md").is_none(),
        "note metadata should be removed when file becomes binary"
    );
    // File entry should still exist
    assert!(index.get_file_entry("note.md").is_some());
}

#[test]
fn stale_note_removed_when_extension_changes_to_non_md() {
    let dir = temp_workspace();
    fs::write(dir.path().join("note.md"), "# Hello\n\nText.\n").unwrap();

    let index = FileIndex::new(dir.path().to_path_buf());
    index.full_scan();

    let tag_index = TagIndex::new();
    tag_index.rebuild(&index);

    assert!(index.get_note_metadata("note.md").is_some());

    // Simulate rename: delete old, create new with .txt extension
    let old_path = dir.path().join("note.md");
    fs::remove_file(&old_path).unwrap();
    index.handle_event(&[old_path], ChangeKind::Deleted);

    fs::write(dir.path().join("note.txt"), "# Hello\n\nText.\n").unwrap();
    let new_path = dir.path().join("note.txt");
    index.handle_event(&[new_path], ChangeKind::Created);

    assert!(
        index.get_note_metadata("note.md").is_none(),
        "old .md note metadata should be gone"
    );
    assert!(
        index.get_note_metadata("note.txt").is_none(),
        ".txt file should not have note metadata"
    );
    assert!(
        index.get_file_entry("note.txt").is_some(),
        "file entry for .txt should exist"
    );

    // Tag index should reflect the change
    tag_index.rebuild(&index);
    // No notes remain, so no tags
    assert!(tag_index.get_all_tags().is_empty());
}

// ---------------------------------------------------------------------------
// Gap 2: Git hunk service works correctly for non-git workspaces
// ---------------------------------------------------------------------------

#[test]
fn git_hunks_empty_for_non_git_workspace() {
    let dir = temp_workspace();
    fs::write(dir.path().join("note.md"), "# Note\n").unwrap();

    let git_svc = GitService::new(dir.path().to_path_buf());
    let resp = git_svc.get_hunks("note.md").unwrap();
    assert!(
        resp.hunks.is_empty(),
        "non-git workspace should return empty hunks without error"
    );
}

#[test]
fn git_hunks_for_modified_file_in_git_repo() {
    let dir = temp_workspace();
    let repo = git2::Repository::init(dir.path()).unwrap();
    let mut config = repo.config().unwrap();
    config.set_str("user.name", "Test").unwrap();
    config.set_str("user.email", "test@test.com").unwrap();

    fs::write(dir.path().join("note.md"), "# Note\n\nOriginal.\n").unwrap();
    stage_and_commit(&repo, "initial");

    // Modify the file
    fs::write(dir.path().join("note.md"), "# Note\n\nModified content.\n").unwrap();

    let git_svc = GitService::new(dir.path().to_path_buf());
    let resp = git_svc.get_hunks("note.md").unwrap();
    assert!(
        !resp.hunks.is_empty(),
        "should detect hunks for modified file"
    );
}

// ---------------------------------------------------------------------------
// Gap 5: Contract version in event payloads
// ---------------------------------------------------------------------------

#[test]
fn index_updated_payload_has_contract_version() {
    use vigil_lib::events::index_events::IndexUpdatedPayload;

    let payload = IndexUpdatedPayload {
        changes: vec![],
        timestamp_ms: 0,
        contract_version: "v1".to_string(),
    };
    let json = serde_json::to_string(&payload).unwrap();
    assert!(json.contains("\"contract_version\":\"v1\""));
}

#[test]
fn git_hunks_payload_has_contract_version() {
    use vigil_lib::events::git_events::GitHunksPayload;

    let payload = GitHunksPayload {
        path: "test.md".to_string(),
        hunks: vec![],
        timestamp_ms: 0,
        contract_version: "v1".to_string(),
    };
    let json = serde_json::to_string(&payload).unwrap();
    assert!(json.contains("\"contract_version\":\"v1\""));
}

#[test]
fn status_updated_payload_has_contract_version() {
    use vigil_lib::events::status_events::StatusUpdatedPayload;
    use vigil_lib::models::status::{SyncState, WorkspaceStatus};

    let status = WorkspaceStatus {
        branch: None,
        sync_state: SyncState::Unknown,
        notes_count: 0,
        tags_count: 0,
        files_count: 0,
        version: "0.0.1".into(),
        last_index_update_ms: 0,
    };
    let payload = StatusUpdatedPayload {
        status,
        timestamp_ms: 0,
        contract_version: "v1".to_string(),
    };
    let json = serde_json::to_string(&payload).unwrap();
    assert!(json.contains("\"contract_version\":\"v1\""));
}

#[test]
fn ipc_contract_doc_matches_typed_success_and_error_envelope_behavior() {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("src-tauri must be nested in repository root");
    let doc_path = repo_root.join("docs/specs/ipc-contracts.md");
    let doc = fs::read_to_string(doc_path).expect("failed to read IPC contract doc");

    // Contract wording should describe typed command success payloads directly.
    assert!(doc.contains("Every `#[tauri::command]` returns `Result<T, VigilError>`."));
    assert!(doc.contains("no backend-provided `ok/data` wrapper"));
    assert!(!doc.contains("{ \"ok\": true, \"data\": <T> }"));

    // Backend success payloads are typed models, not wrapped in an ok/data envelope.
    let success_json = serde_json::to_value(BacklinksResponse { backlinks: vec![] }).unwrap();
    assert!(success_json.get("backlinks").is_some());
    assert!(success_json.get("ok").is_none());
    assert!(success_json.get("data").is_none());

    // Backend errors are serialized ErrorEnvelope values.
    let err_json = serde_json::to_value(ErrorEnvelope::from(VigilError::WorkspaceNotOpen)).unwrap();
    assert_eq!(
        err_json.get("code").and_then(serde_json::Value::as_str),
        Some("WORKSPACE_NOT_OPEN")
    );
    assert!(err_json.get("message").is_some());
}

// ---------------------------------------------------------------------------
// Composite: full coherence after create -> modify -> delete cycle
// ---------------------------------------------------------------------------

#[test]
fn full_coherence_after_lifecycle() {
    let dir = temp_workspace();
    let (ws, _) = WorkspaceFs::open(dir.path().to_str().unwrap()).unwrap();

    fs::write(dir.path().join("base.md"), "# Base\n").unwrap();

    let index = FileIndex::new(dir.path().to_path_buf());
    index.full_scan();

    let tag_index = TagIndex::new();
    tag_index.rebuild(&index);

    let graph = LinkGraph::new();
    graph.rebuild(&index);

    // Step 1: Create a new note with tags and links
    let req = WriteFileRequest {
        path: "new.md".into(),
        content: "---\ntags: [test-tag]\n---\n\n# New\n\nLinks to [[base]].\n".into(),
        etag: None,
    };
    ws.write_file(&req).unwrap();
    let path = dir.path().join("new.md");
    index.handle_event(std::slice::from_ref(&path), ChangeKind::Created);
    tag_index.rebuild(&index);
    graph.rebuild(&index);

    assert_eq!(tag_index.get_files_by_tag("test-tag").len(), 1);
    assert_eq!(graph.get_backlinks("base.md").len(), 1);

    // Step 2: Modify the note -- change tag and link target
    let req = WriteFileRequest {
        path: "new.md".into(),
        content: "---\ntags: [updated-tag]\n---\n\n# New v2\n\nNow links to [[other]].\n".into(),
        etag: None,
    };
    ws.write_file(&req).unwrap();
    index.handle_event(std::slice::from_ref(&path), ChangeKind::Changed);
    tag_index.rebuild(&index);
    graph.rebuild(&index);

    assert!(tag_index.get_files_by_tag("test-tag").is_empty());
    assert_eq!(tag_index.get_files_by_tag("updated-tag").len(), 1);
    assert!(graph.get_backlinks("base.md").is_empty());
    assert_eq!(graph.get_backlinks("other.md").len(), 1);

    // Step 3: Delete the note
    ws.delete_file("new.md").unwrap();
    index.handle_event(&[path], ChangeKind::Deleted);
    tag_index.rebuild(&index);
    graph.rebuild(&index);

    assert!(tag_index.get_files_by_tag("updated-tag").is_empty());
    assert!(graph.get_backlinks("other.md").is_empty());
    assert!(index.get_note_metadata("new.md").is_none());
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn stage_and_commit(repo: &git2::Repository, message: &str) {
    let mut idx = repo.index().unwrap();
    idx.add_all(["*"].iter(), git2::IndexAddOption::DEFAULT, None)
        .unwrap();
    idx.write().unwrap();

    let tree_oid = idx.write_tree().unwrap();
    let tree = repo.find_tree(tree_oid).unwrap();
    let sig = git2::Signature::now("Test", "test@test.com").unwrap();

    let parents: Vec<git2::Commit> = if let Ok(head) = repo.head() {
        vec![head.peel_to_commit().unwrap()]
    } else {
        vec![]
    };
    let parent_refs: Vec<&git2::Commit> = parents.iter().collect();

    repo.commit(Some("HEAD"), &sig, &sig, message, &tree, &parent_refs)
        .unwrap();
}
