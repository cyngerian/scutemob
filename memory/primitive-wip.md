# Primitive WIP: PB-G — Bounce-all / mass zone change

batch: PB-G
title: Bounce-all / mass return to hand
cards_affected: 4
started: 2026-04-01
phase: implement
plan_file: memory/primitives/pb-plan-G.md

## Gap Summary
No `Effect::BounceAll` exists. `DestroyAll { filter }` and `ExileAll { filter }` exist as
templates. Need `BounceAll { filter }` that returns matching permanents to owners' hands.

## Cards Unblocked
1. Aetherize — return all attacking creatures to hand
2. Scourge of Fleets — ETB: return opps' creatures with toughness <= your Islands count
3. Whelming Wave — return all creatures except Kraken/Leviathan/Octopus/Serpent
4. Filter Out — return all noncreature nonland permanents to hand

## Filter Requirements
- `is_attacking: bool` on TargetFilter (for Aetherize)
- `exclude_subtypes: Vec<SubType>` on TargetFilter (for Whelming Wave)
- `toughness_lte: Option<EffectAmount>` on TargetFilter (for Scourge of Fleets)
- Standard `has_card_type` / negation filters (for Filter Out)

## Step Checklist
- [ ] 1. Engine: add `Effect::BounceAll { filter: TargetFilter }` + execute in effects/mod.rs
- [ ] 2. Engine: extend TargetFilter as needed (is_attacking, exclude_subtypes, toughness_lte)
- [ ] 3. Card defs: author Aetherize, Scourge of Fleets, Whelming Wave, Filter Out
- [ ] 4. Tests: unit tests citing CR 701.19/CR 108.3 for mass bounce
- [ ] 5. Build verification
