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
| W6: Primitive + Card Authoring | PB-11: Mana spending restrictions + ETB choice | ACTIVE | 2026-03-15 | **TOP PRIORITY**. PB-0 through PB-10 done. Plan: `docs/primitive-card-plan.md` |

**Status values**: `available` (free to claim), `ACTIVE` (session working on it),
`paused` (partially done, session ended mid-task), `not-started` (blocked/deferred),
`RETIRED` (replaced by another workstream)

## Last Handoff

**Date**: 2026-03-14
**Workstream**: W6: Primitive + Card Authoring
**Task**: PB-10 — Return from zone effects (graveyard targeting)

**Completed**:
- Added 2 new TargetRequirement variants: TargetCardInYourGraveyard, TargetCardInGraveyard
- Added has_subtypes Vec<SubType> to TargetFilter (OR semantics for "Vampire or Wizard")
- Updated casting.rs validation (2 new arms, no hexproof/shroud for GY cards)
- Updated hash.rs (3 entries) and matches_filter in effects/mod.rs
- Fixed 10 card defs: Bloodline Necromancer, Bladewing the Risen, Buried Ruin, Den Protector, Emeria the Sky Ruin, Grim Harvest, Hall of Heliod's Generosity, Nullpriest of Oblivion, Reanimate, Teneb the Harvester
- 10 new tests in graveyard_targeting.rs
- Commit 0b6b24d; 2054 tests, 0 clippy warnings

**Next**:
1. **PB-11**: Mana spending restrictions + ETB choice (13 cards, 2 sessions)
2. Follow execution order in `docs/primitive-card-plan.md` — PB-11 through PB-21
3. Many PB-7 cards remain blocked on deeper gaps (dynamic LayerModification for CDA */* creatures)
4. ~23 PB-5 + ~20 PB-6 cards remain blocked on future primitives

**Hazards**: None. Clean working tree (CLAUDE.md has pre-existing uncommitted changes from prior sessions).

**Commit prefix used**: `W6-prim:`

## Handoff History

### 2026-03-14 — W6: PB-10 graveyard targeting (10 cards)
- 2 TargetRequirement variants + has_subtypes filter + 10 card def fixes + 10 tests; commit 0b6b24d; 2054 tests

### 2026-03-14 — W6: PB-9.5 architecture cleanup
- check_and_flush_triggers() helper extracted (26 copies → 1), 5 test CardDefinition defaults fixed; commits e7b13a1 + 2c8f502; 2044 tests

### 2026-03-14 — W6: PB-9 hybrid/phyrexian/X mana (19 cards)
- 3 enums + 3 ManaCost fields + flatten helper + 12 card fixes + 16 tests; commit c43a1fa; 2044 tests

### 2026-03-14 — W6: PB-8 cost reduction statics (10 cards)
- SpellCostModifier + SelfCostReduction + 10 card fixes + 8 tests; commit c1edb48; 2028 tests

### 2026-03-14 — W6: PB-7 count-based scaling (29 cards)
- 3 EffectAmount variants + AddManaScaled + 5 card fixes + 8 tests; commit 399c8da; 2020 tests
