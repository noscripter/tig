# tig-rs (Rust rewrite scaffold)

This directory hosts the Rust code for an incremental rewrite of Tig.

Status:
- Minimal TUI shell using `crossterm` + `ratatui`
- Basic Git log retrieval via `git2`
- Pager/Diff views with scroll, optional wrapping, commit diff
- Basic inline syntax coloring for diffs (heuristic, single-line, a few languages)
- Simple settings (e.g., `wrap_lines`) with a TOML config at `$XDG_CONFIG_HOME/tig-rs/config.toml`

Next steps (not yet implemented):
- Keybindings and view navigation
- Diff, stage, pager, and search views
- Compatibility layer for existing `~/.tigrc` settings
- Tests and feature parity tracking

Build:
```
cargo build --workspace
```

Run the CLI from a Git repo:
```
cargo run -p tigrs-cli
```

Keys
- List: Enter open, j/k move, w toggle wrap, q quit
- Pager: j/k scroll, g/G top/bottom, w toggle wrap, Tab or d/p switch, q back
- Diff: j/k scroll, g/G top/bottom, w toggle wrap, Tab or d/p switch, q back

Notes on syntax coloring
- Highlighting is heuristic and kept lightweight for TUI performance.
- It infers language from diff headers (e.g., `+++ b/file.rs`).
- Supported basics: Rust, C/C++, Python, JS/TS, Go, Shell.
- Only single-line constructs are highlighted (strings, numbers, keywords, comments).

Contributing
- See `rust/CONTRIBUTING.md` for build, test, lint, and PR guidance.
