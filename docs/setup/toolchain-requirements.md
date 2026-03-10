# Vigil Toolchain Requirements

## Required Tools

| Tool       | Minimum Version | Purpose                        |
|------------|----------------|--------------------------------|
| Node.js    | 20.x LTS+      | SvelteKit frontend build       |
| npm        | 10.x+          | Package management             |
| Rust       | 1.80+           | Tauri backend                  |
| Cargo      | (ships w/Rust)  | Rust build system              |
| rustfmt    | (ships w/Rust)  | Rust code formatting           |
| clippy     | (ships w/Rust)  | Rust linting                   |

## Installation

### Option A: Arch Linux (system packages)

```bash
sudo pacman -S rust nodejs npm
```

On Arch, `rustfmt` and `clippy` are bundled with the `rust` package. No additional setup needed.

### Option B: Other Linux / Windows (rustup + nvm/fnm)

#### Rust via rustup

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup default stable
rustup component add rustfmt clippy
```

#### Node via fnm (recommended)

```bash
# Install fnm
curl -fsSL https://fnm.vercel.app/install | bash

# Install and use Node LTS
fnm install --lts
fnm use lts-latest
```

Alternatively, use nvm or download from https://nodejs.org.

### Option C: Windows-specific notes

- Install Rust via https://rustup.rs (requires Visual Studio C++ build tools).
- Install Node via https://nodejs.org or fnm for Windows.
- Ensure `cargo`, `rustc`, `node`, and `npm` are on your PATH.

## Verification

After installation, run the verification commands in [verification.md](verification.md) to confirm everything is working.
