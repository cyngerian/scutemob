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
| W6: Primitive + Card Authoring | — | available | — | PB-T merged 2026-04-20. 2686 tests. W6 old-queue exhausted. Workstream coordination ESM-managed; this file kept for history. |

**Status values**: `available` (free to claim), `ACTIVE` (session working on it),
`paused` (partially done, session ended mid-task), `not-started` (blocked/deferred),
`RETIRED` (replaced by another workstream)

## Last Handoff

**Date**: 2026-04-20 ~01:10–03:10 EDT (PB-T single-worker dispatch)
**Workstream**: W6: Primitive + Card Authoring (ESM-managed)
**Task**: re-triage W6 backlog → /dispatch PB-T → /collect → update CLAUDE.md

**Completed**:
- **W6 re-triage** (pre-dispatch): old queue (PB-R/Q3/U/V/W/Y/Q2/Q5) verified 0-1 live TODOs each; new-rank candidates identified (Cost::SacrificeFilteredType rank 3 ~12 live, EffectAmount::CounterCount rank 6 ~10 live). PB-T picked for dispatch per user selection.
- **PB-T shipped** (`scutemob-5`, merged `2d447e93`, --no-ff): new `TargetRequirement::UpToN { count, inner }` per CR 601.2c. Two-pass best-fit validator handles out-of-slot-order legal targets (CR 601.2c requires accepting legal target in any declared slot). Auto-target routing for `UpToN{Player}` + nested `UpToN`. HASH_SCHEMA_VERSION 7→8. 22-card oracle sweep → 14 CONFIRMED (64% yield, above filter-PB 50-65% midpoint — re-calibration data point). 14 card defs unblocked: elder_deep_fiend, force_of_vigor, marang_river_regent, sorin_lord_of_innistrad, basri_ket, tamiyo_field_researcher, teferi_temporal_archmage, tyvar_jubilant_brawler, tyvar_kell, teferi_time_raveler, kogla_the_titan_ape, moonsnare_specialist, skemfar_elderhall, sword_of_sinew_and_steel. 13 tests in `pbt_up_to_n_targets.rs` (M1 zero-target, M2 partial 1-of-2, M3-M10, O1-O3). Review cycle: needs-fix (1 HIGH E1 greedy-consume validator + 5 MEDIUM) → fix → re-review PASS. 11 new non-blocking LOWs filed. Tests 2673→2686. Wall clock ~113 min (planner + runner + reviewer + fix-runner + re-reviewer).
- **CLAUDE.md updated** (`160200c9`): test count 2673→2686, PB-T added to DONE list, LOW count 47→58, Last Updated 2026-04-20.

**Not done / deferred**:
- `marisi_breaker_of_the_coil.rs` stale-TODO LOW (carried forward from PB-D review, still open).
- 11 PB-T LOWs logged; none blocking.
- 9 commits ahead of `origin/main` (`a21ec971..160200c9`) — unpushed.

**Next session**:
- **Two candidates**: (a) LOW cleanup sprint (~58 open, `W3:` prefix) or (b) re-triage to spin up PB-SFT (Cost::SacrificeFilteredType, rank 3, ~12 live TODOs, no queue entry yet).
- Push first (`git push`) to sync origin.

**Hazards**:
- **Worker-worktree `.claude/skills/` deletion artifact (HIT during sprint-2 collect 2026-04-25)**: `esm worktree create` (esm-cli:565-577) intentionally `rmtree`s worker's `.claude/skills/` and replaces with review-only set from `$ESM/client/worker-skills/`. Workers will correctly report these as pre-existing `D` entries in `git status`. Main branch untouched IF the worker keeps these as worktree-pending uncommitted changes. **HAZARD**: if a worker `git add`s broadly (e.g. `git add -A` in a fmt commit), the deletions get bundled and DO merge into main. Sprint-2's `4c77bf50` did this; collect had to do a fixup. **Pre-merge scope check: use `git diff main..HEAD --stat`, not `git diff main --stat`.** **Fix recipe (post-merge contamination)**: after `esm worktree merge`, run `git checkout HEAD^1 -- .claude/skills/` to restore from pre-merge main, then `git ls-tree -r HEAD -- .esm/` and `git rm` any tracked `.esm/worker.md`, then commit as a `chore:` fixup. Skills count should match origin/main pre-merge (currently 26+ when adding intentional new skills).
- **Validator greedy-consume bug class**: PB-T's E1 finding (first-pass validator rejected CR-legal out-of-slot-order targets) suggests any Vec-of-target validator can have this failure mode. Two-pass best-fit pattern (collect candidates per slot → bipartite match) is the fix. Watch for this in future target-validation work.
- **BASELINE-LKI-01**, **BASELINE-CLIPPY-01..06**, **PB-Q4-M01**, **PB-S residuals L02-L06**, **PB-D marisi stale-TODO**: all carried forward.

**Commit prefix used**: worker-agent-generated (`scutemob-5:` / `task scutemob-5:`) + coordinator `chore:` for CLAUDE.md bump. No W6-prim/cards prefixes this session — ESM convention dominant.

## Handoff History

### 2026-04-19 (chain-dispatch session) — W6: PB-P + PB-L shipped sequentially via ESM

- **Push** (`52e2c9dc..872ea5d2`): 11 commits pushed to `origin/main` in two pushes. **PB-P shipped** (`scutemob-3`, merged `8ba9c5b7`): `EffectAmount::PowerOfSacrificedCreature` + `AdditionalCost::Sacrifice` reshape to `{ ids, lki_powers }` for CR 608.2b LKI capture-by-value. 3 cards (altar_of_dementia, greater_good, lifes_legacy). HASH 5→6. Review PASS-WITH-NOTES (5 LOW). **PB-L shipped** (`scutemob-4`, merged `872ea5d2`): Step 0 verdict reversed mid-task (EXISTS → PARTIAL-GAP). No new `TriggerCondition` variant (Landfall = ability word CR 207.2c). Minimal primitive: `ETBTriggerFilter.card_type_filter` + battlefield conversion block in `replay_harness.rs`. 3 cards + 5 TODO rewrites. HASH 6→7. Memo `memory/primitives/pb-note-L-collapsed.md`. **Chain-dispatch pattern validated**: single coordinator ran two `/dispatch` → poll → `/collect` cycles, no user intervention mid-chain. Tests 2655→2673.

### 2026-04-19 (A/B session) — W6: ESM install, PB-D A/B, dispatch skill hardening

- ESM install committed (`aca3035e`, `a253c24f`); PB-D A/B via `/dispatch` (scutemob-1 inline 68 files w/ scope creep, scutemob-2 agent-delegated 14 files PASS-WITH-NOTES, scutemob-2 merged at `72cddb62`); dispatch skill hardened (`7d255645`) to require granular TaskCreate list + specialized-agent delegation; two feedback memory files added. PB-D shipped: `TargetController::DamagedPlayer` + 10 dispatch sites + 6 card defs + 7 MANDATORY tests. Hash 4→5. Tests 2648→2655. 1 LOW carried (marisi_breaker TODO).

### 2026-04-13 (PB-D planner session) — W6: PB-D plan phase

- Opus planner run (`b9f43bf1`): `memory/primitives/pb-plan-D.md` written. Verdict PASS-AS-NEW-VARIANT (`TargetController::DamagedPlayer`), 6 confirmed cards of 15 classified (~40% yield), ~10 dispatch sites across casting/abilities/effects/hash, hash bump 4→5, 7 mandatory + 2 optional tests. Step 0 stale-TODO sweep returned positive null; Step 1 PB-P pre-check found PB-P is real-but-narrow (real gap is `EffectAmount::PowerOf(SacrificedCreature)` LKI read — Altar of Dementia / Greater Good). BASELINE-LKI-01 verified structurally NOT reaching PB-D. 0 stop-and-flags. `memory/primitive-wip.md` halted at phase=plan-complete pending oversight greenlight.

### 2026-04-13 (PB-N close session) — W6: PB-N full pipeline

- Full pipeline (plan → implement → review → fix → re-review → close) under coordinator oversight. Step 0 stale-TODO sweep (`fc83d9d0`) shipped bootleggers_stash as first filtered `LayerModification::AddActivatedAbility` grant on `LandsYouControl`. PB-N plan verdict PASS-AS-FIELD-ADDITION (`filter: Option<TargetFilter>` + `triggering_creature_filter` mirroring `combat_damage_filter`). Implement (`d343e1ba`, `7e7d426a`): 7 engine files, hash sentinel 3→4 promoted to `pub const HASH_SCHEMA_VERSION`, `combat_damage_filter` tightened to damage-only (latent bug fix), 56 mechanical card-def backfills, 4 cards + 9 tests (2637 → 2646). Review found 2 HIGH + 3 MEDIUM + 1 LOW; fix phase (`0e5d7cf1`) rewrote Sanctum Seeker drain (no new engine surface), added Utvara Hellkite catch via TODO sweep (yield 4→5), tightened hash assertion, fixed combat_damage_filter regression test. F3 LKI test wedge stop-and-flagged as structurally unreachable — 30-min aura wedge experiment confirmed BASELINE-LKI-01 (death-trigger dispatch re-runs layer filters against graveyard objects, dropping battlefield-gated filters). Close commit logged BASELINE-LKI-01 + BASELINE-CLIPPY-01..06 + PB-N-L01 in remediation doc, added gotcha #39, created 2 new feedback memory files, updated primitive-impl-planner agent with mandatory step 3a (pre-existing TODO sweep). Tests 2637 → 2648. Clippy baseline correction: every prior "clippy clean" handoff was wrong with `--all-targets`; ≥6 pre-existing errors now logged.

### 2026-04-12 (third session) — W6: PB-Q4 full pipeline

- PB-Q4 plan (Opus) + implement (`9c347754`) + review (0 HIGH / 1 MEDIUM / 3 LOW) + fix (`0dd7288a`). New `EnchantFilter` struct (resolves circular dep vs plan's `Box<TargetFilter>`), `EnchantControllerConstraint` enum. 4 cards: Awaken the Ancient, Chained to the Rocks, Ossification, Dimensional Exile. Tests 2625 → 2639. Genju cycle + Corrupted Roots/Spreading Algae deferred — missing trigger types. PB-Q4-M01 + L01 logged.


