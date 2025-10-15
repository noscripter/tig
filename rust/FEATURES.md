## Feature Parity Matrix (tig → tig-rs)

Scope: Track parity for core Tig features to guide the Rust rewrite.

Status legend: [ ] not started, [~] partial, [x] complete

Views
- [~] Main (log list): list, move, open, footer hints
- [ ] Reflog: list, checkout/reset
- [ ] Refs: list, checkout/delete
- [ ] Status: working tree summary, stage/unstage, revert
- [ ] Stage: hunk/line staging, split
- [~] Diff: show patch, scroll, wrap, color
- [~] Pager: show text, scroll, wrap
- [ ] Blob: view file content
- [ ] Blame: annotate file, navigate
- [ ] Grep: search results list/open

Navigation & UI
- [x] Basic event loop (crossterm/ratatui)
- [ ] View router (stack/back)
- [ ] Horizontal/vertical splits, size controls
- [ ] Mouse support and wheel behavior
- [ ] Persisted history (~/.tig_history)
- [ ] Help view with dynamic keybindings

Git interactions
- [x] Repo discovery
- [x] Recent commits (revwalk)
- [~] Show commit patch (single parent)
- [ ] Multi-parent merges, pick parent
- [ ] Status/diff-index, pathspec filters
- [ ] Cherry-pick, checkout, reset (prompted)

Search & Filter
- [ ] In-view search with wrap
- [ ] Ignore-case smart/yes/no
- [ ] File/rev filters (rev-args/file-args)

Config & Keybindings
- [~] Basic settings (TOML, wrap)
- [ ] tigrc parser (subset → full)
- [ ] Toggle options via :toggle and keys
- [ ] Default bindings parity, custom bindings

Rendering & Theming
- [~] Diff header/+/− coloring
- [~] Heuristic code syntax highlight (inline)
- [ ] Tree-sitter syntax highlight (optional)
- [ ] Git color mappings, themes

Quality
- [ ] Unit/integration tests
- [ ] Benchmarks on large repos
- [ ] CI (fmt, clippy, test, build)

Notes
- Focus MVP on: Main/Log, Diff, Pager, Status; then Stage, Refs/Reflog, Blame, Grep.
- Keep UX compatible with Tig where reasonable; deviations documented.

