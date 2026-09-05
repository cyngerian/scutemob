# CLAUDE.md — MTG Commander Rules Engine

> Primary context for Claude Code sessions. Kept under 250 lines (`docs/course-correction-2026-09.md`
> §3.1). Per-batch narrative goes to `CHANGELOG.md` + `memory/primitives/pb-<id>-execution-notes.md`,
> **never** here. The pre-diet "Current State" (4,892 lines) is verbatim in
> `memory/archive/claude-md-current-state-2026-09-05.md`; condensed reference sections are verbatim in
> `memory/archive/claude-md-reference-sections-2026-09-05.md`.

## Current State

- **Active Milestone**: P1 pod-first (`docs/course-correction-2026-09.md` §5, approved 2026-09-05).
  M11-local DONE 2026-08-01 (engine first-playable: one human + three bots in a browser via
  `tools/play-server`). M10+ roadmap milestones are HISTORICAL pending CC-8.
- **Status**: 5,330 tests passing / 0 failing / 6 ignored (after CC-2, 72 targets);
  208 approved golden scripts; CI green since 2026-07-10.
- **Last Updated**: 2026-09-05 — CC-4 change-class table (`scutemob-240`).
- **Headline metric**: live card coverage **1,140 / 1,803 = 63.2%** (`docs/authoring-status.md`,
  regenerate with `tools/authoring-report.py`). Replaced by pod coverage (`docs/pod-coverage.md`)
  once CC-6 lands. Wire: PROTOCOL **44** / HASH **85**.
- **Next dispatch**: NONE from the v4 queue — the second chain is **CLOSED at rank 21** (do NOT
  dispatch PB-DX9 / PB-DX38). Coordinator batch: CC-1/2/3/4 DONE 2026-09-05; CC-17 → CC-15 →
  CC-14 remain (`scutemob-254`/`252`/`251`, doc §10); CC-5 (six pod decklists) needs the owner. Every dispatch needs
  explicit owner approval (`feedback_queue_autonomous_chaining` RETRACTED 2026-08-01).
- **Where the detail is**: `CHANGELOG.md` (one ≤10-line entry per batch, newest first);
  `memory/workstream-state.md` (claims + last handoff); `docs/audits/decision-point-audit.md`
  (the OOS seed registry — ground truth; grep it before filing); v4 queue memo
  `memory/primitives/seed-rerank-2026-08-14.md` §4 (banner'd CLOSED at rank 21).

### Machine-enforced invariants (full text: `docs/engine-invariants.md`)

Read the matching section before touching the subsystem a gate guards.

| Gate | One line |
|------|----------|
| SR-2 | `CardDefinition.completeness` markers; `validate_deck` rejects non-`Complete` cards (Invariant 9) |
| SR-3 | `GameState` sealed `pub(crate)`; only `Command` → `process_command` mutates; `cargo build --workspace` is the seal gate |
| SR-4 | Silent failures in `effects/mod.rs` + `rules/resolution.rs` classified LKI-fizzle vs engine-bug (`expect_*` vs `lki_*`) |
| SR-5 | Every `KeywordAbility` variant classified in `state::keyword_registry::handling` (exhaustive) |
| SR-6 | Card defs (`mtg-card-defs`) depend on `card-types` only, never the engine |
| SR-7 | `PendingTrigger` built through `PendingTrigger::blank`; per-kind payload in `TriggerData` |
| SR-8 | `PROTOCOL_SCHEMA_FINGERPRINT` / `HASH_SCHEMA_VERSION` gates (`protocol_schema`, `hash_schema`): adding an `Effect` variant is a wire change; predict the bump before code |
| SR-9a/b/c | 9 integration-test targets (never a top-level `tests/*.rs`); JSON-script vs direct-`Command` per-step fingerprint; golden corpus partition gate |
| SR-35 | Card defs are format-checked by `tools/check-defs-fmt.sh`, not `cargo fmt` |
| SR-36 | Activation costs are paid only where code pays them; enumerate `all_cards()` for rosters, never grep source |
| SR-37 | Printed fields diffed against a committed Scryfall fixture (`core::cards2_printed_field_fidelity`) |

## Project Overview

An MTG rules engine for **Commander** (4-player) in **Rust**: a pure library crate (`crates/engine`)
wrapped by a simulator, view models and a browser play client. Networking (P3) is deferred until
co-location is the bottleneck; match one is hot-seat.

### Key paths

- Engine: `crates/engine/src/{state,rules,effects}/`; DSL `crates/card-types/src/cards/`; card defs
  `crates/card-defs/src/defs/` (1,803); view models `crates/view-model/`; bots + `LocalGame`
  `crates/simulator/`; tests `crates/engine/tests/<group>/`; golden scripts `test-data/generated-scripts/`
- Clients/tools: `tools/play-server` (axum + Svelte, port 3040), `tools/tui`, `tools/replay-viewer`,
  `tools/authoring-report.py`, `tools/check-defs-fmt.sh`, `tools/mcp-server` (rules MCP)
- Memory: `memory/workstream-state.md`, `memory/conventions.md`, `memory/decisions.md`,
  `memory/gotchas-rules.md`, `memory/gotchas-infra.md`, `memory/primitives/` (plans + notes)

### Key documents

| Document | Purpose |
|----------|---------|
| `docs/course-correction-2026-09.md` | **The live plan** (approved 2026-09-05): context diet, pod-first P0–P3, agents/skills tuning, tasks CC-1..17 |
| `docs/end-state.md` | What "done" means: playable pod matches |
| `docs/mtg-engine-architecture.md` | System design and testing strategy — **read before implementing anything** |
| `docs/engine-invariants.md` | Full text of the SR gates above |
| `docs/mtg-engine-roadmap.md` | Milestone definitions (M10–M15 to be banner'd HISTORICAL by CC-8) |
| `docs/mtg-engine-corner-cases.md` / `-corner-case-audit.md` | 36 hard interactions and their living coverage ledger (35 COVERED / 1 PARTIAL) |
| `docs/audits/decision-point-audit.md` | OOS seed registry (ground truth) + the standing audit program (`docs/audits/README.md`) |
| `docs/mtg-engine-milestone-reviews.md` | Review findings and the live issue index (0 HIGH, 2 MEDIUM, 6 LOW open) |
| `docs/mtg-engine-card-pipeline.md` | Card DSL reference and authoring pipeline; campaign plan `memory/card-authoring/campaign-plan-2026-05-16.md` |
| `docs/mtg-engine-game-scripts.md` | Golden-script schema and replay harness |
| `docs/mtg-engine-simulator.md`, `tools/play-server/README.md` | Bots, `LocalGame`, and the play client (routes, hidden-info rules) |
| `docs/mtg-engine-feedback-engineering.md` | Alpha feedback-loop strategy |
| `docs/cleanup-retention-policy.md` | Archive ladder and the `/cleanup` protocol |
| `CHANGELOG.md` | One entry per shipped batch, newest first, pointing at its notes file |

HISTORICAL/RETIRED docs (`project-status`, `workstream-coordination`, `primitive-card-plan`,
`dsl-gap-closure-plan`, `low-issues-remediation`, `card-authoring-operations`, `ability-batch-plan`,
`strategic-review`, `type-consolidation`, `m11-session-plan`) self-banner; do not plan from them.

## When to Load What

| Task | Load first |
|------|------------|
| Any SR gate | `docs/engine-invariants.md` (matching section) |
| Files in `rules/` | `memory/gotchas-rules.md` |
| Files in `state/`, `cards/`, `effects/`; writing tests | `memory/gotchas-infra.md` |
| Any new code or tests | `memory/conventions.md` (incl. the change-class acceptance table, CC-4) |
| Questioning a design decision | `memory/decisions.md` |
| A new subsystem / a correctness gap | `docs/mtg-engine-corner-cases.md` / `-audit.md` |
| Golden tests | `docs/mtg-engine-game-scripts.md` |
| Card authoring | campaign plan §0 + `docs/mtg-engine-card-pipeline.md`; `/author-wave`, `/triage-cards`, `/audit-cards` |
| A primitive batch | `/implement-primitive` (resolve the queue from the "Next dispatch" line above) |
| Play client / hot-seat work | `tools/play-server/README.md` + `docs/mtg-engine-simulator.md` §"Phase 3b" |
| Deciding what to work on | "Current State" above, then `docs/course-correction-2026-09.md` §9.2 |

`/review-subsystem <name>` loads the right file and surfaces open issues in one step.

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

## MCP Resources

- **Rules / card / rulings search** (`mtg-rules`): by rule number, concept, or exact card name. CR text
  is authoritative; rulings can be stale.
- **rust-analyzer**: semantic navigation; ~70s warmup; call `rust_analyzer_stop` when done (~2.5GB).

## Critical Gotchas

All others: `memory/gotchas-rules.md`, `memory/gotchas-infra.md`.

- **Object identity (CR 400.7)**: a zone change makes a NEW object; the old `ObjectId` is dead.
- **Replacement effects are NOT triggers**: they modify events as they happen, off the stack.
- **SBAs are checked as a batch**: all applicable SBAs happen simultaneously, then their triggers
  go on the stack together (APNAP).

## Agents

Active project agents in `.claude/agents/` (the eight dormant milestone/ability agents move to
`.claude/agents-dormant/` in CC-14; restore with `git mv` when a milestone or ability needs them):

| Agent | Model | Trigger | Purpose |
|-------|-------|---------|---------|
| `primitive-impl-planner` | Opus | `/implement-primitive` (plan) | CR research, engine study, PB plan |
| `primitive-impl-runner` | Sonnet | `/implement-primitive` (implement/fix) | Engine changes, card fixes, tests |
| `primitive-impl-reviewer` | Opus | `/implement-primitive` (review) | Verify against CR / oracle text |
| `card-definition-author` | Sonnet | "add card definition for X" | One oracle text → DSL |
| `bulk-card-author` | Sonnet | `/author-wave` | Batch of card defs |
| `card-batch-reviewer` | Opus | `/author-wave` (review) | Review defs against oracle text |
| `card-fix-applicator` | Sonnet | "apply fixes from review" | Apply review findings, verify build |
| `cr-coverage-auditor` | Sonnet | "check CR coverage for 614" | Test/script coverage per CR section |
| `game-script-generator` | Sonnet | "generate script for X" | JSON golden scripts |

## Session Protocol

- `/start` → work → `/eot`. Coordinator dispatches (`/dispatch`, `/collect`); inline only for
  trivial fixes or when told. Small self-assigned work: `/task` … `/done`.
- Commit prefixes: `scutemob-<N>:` (task work), `W6-prim:` / `W6-cards:`, `chore:`, `merge:`.
  Title line plus at most ten body lines; detail goes in the notes file the message points at.
- Acceptance ritual scales to the change class (table in `memory/conventions.md`, CC-4).
- Gates every change runs: `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D
  warnings`, `cargo fmt --check` **and** `tools/check-defs-fmt.sh` (SR-35).

# Scutemob MTG Engine — ESM-Managed Project

Managed by ESM (External State Machine). Server `http://tower:8765`; CLI `esm --help`.

## Quick Start

- **`/start`** — bootstrap context, start session tracking, orient. **Every session begins here.**
- **`/dispatch <title>`** — create task + worktree, launch a worker in a kitty pane (primary workflow).
- **`/collect [task_id]`** — merge a finished worker's worktree to main, clean up.
- **`/task <title>`** / **`/done`** — small self-assigned work on a branch.
- **`/status`** — tasks, sessions, fleet snapshot. **`/eot`** — end of session (use instead of `/end`).

## Worker Detection

If `.esm/worker.md` exists in the working directory, **you are a worker agent**. Read it
immediately and follow its task/acceptance criteria. The rest of this CLAUDE.md still applies.

## Workflow Rules

1. **Bootstrap first**: `/start` (or `esm project bootstrap scutemob && esm session start --project
   scutemob --agent primary`).
2. **An `in_progress` task must exist before writing code.** Lifecycle: `backlog → in_progress →
   in_review → done` (or `blocked` from either active state).
3. **Branch protocol**: feature branch per task; attest `working_branch=<full-name>` on transition;
   `/done` (self-assigned) or `/collect` (dispatched) merges to main.
4. **Tests are mandatory.** Write alongside implementation. Must pass before `in_review`.
5. **Acceptance criteria**: `esm task satisfy <task_id> <criterion_id> --by <agent>` for each before
   signaling ready.
6. **Task comments are short status lines** — `Completed: X. Next: Y.` / `Blocked: X. Tried: Y.` /
   `Decision: X. Reason: Y.` Detailed design notes belong in `docs/` or `memory/`, not comments.
7. **Dispatch, don't implement.** Coordinator creates tasks and dispatches workers via `/dispatch`
   for PB / ability / card-authoring work. Only implement inline for trivial fixes (<10 lines) or
   when explicitly told.

Sessions without a heartbeat for 10 minutes are auto-ended.

## Required Attestations

- To `in_progress`: `branch_exists=true`, `acceptance_criteria_defined=true`, `working_branch=<branch>`
- To `in_review`: `tests_passing=true`, `implementation_complete=true`
- To `done`: `review_complete=true`
- To `blocked`: `blocked_reason=<what you need>` — unblocking requires admin approval.

## Advisory Mode

ESM runs in **advisory mode**: the hook warns about scope violations and missing tasks on stderr
but does not block. Pay attention to the warnings.

## Documentation Management

`.claude/docs.yaml` lists the managed docs; each carries `<!-- last_updated: YYYY-MM-DD -->`. Update
the date on substantive edits. `/docs status` / `/docs check` audit drift; `/done` and `/eot` check
the docs your changed files trigger.

## Project Info

- **ESM Project ID**: `scutemob`
- **Agent ID**: `primary`
- **ESM Server**: `http://tower:8765`
