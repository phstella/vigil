# Epic 3: Core Features Integration

> Status: Legacy planning document (pre-unified backlog rebaseline).
>
> Canonical Epic 3 scope and task IDs are maintained in:
> - `docs/tickets/00-unified-vigil-backlog.md` (source of truth)
> - `docs/tickets/00-mvp-execution-order.md` (execution order)
>
> Use those two documents for closeout/sign-off decisions. This file is retained for historical context only.

## Task [3.1]: Integrate Monaco in Right Code Pane
Goal: Deliver a performant code-editing surface with minimal configuration overhead.
Commands/Code:
```bash
npm install monaco-editor
```
Create/modify files:
- `src/lib/features/editor/CodeEditor.svelte`
- `src/lib/features/editor/monaco-config.ts`
Acceptance Criteria: Code files open in Monaco and typing remains smooth.

## Task [3.2]: Integrate Markdown Editing Lifecycle in Center Pane
Goal: Connect note open/save/autosave flow to backend file service.
Commands/Code:
Create/modify files:
- `src/lib/features/editor/NoteEditor.svelte`
- `src/lib/features/editor/note-store.ts`
- `src/lib/ipc/files.ts`
Acceptance Criteria: Note edits persist to disk and dirty state clears after save.

## Task [3.3]: Connect Explorer Selection to Dual-Pane Routing
Goal: Open markdown files in center pane and code files in right pane.
Commands/Code:
Create/modify files:
- `src/lib/features/explorer/explorer-store.ts`
- `src/lib/features/workspace/workspace-router.ts`
- `src/lib/features/editor/{NoteEditor.svelte,CodeEditor.svelte}`
Acceptance Criteria: Selecting `.md` targets center pane and selecting code targets right pane.

## Task [3.4]: Connect Ctrl+P Omnibar to Rust Fuzzy Search
Goal: Replace mock omnibar rows with live ranked workspace results.
Commands/Code:
Create/modify files:
- `src/lib/features/omnibar/omnibar-store.ts`
- `src/lib/features/omnibar/Omnibar.svelte`
- `src/lib/ipc/search.ts`
Acceptance Criteria: Query updates results in real time and Enter opens the selected file.

## Task [3.5]: Implement Bidirectional Linking UI + Backend Integration
Goal: Show backlinks for active markdown note in the center workspace.
Commands/Code:
Create/modify files:
- `src/lib/features/links/BacklinksPanel.svelte`
- `src/lib/features/links/links-store.ts`
- `src/lib/ipc/links.ts`
Acceptance Criteria: Backlinks update when link syntax is added or removed.

## Task [3.6]: Implement Git Gutter Decorations in Code Pane
Goal: Surface added/modified/deleted lines directly inside Monaco gutter.
Commands/Code:
Create/modify files:
- `src/lib/features/git/gutter.ts`
- `src/lib/features/editor/CodeEditor.svelte`
- `src/lib/ipc/git.ts`
Acceptance Criteria: Hunk markers refresh on edit/save and match git diff output.

## Task [3.7]: Bind Footer Status Bar to Live Backend Metrics
Goal: Replace placeholder footer values with actual workspace state.
Commands/Code:
Create/modify files:
- `src/lib/features/status/status-store.ts`
- `src/lib/features/status/StatusBar.svelte`
- `src/lib/ipc/status.ts`
Acceptance Criteria: Branch/sync/notes/tags/version reflect backend responses and update on workspace changes.

## Task [3.8]: Execute Performance Hardening Pass
Goal: Preserve performance-first behavior under realistic workloads.
Commands/Code:
Create/modify files:
- `src/lib/features/editor/CodeEditor.svelte` (lazy load Monaco)
- `src/lib/features/omnibar/omnibar-store.ts` (debounce)
- `src/lib/features/explorer/FileTree.svelte` (virtualized tree)
Run checks:
```bash
npm run build
cargo test --manifest-path src-tauri/Cargo.toml
```
Acceptance Criteria: No visible input lag in normal usage and all checks pass.

## Task [3.9]: Build Cross-Platform MVP Artifacts
Goal: Generate Linux and Windows installers for MVP validation.
Commands/Code:
```bash
npx tauri build
```
Validate install/build outputs:
- Linux: AppImage or DEB
- Windows: NSIS or MSI
Acceptance Criteria: Built app launches and core flows (open/edit/save/omnibar/backlinks/git gutter/status) work end-to-end.
