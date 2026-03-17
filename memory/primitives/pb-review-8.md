# Primitive Batch Review: PB-8 -- Cost Reduction Statics

**Date**: 2026-03-16
**Reviewer**: primitive-impl-reviewer (Opus)
**CR Rules**: 601.2f, 700.6
**Engine files reviewed**: `crates/engine/src/cards/card_definition.rs` (SpellCostModifier, SpellCostFilter, CostModifierScope, SelfCostReduction), `crates/engine/src/rules/casting.rs` (apply_spell_cost_modifiers, spell_matches_cost_filter, apply_self_cost_reduction, evaluate_self_cost_reduction, count_permanents_matching, permanent_matches_filter, reduce_generic_by)
**Card defs reviewed**: 10 (thalia_guardian_of_thraben, goblin_warchief, jhoiras_familiar, danitha_capashen_paragon, the_ur_dragon, earthquake_dragon, scion_of_draco, emrakul_the_promised_end, ghalta_primal_hunger, blasphemous_act)
**Test file reviewed**: `crates/engine/tests/spell_cost_modification.rs` (8 tests)

## Verdict: needs-fix

One MEDIUM engine finding (The Ur-Dragon "other" keyword not enforced), one MEDIUM test gap (3 SelfCostReduction variants untested), and several LOW card def TODOs that are legitimate DSL gaps outside PB-8 scope.

## Engine Change Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | **MEDIUM** | `casting.rs:6140-6198` | **No "other" / self-exclusion mechanism for SpellCostModifier.** The Ur-Dragon's eminence says "other Dragon spells you cast cost {1} less" but the engine has no way to skip the modifier when the spell being cast IS the modifier's source. See Finding 1. **Fix:** Add `exclude_self: bool` to `SpellCostModifier`, pass the spell's `ObjectId` to `apply_spell_cost_modifiers`, and skip when `obj.id == spell_id && modifier.exclude_self`. Set `exclude_self: true` on The Ur-Dragon's card def. |
| 2 | **MEDIUM** | `tests/spell_cost_modification.rs` | **Missing tests for 3 of 5 SelfCostReduction variants.** `CardTypesInGraveyard` (Emrakul), `BasicLandTypes` (Scion of Draco), and `TotalManaValue` (Earthquake Dragon) have zero test coverage. **Fix:** Add tests 9-11 covering each variant with positive and negative cases. |

## Card Definition Findings

| # | Severity | Card | Description |
|---|----------|------|-------------|
| 3 | **MEDIUM** | `the_ur_dragon.rs` | **Missing `exclude_self` on cost modifier.** Oracle says "other Dragon spells" but modifier applies to all Dragon spells including itself. **Fix:** After engine adds `exclude_self`, set it to `true` on The Ur-Dragon's `SpellCostModifier`. |
| 4 | LOW | `the_ur_dragon.rs:26-27` | **TODO: attack trigger.** "Whenever one or more Dragons you control attack, draw that many cards, then you may put a permanent card from your hand onto the battlefield." Complex trigger outside PB-8 scope. |
| 5 | LOW | `earthquake_dragon.rs:22` | **TODO: graveyard activated ability.** "{2}{G}, Sacrifice a land: Return this card from your graveyard to your hand." DSL gap: compound costs + graveyard return. Outside PB-8 scope. |
| 6 | LOW | `scion_of_draco.rs:23` | **TODO: color-conditional keyword grant.** Color-conditional static grant not supported in DSL. Outside PB-8 scope. |
| 7 | LOW | `emrakul_the_promised_end.rs:30-31` | **TODO: protection from instants + cast trigger.** Two DSL gaps. Outside PB-8 scope. |

### Finding Details

#### Finding 1: No self-exclusion mechanism for SpellCostModifier

**Severity**: MEDIUM
**File**: `crates/engine/src/rules/casting.rs:6140-6198` + `crates/engine/src/cards/card_definition.rs:2206-2216`
**CR Rule**: N/A (card-specific oracle text: "other Dragon spells you cast cost {1} less to cast")
**Oracle**: The Ur-Dragon: "Eminence -- As long as The Ur-Dragon is in the command zone or on the battlefield, **other** Dragon spells you cast cost {1} less to cast."
**Issue**: `apply_spell_cost_modifiers` iterates all objects and checks each modifier's filter and scope. When The Ur-Dragon is in the command zone and the player casts The Ur-Dragon itself (which is a Dragon spell), the modifier incorrectly reduces its own cost by {1}. The function receives `spell_chars: &Characteristics` but not the spell's `ObjectId`, so it cannot distinguish between the source permanent and the spell being cast. The `SpellCostModifier` struct has no field to express "other" semantics.
**Fix**: (1) Add `pub exclude_self: bool` field (with `#[serde(default)]`) to `SpellCostModifier` in `card_definition.rs`. (2) Add a `spell_id: ObjectId` parameter to `apply_spell_cost_modifiers` in `casting.rs`. Pass it from `handle_cast_spell` (the `card` parameter). (3) In the loop body, after the scope check, add: `if modifier.exclude_self && obj.id == spell_id { continue; }`. (4) Update The Ur-Dragon card def to set `exclude_self: true`. (5) Add `exclude_self: false` to the `Default` impl or use `#[serde(default)]`.

#### Finding 2: Missing tests for 3 SelfCostReduction variants

**Severity**: MEDIUM
**File**: `crates/engine/tests/spell_cost_modification.rs`
**Issue**: Tests 6-8 cover `PerPermanent`, `TotalPowerOfCreatures`, and `Historic` filter. But `CardTypesInGraveyard` (Emrakul-style), `BasicLandTypes` (Scion-style Domain), and `TotalManaValue` (Earthquake Dragon-style) have no test coverage. These are non-trivial evaluation functions with distinct logic paths.
**Fix**: Add 3 tests:
- Test 9: `test_self_cost_reduction_card_types_in_graveyard` -- put cards of 5 different types in the caster's graveyard, verify a {13} spell costs {8}.
- Test 10: `test_self_cost_reduction_basic_land_types` -- give caster lands with 3 basic land types, verify a {12} spell with `per: 2` costs {6}.
- Test 11: `test_self_cost_reduction_total_mana_value` -- give caster Dragons with total MV=8 on the battlefield, verify a {14}{G} spell costs {6}{G}.

#### Finding 3: The Ur-Dragon card def missing exclude_self

**Severity**: MEDIUM
**File**: `crates/engine/src/cards/defs/the_ur_dragon.rs:29-34`
**Oracle**: "other Dragon spells you cast cost {1} less to cast"
**Issue**: The `SpellCostModifier` does not encode the "other" keyword. After Finding 1's engine fix is applied, this card def must set `exclude_self: true`.
**Fix**: After the engine change from Finding 1, update the card def's `SpellCostModifier` to include `exclude_self: true`.

## CR Coverage Check

| CR Rule | Implemented? | Tested? | Notes |
|---------|-------------|---------|-------|
| 601.2f (cost increases) | Yes | Yes | test 1 (Thalia increase) |
| 601.2f (cost reductions) | Yes | Yes | tests 2-4 (Warchief tribal, stacking, floor at 0) |
| 601.2f (generic floor at 0) | Yes | Yes | test 4 |
| 601.2f (self-cost-reduction) | Yes | Partial | tests 6-7 cover PerPermanent + TotalPower; missing 3 variants |
| 601.2f (order: after tax+kicker, before affinity) | Yes | Implicit | casting.rs line ordering is correct |
| 700.6 (Historic definition) | Yes | Yes | test 8 |

## Card Def Summary

| Card | Oracle Match | TODOs Remaining | Game State Correct | Notes |
|------|-------------|-----------------|-------------------|-------|
| thalia_guardian_of_thraben | Yes | 0 | Yes | Perfect |
| goblin_warchief | Yes | 0 | Yes | Haste grant uses self+other pattern (correct) |
| jhoiras_familiar | Yes | 0 | Yes | Perfect |
| danitha_capashen_paragon | Yes | 0 | Yes | Perfect |
| the_ur_dragon | Partial | 1 (attack trigger) | **No** | Missing `exclude_self` -- reduces own cost incorrectly (Finding 3) |
| earthquake_dragon | Yes | 1 (graveyard ability) | Yes | Cost reduction correct; TODO is separate ability |
| scion_of_draco | Yes | 1 (color-conditional grant) | Yes | Cost reduction correct; TODO is separate ability |
| emrakul_the_promised_end | Yes | 2 (protection, cast trigger) | Yes | Cost reduction correct; TODOs are separate abilities |
| ghalta_primal_hunger | Yes | 0 | Yes | Perfect |
| blasphemous_act | Yes | 0 | Yes | Perfect |
