//! Git branch and sync-state queries.
//!
//! Provides lightweight queries for the current branch name and sync state
//! (ahead/behind/diverged relative to upstream). Uses `git2` and gracefully
//! handles non-git directories.

use std::path::Path;

use git2::Repository;

use crate::models::status::SyncState;

/// Query the current branch name from the repository at `root`.
///
/// Returns `None` if `root` is not inside a git repository, the repo is bare,
/// or HEAD is detached with no branch reference.
pub fn current_branch(root: &Path) -> Option<String> {
    let repo = Repository::discover(root).ok()?;
    let head = repo.head().ok()?;

    // If HEAD points to a branch, return its shorthand name.
    if head.is_branch() {
        return head.shorthand().map(|s| s.to_string());
    }

    // Detached HEAD: return the short OID as a fallback.
    head.target().map(|oid| format!("{:.7}", oid))
}

/// Determine the sync state of the current branch relative to its upstream.
///
/// For MVP, this checks whether there are uncommitted changes (working
/// directory or index modifications). If there are, it reports `Unknown`
/// (local dirty). Otherwise it attempts to compare ahead/behind counts
/// with the upstream tracking branch.
///
/// Returns `SyncState::Unknown` for non-git directories.
pub fn sync_state(root: &Path) -> SyncState {
    let repo = match Repository::discover(root) {
        Ok(r) => r,
        Err(_) => return SyncState::Unknown,
    };

    // Check for uncommitted changes (dirty working tree).
    let has_changes = has_uncommitted_changes(&repo);

    // Try to determine ahead/behind relative to upstream.
    let head = match repo.head() {
        Ok(h) => h,
        Err(_) => {
            // No HEAD (empty repo) or other error.
            return SyncState::Unknown;
        }
    };

    let local_oid = match head.target() {
        Some(oid) => oid,
        None => return SyncState::Unknown,
    };

    // Find the upstream branch reference.
    let branch_name = match head.shorthand() {
        Some(name) => name.to_string(),
        None => {
            return if has_changes {
                SyncState::Unknown
            } else {
                SyncState::Synced
            }
        }
    };

    let branch = match repo.find_branch(&branch_name, git2::BranchType::Local) {
        Ok(b) => b,
        Err(_) => {
            return if has_changes {
                SyncState::Unknown
            } else {
                SyncState::Synced
            }
        }
    };

    let upstream = match branch.upstream() {
        Ok(u) => u,
        Err(_) => {
            // No upstream configured; can't determine sync state.
            return if has_changes {
                SyncState::Unknown
            } else {
                SyncState::Synced
            };
        }
    };

    let upstream_oid = match upstream.get().target() {
        Some(oid) => oid,
        None => return SyncState::Unknown,
    };

    match repo.graph_ahead_behind(local_oid, upstream_oid) {
        Ok((ahead, behind)) => match (ahead, behind, has_changes) {
            (0, 0, false) => SyncState::Synced,
            (0, 0, true) => SyncState::Unknown, // dirty but in sync with remote
            (_, 0, _) if ahead > 0 => SyncState::Ahead,
            (0, _, _) if behind > 0 => SyncState::Behind,
            _ => SyncState::Diverged,
        },
        Err(_) => SyncState::Unknown,
    }
}

/// Check if the repository has any uncommitted changes (staged or unstaged).
fn has_uncommitted_changes(repo: &Repository) -> bool {
    let mut opts = git2::StatusOptions::new();
    opts.include_untracked(true).recurse_untracked_dirs(false);

    match repo.statuses(Some(&mut opts)) {
        Ok(statuses) => !statuses.is_empty(),
        Err(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn non_git_dir_returns_none_branch() {
        let dir = tempfile::tempdir().unwrap();
        assert!(current_branch(dir.path()).is_none());
    }

    #[test]
    fn non_git_dir_returns_unknown_sync() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(sync_state(dir.path()), SyncState::Unknown);
    }

    #[test]
    fn git_repo_returns_branch_name() {
        let dir = tempfile::tempdir().unwrap();
        let repo = Repository::init(dir.path()).unwrap();

        // Need at least one commit for HEAD to point to a branch.
        let sig = repo
            .signature()
            .unwrap_or_else(|_| git2::Signature::now("test", "test@example.com").unwrap());
        let tree_id = {
            let mut index = repo.index().unwrap();
            index.write_tree().unwrap()
        };
        let tree = repo.find_tree(tree_id).unwrap();
        repo.commit(Some("HEAD"), &sig, &sig, "initial", &tree, &[])
            .unwrap();

        let branch = current_branch(dir.path());
        // Default branch in git init is usually "master" or "main"
        assert!(branch.is_some());
        let name = branch.unwrap();
        assert!(!name.is_empty());
    }

    #[test]
    fn clean_repo_synced_or_unknown() {
        let dir = tempfile::tempdir().unwrap();
        let repo = Repository::init(dir.path()).unwrap();

        let sig = repo
            .signature()
            .unwrap_or_else(|_| git2::Signature::now("test", "test@example.com").unwrap());
        let tree_id = {
            let mut index = repo.index().unwrap();
            index.write_tree().unwrap()
        };
        let tree = repo.find_tree(tree_id).unwrap();
        repo.commit(Some("HEAD"), &sig, &sig, "initial", &tree, &[])
            .unwrap();

        // No upstream configured, clean tree -> Synced
        let state = sync_state(dir.path());
        assert_eq!(state, SyncState::Synced);
    }

    #[test]
    fn dirty_repo_returns_unknown() {
        let dir = tempfile::tempdir().unwrap();
        let repo = Repository::init(dir.path()).unwrap();

        let sig = repo
            .signature()
            .unwrap_or_else(|_| git2::Signature::now("test", "test@example.com").unwrap());
        let tree_id = {
            let mut index = repo.index().unwrap();
            index.write_tree().unwrap()
        };
        let tree = repo.find_tree(tree_id).unwrap();
        repo.commit(Some("HEAD"), &sig, &sig, "initial", &tree, &[])
            .unwrap();

        // Create an uncommitted file
        fs::write(dir.path().join("dirty.txt"), "changes").unwrap();

        let state = sync_state(dir.path());
        assert_eq!(state, SyncState::Unknown);
    }
}
