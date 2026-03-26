# Primitive WIP: PB-30 -- Combat damage triggers

batch: PB-30
title: Combat damage triggers
cards_affected: ~49
started: 2026-03-25
phase: implement
plan_file: memory/primitives/pb-plan-30.md

## Gap Reference
G-8 from `docs/dsl-gap-closure-plan.md`:
- G-8: Combat damage triggers (per-creature) (~49 cards) — `WheneverCreatureYouControlDealsCombatDamageToPlayer` TriggerCondition; event wiring in combat.rs

## Deferred from Prior PBs
- PB-23 added controller-filtered creature triggers (WheneverCreatureDies, WheneverCreatureYouControlAttacks, WheneverCreatureYouControlDealsCombatDamageToPlayer)
- PB-26 added trigger variants (discard, sacrifice, leaves-battlefield, draw-card, lifegain, cast triggers)
- PB-30 extends this trigger infrastructure for combat damage specifically

## Step Checklist
- [x] 1. Engine changes — 22 changes implemented: new TriggerCondition variants (WheneverCreatureYouControlDealsCombatDamageToPlayer struct form, WhenOneOrMoreCreaturesYouControlDealCombatDamageToPlayer, WhenEquippedCreatureDealsCombatDamageToPlayer, WhenEnchantedCreatureDealsDamageToPlayer, WhenAnyCreatureDealsCombatDamageToOpponent), new TriggerEvent variants (AnyCreatureYouControlBatchCombatDamage, EquippedCreatureDealsCombatDamageToPlayer, EnchantedCreatureDealsDamageToPlayer, AnyCreatureDealsCombatDamageToOpponent), new EffectTarget::TriggeringCreature, PlayerTarget::DamagedPlayer, EffectAmount::CombatDamageDealt, TargetFilter.is_token, TriggeredAbilityDef.combat_damage_filter, PendingTrigger.damaged_player/combat_damage_amount, EffectContext.combat_damage_amount/damaged_player/triggering_creature_id, StackObject.damaged_player/combat_damage_amount/triggering_creature_id, event dispatch in abilities.rs (per-creature + batch + equipped + enchanted + opponent), propagation chain PendingTrigger→StackObject→EffectContext, hash.rs exhaustive updates, replay_harness.rs enrichment for all 5 new TriggerCondition variants
- [x] 2. Card definition fixes — 26 cards fixed: old_gnawbone, the_indomitable, professional_face_breaker, contaminant_grafter, ingenious_infiltrator, prosperous_thief, rakish_heir, stensia_masquerade, alela_cunning_conqueror, curiosity, ophidian_eye, mask_of_memory, sword_of_fire_and_ice, sword_of_body_and_mind, sword_of_sinew_and_steel, sword_of_truth_and_justice, the_reaver_cleaver, lathril_blade_of_the_elves, balefire_dragon (TODO deferred), marisi_breaker_of_the_coil (TODO deferred), sword_of_feast_and_famine, sword_of_light_and_shadow, sword_of_war_and_peace, grim_hireling, natures_will (partial), sigil_of_sleep; 6 existing defs updated to struct form { filter: None }
- [x] 3. New card definitions — none required
- [x] 4. Unit tests — 8 tests in crates/engine/tests/combat_damage_triggers.rs: per_creature, per_creature_per_creature, subtype_filter, batch_fires_once, batch_per_damaged_player, equipped_trigger, equipped_unequipped_no_trigger, enchanted_trigger
- [x] 5. Workspace build verification — 2371 tests pass, 0 clippy warnings, formatting clean
