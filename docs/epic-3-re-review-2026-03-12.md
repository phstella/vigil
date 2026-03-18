# Epic 3 Re-Review (Post-Fix Pass)

Review date: March 12, 2026  
Branch reviewed: `epic3`  
Context: Re-audit after reported recovery fixes

## Executive Verdict

Epic 3 is improved but still **not sufficient for full sign-off**.

The latest recovery commits fixed several important blockers (workspace lifecycle wiring, explorer IPC tree, code save path, backlinks navigation, watcher bridge), but there are still critical/high issues that can impact correctness and release readiness.

## Validation Results

- `npm run check`: PASS (0 errors, 2 GraphView a11y warnings)
- `npm run lint`: PASS
- `npm run build`: PASS (warnings only)
- `npm run format:check`: FAIL (25 files not Prettier-formatted)
- Rust checks (`cargo test`, `cargo clippy`, `cargo fmt --check`): NOT RUN locally (`cargo`/`rustc` unavailable in this environment)

## What Was Fixed Since Prior Review

1. Workspace lifecycle now wired to real IPC open + explorer load.
   - `src/lib/features/workspace/workspace-lifecycle.ts`
   - `src/routes/+page.svelte`
2. Explorer moved from static mock tree to IPC-backed `list_dir`/`read_file`.
   - `src/lib/features/explorer/explorer-store.ts`
   - `src/lib/features/explorer/ExplorerPanel.svelte`
3. `Ctrl+S` for code files now writes to disk via `write_file`.
   - `src/routes/+page.svelte`
4. Backlink click navigation now opens target files.
   - `src/lib/features/editor/NoteEditor.svelte`
5. File-watcher updates are bridged to the visible explorer store.
   - `src/lib/features/workspace/file-watcher.ts`

## Remaining Findings (Severity Ordered)

### 1) Critical: save-before-switch protection is still bypassed in note flow

`noteStore.open()` correctly returns `false` when pre-switch save fails, but `NoteEditor` still force-loads the target file on any `false` result, which bypasses the safety guard and can discard unsaved content.

- `src/lib/features/editor/note-store.ts`
- `src/lib/features/editor/NoteEditor.svelte`

### 2) High: status bar can remain mock/non-live due startup ordering

`statusStore.initialize()` runs before workspace open. If it fails once (common before `open_workspace`), it falls back to mock status and there is no guaranteed re-fetch after workspace opens.

- `src/lib/features/status/status-store.ts`
- `src/routes/+page.svelte`
- `src-tauri/src/commands/status.rs`

### 3) High: graph view remains demo-data backed

Frontend explicitly uses mock fallback because `get_note_graph` is still not registered in backend invoke handlers.

- `src/lib/features/graph/graph-store.ts`
- `src/lib/features/graph/GraphView.svelte`
- `src-tauri/src/lib.rs`

### 4) Medium: workspace open UX is still hardcoded

App startup attempts `openAndLoadWorkspace('.')`; there is still no real user-facing workspace chooser/selection flow.

- `src/routes/+page.svelte`

### 5) Medium: documentation remains inconsistent for Epic 3 scope

Canonical backlog (3.1–3.12) conflicts with older Epic 3 ticket and `vigil-plan` mapping; IPC/spec docs still label some used capabilities as Epic 4.

- `docs/tickets/00-unified-vigil-backlog.md`
- `docs/tickets/epic-3-core-features-integration.md`
- `vigil-plan/epics/epic-3-integration/README.md`
- `docs/specs/ipc-contracts.md`

### 6) Medium: packaging release gate still incomplete

No CI jobs currently run `npx tauri build` for Linux/Windows artifacts.

- `.github/workflows/ci.yml`

### 7) Medium: formatting gate currently failing

Prettier check fails on 25 files, which is a quality gate issue if enforced in CI/PR policy.

## Epic 3 Task Sufficiency (Canonical 3.1-3.12)

| Task | Status | Assessment |
|---|---|---|
| 3.1 Monaco integration | Mostly sufficient | Lazy Monaco/theme/language mapping present |
| 3.2 Markdown lifecycle | Partial | Core save/autosave present, but switch-save safety still bypassed by fallback flow |
| 3.3 Markdown render/toggle | Sufficient (scope-limited) | Toggle + preview behavior present |
| 3.4 File-type routing | Mostly sufficient | Routing works with IPC-backed explorer; workspace-open UX still hardcoded |
| 3.5 Omnibar fuzzy search | Mostly sufficient | IPC fuzzy path wired and debounced |
| 3.6 Omnibar content search | Mostly sufficient | IPC content search wired; line-jump behavior still deferred |
| 3.7 Linking + backlinks | Mostly sufficient | `[[` autocomplete + backlinks + click navigation wired |
| 3.8 Git gutter markers | Mostly sufficient | Decorations + refresh loop in place; full backend/rust validation pending |
| 3.9 Graph view | Partial | Interactive UI works but backend graph command absent; demo fallback used |
| 3.10 Watcher UI sync | Mostly sufficient | Backend events bridged into visible explorer/editor/status paths |
| 3.11 Performance hardening | Partial | Optimizations implemented; no complete budget evidence package |
| 3.12 Build artifacts | Partial | Config + smoke tests exist, but no local Rust/toolchain proof and no packaging CI jobs |

## Release Blockers

1. Fix note-switch data-loss path (`open() == false` must not force-load target file).
2. Ensure status store re-fetches after successful workspace open.
3. Decide graph scope explicitly:
   - implement/register `get_note_graph`, or
   - formally defer and downgrade Epic 3 completion claim.
4. Add artifact CI jobs (`tauri build`) for Linux and Windows.
5. Resolve formatting gate failures.

## Recommended Handoff Priorities

1. Patch `NoteEditor`/`noteStore` integration so failed pre-switch save keeps current buffer untouched.
2. Trigger `statusStore.initialize()` (or direct `workspace_status` fetch) after `openAndLoadWorkspace()` succeeds.
3. Align graph deliverable with runtime reality (live backend vs explicit defer).
4. Add packaging workflows and artifact upload in CI.
5. Normalize Epic 3 docs to one authoritative mapping and update implementation diary claims accordingly.
