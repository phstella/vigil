# UI Mock to Component Map

## Objective
Translate the provided mock into concrete frontend and backend deliverables.

## Mock Zones
1. Left icon rail (narrow vertical strip).
2. Explorer panel with "Notes", "Collections", and item counts.
3. Center note canvas with large title and markdown text.
4. Right code pane for script editing (`main.py` style view).
5. Floating `Ctrl+P` omnibar with ranked results.
6. Bottom status bar with branch/sync/note/tag/version indicators.

## Component Mapping
- `src/lib/components/chrome/PrimaryRail.svelte`: icon rail and active section state.
- `src/lib/features/explorer/Sidebar.svelte`: collections panel shell.
- `src/lib/features/explorer/FileTree.svelte`: notes/scripts hierarchical list.
- `src/lib/features/editor/NoteEditor.svelte`: markdown note pane.
- `src/lib/features/editor/CodeEditor.svelte`: Monaco code pane.
- `src/lib/features/omnibar/Omnibar.svelte`: overlay command palette.
- `src/lib/features/status/StatusBar.svelte`: footer telemetry and workspace metadata.
- `src/lib/components/layout/WorkspaceGrid.svelte`: desktop layout (rail/explorer/note/code).

## Backend Capability Mapping
- `open_workspace`, `list_dir`, `read_file`, `write_file`: explorer + editor data.
- `fuzzy_find`: omnibar live search.
- `search_content`: phrase/snippet search for goto-anything behavior.
- `get_git_hunks` + event stream: code-pane gutter markers.
- `resolve_links` + `get_backlinks`: note pane contextual links.
- `workspace_status`: footer branch/sync/count metrics.

## Expansion Mapping (Epic 4)
- Graph improvements:
  - Frontend: `src/lib/features/graph/GraphView.svelte`
  - Backend: `src-tauri/src/core/graph/service.rs`
- Live markdown WYSIWYG toggle hardening:
  - Frontend: `src/lib/features/preview/InlinePreview.svelte`
  - Backend assist: `src-tauri/src/core/links/service.rs` for link-aware hints
- WASM plugin runtime + local plugin management:
  - Frontend: `src/lib/features/plugins/PluginManagerPanel.svelte`
  - Backend: `src-tauri/src/core/plugins/{runtime.rs,registry.rs}`

## Theme Tokens (Mock-Aligned)
- Background: deep navy-to-black gradient.
- Surface: translucent dark glass cards.
- Accent primary: holographic teal.
- Accent secondary: amber/orange highlights.
- Accent success: green indicators.
- Blur: acrylic blur with fallback to opaque dark surface.
