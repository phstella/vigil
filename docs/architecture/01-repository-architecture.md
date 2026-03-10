# Vigil Repository Architecture

## Purpose
Define a strict separation between the Rust/Tauri backend and the Svelte frontend while preserving fast iteration and clear ownership.

## Top-Level Structure

```text
vigil/
  docs/
    architecture/
      01-repository-architecture.md
      02-ui-mock-component-map.md
    tickets/
      00-mvp-execution-order.md
      epic-0-repository-genesis-infrastructure.md
      epic-1-backend-foundation.md
      epic-2-frontend-skeleton.md
      epic-3-core-features-integration.md
      epic-4-advanced-editing-expandability.md
  src/
    routes/
    lib/
      components/
        layout/
        chrome/
      features/
        workspace/
        explorer/
        editor/
        preview/
        graph/
        omnibar/
        vim/
        links/
        git/
        status/
        plugins/
        theme/
      stores/
      ipc/
      styles/
      types/
      utils/
  src-tauri/
    capabilities/
    src/
      commands/
      core/
        fs/
        index/
        content/
        search/
        graph/
        git/
        links/
        plugins/
      models/
      state/
      events/
    tests/
```

## Ownership Rules
- `src-tauri/src/core/*`: business logic and performance-critical engines.
- `src-tauri/src/commands/*`: thin Tauri command wrappers over core services.
- `src-tauri/src/core/plugins/*`: WASM plugin runtime, plugin loading, and sandbox policy.
- `src/lib/ipc/*`: typed frontend gateway to backend commands and events.
- `src/lib/components/*`: reusable UI primitives and shell/chrome elements.
- `src/lib/features/*`: feature-specific UI state and orchestration.

## UI Composition Target (from mock + expansion chat)
- Primary rail: app icons and workspace switch affordances.
- Explorer sidebar: collections, notes list, counts, quick filters.
- Center pane: markdown note title/body and highlight blocks.
- Right pane: code editor surface and git gutter.
- Overlay: `Ctrl+P` omnibar centered near top.
- Footer: status bar with branch/sync/metrics/version.
- Expansion view: note graph panel and live markdown render toggle.

## API Surface Policy
- Core operations are command-driven (`invoke`) with typed payloads.
- Push updates use event channels for index/git/plugin state.
- Plugin APIs are capability-scoped and versioned.
