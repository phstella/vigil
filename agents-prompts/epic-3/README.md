# Epic 3 Agent Prompts

Use these prompts in strict order (canonical Epic 3):

1. `3.1-monaco-code-editor-integration.md`
2. `3.2-markdown-editing-lifecycle.md`
3. `3.3-live-markdown-render-wysiwyg-toggle.md`
4. `3.4-file-type-routing-note-vs-code.md`
5. `3.5-omnibar-fuzzy-file-search-integration.md`
6. `3.6-omnibar-content-search-integration.md`
7. `3.7-bidirectional-linking-and-backlinks-ui.md`
8. `3.8-git-gutter-markers-and-refresh-loop.md`
9. `3.9-interactive-graph-view-mvp.md`
10. `3.10-file-watcher-ui-sync-and-cache-invalidation.md`
11. `3.11-performance-hardening-and-budget-gates.md`
12. `3.12-linux-windows-artifacts-and-smoke-matrix.md`

Hard rules:
- Source of truth: `docs/tickets/00-unified-vigil-backlog.md`.
- If any other Epic 3 doc conflicts with canonical backlog, canonical backlog wins.
- Every task prompt includes:
  - explicit `<acceptance_criteria>` bullets,
  - narrowed per-task file scope,
  - CI-parity checks (`check`, `lint`, `format:check`, `build`, `cargo fmt`, `clippy -D warnings`, `cargo test`).
- Every task must maintain append-only diary file `docs/implementation-diary/{TASK_ID}.md`.
