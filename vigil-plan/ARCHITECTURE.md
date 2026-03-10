# Vigil — Repository Architecture

## Target Folder Structure

```
vigil/
├── src/                                  # SvelteKit frontend
│   ├── app.css                           # Tailwind entry + glassmorphism theme
│   ├── app.html                          # HTML shell
│   ├── lib/
│   │   ├── components/
│   │   │   ├── editor/
│   │   │   │   ├── EditorRouter.svelte           # Routes .md → WYSIWYG, code → Monaco
│   │   │   │   ├── MonacoEditor.svelte           # Monaco for code files
│   │   │   │   ├── WysiwygEditor.svelte          # Tiptap WYSIWYG for Markdown
│   │   │   │   ├── extensions/
│   │   │   │   │   └── wikilink.ts               # Tiptap [[wikilink]] extension
│   │   │   │   ├── GitGutter.svelte              # Git diff gutter markers
│   │   │   │   ├── BacklinksPanel.svelte         # Backlinks display panel
│   │   │   │   └── ConflictBanner.svelte         # File-changed-on-disk banner
│   │   │   ├── layout/
│   │   │   │   ├── ActivityBar.svelte            # Icon rail (far-left)
│   │   │   │   ├── Sidebar.svelte                # Multi-panel sidebar shell
│   │   │   │   ├── SplitPane.svelte              # Resizable split-pane (H + V)
│   │   │   │   ├── StatusBar.svelte              # Bottom status bar
│   │   │   │   └── TitleBar.svelte               # Minimal Tauri-draggable title bar
│   │   │   ├── sidebar/
│   │   │   │   ├── ExplorerPanel.svelte          # File tree + collections
│   │   │   │   ├── SearchPanel.svelte            # Full-text search panel
│   │   │   │   ├── GraphView.svelte              # Force-directed link graph
│   │   │   │   ├── TagsPanel.svelte              # Tag browser + filter
│   │   │   │   └── FileTreeNode.svelte           # Shared file entry component
│   │   │   └── omnibar/
│   │   │       ├── Omnibar.svelte                # Ctrl+P fuzzy-find overlay
│   │   │       └── OmnibarItem.svelte            # Result row with type badges
│   │   ├── stores/
│   │   │   ├── editor.ts                     # Active file, dirty state, cursor pos
│   │   │   ├── files.ts                      # File tree, collections, note counts
│   │   │   ├── git.ts                        # Git diff/status cache, sync indicator
│   │   │   ├── tags.ts                       # Tag index, active file tags
│   │   │   └── settings.ts                   # User prefs, active panel, auto-hide
│   │   ├── tauri/
│   │   │   ├── commands.ts                   # Typed wrappers around invoke()
│   │   │   └── events.ts                     # Tauri event listeners
│   │   └── utils/
│   │       ├── keybindings.ts                # Keyboard shortcut registry
│   │       └── markdown.ts                   # Wikilink parser, backlink extractor
│   └── routes/
│       ├── +layout.ts                        # ssr=false, prerender=false
│       ├── +layout.svelte                    # Root layout (imports app.css)
│       └── +page.svelte                      # Main workspace shell
│
├── src-tauri/                            # Rust backend
│   ├── src/
│   │   ├── main.rs                           # Tauri bootstrap + plugin registration
│   │   ├── lib.rs                            # Module declarations
│   │   ├── commands/
│   │   │   ├── mod.rs
│   │   │   ├── fs.rs                         # File read/write/rename/delete
│   │   │   ├── git.rs                        # Git status, diff, blame
│   │   │   ├── search.rs                     # Fuzzy-find + full-text search
│   │   │   └── workspace.rs                  # Workspace open, recent files
│   │   ├── indexer/
│   │   │   ├── mod.rs
│   │   │   ├── file_watcher.rs               # notify-based recursive watcher
│   │   │   ├── link_graph.rs                 # Bidirectional wikilink graph
│   │   │   ├── search_index.rs               # Tantivy full-text index
│   │   │   └── tag_index.rs                  # Tag extraction & index
│   │   └── git/
│   │       ├── mod.rs
│   │       └── differ.rs                     # git2-rs diff/status engine
│   ├── capabilities/
│   │   └── default.json                      # Tauri v2 permissions
│   ├── icons/                                # App icons (all sizes)
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   └── build.rs
│
├── static/                               # Static assets
│   └── fonts/                                # JetBrains Mono, etc.
│
├── .eslintrc.cjs                         # ESLint config
├── .prettierrc                           # Prettier config
├── rustfmt.toml                          # Rust formatter config
├── clippy.toml                           # Clippy linter config
├── package.json
├── svelte.config.js
├── vite.config.ts
├── tsconfig.json
└── README.md
```

## Data Flow

```
┌─────────────────────────────────────────────────────────┐
│                    Tauri WebView                         │
│                                                         │
│  ┌─────────┐   ┌──────────┐   ┌────────────────────┐  │
│  │Activity│Sidebar│  Editor   │   │  Omnibar / Search  │  │
│  │  Bar   │(panel)│ (WYSIWYG /│   │   (Ctrl+P / F)     │  │
│  │(icons) │      │  Monaco)  │   │                    │  │
│  └────┬────┘   └─────┬─────┘   └─────────┬──────────┘  │
│       │              │                    │              │
│  ┌────▼──────────────▼────────────────────▼──────────┐  │
│  │            Svelte Stores ($state / $derived)       │  │
│  │   editor.ts  ·  files.ts  ·  git.ts  ·  settings │  │
│  └────────────────────┬──────────────────────────────┘  │
│                       │                                  │
│  ┌────────────────────▼──────────────────────────────┐  │
│  │       Tauri IPC (invoke / listen wrappers)        │  │
│  │       commands.ts  ·  events.ts                   │  │
│  └────────────────────┬──────────────────────────────┘  │
└───────────────────────┼──────────────────────────────────┘
                        │  Tauri IPC bridge
┌───────────────────────┼──────────────────────────────────┐
│                       │         Rust Backend             │
│  ┌────────────────────▼──────────────────────────────┐  │
│  │              Tauri Command Handlers               │  │
│  │   fs.rs  ·  git.rs  ·  search.rs  ·  workspace   │  │
│  └──────┬─────────┬────────────┬─────────────────────┘  │
│         │         │            │                         │
│  ┌──────▼───┐ ┌───▼────┐ ┌────▼──────────────────────┐ │
│  │ git2-rs  │ │ walkdir │ │       Indexer             │ │
│  │ differ   │ │ + trash │ │  file_watcher (notify)    │ │
│  │          │ │         │ │  search_index (tantivy)   │ │
│  │          │ │         │ │  link_graph (wikilinks)   │ │
│  └──────────┘ └─────────┘ └──────────────────────────┘ │
│                                                         │
│                   Local Filesystem                       │
│              ~/.config/vigil/  (settings)                │
│              ~/.cache/vigil/   (search index)            │
│              /path/to/vault/   (user files)              │
└─────────────────────────────────────────────────────────┘
```

## Key Rust Crate Dependencies

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

## Key Frontend Dependencies

| Package | Purpose |
|---|---|
| `@tauri-apps/api` | Tauri JS bridge (invoke, listen, window) |
| `@tauri-apps/cli` | Tauri CLI (dev dependency) |
| `@sveltejs/adapter-static` | SPA output for Tauri |
| `tailwindcss` + `@tailwindcss/vite` | Styling (v4, Vite plugin) |
| `@tiptap/core` + extensions | WYSIWYG Markdown editor (ProseMirror-based) |
| `monaco-editor` | Code editor for non-Markdown files |
| `prettier-plugin-svelte` | Svelte file formatting |

## Design Tokens (Tailwind v4 @theme)

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
/* Sans-serif for Markdown prose, monospace for code */
```
