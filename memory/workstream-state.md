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
| W6: Primitive + Card Authoring | PB-12: Complex replacement effects | ACTIVE | 2026-03-15 | **TOP PRIORITY**. PB-0 through PB-11 done. Plan: `docs/primitive-card-plan.md` |

**Status values**: `available` (free to claim), `ACTIVE` (session working on it),
`paused` (partially done, session ended mid-task), `not-started` (blocked/deferred),
`RETIRED` (replaced by another workstream)

## Last Handoff

**Date**: 2026-03-15
**Workstream**: W6: Primitive + Card Authoring
**Task**: PB-11 — Mana spending restrictions + ETB creature type choice

**Completed**:
- Added ManaRestriction enum (5 variants: CreatureSpellsOnly, SubtypeOnly, SubtypeOrSubtype, ChosenTypeCreaturesOnly, ChosenTypeSpellsOnly)
- Added Effect::AddManaRestricted / AddManaAnyColorRestricted for restricted mana production
- Added Effect::ChooseCreatureType for direct type choice
- Added ReplacementModification::ChooseCreatureType for "As this enters, choose a creature type"
- Added chosen_creature_type: Option<SubType> on GameObject
- Added RestrictedMana + SpellContext on ManaPool with restricted mana tracking
- Added can_pay_cost_with_context / pay_cost_with_context in casting.rs
- Hash support for all new types
- Fixed 10 card defs: Cavern of Souls, Secluded Courtyard, Unclaimed Territory, Haven of the Spirit Dragon, Maelstrom of the Spirit Dragon, Gnarlroot Trapper, Voldaren Estate, The Seedcore, Three Tree City, Etchings of the Chosen
- 11 new tests in mana_restriction.rs
- Commit 382ae7d; 2065 tests, 0 clippy warnings

**Next**:
1. **PB-12**: Complex replacement effects (11 cards, 2-3 sessions)
2. Follow execution order in `docs/primitive-card-plan.md` — PB-12 through PB-21
3. Many PB-7 cards remain blocked on deeper gaps (dynamic LayerModification for CDA */* creatures)
4. ~23 PB-5 + ~20 PB-6 cards remain blocked on future primitives

**Hazards**: CLAUDE.md + several test files have pre-existing uncommitted changes from prior sessions. Not related to PB-11.

**Commit prefix used**: `W6-prim:`

## Handoff History

### 2026-03-15 — W6: PB-11 mana restrictions + ETB choice (10 cards)
- ManaRestriction enum + restricted mana pool + chosen_creature_type + 10 card fixes + 11 tests; commit 382ae7d; 2065 tests

### 2026-03-14 — W6: PB-10 graveyard targeting (10 cards)
- 2 TargetRequirement variants + has_subtypes filter + 10 card def fixes + 10 tests; commit 0b6b24d; 2054 tests

### 2026-03-14 — W6: PB-9.5 architecture cleanup
- check_and_flush_triggers() helper extracted (26 copies → 1), 5 test CardDefinition defaults fixed; commits e7b13a1 + 2c8f502; 2044 tests

### 2026-03-14 — W6: PB-9 hybrid/phyrexian/X mana (19 cards)
- 3 enums + 3 ManaCost fields + flatten helper + 12 card fixes + 16 tests; commit c43a1fa; 2044 tests

### 2026-03-14 — W6: PB-8 cost reduction statics (10 cards)
- SpellCostModifier + SelfCostReduction + 10 card fixes + 8 tests; commit c1edb48; 2028 tests
