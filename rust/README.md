# tig-rs (Rust rewrite scaffold)

This directory hosts a Rust workspace that aims to incrementally rewrite Tig.

Status:
- Minimal TUI shell using `crossterm` + `ratatui`
- Basic Git log retrieval via `git2`
- Pager view with scroll, optional wrapping, commit diff
- Simple settings (e.g., `wrap_lines`) with a TOML config at `$XDG_CONFIG_HOME/tig-rs/config.toml`

Next steps (not yet implemented):
- Keybindings and view navigation
- Diff, stage, pager, and search views
- Compatibility layer for existing `~/.tigrc` settings
- Tests and feature parity tracking

Build:
```
cd rust && cargo build --workspace
```

Run the CLI from a Git repo:
```
cd rust && cargo run -p tigrs-cli
```

Keys
- List: Enter open, j/k move, w toggle wrap, q quit
- Pager: j/k scroll, g/G top/bottom, w toggle wrap, Tab or d/p switch, q back
- Diff: j/k scroll, g/G top/bottom, w toggle wrap, Tab or d/p switch, q back
