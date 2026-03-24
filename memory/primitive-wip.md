# Primitive WIP: PB-25 -- Continuous Effect Grants

batch: PB-25
title: Continuous effect grants ("creatures you control get/have")
cards_affected: ~98
started: 2026-03-23
phase: implement
plan_file: memory/primitives/pb-plan-25.md

## Gap Reference
G-3 from `docs/dsl-gap-closure-plan.md`: Wire `ApplyContinuousEffect` for card defs —
add `EffectFilter::CreaturesYouControl`, `CreaturesOpponentsControl`, filtered grant
patterns. "Creatures you control get/have X" continuous effects.

## Deferred from Prior PBs
- indomitable_archangel.rs: condition expressible (PB-24), blocked on EffectFilter (PB-25)

## Step Checklist
- [x] 1. Engine changes (new types/variants/dispatch) — 11 new EffectFilter variants in continuous_effect.rs, layers.rs match arms, hash.rs discriminants 17-27; workspace builds clean
- [x] 2. Card definition fixes — 28 cards fixed: elesh_norn, massacre_wurm, archetype_of_imagination, archetype_of_endurance, blade_historian, ohran_frostfang, stensia_masquerade, crossway_troublemakers, elderfang_venom, blight_mound, ezuri_renegade_leader, battle_cry_goblin, lathliss_dragon_queen, elvish_warmaster, rith_liberated_primeval, rising_of_the_day, bloodmark_mentor, indomitable_archangel, mikaeus_the_unhallowed, return_of_the_wildspeaker, bladewing_the_risen, silver_fur_master, goblin_surprise, you_see_a_pair_of_goblins, goblin_war_party, goro_goro_disciple_of_ryusei, purphoros_god_of_the_forge, scourge_of_valkas
- [x] 3. New card definitions — none needed (all cards had existing files)
- [x] 4. Unit tests — 13 new tests in static_grants.rs (all 19 tests pass): test_creatures_opponents_control_debuff, test_creatures_opponents_control_multiplayer, test_attacking_creatures_you_control_grants_keyword, test_attacking_creatures_filter_outside_combat, test_creatures_you_control_with_subtype_includes_self, test_creatures_you_control_with_supertype_legendary, test_creatures_you_control_with_color_red, test_artifacts_you_control_grants_shroud, test_other_creatures_excluding_subtype_non_human, test_creatures_excluding_subtype_spell_effect, test_attacking_creatures_with_subtype, test_all_creatures_with_subtype_no_controller, test_other_creatures_with_subtypes_or
- [x] 5. Workspace build verification — 2287 tests pass, 0 failures, clippy clean, workspace builds, fmt clean
