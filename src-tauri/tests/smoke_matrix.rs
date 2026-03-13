//! Smoke Matrix Validation Tests
//!
//! These tests verify that the backend services required by the MVP smoke matrix
//! (QA-001 through QA-011 in docs/qa/test-matrix.md) are functional. They exercise
//! the core service layer without a Tauri runtime, confirming that each QA flow's
//! backend dependencies compile and produce correct results.
//!
//! Smoke matrix coverage:
//! - QA-001 (Launch): Validated by CI artifact build (`npx tauri build`)
//! - QA-002 (Open workspace): WorkspaceFs + FileIndex produce valid tree
//! - QA-003 (Read/write markdown): WorkspaceFs read/write round-trips content
//! - QA-005 (Fuzzy file search): FuzzyFinder returns ranked results
//! - QA-006 (Content search): ContentSearcher returns phrase matches
//! - QA-007 (Git gutter): Validated by git_worker integration test
//! - QA-008 (Backlinks): LinkGraph resolves backlinks
//! - QA-009 (Status bar): WorkspaceStatus assembles live values
//! - QA-010 (Keyboard shortcuts): Frontend-only, validated by UI tests
//! - QA-011 (Packaging): Validated by CI `npx tauri build` producing artifacts

use std::fs;
use vigil_lib::core::content::service::ContentSearcher;
use vigil_lib::core::fs::service::WorkspaceFs;
use vigil_lib::core::index::service::FileIndex;
use vigil_lib::core::links::service::LinkGraph;
use vigil_lib::core::search::service::FuzzyFinder;

/// QA-002: Workspace opens and file tree loads
#[test]
fn smoke_qa002_open_workspace_loads_tree() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    fs::write(root.join("note-a.md"), "# Alpha\nSome content").unwrap();
    fs::create_dir_all(root.join("subdir")).unwrap();
    fs::write(root.join("subdir/note-b.md"), "# Beta\nMore content").unwrap();

    let (ws, resp) = WorkspaceFs::open(root.to_str().unwrap()).unwrap();
    assert!(
        resp.files_count >= 2,
        "Expected at least 2 files in workspace"
    );

    let list_resp = ws.list_dir("").unwrap();
    assert!(
        list_resp.entries.len() >= 2,
        "Expected at least 2 entries in workspace root"
    );
}

/// QA-003: Read/write markdown round-trips
#[test]
fn smoke_qa003_read_write_markdown() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();

    // Write a file first so workspace has content
    let content = "# Test Note\n\nHello from smoke test.";
    fs::write(root.join("test.md"), content).unwrap();

    let (ws, _) = WorkspaceFs::open(root.to_str().unwrap()).unwrap();
    let read_resp = ws.read_file("test.md").unwrap();
    assert_eq!(read_resp.content, content);
    assert!(!read_resp.etag.is_empty());

    // Write updated content via the service
    let updated = "# Updated Note\n\nNew content from smoke test.";
    let write_req = vigil_lib::models::files::WriteFileRequest {
        path: "test.md".into(),
        content: updated.into(),
        etag: None,
    };
    let write_resp = ws.write_file(&write_req).unwrap();
    assert!(write_resp.size_bytes > 0);

    // Verify round-trip
    let re_read = ws.read_file("test.md").unwrap();
    assert_eq!(re_read.content, updated);
}

/// QA-005: Fuzzy file search returns ranked results
#[test]
fn smoke_qa005_fuzzy_find() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    fs::write(root.join("architecture.md"), "# Architecture").unwrap();
    fs::write(root.join("archive.md"), "# Archive").unwrap();
    fs::write(root.join("readme.md"), "# Readme").unwrap();

    let index = FileIndex::new(root.to_path_buf());
    index.full_scan();

    let finder = FuzzyFinder::new(&index);
    let results = finder.fuzzy_find("arch", 10);
    assert!(
        !results.is_empty(),
        "Fuzzy find should return results for 'arch'"
    );
    // Both architecture.md and archive.md should match
    assert!(
        results.len() >= 2,
        "Expected at least 2 matches for 'arch', got {}",
        results.len()
    );
}

/// QA-006: Content search returns phrase matches
#[test]
fn smoke_qa006_content_search() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    fs::write(
        root.join("note1.md"),
        "The quick brown fox jumps over the lazy dog",
    )
    .unwrap();
    fs::write(root.join("note2.md"), "A different note with no fox").unwrap();
    fs::write(root.join("note3.md"), "Another file entirely").unwrap();

    let index = FileIndex::new(root.to_path_buf());
    index.full_scan();

    let searcher = ContentSearcher::new(&index);
    let results = searcher.search_content("brown fox", root, 10);
    assert!(
        !results.is_empty(),
        "Content search should find 'brown fox'"
    );
}

/// QA-008: Backlinks resolve through link graph
#[test]
fn smoke_qa008_backlinks() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    fs::write(root.join("note-a.md"), "# Alpha\nSee [[note-b]]").unwrap();
    fs::write(root.join("note-b.md"), "# Beta\nReferenced by alpha").unwrap();

    let index = FileIndex::new(root.to_path_buf());
    index.full_scan();

    let graph = LinkGraph::new();
    graph.rebuild(&index);

    let backlinks = graph.get_backlinks("note-b.md");
    assert!(
        backlinks.iter().any(|bl| bl.source_path.contains("note-a")),
        "note-b should have a backlink from note-a, got: {:?}",
        backlinks
    );
}

/// QA-009: Workspace status assembles values (file/note/tag counts)
#[test]
fn smoke_qa009_workspace_status() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    fs::write(
        root.join("note.md"),
        "---\ntags: [test, smoke]\n---\n# Note\nBody text.",
    )
    .unwrap();
    fs::write(root.join("other.md"), "# Other\n\nAnother note #extra.").unwrap();

    let index = FileIndex::new(root.to_path_buf());
    index.full_scan();

    assert!(
        index.get_file_count() >= 2,
        "Index should contain at least 2 files"
    );
    assert!(
        index.get_note_count() >= 2,
        "Index should contain at least 2 notes"
    );
    assert!(
        index.get_tag_count() >= 2,
        "Index should contain at least 2 unique tags"
    );
}
