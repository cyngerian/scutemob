# Primitive WIP: PB-26 -- Trigger Variants (all remaining)

batch: PB-26
title: Trigger variants (spell-type, discard, sacrifice, attack, LTB, draw, cast)
cards_affected: ~72
started: 2026-03-24
phase: implement
plan_file: memory/primitives/pb-plan-26.md

## Gap Reference
G-4, G-9, G-10, G-11, G-12, G-13, G-14, G-15 from `docs/dsl-gap-closure-plan.md`:
- G-4: Spell-type filter on triggers (~19 cards) — add spell_type_filter to WheneverYouCastSpell
- G-9: Discard triggers (~9 cards) — WheneverYouDiscard, WheneverOpponentDiscards
- G-10: Sacrifice triggers (~6 cards) — WheneverYouSacrifice { filter }
- G-11: WheneverYouAttack (~8 cards) — fire at declare-attackers step
- G-12: Leaves-battlefield triggers (~6 cards) — WhenLeavesBattlefield
- G-13: Draw-card trigger filtering (~16 cards) — player_filter on WheneverPlayerDrawsCard
- G-14: Lifegain trigger filtering (~3 cards) — verify WheneverYouGainLife covers all patterns
- G-15: Cast triggers ("when you cast this spell") (~5 cards) — WhenYouCastThisSpell

## Deferred from Prior PBs
- Some PB-25 scope cards also need PB-26 triggers (per last handoff)

## Step Checklist
- [x] 1. Engine changes (new types/variants/dispatch)
  - New TriggerCondition variants: WheneverYouDiscard, WheneverOpponentDiscards, WheneverYouSacrifice{filter,player_filter}, WheneverYouAttack, WhenLeavesBattlefield, WhenYouCastThisSpell
  - Extended: WheneverYouCastSpell{spell_type_filter,noncreature_only}, WheneverOpponentCastsSpell{spell_type_filter,noncreature_only}, WheneverPlayerDrawsCard{player_filter}
  - New TriggerEvent variants: ControllerDiscards, OpponentDiscards, ControllerSacrifices, ControllerAttacks, SelfLeavesBattlefield, ControllerDrawsCard, AnyPlayerDrawsCard, OpponentDrawsCard, ControllerGainsLife
  - New GameEvent: PermanentSacrificed
  - New PlayerTarget::TriggeringPlayer
  - Dispatch wiring in check_triggers for all 8 gaps (G-4 through G-15)
  - PermanentSacrificed emitted from all sacrifice paths (effects/mod.rs, abilities.rs, casting.rs, resolution.rs)
  - hash.rs discriminants for all new variants
  - replay_harness.rs TriggerCondition→TriggerEvent mapping for all new variants
- [x] 2. Card definition fixes (~72 cards fixed)
  - G-4 (19 cards): talrand, guttersnipe, murmuring_mystic, archmage_emeritus, beast_whisperer, monastery_mentor, whispering_wizard, lys_alana_huntmaster, sram, bontus_monument, nezahal, mystic_remora, archmage_of_runes, hazorets_monument, slickshot_show_off, hermes_overseer_of_elpis, leaf_crowned_visionary, storm_kiln_artist, chulane, oketras_monument, alela_cunning_conqueror, inexorable_tide
  - G-9 (7 cards): waste_not, lilianas_caress, megrim, raiders_wake, fell_specter, glint_horn_buccaneer, brallin_skyshark_rider
  - G-10 (9 cards): korvold, camellia, smothering_abomination, captain_lannery_storm, tireless_tracker, juri, mirkwood_bats, carmen_cruel_skymarcher, mayhem_devil
  - G-11 (5 cards): caesar_legions_emperor, seasoned_dungeoneer, chivalric_alliance, mishra_claimed_by_gix, anim_pakal_thousandth_moon
  - G-12 (3 cards): aven_riftwatcher, toothy_imaginary_friend, sengir_autocrat
  - G-13 (7 cards): niv_mizzet_the_firemind, consecrated_sphinx, scrawling_crawler, razorkin_needlehead, smothering_tithe (already had filter fix applied), niv_mizzet_parun and others already had WheneverYouDrawACard (now dispatched)
  - G-14 (3 cards): elendas_hierophant, vito_thorn_of_the_dusk_rose (marauding_blight_priest was already correct, dispatch wiring fixed it)
  - G-15 (1 card): elder_deep_fiend
- [x] 3. New card definitions (if any) — none required
- [x] 4. Unit tests — crates/engine/tests/trigger_variants.rs (19 tests, all passing)
- [x] 5. Workspace build verification — 2334 tests passing, 0 clippy warnings, clean build
