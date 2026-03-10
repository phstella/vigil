# Toolchain Verification

Run these commands to verify your development environment is ready for Vigil.

## Commands

```bash
# Node / npm
node --version          # expect: v20.x or higher
npm --version           # expect: 10.x or higher

# Rust / Cargo
rustc --version         # expect: 1.80 or higher
cargo --version         # expect: matches rustc version

# rustfmt
cargo fmt --help        # expect: help text, no errors
rustfmt --version       # expect: version string

# clippy
cargo clippy --help     # expect: help text, no errors
```

## Expected output (reference from current dev machine)

```
$ node --version
v25.7.0

$ npm --version
11.11.0

$ rustc --version
rustc 1.93.1 (01f6ddf75 2026-02-11)

$ cargo --version
cargo 1.93.1 (083ac5135 2025-12-15)

$ rustfmt --version
rustfmt 1.8.0

$ cargo fmt --help
This utility formats all bin and lib files of the current crate using rustfmt.
...

$ cargo clippy --help
Checks a package to catch common mistakes and improve your Rust code.
...
```

## Troubleshooting

- **`cargo fmt` not found**: Install rustfmt via `rustup component add rustfmt` (rustup installs) or ensure the `rust` package includes it (Arch Linux).
- **`cargo clippy` not found**: Install clippy via `rustup component add clippy` (rustup installs) or ensure the `rust` package includes it (Arch Linux).
- **`rustup` not found on Arch**: This is expected if using system packages. Arch bundles rustfmt and clippy with `rust`.
- **PATH issues on Windows**: Ensure `%USERPROFILE%\.cargo\bin` is in your PATH after installing via rustup.
