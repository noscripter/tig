## tig-rs Roadmap

Goal: A full-feature alternative Tig implementation in Rust with near-parity UX and performance, delivered incrementally with clear phases and tests.

Principles
- Compatibility first: commands, views, and defaults map to Tig where practical.
- Modular crates: isolate concerns to keep code testable and swappable.
- Async-ready UI: non-blocking git I/O and background tasks.
- Performance: incremental loading, caching, minimal redraw.

Proposed Architecture
- `tigrs-core`: settings, types, feature flags, error handling
- `tigrs-config`: tigrc parsing, option toggles (NEW)
- `tigrs-keymap`: default bindings + custom maps (NEW)
- `tigrs-git`: repo discovery, revwalk, status/diff, actions
- `tigrs-syntax`: syntax highlight providers (tree-sitter, fallback) (NEW)
- `tigrs-tui`: view framework, widgets, theming (NEW)
- `tigrs-cli`: binary wiring, event loop, routing

View System
- Trait `View`: init, update(Event), render(Frame, Rect), wants_focus, title
- `Router`: manages stack, back/forward, splits; routes keys to focused view
- Views: Main/Log, Reflog, Refs, Status, Stage, Diff, Pager, Blob, Blame, Grep, Help, Options

Data Flow
- Model-Update-View (Elm-like):
  - Messages from terminal events
  - Update mutates view state or dispatches async git jobs
  - Render reads immutable snapshot
- Background tasks via crossbeam channels or tokio (feature-gated)

Phases & Milestones
1) MVP (weeks 1–2)
   - Router + View trait + Main(Log) + Diff + Pager
   - Settings (wrap, theme) persisted; basic Help
   - Tests for router, log listing, diff render
2) Working tree (weeks 3–4)
   - Status + Stage (line/hunk, split) with safeguards
   - Search in views; options menu; mouse scroll
3) Repository views (weeks 5–6)
   - Refs, Reflog, Grep, Blob, Blame
   - Filter by rev/file args; history persistence
4) Polish & parity (weeks 7–8)
   - tigrc parser + keymap compatibility
   - Tree-sitter highlighting (optional feature)
   - CI, benchmarks, packaging

Open Questions
- How strict is tigrc compatibility vs. a simplified TOML/YAML?
- Which platforms are tier-1 (Linux/macOS/Windows)?
- External commands support scope (safety prompts, env passthrough)?

Next Steps
- Agree on architecture and phases
- Stand up `tigrs-tui` skeleton (View, Router)
- Port current CLI to View-based structure
- Add thin `tigrs-config` to parse a subset of tigrc
- Introduce tests for router and log view

