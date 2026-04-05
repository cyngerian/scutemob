# Primitive WIP: PB-F — Damage Multiplier

batch: PB-F
title: Damage multiplier
cards_affected: 3
started: 2026-04-05
phase: closed

## Review
findings: 0 (HIGH: 0, MEDIUM: 0, LOW: 0)
verdict: clean
review_file: memory/primitives/pb-review-F.md
plan_file: memory/primitives/pb-plan-F.md

## Cards
1. Lightning, Army of One — Stagger trigger: damage doubling for a player until next turn (conditional, duration-based)
2. Neriv, Heart of the Storm — Static: creatures that entered this turn deal double damage (conditional)
3. Fiery Emancipation — Static: sources you control deal triple damage (needs TripleDamage)

## Existing Infrastructure
- `ReplacementModification::DoubleDamage` variant exists
- `apply_damage_doubling()` in replacement.rs checks registered DamageWouldBeDealt replacements
- Already used by Twinflame Tyrant and Angrath's Marauders
- Called from combat.rs, effects/mod.rs (DealDamage dispatch)
- Missing: TripleDamage variant, conditional filters (entered-this-turn, target-player-specific)

## Deferred from Prior PBs
- none directly relevant

## Step Checklist
- [x] 1. Engine changes (TripleDamage variant, conditional damage replacement filters)
  - Added `ReplacementModification::TripleDamage` in state/replacement_effect.rs
  - Added `DamageTargetFilter::ToPlayerOrTheirPermanents(PlayerId)` and `FromControllerCreaturesEnteredThisTurn(PlayerId)`
  - Added `entered_turn: Option<u32>` to GameObject (state/game_object.rs, state/mod.rs, state/builder.rs)
  - Extended `apply_damage_doubling()` in rules/replacement.rs for TripleDamage + new filters
  - Added `Effect::RegisterReplacementEffect { trigger, modification, duration }` to card_definition.rs + effects/mod.rs handler
  - Extended `expire_until_next_turn_effects` in rules/layers.rs to expire replacement_effects
  - Extended `check_triggers` in rules/abilities.rs for WhenDealsCombatDamageToPlayer CardDef triggers
  - Added hash arms in state/hash.rs (TripleDamage disc 18, new DamageTargetFilter disc 6-7, RegisterReplacementEffect disc 77, entered_turn field)
- [x] 2. Card definition fixes (Lightning, Neriv)
  - lightning_army_of_one.rs: added Stagger triggered ability using RegisterReplacementEffect + ToPlayerOrTheirPermanents
  - neriv_heart_of_the_storm.rs: added static replacement with FromControllerCreaturesEnteredThisTurn filter
- [x] 3. New card definitions (Fiery Emancipation)
  - Created crates/engine/src/cards/defs/fiery_emancipation.rs with TripleDamage + FromControllerSources
- [x] 4. Unit tests
  - Created crates/engine/tests/damage_multiplier.rs — 10 tests all passing
- [x] 5. Workspace build verification
  - All tests pass (0 failures), cargo clippy clean, cargo build --workspace clean, cargo fmt clean
