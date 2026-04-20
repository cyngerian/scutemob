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
| W6: Primitive + Card Authoring | — | available | — | PB-P + PB-L merged 2026-04-19 via ESM chain-dispatch. 2673 tests. Workstream coordination ESM-managed; this file kept for history. |

**Status values**: `available` (free to claim), `ACTIVE` (session working on it),
`paused` (partially done, session ended mid-task), `not-started` (blocked/deferred),
`RETIRED` (replaced by another workstream)

## Last Handoff

**Date**: 2026-04-19 ~20:45 EDT (chain-dispatch session — PB-P + PB-L shipped sequentially via ESM)
**Workstream**: W6: Primitive + Card Authoring (ESM-managed)
**Task**: push prior commits; chain-dispatch PB-P → collect → PB-L → collect

**Completed**:
- **Push** (`52e2c9dc..872ea5d2`): 11 commits pushed to `origin/main` in two pushes (one pre-dispatch, one post-PB-L).
- **PB-P shipped** (`scutemob-3`, merged `8ba9c5b7`, fast-forward): new `EffectAmount::PowerOfSacrificedCreature` + reshape `AdditionalCost::Sacrifice(Vec<ObjectId>)` → `{ ids, lki_powers }` for CR 608.2b LKI capture-by-value. 23 existing test files touched mechanically. 3 card defs: altar_of_dementia, greater_good, lifes_legacy. 8 mandatory + 3 optional tests in `pbp_power_of_sacrificed_creature.rs` (1168 LOC). HASH_SCHEMA_VERSION 5→6. Review PASS-WITH-NOTES (0 HIGH/MEDIUM, 5 LOW). Wall clock ~92 min (planner + runner + reviewer, all Opus-backed agents).
- **PB-L shipped** (`scutemob-4`, merged `872ea5d2`, fast-forward): Step 0 verdict reversed mid-task — first pass said "EXISTS, collapse to stale-TODO sweep," but writing tests surfaced a real dispatch gap in `enrich_spec_from_def` (no battlefield-conversion block for `WheneverPermanentEntersBattlefield`). Minimal engine primitive added: `ETBTriggerFilter.card_type_filter: Option<CardType>` + new conversion block in `replay_harness.rs:2396-2488` parallel to the Alliance block. No new `TriggerCondition` variant (Landfall is an ability word, CR 207.2c). HASH_SCHEMA_VERSION 6→7. 3 card defs authored (khalni_heart_expedition, druid_class, omnath_locus_of_rage) + 5 TODO rewrites on compound-blocked cards. 9 tests in `pb_l_landfall.rs` (528 LOC). Memo at `memory/primitives/pb-note-L-collapsed.md` documents both passes transparently. Wall clock ~37 min (no planner/runner/reviewer chain; card-fix-applicator agent + inline engine work).
- **Chain-dispatch pattern validated**: single coordinator-driven sequence of two `/dispatch` cycles, each followed by background poll + `/collect`, no user intervention mid-chain. ESM outage mid-session was absorbed by user pausing.

**Not done / deferred**:
- `marisi_breaker_of_the_coil.rs` stale-TODO LOW (carried forward from PB-D review).
- Retire `memory/workstream-state.md` `/start-work` sections in CLAUDE.md (ESM-managed now; file kept for history).

**Next session**:
- Queue is thin. Old-queue demoted PBs (PB-R, PB-Q3, PB-T, PB-U, PB-V, PB-W, PB-Y, PB-Q2/Q5) remain opportunistic. Re-triage the W6 backlog or pivot to LOW cleanup (~47 open).

**Hazards**:
- **Worker-worktree `.claude/skills/` deletion artifact**: PB-P worktree showed all `.claude/skills/*/SKILL.md` as uncommitted `D` entries before merge. Tree hashes on both main and HEAD matched; deletions were working-copy-only and `esm worktree merge` correctly discarded them. **Always use `git diff main..HEAD --stat` (branch vs branch), not `git diff main --stat` (working-tree vs branch), when pre-merge-checking a worker's scope.** Cosmetic but easy to misread as scope creep.
- **Test count drift during cost-shape reshapes**: PB-P touched 23 existing test files mechanically (old `AdditionalCost::Sacrifice(vec![id])` → struct form). Always audit such mass changes against scope-creep but expect mechanical touch-ups to dominate.
- **BASELINE-LKI-01**, **BASELINE-CLIPPY-01..06**, **PB-Q4-M01**, **PB-S residuals L02-L06**: all carried forward.

**Commit prefix used**: worker-agent-generated (`scutemob-3:` / `scutemob-4:`) as the emerging ESM convention. No coordinator commits this session — all work landed inside dispatched tasks.

## Handoff History

### 2026-04-19 (A/B session) — W6: ESM install, PB-D A/B, dispatch skill hardening

- ESM install committed (`aca3035e`, `a253c24f`); PB-D A/B via `/dispatch` (scutemob-1 inline 68 files w/ scope creep, scutemob-2 agent-delegated 14 files PASS-WITH-NOTES, scutemob-2 merged at `72cddb62`); dispatch skill hardened (`7d255645`) to require granular TaskCreate list + specialized-agent delegation; two feedback memory files added. PB-D shipped: `TargetController::DamagedPlayer` + 10 dispatch sites + 6 card defs + 7 MANDATORY tests. Hash 4→5. Tests 2648→2655. 1 LOW carried (marisi_breaker TODO).

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


