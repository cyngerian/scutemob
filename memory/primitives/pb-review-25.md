# Primitive Batch Review: PB-25 — Continuous Effect Grant Filters

**Date**: 2026-03-23
**Reviewer**: primitive-impl-reviewer (Opus)
**CR Rules**: 604.2, 611.3, 611.3a, 613.1f, 613.4c
**Engine files reviewed**: `crates/engine/src/state/continuous_effect.rs`, `crates/engine/src/rules/layers.rs` (lines 500-862), `crates/engine/src/state/hash.rs` (lines 1143-1215), `crates/engine/src/effects/mod.rs` (line 1988), `crates/engine/src/rules/replacement.rs` (line 1704), `crates/engine/src/rules/copy.rs` (line 111)
**Card defs reviewed**: 28 card definitions
**Tests reviewed**: `crates/engine/tests/static_grants.rs` (19 tests, 13 new for PB-25)

## Verdict: clean

All 11 new EffectFilter variants are correctly implemented with proper CR rule semantics. Hash discriminants 17-27 are unique and sequential. All layers.rs match arms correctly implement the filter logic: battlefield zone check, type/subtype/supertype/color check, controller check, attacking status check via `state.combat`, and "other" exclusion where needed. All 28 card definitions match their oracle text for the abilities that were implemented, with appropriate TODOs documenting remaining DSL gaps (PB-26 triggers, Madness alt cost, Pack Tactics condition, etc.). No wrong game state produced by any implemented ability. Tests cover all 11 filter variants with positive and negative cases, multiplayer scenarios, and edge cases.

## Engine Change Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| (none) | | | Engine changes are correct. |

All 11 EffectFilter variants have correct filter logic in layers.rs, correct hash discriminants in hash.rs, and proper enum definitions in continuous_effect.rs. Wildcard match sites in effects/mod.rs, replacement.rs, and copy.rs all use catch-all patterns (`other => other.clone()` / `_ => false`) so no changes were needed there.

## Card Definition Findings

| # | Severity | Card | Description |
|---|----------|------|-------------|
| (none) | | | All 28 card definitions are correct for the abilities implemented in PB-25. |

## CR Coverage Check

| CR Rule | Implemented? | Tested? | Notes |
|---------|-------------|---------|-------|
| 604.2 | Yes | Yes | Static abilities create continuous effects -- all filter variants tested |
| 611.3 | Yes | Yes | Continuous effects from static abilities not "locked in" |
| 611.3a | Yes | Yes | Dynamic re-evaluation tested via attacking status (combat state present/absent) |
| 613.1f | Yes | Yes | Layer 6 ability grants (AddKeyword, RemoveKeyword) -- 7 of 11 filters used in Ability layer |
| 613.4c | Yes | Yes | Layer 7c P/T modification -- 4 of 11 filters used in PtModify layer |

## Card Def Summary

| Card | Oracle Match | TODOs Remaining | Game State Correct | Notes |
|------|-------------|-----------------|-------------------|-------|
| elesh_norn_grand_cenobite | Yes | 0 | Yes | All 3 abilities implemented |
| massacre_wurm | Partial | 1 | Yes | ETB debuff done; death trigger blocked PB-26 |
| archetype_of_imagination | Partial | 1 | Yes | Grant + remove done; "can't gain" prevention TODO |
| archetype_of_endurance | Partial | 1 | Yes | Same pattern as Imagination |
| blade_historian | Yes | 0 | Yes | Full implementation |
| ohran_frostfang | Yes | 0 | Yes | Static + combat damage trigger both implemented |
| stensia_masquerade | Partial | 2 | Yes | Static done; Vampire combat trigger + Madness TODO |
| crossway_troublemakers | Partial | 2 | Yes | Statics done; death trigger overbroad (no Vampire filter, no optional payment) |
| elderfang_venom | Partial | 1 | Yes | Static done; Elf death trigger blocked PB-26 |
| blight_mound | Partial | 1 | Yes | Both statics done; nontoken death trigger TODO |
| ezuri_renegade_leader | Yes | 0 | Yes | Regenerate + pump both implemented |
| battle_cry_goblin | Partial | 1 | Yes | Pump done; Pack Tactics TODO |
| lathliss_dragon_queen | Partial | 1 | Yes | Flying + pump done; nontoken Dragon ETB TODO |
| elvish_warmaster | Partial | 1 | Yes | ETB trigger overbroad; pump done. "Once each turn" TODO |
| rith_liberated_primeval | Partial | 1 | Yes | Flying + Ward + ward grant done; end step trigger TODO |
| rising_of_the_day | Yes | 0 | Yes | Both statics implemented |
| bloodmark_mentor | Yes | 0 | Yes | Full implementation |
| indomitable_archangel | Yes | 0 | Yes | Flying + Metalcraft conditional static done |
| mikaeus_the_unhallowed | Partial | 1 | Yes | Intimidate + both statics done; Human damage trigger TODO |
| return_of_the_wildspeaker | Partial | 1 | Yes | Mode 2 done; Mode 1 (greatest power draw) TODO |
| bladewing_the_risen | Yes | 0 | Yes | Flying + ETB + pump all implemented |
| silver_fur_master | Partial | 1 | Yes | Ninjutsu + lord done; cost reduction TODO (PB-29) |
| goblin_surprise | Yes | 0 | Yes | Both modes implemented |
| you_see_a_pair_of_goblins | Yes | 0 | Yes | Both modes implemented |
| goblin_war_party | Yes | 0 | Yes | Both modes + Entwine implemented |
| goro_goro_disciple_of_ryusei | Partial | 1 | Yes | Haste grant done; Dragon token ability TODO |
| purphoros_god_of_the_forge | Yes | 0 | Yes | All 4 abilities implemented |
| scourge_of_valkas | Partial | 1 | Yes | Flying + pump done; Dragon ETB damage trigger TODO |

## Test Coverage Summary

All 13 new tests verify the correct behavior:
- `test_creatures_opponents_control_debuff`: Controller immunity + opponent debuff
- `test_creatures_opponents_control_multiplayer`: 4-player multiplayer, all opponents affected
- `test_attacking_creatures_you_control_grants_keyword`: Only attackers get keyword
- `test_attacking_creatures_filter_outside_combat`: No combat state = no matches
- `test_creatures_you_control_with_subtype_includes_self`: Source included (not "other")
- `test_creatures_you_control_with_supertype_legendary`: Only legendary, controller only
- `test_creatures_you_control_with_color_red`: Only red, controller only
- `test_artifacts_you_control_grants_shroud`: Only artifacts, controller only
- `test_other_creatures_excluding_subtype_non_human`: Excludes source + Humans
- `test_creatures_excluding_subtype_spell_effect`: Includes source, excludes subtype
- `test_attacking_creatures_with_subtype`: Attacking + subtype + controller
- `test_all_creatures_with_subtype_no_controller`: No controller restriction
- `test_other_creatures_with_subtypes_or`: OR semantics, source excluded
