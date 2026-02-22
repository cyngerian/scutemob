# CLAUDE.md — MTG Commander Rules Engine

> **This file is the primary context document for Claude Code sessions.** Read this before
> doing anything. It tells you where the project is, what the architecture looks like,
> what conventions to follow, and what to watch out for.
>
> **Update this file** at the completion of each milestone or when major design decisions
> change. The "Current State" section should always reflect reality.

---

## Current State

- **Active Milestone**: M8 — Replacement & Prevention Effects
- **Status**: Fix phase complete (all 9 sessions done); 336 tests passing; ready to start M8
- **Last Updated**: 2026-02-22

### What Exists (M7 complete, includes M0-M6)
- `cards/`: CardDefinition framework (30+ Effect primitives), 50 hand-authored cards, CardRegistry
- `effects/`: Full effect execution engine (DealDamage, GainLife, DrawCards, ExileObject, CreateToken, SearchLibrary, ForEach, Conditional, etc.)
- `rules/`: Turn structure, priority, stack, SBAs, layer system (dependency-based), combat (declare/damage), casting, resolution
- `testing/`: Script replay harness, 7 approved game scripts, 303 tests
- Deferred to M8+: replacement/prevention effects; damage prevention; "ETB tapped" replacement; zone-change choice for commander

### Known Issue Summary (from code reviews)
- **21 HIGH open**: unwrap/expect violations (8), combat validation gaps (5), effect bugs (5), hash gap (1), target validation (1), concede edge case (1)
- **~29 MEDIUM open**: SBA-layer integration, integer casts, card definition gaps, concede/cleanup, ForEach, duplicated validation
- **Full details**: `docs/mtg-engine-milestone-reviews.md`

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
| Milestone Code Reviews | `docs/mtg-engine-milestone-reviews.md` | Per-milestone code review findings, file inventories, issue tracking |
| Replay Viewer Design | `docs/mtg-engine-replay-viewer.md` | M9.5 game state stepper: architecture, API, Svelte components, shared-component strategy |
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
│   ├── mtg-engine-corner-cases.md
│   └── mtg-engine-replay-viewer.md
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
    └── replay-viewer/                (M9.5: axum + Svelte game state stepper — see docs/mtg-engine-replay-viewer.md)
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

9. **Every card in a game must have a `CardDefinition` before the game starts.** The deck
   builder enforces this. No mid-game discovery, no graceful degradation during play. The
   rewind/replay/pause system depends on a complete and accurate state history from turn 1 —
   a card whose abilities silently never fired produces a corrupted history that cannot be
   rewound to correctly. Unimplemented cards are surfaced at deck-building time with clear
   messaging, not silently ignored at game time.

---

## Key Design Decisions Log

Non-obvious decisions where the alternative was tempting. Basics (Rust, im-rs, Command/Event,
SQLite, crate separation) are covered by Architecture Invariants above.

| Date | Decision | Rationale |
|------|----------|-----------|
| 2026-02-21 | Distributed verification replaces authoritative host | Eliminates trusted host; all peers run engine independently; see `docs/mtg-engine-network-security.md` |
| 2026-02-21 | M4 legendary rule auto-keeps newest (highest ObjectId) | Real player choice needs a Command that didn't exist yet; auto-newest is deterministic and testable |
| 2026-02-21 | Rewind/pause/manual mode are network/UI features, not engine | Engine only needs `reveals_hidden_info()` on GameEvent (~10 lines); rest lives in M10/M11 |
| 2026-02-22 | Games cannot start with any unimplemented card | Graceful degradation corrupts state history; unimplemented cards blocked at deck-building time |
| 2026-02-22 | Card pipeline: scripted-first, LLM-assisted second | Scryfall structured fields + pattern library handles ~70-80%; LLM for the unmatched tail only |
| 2026-02-22 | `enrich_spec_from_def` populates ObjectSpec from definitions | `ObjectSpec::card()` creates naked objects; enrichment makes PlayLand/TapForMana/casting work |

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
- **All existing tests use 1, 2, or 4 players.** The engine is designed for N players, but
  6-player scenarios are untested. Priority rotation (6 passes to resolve), combat with 5
  defenders, and APNAP ordering with 6 players need dedicated test cases. Add these in M9.
  A `GameStateBuilder::six_player()` convenience method should be added alongside the tests.
- **`ObjectSpec::card` + `.with_types([Creature])` creates a creature with `toughness: None`.**
  SBAs (704.5f/g/h) skip creatures with `None` toughness to avoid false positives.
  Use `ObjectSpec::creature(owner, name, power, toughness)` for any creature that SBAs should affect.
- **Don't test implementation details.** Test observable behavior. "After casting Lightning
  Bolt targeting player B, player B's life is 37" — not "the stack has one item of type
  InstantSpell with damage field 3."
- **Randomness in tests**: Libraries are shuffled. Use a seeded RNG (`StdRng::seed_from_u64`)
  in tests for deterministic library order.
- **Golden tests are fragile**: If you change the Event format, all golden test files break.
  Version the golden test schema.
- **1-player `start_game` doesn't reach Cleanup.** `active_players().len() == 1` makes
  `is_game_over()` return `true` (one winner), so `enter_step` emits a `GameOver` event
  and returns immediately — it never advances through cleanup. Tests that need cleanup to
  fire (e.g., verifying `UntilEndOfTurn` expiry via the full turn cycle) must use 2+ players.
  Layer system tests that only call `calculate_characteristics` can safely use 1 player.
- **Combat step turn-based actions fire when ENTERING the step, not exiting it.** When all
  players pass priority and `advance_step` transitions to e.g. `FirstStrikeDamage`, the
  `enter_step` call immediately runs `first_strike_damage_step()` and emits `CombatDamageDealt`
  + any SBA events (e.g. `CreatureDied`). These events appear in the `pass_all` that transitions
  INTO the step. The `pass_all` that exits the step (players passing priority within the step
  to move to the next step) produces events from entering the NEXT step, not the current one.
  Tests that look for first-strike damage must capture events from the first `pass_all`, not the second.
- **CR 510.1c damage assignment: last blocker gets ALL remaining power (no trample).** The
  "minimum lethal before moving to next blocker" rule only applies when there are subsequent
  blockers in the damage order. The final (or only) blocker without trample absorbs all remaining
  attacker power — it is not capped at lethal. Trample + last blocker: assign lethal, rest to player.
- **Game script harness: `ObjectSpec::card()` creates naked objects.** No card types, no mana
  abilities, no keywords, no power/toughness. Call `enrich_spec_from_def()` to populate these
  from `all_cards()` definitions. Without it: PlayLand fails ("not a land"), TapForMana fails
  (no ability at index 0), instant-speed casts fail for non-active players, permanents go to
  graveyard instead of battlefield.
- **`CardDefinition` struct literals need `..Default::default()` for non-creature cards** after
  the `power`/`toughness` fields were added. Bulk-fixing with a Python depth-counter script
  will miss definitions that contain nested `TokenSpec { power, toughness }` — fix those 3
  manually (Beast Within, Generous Gift, Swan Song).
- **`CardRegistry::new()` returns `Arc<CardRegistry>`** — do NOT wrap in `Arc::new()` again.
- **`EffectAmount::PowerOf(target)` returns 0 if `target.power == None`.** Creatures built
  with `ObjectSpec::card()` have `power: None`; `enrich_spec_from_def` must propagate
  `def.power`/`def.toughness` to fix `GainLife { amount: PowerOf(...) }` spells like STP.

---

## Development Environment

See `.claude/CLAUDE.local.md` for full environment setup (Debian VM, Windows PC, CI).

**Quick reference**: Debian VM over SSH (no display server). Tauri/UI deferred to Windows PC at M11+.

---

## Session Startup

- Current state and conventions are in this file (auto-loaded every session)
- Roadmap: read `docs/mtg-engine-roadmap.md` on demand when needed
- Use `/start-session` for orientation — it runs only `git log --oneline -5`

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
- [ ] Review all new/changed files and update `docs/mtg-engine-milestone-reviews.md`:
  - Add file inventory with line counts
  - List CR sections implemented
  - Record findings (bugs, enforcement gaps, test gaps) with severity and issue IDs
  - Place deferred issues in the correct future milestone stub
  - Update the cross-milestone issue index and statistics
- [ ] Commit: `M<N>: milestone complete — <summary>`
