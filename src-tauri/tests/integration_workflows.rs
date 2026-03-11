//! Cross-service integration tests (ticket 1.11).
//!
//! These tests exercise end-to-end workflows that span multiple services,
//! validating that the full pipeline works when composed together.  Each test
//! creates a temporary workspace with realistic content and chains service
//! calls the way the Tauri commands do at runtime.

use std::fs;

use vigil_lib::core::content::ContentSearcher;
use vigil_lib::core::fs::WorkspaceFs;
use vigil_lib::core::git::GitService;
use vigil_lib::core::index::{ChangeKind, FileIndex, TagIndex};
use vigil_lib::core::links::LinkGraph;
use vigil_lib::core::search::FuzzyFinder;
use vigil_lib::models::files::WriteFileRequest;

/// Create a temporary workspace with realistic content.
///
/// All note files live in the root to keep wikilink resolution simple:
/// `[[target]]` resolves to `target.md` (no subdirectory prefix).
fn populated_workspace() -> (tempfile::TempDir, WorkspaceFs) {
    let dir = tempfile::tempdir().expect("failed to create temp dir");

    // Markdown notes with tags and links (all at root level for wikilink
    // resolution consistency).
    fs::write(
        dir.path().join("readme.md"),
        "# Project README\n\nWelcome to the project. See [[getting-started]] for setup.\n",
    )
    .unwrap();

    fs::write(
        dir.path().join("getting-started.md"),
        "---\ntags: [guide, onboarding]\n---\n\n# Getting Started\n\nFollow these steps to get started.\n\n1. Clone the repo\n2. Run setup\n\nSee also [[architecture]] and #setup.\n",
    )
    .unwrap();

    fs::write(
        dir.path().join("architecture.md"),
        "---\ntags: [design, guide]\n---\n\n# Architecture\n\nThe system is composed of modules. Links back to [[getting-started]].\n\n#backend #frontend\n",
    )
    .unwrap();

    fs::write(
        dir.path().join("daily-2024-01-15.md"),
        "# Daily Log\n\nWorked on #backend improvements today.\n\nUpdated [[architecture]] docs.\n",
    )
    .unwrap();

    fs::write(
        dir.path().join("config.toml"),
        "[workspace]\nname = \"test\"\n",
    )
    .unwrap();

    let (ws, _) = WorkspaceFs::open(dir.path().to_str().unwrap()).unwrap();
    (dir, ws)
}

// ---------------------------------------------------------------------------
// Workflow 1: Open workspace -> index builds -> fuzzy find returns results
// ---------------------------------------------------------------------------

#[test]
fn workflow_open_index_fuzzy_find() {
    let (dir, _ws) = populated_workspace();

    // Build the file index (simulates what open_workspace triggers)
    let index = FileIndex::new(dir.path().to_path_buf());
    let scan = index.full_scan();

    // Verify scan found all files (5 files across root + notes/)
    assert_eq!(scan.files_count, 5, "should index all 5 files");
    assert!(scan.notes_count >= 4, "should find at least 4 markdown notes");

    // Fuzzy find: search for "arch" should match architecture.md
    let finder = FuzzyFinder::new(&index);
    let results = finder.fuzzy_find("arch", 10);
    assert!(!results.is_empty(), "fuzzy find should return results for 'arch'");
    let paths: Vec<&str> = results.iter().map(|m| m.path.as_str()).collect();
    assert!(
        paths.contains(&"architecture.md"),
        "architecture.md should be in results, got: {paths:?}"
    );

    // Fuzzy find: search for "getting" should match getting-started.md
    let results = finder.fuzzy_find("getting", 10);
    assert!(!results.is_empty());
    let paths: Vec<&str> = results.iter().map(|m| m.path.as_str()).collect();
    assert!(paths.contains(&"getting-started.md"));

    // Fuzzy find: empty query returns recent files
    let results = finder.fuzzy_find("", 10);
    assert_eq!(results.len(), 5, "empty query should return all 5 files");

    // Fuzzy find: search for "daily" should match the daily log
    let results = finder.fuzzy_find("daily", 10);
    assert!(!results.is_empty());
    let paths: Vec<&str> = results.iter().map(|m| m.path.as_str()).collect();
    assert!(paths.contains(&"daily-2024-01-15.md"));
}

// ---------------------------------------------------------------------------
// Workflow 2: Open workspace -> write file -> index updates -> search finds new content
// ---------------------------------------------------------------------------

#[test]
fn workflow_write_file_incremental_index_search() {
    let (dir, ws) = populated_workspace();

    // Build initial index
    let index = FileIndex::new(dir.path().to_path_buf());
    index.full_scan();

    let initial_count = index.get_file_count();

    // Write a new file through WorkspaceFs
    let req = WriteFileRequest {
        path: "new-topic.md".into(),
        content: "# New Topic\n\nThis note discusses quantum computing fundamentals.\n".into(),
        etag: None,
    };
    ws.write_file(&req).unwrap();

    // Incrementally update the index (simulates watcher event)
    let new_path = dir.path().join("new-topic.md");
    let changes = index.handle_event(&[new_path], ChangeKind::Created);
    assert_eq!(changes.len(), 1);
    assert_eq!(changes[0].path, "new-topic.md");

    // Verify file count increased
    assert_eq!(
        index.get_file_count(),
        initial_count + 1,
        "file count should increase by 1"
    );

    // Content search should find the new file
    let searcher = ContentSearcher::new(&index);
    let results = searcher.search_content("quantum computing", dir.path(), 50);
    assert!(!results.is_empty(), "content search should find 'quantum computing'");
    assert_eq!(results[0].path, "new-topic.md");

    // Fuzzy find should also find it
    let finder = FuzzyFinder::new(&index);
    let results = finder.fuzzy_find("new-topic", 10);
    assert!(!results.is_empty());
    let paths: Vec<&str> = results.iter().map(|m| m.path.as_str()).collect();
    assert!(paths.contains(&"new-topic.md"));
}

// ---------------------------------------------------------------------------
// Workflow 3: Open workspace -> tag index -> get_all_tags returns correct counts
// ---------------------------------------------------------------------------

#[test]
fn workflow_tag_index_correct_counts() {
    let (dir, _ws) = populated_workspace();

    // Build file index
    let index = FileIndex::new(dir.path().to_path_buf());
    index.full_scan();

    // Build tag index from file index
    let tag_index = TagIndex::new();
    tag_index.rebuild(&index);

    // Verify tags are populated
    assert!(tag_index.is_populated(), "tag index should be populated");

    let all_tags = tag_index.get_all_tags();
    assert!(!all_tags.is_empty(), "should have extracted tags");

    // Check specific tags we know exist in the test data
    let tag_names: Vec<&str> = all_tags.iter().map(|t| t.name.as_str()).collect();

    assert!(
        tag_names.contains(&"guide"),
        "should find 'guide' tag from frontmatter, got: {tag_names:?}"
    );
    assert!(
        tag_names.contains(&"backend"),
        "should find 'backend' inline tag, got: {tag_names:?}"
    );

    // "guide" appears in getting-started.md and architecture.md
    let guide_tag = all_tags.iter().find(|t| t.name == "guide").unwrap();
    assert_eq!(
        guide_tag.count, 2,
        "'guide' tag should appear in 2 files"
    );

    // "backend" appears in architecture.md and daily-2024-01-15.md
    let backend_tag = all_tags.iter().find(|t| t.name == "backend").unwrap();
    assert_eq!(
        backend_tag.count, 2,
        "'backend' tag should appear in 2 files"
    );

    // Verify get_files_by_tag works correctly
    let guide_files = tag_index.get_files_by_tag("guide");
    assert_eq!(guide_files.len(), 2);
    assert!(guide_files.contains(&"getting-started.md".to_string()));
    assert!(guide_files.contains(&"architecture.md".to_string()));

    // Verify get_file_tags works correctly
    let arch_tags = tag_index.get_file_tags("architecture.md");
    assert!(arch_tags.contains(&"design".to_string()));
    assert!(arch_tags.contains(&"guide".to_string()));
    assert!(arch_tags.contains(&"backend".to_string()));
    assert!(arch_tags.contains(&"frontend".to_string()));
}

// ---------------------------------------------------------------------------
// Workflow 4: Open workspace -> link graph -> backlinks resolve correctly
// ---------------------------------------------------------------------------

#[test]
fn workflow_link_graph_backlinks_resolve() {
    let (dir, _ws) = populated_workspace();

    // Build file index
    let index = FileIndex::new(dir.path().to_path_buf());
    index.full_scan();

    // Build link graph
    let graph = LinkGraph::new();
    graph.rebuild(&index);

    assert!(graph.is_populated(), "link graph should be populated");

    // getting-started.md is linked from readme.md and daily log (via architecture)
    let gs_backlinks = graph.get_backlinks("getting-started.md");
    // readme.md links to [[getting-started]] and architecture.md links to [[getting-started]]
    assert!(
        gs_backlinks.len() >= 2,
        "getting-started should have at least 2 backlinks, got {}",
        gs_backlinks.len()
    );
    let sources: Vec<&str> = gs_backlinks.iter().map(|b| b.source_path.as_str()).collect();
    assert!(
        sources.contains(&"readme.md"),
        "readme.md should link to getting-started, sources: {sources:?}"
    );
    assert!(
        sources.contains(&"architecture.md"),
        "architecture.md should link to getting-started, sources: {sources:?}"
    );

    // architecture.md is linked from getting-started.md and daily log
    let arch_backlinks = graph.get_backlinks("architecture.md");
    assert!(
        arch_backlinks.len() >= 2,
        "architecture should have at least 2 backlinks, got {}",
        arch_backlinks.len()
    );
    let sources: Vec<&str> = arch_backlinks.iter().map(|b| b.source_path.as_str()).collect();
    assert!(
        sources.contains(&"getting-started.md"),
        "getting-started should link to architecture, sources: {sources:?}"
    );
    assert!(
        sources.contains(&"daily-2024-01-15.md"),
        "daily log should link to architecture, sources: {sources:?}"
    );

    // Verify outgoing links
    let readme_out = graph.get_outgoing_links("readme.md");
    assert!(
        !readme_out.is_empty(),
        "readme should have outgoing links"
    );

    // Verify graph visualization includes all notes
    let note_graph = graph.get_graph(&index);
    assert!(
        note_graph.nodes.len() >= 4,
        "graph should have at least 4 nodes (the 4 md files), got {}",
        note_graph.nodes.len()
    );
    assert!(
        !note_graph.edges.is_empty(),
        "graph should have edges"
    );
}

// ---------------------------------------------------------------------------
// Workflow 5: Workspace status returns real data after workspace is opened
// ---------------------------------------------------------------------------

#[test]
fn workflow_workspace_status_assembly() {
    let (dir, ws) = populated_workspace();

    // Verify workspace opened successfully
    assert!(ws.root().exists());

    // Build indices (simulates post-open setup)
    let index = FileIndex::new(dir.path().to_path_buf());
    let scan = index.full_scan();
    assert!(scan.files_count > 0);

    let tag_index = TagIndex::new();
    tag_index.rebuild(&index);

    // Collect metrics (simulates what workspace_status command does)
    let metrics = vigil_lib::core::index::metrics::collect_metrics(
        Some(&index),
        Some(&tag_index),
    );

    assert!(metrics.files_count > 0, "should have files");
    assert!(metrics.notes_count > 0, "should have notes");
    assert!(metrics.tags_count > 0, "should have tags");

    // Verify concrete values match what we set up
    assert_eq!(metrics.files_count, 5, "5 files in workspace");
    assert!(
        metrics.notes_count >= 4,
        "at least 4 markdown notes"
    );
}

// ---------------------------------------------------------------------------
// Workflow 6: Write -> modify -> delete cycle with index consistency
// ---------------------------------------------------------------------------

#[test]
fn workflow_file_lifecycle_index_consistency() {
    let (dir, ws) = populated_workspace();

    let index = FileIndex::new(dir.path().to_path_buf());
    index.full_scan();
    let initial_count = index.get_file_count();

    // Create a new file
    let req = WriteFileRequest {
        path: "ephemeral.md".into(),
        content: "# Ephemeral\n\nTemporary note with #temp-tag.\n".into(),
        etag: None,
    };
    ws.write_file(&req).unwrap();
    let path = dir.path().join("ephemeral.md");
    index.handle_event(std::slice::from_ref(&path), ChangeKind::Created);
    assert_eq!(index.get_file_count(), initial_count + 1);

    // Verify we can find it via content search
    let searcher = ContentSearcher::new(&index);
    let results = searcher.search_content("Ephemeral", dir.path(), 50);
    assert!(!results.is_empty());

    // Modify the file
    let req = WriteFileRequest {
        path: "ephemeral.md".into(),
        content: "# Updated Ephemeral\n\nModified content with #updated-tag.\n".into(),
        etag: None,
    };
    ws.write_file(&req).unwrap();
    index.handle_event(std::slice::from_ref(&path), ChangeKind::Changed);

    // Should find updated content
    let results = searcher.search_content("Modified content", dir.path(), 50);
    assert!(!results.is_empty());
    assert_eq!(results[0].path, "ephemeral.md");

    // Old content should not be found (the note metadata is updated)
    let meta = index.get_note_metadata("ephemeral.md").unwrap();
    assert_eq!(meta.title, "Updated Ephemeral");

    // Delete the file
    ws.delete_file("ephemeral.md").unwrap();
    index.handle_event(std::slice::from_ref(&path), ChangeKind::Deleted);
    assert_eq!(index.get_file_count(), initial_count);
    assert!(index.get_note_metadata("ephemeral.md").is_none());
}

// ---------------------------------------------------------------------------
// Workflow 7: Multi-service tag and link coherence after modification
// ---------------------------------------------------------------------------

#[test]
fn workflow_tag_link_coherence_after_edit() {
    let (dir, ws) = populated_workspace();

    let index = FileIndex::new(dir.path().to_path_buf());
    index.full_scan();

    let tag_index = TagIndex::new();
    tag_index.rebuild(&index);

    let graph = LinkGraph::new();
    graph.rebuild(&index);

    // Baseline: architecture.md has tags [design, guide, backend, frontend]
    let initial_tags = tag_index.get_file_tags("architecture.md");
    assert!(initial_tags.contains(&"design".to_string()));

    // Modify architecture.md to change its tags and links
    let req = WriteFileRequest {
        path: "architecture.md".into(),
        content: "---\ntags: [design, refactored]\n---\n\n# Architecture v2\n\nRewritten. Now links to [[readme]].\n\n#new-tag\n".into(),
        etag: None,
    };
    ws.write_file(&req).unwrap();

    // Update index, then rebuild tag index and link graph
    let path = dir.path().join("architecture.md");
    index.handle_event(&[path], ChangeKind::Changed);

    tag_index.rebuild(&index);
    graph.rebuild(&index);

    // Tags should reflect the edit
    let updated_tags = tag_index.get_file_tags("architecture.md");
    assert!(
        updated_tags.contains(&"refactored".to_string()),
        "should have new 'refactored' tag, got: {updated_tags:?}"
    );
    assert!(
        updated_tags.contains(&"new-tag".to_string()),
        "should have new inline 'new-tag', got: {updated_tags:?}"
    );
    // "guide" should be gone (removed from frontmatter)
    assert!(
        !updated_tags.contains(&"guide".to_string()),
        "'guide' should be removed after edit, got: {updated_tags:?}"
    );

    // Link graph should reflect the edit: architecture now links to readme
    let arch_outgoing = graph.get_outgoing_links("architecture.md");
    assert!(!arch_outgoing.is_empty(), "should have outgoing links after edit");
    let readme_backlinks = graph.get_backlinks("readme.md");
    let has_arch_backlink = readme_backlinks
        .iter()
        .any(|b| b.source_path == "architecture.md");
    assert!(
        has_arch_backlink,
        "readme.md should have a backlink from architecture.md after edit"
    );
}

// ---------------------------------------------------------------------------
// Workflow 8: Content search across subdirectories
// ---------------------------------------------------------------------------

#[test]
fn workflow_content_search_cross_directory() {
    let dir = tempfile::tempdir().unwrap();

    // Create a workspace with files in subdirectories
    fs::create_dir(dir.path().join("notes")).unwrap();
    fs::create_dir(dir.path().join("journal")).unwrap();

    fs::write(
        dir.path().join("readme.md"),
        "# Root\n\nThe keyword searchable appears here.\n",
    )
    .unwrap();
    fs::write(
        dir.path().join("notes/topic.md"),
        "# Topic\n\nThis is a searchable document in notes/.\n",
    )
    .unwrap();
    fs::write(
        dir.path().join("journal/day1.md"),
        "# Day 1\n\nJournal entry - also searchable.\n",
    )
    .unwrap();

    let index = FileIndex::new(dir.path().to_path_buf());
    index.full_scan();

    let searcher = ContentSearcher::new(&index);

    // Search for text that appears in multiple files across directories
    let results = searcher.search_content("searchable", dir.path(), 50);
    assert!(
        results.len() >= 3,
        "should find 'searchable' in 3 files across directories, got {}",
        results.len()
    );

    let paths: Vec<&str> = results.iter().map(|m| m.path.as_str()).collect();
    assert!(
        paths.iter().any(|p| p.starts_with("notes/")),
        "results should include files from notes/ subdirectory, got: {paths:?}"
    );
    assert!(
        paths.iter().any(|p| p.starts_with("journal/")),
        "results should include files from journal/ subdirectory, got: {paths:?}"
    );
    assert!(
        paths.contains(&"readme.md"),
        "results should include root-level file, got: {paths:?}"
    );
}

// ---------------------------------------------------------------------------
// Workflow 9: Git integration with workspace operations
// ---------------------------------------------------------------------------

#[test]
fn workflow_git_status_after_workspace_changes() {
    let dir = tempfile::tempdir().unwrap();

    // Initialize a git repo
    let repo = git2::Repository::init(dir.path()).unwrap();
    let mut config = repo.config().unwrap();
    config.set_str("user.name", "Test").unwrap();
    config.set_str("user.email", "test@test.com").unwrap();

    // Create initial content and commit
    fs::write(dir.path().join("readme.md"), "# Readme\n").unwrap();
    fs::write(
        dir.path().join("notes.md"),
        "# Notes\n\nInitial content.\n",
    )
    .unwrap();
    stage_and_commit(&repo, "initial commit");

    // Open workspace and build index
    let (ws, _) = WorkspaceFs::open(dir.path().to_str().unwrap()).unwrap();
    let index = FileIndex::new(dir.path().to_path_buf());
    index.full_scan();

    let git_svc = GitService::new(dir.path().to_path_buf());

    // Status should be clean after initial commit
    let status = git_svc.get_status().unwrap();
    assert!(status.is_empty(), "status should be clean after commit");

    // Modify a file through WorkspaceFs
    let req = WriteFileRequest {
        path: "notes.md".into(),
        content: "# Notes\n\nModified content with new information.\n".into(),
        etag: None,
    };
    ws.write_file(&req).unwrap();

    // Git should detect the modification
    let status = git_svc.get_status().unwrap();
    assert!(!status.is_empty(), "should detect modified file");
    let modified = status.iter().find(|e| e.path == "notes.md");
    assert!(modified.is_some(), "notes.md should appear in status");

    // Hunks should be non-empty for the modified file
    let hunks = git_svc.get_hunks("notes.md").unwrap();
    assert!(!hunks.hunks.is_empty(), "should have hunks for modified file");

    // Index should also pick up the change
    let file_path = dir.path().join("notes.md");
    index.handle_event(&[file_path], ChangeKind::Changed);

    // Content search should find the new content
    let searcher = ContentSearcher::new(&index);
    let results = searcher.search_content("new information", dir.path(), 50);
    assert!(!results.is_empty(), "should find new content via search");
}

/// Helper: stage all and commit.
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
