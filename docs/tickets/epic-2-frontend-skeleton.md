# Epic 2: Frontend Skeleton (Svelte/Tailwind)

## Task [2.1]: Build Workspace Grid Shell from Mock
Goal: Create the 4-zone desktop layout (rail, explorer, note pane, code pane) plus footer.
Commands/Code:
Create/modify files:
- `src/routes/+layout.svelte`
- `src/routes/+page.svelte`
- `src/lib/components/layout/AppShell.svelte`
- `src/lib/components/layout/WorkspaceGrid.svelte`
Acceptance Criteria: Layout visually matches mock proportions at desktop resolution.

## Task [2.2]: Implement Primary Left Icon Rail
Goal: Reproduce narrow navigation rail with active-state icons.
Commands/Code:
Create/modify files:
- `src/lib/components/chrome/PrimaryRail.svelte`
- `src/lib/stores/ui.ts`
Acceptance Criteria: Rail renders icon set with clear active section highlight and click switching.

## Task [2.3]: Implement Explorer Sidebar Panel
Goal: Build notes/collections panel with item counts and selectable items.
Commands/Code:
Create/modify files:
- `src/lib/features/explorer/Sidebar.svelte`
- `src/lib/features/explorer/FileTree.svelte`
- `src/lib/features/explorer/explorer-store.ts`
Acceptance Criteria: Collections, counts, and selected note state render as in mock.

## Task [2.4]: Implement Center Markdown Note Pane Skeleton
Goal: Build note canvas with large title and stylized quote/highlight blocks.
Commands/Code:
Create/modify files:
- `src/lib/features/editor/NoteEditor.svelte`
- `src/lib/features/editor/note-store.ts`
Acceptance Criteria: Pane shows title, body text, and accent line blocks matching mock hierarchy.

## Task [2.5]: Implement Right Code Pane Skeleton
Goal: Build secondary editor panel for code files with independent selection state.
Commands/Code:
Create/modify files:
- `src/lib/features/editor/CodeEditor.svelte`
- `src/lib/features/editor/code-store.ts`
Acceptance Criteria: Right pane renders code-focused surface and file label independent from note pane.

## Task [2.6]: Implement Floating Omnibar Overlay UI
Goal: Add centered command palette shell with input and result rows.
Commands/Code:
Create/modify files:
- `src/lib/features/omnibar/Omnibar.svelte`
- `src/lib/features/omnibar/omnibar-store.ts`
Acceptance Criteria: `Ctrl+P` toggles overlay and displays selectable mock result items.

## Task [2.7]: Implement Footer Status Bar
Goal: Match mock footer with branch/sync/metrics/version segments.
Commands/Code:
Create/modify files:
- `src/lib/features/status/StatusBar.svelte`
- `src/lib/features/status/status-store.ts`
Acceptance Criteria: Footer shows all five segments with accent coloring and compact typography.

## Task [2.8]: Implement Acrylic Glass Theme Tokens
Goal: Recreate dark glass aesthetic with teal/orange/green accents.
Commands/Code:
Create/modify files:
- `src/app.css`
- `src/lib/styles/theme.css`
- `src/lib/features/theme/theme-store.ts`
Acceptance Criteria: Surfaces show blur/transparency with readable contrast and graceful fallback.

## Task [2.9]: Wire Typed Frontend IPC Clients
Goal: Introduce typed service boundary for all backend calls/events.
Commands/Code:
Create/modify files:
- `src/lib/ipc/tauri.ts`
- `src/lib/ipc/files.ts`
- `src/lib/ipc/search.ts`
- `src/lib/ipc/git.ts`
- `src/lib/ipc/links.ts`
- `src/lib/ipc/status.ts`
Acceptance Criteria: UI feature modules depend on IPC service wrappers only.

## Task [2.10]: Add Core Keyboard Shortcut Infrastructure
Goal: Ensure global shortcuts match desktop-editor expectations.
Commands/Code:
Create/modify files:
- `src/lib/utils/shortcuts.ts`
- `src/lib/components/layout/AppShell.svelte`
Register shortcuts:
- `Ctrl+P` omnibar
- `Ctrl+B` explorer toggle
- `Ctrl+S` save
Acceptance Criteria: All shortcuts fire expected UI actions without focus conflicts.
