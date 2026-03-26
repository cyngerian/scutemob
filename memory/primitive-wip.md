# Primitive WIP: PB-32 -- Static/effect primitives (lands, prevention, control, animation)

batch: PB-32
title: Static/effect primitives (additional lands, prevention, control change, land animation)
cards_affected: ~39
started: 2026-03-26
phase: closed
plan_file: memory/primitives/pb-plan-32.md

## Gap Reference
G-18: Additional land plays (~10 cards) — Static: "you may play an additional land on each of your turns" — increment `lands_per_turn`
G-19: Prevention effects / combat damage prevention (~11 cards) — Wire `ApplyContinuousEffect` with damage prevention shield
G-20: Control change effects (~6 cards) — `Effect::GainControl { target, duration }`, `Effect::ExchangeControl { target_a, target_b }`
G-21: Land animation (~12 cards) — `Effect::AnimateLand { target, power, toughness, types, duration }` — "becomes a N/N creature"

## Deferred from Prior PBs
- PB-31: Spike Weaver ability 2 (PreventAllCombatDamage — G-19, now in scope)
- PB-37 accumulator: various complex activated abilities (not in scope here)

## Step Checklist
- [x] 1. Engine changes (new types/variants/dispatch)
  - Effect variants: AdditionalLandPlay, PreventAllCombatDamage, PreventCombatDamageFromOrTo, GainControl, ExchangeControl (card_definition.rs)
  - AbilityDefinition variant: AdditionalLandPlays { count } (card_definition.rs)
  - GameState fields: additional_land_play_sources, prevent_all_combat_damage, combat_damage_prevented_from, combat_damage_prevented_to (state/mod.rs)
  - AdditionalLandPlaySource struct (state/stubs.rs)
  - Builder defaults (state/builder.rs)
  - Effect dispatch arms (effects/mod.rs)
  - AdditionalLandPlays registration in register_static_continuous_effects (rules/replacement.rs)
  - AdditionalLandPlays at ClassLevel resolution (rules/resolution.rs)
  - Turn-start land play application + stale source cleanup + prevention flag reset (rules/turn_actions.rs)
  - Combat damage prevention enforcement (rules/combat.rs)
  - EffectFilter::AttachedPermanent added (state/continuous_effect.rs + rules/layers.rs)
  - Hash updates for all new variants/fields (state/hash.rs)
- [x] 2. Card definition fixes
  - G-18: explore.rs, urban_evolution.rs, aesi_tyrant_of_gyre_strait.rs, mina_and_denn_wildborn.rs, wayward_swordtooth.rs, druid_class.rs (6 cards)
  - G-19: spike_weaver.rs, maze_of_ith.rs, kor_haven.rs (3 cards definitively fixed; 6 left as TODO per plan)
  - G-20: zealous_conscripts.rs, connive.rs, dragonlord_silumgar.rs, sarkhan_vol.rs, olivia_voldaren.rs, thieving_skydiver.rs (6 cards)
  - G-21: den_of_the_bugbear.rs, creeping_tar_pit.rs, destiny_spinner.rs, tatyova_steward_of_tides.rs, wrenn_and_realmbreaker.rs, imprisoned_in_the_moon.rs, oko_thief_of_crowns.rs (7 cards)
- [x] 3. New card definitions (if any) — none needed
- [x] 4. Unit tests — 13 tests in crates/engine/tests/primitive_pb32.rs
- [x] 5. Workspace build verification — 2396 tests, 0 failures, 0 clippy warnings, clean workspace build

## Review
findings: 9 (HIGH: 0, MEDIUM: 2, LOW: 7)
verdict: needs-fix
review_file: memory/primitives/pb-review-32.md
