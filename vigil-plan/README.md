# Vigil — MVP Execution Plan

> Lightning-fast editing. Interconnected thinking.
>
> A desktop-class, cross-platform (Linux & Windows) text editor and note-taking
> application bridging the raw speed of Sublime Text with the interconnected,
> local-first knowledge management of Obsidian.

## Core Philosophy

- **Performance first** — zero input lag, 60fps scrolling, sub-50ms command responses
- **Local & Markdown-based** — plain text / `.md` files on disk, no cloud dependency, no lock-in
- **Keyboard-centric** — everything accessible via the Command Palette, hands never leave the keyboard
- **Zen mode by default** — distraction-free, dark-mode, glassmorphism aesthetic

## Aesthetic

Dark-mode with glassmorphism (acrylic blur). Unobtrusive holographic accents:
- **Teal** `#2dd4bf` — primary accent, links, active states
- **Orange** `#fb923c` — warnings, modified git lines
- **Green** `#4ade80` — success, added git lines

## Tech Stack (Strictly Enforced)

| Layer | Technology |
|---|---|
| Application Shell & Backend | **Tauri v2 + Rust** |
| File-system indexing & search | Rust (`walkdir`, `tantivy`, `fuzzy-matcher`) |
| Git diffing | Rust (`git2-rs`) |
| File watching | Rust (`notify`) |
| Frontend UI | **Svelte 5** (SvelteKit, adapter-static for SPA output) |
| Styling | **Tailwind CSS v4** (Vite plugin) |
| Markdown Editing | **Tiptap** (ProseMirror) for inline WYSIWYG rendering |
| Code Editing | **Monaco Editor** for non-Markdown source files |

## Plan Structure

The plan is divided into 4 sequential **Epics**, each containing numbered **Tasks**:

| Epic | Name | Tasks | Focus |
|---|---|---|---|
| [Epic 0](epics/epic-0-infrastructure/README.md) | Repository Genesis & Infrastructure | 0.1–0.8 | Scaffold, tooling, folder structure, lint config |
| [Epic 1](epics/epic-1-backend/README.md) | Backend Foundation (Rust/Tauri) | 1.1–1.8 | FS, git, search, file watcher, link graph, tags |
| [Epic 2](epics/epic-2-frontend/README.md) | Frontend Skeleton (Svelte/Tailwind) | 2.1–2.8 | Activity bar, multi-panel sidebar, split panes, stores |
| [Epic 3](epics/epic-3-integration/README.md) | Core Features Integration | 3.1–3.9 | WYSIWYG editor, Monaco, omnibar, graph view, git gutter |

**Total: 33 tasks**

## Execution Order & Dependencies

```
Epic 0 (all tasks) ─── sequential, must complete first
       │
       ├──▶ Epic 1 (1.1 → 1.2 → 1.3; then 1.4–1.8 parallelize after 1.2)
       │
       └──▶ Epic 2 (2.8 → 2.2 → 2.3 → 2.4 → 2.5; then 2.6 + 2.7 in parallel)
                │
                └──▶ Epic 3 (requires both Epic 1 + Epic 2)
                     3.1 + 3.2 first (both editor surfaces)
                     then 3.3–3.9 can parallelize
```

**Critical path:** `0.1 → 0.2 → 0.5 → 1.1 → 1.2 → 2.5 → 3.2 → 3.6`

## Ticket Format

Every task follows this structure:

```
Task [X.X]: [Name]
  Goal:                1-sentence summary
  Commands/Code:       Exact terminal commands or file paths to create/modify
  Acceptance Criteria: How to verify it works
```
