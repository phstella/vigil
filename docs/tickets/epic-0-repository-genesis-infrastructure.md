# Epic 0: Repository Genesis & Infrastructure

## Task [0.1]: Initialize Toolchains -- DONE
Goal: Standardize Linux/Windows contributor environments for Rust + Node development.
Commands/Code:
```bash
node --version
npm --version
rustc --version
cargo --version
cargo fmt --help
cargo clippy --help
```
Note: On Arch Linux, use system `rust` package (includes rustfmt + clippy). On other platforms, use `rustup` with `rustup component add rustfmt clippy`. See `docs/setup/toolchain-requirements.md`.

Verified versions (2026-03-10):
- Node v25.7.0, npm 11.11.0
- rustc 1.93.1, cargo 1.93.1
- rustfmt 1.8.0, clippy 0.1.93

Acceptance Criteria: Tool versions resolve without error and `cargo fmt --help` plus `cargo clippy --help` both run. **PASSED**

## Task [0.2]: Scaffold SvelteKit Frontend (TypeScript)
Goal: Create the frontend SPA base that Tauri will host.
Commands/Code:
```bash
mkdir vigil
cd vigil
npx sv create .
npm install
```
Select prompts: `SvelteKit minimal`, `TypeScript`, `ESLint`, `Prettier`.
Acceptance Criteria: `npm run dev` launches on `http://localhost:5173`.

## Task [0.3]: Add Tauri Shell and Rust Backend
Goal: Add desktop runtime and Rust project under `src-tauri/`.
Commands/Code:
```bash
npm install -D @tauri-apps/cli@latest
npx tauri init
```
Prompt values:
- App name `Vigil`
- Window title `Vigil`
- Dev URL `http://localhost:5173`
- Frontend dev command `npm run dev`
- Frontend build command `npm run build`
- Frontend dist `../build`
Acceptance Criteria: `npx tauri dev` opens a desktop window.

## Task [0.4]: Configure SvelteKit Static SPA Output for Tauri
Goal: Ensure a deterministic static build for desktop packaging.
Commands/Code:
```bash
npm install -D @sveltejs/adapter-static
```
Modify files:
- `svelte.config.js`
- `src/routes/+layout.ts`
- `src-tauri/tauri.conf.json`
Acceptance Criteria: `npm run build` emits `build/` and `npx tauri dev` serves the built app without SSR errors.

## Task [0.5]: Install Tailwind and Global Style Entry
Goal: Enable utility-first styling for the mock-aligned interface.
Commands/Code:
```bash
npm install tailwindcss @tailwindcss/vite
```
Modify files:
- `vite.config.ts`
- `src/app.css`
- `src/routes/+layout.svelte`
Acceptance Criteria: Tailwind classes apply to rendered UI.

## Task [0.6]: Materialize Strict Folder Structure
Goal: Create feature-oriented directories mapped to the mocked UI zones.
Commands/Code:
```bash
mkdir -p docs/architecture docs/tickets
mkdir -p src/lib/components/{layout,chrome}
mkdir -p src/lib/features/{workspace,explorer,editor,omnibar,links,git,status,theme}
mkdir -p src/lib/{stores,ipc,styles,types,utils}
mkdir -p src-tauri/src/{commands,core,models,state,events}
mkdir -p src-tauri/src/core/{fs,index,search,git,links}
mkdir -p src-tauri/{tests,capabilities}
```
Acceptance Criteria: All listed directories exist and match `docs/architecture/01-repository-architecture.md`.

## Task [0.7]: Configure Lint/Format Standards
Goal: Enforce consistent code quality across Rust and frontend stacks.
Commands/Code:
```bash
npm install -D eslint prettier
cargo fmt --manifest-path src-tauri/Cargo.toml --all
cargo clippy --manifest-path src-tauri/Cargo.toml --workspace --all-targets -- -D warnings
npx eslint .
npx prettier . --check
```
Create/modify files:
- `package.json`
- `.prettierrc`
- `.prettierignore`
- `.editorconfig`
Acceptance Criteria: All lint and format checks pass locally.

## Task [0.8]: Author Mock-Driven UI Spec
Goal: Convert the screenshot into an implementation contract before coding features.
Commands/Code:
Create/modify files:
- `docs/architecture/02-ui-mock-component-map.md`
- `docs/architecture/01-repository-architecture.md`
Acceptance Criteria: Every visible UI region in the mock maps to a component and backend dependency.

## Task [0.9]: Establish CI Quality Gates
Goal: Prevent regressions in formatting, linting, builds, and tests.
Commands/Code:
Create/modify file:
- `.github/workflows/ci.yml`
Include commands:
```bash
npm ci
npm run build
npx eslint .
npx prettier . --check
cargo fmt --check --manifest-path src-tauri/Cargo.toml
cargo clippy --manifest-path src-tauri/Cargo.toml --workspace --all-targets -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml
```
Acceptance Criteria: CI runs on push/PR and blocks merges on failures.
