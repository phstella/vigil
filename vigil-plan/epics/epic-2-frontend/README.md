# Epic 2: Frontend Skeleton (Svelte/Tailwind)

> Build the complete application shell: custom title bar, collapsible sidebar,
> resizable split panes, status bar, reactive stores, and typed Tauri IPC
> wrappers.

## Why This Epic Exists

Before any features can be integrated, the UI skeleton must exist. This epic
produces a visually complete (but functionally inert) IDE-like layout with
the glassmorphism aesthetic, plus the reactive state management layer that
all features will plug into.

## Tasks

| Task | Name | Depends On |
|---|---|---|
| [2.1](2.1-custom-title-bar.md) | Minimal Title Bar (Zen Mode) | Epic 0 |
| [2.2](2.2-collapsible-sidebar.md) | Multi-Panel Sidebar with Auto-Hide | 2.8 |
| [2.3](2.3-split-pane.md) | Resizable Split-Pane Workspace (H + V) | 2.1 |
| [2.4](2.4-status-bar.md) | Status Bar (Notes, Tags, Sync) | 2.1 |
| [2.5](2.5-root-layout.md) | Root Layout Composition | 2.1–2.4, 2.8 |
| [2.6](2.6-svelte-stores.md) | Svelte Stores for Application State | Epic 0 |
| [2.7](2.7-tauri-command-wrappers.md) | Tauri Command Wrappers (TypeScript) | Epic 0, Epic 1 |
| [2.8](2.8-activity-bar.md) | Activity Bar (Icon Rail) | 2.1 |

## Execution Order

```
2.1 ──▶ 2.8 ──▶ 2.2 ─┐
               2.3 ───┼──▶ 2.5
               2.4 ──┘

2.6 (parallel with all above)
2.7 (parallel, but depends on Epic 1 command signatures)
```

## Key Design Decisions

- **Zen mode by default**: Minimal title bar, auto-hiding sidebar, no ribbons
  or toolbars. Just the text and a subtle status bar.
- **Activity bar + multi-panel sidebar**: A narrow icon rail on the far left
  switches between Explorer, Search, Graph, and Tags panels.
- **Glassmorphism**: Achieved via `backdrop-blur-xl` on semi-transparent
  `bg-vigil-surface` backgrounds. No external CSS libraries.
- **Auto-hiding sidebar**: Collapses after inactivity, reveals on mouse hover
  at the left edge or `Ctrl+B`.
- **Svelte 5 runes**: All state uses `$state` and `$derived` (not legacy
  stores) for fine-grained reactivity and smaller bundles.
- **Typed IPC**: Frontend never calls raw `invoke()`. All Tauri communication
  goes through typed wrapper functions for safety and autocomplete.
- **Dual typography**: Sans-serif (Inter) for Markdown prose, monospace
  (JetBrains Mono) for code files and code blocks.

## Exit Criteria

- The app renders a complete IDE-like layout in the native window
- Activity bar with 4 panel icons renders on the far left
- Minimal title bar is draggable with working minimize/maximize/close
- Sidebar auto-hides after inactivity, reveals on hover or `Ctrl+B`
- Sidebar switches between Explorer, Search, Graph, and Tags panels
- Explorer shows collections (directories) with file counts
- Split panes resize fluidly at 60fps (horizontal and vertical)
- Status bar displays git branch, sync status, note counts, tags, version
- All Svelte stores are reactive and importable from any component
- TypeScript compiles in strict mode with zero errors
