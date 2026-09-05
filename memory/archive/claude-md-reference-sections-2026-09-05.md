# CLAUDE.md reference sections archive — 2026-09-05 (CC-1, `scutemob-237`)

> The "Project Overview" through "Milestone Completion Checklist" sections of `CLAUDE.md` as they
> stood at `9c93019d`, kept VERBATIM because the context diet condensed them (full Primary and
> Secondary document tables, the full When-to-Load table, the Card Authoring Wave Process, the
> 17-agent table, the commit-prefix table, the Milestone Completion Checklist). The nine
> Architecture Invariants and the SR gate pointers survive in `CLAUDE.md` itself.

<!-- BEGIN VERBATIM: CLAUDE.md lines 4906-5188 at 9c93019d -->
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
| Engine Invariants & Gates | `docs/engine-invariants.md` | Full text of the machine-enforced SR gates (SR-2/3/4/5/6/7/8/9a/9b/9c/35/36); read the matching section before touching the subsystem it guards |
| Development Roadmap | `docs/mtg-engine-roadmap.md` | What to build and in what order; milestone definitions |
| Game Script Strategy | `docs/mtg-engine-game-scripts.md` | Engine-independent test script generation, JSON schema, replay harness design |
| Corner Case Reference | `docs/mtg-engine-corner-cases.md` | 36 known difficult interactions the engine must handle correctly |
| Corner Case Audit | `docs/mtg-engine-corner-case-audit.md` | Living correctness ledger: coverage status, card def gaps, deferred items |
| Network Security Strategy | `docs/mtg-engine-network-security.md` | **Deferred P2P upgrade path** — not the active M10 plan. M10 uses a centralized server. |
| Milestone Code Reviews | `docs/mtg-engine-milestone-reviews.md` | Per-milestone code review findings, file inventories, issue tracking |
| Replay Viewer Design | `docs/mtg-engine-replay-viewer.md` | M9.5 game state stepper: architecture, API, Svelte components, shared-component strategy |
| Ability Coverage Audit | `docs/mtg-engine-ability-coverage.md` | Keyword and pattern coverage tracking |
| LOW Issues Remediation | `docs/mtg-engine-low-issues-remediation.md` | **HISTORICAL (2026-02-28 snapshot; "~68 open LOW" is stale, ~6 remain).** Live LOW tally: "Current State → Known issues" above + `docs/mtg-engine-milestone-reviews.md` |
| Workstream Coordination | `docs/workstream-coordination.md` | **HISTORICAL — retired W1–W6 model (frozen 2026-03-08).** For what to work on: "Current State" above + `memory/primitives/oos-retriage-plan-2026-07-18.md` |
| Ability Batch Plan | `docs/ability-batch-plan.md` | **HISTORICAL — campaign COMPLETE.** Live tally: "Current State → Abilities" above; detail `docs/mtg-engine-ability-coverage.md` |
| Card Pipeline & Scaling | `docs/mtg-engine-card-pipeline.md` | Card definition organization, Rust DSL rationale, scaling strategy (112 → 27k), authoring pipeline |
| Strategic Review | `docs/mtg-engine-strategic-review.md` | 2026-03-07 project review: path-to-playable compression, M10/M11/M12 restructuring, action items. **All 9 resolved 2026-07-26** — historical record now; the structure it argued for lives in the roadmap |
| M11-local Session Plan | `memory/m11-session-plan.md` | The active first-playable plan: 8 sessions, crate-by-crate scope, the steppable-driver decision, hidden-info chokepoints, risks |
| Card Authoring Operations | `docs/card-authoring-operations.md` | **HISTORICAL — 2026-03-21 runbook, superseded.** Active campaign: `memory/card-authoring/campaign-plan-2026-05-16.md`; live coverage `docs/authoring-status.md`. (Its "Authoring Order" section is still cited by the Wave Process below.) |
| Runtime Integrity | `docs/mtg-engine-runtime-integrity.md` | Watchdog, recovery, bug reporting — pre-alpha requirement |
| Feedback Engineering | `docs/mtg-engine-feedback-engineering.md` | Alpha feedback-loop strategy: channel inventory, 8 ranked buildout proposals, alpha-pipeline ownership (2026-08-03, dispatch-ready) |
| Type Consolidation Plan | `docs/mtg-engine-type-consolidation.md` | Pre-M10 refactoring: CastSpell, SOK triggers, AbilityDef, Designations — 8 sessions |
| Cleanup Retention Policy | `docs/cleanup-retention-policy.md` | Two-tier ladder, year-month archive convention, /cleanup skill protocol |
| **Course Correction (2026-09)** | `docs/course-correction-2026-09.md` | **APPROVED 2026-09-05, tasks filed (`scutemob-237..254`, §10)** — audit findings, the context diet, pod-first roadmap (P0–P3), agents/skills tuning, addendum reconciled in §9. **The second v4 chain is CLOSED at rank 21: do NOT dispatch PB-DX9 or PB-DX38.** Next work: the coordinator batch CC-1/2/3/4/14/15/17 |
| This file | `CLAUDE.md` | Current project state; session context |

**Read the architecture doc before implementing anything.**

### Secondary Documents & Task Records

Not primary context, but every one is reachable from here. Load on demand for the stated purpose.

| Document | Location | Purpose |
|----------|----------|---------|
| Authoring status (generated) | `docs/authoring-status.md` + `docs/authoring-status-guide.md` | **Canonical card-health source** — regenerated by `tools/authoring-report.py`, self-dating; prefer over any hand-maintained count |
| Engine explanation | `docs/engine_explanation.md` | Narrative walkthrough of the engine for a newcomer |
| Protocol versioning policy | `docs/mtg-engine-protocol-versioning.md` | Wire versioning policy behind SR-8 (also linked from `docs/engine-invariants.md`) |
| Simulator & bots | `docs/mtg-engine-simulator.md` | RandomBot / HeuristicBot / GameDriver / LegalActionProvider design |
| TUI plan | `docs/mtg-engine-tui-plan.md` | Terminal UI dashboard plan |
| Interaction gaps | `docs/mtg-engine-interaction-gaps.md` | Catalogue of known unresolved rules-interaction gaps |
| Project status (RETIRED) | `docs/project-status.md` | **🚫 RETIRED 2026-07-18, do not use or regenerate.** Successors: `docs/authoring-status.md` (card health) + "Current State" above (everything else) |
| Primitive/card plan (HISTORICAL) | `docs/primitive-card-plan.md` | March primitive/card plan — **banner'd historical**; active queue `memory/primitives/oos-retriage-plan-2026-07-18.md`, coverage `docs/authoring-status.md` |
| DSL gap closure (HISTORICAL) | `docs/dsl-gap-closure-plan.md` | March DRAFT — **banner'd superseded** by the EF/OS queues; audit `memory/card-authoring/dsl-gap-audit-2026-05-16.md` |
| SR remediation record | `docs/sr-remediation-plan.md` | Full SR-1..32 remediation log |
| SR task-record audits | `docs/sr-4-silent-failure-audit.md`, `docs/sr-5-keyword-catchall-audit.md`, `docs/sr-9a-test-consolidation.md`, `docs/sr-14-silent-failure-audit-rules.md`, `docs/sr-15-dispatch-enum-catchall-audit.md`, `docs/sr-24-lki-capture-cost.md` | Per-SR method/scope records referenced by the matching gate in `docs/engine-invariants.md` |
| Audit program | `docs/audits/README.md` + `docs/audits/methodology.md` | Index and method for the standing audit program |
| Standing audits | `docs/audits/layer-bypass-audit.md`, `docs/audits/event-log-diagnosability.md`, `docs/audits/stress-test-scenarios.md`, `docs/audits/decision-point-audit.md` | Specific audits (note: layer-bypass "9 HIGH" are its own M10-scheduled class, distinct from the 0-HIGH engine tally; **decision-point audit (2026-07-26, `scutemob-148`) found 5 Tier-0 correctness findings DP-1..DP-5 — incl. priority-after-cast CR 117.3c violation — and a ranked PB-DP1..DP10 insertion list, unranked vs the RS queue as of collection**) |
| Interaction deconstructions | `docs/interactions/` (`blood-moon-urzas-saga.html`) | Shareable, self-contained HTML explainers of engine-resolved interactions; two-layer (table + engine room) |
| Changelog archive | `memory/archive/claude-md-changelog-2026-07.md` | Verbatim PB/SR history moved out of this file's Current State (see "Changelog & history" above) |

### Additional Skills (beyond the ESM/session ones listed below)

- `/crew` — multi-agent orchestration helper.
- `/new-doc` — scaffold a new managed doc.
- `/next-ability` — pick and set up the next ability to implement.
- `/remedy` — SR remediation track driver (agent `sr-coordinator`; does not touch workstream-state).
- `/start-stepper` — launch the replay-viewer game-state stepper.

(Session/workflow skills — `/start`, `/dispatch`, `/collect`, `/eot`, `/task`, `/done`, `/spawn`,
`/status` — are in "Quick Start" below; per-task skills like `/implement-primitive`,
`/author-wave`, `/cleanup`, `/audit-cards` appear in the "When to Load What" table.)

---

## When to Load What

Before starting work, check which files apply to your task:

| Task | Load before starting |
|------|----------------------|
| Understanding / modifying a machine-enforced gate (any SR-N invariant) | `docs/engine-invariants.md` (the SR-2/3/4/5/6/7/8/9a/9b/9c/35/36 gate reference) |
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
| Fixing LOW issues | `docs/mtg-engine-milestone-reviews.md` (live issue index; ~6 LOW remain). `docs/mtg-engine-low-issues-remediation.md` is a HISTORICAL 2026-02-28 snapshot — risk-tier framework still useful, counts stale |
| Authoring card definitions | `memory/card-authoring/campaign-plan-2026-05-16.md` (active campaign, §0 authoritative); `docs/mtg-engine-card-pipeline.md` (DSL reference). `docs/card-authoring-operations.md` is HISTORICAL — its "Authoring Order" section still valid, see Wave Process below |
| Triaging card defs for TODOs | Use `/triage-cards` — scans defs, reclassifies blocked sessions, consolidates review findings |
| Authoring a group of cards | Use `/author-wave <group>` — orchestrates author → review → fix → commit for one group |
| Auditing all card defs | Use `/audit-cards` — scans for TODOs, empty abilities, known-issue patterns, certifies completion |
| Type consolidation refactoring | `docs/mtg-engine-type-consolidation.md` (COMPLETE 2026-03-09 — historical record of the refactor, not an active plan) |
| Working on the play client / local play (M11-local is **COMPLETE** — this is maintenance, not milestone work) | `tools/play-server/README.md` (routes, limitations, hidden-info rules) + `docs/mtg-engine-simulator.md` §"Phase 3b" + `memory/workstream-state.md`'s S8 handoff. `memory/m11-session-plan.md` is now a historical record with its own COMPLETE banner |
| Planning M10a/M10b or the card-scaling track | `docs/mtg-engine-roadmap.md` (restructured 2026-07-26 — read the milestone section itself). `docs/mtg-engine-strategic-review.md` is now a historical record of *why* that structure exists, not a pending-changes list |
| Deciding what to work on / coordinating workstreams | "Current State" above (active milestone + queue) + `memory/primitives/oos-retriage-plan-2026-07-18.md` (ranked queue). `docs/workstream-coordination.md` is HISTORICAL (retired W1–W6 model) — do not use to pick work |

Use `/review-subsystem <name>` to load the right file and see open issues in one step.

---

## Card Authoring Wave Process

The remaining A-29+ groups are ordered into three waves by engine risk level.
**Follow this order** — see the "Authoring Order and Engine Risk Assessment" section of
`docs/card-authoring-operations.md` for the full breakdown. (That doc is banner'd HISTORICAL,
but this specific ordering section remains the valid reference for the wave sequence.)

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
- **rust-analyzer**: semantic code navigation — hover, definition, references, implementations,
  incoming/outgoing calls, workspace symbols. Call `rust_analyzer_stop` when done to free ~2.5GB
  RAM. First call triggers ~70s indexing warmup. Results default to 50 max; pass `limit` to
  override. See your auto-memory MEMORY.md index (rust-analyzer MCP Server section) for details.

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

Seventeen project-scoped agents in `.claude/agents/` encode milestone, ability, primitive, and card authoring workflows:

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

- `/start` — bootstrap ESM, check local state, orient (also covers what `/start-session` used to do
  — workstream state is loaded via `esm project bootstrap` and the auto-memory MEMORY.md index)
- `/start-work W1-B3` — claim a workstream before coding (prevents parallel collisions)
- `/eot` — end-of-turn / end-of-session: ESM session close + workstream-state rotation + memory
  routing (replaces `/end` + `/end-session`)
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
| W6: Card Authoring | `W6-cards:` | `W6-cards: author Skullclamp, Blood Artist` |
| W6: Primitives | `W6-prim:` | `W6-prim: add exclude_self enforcement` |
| SR remediation | `SR-<N>:` | `SR-9a: consolidate test binaries` |
| Cross-cutting | `chore:` | `chore: update workstream-state` |

---

## Milestone Completion Checklist

When completing a milestone:

- [ ] All deliverables checked off in the roadmap
- [ ] All acceptance criteria met
- [ ] All tests pass: `cargo test --all`
- [ ] No clippy warnings: `cargo clippy -- -D warnings`
- [ ] Formatted: `cargo fmt --check` **and** `tools/check-defs-fmt.sh` (SR-35 — `cargo fmt`
      checks none of the 1,798 card defs and still exits 0; the script is the only thing
      that checks them. `cargo test --all` runs it too, via `core card_defs_fmt`.)
- [ ] Performance benchmarks run (if applicable to this milestone)
- [ ] Update "Current State" section of this file
- [ ] Update "Active Milestone" to the next milestone
- [ ] Check off completed deliverables in `docs/mtg-engine-roadmap.md`
- [ ] Update relevant memory topic files (`memory/gotchas-rules.md`, `memory/gotchas-infra.md`,
  `memory/conventions.md`, `memory/decisions.md`) with new learnings
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
  - LOW-only findings do not require a fix phase; collect them in the reviews doc and address
    opportunistically

<!-- END VERBATIM -->
