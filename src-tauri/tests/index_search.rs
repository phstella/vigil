//! Integration tests for the file index and watcher.
//!
//! These tests exercise `FileIndex` directly (not via Tauri IPC) to validate
//! full scan, incremental updates, metadata extraction, and watcher behavior.

use std::fs;
use std::thread;
use std::time::Duration;

use vigil_lib::core::content::ContentSearcher;
use vigil_lib::core::index::{ChangeKind, FileIndex, FileWatcher, TagIndex};
use vigil_lib::core::search::FuzzyFinder;
use vigil_lib::models::files::EntryKind;

/// Create a temporary workspace directory.
fn temp_workspace() -> tempfile::TempDir {
    tempfile::tempdir().expect("failed to create temp dir")
}

// ---------------------------------------------------------------------------
// Full scan
// ---------------------------------------------------------------------------

#[test]
fn full_scan_empty_workspace() {
    let dir = temp_workspace();
    let index = FileIndex::new(dir.path().to_path_buf());
    let result = index.full_scan();

    assert_eq!(result.files_count, 0);
    assert_eq!(result.notes_count, 0);
    assert!(!index.is_populated());
}

#[test]
fn full_scan_with_files_and_notes() {
    let dir = temp_workspace();
    fs::write(dir.path().join("readme.md"), "# Readme\n\nHello.").unwrap();
    fs::write(dir.path().join("config.txt"), "key=value").unwrap();
    fs::create_dir(dir.path().join("notes")).unwrap();
    fs::write(dir.path().join("notes/daily.md"), "# Daily\n\nSome notes.").unwrap();
    fs::write(dir.path().join("notes/todo.md"), "# Todo\n\n- item 1").unwrap();

    let index = FileIndex::new(dir.path().to_path_buf());
    let result = index.full_scan();

    assert_eq!(result.files_count, 4); // readme.md, config.txt, daily.md, todo.md
    assert_eq!(result.notes_count, 3); // readme.md, daily.md, todo.md
    assert!(index.is_populated());
}

#[test]
fn full_scan_skips_git_directory() {
    let dir = temp_workspace();
    fs::create_dir(dir.path().join(".git")).unwrap();
    fs::write(dir.path().join(".git/config"), "[core]").unwrap();
    fs::write(dir.path().join("note.md"), "# Note").unwrap();

    let index = FileIndex::new(dir.path().to_path_buf());
    index.full_scan();

    assert!(index.get_file_entry("note.md").is_some());
    assert!(index.get_file_entry(".git/config").is_none());
}

#[test]
fn full_scan_indexes_hidden_files() {
    let dir = temp_workspace();
    fs::write(dir.path().join(".hidden.md"), "# Hidden").unwrap();
    fs::write(dir.path().join("visible.md"), "# Visible").unwrap();

    let index = FileIndex::new(dir.path().to_path_buf());
    index.full_scan();

    let hidden = index.get_file_entry(".hidden.md").unwrap();
    assert!(hidden.is_hidden);

    let visible = index.get_file_entry("visible.md").unwrap();
    assert!(!visible.is_hidden);
}

#[test]
fn full_scan_detects_binary_files() {
    let dir = temp_workspace();
    let mut binary_content = vec![0u8; 100];
    binary_content[0] = b'P';
    binary_content[1] = b'K';
    binary_content[50] = 0; // null byte
    fs::write(dir.path().join("archive.zip"), &binary_content).unwrap();
    fs::write(dir.path().join("text.md"), "Hello").unwrap();

    let index = FileIndex::new(dir.path().to_path_buf());
    index.full_scan();

    let binary = index.get_file_entry("archive.zip").unwrap();
    assert!(binary.is_binary);

    let text = index.get_file_entry("text.md").unwrap();
    assert!(!text.is_binary);
}

#[test]
fn full_scan_respects_ignore_rules() {
    let dir = temp_workspace();
    // The ignore crate always respects .ignore files.
    fs::write(dir.path().join(".ignore"), "build/\n*.tmp\n").unwrap();
    fs::write(dir.path().join("kept.md"), "# Kept").unwrap();
    fs::write(dir.path().join("scratch.tmp"), "temp").unwrap();
    fs::create_dir(dir.path().join("build")).unwrap();
    fs::write(dir.path().join("build/output.js"), "// built").unwrap();

    let index = FileIndex::new(dir.path().to_path_buf());
    index.full_scan();

    assert!(index.get_file_entry("kept.md").is_some());
    assert!(index.get_file_entry("scratch.tmp").is_none());
    assert!(index.get_file_entry("build/output.js").is_none());
}

// ---------------------------------------------------------------------------
// Note metadata extraction
// ---------------------------------------------------------------------------

#[test]
fn note_metadata_with_frontmatter_title_and_tags() {
    let dir = temp_workspace();
    let content = "\
---
title: My Project
tags: [design, mvp]
---

# Heading ignored when frontmatter title exists

Body text with #inline-tag and more words.
";
    fs::write(dir.path().join("project.md"), content).unwrap();

    let index = FileIndex::new(dir.path().to_path_buf());
    index.full_scan();

    let meta = index.get_note_metadata("project.md").unwrap();
    assert_eq!(meta.title, "My Project");
    assert!(meta.has_frontmatter);
    assert!(meta.tags.contains(&"design".to_string()));
    assert!(meta.tags.contains(&"mvp".to_string()));
    assert!(meta.tags.contains(&"inline-tag".to_string()));
    assert!(meta.word_count > 0);
}

#[test]
fn note_metadata_heading_fallback() {
    let dir = temp_workspace();
    fs::write(dir.path().join("note.md"), "# My Heading\n\nBody here.\n").unwrap();

    let index = FileIndex::new(dir.path().to_path_buf());
    index.full_scan();

    let meta = index.get_note_metadata("note.md").unwrap();
    assert_eq!(meta.title, "My Heading");
    assert!(!meta.has_frontmatter);
}

#[test]
fn note_metadata_filename_fallback() {
    let dir = temp_workspace();
    fs::write(dir.path().join("plain-note.md"), "Just body text.\n").unwrap();

    let index = FileIndex::new(dir.path().to_path_buf());
    index.full_scan();

    let meta = index.get_note_metadata("plain-note.md").unwrap();
    assert_eq!(meta.title, "plain-note");
}

#[test]
fn note_metadata_extracts_wikilinks() {
    let dir = temp_workspace();
    let content = "# Note\n\nSee [[other-note]] and [[folder/deep|alias]].\n";
    fs::write(dir.path().join("linked.md"), content).unwrap();

    let index = FileIndex::new(dir.path().to_path_buf());
    index.full_scan();

    let meta = index.get_note_metadata("linked.md").unwrap();
    assert!(meta.links_out.contains(&"other-note.md".to_string()));
    assert!(meta.links_out.contains(&"folder/deep.md".to_string()));
}

#[test]
fn note_metadata_extracts_markdown_links() {
    let dir = temp_workspace();
    let content = "# Note\n\nSee [reference](docs/ref.md) and [ext](https://example.com).\n";
    fs::write(dir.path().join("linked.md"), content).unwrap();

    let index = FileIndex::new(dir.path().to_path_buf());
    index.full_scan();

    let meta = index.get_note_metadata("linked.md").unwrap();
    assert!(meta.links_out.contains(&"docs/ref.md".to_string()));
    // External URLs should not be included.
    assert!(!meta.links_out.iter().any(|l| l.contains("example.com")));
}

// ---------------------------------------------------------------------------
// Tag queries
// ---------------------------------------------------------------------------

#[test]
fn tag_count_across_notes() {
    let dir = temp_workspace();
    fs::write(
        dir.path().join("a.md"),
        "---\ntags: [alpha, beta]\n---\n\nText.\n",
    )
    .unwrap();
    fs::write(
        dir.path().join("b.md"),
        "---\ntags: [beta, gamma]\n---\n\nMore text.\n",
    )
    .unwrap();

    let index = FileIndex::new(dir.path().to_path_buf());
    index.full_scan();

    // Unique tags: alpha, beta, gamma = 3
    assert_eq!(index.get_tag_count(), 3);

    let tags = index.get_all_tags();
    assert_eq!(*tags.get("beta").unwrap(), 2);
    assert_eq!(*tags.get("alpha").unwrap(), 1);
    assert_eq!(*tags.get("gamma").unwrap(), 1);
}

// ---------------------------------------------------------------------------
// Incremental updates
// ---------------------------------------------------------------------------

#[test]
fn incremental_create_updates_index() {
    let dir = temp_workspace();
    fs::write(dir.path().join("existing.md"), "# Existing").unwrap();

    let index = FileIndex::new(dir.path().to_path_buf());
    index.full_scan();
    assert_eq!(index.get_file_count(), 1);
    assert_eq!(index.get_note_count(), 1);

    // Create a new file on disk.
    let new_path = dir.path().join("added.md");
    fs::write(&new_path, "# Added Note\n\nWith #some-tag.\n").unwrap();

    let changes = index.handle_event(&[new_path], ChangeKind::Created);
    assert_eq!(changes.len(), 1);
    assert_eq!(changes[0].path, "added.md");
    assert_eq!(changes[0].change_kind, ChangeKind::Created);

    assert_eq!(index.get_file_count(), 2);
    assert_eq!(index.get_note_count(), 2);

    let meta = index.get_note_metadata("added.md").unwrap();
    assert_eq!(meta.title, "Added Note");
    assert!(meta.tags.contains(&"some-tag".to_string()));
}

#[test]
fn incremental_modify_updates_metadata() {
    let dir = temp_workspace();
    let file_path = dir.path().join("note.md");
    fs::write(&file_path, "# Version 1\n\nOriginal content.\n").unwrap();

    let index = FileIndex::new(dir.path().to_path_buf());
    index.full_scan();

    let meta = index.get_note_metadata("note.md").unwrap();
    assert_eq!(meta.title, "Version 1");

    // Modify the file.
    fs::write(
        &file_path,
        "# Version 2\n\nUpdated content with #new-tag.\n",
    )
    .unwrap();
    let changes = index.handle_event(&[file_path], ChangeKind::Changed);
    assert_eq!(changes.len(), 1);

    let meta = index.get_note_metadata("note.md").unwrap();
    assert_eq!(meta.title, "Version 2");
    assert!(meta.tags.contains(&"new-tag".to_string()));
}

#[test]
fn incremental_delete_removes_from_index() {
    let dir = temp_workspace();
    let file_path = dir.path().join("doomed.md");
    fs::write(&file_path, "# Doomed").unwrap();

    let index = FileIndex::new(dir.path().to_path_buf());
    index.full_scan();
    assert_eq!(index.get_file_count(), 1);

    fs::remove_file(&file_path).unwrap();
    let changes = index.handle_event(&[file_path], ChangeKind::Deleted);
    assert_eq!(changes.len(), 1);
    assert_eq!(changes[0].change_kind, ChangeKind::Deleted);

    assert_eq!(index.get_file_count(), 0);
    assert_eq!(index.get_note_count(), 0);
}

#[test]
fn incremental_skips_git_paths() {
    let dir = temp_workspace();
    fs::create_dir(dir.path().join(".git")).unwrap();
    fs::write(dir.path().join(".git/config"), "[core]").unwrap();

    let index = FileIndex::new(dir.path().to_path_buf());
    index.full_scan();

    let git_path = dir.path().join(".git/config");
    let changes = index.handle_event(&[git_path], ChangeKind::Changed);
    assert!(changes.is_empty());
}

#[test]
fn incremental_handles_multiple_paths() {
    let dir = temp_workspace();
    fs::write(dir.path().join("a.md"), "# A").unwrap();

    let index = FileIndex::new(dir.path().to_path_buf());
    index.full_scan();
    assert_eq!(index.get_file_count(), 1);

    // Create two new files and report them in one event.
    let b_path = dir.path().join("b.md");
    let c_path = dir.path().join("c.txt");
    fs::write(&b_path, "# B").unwrap();
    fs::write(&c_path, "plain text").unwrap();

    let changes = index.handle_event(&[b_path, c_path], ChangeKind::Created);
    assert_eq!(changes.len(), 2);
    assert_eq!(index.get_file_count(), 3);
    assert_eq!(index.get_note_count(), 2); // a.md and b.md
}

// ---------------------------------------------------------------------------
// FileIndex thread safety
// ---------------------------------------------------------------------------

#[test]
fn file_index_is_thread_safe() {
    let dir = temp_workspace();
    fs::write(dir.path().join("a.md"), "# Alpha").unwrap();
    fs::write(dir.path().join("b.md"), "# Beta").unwrap();

    let index = FileIndex::new(dir.path().to_path_buf());
    index.full_scan();

    // Clone and access from multiple threads.
    let index_clone = index.clone();
    let handle = thread::spawn(move || {
        assert_eq!(index_clone.get_file_count(), 2);
        assert_eq!(index_clone.get_note_count(), 2);
        index_clone.get_all_files()
    });

    // Concurrent read from main thread.
    assert_eq!(index.get_note_count(), 2);

    let files = handle.join().unwrap();
    assert_eq!(files.len(), 2);
}

// ---------------------------------------------------------------------------
// File watcher integration
// ---------------------------------------------------------------------------

#[test]
fn watcher_detects_file_creation() {
    let dir = temp_workspace();
    fs::write(dir.path().join("initial.md"), "# Initial").unwrap();

    let index = FileIndex::new(dir.path().to_path_buf());
    index.full_scan();
    assert_eq!(index.get_file_count(), 1);

    // Track changes via a shared counter.
    let change_count = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));
    let change_count_clone = change_count.clone();

    let watcher_index = index.clone();
    let watcher = FileWatcher::start(dir.path(), move |events| {
        let mut by_kind = std::collections::HashMap::new();
        for event in events {
            by_kind
                .entry(event.kind)
                .or_insert_with(Vec::new)
                .push(event.path);
        }
        for (kind, paths) in by_kind {
            let changes = watcher_index.handle_event(&paths, kind);
            change_count_clone.fetch_add(changes.len() as u32, std::sync::atomic::Ordering::SeqCst);
        }
    })
    .expect("failed to start watcher");

    // Give the watcher time to start.
    thread::sleep(Duration::from_millis(100));

    // Create a new file.
    fs::write(dir.path().join("watched.md"), "# Watched").unwrap();

    // Wait for debounce + processing.
    thread::sleep(Duration::from_millis(500));

    // The watcher should have detected the new file.
    assert!(
        index.get_file_count() >= 2,
        "expected at least 2 files, got {}",
        index.get_file_count()
    );

    watcher.stop();
}

#[test]
fn watcher_detects_file_modification() {
    let dir = temp_workspace();
    let file_path = dir.path().join("note.md");
    fs::write(&file_path, "# Original").unwrap();

    let index = FileIndex::new(dir.path().to_path_buf());
    index.full_scan();

    let change_count = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));
    let change_count_clone = change_count.clone();

    let watcher_index = index.clone();
    let watcher = FileWatcher::start(dir.path(), move |events| {
        let mut by_kind = std::collections::HashMap::new();
        for event in events {
            by_kind
                .entry(event.kind)
                .or_insert_with(Vec::new)
                .push(event.path);
        }
        for (kind, paths) in by_kind {
            let changes = watcher_index.handle_event(&paths, kind);
            change_count_clone.fetch_add(changes.len() as u32, std::sync::atomic::Ordering::SeqCst);
        }
    })
    .expect("failed to start watcher");

    thread::sleep(Duration::from_millis(100));

    // Modify the file.
    fs::write(&file_path, "# Modified Title\n\nNew content.\n").unwrap();

    thread::sleep(Duration::from_millis(500));

    let meta = index.get_note_metadata("note.md").unwrap();
    assert_eq!(meta.title, "Modified Title");

    watcher.stop();
}

#[test]
fn watcher_detects_file_deletion() {
    let dir = temp_workspace();
    let file_path = dir.path().join("to-delete.md");
    fs::write(&file_path, "# To Delete").unwrap();

    let index = FileIndex::new(dir.path().to_path_buf());
    index.full_scan();
    assert_eq!(index.get_file_count(), 1);

    let watcher_index = index.clone();
    let watcher = FileWatcher::start(dir.path(), move |events| {
        let mut by_kind = std::collections::HashMap::new();
        for event in events {
            by_kind
                .entry(event.kind)
                .or_insert_with(Vec::new)
                .push(event.path);
        }
        for (kind, paths) in by_kind {
            watcher_index.handle_event(&paths, kind);
        }
    })
    .expect("failed to start watcher");

    thread::sleep(Duration::from_millis(100));

    // Delete the file.
    fs::remove_file(&file_path).unwrap();

    thread::sleep(Duration::from_millis(500));

    assert_eq!(index.get_file_count(), 0);

    watcher.stop();
}

// ---------------------------------------------------------------------------
// AppState integration
// ---------------------------------------------------------------------------

#[test]
fn app_state_holds_index() {
    let dir = temp_workspace();
    fs::write(dir.path().join("note.md"), "# Note").unwrap();

    let state = vigil_lib::state::AppState::new();
    assert!(state.index().is_none());

    let index = FileIndex::new(dir.path().to_path_buf());
    index.full_scan();
    state.set_index(index);

    let retrieved = state.index().unwrap();
    assert_eq!(retrieved.get_file_count(), 1);
    assert_eq!(retrieved.get_note_count(), 1);
}

#[test]
fn app_state_clear_all_resets_everything() {
    let dir = temp_workspace();
    fs::write(dir.path().join("note.md"), "# Note").unwrap();

    let state = vigil_lib::state::AppState::new();

    let (ws, _) = vigil_lib::core::fs::WorkspaceFs::open(dir.path().to_str().unwrap()).unwrap();
    state.set_workspace(ws);

    let index = FileIndex::new(dir.path().to_path_buf());
    index.full_scan();
    state.set_index(index);

    assert!(state.workspace().is_some());
    assert!(state.index().is_some());

    state.clear_all();

    assert!(state.workspace().is_none());
    assert!(state.index().is_none());
}

// ---------------------------------------------------------------------------
// Query methods
// ---------------------------------------------------------------------------

#[test]
fn get_all_files_returns_complete_list() {
    let dir = temp_workspace();
    fs::write(dir.path().join("a.md"), "# A").unwrap();
    fs::write(dir.path().join("b.txt"), "text").unwrap();
    fs::create_dir(dir.path().join("sub")).unwrap();
    fs::write(dir.path().join("sub/c.md"), "# C").unwrap();

    let index = FileIndex::new(dir.path().to_path_buf());
    index.full_scan();

    let all = index.get_all_files();
    let paths: Vec<&str> = all.iter().map(|f| f.path.as_str()).collect();
    assert!(paths.contains(&"a.md"));
    assert!(paths.contains(&"b.txt"));
    assert!(paths.contains(&"sub/c.md"));
}

#[test]
fn get_all_notes_returns_only_markdown() {
    let dir = temp_workspace();
    fs::write(dir.path().join("note.md"), "# Note").unwrap();
    fs::write(dir.path().join("config.txt"), "key=val").unwrap();
    fs::write(dir.path().join("data.json"), "{}").unwrap();

    let index = FileIndex::new(dir.path().to_path_buf());
    index.full_scan();

    let notes = index.get_all_notes();
    assert_eq!(notes.len(), 1);
    assert_eq!(notes[0].path, "note.md");
}

#[test]
fn get_file_entry_nonexistent_returns_none() {
    let dir = temp_workspace();
    let index = FileIndex::new(dir.path().to_path_buf());
    index.full_scan();

    assert!(index.get_file_entry("nonexistent.md").is_none());
    assert!(index.get_note_metadata("nonexistent.md").is_none());
}

// ---------------------------------------------------------------------------
// Scan result timing
// ---------------------------------------------------------------------------

#[test]
fn scan_result_has_valid_duration() {
    let dir = temp_workspace();
    for i in 0..50 {
        fs::write(
            dir.path().join(format!("note-{i}.md")),
            format!("# Note {i}\n\nContent for note {i}."),
        )
        .unwrap();
    }

    let index = FileIndex::new(dir.path().to_path_buf());
    let result = index.full_scan();

    assert_eq!(result.files_count, 50);
    assert_eq!(result.notes_count, 50);
    // Duration should be reasonable (< 5 seconds for 50 files).
    assert!(result.duration_ms < 5000);
}

// ---------------------------------------------------------------------------
// Fuzzy search integration
// ---------------------------------------------------------------------------

#[test]
fn fuzzy_find_empty_query_returns_recent_files() {
    let dir = temp_workspace();
    fs::write(dir.path().join("alpha.md"), "# Alpha").unwrap();
    fs::write(dir.path().join("beta.md"), "# Beta").unwrap();
    fs::write(dir.path().join("gamma.txt"), "text").unwrap();

    let index = FileIndex::new(dir.path().to_path_buf());
    index.full_scan();

    let finder = FuzzyFinder::new(&index);
    let results = finder.fuzzy_find("", 10);

    assert_eq!(results.len(), 3);
    assert!(results.iter().all(|m| m.score == 0.0));
    assert!(results.iter().all(|m| m.kind == EntryKind::File));
}

#[test]
fn fuzzy_find_matches_by_filename() {
    let dir = temp_workspace();
    fs::write(dir.path().join("readme.md"), "# Readme").unwrap();
    fs::write(dir.path().join("config.toml"), "key=val").unwrap();
    fs::write(dir.path().join("recipe.md"), "# Recipe").unwrap();

    let index = FileIndex::new(dir.path().to_path_buf());
    index.full_scan();

    let finder = FuzzyFinder::new(&index);
    let results = finder.fuzzy_find("read", 10);

    // "readme.md" should match "read"
    let paths: Vec<&str> = results.iter().map(|m| m.path.as_str()).collect();
    assert!(paths.contains(&"readme.md"));
    assert!(results.iter().all(|m| m.score > 0.0));
}

#[test]
fn fuzzy_find_respects_limit() {
    let dir = temp_workspace();
    for i in 0..30 {
        fs::write(
            dir.path().join(format!("note-{i:03}.md")),
            format!("# Note {i}"),
        )
        .unwrap();
    }

    let index = FileIndex::new(dir.path().to_path_buf());
    index.full_scan();

    let finder = FuzzyFinder::new(&index);
    let results = finder.fuzzy_find("note", 5);

    assert_eq!(results.len(), 5);
}

#[test]
fn fuzzy_find_results_sorted_by_score() {
    let dir = temp_workspace();
    fs::write(dir.path().join("abc.md"), "text").unwrap();
    fs::write(dir.path().join("a_long_b_long_c.md"), "text").unwrap();
    fs::write(dir.path().join("xyz.md"), "text").unwrap();

    let index = FileIndex::new(dir.path().to_path_buf());
    index.full_scan();

    let finder = FuzzyFinder::new(&index);
    let results = finder.fuzzy_find("abc", 10);

    // Results should be in descending score order.
    for window in results.windows(2) {
        assert!(
            window[0].score >= window[1].score,
            "expected {} >= {} for {} vs {}",
            window[0].score,
            window[1].score,
            window[0].path,
            window[1].path,
        );
    }
}

#[test]
fn fuzzy_find_excludes_directories() {
    let dir = temp_workspace();
    fs::create_dir(dir.path().join("subdir")).unwrap();
    fs::write(dir.path().join("subdir/file.md"), "# File").unwrap();

    let index = FileIndex::new(dir.path().to_path_buf());
    index.full_scan();

    let finder = FuzzyFinder::new(&index);

    // "sub" could match the directory name, but we only return files.
    let results = finder.fuzzy_find("sub", 10);
    assert!(results.iter().all(|m| m.kind == EntryKind::File));
}

#[test]
fn fuzzy_find_returns_matched_indices() {
    let dir = temp_workspace();
    fs::write(dir.path().join("hello.md"), "# Hello").unwrap();

    let index = FileIndex::new(dir.path().to_path_buf());
    index.full_scan();

    let finder = FuzzyFinder::new(&index);
    let results = finder.fuzzy_find("hlo", 10);

    assert!(!results.is_empty());
    let first = &results[0];
    assert_eq!(first.path, "hello.md");
    assert!(!first.matched_indices.is_empty());
}

#[test]
fn fuzzy_find_searches_full_path() {
    let dir = temp_workspace();
    fs::create_dir_all(dir.path().join("docs/api")).unwrap();
    fs::write(dir.path().join("docs/api/reference.md"), "# Ref").unwrap();
    fs::write(dir.path().join("notes.md"), "# Notes").unwrap();

    let index = FileIndex::new(dir.path().to_path_buf());
    index.full_scan();

    let finder = FuzzyFinder::new(&index);
    let results = finder.fuzzy_find("api/ref", 10);

    assert!(!results.is_empty());
    assert_eq!(results[0].path, "docs/api/reference.md");
}

#[test]
fn fuzzy_find_case_insensitive_for_lowercase_query() {
    let dir = temp_workspace();
    fs::write(dir.path().join("README.md"), "# Readme").unwrap();
    fs::write(dir.path().join("other.txt"), "text").unwrap();

    let index = FileIndex::new(dir.path().to_path_buf());
    index.full_scan();

    let finder = FuzzyFinder::new(&index);
    // Lowercase query should match uppercase filename (smart case).
    let results = finder.fuzzy_find("readme", 10);

    let paths: Vec<&str> = results.iter().map(|m| m.path.as_str()).collect();
    assert!(paths.contains(&"README.md"));
}

#[test]
fn fuzzy_find_performance_with_many_files() {
    let dir = temp_workspace();
    // Create a modest workspace (1000 files across subdirectories).
    for i in 0..10 {
        let subdir = dir.path().join(format!("dir-{i}"));
        fs::create_dir(&subdir).unwrap();
        for j in 0..100 {
            fs::write(
                subdir.join(format!("file-{j:03}.md")),
                format!("# File {i}-{j}"),
            )
            .unwrap();
        }
    }

    let index = FileIndex::new(dir.path().to_path_buf());
    index.full_scan();
    assert_eq!(index.get_file_count(), 1000);

    let finder = FuzzyFinder::new(&index);

    let start = std::time::Instant::now();
    let results = finder.fuzzy_find("file-05", 20);
    let elapsed = start.elapsed();

    assert!(!results.is_empty());
    // Must complete well under the 50ms budget for 10K files; 1K files
    // should be comfortably under 50ms even in debug mode.
    assert!(
        elapsed.as_millis() < 500,
        "fuzzy_find took {}ms, expected < 500ms for 1K files in debug mode",
        elapsed.as_millis()
    );
}

#[test]
fn fuzzy_find_no_match_returns_empty() {
    let dir = temp_workspace();
    fs::write(dir.path().join("hello.md"), "# Hello").unwrap();

    let index = FileIndex::new(dir.path().to_path_buf());
    index.full_scan();

    let finder = FuzzyFinder::new(&index);
    let results = finder.fuzzy_find("zzzzzzzzz", 10);

    assert!(results.is_empty());
}

#[test]
fn fuzzy_find_empty_index_returns_empty() {
    let dir = temp_workspace();

    let index = FileIndex::new(dir.path().to_path_buf());
    index.full_scan();

    let finder = FuzzyFinder::new(&index);
    let results = finder.fuzzy_find("anything", 10);
    assert!(results.is_empty());

    let results = finder.fuzzy_find("", 10);
    assert!(results.is_empty());
}

// ---------------------------------------------------------------------------
// Content search integration
// ---------------------------------------------------------------------------

#[test]
fn content_search_basic_match() {
    let dir = temp_workspace();
    fs::write(
        dir.path().join("note.md"),
        "# Hello\n\nThis is a test note.\n",
    )
    .unwrap();
    fs::write(dir.path().join("other.md"), "# Other\n\nNo match here.\n").unwrap();

    let index = FileIndex::new(dir.path().to_path_buf());
    index.full_scan();

    let searcher = ContentSearcher::new(&index);
    let results = searcher.search_content("test note", dir.path(), 50);

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].path, "note.md");
    assert_eq!(results[0].line_number, 3);
    assert!(results[0].preview.contains("test note"));
}

#[test]
fn content_search_case_insensitive() {
    let dir = temp_workspace();
    fs::write(
        dir.path().join("note.md"),
        "Hello WORLD\nworld hello\nno match\n",
    )
    .unwrap();

    let index = FileIndex::new(dir.path().to_path_buf());
    index.full_scan();

    let searcher = ContentSearcher::new(&index);
    let results = searcher.search_content("world", dir.path(), 50);

    assert_eq!(results.len(), 2);
}

#[test]
fn content_search_empty_query_returns_nothing() {
    let dir = temp_workspace();
    fs::write(dir.path().join("note.md"), "Some content\n").unwrap();

    let index = FileIndex::new(dir.path().to_path_buf());
    index.full_scan();

    let searcher = ContentSearcher::new(&index);
    let results = searcher.search_content("", dir.path(), 50);

    assert!(results.is_empty());
}

#[test]
fn content_search_respects_limit() {
    let dir = temp_workspace();
    let content: String = (0..100).map(|i| format!("line {i} keyword\n")).collect();
    fs::write(dir.path().join("big.md"), &content).unwrap();

    let index = FileIndex::new(dir.path().to_path_buf());
    index.full_scan();

    let searcher = ContentSearcher::new(&index);
    let results = searcher.search_content("keyword", dir.path(), 5);

    assert_eq!(results.len(), 5);
}

#[test]
fn content_search_skips_binary_files() {
    let dir = temp_workspace();
    // Binary file with null bytes.
    let mut binary = vec![0u8; 200];
    binary[0] = b's';
    binary[1] = b'e';
    binary[2] = b'a';
    binary[3] = b'r';
    binary[4] = b'c';
    binary[5] = b'h';
    binary[100] = 0; // null byte makes it binary
    fs::write(dir.path().join("data.bin"), &binary).unwrap();
    fs::write(dir.path().join("text.md"), "search here\n").unwrap();

    let index = FileIndex::new(dir.path().to_path_buf());
    index.full_scan();

    let searcher = ContentSearcher::new(&index);
    let results = searcher.search_content("search", dir.path(), 50);

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].path, "text.md");
}

#[test]
fn content_search_correct_columns() {
    let dir = temp_workspace();
    fs::write(dir.path().join("note.md"), "hello world foo\n").unwrap();

    let index = FileIndex::new(dir.path().to_path_buf());
    index.full_scan();

    let searcher = ContentSearcher::new(&index);
    let results = searcher.search_content("world", dir.path(), 50);

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].line_number, 1);
    assert_eq!(results[0].line_start_col, 6);
    assert_eq!(results[0].line_end_col, 11);
}

#[test]
fn content_search_multiple_matches_per_line() {
    let dir = temp_workspace();
    fs::write(dir.path().join("note.md"), "foo bar foo baz foo\n").unwrap();

    let index = FileIndex::new(dir.path().to_path_buf());
    index.full_scan();

    let searcher = ContentSearcher::new(&index);
    let results = searcher.search_content("foo", dir.path(), 50);

    assert_eq!(results.len(), 3);
    let cols: Vec<u32> = results.iter().map(|m| m.line_start_col).collect();
    assert!(cols.contains(&0));
    assert!(cols.contains(&8));
    assert!(cols.contains(&16));
}

#[test]
fn content_search_across_multiple_files() {
    let dir = temp_workspace();
    fs::write(dir.path().join("a.md"), "target found here\n").unwrap();
    fs::write(dir.path().join("b.md"), "nothing here\n").unwrap();
    fs::write(dir.path().join("c.md"), "another target line\n").unwrap();

    let index = FileIndex::new(dir.path().to_path_buf());
    index.full_scan();

    let searcher = ContentSearcher::new(&index);
    let results = searcher.search_content("target", dir.path(), 50);

    assert_eq!(results.len(), 2);
    let paths: Vec<&str> = results.iter().map(|m| m.path.as_str()).collect();
    assert!(paths.contains(&"a.md"));
    assert!(paths.contains(&"c.md"));
}

#[test]
fn content_search_no_match_returns_empty() {
    let dir = temp_workspace();
    fs::write(dir.path().join("note.md"), "Hello world\n").unwrap();

    let index = FileIndex::new(dir.path().to_path_buf());
    index.full_scan();

    let searcher = ContentSearcher::new(&index);
    let results = searcher.search_content("zzzznotfound", dir.path(), 50);

    assert!(results.is_empty());
}

#[test]
fn content_search_in_subdirectories() {
    let dir = temp_workspace();
    fs::create_dir_all(dir.path().join("notes/daily")).unwrap();
    fs::write(
        dir.path().join("notes/daily/monday.md"),
        "# Monday\n\nImportant meeting today.\n",
    )
    .unwrap();
    fs::write(dir.path().join("top.md"), "Top level file.\n").unwrap();

    let index = FileIndex::new(dir.path().to_path_buf());
    index.full_scan();

    let searcher = ContentSearcher::new(&index);
    let results = searcher.search_content("meeting", dir.path(), 50);

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].path, "notes/daily/monday.md");
    assert_eq!(results[0].line_number, 3);
}

#[test]
fn content_search_performance_1k_files() {
    let dir = temp_workspace();
    // Create 1000 files across 10 directories.
    for i in 0..10 {
        let subdir = dir.path().join(format!("dir-{i}"));
        fs::create_dir(&subdir).unwrap();
        for j in 0..100 {
            let content = if j == 50 {
                format!("# File {i}-{j}\n\nThis contains the needle phrase.\n")
            } else {
                format!("# File {i}-{j}\n\nJust regular content here.\n")
            };
            fs::write(subdir.join(format!("file-{j:03}.md")), &content).unwrap();
        }
    }

    let index = FileIndex::new(dir.path().to_path_buf());
    index.full_scan();
    assert_eq!(index.get_file_count(), 1000);

    let searcher = ContentSearcher::new(&index);

    let start = std::time::Instant::now();
    let results = searcher.search_content("needle phrase", dir.path(), 50);
    let elapsed = start.elapsed();

    // Should find exactly 10 matches (one per directory, in file-050.md).
    assert_eq!(results.len(), 10);

    // Should complete in reasonable time even in debug mode.
    assert!(
        elapsed.as_millis() < 5000,
        "content search took {}ms, expected < 5000ms for 1K files in debug mode",
        elapsed.as_millis()
    );
}

// ---------------------------------------------------------------------------
// Tag index integration
// ---------------------------------------------------------------------------

#[test]
fn tag_index_rebuild_from_file_index() {
    let dir = temp_workspace();
    fs::write(
        dir.path().join("a.md"),
        "---\ntags: [alpha, beta]\n---\n\nText.\n",
    )
    .unwrap();
    fs::write(
        dir.path().join("b.md"),
        "---\ntags: [beta, gamma]\n---\n\nMore text.\n",
    )
    .unwrap();

    let index = FileIndex::new(dir.path().to_path_buf());
    index.full_scan();

    let tag_index = TagIndex::new();
    tag_index.rebuild(&index);

    let all_tags = tag_index.get_all_tags();
    assert_eq!(all_tags.len(), 3);

    // beta has the highest count
    assert_eq!(all_tags[0].name, "beta");
    assert_eq!(all_tags[0].count, 2);
    assert_eq!(all_tags[0].files.len(), 2);

    // alpha and gamma have count 1
    let single_tags: Vec<&str> = all_tags[1..].iter().map(|t| t.name.as_str()).collect();
    assert!(single_tags.contains(&"alpha"));
    assert!(single_tags.contains(&"gamma"));
}

#[test]
fn tag_index_get_file_tags() {
    let dir = temp_workspace();
    fs::write(
        dir.path().join("note.md"),
        "---\ntags: [zebra, alpha]\n---\n\nBody #middle content.\n",
    )
    .unwrap();

    let index = FileIndex::new(dir.path().to_path_buf());
    index.full_scan();

    let tag_index = TagIndex::new();
    tag_index.rebuild(&index);

    let tags = tag_index.get_file_tags("note.md");
    assert_eq!(tags.len(), 3);
    assert!(tags.contains(&"alpha".to_string()));
    assert!(tags.contains(&"middle".to_string()));
    assert!(tags.contains(&"zebra".to_string()));
    // Verify sorted
    assert_eq!(tags, {
        let mut sorted = tags.clone();
        sorted.sort();
        sorted
    });
}

#[test]
fn tag_index_get_files_by_tag_case_insensitive() {
    let dir = temp_workspace();
    fs::write(
        dir.path().join("a.md"),
        "---\ntags: [Project]\n---\n\nText.\n",
    )
    .unwrap();
    fs::write(dir.path().join("b.md"), "# Note\n\n#project content.\n").unwrap();

    let index = FileIndex::new(dir.path().to_path_buf());
    index.full_scan();

    let tag_index = TagIndex::new();
    tag_index.rebuild(&index);

    // All case variants should return the same results
    let files_lower = tag_index.get_files_by_tag("project");
    let files_upper = tag_index.get_files_by_tag("PROJECT");
    let files_mixed = tag_index.get_files_by_tag("Project");

    assert_eq!(files_lower.len(), 2);
    assert_eq!(files_lower, files_upper);
    assert_eq!(files_lower, files_mixed);
}

#[test]
fn tag_index_nonexistent_tag_returns_empty() {
    let dir = temp_workspace();
    fs::write(dir.path().join("a.md"), "# Note\n\nText.\n").unwrap();

    let index = FileIndex::new(dir.path().to_path_buf());
    index.full_scan();

    let tag_index = TagIndex::new();
    tag_index.rebuild(&index);

    assert!(tag_index.get_files_by_tag("nonexistent").is_empty());
    assert!(tag_index.get_file_tags("nonexistent.md").is_empty());
}

#[test]
fn tag_index_excludes_code_block_tags() {
    let dir = temp_workspace();
    let content = "# Note\n\nReal #valid-tag here.\n\n```\n#code-tag\n```\n";
    fs::write(dir.path().join("note.md"), content).unwrap();

    let index = FileIndex::new(dir.path().to_path_buf());
    index.full_scan();

    let tag_index = TagIndex::new();
    tag_index.rebuild(&index);

    let tags = tag_index.get_file_tags("note.md");
    assert!(tags.contains(&"valid-tag".to_string()));
    assert!(
        !tags.contains(&"code-tag".to_string()),
        "tags inside code blocks should not be indexed"
    );
}

#[test]
fn tag_index_rebuild_clears_stale_data() {
    let dir = temp_workspace();
    fs::write(
        dir.path().join("note.md"),
        "---\ntags: [old-tag]\n---\n\nText.\n",
    )
    .unwrap();

    let index = FileIndex::new(dir.path().to_path_buf());
    index.full_scan();

    let tag_index = TagIndex::new();
    tag_index.rebuild(&index);
    assert_eq!(tag_index.get_files_by_tag("old-tag").len(), 1);

    // Modify file and rescan
    fs::write(
        dir.path().join("note.md"),
        "---\ntags: [new-tag]\n---\n\nText.\n",
    )
    .unwrap();
    index.full_scan();
    tag_index.rebuild(&index);

    assert!(tag_index.get_files_by_tag("old-tag").is_empty());
    assert_eq!(tag_index.get_files_by_tag("new-tag").len(), 1);
}

#[test]
fn tag_index_with_inline_tags() {
    let dir = temp_workspace();
    fs::write(
        dir.path().join("note.md"),
        "# My Note\n\nSome text #first-tag and #second_tag.\n",
    )
    .unwrap();

    let index = FileIndex::new(dir.path().to_path_buf());
    index.full_scan();

    let tag_index = TagIndex::new();
    tag_index.rebuild(&index);

    let tags = tag_index.get_file_tags("note.md");
    assert!(tags.contains(&"first-tag".to_string()));
    assert!(tags.contains(&"second_tag".to_string()));

    let files = tag_index.get_files_by_tag("first-tag");
    assert_eq!(files, vec!["note.md"]);
}

#[test]
fn tag_index_app_state_integration() {
    let dir = temp_workspace();
    fs::write(
        dir.path().join("a.md"),
        "---\ntags: [testing]\n---\n\nText.\n",
    )
    .unwrap();

    let state = vigil_lib::state::AppState::new();
    assert!(state.tag_index().is_none());

    let index = FileIndex::new(dir.path().to_path_buf());
    index.full_scan();
    state.set_index(index.clone());

    let tag_index = TagIndex::new();
    tag_index.rebuild(&index);
    state.set_tag_index(tag_index);

    let retrieved = state.tag_index().unwrap();
    let all_tags = retrieved.get_all_tags();
    assert_eq!(all_tags.len(), 1);
    assert_eq!(all_tags[0].name, "testing");
}

#[test]
fn tag_index_cleared_on_state_clear_all() {
    let dir = temp_workspace();
    fs::write(dir.path().join("a.md"), "---\ntags: [test]\n---\n\nText.\n").unwrap();

    let state = vigil_lib::state::AppState::new();
    let index = FileIndex::new(dir.path().to_path_buf());
    index.full_scan();
    state.set_index(index.clone());

    let tag_index = TagIndex::new();
    tag_index.rebuild(&index);
    state.set_tag_index(tag_index);

    assert!(state.tag_index().is_some());

    state.clear_all();
    assert!(state.tag_index().is_none());
}

#[test]
fn tag_index_thread_safety() {
    let dir = temp_workspace();
    fs::write(
        dir.path().join("a.md"),
        "---\ntags: [shared]\n---\n\nText.\n",
    )
    .unwrap();

    let index = FileIndex::new(dir.path().to_path_buf());
    index.full_scan();

    let tag_index = TagIndex::new();
    tag_index.rebuild(&index);

    let tag_clone = tag_index.clone();
    let handle = thread::spawn(move || {
        let tags = tag_clone.get_all_tags();
        assert_eq!(tags.len(), 1);
        tags
    });

    // Concurrent read from main thread
    let files = tag_index.get_files_by_tag("shared");
    assert_eq!(files.len(), 1);

    let tags = handle.join().unwrap();
    assert_eq!(tags[0].name, "shared");
}
