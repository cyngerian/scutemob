# CLAUDE.md — MTG Commander Rules Engine

> **This file is the primary context document for Claude Code sessions.** Read this before
> doing anything. It tells you where the project is, what the architecture looks like,
> and what to watch out for.
>
> **Update this file** at the completion of each milestone or when major design decisions
> change. The "Current State" section should always reflect reality.

---

## Current State

- **Active Milestone**: M9.5 DONE — **TYPE CONSOLIDATION COMPLETE** (all workstreams unpaused)
- **Status**: 2281 tests passing; ~195 validated; 42/42 P1; 17/17 P2; 40/40 P3; 95/105 P4 (95/95 implemented; 0 planned; 9 permanent-n/a; 1 deferred: Banding post-alpha); Batch 0-16 + Mutate + Transform + Morph + Dungeon + Ring complete; Type consolidation COMPLETE; **ALL PRIMITIVE BATCHES COMPLETE (PB-0 through PB-21)**; **PB-22 deferred cleanup ALL 7 SESSIONS DONE** (activation_condition, coin flip/d20, reveal-route/flicker, tapped-attacking tokens/auto-attach, copy/clone, emblem creation, adventure/dual-zone search); W6-review 21/21 COMPLETE; 0 HIGH/MEDIUM open; **~29 LOW open**; W3 LOW sprint COMPLETE
- **Active Plan**: **W6 Primitive + Card Authoring** — primitives DONE; **card authoring operations**: `docs/card-authoring-operations.md` (**Phase 2 IN PROGRESS** — **A-20 through A-23 COMPLETE (121 new cards)**; next: A-24 attack-trigger). W5 RETIRED. Goal: all 1,743 cards complete pre-alpha, zero TODOs.
- **Strategic Review**: `docs/mtg-engine-strategic-review.md` (historical snapshot 2026-03-07) — decouple M11 from M10, split M10, downscope M12, web-vs-Tauri decision pending
- **Last Updated**: 2026-03-23 (A-20 through A-23: 121 new cards; ~1391 card defs total; 2281 tests, 0 clippy)

### What Exists (M9.5 complete + 90 abilities through Batch 15 + Mutate + Transform, includes M0-M9 + Engine Core Complete checkpoint)
- `cards/`: CardDefinition framework (30+ Effect primitives), ~1391 card defs (149 hand-authored + 114 Phase 1 templates + 82 Phase 2 Wave 1 + 108 prior + 88 Phase 2 Tier 1 + 88 Phase 2 Tier 2 + 162 A-18 draw + 144 A-19 token-create + 121 A-20/A-21/A-22/A-23 + 334 prior Phase 2), CardRegistry
- `effects/`: Full effect execution engine (DealDamage, GainLife, DrawCards, ExileObject, CreateToken, SearchLibrary, ForEach, Conditional, Scry, Surveil, DrainLife, Goad, PlayExiledCard, DetachEquipment, Fight, Bite, etc.)
- `rules/`: Turn structure, priority, stack, SBAs, layer system (dependency-based), combat (declare/damage), casting (Convoke/Improvise/Delve/Evoke/Kicker alt costs), resolution (card registry fallback for CardDef triggers via PendingTriggerKind::Normal — B14 fix), ETB trigger queueing via queue_carddef_etb_triggers() (CR 603.3 — replaces inline execute, triggers properly queue and resolve from stack; CR 603.4 intervening-if re-evaluated at resolution), ETB replacements, prevention effects, global replacement registration, Commander rules (commander.rs: deck validation, command zone casting, commander tax, zone return SBA with player choice, mulligan, companion, Friends Forever/ChooseABackground/DoctorsCompanion partner variants — B15), protection.rs (DEBT), copy.rs (Layer 1 + storm + cascade), loop_detection.rs (mandatory loop = draw CR 104.4b), Enchant enforcement (cast-time + SBA), suspend.rs (upkeep trigger, free cast, haste); end_step_actions() generic CardDef sweep (B14 fix); Mutate (CR 702.140): merged_cards/MergedComponent on GameObject, CastWithMutate command, resolution merge (over/under), zone-change splitting (CR 729.5), mutate trigger; Transform/DFC (CR 701.28, 712): CardFace struct on CardDefinition, is_transformed on GameObject, layer system face resolution, Command::Transform; Daybound/Nightbound (CR 702.146): DayNight enum on GameState, automatic transform triggers; Disturb (CR 702.145): AltCostKind::Disturb graveyard casting, exile replacement; Craft (CR 702.167): activated ability with material validation; Morph/Megamorph (CR 702.37): FaceDownKind enum on GameObject, AltCostKind::Morph casting, Command::TurnFaceUp + TurnFaceUpMethod, face-down layer override (2/2 colorless no abilities), FaceDownRevealed event (CR 708.9), Manifest (CR 701.40) + Cloak (CR 701.58) Effect variants, Disguise (CR 702.162) ward-2 face-down; Type Consolidation (RC-1 through RC-3): CastSpell 32→13 fields (AdditionalCost vec), SOK 62→~20 (KeywordTrigger), AbilDef 64→55 (AltCastAbility), Designations bitfield
- `testing/`: Script replay harness (`crates/engine/src/testing/replay_harness.rs` — public, shared with replay viewer), ~112 approved game scripts (scripts 200-204 approved; 187/190/191/195/196/197/198 pending_review), ~1934 tests; 6-player test suite; 54 property invariant tests; `declare_attackers`/`declare_blockers`/`crew_vehicle`/`improvise`/`suspend_card`/`cast_spell_modal`/`cast_spell_fuse`/`cast_spell_spree`/`cast_with_mutate`/`turn_face_up`/`search_library` action types; `activate_ability` with `discard_card_name` (Blood tokens, B14); `activate_loyalty_ability` (PB-14); `PlayerAction.gift_opponent` field (script_schema.rs) for Gift spells
- `benches/`: criterion benchmarks (engine_perf.rs) — priority_cycle_4p: 23µs, priority_cycle_6p: 37µs, sba_check: 14µs, full_turn_4p: 205µs, full_turn_6p: 303µs
- `tools/replay-viewer/`: axum HTTP server + Svelte 5 frontend; 5 API endpoints; full StateViewModel serialization; 12 Svelte components (PlayerPanel, ZoneBattlefield, ZoneStack, ZoneHand, ZoneGraveyard, ZoneExile, PhaseIndicator, EventTimeline, ScriptPicker, CombatView, CardDisplay, AssertionBadges); diff highlighting; keyboard nav
- All 36 corner cases: 32 COVERED, 4 GAP, 0 DEFERRED — Morph/Manifest face-down (case 30) promoted from DEFERRED to COVERED (Morph mini-milestone)

### Known Issue Summary (from code reviews)
- **HIGH open**: 0 — all resolved through M9.4
- **MEDIUM open**: 2 — pre-M8 deferred to M10+ (MR-M7-09, MR-M7-12)
- **~76 LOW open**: schema improvements, partial name matching, FTS trigger gaps, stale replacement cleanup, hidden-info gaps, HashMap usage, ~22 type consolidation stale doc comments — deferred, address opportunistically
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
| Codebase Analysis | `codebase_analysis_220260228.md` | Comprehensive codebase snapshot (2026-02-28): architecture, file inventory, stats |
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
- **rust-analyzer**: semantic code navigation — hover, definition, references, implementations, incoming/outgoing calls, workspace symbols. Call `rust_analyzer_stop` when done to free ~2.5GB RAM. First call triggers ~70s indexing warmup. Results default to 50 max; pass `limit` to override. See `memory/MEMORY.md` "rust-analyzer MCP Server" section for details.

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

- `/start-session` — orientation: git log, workstream state, dispatch table
- `/start-work W1-B3` — claim a workstream before coding (prevents parallel collisions)
- `/end-session` — release claim, write handoff, update memory
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
