# tig-rs TODO (Feature Checklist)

Use this as our working plan for the Rust refactor. Check items as we complete them; annotate partials.

## MVP
- [x] View framework and router (List, Diff, Pager)
- [ ] Async git loading (revwalk, patch) with UI redraw
- [ ] Basic search in views (/, n/N, wrap-search)
- [x] Settings persist/load (wrap, syntax_highlight)
- [ ] Help/keys footer hints and a minimal Help view (footer hints exist; Help view missing)
- [ ] Error/status reporting (non-blocking popups/toasts)

## Core Views
- [ ] Main (log): graph, refs/tags, dates, overflow highlighting (basic list only)
- [ ] Reflog: list, checkout/reset
- [ ] Refs: list, checkout/delete
- [ ] Status: index/worktree listing, stage/unstage, revert
- [ ] Stage: hunk/line staging, split-chunk, next hunk (@)
- [x] Diff: show patch, scroll, wrap, color (basic diff + optional code syntax)
- [~] Pager: large text, wrap, basic diff coloring (exists; not diff-aware yet)
- [ ] Blob: file viewer with wrap/number
- [ ] Blame: annotate + file navigation
- [ ] Grep: run git grep, list → open result

## Navigation & Layout
- [x] View stack (enter/back)
- [ ] Focus switching, view-next
- [ ] Horizontal/vertical splits, auto mode, size controls
- [ ] Mouse support (scroll, focus), wheel-cursor behavior
- [ ] Smooth scrolling, half/full page scroll

## Keybindings & Commands
- [ ] tigrc parser (subset → full)
- [ ] Default bindings parity; Vim-like contrib bindings
- [ ] :toggle options; :goto HEAD; search prompts
- [ ] Conflict detection for keymaps; dynamic Help shows active bindings
- [ ] Editor integration (open file/line)

## Rendering & Theming
- [x] Always color diff headers and +/- lines
- [x] Optional code syntax highlighting (fallback inline rules)
- [ ] Tree-sitter syntax highlighting (optional feature)
- [ ] Themes + git color mapping; line-graphics modes; overflow delimiter
- [ ] Commit title refs/graph toggles; id/date/author display toggles

## Git Features
- [x] Repo discovery, recent commits (revwalk)
- [x] Show commit patch (single parent)
- [ ] Multi-parent merges; select parent
- [ ] Status/diff-index, pathspec filters
- [ ] Actions: cherry-pick, checkout, reset, stash (apply/pop/drop) with prompts
- [ ] Configurable diff-options, word-diff, diff-highlight integration

## Search & Filter
- [ ] In-view incremental search; smart-case/ignore-case
- [ ] Wrap-search option
- [ ] File/revision filters (file-args/rev-args toggles)

## Performance
- [ ] Background jobs via channels; cancelable tasks
- [ ] Virtualized lists for large repos; incremental render
- [ ] Caching (commit metadata, diff previews)

## Persistence
- [ ] History file (~/.tig_history) for prompts
- [x] Settings at $XDG_CONFIG_HOME/tig-rs/config.toml
- [ ] Restore last view/layout (optional)

## Quality & Releases
- [ ] Unit/integration tests (router, views, git ops)
- [ ] Clippy + fmt gates; CI for Linux/macOS/Windows
- [ ] Benchmarks on large repos
- [ ] Packaging: Homebrew, deb/rpm, Scoop; shell completions

