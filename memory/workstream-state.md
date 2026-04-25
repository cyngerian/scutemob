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
| W3: LOW Remediation | — | available | — | W3-LOW sprint-1 + sprint-2 shipped 2026-04-25: 13 LOWs closed (SR-FS-01, PB-N-L01, BASELINE-CLIPPY-01..06, PB-S-L02..L06). ~50→~45 open. Tests 2686→2689. Clippy baseline actually clean. |
| W4: M10 Networking | — | not-started | — | After W1 completes |
| W5: Card Authoring | — | **RETIRED** | — | Replaced by W6. See `docs/primitive-card-plan.md` |
| W6: Primitive + Card Authoring | — | available | — | PB-T merged 2026-04-20. 2686 tests. W6 old-queue exhausted. Workstream coordination ESM-managed; this file kept for history. |

**Status values**: `available` (free to claim), `ACTIVE` (session working on it),
`paused` (partially done, session ended mid-task), `not-started` (blocked/deferred),
`RETIRED` (replaced by another workstream)

## Last Handoff

**Date**: 2026-04-25 ~16:00–19:00 EDT (W3-LOW sprint-1 + sprint-2 chain dispatch)
**Workstream**: W3: LOW Remediation (ESM-managed)
**Task**: pick LOW cleanup over PB-SFT re-triage → /dispatch sprint-1 (T1 mechanical) → /collect → /dispatch sprint-2 (T3 PB-S base-char sweep) → /collect+fixup → push

**Completed**:
- **W3-LOW sprint-1 shipped** (`scutemob-6`, merged `c6c3592b`, --no-ff): T1 mechanical cleanup, ~14 min wall clock. SR-FS-01 closed (`first_strike_damage_resolved` confirmed already-removed by grep). PB-N-L01 indentation reflowed in 5 card defs (grim_haruspex, cruel_celebrant, blood_artist, marionette_apprentice, syr_konrad_the_grim). BASELINE-CLIPPY-04 (`fn on_battlefield` in scavenge.rs) deleted + 27 additional clippy warnings fixed (len_zero, manual_while_let_loop, field_reassign_with_default, items_after_test_module, too_many_arguments, etc.). `cargo clippy --all-targets -- -D warnings` now ACTUALLY exits 0 (PB-T's prior claim was wrong/regressed). 4 W3: commits, 54 files, net −117 LOC, zero new `#[allow]`. Tests still 2686. Audit trail: `~~strikethrough~~` + `**Status: CLOSED 2026-04-25**` annotations preserve history in remediation doc.
- **W3-LOW sprint-2 shipped** (`scutemob-7`, merged `c7a93c5e` + `afd7c34d` artifact-fixup, --no-ff): T3 behavioral, ~38 min wall clock. PB-S-L02 Channel/graveyard activation_zone, PB-S-L03 sacrifice-self cost path, PB-S-L04 sacrifice-filter cost path → all read `calculate_characteristics(state, id)` (CR 613.1f). PB-S-L05 granted-index invariant documented at callsite (option-b: debug_assert attempted then removed as false-positive). PB-S-L06 `test_humility_before_grant_preserves_grant` added (CR 613.1f layer ordering). +3 regression tests in `crates/engine/tests/animated_creature_sacrifice_cost.rs` (sac-self + sac-filter) + `grant_activated_ability.rs` (Humility-before-Rite). Tests 2686→2689. Worker delegation chain: primitive-impl-runner → primitive-impl-reviewer → /review Opus PASS. 9 W3: commits.
- **Worker-worktree contamination caught + fixed**: sprint-2 worker's fmt commit `4c77bf50` broadly `git add`-ed and bundled the .claude/skills/ deletion artifact (27 deleted) + .esm/worker.md (tracked addition). Caught at post-merge `git diff main..HEAD --stat`. Applied fix recipe: `git checkout HEAD^1 -- .claude/skills/` + `git rm .esm/worker.md`. Kept the worker's intentional `.claude/skills/review/SKILL.md` add (useful /review slash command). Recipe now documented in workstream-state.md hazards.
- **Pushed**: `f36f0792`, `65b3a492` to `origin/main`. Branch up to date.
- **CLAUDE.md updated** twice (`f36f0792`, `65b3a492`): test count 2686→2689, LOW count ~58→~45, Last Updated 2026-04-25.

**Not done / deferred**:
- 4 SR-PRO-03/04 + SR-FS-02/03 additive coverage tests (Option B from session menu).
- Marisi authoring — DamagedPlayer goad ForEach now shippable, but "opponents can't cast spells during combat" remains a real DSL gap (would need a phase-scoped CantCast primitive).
- PB-SFT re-triage (Cost::SacrificeFilteredType, rank 3, ~12 live TODOs) and PB-CC re-triage (EffectAmount::CounterCount, rank 6, ~10 live TODOs) — neither has a queue entry.
- ~45 LOWs open (carried: BASELINE-LKI-01, PB-Q4-M01, marisi stale TODO, 11 PB-T LOWs, 5 PB-P LOWs, 1 PB-D LOW).

**Next session**:
- Three candidates: (a) Option B test-coverage fill (4 additive tests, T1 risk), (b) marisi authoring + PB-CC-Combat (Cant Cast During Combat) primitive, (c) PB-SFT re-triage + dispatch.
- Worker-worktree contamination protocol: include the fix recipe call in /collect skill OR have workers explicitly avoid `git add -A` in worktree commits.

**Hazards**:
- **Worker-worktree `.claude/skills/` deletion artifact (still relevant; recipe documented)** — see hazards section above. `git diff main..HEAD --stat` is the must-do post-merge check.
- **Validator greedy-consume bug class**: PB-T's E1 finding (first-pass validator rejected CR-legal out-of-slot-order targets) — any Vec-of-target validator can have this failure mode. Two-pass best-fit pattern (collect candidates per slot → bipartite match) is the fix.
- **Handoff "clippy clean" claims can be wrong**: PB-T claimed BASELINE-CLIPPY-01..06 resolved; sprint-1 found they regressed (or never were). Always run `cargo clippy --all-targets -- -D warnings` independently before claiming clean state in handoff notes.
- **Carried-forward LOWs**: BASELINE-LKI-01, PB-Q4-M01, PB-D marisi stale-TODO (now reclassified as authorable not stale), 11 PB-T LOWs, 5 PB-P LOWs.

**Commit prefix used**: worker-agent-generated (`W3:` / `task scutemob-N:`) + coordinator `chore:` for CLAUDE.md/workstream-state.md updates. Followed convention.

## Handoff History

### 2026-04-20 (PB-T single-worker dispatch) — W6: TargetRequirement::UpToN

- W6 re-triage (pre-dispatch): old queue (PB-R/Q3/U/V/W/Y/Q2/Q5) verified 0-1 live TODOs each; new-rank candidates identified (Cost::SacrificeFilteredType rank 3 ~12 live, EffectAmount::CounterCount rank 6 ~10 live). PB-T picked. **PB-T shipped** (`scutemob-5`, merged `2d447e93`): `TargetRequirement::UpToN { count, inner }` per CR 601.2c. Two-pass best-fit validator (out-of-slot-order legal). Auto-target routing for `UpToN{Player}` + nested `UpToN`. HASH 7→8. 22-card oracle sweep → 14 CONFIRMED (64% yield). 14 cards unblocked. 13 tests in `pbt_up_to_n_targets.rs`. Review: needs-fix (1 HIGH validator + 5 MEDIUM) → fix → re-review PASS. Tests 2673→2686. Wall clock ~113 min.

### 2026-04-19 (chain-dispatch session) — W6: PB-P + PB-L shipped sequentially via ESM

- **Push** (`52e2c9dc..872ea5d2`): 11 commits pushed to `origin/main` in two pushes. **PB-P shipped** (`scutemob-3`, merged `8ba9c5b7`): `EffectAmount::PowerOfSacrificedCreature` + `AdditionalCost::Sacrifice` reshape to `{ ids, lki_powers }` for CR 608.2b LKI capture-by-value. 3 cards (altar_of_dementia, greater_good, lifes_legacy). HASH 5→6. Review PASS-WITH-NOTES (5 LOW). **PB-L shipped** (`scutemob-4`, merged `872ea5d2`): Step 0 verdict reversed mid-task (EXISTS → PARTIAL-GAP). No new `TriggerCondition` variant (Landfall = ability word CR 207.2c). Minimal primitive: `ETBTriggerFilter.card_type_filter` + battlefield conversion block in `replay_harness.rs`. 3 cards + 5 TODO rewrites. HASH 6→7. Memo `memory/primitives/pb-note-L-collapsed.md`. **Chain-dispatch pattern validated**: single coordinator ran two `/dispatch` → poll → `/collect` cycles, no user intervention mid-chain. Tests 2655→2673.

### 2026-04-19 (A/B session) — W6: ESM install, PB-D A/B, dispatch skill hardening

- ESM install committed (`aca3035e`, `a253c24f`); PB-D A/B via `/dispatch` (scutemob-1 inline 68 files w/ scope creep, scutemob-2 agent-delegated 14 files PASS-WITH-NOTES, scutemob-2 merged at `72cddb62`); dispatch skill hardened (`7d255645`) to require granular TaskCreate list + specialized-agent delegation; two feedback memory files added. PB-D shipped: `TargetController::DamagedPlayer` + 10 dispatch sites + 6 card defs + 7 MANDATORY tests. Hash 4→5. Tests 2648→2655. 1 LOW carried (marisi_breaker TODO).

### 2026-04-13 (PB-D planner session) — W6: PB-D plan phase

- Opus planner run (`b9f43bf1`): `memory/primitives/pb-plan-D.md` written. Verdict PASS-AS-NEW-VARIANT (`TargetController::DamagedPlayer`), 6 confirmed cards of 15 classified (~40% yield), ~10 dispatch sites across casting/abilities/effects/hash, hash bump 4→5, 7 mandatory + 2 optional tests. Step 0 stale-TODO sweep returned positive null; Step 1 PB-P pre-check found PB-P is real-but-narrow (real gap is `EffectAmount::PowerOf(SacrificedCreature)` LKI read — Altar of Dementia / Greater Good). BASELINE-LKI-01 verified structurally NOT reaching PB-D. 0 stop-and-flags. `memory/primitive-wip.md` halted at phase=plan-complete pending oversight greenlight.

### 2026-04-13 (PB-N close session) — W6: PB-N full pipeline

- Full pipeline (plan → implement → review → fix → re-review → close) under coordinator oversight. Step 0 stale-TODO sweep (`fc83d9d0`) shipped bootleggers_stash as first filtered `LayerModification::AddActivatedAbility` grant on `LandsYouControl`. PB-N plan verdict PASS-AS-FIELD-ADDITION (`filter: Option<TargetFilter>` + `triggering_creature_filter` mirroring `combat_damage_filter`). Implement (`d343e1ba`, `7e7d426a`): 7 engine files, hash sentinel 3→4 promoted to `pub const HASH_SCHEMA_VERSION`, `combat_damage_filter` tightened to damage-only (latent bug fix), 56 mechanical card-def backfills, 4 cards + 9 tests (2637 → 2646). Review found 2 HIGH + 3 MEDIUM + 1 LOW; fix phase (`0e5d7cf1`) rewrote Sanctum Seeker drain (no new engine surface), added Utvara Hellkite catch via TODO sweep (yield 4→5), tightened hash assertion, fixed combat_damage_filter regression test. F3 LKI test wedge stop-and-flagged as structurally unreachable — 30-min aura wedge experiment confirmed BASELINE-LKI-01 (death-trigger dispatch re-runs layer filters against graveyard objects, dropping battlefield-gated filters). Close commit logged BASELINE-LKI-01 + BASELINE-CLIPPY-01..06 + PB-N-L01 in remediation doc, added gotcha #39, created 2 new feedback memory files, updated primitive-impl-planner agent with mandatory step 3a (pre-existing TODO sweep). Tests 2637 → 2648. Clippy baseline correction: every prior "clippy clean" handoff was wrong with `--all-targets`; ≥6 pre-existing errors now logged.



