# Epic 3 Implementation Sufficiency Review

Review date: March 12, 2026  
Branch reviewed: `epic3`

## Scope

Deep implementation review of Epic 3 against repository documentation and acceptance criteria, including:

- `docs/tickets/00-unified-vigil-backlog.md` (canonical source of truth)
- `docs/tickets/epic-3-core-features-integration.md`
- `vigil-plan/epics/epic-3-integration/*`
- frontend and backend code paths involved in tasks 3.1-3.12
- local quality gates executable in this environment

## Executive Verdict

Epic 3 is **not sufficient for MVP sign-off yet**.

There is substantial progress and many implemented pieces, but several critical integration paths are still mocked or incomplete, and core end-to-end flows are not reliably functional.

## Validation Results

- `npm run check`: PASS (0 errors, 2 GraphView a11y warnings)
- `npm run lint`: PASS
- `npm run build`: PASS
- Rust checks (`cargo test`, `cargo clippy`, `cargo fmt --check`): NOT RUN locally (Rust toolchain unavailable in this environment)

## Findings (Severity Ordered)

### 1. Critical: No real workspace-open flow in frontend

Many IPC-backed features depend on an opened workspace, but frontend flow remains mock-driven.

- Explorer still uses mock tree and mock open-file content
  - `src/lib/features/explorer/explorer-store.ts`
- `openWorkspace` IPC exists but is not wired into app flow
  - `src/lib/ipc/files.ts`
- Backend commands require open workspace/index
  - `src-tauri/src/commands/fs.rs`
  - `src-tauri/src/commands/search.rs`
  - `src-tauri/src/commands/status.rs`

Impact: core flows (open/read/write/search/status/git/backlinks) cannot be guaranteed end-to-end in real usage.

### 2. Critical: Explorer mock paths violate IPC path contract

Mock paths are absolute (`/workspace/...`), while IPC requires workspace-relative paths.

- Mock paths:
  - `src/lib/features/explorer/explorer-store.ts`
- Contract:
  - `docs/specs/ipc-contracts.md` (paths must be workspace-relative)
- Backend path confinement:
  - `src-tauri/src/core/fs/service.rs`

Impact: if these paths hit IPC, behavior will diverge from real contract and can fail with path validation errors.

### 3. High: Unsaved note changes can be lost during file switch

`noteStore.open()` attempts save before switching but does not block switching when save fails.

- `src/lib/features/editor/note-store.ts`

Impact: failed save can still be followed by buffer replacement for another file.

### 4. High: NoteEditor fallback logic is ineffective

NoteEditor expects `noteStore.open()` to reject, but `open()` catches errors and returns `false`, so `.catch(...)` fallback never executes.

- `src/lib/features/editor/NoteEditor.svelte`
- `src/lib/features/editor/note-store.ts`

Impact: intended fallback path is non-functional.

### 5. High: Code save path is not implemented

`Ctrl+S` for code files only logs and marks dirty state clean; no write IPC occurs.

- `src/routes/+page.svelte`

Impact: user can think code is saved when it is not.

### 6. High: Graph view not backed by real backend command

Frontend calls `get_note_graph`, but backend command is not registered/implemented for current MVP runtime.

- frontend wrapper:
  - `src/lib/ipc/links.ts`
- backend command registration:
  - `src-tauri/src/lib.rs` (no `get_note_graph`)
- graph store fallback to mock data:
  - `src/lib/features/graph/graph-store.ts`

Impact: graph can appear functional while showing fallback/mock data.

### 7. Medium: File-watcher UI sync is partial

Watcher handlers update `filesStore`, but visible explorer renders `explorerStore` mock tree.

- watcher integration:
  - `src/lib/features/workspace/file-watcher.ts`
- rendered explorer source:
  - `src/lib/features/explorer/ExplorerPanel.svelte`
  - `src/lib/features/explorer/explorer-store.ts`

Impact: event-driven updates are not fully reflected in visible explorer UI.

### 8. Medium: Backlink click navigation is still TODO

Backlinks panel emits clicks, but navigation handler is placeholder.

- `src/lib/features/editor/NoteEditor.svelte`

Impact: backlinks are visible but not fully actionable.

### 9. Medium: Status bar bootstraps from mock data and no initial status fetch

Status store initializes with `MOCK_STATUS` and relies on watcher events; no explicit initial `workspace_status` fetch on startup path.

- `src/lib/features/status/status-store.ts`
- `src/lib/ipc/status.ts`

Impact: correctness depends on event timing and workspace lifecycle wiring.

### 10. Medium: Epic 3 documentation is internally inconsistent

There are multiple, conflicting task mappings:

- canonical source says 3.1-3.12:
  - `docs/tickets/00-unified-vigil-backlog.md`
- ticket doc still has older 3.1-3.9 mapping:
  - `docs/tickets/epic-3-core-features-integration.md`
- vigil-plan has separate mapping/assumptions:
  - `vigil-plan/epics/epic-3-integration/README.md`

Impact: completion claims vary based on which source is used.

## Task-by-Task Sufficiency (Canonical 3.1-3.12)

| Task | Status | Assessment |
|---|---|---|
| 3.1 Monaco integration | Mostly sufficient | Monaco loads, themed, lazy-loaded, language mapping present |
| 3.2 Markdown lifecycle | Partial | Save/autosave exists, but switch-save failure handling and fallback bug remain |
| 3.3 Preview/WYSIWYG toggle | Sufficient (current scope) | Toggle and renderer in place |
| 3.4 File-type routing | Partial | Routing exists; explorer/open flow still mock and contract-misaligned |
| 3.5 Omnibar fuzzy search | Partial | Logic exists; real behavior depends on workspace-open flow |
| 3.6 Omnibar content search | Partial | Logic exists; same workspace/index dependency caveat |
| 3.7 Backlinks + `[[` | Partial | UI/search present; backlink navigation not wired |
| 3.8 Git gutter | Partial | Decorations and refresh loop exist; code save path not real |
| 3.9 Graph view | Partial | Interactions exist; backend graph command missing, fallback mock used |
| 3.10 Watcher UI sync | Partial | Event wiring exists; not fully connected to visible explorer model |
| 3.11 Performance hardening | Partial | Implemented techniques exist; no full budget evidence package |
| 3.12 Build artifacts | Partial | Config and smoke tests added; no local Rust/toolchain build proof, no CI packaging job yet |

## Release-Risk Summary

Current blockers for claiming Epic 3 “complete”:

1. Real workspace lifecycle + explorer data source integration
2. Correct file path contract adherence (workspace-relative throughout UI)
3. Real code save implementation (`Ctrl+S`)
4. Real graph backend command integration or explicit scope rollback
5. Resolve note-switch data-loss risk
6. Align docs so one Epic 3 definition is authoritative in practice

## Recommended Handoff Focus for Next Agent

Suggested execution order:

1. Replace mock explorer/open flow with IPC-driven workspace + `list_dir`/`read_file`
2. Implement robust save-before-switch guard in `note-store`
3. Implement code save via `write_file` and proper dirty-state transitions
4. Decide graph scope: implement `get_note_graph` now or formally defer and remove “PASS” claim
5. Connect watcher updates to whichever store actually renders the explorer
6. Normalize Epic 3 docs to canonical backlog mapping and update diary claims accordingly
