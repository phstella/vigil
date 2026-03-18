# Epic 3 Recovery Handoff

## Objective

Bring Epic 3 (Core MVP Integration) to implementation sufficiency while recording remaining release gates accurately.

## Scope Source

- Canonical backlog: `docs/tickets/00-unified-vigil-backlog.md`
- Task IDs in scope: 3.1–3.12
- Recovery baseline: `docs/epic-3-re-review-2026-03-12.md`

## Current Status

- Overall status: `PARTIAL`
- Branch: `epic3`
- Recovery date: 2026-03-12

## Recovery Actions Completed

### 1. Critical: Note-switch failure path and identity mismatch (3.2-fix) — RESOLVED

- **Problem**: Failed note switches could leave UI file identity ahead of the actually loaded note.
- **Fix**: Removed fallback load behavior, blocked repeated retries for unchanged failed targets, rolled editor identity back to the loaded note on failure, and only updated backlinks after a confirmed switch.
- **Evidence**: `npm run check/lint/build` all pass. Code review confirms no force-load on `false`.
- **Code references**: `src/lib/features/editor/NoteEditor.svelte:53-97`, `src/lib/stores/editor.ts:150-172`
- **Diary**: `docs/implementation-diary/3.2-fix.md`

### 2. High: Status bar mock/stale after workspace open (status-fix) — RESOLVED

- **Problem**: `statusStore.initialize()` ran before `openAndLoadWorkspace()` and never retried after workspace opened.
- **Fix**: Chained `statusStore.initialize()` after `openAndLoadWorkspace('.')` succeeds. Initial call preserved for fast-path.
- **Evidence**: `npm run check/lint/build` all pass.
- **Code references**: `src/routes/+page.svelte:196-204`
- **Diary**: `docs/implementation-diary/status-fix.md`

### 3. High: Graph view demo-data only (3.9-fix) — RESOLVED

- **Problem**: `get_note_graph` Tauri command not defined/registered despite backend `LinkGraph::get_graph()` existing.
- **Fix**: Added `get_note_graph` command in `commands/links.rs` and registered in `lib.rs`. Updated graph-store comment.
- **Evidence**: Frontend + Rust gates pass locally.
- **Code references**: `src-tauri/src/commands/links.rs:35-44`, `src-tauri/src/lib.rs:27`
- **Diary**: `docs/implementation-diary/3.9-fix.md`

### 4. Medium: CI packaging jobs missing (3.12-fix) — PARTIAL

- **Problem**: No `tauri build` jobs in CI for Linux/Windows artifacts.
- **Fix**: Added `package` job with Linux/Windows matrix and artifact upload; trigger now supports push/PR/manual runs. Added Linux-specific Tauri config to target `.deb`/`.rpm` and defer AppImage.
- **Evidence**: YAML validated; local `npx tauri build` now succeeds for Linux `.deb`/`.rpm` artifacts.
- **Code references**: `.github/workflows/ci.yml:95-149`
- **Diary**: `docs/implementation-diary/3.12.md`

### 5. Medium: Formatting gate failing (format-fix) — RESOLVED

- **Problem**: `npm run format:check` failed on 25+ files.
- **Fix**: Ran `npm run format` — 28 files auto-formatted.
- **Evidence**: `npm run format:check` now returns "All matched files use Prettier code style!"
- **Diary**: `docs/implementation-diary/format-fix.md`

## Validation Evidence

| Command | Result |
|---------|--------|
| `npm run check` | PASS (0 errors, 3 pre-existing warnings) |
| `npm run lint` | PASS (0 errors) |
| `npm run build` | PASS |
| `npm run format:check` | PASS |
| `cargo fmt --check` | PASS |
| `cargo clippy` | PASS |
| `cargo test` | PASS |

## Task Matrix

| Task | Status | Evidence Summary |
|---|---|---|
| 3.1 Monaco integration | PASS | Lazy-load, theme, language mapping |
| 3.2 Markdown lifecycle | PASS | Save/autosave/dirty + data-loss fix |
| 3.3 Markdown render/toggle | PASS | Toggle + preview preserving raw text |
| 3.4 File-type routing | PASS | .md vs code routing works; workspace open UX accepted for MVP |
| 3.5 Omnibar fuzzy search | PASS | IPC fuzzy path wired |
| 3.6 Omnibar content search | PASS | IPC content search wired |
| 3.7 Linking + backlinks | PASS | Autocomplete + backlinks + click navigation |
| 3.8 Git gutter markers | PASS | Decorations + refresh loop; runtime via CI |
| 3.9 Graph view | PASS | Backend command registered; UI interactive |
| 3.10 Watcher UI sync | PASS | Backend events bridged to stores, including `vigil://fs-renamed` from `rename_file` |
| 3.11 Performance hardening | PASS | Techniques implemented; profiling deferred |
| 3.12 Build artifacts | PARTIAL | CI packaging configured; local Linux `.deb`/`.rpm` pass with explicit AppImage defer; CI package evidence pending |

## Next Actions (Ordered)

1. Verify CI `package` job produces Linux/Windows artifacts on this scope.
2. Keep AppImage deferred for MVP unless a release requirement explicitly restores it.
3. Perform runtime smoke testing with Tauri desktop app before production release.
4. (Post-MVP) Add workspace chooser UX, open-at-line jump, full performance profiling.

## Done Criteria for Next Agent

1. `package` job successfully builds and uploads artifacts for Linux and Windows.
2. Runtime smoke test of note editing (open, edit, switch, save) confirms no data loss.
