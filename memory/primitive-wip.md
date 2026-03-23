# Primitive WIP: PB-23 -- Controller-filtered creature triggers

batch: PB-23
title: Controller-filtered creature triggers
cards_affected: ~145
started: 2026-03-23
phase: implement
plan_file: memory/primitives/pb-plan-23.md

## Gap Reference
- G-1 from docs/dsl-gap-closure-plan.md
- Add `controller` field to `WheneverCreatureDies`, `WheneverCreatureEntersBattlefield`
- New variants: `WheneverCreatureYouControlAttacks`, `WheneverCreatureYouControlDealsCombatDamage`
- ~145 card defs have TODOs blocked by missing controller-filtered triggers

## Deferred from Prior PBs
- `WheneverCreatureDies` has no controller filter — fires on ALL creatures dying (from A-23 handoff)
- `WheneverCreatureYouControlAttacks` trigger does not exist (blocks most A-24 cards)

## Step Checklist
- [x] 1. Engine changes (add controller field to existing triggers, new trigger variants, event wiring)
  - Added `controller: Option<TargetController>` to `WheneverCreatureDies` in card_definition.rs
  - Added `WheneverCreatureYouControlAttacks` and `WheneverCreatureYouControlDealsCombatDamageToPlayer` to TriggerCondition
  - Added `AnyCreatureDies`, `AnyCreatureYouControlAttacks`, `AnyCreatureYouControlDealsCombatDamageToPlayer` to TriggerEvent
  - Added `DeathTriggerFilter` struct with controller_you/controller_opponent/exclude_self/nontoken_only fields
  - Added `death_filter: Option<DeathTriggerFilter>` to `TriggeredAbilityDef`
  - Updated hash.rs for all new types/variants
  - Updated exports: state/mod.rs, lib.rs, cards/helpers.rs, testing/replay_harness.rs imports
  - Wired all three trigger conditions in enrich_spec_from_def (replay_harness.rs)
  - Wired AnyCreatureDies dispatch in CreatureDied handler (abilities.rs)
  - Wired AnyCreatureYouControlAttacks dispatch in AttackersDeclared handler
  - Wired AnyCreatureYouControlDealsCombatDamageToPlayer dispatch in CombatDamageDealt handler
  - Updated TriggerDoublerFilter::CreatureDeath to also match AnyCreatureDies
  - Updated 23 card defs with bare WheneverCreatureDies to use controller parameter
  - Fixed all TriggeredAbilityDef struct literals (card defs, tests, engine files) to include death_filter: None
- [x] 2. Card definition fixes (controller-filtered trigger TODO → real implementation)
  - WheneverCreatureDies (controller_you): bastion_of_remembrance, dark_prophecy, moldervine_reclamation, liliana_dreadhorde_general, marionette_apprentice, midnight_reaper, morbid_opportunist, pawn_of_ulamog, skemfar_avenger, crossway_troublemakers, vindictive_vampire, elas_il_kor, pitiless_plunderer, grim_haruspex, sifter_of_skulls, vengeful_bloodwitch, agent_venom (+ controller update from TODO)
  - WheneverCreatureDies (controller_opponent): yahenni_undying_partisan
  - WheneverCreatureDies (controller: None, global): blood_artist, cordial_vampire, falkenrath_noble, fecundity, black_market, vein_ripper, poison_tip_archer, zulaport_cutthroat, elenda_the_dusk_rose, syr_konrad_the_grim
  - The Meathook Massacre: two death triggers (you + opponent) now implemented
  - WheneverCreatureYouControlAttacks: beastmaster_ascension, druids_repository, mardu_ascendancy, utvara_hellkite, kolaghan_the_storms_fury
  - WheneverCreatureYouControlDealsCombatDamageToPlayer: coastal_piracy, reconnaissance_mission, bident_of_thassa, toski_bearer_of_secrets, ohran_frostfang, enduring_curiosity
  - Remaining TODO cards kept as TODO (complex effects, subtype filters, or other DSL gaps)
- [x] 3. Unit tests (10 tests in creature_triggers.rs, all passing)
  - test_whenever_creature_you_control_dies_fires_on_your_creature
  - test_whenever_creature_you_control_dies_ignores_opponent_creature
  - test_whenever_any_creature_dies_fires_on_any
  - test_whenever_creature_opponent_controls_dies_fires
  - test_whenever_creature_opponent_controls_dies_ignores_your_creature
  - test_whenever_creature_you_control_attacks_fires
  - test_whenever_creature_you_control_attacks_ignores_opponent
  - test_whenever_creature_you_control_attacks_fires_per_creature
  - test_whenever_creature_you_control_deals_combat_damage_to_player_fires
  - test_whenever_creature_you_control_deals_combat_damage_ignores_opponent
- [x] 4. Workspace build verification
  - cargo test --all: 2291 passing (2281 + 10 new), 0 failures
  - cargo clippy -- -D warnings: 0 warnings (fixed 2 map_or → is_some_and/is_none_or)
  - cargo build --workspace: clean build (engine + simulator + network + replay-viewer + tui)
  - cargo fmt --check: clean
