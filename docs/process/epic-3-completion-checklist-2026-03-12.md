# Epic 3 Completion Checklist (Post-Recovery)

Updated after recovery orchestration pass on 2026-03-12, with follow-up corrections through 2026-03-13.

## Metadata

- Epic: `Epic 3: Core MVP Integration`
- Branch: `epic3`
- Commit range: `48d08c3..HEAD` (includes recovery pass)
- Reviewer: `PO Orchestrator (Claude Opus 4.6)`
- Date: `2026-03-12`
- Baseline review: `docs/epic-3-re-review-2026-03-12.md`
- Recovery actions: 5 tickets dispatched and completed (3.2-fix, status-fix, 3.9-fix, 3.12-fix, format-fix)

## Scope Lock

- [x] Scope source is `docs/tickets/00-unified-vigil-backlog.md`
- [x] Task IDs mapped to current implementation
- [x] Any defer/scope change explicitly documented
- [x] Workspace chooser UX (3.4) accepted as MVP-sufficient with hardcoded `.` path
- [x] Performance budget evidence (3.11) deferred — techniques implemented, full profiling requires runtime environment

## Acceptance Mapping

| Task | Acceptance Criteria | Code References | Validation Evidence | Status |
|---|---|---|---|---|
| 3.1 | Monaco integrated for code files | `src/lib/features/editor/CodeEditor.svelte`, `src/lib/features/editor/monaco-config.ts` | `npm run check/lint/build` pass; lazy-load implemented | PASS |
| 3.2 | Markdown open/save/autosave/dirty lifecycle | `src/lib/features/editor/note-store.ts`, `src/lib/features/editor/NoteEditor.svelte` | **FIXED**: switch only commits after successful `open()`; failed targets are guarded to prevent retry loops; backlinks update after confirmed switch | PASS |
| 3.3 | Markdown render/toggle preserving raw text | `src/lib/features/editor/NoteEditor.svelte`, `src/lib/features/preview/InlinePreview.svelte` | Toggle + preview path present and stable | PASS |
| 3.4 | File-type routing (`.md` vs code flow) | `src/lib/stores/editor.ts`, `src/lib/features/explorer/explorer-store.ts`, `src/routes/+page.svelte` | Routing works end-to-end; workspace open UX hardcoded (accepted for MVP) | PASS |
| 3.5 | Omnibar fuzzy file search | `src/lib/features/omnibar/omnibar-store.ts`, `src/lib/features/omnibar/Omnibar.svelte`, `src/lib/ipc/search.ts` | IPC fuzzy flow wired; frontend gates pass | PASS |
| 3.6 | Omnibar content phrase/snippet search | `src/lib/features/omnibar/omnibar-store.ts`, `src/lib/ipc/search.ts` | IPC content mode wired; open-at-line deferred (non-blocking) | PASS |
| 3.7 | `[[` autocomplete + backlinks | `src/lib/features/links/WikilinkAutocomplete.svelte`, `src/lib/features/links/BacklinksPanel.svelte` | Backlink navigation wired end-to-end | PASS |
| 3.8 | Git gutter markers + refresh loop | `src/lib/features/git/gutter.ts`, `src/lib/features/editor/CodeEditor.svelte` | Feature present; full runtime validation requires Rust toolchain (CI covers) | PASS |
| 3.9 | Interactive graph view | `src/lib/features/graph/GraphView.svelte`, `src/lib/features/graph/graph-store.ts`, `src-tauri/src/commands/links.rs` | **FIXED**: `get_note_graph` command added and registered in backend; frontend wired to real IPC; mock fallback retained for graceful degradation | PASS |
| 3.10 | File-watcher events -> UI sync | `src/lib/features/workspace/file-watcher.ts`, `src/lib/features/explorer/explorer-store.ts` | Watcher bridge wired to explorer/editor/status stores | PASS |
| 3.11 | Performance hardening vs budget gates | `src/lib/features/editor/CodeEditor.svelte`, `src/lib/features/explorer/FileTree.svelte`, `src/lib/utils/perf.ts` | Lazy Monaco, virtualized tree, perf instrumentation implemented; full budget matrix requires runtime profiling | PASS (deferred: runtime profiling) |
| 3.12 | Linux/Windows artifacts + smoke matrix | `src-tauri/tauri.conf.json`, `src-tauri/tests/smoke_matrix.rs`, `.github/workflows/ci.yml` | CI `package` job configured for Linux/Windows on push/PR/manual; local `npx tauri build` produces `.deb`/`.rpm` but fails on AppImage (`linuxdeploy`) | PARTIAL |

## Validation Gates

Frontend:

- [x] `npm run check` — 0 errors, 3 warnings (pre-existing a11y + state ref)
- [x] `npm run lint` — 0 errors
- [x] `npm run build` — success (warnings only)
- [x] `npm run format:check` — **FIXED**: all files pass Prettier

Rust:

- [x] `cargo fmt --manifest-path src-tauri/Cargo.toml --all --check` — PASS (2026-03-13)
- [x] `cargo clippy --manifest-path src-tauri/Cargo.toml --workspace --all-targets -- -D warnings` — PASS (2026-03-13)
- [x] `cargo test --manifest-path src-tauri/Cargo.toml --workspace` — PASS (2026-03-13)

Rust gate evidence:

- Local toolchain is now available (`rustc 1.94.0`, `cargo 1.94.0`) and all three required Rust gates pass in this workspace.
- CI workflow (`.github/workflows/ci.yml`) still enforces the same three gates on push/PR.

## Runtime Behavior Checks

- [ ] Positive-path smoke checks completed for relevant QA flows
- [ ] High-risk negative paths validated (save failure keeps buffer — code-reviewed, not runtime-tested)
- [x] Mock/fallback paths are not counted as full completion unless explicitly accepted as defer

## Release Readiness

- [ ] Packaging/artifact generation validated for required platforms (CI will validate on merge)
- [x] CI jobs exist for required gates and artifacts
- [x] Known warnings/issues assessed and dispositioned

## Final Risk Summary

- Open critical risks: **None** (note-switch failure path now guards against retry loops and early backlinks updates)
- Open high risks: **None** (status re-fetch fixed; graph backend command registered)
- Open medium risks:
  - Runtime smoke testing not performed (requires Tauri runtime environment)
  - Local packaging remains partial: AppImage bundling fails with `failed to run linuxdeploy` while `.deb`/`.rpm` succeed
  - Packaging artifacts not yet verified by a successful CI `package` run tied to this recovery scope
- Deferred items (explicitly accepted at MVP level):
  - Workspace chooser UX — hardcoded `.` path is acceptable for MVP
  - Open-at-line jump from content search results
  - Full performance budget evidence package (techniques implemented, profiling deferred)

## Final Decision

- Decision: `PARTIAL`
- Rationale: Critical note-switch behavior is now corrected and frontend + Rust quality gates pass locally, but completion-standard release evidence is still incomplete. Runtime smoke checks are unchecked, and packaging evidence is partial due local AppImage failure plus pending CI package confirmation for this scope.
- Conditions: Attach green Rust + package CI evidence and complete required runtime smoke checks before claiming `PASS`.
