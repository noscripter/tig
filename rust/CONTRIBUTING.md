# Contributing to the Rust Implementation

Thanks for your interest in improving the Rust rewrite of Tig ("tig-rs"). This guide covers how to build, test, and contribute changes in the `rust/` workspace.

## Overview
- Workspace root: repository root (`Cargo.toml` defines a virtual workspace)
- Rust code: `rust/crates/*`
  - `tigrs-core`: configuration, shared types/utilities
  - `tigrs-git`: Git interactions using `git2`
  - `tigrs-cli`: experimental TUI CLI using `crossterm` + `ratatui`
- Lockfile: tracked at repo root `Cargo.lock`

## Prerequisites
- Rust (stable) via `rustup` (recommended)
- A C toolchain for building dependencies
  - macOS: Xcode Command Line Tools
  - Linux: `build-essential`/`gcc`, `pkg-config`
- OpenSSL and/or system libraries may be needed by transitive dependencies depending on your platform

## Build and Run
From the repository root:

```bash
cargo build --workspace
cargo run -p tigrs-cli -- --help
```

Run the CLI from a Git repository to see history and diffs:

```bash
cargo run -p tigrs-cli -- -n 100
```

## Tests
```bash
cargo test --workspace
```

## Linting and Formatting
- Format (required):
  ```bash
  cargo fmt --all
  ```
- Lint (recommended):
  ```bash
  cargo clippy --workspace --all-targets -- -D warnings
  ```

## Pull Request Checklist
- Keep PRs focused and reasonably small
- Include tests for behavior changes where practical
- Update docs (`rust/README.md`) if UX or commands change
- Ensure `cargo fmt` produces no diffs
- Run Clippy locally and address warnings
- Build and test the entire workspace before submitting

## Adding or Changing Crates
- Place new crates under `rust/crates/<name>`
- Update the root `Cargo.toml` `[workspace].members` list to include the new crate
- If exposing new features, consider gating behind Cargo features

## Commit Guidance
- Clear, imperative commit messages (e.g., "add X", "fix Y")
- Reference issues when relevant (e.g., "Fixes #123")

## Platform Notes
- `tigrs-git` uses `git2`/`libgit2`. On some platforms, you may need system libraries; if builds fail, check your OS docs for OpenSSL/pkg-config setup.

## Code Style
- Prefer stable Rust features
- Avoid introducing warnings; treat Clippy warnings as actionable
- Keep public APIs documented when they settle

## Getting Help
Open a discussion or issue with details about your platform, Rust version (`rustc -V`), and reproduction steps.

