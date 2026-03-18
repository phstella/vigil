# Epic 3 Agent Brief (1-Page)

Date: March 12, 2026  
Source review: `docs/epic-3-implementation-sufficiency-review-2026-03-12.md`

## Objective

Close Epic 3 MVP gaps so core flows are truly end-to-end, not mock-backed.

## Current Status

Epic 3 is **partial**, not release-ready.

- Frontend checks pass locally (`check`, `lint`, `build`)
- Rust checks/build not runnable locally in this environment (no Rust toolchain)
- Several critical flows still depend on mock data or incomplete wiring

## Top Blockers (Must Fix)

1. No real workspace-open lifecycle wired in frontend
2. Explorer still mock-based; paths are not consistently workspace-relative
3. Code save (`Ctrl+S`) is not implemented (marks clean without writing)
4. Note switch can lose unsaved changes if pre-switch save fails
5. Graph UI depends on `get_note_graph`, but runtime backend command is missing
6. Watcher updates are applied to `filesStore`, while visible explorer uses `explorerStore`

## Execution Order (Recommended)

1. Replace explorer mock flow with IPC-backed workspace + `list_dir` + `read_file`
2. Enforce workspace-relative paths end-to-end in frontend state and UI events
3. Implement real code save via `write_file` and correct dirty-state transitions
4. Fix note switch safety: do not switch file on failed save; surface conflict/error path
5. Unify explorer data source: either migrate UI to `filesStore` or bridge stores properly
6. Resolve graph scope:
   - Option A: implement/register backend `get_note_graph` now
   - Option B: explicitly defer graph data to Epic 4 and adjust Epic 3 completion claim/docs

## Definition of Done for This Recovery

Epic 3 can be considered sufficient when all are true:

1. Open workspace -> browse files -> open note/code -> edit -> save works against real IPC
2. `Ctrl+S` persists both note and code edits to disk (no fake clean state)
3. Omnibar file/content search works from real indexed workspace
4. Backlinks refresh from real data and navigation behavior is defined (implemented or explicitly deferred with acceptance)
5. Graph behavior is aligned with actual backend capability (live or formally deferred)
6. Watcher-driven changes visibly update the explorer/editor/status path actually used by UI
7. Rust CI gates pass (`fmt`, `clippy -D warnings`, `test`) and artifact jobs exist for Linux/Windows packaging

## Fast Validation Checklist

- Run frontend: `npm run check && npm run lint && npm run build`
- Run Rust (in toolchain-enabled env):
  - `cargo fmt --manifest-path src-tauri/Cargo.toml --all --check`
  - `cargo clippy --manifest-path src-tauri/Cargo.toml --workspace --all-targets -- -D warnings`
  - `cargo test --manifest-path src-tauri/Cargo.toml --workspace`
- Manually verify QA-001 to QA-011 core smoke flows from `docs/qa/test-matrix.md`

## Key References

- Canonical backlog: `docs/tickets/00-unified-vigil-backlog.md`
- Epic 3 ticket doc: `docs/tickets/epic-3-core-features-integration.md`
- QA matrix: `docs/qa/test-matrix.md`
- Full deep review: `docs/epic-3-implementation-sufficiency-review-2026-03-12.md`
