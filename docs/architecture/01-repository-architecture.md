# Vigil Repository Architecture

## Purpose

Define a strict separation between the Rust/Tauri backend and the Svelte frontend while preserving fast iteration and clear ownership. This document is the authoritative reference for directory layout, module ownership, composition targets, and API surface policy. All implementation tasks should trace back to the structures defined here.

## Top-Level Structure

The tree below reflects the canonical directory layout. Every directory listed here must exist before Epic 1 begins. Directories marked `(Epic 4)` are scaffolded early but receive implementation content only during post-MVP expansion.

```text
vigil/
  docs/
    architecture/
      01-repository-architecture.md      # This file
      02-ui-mock-component-map.md        # UI zone-to-component mapping
    specs/
      ipc-contracts.md                   # Frontend-backend command contracts
      workspace-data-model.md            # Data structures and caching
      editor-performance-budget.md       # Latency and memory gates
      keyboard-map.md                    # Shortcut registry
      plugin-sandbox-model.md            # WASM plugin isolation (Epic 4)
    tickets/
      00-mvp-execution-order.md
      00-unified-vigil-backlog.md
      epic-0-repository-genesis-infrastructure.md
      epic-1-backend-foundation.md
      epic-2-frontend-skeleton.md
      epic-3-core-features-integration.md
      epic-4-advanced-editing-expandability.md
    implementation-diary/                # Append-only per-task logs
  src/                                   # SvelteKit frontend (TypeScript)
    app.css                              # Tailwind v4 entry + design tokens
    app.html                             # HTML shell
    routes/
      +layout.ts                         # ssr=false, prerender=true
      +layout.svelte                     # Root layout (imports app.css)
      +page.svelte                       # Main workspace shell
    lib/
      components/
        layout/                          # Structural shell elements
          ActivityBar.svelte             # Icon rail (far-left vertical strip)
          Sidebar.svelte                 # Multi-panel sidebar shell
          SplitPane.svelte               # Resizable split-pane (H + V)
          StatusBar.svelte               # Bottom status bar
          TitleBar.svelte                # Minimal Tauri-draggable title bar
          WorkspaceGrid.svelte           # Top-level desktop grid layout
        chrome/                          # Reusable UI primitives
          PrimaryRail.svelte             # Icon rail active-section state
      features/
        workspace/                       # Workspace lifecycle and root state
        explorer/                        # File tree, collections, search panel
          ExplorerPanel.svelte           # File tree + collections
          FileTree.svelte                # Notes/scripts hierarchical list
          FileTreeNode.svelte            # Individual tree entry
          SearchPanel.svelte             # Full-text search panel
          TagsPanel.svelte               # Tag browser + filter
        editor/                          # Note and code editing surfaces
          EditorRouter.svelte            # Routes .md -> WYSIWYG, code -> Monaco
          NoteEditor.svelte              # Tiptap WYSIWYG for Markdown
          CodeEditor.svelte              # Monaco for code files
          GitGutter.svelte               # Git diff gutter markers
          BacklinksPanel.svelte          # Backlinks display panel
          ConflictBanner.svelte          # File-changed-on-disk banner
        preview/                         # Live markdown render
          InlinePreview.svelte           # WYSIWYG/raw toggle surface
        graph/                           # Note graph visualization
          GraphView.svelte               # Force-directed link graph
        omnibar/                         # Command palette and fuzzy search
          Omnibar.svelte                 # Ctrl+P fuzzy-find overlay
          OmnibarItem.svelte             # Result row with type badges
        links/                           # Wikilink and backlink UI
        git/                             # Git status display
        status/                          # Footer status telemetry
          StatusBar.svelte               # Branch/sync/count/version display
        plugins/                         # (Epic 4) Local plugin management
          PluginManagerPanel.svelte       # List/enable/disable/capabilities
        theme/                           # Theme tokens and switching
      stores/
        editor.ts                        # Active file, dirty state, cursor pos
        files.ts                         # File tree, collections, note counts
        git.ts                           # Git diff/status cache, sync indicator
        tags.ts                          # Tag index, active file tags
        settings.ts                      # User prefs, active panel, auto-hide
      ipc/
        commands.ts                      # Typed wrappers around invoke()
        events.ts                        # Tauri event listeners
      styles/                            # Shared CSS utilities
      types/                             # TypeScript type mirrors of Rust models
      utils/
        keybindings.ts                   # Keyboard shortcut registry
        markdown.ts                      # Wikilink parser, backlink extractor
  src-tauri/                             # Rust backend
    capabilities/
      default.json                       # Tauri v2 permissions
    src/
      main.rs                            # Tauri bootstrap + plugin registration
      lib.rs                             # Module declarations
      commands/
        mod.rs
        fs.rs                            # File read/write/rename/delete
        git.rs                           # Git status, diff, blame
        search.rs                        # Fuzzy-find + full-text search
        workspace.rs                     # Workspace open, recent files
      core/
        fs/                              # Filesystem service (list/read/write)
        index/                           # Incremental file indexer
          file_watcher.rs                # notify-based recursive watcher
          tag_index.rs                   # Tag extraction and index
        content/                         # Content pipeline and transforms
        search/                          # Search engines
          search_index.rs                # Tantivy full-text index
          fuzzy.rs                       # Skim-based fuzzy file matching
        graph/                           # Link graph engine
          link_graph.rs                  # Bidirectional wikilink graph
          service.rs                     # Graph query service
        git/                             # Git integration
          differ.rs                      # git2-rs diff/status engine
        links/                           # Link resolution and backlinks
          service.rs                     # Link-aware hints
        plugins/                         # (Epic 4) WASM plugin runtime
          runtime.rs                     # wasmtime execution host
          registry.rs                    # Plugin manifest registry
      models/                            # Shared request/response types
      state/                             # App-level managed state
      events/                            # Event channel definitions
    tests/                               # Backend integration tests
    icons/                               # App icons (all sizes)
    Cargo.toml
    tauri.conf.json
    build.rs
  static/
    fonts/                               # JetBrains Mono, Inter
  .eslintrc.cjs
  .prettierrc
  .editorconfig
  rustfmt.toml
  clippy.toml
  package.json
  svelte.config.js
  vite.config.ts
  tsconfig.json
```

## Ownership Rules

### Backend: `src-tauri/`

| Path | Owner | Responsibility |
|---|---|---|
| `src-tauri/src/core/fs/` | FS service | Workspace-confined file read/write/create/delete, path canonicalization, traversal guards |
| `src-tauri/src/core/index/` | Indexer | Recursive file watcher (notify), incremental index updates, tag extraction |
| `src-tauri/src/core/search/` | Search engine | Tantivy full-text index, skim-based fuzzy file matching |
| `src-tauri/src/core/graph/` | Graph engine | Bidirectional wikilink graph construction and query |
| `src-tauri/src/core/git/` | Git worker | git2-rs diff, status, blame; hunk event stream |
| `src-tauri/src/core/links/` | Link resolver | Backlink resolution, link-aware editor hints |
| `src-tauri/src/core/content/` | Content pipeline | Content transforms and processing |
| `src-tauri/src/core/plugins/` | Plugin runtime (Epic 4) | wasmtime WASM host, manifest validation, capability sandbox |
| `src-tauri/src/commands/` | Command layer | Thin Tauri `#[command]` wrappers that delegate to core services; no business logic here |
| `src-tauri/src/models/` | Shared types | Rust structs for all IPC payloads; source of truth for frontend type mirrors |
| `src-tauri/src/state/` | App state | Tauri managed state containers, workspace lifecycle |
| `src-tauri/src/events/` | Event channels | Typed event definitions for push updates to frontend |

### Frontend: `src/`

| Path | Owner | Responsibility |
|---|---|---|
| `src/lib/ipc/` | IPC gateway | Typed `invoke()` wrappers and event listeners; single point of contact with backend |
| `src/lib/components/layout/` | Shell layout | Structural elements: activity bar, sidebar, split pane, status bar, title bar, workspace grid |
| `src/lib/components/chrome/` | UI primitives | Reusable chrome elements: primary rail, buttons, icons |
| `src/lib/features/workspace/` | Workspace | Workspace lifecycle orchestration |
| `src/lib/features/explorer/` | Explorer | File tree, collections, search panel, tags panel |
| `src/lib/features/editor/` | Editor | Note editor (Tiptap WYSIWYG), code editor (Monaco), editor routing, git gutter, backlinks, conflict detection |
| `src/lib/features/preview/` | Preview | Live markdown WYSIWYG/raw toggle |
| `src/lib/features/graph/` | Graph | Force-directed link graph visualization |
| `src/lib/features/omnibar/` | Omnibar | Ctrl+P command palette, fuzzy search, result rendering |
| `src/lib/features/links/` | Links | Wikilink autocomplete and backlink display |
| `src/lib/features/git/` | Git UI | Git status display and refresh coordination |
| `src/lib/features/status/` | Status bar | Footer telemetry: branch, sync state, counts, version |
| `src/lib/features/plugins/` | Plugins (Epic 4) | Local plugin management panel (no marketplace) |
| `src/lib/features/theme/` | Theme | Theme token management and preset switching |
| `src/lib/stores/` | State management | Svelte stores for editor, files, git, tags, settings |
| `src/lib/types/` | Type mirrors | TypeScript types synced from `src-tauri/src/models/` |
| `src/lib/utils/` | Utilities | Keybinding registry, markdown/wikilink parser |

### Cross-Cutting Rules

1. **No business logic in commands.** `src-tauri/src/commands/` must only deserialize, delegate to `core/`, and serialize the response.
2. **No direct Tauri calls from components.** All IPC flows through `src/lib/ipc/`; components and features never call `invoke()` directly.
3. **Types flow Rust-to-TypeScript.** `src-tauri/src/models/` is the source of truth. `src/lib/types/` must stay in sync on every contract change.
4. **Feature isolation.** Each `src/lib/features/*` directory owns its UI, local state, and wiring. Cross-feature communication goes through stores.
5. **Plugin scope.** Plugin runtime exists under `src-tauri/src/core/plugins/` and frontend under `src/lib/features/plugins/`. There is no plugin marketplace or store flow. Plugins are local artifacts only.

## UI Composition Target

The application shell composes into seven visual zones. Each zone maps to specific layout components and feature modules.

```text
+------+-------------------+-------------------+
|      |                   |                   |
| RAIL |    SIDEBAR        |                   |
| (A)  |    (B)            |    CENTER PANE    |
|      |                   |    (C)            |
|      +-------------------+                   |
|      |                   |                   |
|      |  (B) panels:      +-------------------+
|      |  - Explorer       |                   |
|      |  - Search         |    RIGHT PANE     |
|      |  - Graph          |    (D)            |
|      |  - Tags           |                   |
|      |                   |                   |
+------+-------------------+-------------------+
|              STATUS BAR (F)                   |
+-----------------------------------------------+

              OMNIBAR OVERLAY (E)
         (Ctrl+P, centered near top)
```

### Zone Definitions

| Zone | ID | Layout Component | Feature Module | Description |
|---|---|---|---|---|
| Activity Rail | A | `ActivityBar.svelte` | -- | Far-left icon strip; switches active sidebar panel, workspace selector |
| Sidebar | B | `Sidebar.svelte` | `features/explorer/` | Auto-hide multi-panel: Explorer, Search, Graph, Tags |
| Center Pane | C | `SplitPane.svelte` | `features/editor/` | Markdown note: title, body, inline preview, backlinks panel |
| Right Pane | D | `SplitPane.svelte` | `features/editor/` | Code editor (Monaco), git gutter markers |
| Omnibar | E | (overlay) | `features/omnibar/` | Floating Ctrl+P palette: file search, content search, commands |
| Status Bar | F | `StatusBar.svelte` | `features/status/` | Footer: git branch, sync state, note/tag counts, app version |
| Graph View | -- | (sidebar panel) | `features/graph/` | Force-directed note link graph (sidebar or expansion) |

### Layout Behavior

- **Split pane** between center (C) and right (D) is resizable horizontally. Vertical split is reserved for future use.
- **Sidebar** (B) supports `Ctrl+B` toggle and auto-hide based on focus.
- **Omnibar** (E) is a modal overlay that captures keyboard focus; `Esc` dismisses.
- **Status bar** (F) is always visible; content updates via `status://updated` events.
- **Activity rail** (A) icons correspond 1:1 with sidebar panels.

## Data Flow

```text
+-------------------------------------------------------------+
|                      Tauri WebView                           |
|                                                              |
|  +----------+ +----------+ +--------------------+            |
|  | Activity | | Sidebar  | | Editor (WYSIWYG /  |           |
|  |   Bar    | | (panels) | |  Monaco)           |           |
|  +----+-----+ +----+-----+ +--------+-----------+           |
|       |             |                |                       |
|  +----v-------------v----------------v---+ +-----------+    |
|  |      Svelte Stores ($state/$derived)  | | Omnibar / |    |
|  |  editor.ts | files.ts | git.ts | ...  | | Search    |    |
|  +-------------------+-------------------+ +-----+-----+    |
|                       |                          |           |
|  +--------------------v--------------------------v------+    |
|  |          IPC Layer (commands.ts / events.ts)         |    |
|  +--------------------+--------------------------------+    |
+------------------------|---------------------------------+
                         | Tauri IPC bridge (invoke / listen)
+------------------------|---------------------------------+
|                        v          Rust Backend            |
|  +-----------------------------------------------------+ |
|  |             Tauri Command Handlers                   | |
|  |  fs.rs | git.rs | search.rs | workspace.rs          | |
|  +---+----------+-----------+---------------------------+ |
|      |          |           |                             |
|  +---v---+ +---v----+ +----v------------------------+    |
|  | git2  | | walkdir | |         Core Services       |    |
|  | differ | | + trash | |  file_watcher (notify)     |    |
|  |        | |         | |  search_index (tantivy)    |    |
|  |        | |         | |  link_graph (wikilinks)    |    |
|  |        | |         | |  tag_index                 |    |
|  +--------+ +---------+ +---------------------------+    |
|                                                           |
|                   Local Filesystem                        |
|             ~/.config/vigil/  (settings)                  |
|             ~/.cache/vigil/   (search index)              |
|             /path/to/vault/   (user files)                |
+-----------------------------------------------------------+
```

## API Surface Policy

- **Command-driven.** All frontend-to-backend operations use Tauri `invoke()` with typed request/response payloads defined in `src-tauri/src/models/`.
- **Event-driven push.** Backend pushes state changes via typed event channels (`index://updated`, `git://hunks`, `status://updated`). Frontend subscribes through `src/lib/ipc/events.ts`.
- **Response format.** Commands resolve to typed payloads on success and reject with `ErrorEnvelope { code, message, details? }` on failure, as defined in `docs/specs/ipc-contracts.md`.
- **Path confinement.** Every command validates that file paths resolve within the workspace root. Traversal outside the root returns `PATH_OUTSIDE_WORKSPACE`.
- **Plugin APIs (Epic 4).** Plugin host calls are capability-scoped and versioned. No remote plugin store. Plugins are loaded from local filesystem only.

## Concurrency Model

- **File watcher** runs on a dedicated Rust thread; debounced via `notify-debouncer-full`.
- **Search and index reads** operate against lock-protected snapshot state.
- **File writes** acquire per-file mutation locks.
- **UI event delivery** is serialized to preserve ordering.
- **Background tasks** (indexing, git refresh) run on `tokio` async runtime and push results via event channels.

See `docs/specs/workspace-data-model.md` for invalidation rules and caching strategy.

## Key Dependencies

### Rust Crates

| Crate | Version | Purpose |
|---|---|---|
| `tauri` | 2.x | Application shell, IPC, window management |
| `serde` / `serde_json` | 1.x | Serialization for all IPC payloads |
| `git2` | 0.20 | Git status, diff, blame |
| `notify` | 8.x | Filesystem event watching |
| `notify-debouncer-full` | 0.5 | Debounced file watcher events |
| `tantivy` | 0.25 | Full-text search indexing |
| `rayon` | 1.11 | Parallel file processing |
| `walkdir` | 2.x | Recursive directory traversal |
| `fuzzy-matcher` | 0.3 | Skim-based fuzzy file matching |
| `tokio` | 1.x | Async runtime for background tasks |
| `trash` | 5.x | Safe file deletion (OS trash) |

### Frontend Packages

| Package | Purpose |
|---|---|
| `@tauri-apps/api` | Tauri JS bridge (invoke, listen, window) |
| `@tauri-apps/cli` | Tauri CLI (dev dependency) |
| `@sveltejs/adapter-static` | SPA output for Tauri |
| `tailwindcss` + `@tailwindcss/vite` | Styling (v4, Vite plugin) |
| `@tiptap/core` + extensions | WYSIWYG Markdown editor (ProseMirror-based) |
| `monaco-editor` | Code editor for non-Markdown files |
| `prettier-plugin-svelte` | Svelte file formatting |

## Design Tokens

Tokens are defined in `src/app.css` using Tailwind v4 `@theme` directive. See `vigil-plan/ARCHITECTURE.md` for the full token reference.

```css
@theme {
  --color-vigil-bg:         #0d0f12;
  --color-vigil-surface:    rgba(20, 24, 33, 0.75);
  --color-vigil-border:     rgba(255, 255, 255, 0.06);
  --color-vigil-teal:       #2dd4bf;
  --color-vigil-orange:     #fb923c;
  --color-vigil-green:      #4ade80;
  --color-vigil-text:       #e2e8f0;
  --color-vigil-text-muted: #64748b;
  --font-family-sans:       'Inter', system-ui, sans-serif;
  --font-family-mono:       'JetBrains Mono', 'Fira Code', monospace;
}
```

Visual identity: deep navy-to-black gradient backgrounds, translucent dark glass surfaces, holographic teal accent, amber/orange secondary, acrylic blur with opaque fallback.

## Scope Exclusions

The following are explicitly out of scope for the canonical plan:

- **Vim mode.** No Vim keybinding emulation layer.
- **Plugin marketplace.** No remote plugin store, discovery, or download flow. Plugins are local-only artifacts managed through `src/lib/features/plugins/`.

## Cross-References

| Document | Relation |
|---|---|
| `docs/architecture/02-ui-mock-component-map.md` | Zone-to-component and backend capability mapping |
| `docs/specs/ipc-contracts.md` | Command signatures and event channel definitions |
| `docs/specs/workspace-data-model.md` | Data structures, caching, invalidation rules |
| `docs/specs/editor-performance-budget.md` | Latency and memory budget gates |
| `docs/specs/keyboard-map.md` | Shortcut registry and conflict rules |
| `docs/specs/plugin-sandbox-model.md` | WASM plugin isolation model (Epic 4) |
| `docs/tickets/00-unified-vigil-backlog.md` | Canonical task backlog |
