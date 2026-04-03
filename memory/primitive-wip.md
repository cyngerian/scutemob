# Primitive WIP: PB-D — Chosen Creature Type

batch: PB-D
title: Chosen creature type
cards_affected: 12
started: 2026-04-02
completed: 2026-04-02
phase: closed

## Cards (from ops plan + grep)
Cards with TODOs referencing chosen creature type:
1. Morophon the Boundless — choose type ETB, cost reduction, +1/+1 anthem
2. Vanquisher's Banner — choose type ETB, +1/+1 anthem, cast-trigger draw
3. Patchwork Banner — choose type ETB, +1/+1 anthem, mana
4. Herald's Horn — choose type ETB, cost reduction, upkeep look-at-top
5. Kindred Dominance — choose type on cast, destroy non-chosen
6. Cavern of Souls — choose type ETB, mana restriction, can't be countered (deferred — CounterRestriction not implemented)
7. Unclaimed Territory — choose type ETB, mana restriction (already partial)
8. Secluded Courtyard — choose type ETB, mana restriction (already partial)
9. Urza's Incubator — choose type ETB, cost reduction (already partial)
10. Three Tree City — choose type ETB, count-based mana
11. Etchings of the Chosen — chosen type anthem + sac protection
12. Pact of the Serpent — chosen type draw

## Existing Infrastructure
- `chosen_creature_type: Option<SubType>` on GameObject (already exists)
- `ReplacementEffect::ChooseCreatureType` (ETB replacement, already exists)
- `SpellCostFilter::HasChosenCreatureSubtype` (cost reduction, already exists)
- `ManaRestriction::ChosenCreatureTypeSpells` (mana restriction, already exists)

## Step Checklist
- [x] 1. Engine changes (new types/variants/dispatch)
  - `EffectFilter::CreaturesYouControlOfChosenType` + `OtherCreaturesYouControlOfChosenType` (continuous_effect.rs, layers.rs, hash.rs)
  - `TriggerCondition::WheneverYouCastSpell.chosen_subtype_filter: bool` (card_definition.rs, hash.rs, abilities.rs)
  - `TargetFilter.has_chosen_subtype: bool` + `exclude_chosen_subtype: bool` (card_definition.rs, effects/mod.rs via check_chosen_subtype_filter helper)
  - `SpellCostModifier.colored_mana_reduction: Option<ManaCost>` (card_definition.rs, casting.rs)
  - `EffectAmount::ChosenTypeCreatureCount { controller }` (card_definition.rs, effects/mod.rs, hash.rs)
  - `Effect::AddManaOfAnyColorAmount { player, amount }` (card_definition.rs, effects/mod.rs, hash.rs)
  - `Condition::TopCardIsCreatureOfChosenType` (card_definition.rs, effects/mod.rs, hash.rs)
  - `SacrificeFilter::CreatureOfChosenType` (game_object.rs, abilities.rs, hash.rs, replay_harness.rs)
  - `EffectContext.chosen_creature_type: Option<SubType>` (effects/mod.rs)
  - `Effect::ChooseCreatureType` extended to also set `ctx.chosen_creature_type`
  - Updated 19 card defs with `colored_mana_reduction: None`
  - Updated 21 card defs with `chosen_subtype_filter: false`
- [x] 2. Card definition fixes (8 cards done, Cavern of Souls deferred)
  - morophon_the_boundless.rs — anthem + colored_mana_reduction
  - vanquishers_banner.rs — ChooseCreatureType + anthem + chosen_subtype_filter cast trigger
  - patchwork_banner.rs — ChooseCreatureType + anthem
  - heralds_horn.rs — ChooseCreatureType + cost reduction + TopCardIsCreatureOfChosenType
  - kindred_dominance.rs — Sequence(ChooseCreatureType, DestroyAll(exclude_chosen))
  - three_tree_city.rs — AddManaOfAnyColorAmount + ChosenTypeCreatureCount
  - etchings_of_the_chosen.rs — anthem + CreatureOfChosenType sac cost
  - pact_of_the_serpent.rs — ChosenTypeCreatureCount draw + life loss
- [x] 3. Unit tests (12 tests in crates/engine/tests/chosen_creature_type.rs)
  - test_chosen_creature_type_etb_sets_type
  - test_chosen_type_anthem_basic
  - test_chosen_type_anthem_other
  - test_chosen_type_cost_reduction_colored
  - test_chosen_type_cost_reduction_generic
  - test_chosen_type_spell_level_choice_propagates
  - test_chosen_type_cast_trigger_definition
  - test_chosen_type_destroy_all_except
  - test_top_card_creature_of_chosen_type_true
  - test_top_card_creature_of_chosen_type_false
  - test_pact_of_the_serpent_definition
  - test_chosen_type_permanent_count_via_effect
- [x] 4. Workspace build verification — 2474 tests pass, 0 clippy warnings, build clean

## Deferred
- Cavern of Souls "can't be countered" — requires `CounterRestriction` primitive (separate PB)
