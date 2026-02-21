# CLAUDE.md — MTG Commander Rules Engine

> **This file is the primary context document for Claude Code sessions.** Read this before
> doing anything. It tells you where the project is, what the architecture looks like,
> what conventions to follow, and what to watch out for.
>
> **Update this file** at the completion of each milestone or when major design decisions
> change. The "Current State" section should always reflect reality.

---

## Current State

- **Active Milestone**: M0 — Project Scaffold & Data Foundation
- **Status**: In progress — scaffold complete, data pipeline remaining
- **Last Updated**: 2026-02-20

### What Exists
- Git repository initialized
- Cargo workspace with 4 crates: `engine`, `network`, `card-db`, `card-pipeline`
- Engine crate with `im-rs`, `serde`, `thiserror` dependencies; module stubs for `state`, `rules`, `cards`, `effects`
- Network crate with `tokio` dependency (placeholder)
- Card-db crate with `rusqlite` (bundled) dependency (placeholder)
- Card-pipeline crate (placeholder)
- im-rs proof-of-concept tests (4 passing: clone independence, structural sharing, vector ordering, snapshot feasibility)
- GitHub Actions CI pipeline (`cargo fmt --check`, `cargo clippy -D warnings`, `cargo test --all`)
- `rust-toolchain.toml` (stable), `.nvmrc` (22), `.gitignore`
- Architecture doc: `docs/mtg-engine-architecture.md`
- Development roadmap: `docs/mtg-engine-roadmap.md`

### What's Next (remaining M0 deliverables)
- Set up Tauri app shell with Svelte frontend
- Scryfall bulk data importer → SQLite
- SQLite schema for cards, rulings, card_faces, card_definitions
- Configure MCP server for CR + card data RAG

---

## Project Overview

We are building an MTG rules engine targeting **Commander format** (4-player multiplayer) with
**networked play**. The engine is written in **Rust**, the desktop app uses **Tauri v2** with a
**Svelte** frontend.

The engine is a standalone library crate with no UI or network dependencies. It can be tested
entirely in isolation. The network layer wraps the engine. The Tauri app wraps the network layer.

### Primary Documents

| Document | Location | Purpose |
|----------|----------|---------|
| Architecture & Testing Strategy | `docs/mtg-engine-architecture.md` | Why decisions were made; system design; testing approach |
| Development Roadmap | `docs/mtg-engine-roadmap.md` | What to build and in what order; milestone definitions |
| This file | `CLAUDE.md` | Current project state; coding conventions; session context |

**Read the architecture doc before implementing anything.** It explains the rationale behind
the state model, layer system, command/event pattern, and testing strategy. The roadmap tells
you what the current milestone's deliverables and acceptance criteria are.

---

## Repository Structure

```
mtg-engine/
├── CLAUDE.md                         ← you are here
├── Cargo.toml                        (workspace root)
├── docs/
│   ├── mtg-engine-architecture.md
│   └── mtg-engine-roadmap.md
├── crates/
│   ├── engine/                       (core rules engine — THE product)
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── state/                (GameState, zones, objects, players)
│   │   │   ├── rules/                (turn structure, priority, stack, SBAs, layers, combat)
│   │   │   ├── cards/                (CardDefinition types, keyword implementations)
│   │   │   └── effects/              (effect resolution, replacement effects, triggers)
│   │   └── tests/
│   │       ├── rules/                (unit tests by CR section)
│   │       ├── interactions/         (multi-card integration tests)
│   │       ├── golden/               (full game replay tests)
│   │       └── properties/           (property-based fuzz tests)
│   ├── network/                      (WebSocket host/client, lobby, state sync)
│   ├── card-db/                      (SQLite schema, queries, Scryfall import)
│   └── card-pipeline/                (dev tool: oracle text → CardDefinition generation)
├── tauri-app/                        (Tauri v2 desktop application)
│   ├── src-tauri/                    (Rust backend: IPC bridge to engine + network)
│   └── src/                          (Svelte frontend)
├── test-data/
│   ├── golden-games/                 (JSON game replay files)
│   ├── corner-cases.json             (curated interaction test cases)
│   └── test-cards/                   (synthetic cards for testing)
└── tools/
    ├── scryfall-import/              (bulk data download + SQLite population)
    └── replay-viewer/                (future: visualize game replays)
```

---

## Coding Conventions

### Rust Style

- **Edition**: 2021
- **Formatting**: `rustfmt` with default settings. Run `cargo fmt` before every commit.
- **Linting**: `cargo clippy` with `-D warnings`. No clippy warnings allowed in CI.
- **Error handling**: Use `thiserror` for library errors, `anyhow` in binaries/tools only.
  The engine crate uses typed errors — never `unwrap()` or `expect()` in engine logic.
  Tests may use `unwrap()`.
- **Naming**:
  - Types: `PascalCase`
  - Functions/methods: `snake_case`
  - Constants: `SCREAMING_SNAKE_CASE`
  - Modules: `snake_case`
  - Test functions: `test_<rule_number_or_feature>_<scenario>` (e.g., `test_704_5f_zero_toughness_creature_dies`)

### Comprehensive Rules Citation Format

Every rules implementation MUST cite the CR section it implements. Use this format:

```rust
/// Implements CR 704.5f: "If a creature has toughness 0 or less, it's put into
/// its owner's graveyard. Regeneration can't replace this event."
fn check_zero_toughness(state: &GameState) -> Vec<GameEvent> {
    // ...
}
```

For tests, cite the rule AND the source of the test case:

```rust
#[test]
/// CR 704.5f — creature with 0 toughness dies as SBA
/// Source: CR example under 704.5f
fn test_704_5f_zero_toughness_creature_dies() {
    // ...
}

#[test]
/// CR 613.10 — Humility + Opalescence interaction
/// Source: CR example under 613.10, confirmed by Forge engine
fn test_613_10_humility_opalescence() {
    // ...
}
```

### Testing Conventions

- **Test location**: Unit tests in `crates/engine/tests/`, not inline `#[cfg(test)]` modules.
  This keeps the source files clean and allows tests to access the public API only (black-box testing).
- **GameStateBuilder**: Always use the builder to construct test states. Never manually construct
  `GameState` structs — the builder ensures invariants.
- **One assertion focus per test**: Tests should have a clear, single behavior they're verifying.
  Multiple related assertions are fine, but the test name should describe the specific behavior.
- **Test naming**: `test_<system>_<scenario>_<expected_behavior>`
  - Good: `test_sba_creature_zero_toughness_goes_to_graveyard`
  - Good: `test_priority_all_four_players_pass_stack_resolves`
  - Bad: `test_combat` (too vague)
  - Bad: `test_1` (meaningless)
- **Golden test format**: JSON files in `test-data/golden-games/`. Schema documented in
  architecture doc Section 6.4.
- **Property tests**: Use `proptest` crate. Define invariants in `tests/properties/`.

### Commit Conventions

- **Format**: `M<number>: <short description>` (e.g., `M1: implement GameState struct with zone system`)
- **PR scope**: One logical change per PR. A PR can span multiple files but should have one purpose.
- **Tests required**: Every PR that changes engine logic must include or update tests.
- **Benchmark check**: If the PR touches state cloning, layer calculation, or SBA checks,
  run benchmarks and note any regression.

### Dependencies Policy

- **Engine crate**: Minimal dependencies. `im` (persistent data structures), `serde` (serialization),
  `thiserror` (error types). No async runtime, no IO, no network, no UI.
- **Network crate**: `tokio`, `tokio-tungstenite` or `axum`, `serde`, `rmp-serde` (MessagePack).
- **Card-db crate**: `rusqlite`, `serde`.
- **Tauri app**: `tauri`, `serde`, whatever the frontend needs.

The engine crate must NEVER depend on the network, card-db, or tauri-app crates. Information
flows inward: the app depends on network, network depends on engine. Never the reverse.

---

## Architecture Invariants

These are non-negotiable. If a change would violate any of these, stop and reconsider.

1. **Engine is a pure library.** No IO, no network, no filesystem access, no async runtime
   in the engine crate. It takes commands in and emits state changes out. Everything else
   is the caller's responsibility.

2. **Game state is immutable.** Use `im-rs` persistent data structures. State transitions
   produce new states; old states are retained for undo/replay. Never mutate state in place.

3. **All player actions are Commands.** There is no way to change game state except through
   the Command enum. This enables networking, replay, and deterministic testing.

4. **All state changes are Events.** The engine emits Events describing what happened.
   The network layer broadcasts these. The UI consumes these. Events are the single
   source of truth for "what happened."

5. **Multiplayer-first.** Priority, triggers, combat — everything is designed for N players.
   1v1 is N=2, not a special case.

6. **Commander-first.** The command zone, commander tax, commander damage, color identity —
   these are core features, not bolted-on extensions.

7. **Hidden information is enforced.** The engine knows everything. Clients receive
   projections that hide what they shouldn't see. Never send a client another player's
   hand or library order.

8. **Tests cite their rules source.** Every test references the CR section or known
   interaction it validates. Untraceable tests are technical debt.

---

## Key Design Decisions Log

Record significant decisions here so future sessions have context for WHY things are
the way they are. Format: date, decision, rationale.

| Date | Decision | Rationale |
|------|----------|-----------|
| (project start) | Rust for engine, Tauri for app | Performance for layer calculations; Tauri gives native Rust backend + web UI without Electron overhead |
| (project start) | `im-rs` for immutable state | Structural sharing makes state snapshots O(1); enables free undo/replay; fits Rust ownership model |
| (project start) | Command/Event model | Single pattern for networking, replay, testing, and undo; enforces determinism |
| (project start) | Authoritative host (not P2P) | Hidden information requires a trusted authority; simpler than consensus protocols |
| (project start) | SQLite for card data | Structured queries for card lookup; embedded DB ships with the app; no external server needed |
| (project start) | Separate engine/network/UI crates | Engine testable without IO; prevents coupling; allows future WASM compilation of engine alone |

---

## MCP Server Resources

When working on this project, the following MCP resources are available:

### Comprehensive Rules Search
- **Purpose**: Look up MTG rules by section number or concept
- **Use when**: Implementing any game rule, writing tests, resolving ambiguity
- **Query tips**: Search by rule number ("613.8") or concept ("dependency continuous effects")

### Card Data Lookup
- **Purpose**: Query oracle text, types, rulings for specific cards
- **Use when**: Implementing card definitions, writing interaction tests, verifying behavior
- **Query tips**: Search by exact card name for best results

### Rulings Search
- **Purpose**: Semantic search across all card rulings
- **Use when**: Implementing complex interactions, finding edge cases
- **Query tips**: Describe the interaction conceptually ("copy effect on double-faced card")

---

## Common Pitfalls & Gotchas

Things to watch out for, accumulated over development:

### MTG Rules Gotchas
- **Object identity (CR 400.7)**: When an object changes zones, it becomes a NEW object.
  The old ObjectId is dead. Auras fall off. "When this dies" triggers reference the old
  object. This is the #1 source of bugs in MTG engines.
- **Replacement effects are NOT triggers.** They modify events as they happen. They don't
  use the stack. Getting this wrong breaks the entire event system.
- **SBAs are checked as a batch, not individually.** All applicable SBAs happen simultaneously.
  Then triggers from all of them go on the stack together (in APNAP order).
- **The layer system dependency check must handle circular dependencies.** CR 613.8k says
  to fall back to timestamp order. If your dependency resolver can infinite-loop, it will.
- **"Commander damage" only counts COMBAT damage.** Not regular damage. And damage from
  a copy of a commander does NOT count — the copy isn't a commander.
- **Tokens cease to exist when they leave the battlefield** — but they DO briefly exist in
  the new zone first (long enough to trigger "when this dies" etc.).

### Rust Gotchas
- **`im-rs` HashMap iteration order is not deterministic** across different program runs
  (unless you use a fixed hasher). For deterministic replay, either sort before iterating
  or use `im::OrdMap`.
- **Recursive enums need `Box`** for the recursive variant. The `Effect` enum will need this
  for `Sequence(Vec<Effect>)` and `Conditional`.
- **Serialization of `im-rs` types**: `im` supports serde behind a feature flag. Enable it
  in Cargo.toml: `im = { version = "15", features = ["serde"] }`.

### Testing Gotchas
- **Don't test implementation details.** Test observable behavior. "After casting Lightning
  Bolt targeting player B, player B's life is 37" — not "the stack has one item of type
  InstantSpell with damage field 3."
- **Randomness in tests**: Libraries are shuffled. Use a seeded RNG (`StdRng::seed_from_u64`)
  in tests for deterministic library order.
- **Golden tests are fragile**: If you change the Event format, all golden test files break.
  Version the golden test schema.

---

## Development Environment

### Environment Split

Engine development (M0-M9), networking (M10), and the card pipeline (M12) are pure Rust
with zero GUI dependencies. All of this work happens on the **Debian VM** over SSH.

Tauri UI work (M11+) requires a display server and platform webview libraries. This work
happens on the **Windows PC** with the same repo. Push from one machine, pull on the other.

This split doesn't need to be solved until M11 — roughly 6+ months into the project.

### Global Installs (Debian VM — one-time setup)

```bash
# Rust toolchain manager (per-user install, manages versions globally)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# sqlite3 CLI — for ad-hoc queries during development only
# The engine uses rusqlite with the "bundled" feature, so libsqlite3-dev is NOT needed
apt install sqlite3

# git (likely already installed)
apt install git

# nvm (Node Version Manager) — manages Node.js versions per-project
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.40.1/install.sh | bash
source ~/.bashrc  # or restart shell
```

### Project-Scoped Version Pinning

The repo pins its own tool versions so any machine or CI runner reproduces the same build:

```toml
# rust-toolchain.toml (repo root) — pins Rust version
[toolchain]
channel = "stable"
```

```
# .nvmrc (repo root) — pins Node.js version
22
```

```toml
# crates/card-db/Cargo.toml — bundles SQLite, no system dependency needed
[dependencies]
rusqlite = { version = "0.32", features = ["bundled"] }
```

After cloning the repo, the full setup is:
```bash
nvm use          # activates pinned Node version from .nvmrc
cargo build      # rustup reads rust-toolchain.toml automatically
cargo test --all # verify everything works
```

### Why These Choices

| Tool | Install Scope | Rationale |
|------|--------------|-----------|
| `rustup` | Global (per-user) | Designed to be global; reads `rust-toolchain.toml` per-project automatically |
| `sqlite3` CLI | Global (apt) | Lightweight dev convenience tool for ad-hoc queries; not a build dependency |
| `libsqlite3-dev` | **Not installed** | rusqlite's `bundled` feature compiles SQLite from source — no system lib needed, more portable |
| Node.js | Project-scoped (nvm) | Prevents version conflicts across projects; `.nvmrc` pins version in repo |
| `git` | Global (apt) | Already present on most systems; no version sensitivity |

### Windows PC Setup (M11+ only — not needed until Tauri UI work)

```powershell
# Rust
winget install Rustlang.Rustup

# Node.js (use nvm-windows: https://github.com/coreybutler/nvm-windows)
# Then: nvm install 22 && nvm use 22

# Tauri prerequisites (when the time comes)
# WebView2 — pre-installed on Windows 10/11
# Tauri CLI: cargo install tauri-cli
```

Same git repo, same `rust-toolchain.toml`, same `.nvmrc`. Everything builds identically.

### CI: GitHub Actions

- Runs on: Ubuntu (Linux), Windows, macOS
- Runs: `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test --all`
- Nightly: performance benchmarks with regression alerts
- Tauri builds: cross-platform binaries via `tauri-action` (configured in M11)

---

## Session Startup Checklist

At the start of each Claude Code session:

1. Read this file (you're doing it now)
2. Check "Current State" above — what milestone are we on?
3. Check the roadmap for that milestone's deliverables and acceptance criteria
4. Check git log for recent changes: `git log --oneline -20`
5. Run tests to confirm current state: `cargo test --all`
6. Ask if there's a specific task to focus on, or continue with the next unchecked deliverable

---

## Milestone Completion Checklist

When completing a milestone:

- [ ] All deliverables checked off in the roadmap
- [ ] All acceptance criteria met
- [ ] All tests pass: `cargo test --all`
- [ ] No clippy warnings: `cargo clippy -- -D warnings`
- [ ] Formatted: `cargo fmt --check`
- [ ] Performance benchmarks run (if applicable to this milestone)
- [ ] Update "Current State" section of this file
- [ ] Update "Active Milestone" to the next milestone
- [ ] Add any new design decisions to the Decision Log
- [ ] Add any new gotchas discovered to the Pitfalls section
- [ ] Commit: `M<N>: milestone complete — <summary>`
