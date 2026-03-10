# Epic 4: Advanced Editing + Expandability

## Task [4.1]: Implement Content-Indexed "Goto Anything" Search
Goal: Extend omnibar search from filenames to fast phrase-level content search across large workspaces.
Commands/Code:
```bash
cd src-tauri
cargo add tantivy
```
Create/modify files:
- `src-tauri/src/core/content/indexer.rs`
- `src-tauri/src/core/content/query.rs`
- `src-tauri/src/commands/search.rs`
- `src-tauri/src/models/search.rs`
Acceptance Criteria: Omnibar can return file+line snippet matches for text phrases across large note/code sets.

## Task [4.2]: Extend Omnibar Grammar for Commands and Install Actions
Goal: Support `> command` mode for actions (including plugin install) while preserving file/content search mode.
Commands/Code:
Create/modify files:
- `src/lib/features/omnibar/omnibar-parser.ts`
- `src/lib/features/omnibar/omnibar-store.ts`
- `src/lib/features/omnibar/Omnibar.svelte`
- `src/lib/ipc/search.ts`
Acceptance Criteria: `Ctrl+P` handles file queries, content queries, and `>` action commands in one interface.

## Task [4.3]: Implement Note Graph Data Service
Goal: Build graph nodes/edges from link relationships for visual navigation.
Commands/Code:
Create/modify files:
- `src-tauri/src/core/graph/service.rs`
- `src-tauri/src/commands/links.rs`
- `src-tauri/src/models/links.rs`
Add command:
- `get_note_graph()`
Acceptance Criteria: Backend returns deterministic node/edge graph for current workspace.

## Task [4.4]: Implement Graph View UI Panel
Goal: Add an interactive note graph view accessible from the sidebar tab set.
Commands/Code:
```bash
npm install graphology sigma
```
Create/modify files:
- `src/lib/features/graph/GraphView.svelte`
- `src/lib/features/graph/graph-store.ts`
- `src/lib/features/explorer/Sidebar.svelte`
- `src/lib/ipc/links.ts`
Acceptance Criteria: User can open graph tab, pan/zoom graph, and select a note to open it.

## Task [4.5]: Implement Live Markdown WYSIWYG Toggle
Goal: Render markdown formatting inline while preserving raw text editing mode.
Commands/Code:
Create/modify files:
- `src/lib/features/preview/InlinePreview.svelte`
- `src/lib/features/editor/NoteEditor.svelte`
- `src/lib/features/editor/note-store.ts`
- `src/lib/stores/ui.ts`
Acceptance Criteria: Toggle switches between raw markdown and live render without losing cursor/edit state.

## Task [4.6]: Add Native Vim Keybindings
Goal: Provide out-of-the-box Vim motions for power users.
Commands/Code:
```bash
npm install monaco-vim
```
Create/modify files:
- `src/lib/features/vim/vim-mode.ts`
- `src/lib/features/editor/CodeEditor.svelte`
- `src/lib/features/editor/NoteEditor.svelte`
- `src/lib/stores/ui.ts`
Acceptance Criteria: Vim mode toggle enables normal/insert navigation behavior in both note and code editors.

## Task [4.7]: Implement Default Typography System
Goal: Apply readable sans-serif for prose and crisp monospace with ligatures for code.
Commands/Code:
```bash
npm install @fontsource-variable/inter @fontsource/jetbrains-mono
```
Create/modify files:
- `src/app.css`
- `src/lib/styles/theme.css`
- `src/lib/features/theme/theme-store.ts`
Acceptance Criteria: Notes default to Inter and code defaults to JetBrains Mono with configurable overrides.

## Task [4.8]: Implement WASM Plugin Runtime (Rust)
Goal: Add a secure, lightweight plugin execution model without Node.js plugin overhead.
Commands/Code:
```bash
cd src-tauri
cargo add wasmtime semver toml
```
Create/modify files:
- `src-tauri/src/core/plugins/runtime.rs`
- `src-tauri/src/core/plugins/manifest.rs`
- `src-tauri/src/core/plugins/loader.rs`
- `src-tauri/src/models/plugins.rs`
Acceptance Criteria: A sample WASM plugin loads, executes, and is sandbox-scoped by manifest capabilities.

## Task [4.9]: Expose Versioned Plugin API Hooks
Goal: Make editor, sidebar, and command-palette extension points available to plugins.
Commands/Code:
Create/modify files:
- `src-tauri/src/core/plugins/api.rs`
- `src-tauri/src/commands/plugins.rs`
- `src/lib/ipc/plugins.ts`
- `src/lib/types/plugins.ts`
Acceptance Criteria: Plugin API version negotiation works and unsupported plugin versions are rejected clearly.

## Task [4.10]: Implement Plugin Store Flow in Omnibar
Goal: Enable `> Install Plugin` flow directly inside the command palette.
Commands/Code:
Create/modify files:
- `src/lib/features/plugins/PluginStorePanel.svelte`
- `src/lib/features/plugins/plugin-store.ts`
- `src/lib/features/omnibar/Omnibar.svelte`
- `src/lib/ipc/plugins.ts`
- `src-tauri/src/core/plugins/registry.rs`
Acceptance Criteria: User can search plugin registry entries and install/enable a plugin from the omnibar flow.
