# Vigil Unified Backlog (Canonical)

This is the canonical merged plan for `master` after integrating the codex and claude planning branches.

Product identity:
- Name: `Vigil`
- Tagline: `Lightning-fast editing. Interconnected thinking.`
- Pillars: performance first, local markdown/files, keyboard-centric workflow

Scope rules:
- MVP: Epics 0-3
- Post-MVP stabilization: Epic 3.5
- Post-MVP expansion: Epic 4
- Explicitly out of scope: Vim mode, plugin store marketplace flow

## Epic 0: Repository Genesis & Governance

- Task [0.1]: Define product identity and MVP boundaries.
- Task [0.2]: Initialize toolchains (Node, Rust, rustfmt, clippy).
- Task [0.3]: Scaffold SvelteKit frontend (TypeScript, lint, prettier).
- Task [0.4]: Add Tauri v2 shell and Rust backend.
- Task [0.5]: Configure static SPA output for Tauri.
- Task [0.6]: Install Tailwind and baseline design tokens.
- Task [0.7]: Materialize strict folder structure.
- Task [0.8]: Define IPC contracts and workspace data model docs.
- Task [0.9]: Author architecture and UI mock/component map docs.
- Task [0.10]: Configure lint/format standards.
- Task [0.11]: Add CI quality gates and baseline Tauri capabilities.

## Epic 1: Backend Foundation (Rust/Tauri)

- Task [1.1]: Add backend dependencies and module entrypoints.
- Task [1.2]: Define shared request/response models and error envelope.
- Task [1.3]: Implement workspace filesystem service (`open/list/read/write/create`).
- Task [1.4]: Build incremental indexer and recursive file watcher.
- Task [1.5]: Implement fuzzy file finder service for omnibar filename mode.
- Task [1.6]: Implement content search index/service for phrase/snippet search.
- Task [1.7]: Implement git diff/status worker and event stream.
- Task [1.8]: Implement link graph and backlinks resolver.
- Task [1.9]: Implement tag extraction and index.
- Task [1.10]: Implement workspace status service.
- Task [1.11]: Add backend integration tests and performance instrumentation hooks.

## Epic 2: Frontend Skeleton (Svelte)

- Task [2.1]: Build app shell and zen title bar baseline.
- Task [2.2]: Implement left activity rail and auto-hide multi-panel sidebar.
- Task [2.3]: Implement explorer panel and file tree UI.
- Task [2.4]: Implement search panel shell.
- Task [2.5]: Implement graph panel shell.
- Task [2.6]: Implement resizable split-pane workspace (horizontal + vertical).
- Task [2.7]: Implement note pane and code pane skeletons.
- Task [2.8]: Implement floating omnibar UI.
- Task [2.9]: Implement footer status bar UI.
- Task [2.10]: Wire typed frontend IPC clients.
- Task [2.11]: Implement app state stores (editor/files/git/settings/ui).
- Task [2.12]: Implement keyboard shortcut infrastructure (`Ctrl+P`, `Ctrl+S`, `Ctrl+B`, `Ctrl+N`).

## Epic 3: Core MVP Integration

- Task [3.1]: Integrate Monaco for code files with performance-focused config.
- Task [3.2]: Integrate markdown editing lifecycle (open/save/autosave/dirty state).
- Task [3.3]: Implement live markdown render/WYSIWYG toggle while preserving raw text.
- Task [3.4]: Implement file-type routing (`.md` note flow vs code flow).
- Task [3.5]: Connect omnibar to fuzzy file search.
- Task [3.6]: Connect omnibar/content mode to phrase/snippet search.
- Task [3.7]: Implement bidirectional linking UI + `[[` autocomplete + backlinks.
- Task [3.8]: Implement git gutter markers and refresh loop.
- Task [3.9]: Implement interactive graph view (open/select/pan/zoom MVP).
- Task [3.10]: Wire file-watcher events to UI sync and cache invalidation.
- Task [3.11]: Execute performance hardening against budget gates.
- Task [3.12]: Build Linux/Windows artifacts and pass MVP smoke matrix.

## Epic 3.5: Editor Workflow Stabilization & UX Cleanup

- Task [3.5.1]: Remove default split layout/code split behavior; use one adaptive main editor pane.
- Task [3.5.2]: Add multi-tab editing with optional side-by-side mode (user-invoked only).
- Task [3.5.3]: Fix Linux Monaco crash path and add reproducible runtime validation evidence.
- Task [3.5.4]: Remove notes/files metadata blocks from explorer (Notes card + Collections summary).
- Task [3.5.5]: Render Mermaid charts in markdown notes (` ```mermaid ` code fences in preview mode).

## Epic 4: Expandability & Advanced Capabilities (No Vim, No Plugin Store)

- Task [4.1]: Extend omnibar command mode (actions/commands, no marketplace flow).
- Task [4.2]: Implement WASM plugin runtime (loader, manifest, capability sandbox).
- Task [4.3]: Expose versioned plugin API hooks.
- Task [4.4]: Add local plugin management flow (local sources, enable/disable).
- Task [4.5]: Upgrade graph capabilities (filters, large-workspace performance).
- Task [4.6]: Upgrade search grammar/ranking (operators, scoped queries).
- Task [4.7]: Add full theme customization API and preset themes.
- Task [4.8]: Add extension security/performance QA gates and release policy.

## Mapping to Source Plans

- Structured scaffold and governance baselines came from `docs/` tickets/specs.
- Detailed execution guidance came from `vigil-plan/epics/*`.
- For implementation detail, keep both sources, but follow this backlog as the single source of truth.
