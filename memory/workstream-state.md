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
| W6: Primitive + Card Authoring | PB-13: Specialized mechanics | ACTIVE | 2026-03-15 | **TOP PRIORITY**. PB-0 through PB-12 done. Plan: `docs/primitive-card-plan.md` |

**Status values**: `available` (free to claim), `ACTIVE` (session working on it),
`paused` (partially done, session ended mid-task), `not-started` (blocked/deferred),
`RETIRED` (replaced by another workstream)

## Last Handoff

**Date**: 2026-03-15
**Workstream**: W6: Primitive + Card Authoring
**Task**: PB-12 — Complex replacement effects

**Completed**:
- 7 new ReplacementTrigger variants: WouldPlaceCounters, WouldCreateTokens, WouldSearchLibrary, WouldLoseLife, WouldProliferate + DamageTargetFilter::FromControllerSources
- 8 new ReplacementModification variants: DoubleCounters, HalveCounters, AddExtraCounter, DoubleTokens, RestrictSearchTopN, DoubleDamage, DoubleLifeLoss, DoubleProliferate
- TriggerDoublerFilter::CreatureDeath for Teysa Karlov
- 6 helper functions in replacement.rs wired into effects/mod.rs (AddCounter, Proliferate, CreateToken, SearchLibrary, DealDamage)
- PlayerFilter/ObjectFilter binding at registration via PlayerId(0) placeholder pattern
- PlayerFilter + DamageTargetFilter exported from helpers.rs
- 8 card def fixes: Vorinclex, Pir, Adrix and Nev, Bloodletter, Aven Mindcensor, Twinflame Tyrant, Tekuthal, Teysa Karlov
- 14 new tests across 2 test files (counter_replacement.rs, token_damage_search_replacement.rs)
- Commit 20d8981; 2079 tests, 0 clippy warnings

**Next**:
1. **PB-13**: Specialized mechanics (19 cards, 3-4 sessions) — land animation, channel, ascend, equipment auto-attach, dredge, buyback, player hexproof, coin flip, timing restriction, clone, monarch, flicker, adventure, living weapon
2. 3 PB-12 cards remain with deeper DSL gaps: Neriv (entered-this-turn condition on damage doubling), Lightning Army of One (temporal replacement duration), Mossborn Hydra (landfall trigger)
3. Follow execution order in `docs/primitive-card-plan.md` — PB-13 through PB-21

**Hazards**: None — clean working tree after commit.

**Commit prefix used**: `W6-prim:`

## Handoff History

### 2026-03-15 — W6: PB-12 complex replacement effects (8 cards)
- 7 triggers + 8 modifications + 1 TriggerDoublerFilter + 6 helpers + 8 card fixes + 14 tests; commit 20d8981; 2079 tests

### 2026-03-15 — W6: PB-11 mana restrictions + ETB choice (10 cards)
- ManaRestriction enum + restricted mana pool + chosen_creature_type + 10 card fixes + 11 tests; commit 382ae7d; 2065 tests

### 2026-03-14 — W6: PB-10 graveyard targeting (10 cards)
- 2 TargetRequirement variants + has_subtypes filter + 10 card def fixes + 10 tests; commit 0b6b24d; 2054 tests

### 2026-03-14 — W6: PB-9.5 architecture cleanup
- check_and_flush_triggers() helper extracted (26 copies → 1), 5 test CardDefinition defaults fixed; commits e7b13a1 + 2c8f502; 2044 tests

### 2026-03-14 — W6: PB-9 hybrid/phyrexian/X mana (19 cards)
- 3 enums + 3 ManaCost fields + flatten helper + 12 card fixes + 16 tests; commit c43a1fa; 2044 tests
