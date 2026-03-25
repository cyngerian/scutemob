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
| W6: Primitive + Card Authoring | PB-28: CDA / count-based P/T | ACTIVE | 2026-03-25 | PB-28 next (~32 cards) |

**Status values**: `available` (free to claim), `ACTIVE` (session working on it),
`paused` (partially done, session ended mid-task), `not-started` (blocked/deferred),
`RETIRED` (replaced by another workstream)

## Last Handoff

**Date**: 2026-03-25
**Workstream**: W6 (PB-27)
**Task**: PB-27 X-cost spells

**Completed**:
- **PB-27 DONE**: G-5 X-cost spells. Engine: Condition::XValueAtLeast, Effect::Repeat, x_value on ActivateAbility, ETB x_value propagation (CR 107.3m), replay harness wiring. 15 card defs fixed (7 fully, 8 partially). 10 new tests. Review: 2M fixed. 2344 tests, 0 clippy.
- Commits: 7972512 (implement), 04664ea (fixes).

**Next**:
1. Continue gap closure: PB-28 (CDA / count-based P/T, ~32 cards) is next
2. ~27 remaining X-cost cards need other PB gaps (count-based patterns → PB-28, variable sacrifice → future)
3. Backfill sweeps for PB-23/24/25/26/27 accumulating — consider dedicated backfill session

**Hazards**:
- Backfill sweeps accumulating (PB-23 ~111, PB-24 ~188, PB-25 ~70, PB-26 ~17, PB-27 ~27)

**Commit prefix used**: `W6-prim:`

## Handoff History

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
