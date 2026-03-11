# Epic 0: Repository Genesis & Infrastructure

Canonical numbering: Tasks [0.1]–[0.11] per `00-unified-vigil-backlog.md`.

## Task [0.1]: Define Product Identity and MVP Boundaries -- DONE
Goal: Establish name, tagline, MVP scope (Epics 0-3), and out-of-scope items.
Acceptance Criteria: "Vigil" name + tagline consistent across README and docs. **PASSED**

## Task [0.2]: Initialize Toolchains -- DONE
Goal: Standardize Linux/Windows contributor environments for Rust + Node development.
Verified versions (2026-03-10):
- Node v25.7.0, npm 11.11.0
- rustc 1.93.1, cargo 1.93.1
- rustfmt 1.8.0, clippy 0.1.93
Acceptance Criteria: Tool versions resolve without error. **PASSED**

## Task [0.3]: Scaffold SvelteKit Frontend (TypeScript) -- DONE
Goal: Create the frontend SPA base that Tauri will host.
Acceptance Criteria: `npm run dev` launches on `http://localhost:5173`. **PASSED**

## Task [0.4]: Add Tauri Shell and Rust Backend -- DONE
Goal: Add desktop runtime and Rust project under `src-tauri/`.
Acceptance Criteria: `cargo check --manifest-path src-tauri/Cargo.toml` passes. **PASSED**

## Task [0.5]: Configure SvelteKit Static SPA Output for Tauri -- DONE
Goal: Ensure a deterministic static build for desktop packaging.
Acceptance Criteria: `npm run build` emits `build/` with static output. **PASSED**

## Task [0.6]: Install Tailwind and Baseline Design Tokens -- DONE
Goal: Enable utility-first styling with dark theme design tokens.
Acceptance Criteria: Tailwind classes apply to rendered UI; build passes. **PASSED**

## Task [0.7]: Materialize Strict Folder Structure -- DONE
Goal: Create feature-oriented directories mapped to the mocked UI zones.
Acceptance Criteria: All directories exist and match `docs/architecture/01-repository-architecture.md`. **PASSED**

## Task [0.8]: Define IPC Contracts and Workspace Data Model Docs -- DONE
Goal: Complete IPC command/event surface and workspace data model specs.
Acceptance Criteria: 11 MVP commands, 6 event channels, 18 data models documented. **PASSED**

## Task [0.9]: Author Architecture and UI Mock/Component Map -- DONE
Goal: Convert the design mock into architecture and component mapping docs.
Acceptance Criteria: Every UI region maps to a component and backend dependency. **PASSED**

## Task [0.10]: Configure Lint/Format Standards -- DONE
Goal: Enforce consistent code quality across Rust and frontend stacks.
Files: `.editorconfig`, `rustfmt.toml`, `clippy.toml`, `eslint.config.js`, `.prettierrc`
Scripts: `lint:all`, `format:all:check`
Acceptance Criteria: All lint and format checks pass locally. **PASSED**

## Task [0.11]: Add CI Quality Gates and Baseline Tauri Capabilities -- DONE
Goal: Prevent regressions via CI and declare baseline Tauri permissions.
Files: `.github/workflows/ci.yml`, `src-tauri/capabilities/default.json`
Acceptance Criteria: CI runs on push/PR; baseline capabilities declared. **PASSED**
