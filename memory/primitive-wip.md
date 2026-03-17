# Primitive WIP: PB-8 -- Cost Reduction Statics (REVIEW-ONLY)

batch: PB-8
title: Cost reduction statics
cards_affected: 10
mode: review-only
started: 2026-03-16
phase: fix
plan_file: n/a (retroactive review -- no plan needed)

## Review Scope
Engine changes and card definition fixes from the original PB-8 implementation.
Review against CR rules and oracle text. No plan file -- reviewer works from
primitive-card-plan.md batch specification and the actual code.

Cards: 10 cards with cost reduction/increase statics ("Noncreature spells cost {1} more,"
"Goblin spells cost {1} less," etc.)

Engine changes: LayerModification::ModifySpellCost { change: i32, filter: SpellCostFilter }.
Applied in casting.rs at cast time.

Key areas:
- SpellCostFilter enum and ModifySpellCost layer modification
- casting.rs cost calculation logic
- continuous_effect.rs layer application
- Oracle text accuracy for all 10 cards
- Correct cost modification semantics (increase vs decrease, type filters)

## Review
findings: 7 (HIGH: 0, MEDIUM: 3, LOW: 4)
verdict: needs-fix
review_file: memory/primitives/pb-review-8.md

Actionable findings (PB-8 scope):
- [x] Finding 1 (MEDIUM): Added `exclude_self: bool` to SpellCostModifier in card_definition.rs; updated apply_spell_cost_modifiers in casting.rs to accept spell_id: ObjectId and skip modifier when obj.id == spell_id && modifier.exclude_self; updated all 5 card defs + test file initializers with exclude_self: false.
- [x] Finding 2 (MEDIUM): Added 3 tests to spell_cost_modification.rs: test_self_cost_reduction_card_types_in_graveyard (test 9), test_self_cost_reduction_basic_land_types (test 10), test_self_cost_reduction_total_mana_value (test 11). All pass.
- [x] Finding 3 (MEDIUM): Set exclude_self: true on The Ur-Dragon's SpellCostModifier in the_ur_dragon.rs; added test_spell_cost_modifier_ur_dragon_exclude_self (test 12). 12/12 tests pass.
- Findings 4-7 (LOW): TODOs on 4 cards (attack trigger, graveyard ability, color-conditional grant, protection+cast trigger) — legitimate DSL gaps, outside PB-8 scope
