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
| W6: Primitive + Card Authoring | PB-9: Hybrid mana & X costs | ACTIVE | 2026-03-14 | **TOP PRIORITY**. PB-0 through PB-8 done. Plan: `docs/primitive-card-plan.md` |

**Status values**: `available` (free to claim), `ACTIVE` (session working on it),
`paused` (partially done, session ended mid-task), `not-started` (blocked/deferred),
`RETIRED` (replaced by another workstream)

## Last Handoff

**Date**: 2026-03-14
**Workstream**: W6: Primitive + Card Authoring
**Task**: PB-8 — Cost reduction statics (10 cards)

**Completed**:
- 2 new CardDefinition fields: `spell_cost_modifiers: Vec<SpellCostModifier>`, `self_cost_reduction: Option<SelfCostReduction>`
- SpellCostModifier struct + SpellCostFilter enum (NonCreature, HasSubtype, Historic, HasCardType, AuraOrEquipment)
- CostModifierScope enum (AllPlayers, Controller) + eminence flag for command zone
- SelfCostReduction enum (PerPermanent, TotalPowerOfCreatures, CardTypesInGraveyard, BasicLandTypes, TotalManaValue)
- apply_spell_cost_modifiers() + apply_self_cost_reduction() in casting.rs
- Pipeline: tax → kicker → cost modifiers → self-reduction → affinity → undaunted → convoke
- 8 new tests in spell_cost_modification.rs
- 5 permanent-modifier cards fixed: thalia_guardian_of_thraben, goblin_warchief (+haste grant), jhoiras_familiar, danitha_capashen_paragon, the_ur_dragon
- 5 self-reduction cards fixed: blasphemous_act, ghalta_primal_hunger, emrakul_the_promised_end, scion_of_draco, earthquake_dragon
- 129+ card defs + 15+ test files: added new fields to explicit struct constructions
- Commit c1edb48; 2028 tests, 0 clippy warnings

**Next**:
1. Follow execution order in `docs/primitive-card-plan.md` — PB-9 through PB-21
2. **PB-9**: Hybrid mana & X costs (7 cards) — hybrid: Vec<HybridMana> and x_count on ManaCost
3. Many PB-7 cards remain blocked on deeper gaps (dynamic LayerModification for CDA */* creatures)
4. ~23 PB-5 + ~20 PB-6 cards remain blocked on future primitives

**Hazards**: CLAUDE.md has uncommitted changes (test count update). No conflicts.

**Commit prefix used**: `W6-prim:`

## Handoff History

### 2026-03-14 — W6: PB-8 cost reduction statics (10 cards)
- SpellCostModifier + SelfCostReduction + 10 card fixes + 8 tests; commit c1edb48; 2028 tests

### 2026-03-14 — W6: PB-7 count-based scaling (29 cards)
- 3 EffectAmount variants + AddManaScaled + 5 card fixes + 8 tests; commit 399c8da; 2020 tests

### 2026-03-14 — W6: PB-6 static grant with controller filter (30 cards)
- 3 EffectFilter variants + 10 card fixes + 6 tests; commit f722014; 2012 tests

### 2026-03-14 — W6: PB-5 targeted activated/triggered abilities (32 cards)
- targets field on Activated/Triggered + validation + 9 card fixes + 9 tests; commit 9ca054a; 2006 tests

### 2026-03-14 — W6: PB-4 sacrifice as activation cost (26 cards)
- SacrificeFilter + Cost::SacrificeSelf + 13 card fixes + 4 tests; commit 0344539; 1997 tests
