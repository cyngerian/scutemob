# Workstream State

> Coordination file for parallel sessions. Read by `/start-session`, claimed by
> `/start-work`, released by `/end-session`. This file is the source of truth for
> which workstreams are actively being worked on.
>
> **Protocol**: Read before starting. Claim before coding. Release when done.

## Active Claims

| Workstream | Task | Status | Claimed | Notes |
|------------|------|--------|---------|-------|
| W1: Abilities | — | available | — | B16 complete (Dungeon + Ring); all abilities done |
| W2: TUI & Simulator | — | available | — | Phase 1 done; 6 UX fixes done; hardening pending |
| W3: LOW Remediation | — | available | — | **W3 LOW sprint DONE** (S1-S6): 83→29 open (119 closed total). TC-21 done. 2233 tests. |
| W4: M10 Networking | — | not-started | — | After W1 completes |
| W5: Card Authoring | — | **RETIRED** | — | Replaced by W6. See `docs/primitive-card-plan.md` |
| W6: Primitive + Card Authoring | — | available | — | PB-D merged 2026-04-19 via ESM dispatch A/B. Workstream coordination migrating to ESM tasks; this file kept for history. |

**Status values**: `available` (free to claim), `ACTIVE` (session working on it),
`paused` (partially done, session ended mid-task), `not-started` (blocked/deferred),
`RETIRED` (replaced by another workstream)

## Last Handoff

**Date**: 2026-04-19 (post-migration session — first ESM dispatch A/B on scutemob, PB-D shipped)
**Workstream**: W6: Primitive + Card Authoring + coordination migration
**Task**: commit ESM install, A/B PB-D via /dispatch, merge winning run, validate new dispatch skill

**Completed**:
- **ESM install committed** (`aca3035e`, `a253c24f`): `.claude/settings.json` hook, CLAUDE.md ESM section, 10 new ESM slash-command skills, `CLAUDE.md.pre-esm` → gitignored, stray `package-lock.json` separated.
- **PB-D A/B experiment via /dispatch**: two worker tasks (scutemob-1 inline, scutemob-2 agent-delegated) against identical acceptance criteria + plan. scutemob-2 used `primitive-impl-runner` + `primitive-impl-reviewer`, landed 14 files / 4 clean commits / PASS-WITH-NOTES. scutemob-1 ran inline, landed 68 files (54 out-of-scope, including simulator + replay-viewer + TUI drift). scutemob-2 merged at `72cddb62`. scutemob-1 worktree + branch torn down, task marked done with supersession comment.
- **Dispatch skill hardened** (`7d255645`): `.claude/skills/dispatch/SKILL.md` + `.claude/skills/spawn/SKILL.md` now instruct workers to (1) TaskCreate a granular live-updated task list before implementing, and (2) delegate to specialized agents (primitive-impl-runner, ability-impl-runner, bulk-card-author, etc.) rather than grinding inline. Validated by the A/B: inline produced drift; delegated produced tight, reviewed work.
- **Two feedback memory files added**: `feedback_worker_task_list.md`, `feedback_worker_delegate_to_agents.md` (+ MEMORY.md pointers).
- **PB-D shipped**: `TargetController::DamagedPlayer` variant + 10 dispatch sites + 6 card defs (Throat Slitter, Sigil of Sleep, Mistblade Shinobi, Alela, Nature's Will, Balefire Dragon) + 7 MANDATORY tests. Hash sentinel 4→5. Tests 2648 → 2655.
- **1 LOW finding open**: stale TODO in `marisi_breaker_of_the_coil.rs` per `memory/primitives/pb-review-D.md`.

**Not done (deferred to next session)**:
- Update CLAUDE.md "Active Plan" to reflect PB-D merged + queue next PB (PB-P narrow-gap triage, PB-L rank 3).
- Retire workstream-state.md `/start-work`-based claim sections in CLAUDE.md (kept for history but the protocol is now ESM-managed).
- Apply the marisi_breaker stale-TODO LOW fix opportunistically.

**Next session**:
- Plan more ESM tasks (per-PB, following the v2 pattern). Likely candidates: PB-P re-triage (narrow gap: `EffectAmount::PowerOf(SacrificedCreature)` LKI only), PB-L (~3-4 card yield). Oversight makes the call.
- Consider wiring `CARGO_TARGET_DIR` per-worktree to avoid another disk-pressure incident during concurrent dispatches.

**Hazards**:
- **Disk pressure during parallel dispatches**: two concurrent worker builds consumed ~54G of `target/` at peak. Budget: single worker ≈ 15-18G; two parallel ≈ 30-35G; free space should be >50G before dispatching two. Consider shared-but-isolated `CARGO_TARGET_DIR` pattern.
- **ESM advisory-mode attestation noise**: `esm task transition` interprets `--attest branch_exists=true` as a literal branch-name lookup ("Branch 'true' not found"). Cosmetic; transitions still accepted.
- **BASELINE-LKI-01** carried forward: graveyard re-filter dispatch gap. Dedicated audit session still pending.
- **BASELINE-CLIPPY-01..06** carried forward.
- **PB-Q4-M01**: `EnchantFilter` vs `TargetFilter` divergence still open.
- **PB-S residuals L02-L06**: open in `abilities.rs`.

**Commit prefix used**: mostly `chore:` (infra migration work + skill hardening). The worker-delegated PB-D commits used `scutemob-2:` prefix naturally from the worker agent; that's the emerging convention for ESM-dispatched work (`<task_id>:` prefix on worker commits, `chore:`/`W6-prim:` for coordinator commits).

## Handoff History

### 2026-04-13 (PB-D planner session) — W6: PB-D plan phase

- Opus planner run (`b9f43bf1`): `memory/primitives/pb-plan-D.md` written. Verdict PASS-AS-NEW-VARIANT (`TargetController::DamagedPlayer`), 6 confirmed cards of 15 classified (~40% yield), ~10 dispatch sites across casting/abilities/effects/hash, hash bump 4→5, 7 mandatory + 2 optional tests. Step 0 stale-TODO sweep returned positive null; Step 1 PB-P pre-check found PB-P is real-but-narrow (real gap is `EffectAmount::PowerOf(SacrificedCreature)` LKI read — Altar of Dementia / Greater Good). BASELINE-LKI-01 verified structurally NOT reaching PB-D. 0 stop-and-flags. `memory/primitive-wip.md` halted at phase=plan-complete pending oversight greenlight.

### 2026-04-13 (PB-N close session) — W6: PB-N full pipeline

- Full pipeline (plan → implement → review → fix → re-review → close) under coordinator oversight. Step 0 stale-TODO sweep (`fc83d9d0`) shipped bootleggers_stash as first filtered `LayerModification::AddActivatedAbility` grant on `LandsYouControl`. PB-N plan verdict PASS-AS-FIELD-ADDITION (`filter: Option<TargetFilter>` + `triggering_creature_filter` mirroring `combat_damage_filter`). Implement (`d343e1ba`, `7e7d426a`): 7 engine files, hash sentinel 3→4 promoted to `pub const HASH_SCHEMA_VERSION`, `combat_damage_filter` tightened to damage-only (latent bug fix), 56 mechanical card-def backfills, 4 cards + 9 tests (2637 → 2646). Review found 2 HIGH + 3 MEDIUM + 1 LOW; fix phase (`0e5d7cf1`) rewrote Sanctum Seeker drain (no new engine surface), added Utvara Hellkite catch via TODO sweep (yield 4→5), tightened hash assertion, fixed combat_damage_filter regression test. F3 LKI test wedge stop-and-flagged as structurally unreachable — 30-min aura wedge experiment confirmed BASELINE-LKI-01 (death-trigger dispatch re-runs layer filters against graveyard objects, dropping battlefield-gated filters). Close commit logged BASELINE-LKI-01 + BASELINE-CLIPPY-01..06 + PB-N-L01 in remediation doc, added gotcha #39, created 2 new feedback memory files, updated primitive-impl-planner agent with mandatory step 3a (pre-existing TODO sweep). Tests 2637 → 2648. Clippy baseline correction: every prior "clippy clean" handoff was wrong with `--all-targets`; ≥6 pre-existing errors now logged.

### 2026-04-12 (third session) — W6: PB-Q4 full pipeline

- PB-Q4 plan (Opus) + implement (`9c347754`) + review (0 HIGH / 1 MEDIUM / 3 LOW) + fix (`0dd7288a`). New `EnchantFilter` struct (resolves circular dep vs plan's `Box<TargetFilter>`), `EnchantControllerConstraint` enum. 4 cards: Awaken the Ancient, Chained to the Rocks, Ossification, Dimensional Exile. Tests 2625 → 2639. Genju cycle + Corrupted Roots/Spreading Algae deferred — missing trigger types. PB-Q4-M01 + L01 logged.

### 2026-04-12 (second session) — W6: PB-Q close + PB-Q4 yield audit
- PB-Q close (`464d9e79`): deleted gauntlet_of_power.rs + utopia_sprawl.rs, reverted throne_of_eldraine.rs. Removed parked-only engine variants (`ReplacementManaSourceFilter::{BasicLand, EnchantedLand}`, `EffectFilter::AllCreaturesOfChosenColor`). 2627→2625. Fixed CR citation LOWs.
- Reviewer agent hardened: added oracle-vs-filter semantic gate as step 3 in `.claude/agents/primitive-impl-reviewer.md` (5th appearance of verify-existence-not-completeness failure mode).
- Reservations written: PB-Q2/Q3/Q4/Q5 in `docs/primitive-card-plan.md` Phase 1.7 + `docs/project-status.md`.
- Auto-memory: `feedback_pb_yield_calibration.md` — PB planners overcount in-scope cards by 2-3x; discount 40-50%.
- PB-Q4 yield audit (SQLite, not grep): direct LandSubtype yield 10 cards; bundled scope ~20 cards. Verdict: PB-Q4 #1 priority.
- Three verification gates queued for next session (Genju animate-land make-or-break; Chained controller filter; Corrupted Roots disjunction).

### 2026-04-11 (fourth session) — W6: PB-X close + PB-Q plan + implement
- PB-X close (`c502f8fc`). PB-Q plan caught 2 oversight roster errors via MCP. PB-Q implement (`880b7797`): `GameObject.chosen_color`, `ReplacementModification::ChooseColor` + `AddOneManaOfChosenColor`, `ChosenColorRef`, `ReplacementManaSourceFilter`, 2 EffectFilter variants, `Effect::AddManaOfChosenColor`, `apply_mana_production_replacements` refactor, hash sentinel 2→3. 6 cards, 11 tests, 2616→2627. Review deferred per context pressure.

