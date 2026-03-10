# Vigil

Lightning-fast editing. Interconnected thinking.

Desktop-class, cross-platform (Linux/Windows) text editor and note-taking app.

- Performance first (zero input lag)
- Local-first (`.txt`/`.md` files)
- Keyboard-centric workflow (`Ctrl+P` omnibar)
- Stack: Tauri + Rust backend, SvelteKit frontend, Tailwind styling

## MVP Scope

Epics 0-3 define the MVP baseline. Epic 4 is post-MVP expansion.

**Out of scope:** Vim mode, plugin store/marketplace flow.

## Prerequisites

| Tool    | Minimum | Install guide                                        |
|---------|---------|------------------------------------------------------|
| Node.js | 20.x+  | [docs/setup/toolchain-requirements.md](docs/setup/toolchain-requirements.md) |
| npm     | 10.x+  | (ships with Node)                                    |
| Rust    | 1.80+  | [docs/setup/toolchain-requirements.md](docs/setup/toolchain-requirements.md) |
| rustfmt | any     | Bundled with Rust or `rustup component add rustfmt`  |
| clippy  | any     | Bundled with Rust or `rustup component add clippy`   |

Run [`docs/setup/verification.md`](docs/setup/verification.md) commands to confirm your environment.

## Docs

Planning docs live in [docs/README.md](docs/README.md).
