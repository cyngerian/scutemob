# Primitive WIP: PB-30 -- Combat damage triggers

batch: PB-30
title: Combat damage triggers
cards_affected: ~49
started: 2026-03-25
phase: fix
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

## Fix Phase Results (2026-03-25)
All 5 HIGH and 4 MEDIUM findings resolved:
- H1 (hash.rs): Added `self.combat_damage_filter.hash_into(hasher);` to TriggeredAbilityDef hash impl
- H2 (hash.rs): Added `self.damaged_player.hash_into(hasher);` and `self.combat_damage_amount.hash_into(hasher);` to PendingTrigger hash impl
- H3 (abilities.rs): Captured `pre_len` before `collect_triggers_for_event` for SelfDealsCombatDamageToPlayer, then populated `damaged_player`, `combat_damage_amount`, `entering_object_id` on triggers[pre_len..] — fixes Lathril token creation
- H4 (abilities.rs): Added `combat_damage_filter` check in batch trigger dispatch — iterates assignments to verify at least one creature matches filter before emitting PendingTrigger — fixes Prosperous Thief and Alela false triggers
- H5 (edric_spymaster_of_trest.rs): Added `WhenAnyCreatureDealsCombatDamageToOpponent` triggered ability with DrawCards effect; documented PlayerTarget::Controller approximation
- M5 (abilities.rs): Added TODO(PB-37) comment on EnchantedCreatureDealsDamageToPlayer noncombat path
- M6 (card_definition.rs): Added doc comment on is_token field explaining it's only checked in combat_damage_filter path
- M8 (curiosity.rs, ophidian_eye.rs): Added TODO(PB-37) comments for opponent-vs-player approximation and noncombat gap
- M9 (alela_cunning_conqueror.rs): Replaced DeclaredTarget { index: 0 } Goad with Effect::Sequence(vec![]) placeholder + TODO(PB-37)
Post-fix: 2371 tests pass, 0 clippy warnings, workspace build clean
