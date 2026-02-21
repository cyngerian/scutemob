# CLAUDE.md — MTG Commander Rules Engine

> **This file is the primary context document for Claude Code sessions.** Read this before
> doing anything. It tells you where the project is, what the architecture looks like,
> what conventions to follow, and what to watch out for.
>
> **Update this file** at the completion of each milestone or when major design decisions
> change. The "Current State" section should always reflect reality.

---

## Current State

- **Active Milestone**: M3 — Stack & Spell Resolution
- **Status**: In progress (M3-A + M3-B done; M3-C stack resolution next)
- **Last Updated**: 2026-02-21

### What Exists (M3 in progress)
- Everything from M2, plus:
- `blake3 = "1"` dependency for deterministic hashing
- `HashInto` trait in `state/hash.rs`: manual field-by-field hashing into `blake3::Hasher`
- `public_state_hash()` on `GameState`: hashes all publicly visible state (turn, players, public zones, effects, combat); excludes hand/library contents and history
- `private_state_hash(player)` on `GameState`: hashes hand contents, library contents (ordered), face-down cards
- 19 hashing tests: determinism (3), sensitivity (7), public/private partition (4), dual-instance proptest (3+proptest)
- **M3-A complete**: Stack foundation, mana abilities, PlayLand and TapForMana commands
  - `state/stack.rs`: `StackObject` + `StackObjectKind` (Spell, ActivatedAbility, TriggeredAbility)
  - `ManaAbility` struct in `game_object.rs` with `tap_for()` helper; `mana_abilities` field in `Characteristics`
  - `ObjectSpec.with_mana_ability()` builder method
  - New errors: `ObjectNotOnBattlefield`, `NotController`, `PermanentAlreadyTapped`, `NoLandPlaysRemaining`, `InvalidAbilityIndex`, `NotMainPhase`, `StackNotEmpty`
  - `TapForMana` and `PlayLand` commands with `LandPlayed`, `ManaAdded`, `PermanentTapped` events
  - `rules/mana.rs`: CR 605 handler; `rules/lands.rs`: CR 305.1 handler
  - 19 new tests in `tests/mana_and_lands.rs`
- **M3-B complete**: CastSpell command, casting windows, Flash, priority reset
  - `keywords: OrdSet<KeywordAbility>` added to `Characteristics` (hash.rs updated; `ObjectSpec.with_keyword()` builder method)
  - `Command::CastSpell { player, card }` — no cost/targets yet (M3-D)
  - `GameEvent::SpellCast { player, stack_object_id, source_object_id }`
  - `rules/casting.rs`: CR 601 handler — validates casting speed (instant vs sorcery), moves card to Stack zone (CR 400.7 new ID), pushes `StackObject`, resets priority to active player (CR 601.2i)
  - Sorcery speed: active player + main phase + empty stack; Flash/Instants bypass all three
  - After casting, ACTIVE PLAYER gets priority (not necessarily the caster) — this differs from PlayLand which lets caster retain
  - 12 new tests in `tests/casting.rs`
- `snapshot_perf.rs` tests use 32 MB thread stack (debug mode struct growth from M3-A)
- 168 tests passing total, zero clippy warnings

### What Exists (M2 complete)
- Everything from M1, plus:
- `Command` enum (`PassPriority`, `Concede`) in `rules/command.rs`
- `GameEvent` enum (14 variants) in `rules/events.rs` — replaces stub; `LossReason` enum
- `process_command()` free function in `rules/engine.rs` — single public entry point for all game actions
- `start_game()` for initializing the first turn
- Turn FSM in `rules/turn_structure.rs`: `STEP_ORDER`, `advance_step`, `advance_turn`, `next_player_in_turn_order`
- Priority system in `rules/priority.rs`: APNAP ordering, `pass_priority`, `grant_initial_priority`
- Turn-based actions in `rules/turn_actions.rs`: untap (CR 502.2), draw (CR 504.1), cleanup discard/damage clear (CR 514), mana pool emptying (CR 500.4)
- `Step::next()` method for step ordering (skips FirstStrikeDamage)
- `PlayerState.max_hand_size` field (default 7)
- `TurnState` additions: `extra_combats`, `in_extra_combat`, `is_first_turn_of_game`, `last_regular_active`
- Extra turn queue (LIFO) with proper normal-order resumption after extra turns
- CR 103.8: first player skips first draw
- CR 104.3b: draw from empty library → player loses
- Eliminated players skipped in turn order and priority
- Removed duplicate `turn_number` field from `GameState` (now only in `TurnState`)
- Builder additions: `at_step()`, `active_player()`, `first_turn_of_game()`, `max_hand_size()`
- New error variants: `NotPriorityHolder`, `GameAlreadyOver`, `PlayerEliminated`, `NoActivePlayers`, `LibraryEmpty`, `InvalidCommand`
- 104 tests passing (33 new M2 + 71 existing), zero clippy warnings
- Test coverage: turn structure (6), priority (7), turn actions (7), extra turns (4), concede (5), proptest invariants (4)

### What Exists (M1 complete)
- Everything from M0, plus:
- `GameState` struct with `im::OrdMap`/`OrdSet`/`Vector` for all fields (deterministic iteration)
- `GameObject` with full `Characteristics`, `ObjectStatus`, counters, attachments, timestamps
- Zone system: `ZoneId` enum (type-safe per-player/shared encoding), `Zone` (Ordered/Unordered variants)
- `ObjectId` generation via monotonic `timestamp_counter`; CR 400.7 zone-change identity in `move_object_to_zone`
- `PlayerId`, `CardId`, `PlayerState` with Commander fields (life=40, commander_tax, commander_damage matrix, poison)
- `ManaPool` with color-based add/empty/total operations
- `TurnState`, `Phase`, `Step` enums with phase mapping and priority flags
- `GameStateError` enum with `thiserror` integration
- `GameStateBuilder` + `ObjectSpec` + `PlayerBuilder` fluent test API
- `rand = "0.8"` dependency for `Zone::shuffle` (seeded Fisher-Yates)

### What Exists (M0 complete)
- Cargo workspace with 6 members: `engine`, `network`, `card-db`, `card-pipeline`, `scryfall-import`, `mcp-server`
- Card-db crate with SQLite schema (`cards`, `card_faces`, `rulings`, `card_definitions` tables)
- Scryfall bulk importer (`tools/scryfall-import`): 36,923 cards, 74,277 rulings imported
- MCP server (`tools/mcp-server`): 4 tools — `search_rules`, `get_rule`, `lookup_card`, `search_rulings`
  - CR parser: 3,114 rules in FTS5; auto-rebuild wrapper script (`run.sh`)
  - Project-scoped config in `.mcp.json` (gitignored — machine-specific paths)
- Tauri v2 + Svelte app shell (not in workspace — requires display server)
- GitHub Actions CI, `rust-toolchain.toml`, `.nvmrc`, `.gitignore`
- Docs: `docs/mtg-engine-architecture.md`, `docs/mtg-engine-roadmap.md`, `docs/mtg-engine-game-scripts.md`, `docs/mtg-engine-corner-cases.md`

### What's Next (M3 remaining)
- ~~Deterministic state hashing (Tier 1)~~ — **DONE**
- ~~M3-A: Stack foundation + mana (StackObject, ManaAbility, TapForMana, PlayLand)~~ — **DONE**
- ~~M3-B: Casting spells (CastSpell command, sorcery/instant speed, spell enters stack, priority resets)~~ — **DONE**
- M3-C: Stack resolution (all-pass → resolve top, LIFO order, move to graveyard, countering)
- M3-D: Target legality (fizzle rule, partial fizzle)
- M3-E: Triggered abilities (TriggeredAbility proper type, APNAP, intervening-if, ActivateAbility)

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
| Game Script Strategy | `docs/mtg-engine-game-scripts.md` | Engine-independent test script generation, JSON schema, replay harness design |
| Corner Case Reference | `docs/mtg-engine-corner-cases.md` | 35 known difficult interactions the engine must handle correctly |
| Network Security Strategy | `docs/mtg-engine-network-security.md` | Three-tier security: state hashing, distributed verification, Mental Poker |
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
│   ├── mtg-engine-roadmap.md
│   ├── mtg-engine-game-scripts.md
│   └── mtg-engine-corner-cases.md
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

7. **Hidden information is enforced.** The engine knows everything. In the distributed
   verification model (see `docs/mtg-engine-network-security.md`), each peer runs
   the engine independently and only knows their own private state. Cryptographic
   protocols (Mental Poker) protect hidden information. Never expose another player's
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
| 2026-02-21 | Distributed verification replaces authoritative host | Eliminates trusted host; all peers run engine independently; coordinator is lightweight; see `docs/mtg-engine-network-security.md` |
| 2026-02-21 | Three-tier network security (hashing → distributed → Mental Poker) | Tier 1 (state hashing) catches non-determinism early; Tier 2 (all peers verify) prevents tampering; Tier 3 (cryptographic dealing) protects hidden information |
| 2026-02-21 | Deterministic state hashing from M3 onward | Catching non-determinism during engine development is dramatically cheaper than discovering it during M10 networking |
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
- [ ] Check off completed deliverables in `docs/mtg-engine-roadmap.md`
- [ ] Add any new design decisions to the Decision Log
- [ ] Add any new gotchas discovered to the Pitfalls section
- [ ] Commit: `M<N>: milestone complete — <summary>`
