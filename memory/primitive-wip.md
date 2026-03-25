# Primitive WIP: PB-28 -- CDA / count-based P/T

batch: PB-28
title: CDA / count-based P/T
cards_affected: ~32
started: 2026-03-25
phase: closed
plan_file: memory/primitives/pb-plan-28.md

## Gap Reference
G-6 from `docs/dsl-gap-closure-plan.md`:
- G-6: CDA / count-based P/T (~32 cards) — CharacteristicDefiningAbility mechanism: `PowerToughness::CDA(EffectAmount)` evaluated in Layer 7a

## Deferred from Prior PBs
- PB-7 already has count-based scaling (EffectAmount::CountCreaturesYouControl etc.)
- PB-27 added EffectAmount::XValue, Effect::Repeat, Condition::XValueAtLeast
- Layer system already handles 7a (CDA) vs 7b (other P/T changes)

## Step Checklist
- [x] 1. Engine changes (new types/variants/dispatch)
  - Added `EffectAmount::Sum(Box<EffectAmount>, Box<EffectAmount>)` to card_definition.rs
  - Added `LayerModification::SetPtDynamic { power: Box<EffectAmount>, toughness: Box<EffectAmount> }` to continuous_effect.rs
  - Added `AbilityDefinition::CdaPowerToughness { power, toughness }` to card_definition.rs (discriminant 70)
  - Added `resolve_cda_amount()`, `resolve_cda_player_target()`, `resolve_cda_zone_target()` to layers.rs
  - Updated `apply_layer_modification()` to take `object_id` parameter + `SetPtDynamic` arm
  - Added `CdaPowerToughness` registration in `register_static_continuous_effects()` (replacement.rs)
  - Added hash arms for Sum (disc 11), SetPtDynamic (disc 22), CdaPowerToughness (disc 70) in hash.rs
  - Added `EffectAmount::Sum` arm in `resolve_amount()` (effects/mod.rs)
  - Fixed infinite recursion in PermanentCount evaluation (uses base chars, not calculate_characteristics)
- [x] 2. Card definition fixes
  - battle_squadron.rs: Added CdaPowerToughness (creatures you control)
  - molimo_maro_sorcerer.rs: Added CdaPowerToughness (lands you control)
  - greensleeves_maro_sorcerer.rs: Added CdaPowerToughness (lands you control)
  - cultivator_colossus.rs: Added CdaPowerToughness (lands you control); ETB loop still TODO
  - reckless_one.rs: Added CdaPowerToughness (Goblins on battlefield, EachPlayer)
  - psychosis_crawler.rs: Added CdaPowerToughness (cards in hand)
  - abomination_of_llanowar.rs: Added CdaPowerToughness with Sum (Elves + graveyard Elves)
  - jagged_scar_archers.rs: Added CdaPowerToughness (Elves you control); activated ability still TODO
  - adeline_resplendent_cathar.rs: Added CdaPowerToughness (power=creatures, toughness=Fixed(4))
  - nighthawk_scavenger.rs: DEFERRED — needs DistinctCardTypesInGraveyards variant
- [x] 3. New card definitions (if any) — none needed
- [x] 4. Unit tests — 9 tests in crates/engine/tests/cda_tests.rs (all passing)
- [x] 5. Workspace build verification — cargo build --workspace OK, clippy clean, fmt OK, 9/9 CDA tests pass

## Review
findings: 3 (HIGH: 0, MEDIUM: 2, LOW: 1)
verdict: needs-fix
review_file: memory/primitives/pb-review-28.md

## Fix Phase Results (2026-03-25)
- MEDIUM-1 (PermanentCount base-chars): documented design tradeoff, no code fix needed
- MEDIUM-2 (abomination_of_llanowar graveyard filter): FIXED — removed `has_card_type: Some(CardType::Creature)` from both graveyard CardCount filters (power + toughness). Graveyard filter now matches all Elf cards (any type) per oracle text.
- LOW-1 (CR 604.3 non-battlefield CDA): deferred to post-alpha, no code fix needed
- All tests pass; clippy clean
- phase: DONE
