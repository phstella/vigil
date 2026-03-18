# QA Test Matrix

## Platforms
- Linux (Ubuntu LTS, x86_64)
- Windows 11 (x86_64)

## Build Modes
- Development (`npx tauri dev`)
- Release (`npx tauri build`)

## Core Smoke Matrix (MVP)

| Test ID | Flow | Linux | Windows | Pass Criteria |
|---|---|---|---|---|
| QA-001 | Launch app | Required | Required | Main shell interactive within budget |
| QA-002 | Open workspace | Required | Required | Explorer tree loads and is navigable |
| QA-003 | Read/write markdown | Required | Required | Save persists and reload confirms content |
| QA-004 | Open code file | Required | Required | Right pane editor opens and edits correctly |
| QA-005 | Ctrl+P fuzzy file search | Required | Required | Ranked results and open-on-enter works |
| QA-006 | Ctrl+P content search | Required | Required | Phrase/snippet results appear within target latency |
| QA-007 | Git gutter markers | Required | Required | Added/modified/deleted markers reflect file diff |
| QA-008 | Backlinks + `[[` linking | Required | Required | Link updates reflected in backlinks list |
| QA-009 | Status bar live values | Required | Required | Branch/sync/count/version values display correctly |
| QA-010 | Keyboard shortcuts | Required | Required | Ctrl+P/Ctrl+S/Ctrl+B/Ctrl+N behave consistently |
| QA-011 | Packaging | Required | Required | Installer/binary launches and runs smoke suite |
| QA-012 | Linux Monaco load stability (dev + prod) | Required | N/A | `npx tauri dev` and `npx tauri build` + open code file does not reset/crash; no repeated `WebLoaderStrategy::internallyFailedLoadTimerFired` loop; no repeated Monaco `Missing requestHandler` worker errors; inline workers used on all Linux Tauri builds; transient failures auto-retry up to 2 times with exponential backoff; user-facing retry button shown if all retries exhausted |
| QA-013 | Markdown Mermaid render | Required | Required | ` ```mermaid ` fenced blocks render in preview mode; invalid diagrams show fallback without crashing |

## Expansion Matrix (Epic 4)

| Test ID | Flow | Linux | Windows | Pass Criteria |
|---|---|---|---|---|
| QA-101 | Advanced content search grammar | Required | Required | Scoped/operator queries return expected ranked results |
| QA-102 | Graph view open/select/filter | Required | Required | Selecting graph node opens note; filters behave correctly |
| QA-103 | Markdown WYSIWYG toggle hardening | Required | Required | Toggle keeps cursor state and content integrity |
| QA-104 | Local plugin lifecycle | Required | Required | Add, enable, disable, remove plugin works from local source |
| QA-105 | Plugin capability prompt/enforcement | Required | Required | Requested capabilities shown and enforced |
| QA-106 | Plugin runtime isolation | Required | Required | Faulted plugin is isolated without crashing host |

## CI Quality Gates (Automated)

All of the following are enforced by `.github/workflows/ci.yml` on every push to main/master and on every pull request:

| Gate | Job | Command | Blocks merge on failure |
|---|---|---|---|
| Type checking | Frontend | `npm run check` | Yes |
| ESLint | Frontend | `npm run lint` | Yes |
| Prettier | Frontend | `npm run format:check` | Yes |
| Frontend build | Frontend | `npm run build` | Yes |
| Rust formatting | Rust | `cargo fmt --check` | Yes |
| Clippy lint | Rust | `cargo clippy -- -D warnings` | Yes |
| Rust tests | Rust | `cargo test` | Yes |

## Backend Integration Test Coverage (Epic 1)

The following cross-service integration tests run in CI via `cargo test`. They
exercise end-to-end workflows without a Tauri runtime, using temporary
workspaces with realistic content.

| Test file | Test name | Services exercised | QA flow covered |
|---|---|---|---|
| `integration_workflows.rs` | `workflow_open_index_fuzzy_find` | WorkspaceFs, FileIndex, FuzzyFinder | QA-002, QA-005 |
| `integration_workflows.rs` | `workflow_write_file_incremental_index_search` | WorkspaceFs, FileIndex, ContentSearcher, FuzzyFinder | QA-003, QA-005, QA-006 |
| `integration_workflows.rs` | `workflow_tag_index_correct_counts` | FileIndex, TagIndex | QA-009 |
| `integration_workflows.rs` | `workflow_link_graph_backlinks_resolve` | FileIndex, LinkGraph | QA-008 |
| `integration_workflows.rs` | `workflow_workspace_status_assembly` | WorkspaceFs, FileIndex, TagIndex, metrics | QA-009 |
| `integration_workflows.rs` | `workflow_file_lifecycle_index_consistency` | WorkspaceFs, FileIndex, ContentSearcher | QA-003 |
| `integration_workflows.rs` | `workflow_tag_link_coherence_after_edit` | WorkspaceFs, FileIndex, TagIndex, LinkGraph | QA-003, QA-008, QA-009 |
| `integration_workflows.rs` | `workflow_content_search_cross_directory` | FileIndex, ContentSearcher | QA-006 |
| `integration_workflows.rs` | `workflow_git_status_after_workspace_changes` | WorkspaceFs, FileIndex, GitService, ContentSearcher | QA-003, QA-006, QA-007 |

## Performance Budget Tests (Epic 1)

Performance budget validation tests in `perf_budget.rs` measure hot-path
latency against generous debug-build thresholds (10-20x release budget).
They print timing data during `cargo test` for manual inspection.

| Test name | Operation | Budget (release) | Threshold (debug) |
|---|---|---|---|
| `perf_full_scan_500_files` | FileIndex::full_scan | 1500 ms / 10k files | 5000 ms / 500 files |
| `perf_fuzzy_find_latency` | FuzzyFinder::fuzzy_find | 80 ms | 1600 ms median |
| `perf_content_search_latency` | ContentSearcher::search_content | 150 ms median | 3000 ms median |
| `perf_tag_index_rebuild` | TagIndex::rebuild | (no explicit budget) | 500 ms |
| `perf_link_graph_rebuild` | LinkGraph::rebuild | (no explicit budget) | 5000 ms |
| `perf_git_hunks_latency` | GitService::get_hunks | 200 ms | 2000 ms |
| `perf_full_workflow_open_to_search` | Full pipeline (scan+tags+links+search) | (composite) | 10000 ms |

## Regression Checklist per PR
- CI quality gates pass (automated, see table above).
- Lint/format checks pass.
- Rust tests pass.
- At least QA-001 through QA-007 manually verified for UI-affecting PRs.
- Performance spot check for omnibar and typing latency.

## Release Gate
Release candidate is blocked if any of these fail:
- Any required smoke test.
- Any security test for plugin capability isolation.
- Performance budgets exceeded by > 10% at p95.

## QA-012 Environment Baseline (Task 3.5.3)

The following environment was used to reproduce and validate the Linux Monaco crash fix:

| Field | Value |
|---|---|
| Distro | Fedora KDE |
| Compositor | Hyprland (Wayland) |
| GPU | Intel ADL GT2 |
| Driver | Mesa 25.3.6 (OpenGL 4.6, direct rendering: Yes) |
| WebKitGTK | libwebkit2gtk-4.1 (Tauri 2.x default) |
| Tauri | 2.x (see `src-tauri/Cargo.toml`) |
| Monaco | 0.55.x (see `package.json`) |

**Original crash trace:**
```
ERROR: WebKit encountered an internal error. This is a WebKit bug.
WebLoaderStrategy.cpp(618) : internallyFailedLoadTimerFired()
```

**Root cause:** WebKitGTK intermittently fails loading web worker blob/URL resources. Monaco workers constructed via `new Worker(url)` trigger fetch requests that WebKitGTK's internal loader can time out on, leading to cascading failures.

**Mitigation applied (Task 3.5.3):**
1. Inline workers (`?worker&inline`) used on all Linux Tauri builds (not just dev mode).
2. Worker construction failures are transient-safe: consecutive failure counter replaces permanent disable flag.
3. CodeEditor auto-retries Monaco load up to 2 times with exponential backoff (500ms, 1000ms).
4. User-facing "Retry" button shown if all auto-retries are exhausted.

**Trade-off:** Inline workers increase the Linux bundle by ~200-400 KB (worker JS embedded as data-URIs). This is acceptable given the alternative is a hard crash. Windows and macOS builds are unaffected; they continue to use standard URL-based workers.

## Evidence Collection
For each executed test, record:
- build commit hash
- OS/version
- test result (`PASS`/`FAIL`)
- short notes and screenshot/log links
