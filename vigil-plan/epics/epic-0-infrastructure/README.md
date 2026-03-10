# Epic 0: Repository Genesis & Infrastructure

> Scaffold the complete Tauri + SvelteKit + Tailwind project from scratch,
> establish the folder structure, configure all tooling, and verify the
> build pipeline works end-to-end.

## Why This Epic Exists

Nothing else can begin until the repository is scaffolded, dependencies
resolve, the dev server launches a native window, and the linting/formatting
pipeline is in place. This epic is the foundation for all subsequent work.

## Tasks

| Task | Name | Depends On |
|---|---|---|
| [0.1](0.1-scaffold-sveltekit.md) | Scaffold SvelteKit Project | — |
| [0.2](0.2-initialize-tauri.md) | Initialize Tauri v2 | 0.1 |
| [0.3](0.3-static-spa-output.md) | Configure Static SPA Output | 0.1 |
| [0.4](0.4-tailwind-css.md) | Install & Configure Tailwind CSS v4 | 0.1 |
| [0.5](0.5-rust-dependencies.md) | Add Rust Dependencies | 0.2 |
| [0.6](0.6-rust-module-skeleton.md) | Create Rust Module Skeleton | 0.5 |
| [0.7](0.7-linting-formatting.md) | Configure Linting & Formatting | 0.1, 0.2 |
| [0.8](0.8-tauri-capabilities.md) | Configure Tauri Capabilities & Permissions | 0.2 |

## Execution Order

```
0.1 ──▶ 0.2 ──▶ 0.5 ──▶ 0.6
  │       │
  │       ├──▶ 0.7
  │       └──▶ 0.8
  ├──▶ 0.3
  └──▶ 0.4
```

Tasks 0.3 and 0.4 can run in parallel with 0.2.
Tasks 0.7 and 0.8 can run in parallel after 0.2.

## Exit Criteria

- `npx tauri dev` opens a native window with the Svelte page rendered
- `npx tauri build` produces a distributable binary
- Tailwind utility classes and custom `vigil-*` tokens render correctly
- `cargo check` passes with all Rust crates resolved
- `npm run lint` and `npm run format` complete without errors
