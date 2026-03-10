# UI Mock to Component Map

## Objective

Translate the UI mock into concrete frontend components, backend commands, and event channels. Every visible region in the mock must map to an owning component and its data dependencies. This document is the implementation contract for Epics 2 and 3.

## Mock Zones

The mock defines six primary interaction zones. Each zone is a distinct visual region with its own layout, state, and backend dependencies.

| # | Zone | Visual Description | Keyboard Access |
|---|---|---|---|
| 1 | Activity Rail | Narrow vertical icon strip on the far left | `Ctrl+1` focuses explorer |
| 2 | Sidebar | Multi-panel area: Explorer, Search, Graph, Tags | `Ctrl+B` toggles visibility |
| 3 | Center Note Pane | Large canvas: note title, markdown body, inline preview | `Ctrl+2` focuses note pane |
| 4 | Right Code Pane | Code editor surface with git gutter markers | `Ctrl+3` focuses code pane |
| 5 | Omnibar Overlay | Floating command palette centered near top of window | `Ctrl+P` opens; `Esc` closes |
| 6 | Status Bar | Bottom strip: branch, sync state, note/tag counts, version | Always visible |

### Zone 1: Activity Rail

- Fixed-width vertical strip on the far-left edge.
- Icons for: Explorer, Search, Graph, Tags.
- Active icon highlights current sidebar panel.
- Workspace switcher affordance at bottom of rail.

### Zone 2: Sidebar (Multi-Panel)

The sidebar hosts four switchable panels, each activated by the corresponding rail icon:

| Panel | Description | Data Source |
|---|---|---|
| Explorer | File tree with collections, note counts, quick filters | `list_dir`, `index://updated` |
| Search | Full-text search input + snippet results | `search_content` (Epic 4), `fuzzy_find` (MVP filename mode) |
| Graph | Force-directed note link graph (pan/zoom/select) | `get_note_graph` |
| Tags | Tag browser with filter and note counts | Tag index from `index://updated` |

Sidebar supports auto-hide on focus loss and manual `Ctrl+B` toggle. Width is resizable via drag handle.

### Zone 3: Center Note Pane

- Displays the active markdown note.
- Large editable title field at top.
- Tiptap WYSIWYG body with `[[wikilink]]` inline rendering.
- `Ctrl+.` toggles between WYSIWYG and raw markdown view.
- Backlinks panel below note body (collapsible).
- Conflict banner at top if file changed on disk.

### Zone 4: Right Code Pane

- Monaco editor instance for non-markdown files.
- Git gutter markers (added/modified/deleted line indicators).
- File type detection and syntax highlighting.
- Split with center pane via resizable horizontal divider.

### Zone 5: Omnibar Overlay

- Modal floating overlay, centered horizontally, positioned near top.
- Captures all keyboard input while open.
- Modes:
  - **File mode** (default `Ctrl+P`): fuzzy filename search.
  - **Content mode** (`Ctrl+Shift+F` or prefix `>`): phrase/snippet search (Epic 4 for full content; MVP uses filename only).
  - **Command mode** (`Ctrl+Shift+P`): action palette (Epic 4 expansion).
- Each result row shows: icon, filename/title, path, match score badge.
- `Enter` opens selected; `Esc` closes; `ArrowUp/Down` navigates.

### Zone 6: Status Bar

- Full-width strip at bottom edge.
- Segments (left to right):
  - Git branch name and sync indicator.
  - Note count and tag count.
  - Active file path (truncated).
  - App version.
- Updates via `status://updated` event channel.

## Component Mapping

### Layout Components (`src/lib/components/layout/`)

| Component | Zone | Responsibility |
|---|---|---|
| `WorkspaceGrid.svelte` | All | Top-level CSS grid: rail, sidebar, center, right, status |
| `ActivityBar.svelte` | 1 | Icon rail rendering and panel switch dispatch |
| `Sidebar.svelte` | 2 | Multi-panel shell with auto-hide and resize handle |
| `SplitPane.svelte` | 3+4 | Resizable horizontal divider between center and right panes |
| `StatusBar.svelte` | 6 | Footer bar layout and segment rendering |
| `TitleBar.svelte` | -- | Minimal Tauri-draggable custom title bar |

### Chrome Components (`src/lib/components/chrome/`)

| Component | Responsibility |
|---|---|
| `PrimaryRail.svelte` | Icon rail active-section state indicator |

### Feature Components

#### Explorer (`src/lib/features/explorer/`)

| Component | Zone | Responsibility |
|---|---|---|
| `ExplorerPanel.svelte` | 2 | File tree + collections panel shell |
| `FileTree.svelte` | 2 | Notes/scripts hierarchical list with expand/collapse |
| `FileTreeNode.svelte` | 2 | Individual tree entry (icon, name, count badge) |
| `SearchPanel.svelte` | 2 | Search input + result list panel |
| `TagsPanel.svelte` | 2 | Tag browser with filter |

#### Editor (`src/lib/features/editor/`)

| Component | Zone | Responsibility |
|---|---|---|
| `EditorRouter.svelte` | 3+4 | Routes `.md` files to WYSIWYG, code files to Monaco |
| `NoteEditor.svelte` | 3 | Tiptap WYSIWYG markdown pane with title field |
| `CodeEditor.svelte` | 4 | Monaco code editor with syntax highlighting |
| `GitGutter.svelte` | 4 | Git diff gutter markers overlay |
| `BacklinksPanel.svelte` | 3 | Collapsible backlinks list below note body |
| `ConflictBanner.svelte` | 3 | File-changed-on-disk notification banner |

#### Preview (`src/lib/features/preview/`)

| Component | Zone | Responsibility |
|---|---|---|
| `InlinePreview.svelte` | 3 | WYSIWYG/raw markdown toggle surface |

#### Graph (`src/lib/features/graph/`)

| Component | Zone | Responsibility |
|---|---|---|
| `GraphView.svelte` | 2 | Force-directed link graph with pan/zoom/select |

#### Omnibar (`src/lib/features/omnibar/`)

| Component | Zone | Responsibility |
|---|---|---|
| `Omnibar.svelte` | 5 | Floating overlay shell, keyboard capture, mode routing |
| `OmnibarItem.svelte` | 5 | Result row: icon, name, path, score badge |

#### Status (`src/lib/features/status/`)

| Component | Zone | Responsibility |
|---|---|---|
| `StatusBar.svelte` | 6 | Footer segment rendering: branch, sync, counts, version |

#### Links (`src/lib/features/links/`)

| Component | Zone | Responsibility |
|---|---|---|
| (integrated into editor) | 3 | `[[` autocomplete trigger, link resolution hints |

#### Plugins (`src/lib/features/plugins/`) -- Epic 4

| Component | Zone | Responsibility |
|---|---|---|
| `PluginManagerPanel.svelte` | 2 | List installed plugins, enable/disable, show capabilities |

Note: There is no plugin marketplace or remote store UI. Plugin management is local-only.

## Backend Capability Mapping

### MVP Commands (Epics 1-3)

| Command | Zones Served | Frontend Consumer | Description |
|---|---|---|---|
| `open_workspace(root_path)` | All | Workspace init | Opens vault, returns metadata and file counts |
| `list_dir(path)` | 2 | `ExplorerPanel`, `FileTree` | Returns sorted directory entries |
| `read_file(path)` | 3, 4 | `NoteEditor`, `CodeEditor` | Returns UTF-8 content + metadata |
| `write_file(path, content)` | 3, 4 | `NoteEditor`, `CodeEditor` | Saves content, returns new metadata |
| `create_note(path)` | 2, 3 | `ExplorerPanel`, `NoteEditor` | Creates new markdown file |
| `fuzzy_find(query, limit)` | 5 | `Omnibar` | Ranked filename matches for Ctrl+P |
| `get_git_hunks(path)` | 4 | `GitGutter` | Line-range hunks (added/modified/deleted) |
| `get_backlinks(path)` | 3 | `BacklinksPanel` | Notes linking to the target file |
| `workspace_status()` | 6 | `StatusBar` | Branch, sync state, note/tag counts, version |

### Event Channels (MVP)

| Channel | Payload | Frontend Consumer | Description |
|---|---|---|---|
| `index://updated` | Changed paths + update type | `FileTree`, `ExplorerPanel`, stores | File create/change/delete notifications |
| `git://hunks` | File path + hunk list | `GitGutter`, `git` store | Updated git diff markers |
| `status://updated` | Workspace status snapshot | `StatusBar`, `status` store | Branch/sync/count refresh |

### Epic 4 Commands

| Command | Zones Served | Frontend Consumer | Description |
|---|---|---|---|
| `search_content(query, limit)` | 5, 2 | `Omnibar`, `SearchPanel` | Snippet-level phrase/content matches |
| `get_note_graph()` | 2 | `GraphView` | Graph payload `{ nodes[], edges[] }` |
| `plugin_list()` | 2 | `PluginManagerPanel` | Installed plugin manifests + enabled state |
| `plugin_enable(plugin_id, enabled)` | 2 | `PluginManagerPanel` | Toggle plugin active state |

### Epic 4 Event Channels

| Channel | Payload | Frontend Consumer | Description |
|---|---|---|---|
| `plugins://updated` | Plugin state transition | `PluginManagerPanel`, `plugins` store | Install/enable/runtime changes |

## Store-to-Zone Mapping

| Store | File | Zones | Feeds |
|---|---|---|---|
| Editor | `stores/editor.ts` | 3, 4 | Active file path, dirty state, cursor position, content |
| Files | `stores/files.ts` | 2, 5 | File tree data, collections, note counts |
| Git | `stores/git.ts` | 4, 6 | Diff/status cache, sync indicator |
| Tags | `stores/tags.ts` | 2, 6 | Tag index, active file tags, counts |
| Settings | `stores/settings.ts` | All | Active panel, auto-hide, user preferences |

## IPC Flow per Zone

### Explorer (Zone 2) Flow

```
User clicks folder -> FileTree dispatches list_dir via ipc/commands.ts
                   -> Backend returns FileEntry[]
                   -> files store updates
                   -> FileTree re-renders

File watcher detects change -> Backend emits index://updated
                            -> events.ts listener fires
                            -> files store invalidates affected entries
                            -> FileTree re-renders
```

### Editor (Zones 3+4) Flow

```
User opens file -> EditorRouter calls read_file via ipc/commands.ts
               -> Backend returns content + metadata
               -> editor store sets active file
               -> EditorRouter routes to NoteEditor or CodeEditor

User saves -> Editor calls write_file via ipc/commands.ts
           -> Backend writes, returns new metadata
           -> editor store clears dirty flag
           -> Backend emits index://updated for changed file
```

### Omnibar (Zone 5) Flow

```
User presses Ctrl+P -> Omnibar opens, captures focus
User types query    -> Omnibar calls fuzzy_find via ipc/commands.ts
                    -> Backend returns FuzzyMatch[]
                    -> Omnibar renders OmnibarItem list
User presses Enter  -> Omnibar dispatches open action
                    -> EditorRouter opens selected file
                    -> Omnibar closes
```

### Status Bar (Zone 6) Flow

```
App init            -> StatusBar calls workspace_status via ipc/commands.ts
                    -> Backend returns WorkspaceStatus
                    -> StatusBar renders segments

Backend state changes -> Backend emits status://updated
                      -> events.ts listener fires
                      -> status store updates
                      -> StatusBar re-renders
```

## Theme Tokens (Mock-Aligned)

| Token | Value | Usage |
|---|---|---|
| Background | Deep navy-to-black gradient (`#0d0f12`) | App shell background |
| Surface | Translucent dark glass (`rgba(20, 24, 33, 0.75)`) | Cards, panels, sidebar |
| Border | Subtle white (`rgba(255, 255, 255, 0.06)`) | Panel dividers, card edges |
| Accent primary | Holographic teal (`#2dd4bf`) | Active states, links, highlights |
| Accent secondary | Amber/orange (`#fb923c`) | Warnings, secondary indicators |
| Success | Green (`#4ade80`) | Sync OK, healthy plugin state |
| Text primary | Light slate (`#e2e8f0`) | Body text, titles |
| Text muted | Mid slate (`#64748b`) | Metadata, placeholders |
| Blur | Acrylic blur with opaque dark fallback | Sidebar, omnibar overlay |
| Font sans | Inter, system-ui, sans-serif | Markdown prose, UI labels |
| Font mono | JetBrains Mono, Fira Code, monospace | Code editor, inline code |

## Scope Exclusions

- **No Vim mode.** No Vim keybinding emulation in any editor surface.
- **No plugin marketplace.** Plugin management is local-only. No remote store, discovery, or download flow.

## Cross-References

| Document | Relation |
|---|---|
| `docs/architecture/01-repository-architecture.md` | Directory structure and ownership rules |
| `docs/specs/ipc-contracts.md` | Command signatures, error codes, event definitions |
| `docs/specs/workspace-data-model.md` | Data structures (FileEntry, FuzzyMatch, GitHunk, etc.) |
| `docs/specs/editor-performance-budget.md` | Latency gates for each zone interaction |
| `docs/specs/keyboard-map.md` | Full shortcut registry and conflict rules |
| `docs/specs/plugin-sandbox-model.md` | WASM plugin isolation (Epic 4) |
