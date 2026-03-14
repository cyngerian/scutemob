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
| W6: Primitive + Card Authoring | PB-7: Count-based scaling (29 cards) | ACTIVE | 2026-03-14 | **TOP PRIORITY**. PB-0 through PB-6 done. Plan: `docs/primitive-card-plan.md` |

**Status values**: `available` (free to claim), `ACTIVE` (session working on it),
`paused` (partially done, session ended mid-task), `not-started` (blocked/deferred),
`RETIRED` (replaced by another workstream)

## Last Handoff

**Date**: 2026-03-14
**Workstream**: W6: Primitive + Card Authoring
**Task**: PB-6 — Static grant with controller filter (30 cards)

**Completed**:
- Added 3 new EffectFilter variants: CreaturesYouControl, OtherCreaturesYouControl, OtherCreaturesYouControlWithSubtype(SubType)
- Hash discriminants 14-16 in hash.rs
- 3 match arms in layers.rs effect_applies_to_object() — resolve source controller dynamically
- 6 new tests in static_grants.rs (controller scoping, self-exclusion, subtype filtering, no-source, multi-controller)
- 10 card defs fixed: fervor, mass_hysteria, goblin_war_drums, brave_the_sands, crashing_drawbridge, dragonlord_kolaghan, ultramarines_honour_guard, markov_baron, karrthus_tyrant_of_jund, camellia_the_seedmiser
- 20 of 30 PB-6 cards remain blocked on OTHER DSL gaps (color filters, combat-state, devotion, token filters, etc.)
- Commit f722014; 2012 tests, 0 clippy warnings

**Next**:
1. Follow execution order in `docs/primitive-card-plan.md` — PB-7 through PB-21
2. **PB-7**: Count-based scaling (29 cards) — EffectAmount::PermanentCount, DevotionTo, CounterCount
3. ~23 PB-5 + ~20 PB-6 cards remain blocked on future primitives

**Hazards**: CLAUDE.md has uncommitted changes (test count update from prior session). No conflicts.

**Commit prefix used**: `W6-prim:`

## Handoff History

### 2026-03-14 — W6: PB-6 static grant with controller filter (30 cards)
- 3 EffectFilter variants + 10 card fixes + 6 tests; commit f722014; 2012 tests

### 2026-03-14 — W6: PB-5 targeted activated/triggered abilities (32 cards)
- targets field on Activated/Triggered + validation + 9 card fixes + 9 tests; commit 9ca054a; 2006 tests

### 2026-03-14 — W6: PB-4 sacrifice as activation cost (26 cards)
- SacrificeFilter + Cost::SacrificeSelf + 13 card fixes + 4 tests; commit 0344539; 1997 tests

### 2026-03-14 — W6: PB-3 shockland pay-life-or-tapped (10 cards)
- EntersTappedUnlessPayLife(u32) variant + 10 card defs fixed; commit 734cfff; 1993 tests

### 2026-03-14 — W6: PB-2 conditional ETB tapped (56 cards)
- unless_condition on AbilityDefinition::Replacement + 10 Condition variants; 56 card defs fixed; commit 091baa5; 1990 tests
