# Epic 3: Core Features Integration

> Wire the Rust backend to the Svelte frontend: embed the code editor, build
> the omnibar, render git gutter markers, implement bidirectional linking UI,
> and synchronize file-watcher events to the UI.

## Why This Epic Exists

Epics 1 and 2 built the backend engine and the UI shell independently. This
epic connects them into a working product — the actual text editor that users
interact with.

## Tasks

| Task | Name | Depends On |
|---|---|---|
| [3.1](3.1-monaco-editor.md) | Monaco Editor (Code Files) | Epic 2 |
| [3.2](3.2-wysiwyg-markdown.md) | Inline WYSIWYG Markdown Editor (Tiptap) | Epic 2 |
| [3.3](3.3-omnibar.md) | Ctrl+P Omnibar — Fuzzy File Finder | 3.1 or 3.2, Task 1.5 |
| [3.4](3.4-workspace-search.md) | Ctrl+Shift+F — Workspace Content Search | 3.3, Task 1.6 |
| [3.5](3.5-git-gutter.md) | Git Gutter Markers | 3.1, Task 1.4 |
| [3.6](3.6-bidirectional-links.md) | Bidirectional Linking UI + `[[` Autocomplete | 3.2, Task 1.7 |
| [3.7](3.7-keybinding-registry.md) | Keyboard Shortcut Registry | Epic 2 |
| [3.8](3.8-file-watcher-ui-sync.md) | File Watcher → UI Sync | Task 1.3, Task 2.6 |
| [3.9](3.9-graph-view.md) | Graph View (Note Connections) | Task 1.7, Task 2.8 |

## Execution Order

```
3.1 + 3.2 (both editor surfaces, can build in parallel)
  │     │
  │     └──▶ 3.6  (wikilinks + [[ autocomplete in WYSIWYG)
  │
  ├──▶ 3.3 ──▶ 3.4  (omnibar, then search extends it)
  └──▶ 3.5           (git gutter — Monaco only)

3.7 (parallel with all above)
3.8 (parallel with all above)
3.9 (parallel, requires Activity Bar from 2.8 + link graph from 1.7)
```

## Key Design Decisions

- **Dual editor model**: Tiptap WYSIWYG for Markdown (the primary experience),
  Monaco for code files. An `EditorRouter` switches based on file extension.
- **Inline WYSIWYG, not split preview**: Markdown renders directly as you
  type. Placing the cursor on a formatted element reveals its raw syntax.
- **`[[` autocomplete**: Typing `[[` triggers an inline popup to select a
  file — core Obsidian-like interaction for knowledge linking.
- **Omnibar is extensible**: `Ctrl+P` opens file finder by default;
  `Ctrl+Shift+F` switches to content search. Future: prefix `>` for commands.
- **Git gutter is decorative**: Uses Monaco decorations API to show colored
  bars — no interactive diff hunks in MVP (Monaco/code files only).
- **Graph view**: Force-directed canvas visualization of the link graph,
  accessible as a sidebar panel via the Activity Bar.

## Exit Criteria

- Markdown files render inline with WYSIWYG formatting (headings, bold, blockquotes)
- Code files open in Monaco with syntax highlighting
- Editing has zero perceptible input lag in both editor modes
- `Ctrl+S` saves, `Ctrl+P` finds files, `Ctrl+Shift+F` searches content
- Typing `[[` triggers an autocomplete popup for file linking
- Git gutter markers match `git diff` output for code files
- `[[wikilinks]]` are styled and clickable in both editor modes
- Backlinks panel lists all files referencing the current file
- Graph view renders note connections as an interactive force-directed graph
- External file changes sync to the UI in real time
- All keyboard shortcuts work without conflicts
