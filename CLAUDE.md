# CLAUDE.md — MTG Commander Rules Engine

> **This file is the primary context document for Claude Code sessions.** Read this before
> doing anything. It tells you where the project is, what the architecture looks like,
> and what to watch out for.
>
> **Update this file** at the completion of each milestone or when major design decisions
> change. The "Current State" section should always reflect reality.

---

## Current State

> Detailed PB-by-PB handoffs, hazards, and seed inventories live in `memory/workstream-state.md`.
> Worker sessions: append detail there, not here. CLAUDE.md tracks current snapshot only.

- **Active Milestone**: M9.5 DONE — **Card Authoring Campaign ACTIVE** (plan: `memory/card-authoring/campaign-plan-2026-05-16.md` §0 recalibration 2026-07-07; clean coverage 951/1,748 = 54.4% per `tools/authoring-report.py`)
- **Tests**: **2940 passing**; build/clippy/fmt clean
- **Abilities**: ~199 validated; 42/42 P1; 17/17 P2; 40/40 P3; 95/95 P4 implemented (9 permanent-n/a; 1 deferred: Banding)
- **Primitives**: PB-0..PB-37 + named-letter chain (PB-A/B/E/J/M/S/X/Q/Q4/N/D/P/L/T/SFT/CC-{W,B,C,A}/TS/LKI-CC/CD/LKI-Power/EWC/XS/XS-E/XA/EAT/XA2/EWC-D) all DONE. PB-Q2/Q3/Q5 reserved.
- **Last shipped**: **PB-AC3** (`scutemob-45`, merge `0bd7c7a3`) — `EffectAmount::{AttackingCreatureCount,TappedCreatureCount,HandSize}` + `LayerModification::SetBothDynamic` (Layer 7b); fixed pre-existing hash disc-26 collision; backfill incl. 4 HIGH wrong-game-state fixes (Mishra, Ashaya, Multani, Wight). Prior same-day: PB-AC2 (`scutemob-44`, MayPayThenEffect + CounterUnlessPays, 12 clean) and PB-AC1 (`scutemob-43`, untap/counter/once-per-turn, 8 clean). Next: PB-AC4 (modal & optional targeting). Known tooling nit: authoring-report.py regex misclassifies Krenko as 'empty' (no word boundary on `abilities: vec![]` match).
- **Open primitive seeds**: OOS-XA2-1/2/4/5, OOS-EWCD-1..3, OOS-EAT-1..3, OOS-XS-E-2; older OOS-XS-1/3/4, OOS-LKI-Power-1/4/5, OOS-LKI-1..4, OOS-TS-1..4 — all 0-yield defensives or card-gated; high-confidence backlog exhausted. (OOS-XA-3/XA2-3 RESOLVED by `scutemob-30`; OOS-LKI-Power-3 shipped.) Full list: `memory/primitives/pb-retriage-CC.md`.
- **Known issues**: 0 HIGH; 2 MEDIUM (pre-M8 deferred to M10+); **6 LOW open** (4 M10-gated: MR-M8-11, MR-B16-04/05/06; 2 permanent perf: MR-M1-18, MR-M6-14). Full: `docs/mtg-engine-milestone-reviews.md`.
- **Strategic Review**: `docs/mtg-engine-strategic-review.md` (2026-03-07) — decouple M11 from M10, split M10, downscope M12, web-vs-Tauri decision pending
- **Last Updated**: 2026-07-07 (PB-AC3 collected — 2940 tests, clean coverage 54.4%)

### What Exists (M0-M9.5 + Engine Core Complete + all P3/P4 abilities)

- `cards/`: CardDefinition framework (30+ Effect primitives), ~1693 card defs across hand-authored + templated waves; CardRegistry
- `effects/`: Full effect execution engine (DealDamage, GainLife, DrawCards, ExileObject, CreateToken, SearchLibrary, ForEach, Conditional, Scry, Surveil, DrainLife, Goad, Fight, etc.)
- `rules/`: Turn structure, priority, stack, SBAs, dependency-based layer system, combat, casting (Convoke/Improvise/Delve/Evoke/Kicker/Morph/Disturb alt costs), resolution, ETB trigger queueing (CR 603.3/603.4), ETB & global replacements, prevention, Commander (deck validation, command zone, tax, zone-return SBA, mulligan, companion, partner variants), protection (DEBT), copy (Layer 1 + storm + cascade), loop detection (CR 104.4b), Enchant, suspend, Mutate (CR 702.140), Transform/DFC (CR 701.28/712), Daybound/Nightbound, Craft, Morph/Megamorph/Disguise/Manifest/Cloak; Type Consolidation refactor complete (CastSpell 32→13, SOK ~20, AbilDef 55)
- `testing/`: Replay harness (`crates/engine/src/testing/replay_harness.rs` — public, shared with replay viewer), ~112 approved scripts, ~1934 harness tests, 6-player suite, 54 property invariants
- `benches/`: criterion (priority_cycle_4p 23µs, sba_check 14µs, full_turn_4p 205µs)
- `tools/replay-viewer/`: axum + Svelte 5, 5 API endpoints, 12 components, diff highlighting, keyboard nav
- 36 corner cases: 32 COVERED, 4 GAP, 0 DEFERRED

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
| Corner Case Reference | `docs/mtg-engine-corner-cases.md` | 36 known difficult interactions the engine must handle correctly |
| Corner Case Audit | `docs/mtg-engine-corner-case-audit.md` | Living correctness ledger: coverage status, card def gaps, deferred items |
| Network Security Strategy | `docs/mtg-engine-network-security.md` | **Deferred P2P upgrade path** — not the active M10 plan. M10 uses a centralized server. |
| Milestone Code Reviews | `docs/mtg-engine-milestone-reviews.md` | Per-milestone code review findings, file inventories, issue tracking |
| Replay Viewer Design | `docs/mtg-engine-replay-viewer.md` | M9.5 game state stepper: architecture, API, Svelte components, shared-component strategy |
| Ability Coverage Audit | `docs/mtg-engine-ability-coverage.md` | Keyword and pattern coverage tracking |
| LOW Issues Remediation | `docs/mtg-engine-low-issues-remediation.md` | Tiered plan for ~68 open LOW issues with risk assessment |
| Workstream Coordination | `docs/workstream-coordination.md` | Cross-session coordination for 4 parallel workstreams (abilities, TUI, LOWs, M10) |
| Ability Batch Plan | `docs/ability-batch-plan.md` | 16 batches covering all ~75 implementable abilities (P3+P4) with dependency map |
| Card Pipeline & Scaling | `docs/mtg-engine-card-pipeline.md` | Card definition organization, Rust DSL rationale, scaling strategy (112 → 27k), authoring pipeline |
| Strategic Review | `docs/mtg-engine-strategic-review.md` | 2026-03-07 project review: path-to-playable compression, M10/M11/M12 restructuring, action items |
| Card Authoring Operations | `docs/card-authoring-operations.md` | Ordered task list for triage → fix → author → audit (68 tasks) |
| Runtime Integrity | `docs/mtg-engine-runtime-integrity.md` | Watchdog, recovery, bug reporting — pre-alpha requirement |
| Type Consolidation Plan | `docs/mtg-engine-type-consolidation.md` | Pre-M10 refactoring: CastSpell, SOK triggers, AbilityDef, Designations — 8 sessions |
| Cleanup Retention Policy | `docs/cleanup-retention-policy.md` | Two-tier ladder, year-month archive convention, /cleanup skill protocol |
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
| Checking correctness gaps | `docs/mtg-engine-corner-case-audit.md` |
| Starting a new milestone | Use `/start-milestone <N>` — reads only the relevant roadmap section via Grep+offset, never the full file. |
| Writing golden tests | `docs/mtg-engine-game-scripts.md` |
| Implementing network features (M10+) | `docs/mtg-engine-roadmap.md` M10 section (centralized server); `docs/mtg-engine-network-security.md` only for deferred P2P upgrade |
| Implementing replay viewer (M9.5) | `docs/mtg-engine-replay-viewer.md` |
| Implementing a keyword ability | `docs/mtg-engine-ability-coverage.md` |
| Checking ability gaps | Use `/audit-abilities` or `/ability-status` |
| Implementing a single ability end-to-end | Use `/implement-ability` — orchestrates plan → implement → review → fix → card → script → close |
| End-of-milestone cleanup pass | Use `/cleanup` — reads `docs/cleanup-retention-policy.md` and runs Gate A → B → dry-run → execute |
| Fixing LOW issues | `docs/mtg-engine-low-issues-remediation.md` |
| Authoring card definitions | `docs/card-authoring-operations.md` (operations plan with ordered tasks); `docs/mtg-engine-card-pipeline.md` (DSL reference) |
| Triaging card defs for TODOs | Use `/triage-cards` — scans defs, reclassifies blocked sessions, consolidates review findings |
| Authoring a group of cards | Use `/author-wave <group>` — orchestrates author → review → fix → commit for one group |
| Auditing all card defs | Use `/audit-cards` — scans for TODOs, empty abilities, known-issue patterns, certifies completion |
| Type consolidation refactoring | `docs/mtg-engine-type-consolidation.md` (must read — this is the active plan) |
| Planning M10, M11, or M12 | `docs/mtg-engine-strategic-review.md` (must read before starting) |
| Deciding what to work on / coordinating workstreams | `docs/workstream-coordination.md` |

Use `/review-subsystem <name>` to load the right file and see open issues in one step.

---

## Card Authoring Wave Process

The remaining A-29+ groups are ordered into three waves by engine risk level.
**Follow this order** — see `docs/card-authoring-operations.md` "Authoring Order and
Engine Risk Assessment" for the full breakdown.

1. **Wave A** (A-29, A-32, A-33, A-34, A-35, A-39): Safe to author now. Minor/no engine changes.
2. **Wave B** (A-38, A-42): Re-triage each group first — split authorable cards from blocked ones.
3. **Wave C** (A-30, A-36, A-40, A-41): Blocked on significant engine work. Treat as PB-style batch.

**Engine review checkpoints**: After each wave completes, batch-review all engine
changes before starting the next wave. Run `git diff <pre-wave-commit>..HEAD -- crates/engine/src/`
and review the accumulated engine additions. Fix any issues found. This is a single
review pass per wave, not per-session — but it is **mandatory** before advancing to
the next wave. The PB pipeline had plan → implement → review → fix; the authoring
pipeline adds engine code inline without review, so these checkpoints catch that gap.

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

7. **Hidden information is enforced.** The engine knows everything. The centralized server
   filters events before broadcasting — private events go only to the relevant player via
   `GameEvent::private_to() -> Option<PlayerId>`. Never expose another player's hand or
   library order to the wrong client. (P2P + Mental Poker is a deferred upgrade path —
   see `docs/mtg-engine-network-security.md`.)

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
- **rust-analyzer**: semantic code navigation — hover, definition, references, implementations, incoming/outgoing calls, workspace symbols. Call `rust_analyzer_stop` when done to free ~2.5GB RAM. First call triggers ~70s indexing warmup. Results default to 50 max; pass `limit` to override. See your auto-memory MEMORY.md index (rust-analyzer MCP Server section) for details.

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

## Agents

Fifteen project-scoped agents in `.claude/agents/` encode milestone, ability, primitive, and card authoring workflows:

| Agent | Model | RA | Trigger | Purpose |
|-------|-------|----|---------|---------|
| `rules-implementation-planner` | Opus | yes | "plan M9 implementation" | Session plan with architecture, CR refs, session breakdown |
| `session-runner` | Sonnet | — | "run session 1" / "next session" | Execute one implementation session from the plan |
| `milestone-reviewer` | Opus | yes | "review milestone M9" | Structured code review with HIGH/MEDIUM/LOW findings; creates fix-session-plan |
| `fix-session-runner` | Sonnet | — | "run fix session 3" | Execute 5-8 fixes, run tests, close issues |
| `card-definition-author` | Sonnet | — | "add card definition for X" | Translate oracle text to CardDefinition DSL |
| `bulk-card-author` | Sonnet | — | "author session 5" | Write batch of 8-20 card defs from authoring plan |
| `card-batch-reviewer` | Opus | — | "review cards batch 5" | Review 5 card defs against oracle text |
| `card-fix-applicator` | Sonnet | — | "apply fixes from review" | Apply review findings to card def files, verify build |
| `cr-coverage-auditor` | Sonnet | — | "check CR coverage for 614" | Audit test/script coverage for CR sections |
| `game-script-generator` | Sonnet | — | "generate script for X interaction" | JSON game scripts for replay harness |
| `ability-coverage-auditor` | Opus | — | `/audit-abilities` | Scan engine + card defs + scripts → refresh ability coverage doc |
| `ability-impl-planner` | Opus | yes | `/implement-ability` (plan phase) | CR research, study similar abilities, write implementation plan |
| `ability-impl-runner` | Sonnet | — | `/implement-ability` (implement/fix phase) | Execute steps 1-4 (enum, enforcement, triggers, tests), apply fixes |
| `ability-impl-reviewer` | Opus | yes | `/implement-ability` (review phase) | Verify implementation against CR, check edge cases, write findings |
| `primitive-impl-planner` | Opus | yes | `/implement-primitive` (plan phase) | CR research, study engine architecture, write PB plan |
| `primitive-impl-runner` | Sonnet | — | `/implement-primitive` (implement/fix phase) | Engine changes, card def fixes, tests, apply review fixes |
| `primitive-impl-reviewer` | Opus | yes | `/implement-primitive` (review phase) | Verify engine + card defs against CR/oracle text, write findings |

---

## Session & Workstream Protocol

- `/start` — bootstrap ESM, check local state, orient (also covers what `/start-session` used to do — workstream state is loaded via `esm project bootstrap` and the auto-memory MEMORY.md index)
- `/start-work W1-B3` — claim a workstream before coding (prevents parallel collisions)
- `/eot` — end-of-turn / end-of-session: ESM session close + workstream-state rotation + memory routing (replaces `/end` + `/end-session`)
- State file: `memory/workstream-state.md` (shared across sessions)
- Conventions: `memory/conventions.md` | Decisions: `memory/decisions.md`
- Dev environment: `.claude/CLAUDE.local.md`

### Commit Prefix Convention

| Workstream | Prefix | Example |
|------------|--------|---------|
| W1: Abilities | `W1-B<N>:` | `W1-B3: implement Ninjutsu` |
| W2: TUI & Simulator | `W2:` | `W2: fix blocker declaration` |
| W3: LOW Remediation | `W3:` | `W3: add debug_assert to sba.rs` |
| W4: M10 Networking | `W4:` | `W4: add GameServer skeleton` |
| W5: Card Authoring | `W5-cards:` | `W5-cards: author Skullclamp, Blood Artist` |
| Cross-cutting | `chore:` | `chore: update workstream-state` |

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
  - Run the `milestone-reviewer` agent (Opus) — writes findings to `docs/mtg-engine-milestone-reviews.md`
    and creates `memory/m<N>-fix-session-plan.md` grouping issues into sessions of 5-8 fixes each
  - Work through fix sessions with the `fix-session-runner` agent (Sonnet):
    reads `memory/m<N>-fix-session-plan.md` → applies fixes → `cargo test --all` → `cargo clippy -- -D warnings` → closes issues in reviews doc → commit
  - When all sessions complete, update "Current State" and advance to the next milestone
  - LOW-only findings do not require a fix phase; collect them in the reviews doc and address opportunistically

---

# Scutemob MTG Engine — ESM-Managed Project

This project is managed by ESM (External State Machine). Use the `esm` CLI and slash commands to interact with it.

## Quick Start

Use these slash commands to manage your ESM session:

- **`/start`** — Begin a session. Bootstraps context from ESM, starts session tracking, orients you.
- **`/dispatch <title>`** — **Primary workflow.** Create a task, worktree, and auto-launch a worker in a kitty pane. Use this for all implementation work.
- **`/status`** — Quick snapshot of tasks, sessions, and fleet-wide context.
- **`/collect [task_id]`** — Collect a finished worker's work: merge worktree to main, clean up.
- **`/task <title>`** — Create a task and work on it yourself (for small, self-assigned work only).
- **`/done [task_id]`** — Complete a self-assigned task: transition to done, merge branch to main.
- **`/spawn <title>`** — Like /dispatch, but you launch the worker manually.
- **`/eot`** — End-of-turn / end-of-session: ESM close + workstream-state rotation + memory routing. **Use this instead of `/end`** for scutemob — `/end` still works but skips the project-specific bookkeeping.

**Every session must begin with `/start`** (or manually running `esm project bootstrap scutemob` + `esm session start`).

## Worker Detection

If `.esm/worker.md` exists in the working directory, **you are a worker agent**. Read it
immediately and follow its task/acceptance criteria. The rest of this CLAUDE.md still applies.

## Workflow Rules

1. **Bootstrap first**: `/start` (or `esm project bootstrap scutemob && esm session start --project scutemob --agent primary`).
2. **An `in_progress` task must exist before writing code.** Lifecycle: `backlog → in_progress → in_review → done` (or `blocked` from either active state).
3. **Branch protocol**: feature branch per task; attest `working_branch=<full-name>` on transition; `/done` (self-assigned) or `/collect` (dispatched) merges to main.
4. **Tests are mandatory.** Write alongside implementation. Must pass before `in_review`.
5. **Acceptance criteria**: `esm task satisfy <task_id> <criterion_id> --by <agent>` for each before signaling ready.
6. **Task comments are short status lines** — `Completed: X. Next: Y.` / `Blocked: X. Tried: Y.` / `Decision: X. Reason: Y.` Detailed design notes belong in `docs/` or `memory/`, not comments.
7. **Dispatch, don't implement.** Coordinator creates tasks and dispatches workers via `/dispatch` for PB / ability / card-authoring work. Only implement inline for trivial fixes (<10 lines) or when explicitly told.

ESM CLI reference: `esm --help` or `esm <command> --help`. Sessions without a heartbeat for 10 minutes are auto-ended.

## Required Attestations

When transitioning to `in_progress`:
- `branch_exists`: "true"
- `acceptance_criteria_defined`: "true"
- `working_branch`: "<branch-name>"

When transitioning to `in_review`:
- `tests_passing`: "true"
- `implementation_complete`: "true"

When transitioning to `done`:
- `review_complete`: "true"

When transitioning to `blocked`:
- `blocked_reason`: describe what you need before you can continue

Unblocking requires admin approval — you cannot unblock yourself.

## Advisory Mode

ESM runs in **advisory mode** by default. The hook will warn you about scope violations and missing tasks, but won't block your work. Warnings appear in stderr — pay attention to them.

If this project uses **blocking mode**, scope violations will be denied. Check the project's `enforcement_mode` setting.

## Documentation Management

If `.claude/docs.yaml` exists, this project uses ESM documentation management.
Managed docs have a `<!-- last_updated: YYYY-MM-DD -->` comment that tracks freshness.

- **`/docs status`** — Quick health overview of all managed docs
- **`/docs check`** — Audit docs for drift (checks triggers against git history)
- **`/docs init`** — Interactive setup: scan existing docs, detect features, scaffold new ones

When you update a managed doc, always update the `<!-- last_updated: YYYY-MM-DD -->`
comment to today's date. Only update it for substantive changes — not typo fixes.

The `/done` and `/eot` skills automatically check for stale docs based on which
files you changed. Follow their recommendations or dismiss with a reason.

## Project Info

- **ESM Project ID**: `scutemob`
- **Agent ID**: `primary`
- **ESM Server**: `http://tower:8765`
