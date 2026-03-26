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
| W6: Primitive + Card Authoring | — | available | — | **PB-31 DONE**. PB-32 next. |

**Status values**: `available` (free to claim), `ACTIVE` (session working on it),
`paused` (partially done, session ended mid-task), `not-started` (blocked/deferred),
`RETIRED` (replaced by another workstream)

## Last Handoff

**Date**: 2026-03-26
**Workstream**: W6 (PB-31)
**Task**: PB-31 Cost primitives (RemoveCounter, SpellAdditionalCost)

**Completed**:
- **PB-31 DONE**: G-16 Cost::RemoveCounter + G-17 SpellAdditionalCost. Engine: Cost::RemoveCounter variant, SpellAdditionalCost enum (5 variants), ActivationCost.remove_counter_cost field, casting.rs spell sacrifice validation+execution, abilities.rs counter removal payment, hash.rs updates. 18 card defs fixed (10 G-16 + 8 G-17). 12 new tests. Review: 2M fixed (Jitte trigger TODO, Life's Legacy placeholder). 2383 tests, 0 clippy.
- Commits: b9f8efa (implement), aeb87d5 (fixes).

**Next**:
1. Continue gap closure: PB-32 (static/effect primitives — additional lands, prevention, control change, land animation, ~39 cards)
2. Deferred to PB-37: Jitte unqualified combat damage trigger, Life's Legacy SacrificedCreaturePower, Ghave cross-creature counter removal, Ramos once-per-turn, Crucible X counters, Tekuthal multi-source, Plumb the Forbidden variable sacrifice, Flare of Fortitude alt cost
3. Backfill sweeps accumulating (~480+ cards across PB-23–31)

**Hazards**:
- Backfill sweeps accumulating (~480+ cards across PB-23–31)
- Planner agent created spurious docs (engine_explanation.md, scutemob-architecture-review.md) — included in impl commit, harmless but should be cleaned up

**Commit prefix used**: `W6-prim:`

## Handoff History

### 2026-03-26 — W6: PB-31
- PB-31: G-16 RemoveCounter + G-17 SpellAdditionalCost, 18 card defs fixed, 2M fixed. 2383 tests. Commits: b9f8efa, aeb87d5.

### 2026-03-25 — W6: PB-30
- PB-30: G-8 Combat damage triggers, 27 card defs fixed, 5H 4M fixed. 2371 tests. Commits: b5577c7, b8c8dc6.

### 2026-03-25 — W6: PB-28 + PB-29
- PB-28: G-6 CDA, 9 card defs fixed, 1M fixed. 2353 tests. Commits: ee56134, 3882c1b.
- PB-29: G-7 Cost reduction statics, 13 card defs fixed, 1H fixed. 2363 tests. Commits: e562ec0, bf6e992.

### 2026-03-25 — W6: PB-27 X-cost spells
- PB-27: G-5 X-cost spells, 15 card defs fixed, 2M fixed. 2344 tests.
- Commits: 7972512, 04664ea.

### 2026-03-24 — W6: PB-26 trigger variants
- PB-26: 8 trigger gaps (G-4 through G-15), ~55 card defs fixed, 1H 2M fixed. 2334 tests.
- Commits: b25abc3, 00a334d.

### 2026-03-23 — W6: PB-24 conditional statics + TUI redesign
- PB-24: 13 card defs fixed, 1H 2M fixed, ~170 defs updated with `condition: None`. 2302 tests.
- TUI redesign: 8→4 tabs, live card scanner, pipeline tab, worker status display. Zero warnings.
- Commits: a69d458, aa23d26, 12b594d, 4a89567.

### 2026-03-23 — Executive/oversight: DSL gap plan + TUI improvements
- Full DSL gap audit (1,348 TODOs, 31 gap types), created `docs/dsl-gap-closure-plan.md` (PB-23 through PB-37)
- TUI: live card health scanner, dynamic stats, pipeline funnel
- Reviewed PB-23 (34 cards fixed, 2H fixed). Commits: 2665479, 95da16f.

### 2026-03-23 — W6: PB-23 controller-filtered creature triggers
- PB-23: 34 cards fixed, 2H 11M fixed. New: controller filter on WheneverCreatureDies, WheneverCreatureYouControlAttacks, WheneverCreatureYouControlDealsCombatDamageToPlayer.
- Commits: b5066d4, 5104b27. 2291 tests.

### 2026-03-23 — W6: Phase 2 authoring A-20 through A-23
- A-20 pump-buff (27), A-21 counters-plus (49), A-22 equipment (11), A-23 death-trigger (34). 121 total. 4H fixed.
- Commits: e5b0436, ec08405. 2281 tests.

### 2026-03-23 — W6: Phase 2 authoring A-19 token-create S44-S52
- A-19 token-create S44-S52: 96 new cards. Reviewed, 3H fixed.
- Commits: 5d967ca, 83c1302. 2281 tests.
