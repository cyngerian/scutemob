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
| W3: LOW Remediation | LOW remediation — T2/T3 items | available | — | Phase 0 complete; T2 done; T3 ManaPool pending |
| W4: M10 Networking | — | not-started | — | After W1 completes |
| W5: Card Authoring | — | **RETIRED** | — | Replaced by W6. See `docs/primitive-card-plan.md` |
| W6: Primitive + Card Authoring | PB-8: Cost reduction statics | ACTIVE | 2026-03-14 | **TOP PRIORITY**. PB-0 through PB-7 done. Plan: `docs/primitive-card-plan.md` |

**Status values**: `available` (free to claim), `ACTIVE` (session working on it),
`paused` (partially done, session ended mid-task), `not-started` (blocked/deferred),
`RETIRED` (replaced by another workstream)

## Last Handoff

**Date**: 2026-03-14
**Workstream**: W6: Primitive + Card Authoring
**Task**: PB-7 — Count-based scaling (29 cards)

**Completed**:
- 3 new EffectAmount variants: PermanentCount, DevotionTo (CR 700.5), CounterCount
- 1 new Effect variant: AddManaScaled (variable mana of specific color)
- Hash discriminants 6-8 (EffectAmount) + 56 (Effect) in hash.rs
- Resolution logic in effects/mod.rs for all 4 new variants
- Mana ability filter updated in replay_harness.rs for AddManaScaled
- 8 new tests in count_based_scaling.rs
- 5 card defs fixed: gaeas_cradle, cabal_coffers, cabal_stronghold, malakir_bloodwitch, crypt_of_agadeem
- 1 card def corrected: call_of_the_ring (wrong oracle text → fixed to {1}{B} upkeep trigger)
- 3 card def TODOs updated: craterhoof_behemoth, reckless_one, nykthos_shrine_to_nyx
- Game script 206 set to pending_review (based on old incorrect oracle text)
- Commit 399c8da; 2020 tests, 0 clippy warnings

**Next**:
1. Follow execution order in `docs/primitive-card-plan.md` — PB-8 through PB-21
2. **PB-8**: Cost reduction statics (10 cards) — LayerModification::ModifySpellCost
3. Many PB-7 cards remain blocked on deeper gaps (dynamic LayerModification for CDA */* creatures, mass continuous effect grants with dynamic amounts)
4. ~23 PB-5 + ~20 PB-6 cards remain blocked on future primitives

**Hazards**: CLAUDE.md has uncommitted changes (test count update). No conflicts.

**Commit prefix used**: `W6-prim:`

## Handoff History

### 2026-03-14 — W6: PB-7 count-based scaling (29 cards)
- 3 EffectAmount variants + AddManaScaled + 5 card fixes + 8 tests; commit 399c8da; 2020 tests

### 2026-03-14 — W6: PB-6 static grant with controller filter (30 cards)
- 3 EffectFilter variants + 10 card fixes + 6 tests; commit f722014; 2012 tests

### 2026-03-14 — W6: PB-5 targeted activated/triggered abilities (32 cards)
- targets field on Activated/Triggered + validation + 9 card fixes + 9 tests; commit 9ca054a; 2006 tests

### 2026-03-14 — W6: PB-4 sacrifice as activation cost (26 cards)
- SacrificeFilter + Cost::SacrificeSelf + 13 card fixes + 4 tests; commit 0344539; 1997 tests

### 2026-03-14 — W6: PB-3 shockland pay-life-or-tapped (10 cards)
- EntersTappedUnlessPayLife(u32) variant + 10 card defs fixed; commit 734cfff; 1993 tests
