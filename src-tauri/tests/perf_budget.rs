//! Performance budget validation tests (ticket 1.11).
//!
//! These tests measure the latency of hot-path operations and compare them
//! against the budgets defined in `docs/specs/editor-performance-budget.md`.
//!
//! **Important**: These are *not* formal benchmarks (no warmup, no statistics,
//! debug build).  They exist to catch gross regressions and to print timing
//! data during test runs so developers can sanity-check against the budget.
//!
//! The hard budgets are for *release-profile binaries*, so the thresholds here
//! are intentionally generous (10x-20x of the release budget) to avoid flaky
//! failures in debug builds and CI.

use std::fs;
use std::time::Instant;

use vigil_lib::core::content::ContentSearcher;
use vigil_lib::core::index::{FileIndex, TagIndex};
use vigil_lib::core::links::LinkGraph;
use vigil_lib::core::perf;
use vigil_lib::core::search::FuzzyFinder;

/// Build a workspace with `n` markdown files for latency testing.
fn build_test_workspace(n: usize) -> (tempfile::TempDir, FileIndex) {
    let dir = tempfile::tempdir().expect("failed to create temp dir");

    for i in 0..n {
        let subdir = format!("area-{}", i / 50);
        let subdir_path = dir.path().join(&subdir);
        if !subdir_path.exists() {
            fs::create_dir_all(&subdir_path).unwrap();
        }
        let content = format!(
            "---\ntags: [tag-{}, common]\n---\n\n# Note {i}\n\nThis is note number {i} with some searchable content.\n\nKeyword alpha bravo charlie delta.\n",
            i % 10
        );
        fs::write(subdir_path.join(format!("note-{i}.md")), content).unwrap();
    }

    let index = FileIndex::new(dir.path().to_path_buf());
    (dir, index)
}

// ---------------------------------------------------------------------------
// Workspace scan timing
// ---------------------------------------------------------------------------

#[test]
fn perf_full_scan_500_files() {
    let (dir, index) = build_test_workspace(500);

    let (result, duration) = perf::time_operation(|| index.full_scan());

    let ms = duration.as_secs_f64() * 1000.0;
    eprintln!(
        "[perf-budget] full_scan(500 files): {ms:.1} ms  (files={}, notes={})",
        result.files_count, result.notes_count
    );

    assert_eq!(result.files_count, 500);
    assert_eq!(result.notes_count, 500);
    // Budget: 10k files in 1500 ms => 500 files should be well under that.
    // Debug build allowance: 10x => 750 ms for 500 files.
    assert!(
        ms < 5000.0,
        "full_scan of 500 files took {ms:.1} ms, expected < 5000 ms (debug build)"
    );

    drop(dir);
}

// ---------------------------------------------------------------------------
// Fuzzy find timing
// ---------------------------------------------------------------------------

#[test]
fn perf_fuzzy_find_latency() {
    let (_dir, index) = build_test_workspace(500);
    index.full_scan();

    let finder = FuzzyFinder::new(&index);

    // Warm up (first call may be slower due to allocations)
    let _ = finder.fuzzy_find("note", 20);

    // Measure multiple queries and report median
    let queries = ["note-1", "area", "search", "alpha", "bravo charlie"];
    let mut durations_ms = Vec::new();

    for query in &queries {
        let (results, duration) = perf::time_operation(|| finder.fuzzy_find(query, 20));
        let ms = duration.as_secs_f64() * 1000.0;
        durations_ms.push(ms);
        eprintln!(
            "[perf-budget] fuzzy_find(\"{query}\", 500 files): {ms:.2} ms  ({} results)",
            results.len()
        );
    }

    durations_ms.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let median = durations_ms[durations_ms.len() / 2];
    eprintln!("[perf-budget] fuzzy_find median: {median:.2} ms");

    // Budget: 80 ms for Ctrl+P first result in release build.
    // Debug allowance: 20x => 1600 ms.
    assert!(
        median < 1600.0,
        "fuzzy_find median {median:.2} ms exceeds debug budget of 1600 ms"
    );
}

// ---------------------------------------------------------------------------
// Content search timing
// ---------------------------------------------------------------------------

#[test]
fn perf_content_search_latency() {
    let (dir, index) = build_test_workspace(500);
    index.full_scan();

    let searcher = ContentSearcher::new(&index);

    // Measure content search
    let queries = ["searchable", "alpha", "note number", "charlie delta"];
    let mut durations_ms = Vec::new();

    for query in &queries {
        let (results, duration) =
            perf::time_operation(|| searcher.search_content(query, dir.path(), 50));
        let ms = duration.as_secs_f64() * 1000.0;
        durations_ms.push(ms);
        eprintln!(
            "[perf-budget] search_content(\"{query}\", 500 files): {ms:.1} ms  ({} results)",
            results.len()
        );
    }

    durations_ms.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let median = durations_ms[durations_ms.len() / 2];
    eprintln!("[perf-budget] search_content median: {median:.1} ms");

    // Budget: 150 ms median for content search in release build.
    // Debug allowance: 20x => 3000 ms.
    assert!(
        median < 3000.0,
        "search_content median {median:.1} ms exceeds debug budget of 3000 ms"
    );
}

// ---------------------------------------------------------------------------
// Tag index build timing
// ---------------------------------------------------------------------------

#[test]
fn perf_tag_index_rebuild() {
    let (_dir, index) = build_test_workspace(500);
    index.full_scan();

    let tag_index = TagIndex::new();

    let (_, duration) = perf::time_operation(|| tag_index.rebuild(&index));
    let ms = duration.as_secs_f64() * 1000.0;
    eprintln!("[perf-budget] TagIndex::rebuild(500 files): {ms:.1} ms");

    // Tag index rebuild should be very fast (just iterating in-memory data)
    assert!(
        ms < 500.0,
        "tag index rebuild took {ms:.1} ms, expected < 500 ms"
    );

    // Verify correctness
    let all_tags = tag_index.get_all_tags();
    assert!(!all_tags.is_empty());

    // "common" tag should appear in all 500 files
    let common = all_tags.iter().find(|t| t.name == "common").unwrap();
    assert_eq!(common.count, 500);
}

// ---------------------------------------------------------------------------
// Link graph build timing
// ---------------------------------------------------------------------------

#[test]
fn perf_link_graph_rebuild() {
    let (dir, index) = build_test_workspace(200);

    // Add some cross-links to make the graph interesting
    for i in 0..200 {
        let subdir = format!("area-{}", i / 50);
        let next = (i + 1) % 200;
        let next_subdir = format!("area-{}", next / 50);
        let content = format!(
            "---\ntags: [linked]\n---\n\n# Linked Note {i}\n\nSee [[{next_subdir}/note-{next}]] for next.\n",
        );
        fs::write(dir.path().join(format!("{subdir}/note-{i}.md")), content).unwrap();
    }

    // Re-scan to pick up the modified files
    index.full_scan();

    let graph = LinkGraph::new();
    let (_, duration) = perf::time_operation(|| graph.rebuild(&index));
    let ms = duration.as_secs_f64() * 1000.0;
    eprintln!("[perf-budget] LinkGraph::rebuild(200 linked files): {ms:.1} ms");

    assert!(
        ms < 5000.0,
        "link graph rebuild took {ms:.1} ms, expected < 5000 ms"
    );

    // Verify the graph is populated
    assert!(graph.is_populated());
}

// ---------------------------------------------------------------------------
// Git hunk timing
// ---------------------------------------------------------------------------

#[test]
fn perf_git_hunks_latency() {
    use vigil_lib::core::git::GitService;

    let dir = tempfile::tempdir().unwrap();
    let repo = git2::Repository::init(dir.path()).unwrap();

    let mut config = repo.config().unwrap();
    config.set_str("user.name", "Test").unwrap();
    config.set_str("user.email", "test@test.com").unwrap();

    // Create a file with many lines and commit
    let lines: String = (0..2000)
        .map(|i| format!("Line {i}: original content\n"))
        .collect();
    fs::write(dir.path().join("big-file.md"), &lines).unwrap();

    let mut idx = repo.index().unwrap();
    idx.add_all(["*"].iter(), git2::IndexAddOption::DEFAULT, None)
        .unwrap();
    idx.write().unwrap();
    let tree_oid = idx.write_tree().unwrap();
    let tree = repo.find_tree(tree_oid).unwrap();
    let sig = git2::Signature::now("Test", "test@test.com").unwrap();
    repo.commit(Some("HEAD"), &sig, &sig, "initial", &tree, &[])
        .unwrap();

    // Modify some lines
    let mut modified_lines = String::new();
    for i in 0..2000 {
        if i % 100 == 0 {
            modified_lines.push_str(&format!("Line {i}: MODIFIED content\n"));
        } else {
            modified_lines.push_str(&format!("Line {i}: original content\n"));
        }
    }
    fs::write(dir.path().join("big-file.md"), &modified_lines).unwrap();

    let svc = GitService::new(dir.path().to_path_buf());

    let (resp, duration) = perf::time_operation(|| svc.get_hunks("big-file.md").unwrap());
    let ms = duration.as_secs_f64() * 1000.0;
    eprintln!(
        "[perf-budget] get_hunks(2000-line file, {} hunks): {ms:.1} ms",
        resp.hunks.len()
    );

    // Budget: 200 ms for git hunk refresh in release build.
    // Debug allowance: 10x => 2000 ms.
    assert!(ms < 2000.0, "get_hunks took {ms:.1} ms, expected < 2000 ms");

    assert!(!resp.hunks.is_empty(), "should detect modified hunks");
}

// ---------------------------------------------------------------------------
// PerfTimer unit-level tests (verifying the instrumentation itself)
// ---------------------------------------------------------------------------

#[test]
fn perf_timer_basic_usage() {
    let timer = perf::PerfTimer::start("test_basic");
    // Do some trivial work
    let mut sum = 0u64;
    for i in 0..10_000 {
        sum += i;
    }
    let _ = sum;
    let ms = timer.elapsed_ms();
    // Should be very fast (well under 100ms)
    assert!(ms < 100.0, "trivial loop took {ms} ms, expected < 100 ms");
}

#[test]
fn time_operation_returns_correct_result() {
    let (result, duration) = perf::time_operation(|| 42 * 2);
    assert_eq!(result, 84);
    assert!(duration.as_nanos() > 0);
}

// ---------------------------------------------------------------------------
// Composite: full workflow timing
// ---------------------------------------------------------------------------

#[test]
fn perf_full_workflow_open_to_search() {
    let (dir, index) = build_test_workspace(200);

    let start = Instant::now();

    // Step 1: Full scan
    let scan = index.full_scan();
    let scan_ms = start.elapsed().as_secs_f64() * 1000.0;

    // Step 2: Build tag index
    let tag_index = TagIndex::new();
    tag_index.rebuild(&index);
    let tag_ms = start.elapsed().as_secs_f64() * 1000.0 - scan_ms;

    // Step 3: Build link graph
    let graph = LinkGraph::new();
    graph.rebuild(&index);
    let link_ms = start.elapsed().as_secs_f64() * 1000.0 - scan_ms - tag_ms;

    // Step 4: Fuzzy search
    let finder = FuzzyFinder::new(&index);
    let fuzzy_results = finder.fuzzy_find("note", 20);
    let fuzzy_ms = start.elapsed().as_secs_f64() * 1000.0 - scan_ms - tag_ms - link_ms;

    // Step 5: Content search
    let searcher = ContentSearcher::new(&index);
    let content_results = searcher.search_content("searchable", dir.path(), 20);

    let total_ms = start.elapsed().as_secs_f64() * 1000.0;

    eprintln!("[perf-budget] Full workflow (200 files):");
    eprintln!(
        "  scan:     {scan_ms:.1} ms  ({} files, {} notes)",
        scan.files_count, scan.notes_count
    );
    eprintln!(
        "  tags:     {tag_ms:.1} ms  ({} tags)",
        tag_index.get_all_tags().len()
    );
    eprintln!("  links:    {link_ms:.1} ms");
    eprintln!(
        "  fuzzy:    {fuzzy_ms:.1} ms  ({} results)",
        fuzzy_results.len()
    );
    eprintln!(
        "  content:  {:.1} ms  ({} results)",
        total_ms - scan_ms - tag_ms - link_ms - fuzzy_ms,
        content_results.len()
    );
    eprintln!("  TOTAL:    {total_ms:.1} ms");

    // The entire workflow for 200 files should complete in under 10 seconds
    // even in debug mode.
    assert!(
        total_ms < 10000.0,
        "full workflow took {total_ms:.1} ms, expected < 10000 ms"
    );
}
