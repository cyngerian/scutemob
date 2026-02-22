# CLAUDE.md — MTG Commander Rules Engine

> **This file is the primary context document for Claude Code sessions.** Read this before
> doing anything. It tells you where the project is, what the architecture looks like,
> and what to watch out for.
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
- **HIGH open**: 0 — all resolved in fix phase sessions 1-9
- **MEDIUM open**: 0 — all resolved in fix phase sessions 1-9
- **~28 LOW open**: schema improvements, partial name matching, FTS trigger gaps — deferred, address opportunistically
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
| This file | `CLAUDE.md` | Current project state; session context |

**Read the architecture doc before implementing anything.**

---

## When to Load What

Before starting work, check which files apply to your task:

| Task | Load before starting |
|------|----------------------|
| Touching any file in `rules/` | `memory/gotchas-rules.md` |
| Touching any file in `state/`, `cards/`, `effects/` | `memory/gotchas-infra.md` |
| Writing or modifying tests | `memory/gotchas-infra.md` (testing gotchas) |
| Writing new code or tests | `memory/conventions.md` |
| Questioning a design decision | `memory/decisions.md` |
| Implementing a new subsystem | `docs/mtg-engine-corner-cases.md` (full) |
| Starting a new milestone | Use `/start-milestone <N>` — reads only the relevant roadmap section via Grep+offset, never the full file. |
| Writing golden tests | `docs/mtg-engine-game-scripts.md` |
| Implementing network features (M10+) | `docs/mtg-engine-network-security.md` |
| Implementing replay viewer (M9.5) | `docs/mtg-engine-replay-viewer.md` |

Use `/review-subsystem <name>` to load the right file and see open issues in one step.

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

## MCP Resources
- **Rules search**: query by rule number ("613.8") or concept ("dependency continuous effects")
- **Card lookup**: query by exact card name for oracle text, types, rulings
- **Rulings search**: query by interaction concept ("copy effect on double-faced card")

---

## Critical Gotchas

These 3 apply to nearly every session. All other gotchas are in `memory/gotchas-rules.md` and `memory/gotchas-infra.md`.

- **Object identity (CR 400.7)**: When an object changes zones, it becomes a NEW object.
  The old ObjectId is dead. Auras fall off. "When this dies" triggers reference the old
  object. This is the #1 source of bugs in MTG engines.
- **Replacement effects are NOT triggers.** They modify events as they happen. They don't
  use the stack. Getting this wrong breaks the entire event system.
- **SBAs are checked as a batch, not individually.** All applicable SBAs happen simultaneously.
  Then triggers from all of them go on the stack together (in APNAP order).

---

## Session Startup

- Use `/start-session` for orientation — it runs only `git log --oneline -5`
- Conventions: `memory/conventions.md` | Decisions: `memory/decisions.md`
- Dev environment: `.claude/CLAUDE.local.md`

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
- [ ] Update relevant memory topic files (`memory/gotchas-rules.md`, `memory/gotchas-infra.md`, `memory/conventions.md`, `memory/decisions.md`) with new learnings
- [ ] Review all new/changed files and update `docs/mtg-engine-milestone-reviews.md`:
  - Add file inventory with line counts
  - List CR sections implemented
  - Record findings (bugs, enforcement gaps, test gaps) with severity and issue IDs
  - Place deferred issues in the correct future milestone stub
  - Update the cross-milestone issue index and statistics
- [ ] Commit: `M<N>: milestone complete — <summary>`
- [ ] **Code review → fix phase** (if any HIGH or MEDIUM findings):
  - Run the code review with **Opus 4.6** (`/model opus` or start a new Opus session)
  - Opus authors a new `memory/m<N>-session-plan.md` grouping issues into sessions of 5-8 fixes each,
    ordered by subsystem, with exact file:line locations and fix descriptions
  - Work through fix sessions with **Sonnet 4.6** using the same per-session workflow:
    tests → `cargo test --all` → `cargo clippy -- -D warnings` → close issues in reviews doc → commit
  - When all sessions complete, update "Current State" and advance to the next milestone
  - LOW-only findings do not require a fix phase; collect them in the reviews doc and address opportunistically
