//! Integration tests for workspace status service (ticket 1.10).
//!
//! Tests cover:
//! - Git branch/sync detection from real git repos
//! - Metrics collection from file and tag indices
//! - WorkspaceStatus aggregation without a Tauri runtime

use std::fs;

use vigil_lib::core::git::status as git_status;
use vigil_lib::core::index::metrics;
use vigil_lib::core::index::{FileIndex, TagIndex};
use vigil_lib::models::status::SyncState;

// ---------------------------------------------------------------------------
// Git status tests
// ---------------------------------------------------------------------------

#[test]
fn branch_name_from_git_repo() {
    let dir = tempfile::tempdir().unwrap();
    let repo = git2::Repository::init(dir.path()).unwrap();

    // Create an initial commit so HEAD points to a branch.
    let sig = git2::Signature::now("test", "test@example.com").unwrap();
    let tree_id = {
        let mut index = repo.index().unwrap();
        index.write_tree().unwrap()
    };
    let tree = repo.find_tree(tree_id).unwrap();
    repo.commit(Some("HEAD"), &sig, &sig, "initial", &tree, &[])
        .unwrap();

    let branch = git_status::current_branch(dir.path());
    assert!(branch.is_some(), "should return a branch name");
    let name = branch.unwrap();
    // git init defaults to "master" or "main" depending on config
    assert!(
        name == "master" || name == "main",
        "expected master or main, got: {name}"
    );
}

#[test]
fn branch_name_returns_none_for_non_git() {
    let dir = tempfile::tempdir().unwrap();
    assert!(git_status::current_branch(dir.path()).is_none());
}

#[test]
fn sync_state_clean_no_upstream() {
    let dir = tempfile::tempdir().unwrap();
    let repo = git2::Repository::init(dir.path()).unwrap();

    let sig = git2::Signature::now("test", "test@example.com").unwrap();
    let tree_id = {
        let mut index = repo.index().unwrap();
        index.write_tree().unwrap()
    };
    let tree = repo.find_tree(tree_id).unwrap();
    repo.commit(Some("HEAD"), &sig, &sig, "initial", &tree, &[])
        .unwrap();

    // Clean repo with no upstream -> Synced
    assert_eq!(git_status::sync_state(dir.path()), SyncState::Synced);
}

#[test]
fn sync_state_dirty_repo() {
    let dir = tempfile::tempdir().unwrap();
    let repo = git2::Repository::init(dir.path()).unwrap();

    let sig = git2::Signature::now("test", "test@example.com").unwrap();
    let tree_id = {
        let mut index = repo.index().unwrap();
        index.write_tree().unwrap()
    };
    let tree = repo.find_tree(tree_id).unwrap();
    repo.commit(Some("HEAD"), &sig, &sig, "initial", &tree, &[])
        .unwrap();

    // Create an uncommitted file
    fs::write(dir.path().join("dirty.txt"), "changes").unwrap();
    assert_eq!(git_status::sync_state(dir.path()), SyncState::Unknown);
}

#[test]
fn sync_state_non_git() {
    let dir = tempfile::tempdir().unwrap();
    assert_eq!(git_status::sync_state(dir.path()), SyncState::Unknown);
}

// ---------------------------------------------------------------------------
// Metrics tests
// ---------------------------------------------------------------------------

#[test]
fn metrics_none_indices() {
    let m = metrics::collect_metrics(None, None);
    assert_eq!(m.notes_count, 0);
    assert_eq!(m.tags_count, 0);
    assert_eq!(m.files_count, 0);
}

#[test]
fn metrics_from_real_index() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(
        dir.path().join("note.md"),
        "---\ntags: [alpha, beta]\n---\n\n# Note\n\nContent #gamma.\n",
    )
    .unwrap();
    fs::write(dir.path().join("readme.txt"), "plain file").unwrap();
    fs::create_dir(dir.path().join("sub")).unwrap();
    fs::write(dir.path().join("sub/deep.md"), "# Deep note").unwrap();

    let file_index = FileIndex::new(dir.path().to_path_buf());
    file_index.full_scan();

    let tag_index = TagIndex::new();
    tag_index.rebuild(&file_index);

    let m = metrics::collect_metrics(Some(&file_index), Some(&tag_index));
    assert_eq!(m.notes_count, 2); // note.md + sub/deep.md
    assert_eq!(m.files_count, 3); // note.md + readme.txt + sub/deep.md
    assert_eq!(m.tags_count, 3); // alpha, beta, gamma
}

// ---------------------------------------------------------------------------
// End-to-end status assembly (without Tauri runtime)
// ---------------------------------------------------------------------------

#[test]
fn assemble_status_manually() {
    let dir = tempfile::tempdir().unwrap();

    // Set up a git repo
    let repo = git2::Repository::init(dir.path()).unwrap();
    let sig = git2::Signature::now("test", "test@example.com").unwrap();
    let tree_id = {
        let mut index = repo.index().unwrap();
        index.write_tree().unwrap()
    };
    let tree = repo.find_tree(tree_id).unwrap();
    repo.commit(Some("HEAD"), &sig, &sig, "initial", &tree, &[])
        .unwrap();

    // Create workspace files
    fs::write(dir.path().join("hello.md"), "# Hello\n\n#tag1 content").unwrap();
    fs::write(dir.path().join("world.md"), "# World\n\n#tag2 text").unwrap();

    // Build index
    let file_index = FileIndex::new(dir.path().to_path_buf());
    file_index.full_scan();

    let tag_index = TagIndex::new();
    tag_index.rebuild(&file_index);

    // Assemble status (mimicking what the command does)
    let branch = git_status::current_branch(dir.path());
    let sync = git_status::sync_state(dir.path());
    let m = metrics::collect_metrics(Some(&file_index), Some(&tag_index));

    assert!(branch.is_some());
    // Dirty because the .md files are untracked
    assert_eq!(sync, SyncState::Unknown);
    assert_eq!(m.notes_count, 2);
    assert_eq!(m.tags_count, 2);
    assert_eq!(m.files_count, 2);
}
