# Epic 1: Backend Foundation (Rust/Tauri)

## Task [1.1]: Add Backend Dependencies and Module Entrypoints
Goal: Establish required crates and module boundaries for FS/index/search/git/links services.
Commands/Code:
```bash
cd src-tauri
cargo add serde --features derive
cargo add serde_json thiserror anyhow git2 walkdir ignore notify crossbeam-channel parking_lot nucleo-matcher
```
Modify files:
- `src-tauri/src/lib.rs`
- `src-tauri/src/core/mod.rs`
- `src-tauri/src/commands/mod.rs`
Acceptance Criteria: `cargo check --manifest-path src-tauri/Cargo.toml` passes.

## Task [1.2]: Define Shared Request/Response Models
Goal: Create typed contracts for frontend IPC and event payloads.
Commands/Code:
Create/modify files:
- `src-tauri/src/models/files.rs`
- `src-tauri/src/models/search.rs`
- `src-tauri/src/models/git.rs`
- `src-tauri/src/models/links.rs`
- `src-tauri/src/models/status.rs`
Acceptance Criteria: All commands and events use explicit typed structs.

## Task [1.3]: Implement Workspace File System Service
Goal: Provide safe local-first file operations constrained to workspace root.
Commands/Code:
Create/modify files:
- `src-tauri/src/core/fs/service.rs`
- `src-tauri/src/commands/files.rs`
Implement command set:
- `open_workspace(root_path)`
- `list_dir(path)`
- `read_file(path)`
- `write_file(path, content)`
- `create_note(path)`
Acceptance Criteria: Access outside workspace is rejected and valid operations succeed.

## Task [1.4]: Build Incremental Indexer and File Watcher
Goal: Keep an in-memory index synced with disk for fast UI updates.
Commands/Code:
Create/modify files:
- `src-tauri/src/core/index/service.rs`
- `src-tauri/src/state/app_state.rs`
- `src-tauri/src/events/index_events.rs`
Acceptance Criteria: Create/edit/delete file actions emit index update events without restart.

## Task [1.5]: Implement Omnibar Fuzzy Search Service
Goal: Return ranked results for floating `Ctrl+P` command palette.
Commands/Code:
Create/modify files:
- `src-tauri/src/core/search/service.rs`
- `src-tauri/src/commands/search.rs`
Implement command:
- `fuzzy_find(query, limit)`
Acceptance Criteria: Repeated queries return ranked matches in low latency for medium workspace size.

## Task [1.6]: Implement Git Diff Worker and Event Stream
Goal: Compute line-level deltas for right-pane gutter decorations.
Commands/Code:
Create/modify files:
- `src-tauri/src/core/git/service.rs`
- `src-tauri/src/events/git_events.rs`
- `src-tauri/src/commands/git.rs`
Implement commands/events:
- `get_git_hunks(path)`
- `subscribe_git_hunks(path)`
Acceptance Criteria: File edits produce added/modified/deleted hunk updates consumable by frontend.

## Task [1.7]: Implement Link Graph and Backlink Resolver
Goal: Support bidirectional links for markdown notes.
Commands/Code:
Create/modify files:
- `src-tauri/src/core/links/parser.rs`
- `src-tauri/src/core/links/service.rs`
- `src-tauri/src/commands/links.rs`
Implement command:
- `get_backlinks(path)`
Acceptance Criteria: Updating `[[wikilinks]]` or markdown links updates backlink results.

## Task [1.8]: Implement Workspace Status Service
Goal: Feed footer status bar with branch, sync state, and note/tag counts.
Commands/Code:
Create/modify files:
- `src-tauri/src/core/git/status.rs`
- `src-tauri/src/core/index/metrics.rs`
- `src-tauri/src/commands/status.rs`
Implement command:
- `workspace_status()`
Acceptance Criteria: Status response includes branch name, sync text, notes count, tags count, and app version.

## Task [1.9]: Build Backend Integration Tests
Goal: Lock core behavior before UI coupling.
Commands/Code:
Create files:
- `src-tauri/tests/fs_commands.rs`
- `src-tauri/tests/index_search.rs`
- `src-tauri/tests/git_worker.rs`
- `src-tauri/tests/links.rs`
- `src-tauri/tests/status.rs`
Run command:
```bash
cargo test --manifest-path src-tauri/Cargo.toml
```
Acceptance Criteria: All integration tests pass consistently.
