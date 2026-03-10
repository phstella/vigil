# Epic 4: Advanced Editing + Expandability

This epic is post-MVP. It extends Vigil without adding Vim mode or a plugin store marketplace flow.

## Task [4.1]: Extend Omnibar Command Mode (No Marketplace)
Goal: Support `> command` mode for internal actions while preserving file/content search modes.
Commands/Code:
Create/modify files:
- `src/lib/features/omnibar/omnibar-parser.ts`
- `src/lib/features/omnibar/omnibar-store.ts`
- `src/lib/features/omnibar/Omnibar.svelte`
- `src/lib/ipc/search.ts`
Acceptance Criteria: Omnibar handles file queries, content queries, and command actions in one interface.

## Task [4.2]: Implement WASM Plugin Runtime (Rust)
Goal: Add a secure lightweight plugin execution model.
Commands/Code:
```bash
cd src-tauri
cargo add wasmtime semver toml
```
Create/modify files:
- `src-tauri/src/core/plugins/runtime.rs`
- `src-tauri/src/core/plugins/manifest.rs`
- `src-tauri/src/core/plugins/loader.rs`
- `src-tauri/src/models/plugins.rs`
Acceptance Criteria: A sample WASM plugin loads and executes within capability constraints.

## Task [4.3]: Expose Versioned Plugin API Hooks
Goal: Provide stable extension points for editor/sidebar/omnibar integrations.
Commands/Code:
Create/modify files:
- `src-tauri/src/core/plugins/api.rs`
- `src-tauri/src/commands/plugins.rs`
- `src/lib/ipc/plugins.ts`
- `src/lib/types/plugins.ts`
Acceptance Criteria: Plugin API version negotiation works and incompatible versions are rejected clearly.

## Task [4.4]: Implement Local Plugin Management Flow
Goal: Manage plugin lifecycle without a marketplace.
Commands/Code:
Create/modify files:
- `src/lib/features/plugins/PluginManagerPanel.svelte`
- `src/lib/features/plugins/plugin-manager-store.ts`
- `src/lib/ipc/plugins.ts`
- `src-tauri/src/core/plugins/registry.rs`
Acceptance Criteria: Users can add plugins from local sources and enable/disable them.

## Task [4.5]: Upgrade Graph View Capabilities
Goal: Improve graph navigation for larger knowledge bases.
Commands/Code:
Create/modify files:
- `src/lib/features/graph/GraphView.svelte`
- `src/lib/features/graph/graph-store.ts`
- `src-tauri/src/core/graph/service.rs`
Acceptance Criteria: Graph supports filtering and remains responsive on large workspaces.

## Task [4.6]: Upgrade Search Grammar and Ranking
Goal: Add advanced query operators and better ranking behavior.
Commands/Code:
Create/modify files:
- `src-tauri/src/core/content/query.rs`
- `src-tauri/src/core/content/indexer.rs`
- `src/lib/features/omnibar/omnibar-parser.ts`
Acceptance Criteria: Scoped and operator queries return relevant ranked results consistently.

## Task [4.7]: Implement Theme Customization API
Goal: Enable full CSS/theme customization with preset packs.
Commands/Code:
Create/modify files:
- `src/app.css`
- `src/lib/styles/theme.css`
- `src/lib/features/theme/theme-store.ts`
Acceptance Criteria: Users can switch presets and apply custom theme overrides safely.

## Task [4.8]: Add Extension Security/Performance QA Gates
Goal: Ensure plugin and extension features meet security and performance requirements.
Commands/Code:
Create/modify files:
- `docs/qa/test-matrix.md`
- `docs/specs/plugin-sandbox-model.md`
- `.github/workflows/ci.yml`
Acceptance Criteria: Release gates include plugin capability checks and extension performance thresholds.
