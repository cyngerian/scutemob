# Primitive WIP: PB-34 -- Mana production (filter lands, devotion, conditional)

batch: PB-34
title: Mana production (filter lands, devotion, conditional)
cards_affected: ~40
started: 2026-03-27
phase: implement
plan_file: memory/primitives/pb-plan-34.md

## Gap Reference
G-23: Filter lands (~20 cards) — Activated ability: pay hybrid mana, produce two mana from a choice set. Pattern: `AddManaChoice` with constrained color pairs
G-24: Devotion-based mana (~5 cards) — `AddManaScaled` with `EffectAmount::DevotionTo` — exists but verify wiring
G-25: Conditional mana abilities (~15 cards) — Activated abilities with conditions (e.g., "sacrifice: add {B}{B}") — many are just `Cost::Sequence` + `AddMana` wiring

## Deferred from Prior PBs
- None directly applicable to mana production

## Step Checklist
- [x] 1. Engine changes (new types/variants/dispatch) — Effect::AddManaFilterChoice added to card_definition.rs, effects/mod.rs, state/hash.rs (disc 73); try_as_tap_mana_ability extended for AddManaFilterChoice + AddManaScaled in replay_harness.rs; is_tap_mana_ability skip list updated
- [x] 2. Card definition fixes — 7 filter lands fixed: fetid_heath, rugged_prairie, twilight_mire, flooded_grove, cascade_bluffs (new ability added), sunken_ruins (new ability added), graven_cairns (new ability added). All TODO comments removed. AddManaScaled orphan bug fixed via try_as_tap_mana_ability extension.
- [ ] 3. New card definitions (if any)
- [x] 4. Unit tests — 5 tests in crates/engine/tests/mana_filter.rs: test_filter_land_produces_two_mana_fetid_heath, test_filter_land_tap_required, test_all_filter_lands_produce_correct_colors, test_add_mana_scaled_registered_as_mana_ability, test_add_mana_scaled_orphan_fix_all_cards — all pass
- [x] 5. Workspace build verification — cargo test --all (2408 passing, 0 failed), cargo clippy 0 warnings, cargo build --workspace clean, cargo fmt --check clean
