//! Git diff and status service.
//!
//! Uses `git2` to compute line-level diff hunks and repository status.
//! All operations gracefully handle non-git directories by returning empty
//! results rather than errors.

use std::path::PathBuf;

use git2::{DiffOptions, Repository, StatusOptions};

use crate::models::error::VigilError;
use crate::models::git::{
    GitFileStatus, GitHunk, GitHunksResponse, GitStatusEntry, HunkChangeType,
};

/// Service for git diff and status operations against a workspace root.
///
/// Constructed from a workspace root path. If the root is not inside a git
/// repository, operations return empty results instead of errors.
pub struct GitService {
    /// Workspace root (canonical).
    root: PathBuf,
}

impl GitService {
    /// Create a new `GitService` rooted at the given workspace path.
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    /// Open the git repository that contains (or is) the workspace root.
    ///
    /// Returns `None` if the workspace is not inside a git repository.
    fn open_repo(&self) -> Option<Repository> {
        Repository::discover(&self.root).ok()
    }

    /// Compute line-level diff hunks for a single file against HEAD.
    ///
    /// `rel_path` is workspace-relative. Returns an empty list when:
    /// - The workspace is not a git repo
    /// - The file is untracked
    /// - HEAD does not exist (empty repo with no commits)
    pub fn get_hunks(&self, rel_path: &str) -> Result<GitHunksResponse, VigilError> {
        let repo = match self.open_repo() {
            Some(r) => r,
            None => return Ok(GitHunksResponse { hunks: vec![] }),
        };

        // Resolve the file path relative to the repo workdir.
        let repo_workdir = match repo.workdir() {
            Some(wd) => wd.to_path_buf(),
            None => return Ok(GitHunksResponse { hunks: vec![] }), // bare repo
        };

        let abs_path = self.root.join(rel_path);
        let repo_rel = match abs_path.strip_prefix(&repo_workdir) {
            Ok(p) => p.to_path_buf(),
            Err(_) => {
                // File is outside the git workdir.
                return Ok(GitHunksResponse { hunks: vec![] });
            }
        };

        // Get HEAD tree. If HEAD doesn't exist (no commits), diff against empty.
        let head_tree = repo
            .head()
            .ok()
            .and_then(|r| r.peel_to_tree().ok());

        let mut diff_opts = DiffOptions::new();
        diff_opts.pathspec(repo_rel.to_string_lossy().as_ref());

        let diff = repo
            .diff_tree_to_workdir_with_index(head_tree.as_ref(), Some(&mut diff_opts))
            .map_err(VigilError::from)?;

        let mut hunks: Vec<GitHunk> = Vec::new();

        diff.foreach(
            &mut |_delta, _progress| true,
            None, // binary callback
            Some(&mut |_delta, hunk| {
                let old_start = hunk.old_start();
                let old_lines = hunk.old_lines();
                let new_start = hunk.new_start();
                let new_lines = hunk.new_lines();

                let change_type = classify_hunk(old_lines, new_lines);

                let (base_start, base_end) = if old_lines > 0 {
                    (Some(old_start), Some(old_start + old_lines - 1))
                } else {
                    (None, None)
                };

                let (start_line, end_line) = if new_lines > 0 {
                    (new_start, new_start + new_lines - 1)
                } else {
                    // Pure deletion: report at the line where content was removed.
                    // Use new_start as the anchor point (line after which deletion occurred).
                    (new_start.max(1), new_start.max(1))
                };

                hunks.push(GitHunk {
                    change_type,
                    start_line,
                    end_line,
                    base_start_line: base_start,
                    base_end_line: base_end,
                });

                true
            }),
            None, // line callback
        )
        .map_err(VigilError::from)?;

        Ok(GitHunksResponse { hunks })
    }

    /// Get the git status of all files in the workspace.
    ///
    /// Returns an empty list if the workspace is not a git repository.
    pub fn get_status(&self) -> Result<Vec<GitStatusEntry>, VigilError> {
        let repo = match self.open_repo() {
            Some(r) => r,
            None => return Ok(vec![]),
        };

        let mut opts = StatusOptions::new();
        opts.include_untracked(true);
        opts.recurse_untracked_dirs(true);

        let statuses = repo.statuses(Some(&mut opts)).map_err(VigilError::from)?;

        let entries: Vec<GitStatusEntry> = statuses
            .iter()
            .filter_map(|entry| {
                let path = entry.path()?.to_string();
                let status = map_git_status(entry.status());
                Some(GitStatusEntry { path, status })
            })
            .collect();

        Ok(entries)
    }
}

/// Classify a diff hunk based on old/new line counts.
fn classify_hunk(old_lines: u32, new_lines: u32) -> HunkChangeType {
    match (old_lines, new_lines) {
        (0, _) => HunkChangeType::Added,
        (_, 0) => HunkChangeType::Deleted,
        _ => HunkChangeType::Modified,
    }
}

/// Map a `git2::Status` bitflags value to our `GitFileStatus` enum.
fn map_git_status(status: git2::Status) -> GitFileStatus {
    if status.is_conflicted() {
        return GitFileStatus::Conflicted;
    }
    if status.is_wt_new() || status.is_index_new() {
        return GitFileStatus::New;
    }
    if status.is_wt_deleted() || status.is_index_deleted() {
        return GitFileStatus::Deleted;
    }
    if status.is_wt_renamed() || status.is_index_renamed() {
        return GitFileStatus::Renamed;
    }
    if status.is_wt_modified() || status.is_index_modified() || status.is_wt_typechange() || status.is_index_typechange() {
        return GitFileStatus::Modified;
    }
    if status.is_ignored() {
        return GitFileStatus::Unknown;
    }
    GitFileStatus::Clean
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classify_hunk_added() {
        assert_eq!(classify_hunk(0, 5), HunkChangeType::Added);
    }

    #[test]
    fn classify_hunk_deleted() {
        assert_eq!(classify_hunk(3, 0), HunkChangeType::Deleted);
    }

    #[test]
    fn classify_hunk_modified() {
        assert_eq!(classify_hunk(2, 4), HunkChangeType::Modified);
    }

    #[test]
    fn non_git_directory_returns_empty_hunks() {
        let dir = tempfile::tempdir().unwrap();
        let svc = GitService::new(dir.path().to_path_buf());
        let resp = svc.get_hunks("nonexistent.txt").unwrap();
        assert!(resp.hunks.is_empty());
    }

    #[test]
    fn non_git_directory_returns_empty_status() {
        let dir = tempfile::tempdir().unwrap();
        let svc = GitService::new(dir.path().to_path_buf());
        let entries = svc.get_status().unwrap();
        assert!(entries.is_empty());
    }

    #[test]
    fn map_status_new() {
        assert_eq!(map_git_status(git2::Status::WT_NEW), GitFileStatus::New);
        assert_eq!(map_git_status(git2::Status::INDEX_NEW), GitFileStatus::New);
    }

    #[test]
    fn map_status_modified() {
        assert_eq!(
            map_git_status(git2::Status::WT_MODIFIED),
            GitFileStatus::Modified
        );
    }

    #[test]
    fn map_status_deleted() {
        assert_eq!(
            map_git_status(git2::Status::WT_DELETED),
            GitFileStatus::Deleted
        );
    }

    #[test]
    fn map_status_conflicted() {
        assert_eq!(
            map_git_status(git2::Status::CONFLICTED),
            GitFileStatus::Conflicted
        );
    }
}
