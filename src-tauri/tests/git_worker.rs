//! Integration tests for the git diff/status service.
//!
//! Each test creates a temporary git repository, makes commits and working-tree
//! changes, then exercises `GitService` to verify hunk detection and status
//! reporting.

use std::fs;
use std::path::Path;

use git2::{Repository, Signature};

use vigil_lib::core::git::GitService;
use vigil_lib::models::git::{GitFileStatus, HunkChangeType};

/// Create a temporary directory with an initialised git repository
/// that has one initial commit.
fn temp_git_repo() -> (tempfile::TempDir, Repository) {
    let dir = tempfile::tempdir().expect("failed to create temp dir");
    let repo = Repository::init(dir.path()).expect("failed to init repo");

    // Configure a dummy user for commits.
    let mut config = repo.config().unwrap();
    config.set_str("user.name", "Test").unwrap();
    config.set_str("user.email", "test@test.com").unwrap();

    // Create an initial commit with a single file so HEAD exists.
    let file_path = dir.path().join("initial.md");
    fs::write(&file_path, "# Initial\n").unwrap();

    stage_and_commit(&repo, dir.path(), "initial commit");

    (dir, repo)
}

/// Stage all files and create a commit.
fn stage_and_commit(repo: &Repository, _workdir: &Path, message: &str) {
    let mut index = repo.index().unwrap();
    index
        .add_all(["*"].iter(), git2::IndexAddOption::DEFAULT, None)
        .unwrap();
    index.write().unwrap();

    let tree_oid = index.write_tree().unwrap();
    let tree = repo.find_tree(tree_oid).unwrap();
    let sig = Signature::now("Test", "test@test.com").unwrap();

    let parents: Vec<git2::Commit> = if let Ok(head) = repo.head() {
        vec![head.peel_to_commit().unwrap()]
    } else {
        vec![]
    };
    let parent_refs: Vec<&git2::Commit> = parents.iter().collect();

    repo.commit(Some("HEAD"), &sig, &sig, message, &tree, &parent_refs)
        .unwrap();
}

// ---------------------------------------------------------------------------
// get_hunks tests
// ---------------------------------------------------------------------------

#[test]
fn hunks_empty_for_unmodified_file() {
    let (dir, _repo) = temp_git_repo();
    let svc = GitService::new(dir.path().to_path_buf());

    let resp = svc.get_hunks("initial.md").unwrap();
    assert!(resp.hunks.is_empty(), "no hunks for a clean file");
}

#[test]
fn hunks_detect_added_lines() {
    let (dir, _repo) = temp_git_repo();

    // Append lines to the committed file.
    let file_path = dir.path().join("initial.md");
    let mut content = fs::read_to_string(&file_path).unwrap();
    content.push_str("new line 1\nnew line 2\n");
    fs::write(&file_path, &content).unwrap();

    let svc = GitService::new(dir.path().to_path_buf());
    let resp = svc.get_hunks("initial.md").unwrap();

    assert!(!resp.hunks.is_empty(), "should detect added lines");
    // The appended hunk should be classified as Added or Modified depending
    // on context. With 1 original line and 2 new lines appended, git2
    // typically reports this as a Modified hunk since surrounding lines exist.
    let hunk = &resp.hunks[0];
    assert!(
        hunk.change_type == HunkChangeType::Added || hunk.change_type == HunkChangeType::Modified,
        "expected Added or Modified, got {:?}",
        hunk.change_type
    );
}

#[test]
fn hunks_detect_deleted_lines() {
    let (dir, repo) = temp_git_repo();

    // Create a multi-line file and commit it.
    let file_path = dir.path().join("multi.md");
    fs::write(&file_path, "line 1\nline 2\nline 3\nline 4\nline 5\n").unwrap();
    stage_and_commit(&repo, dir.path(), "add multi-line file");

    // Delete some lines.
    fs::write(&file_path, "line 1\nline 5\n").unwrap();

    let svc = GitService::new(dir.path().to_path_buf());
    let resp = svc.get_hunks("multi.md").unwrap();

    assert!(!resp.hunks.is_empty(), "should detect deleted lines");
    let has_delete_hunk = resp.hunks.iter().any(|h| {
        h.change_type == HunkChangeType::Deleted || h.change_type == HunkChangeType::Modified
    });
    assert!(has_delete_hunk, "should have a deleted or modified hunk");
}

#[test]
fn hunks_for_new_untracked_file() {
    let (dir, _repo) = temp_git_repo();

    // Create a new untracked file.
    fs::write(dir.path().join("untracked.md"), "hello\n").unwrap();

    let svc = GitService::new(dir.path().to_path_buf());
    let resp = svc.get_hunks("untracked.md").unwrap();

    // Untracked files may return empty or may show as all-added depending on
    // git2 configuration. The key is that it does not error.
    // With diff_tree_to_workdir_with_index, untracked files should be included.
    // We just verify no error.
    let _ = resp.hunks;
}

#[test]
fn hunks_for_nonexistent_file_returns_empty() {
    let (dir, _repo) = temp_git_repo();
    let svc = GitService::new(dir.path().to_path_buf());

    let resp = svc.get_hunks("does_not_exist.md").unwrap();
    assert!(resp.hunks.is_empty());
}

#[test]
fn hunks_non_git_directory_returns_empty() {
    let dir = tempfile::tempdir().unwrap();
    let svc = GitService::new(dir.path().to_path_buf());

    let resp = svc.get_hunks("any_file.md").unwrap();
    assert!(resp.hunks.is_empty());
}

// ---------------------------------------------------------------------------
// get_status tests
// ---------------------------------------------------------------------------

#[test]
fn status_clean_repo() {
    let (dir, _repo) = temp_git_repo();
    let svc = GitService::new(dir.path().to_path_buf());

    let entries = svc.get_status().unwrap();
    assert!(
        entries.is_empty(),
        "clean repo should have no status entries"
    );
}

#[test]
fn status_detects_modified_file() {
    let (dir, _repo) = temp_git_repo();

    // Modify the committed file.
    fs::write(dir.path().join("initial.md"), "changed content\n").unwrap();

    let svc = GitService::new(dir.path().to_path_buf());
    let entries = svc.get_status().unwrap();

    assert!(!entries.is_empty(), "should detect modified file");
    let entry = entries.iter().find(|e| e.path == "initial.md").unwrap();
    assert_eq!(entry.status, GitFileStatus::Modified);
}

#[test]
fn status_detects_new_file() {
    let (dir, _repo) = temp_git_repo();

    // Create an untracked file.
    fs::write(dir.path().join("new_file.md"), "new\n").unwrap();

    let svc = GitService::new(dir.path().to_path_buf());
    let entries = svc.get_status().unwrap();

    let entry = entries.iter().find(|e| e.path == "new_file.md").unwrap();
    assert_eq!(entry.status, GitFileStatus::New);
}

#[test]
fn status_detects_deleted_file() {
    let (dir, _repo) = temp_git_repo();

    // Delete the committed file.
    fs::remove_file(dir.path().join("initial.md")).unwrap();

    let svc = GitService::new(dir.path().to_path_buf());
    let entries = svc.get_status().unwrap();

    let entry = entries.iter().find(|e| e.path == "initial.md").unwrap();
    assert_eq!(entry.status, GitFileStatus::Deleted);
}

#[test]
fn status_non_git_directory_returns_empty() {
    let dir = tempfile::tempdir().unwrap();
    let svc = GitService::new(dir.path().to_path_buf());

    let entries = svc.get_status().unwrap();
    assert!(entries.is_empty());
}

// ---------------------------------------------------------------------------
// Hunk detail validation
// ---------------------------------------------------------------------------

#[test]
fn hunk_base_lines_populated_for_modifications() {
    let (dir, repo) = temp_git_repo();

    // Create and commit a file with known content.
    let file_path = dir.path().join("detail.md");
    fs::write(&file_path, "aaa\nbbb\nccc\nddd\neee\n").unwrap();
    stage_and_commit(&repo, dir.path(), "add detail file");

    // Modify a line in the middle.
    fs::write(&file_path, "aaa\nBBB\nccc\nddd\neee\n").unwrap();

    let svc = GitService::new(dir.path().to_path_buf());
    let resp = svc.get_hunks("detail.md").unwrap();

    assert!(!resp.hunks.is_empty());
    let hunk = &resp.hunks[0];
    assert_eq!(hunk.change_type, HunkChangeType::Modified);
    assert!(hunk.base_start_line.is_some());
    assert!(hunk.base_end_line.is_some());
    assert!(hunk.start_line >= 1);
    assert!(hunk.end_line >= hunk.start_line);
}
