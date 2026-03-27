# Primitive WIP: PB-33 -- Copy/clone + exile/flicker timing

batch: PB-33
title: Copy/clone + exile/flicker timing
cards_affected: ~39
started: 2026-03-27
phase: implement
plan_file: memory/primitives/pb-plan-33.md

## Gap Reference
G-22: Copy/clone (DSL wiring) (~12 cards) — Wire `Effect::CopyPermanent` for card defs — clone ETB replacement
G-28: Exile/flicker timing (~27 cards) — Delayed return triggers — "exile until end of turn" / "exile, return at beginning of next end step". Extend `Effect::ExileObject` with `return_timing`

## Deferred from Prior PBs
- Clone/copy ETB choice (PB-13j) — BecomeCopyOf exists, choose-target gap remains
- Scion of the Ur-Dragon copy-self (PB-17) — needs EffectTarget::LastSearchResult

## Step Checklist
- [x] 1. Engine changes (new types/variants/dispatch)
  - DelayedTrigger expanded (stubs.rs): controller, target_object, action, timing, fired
  - DelayedTriggerAction enum: ReturnFromExileToBattlefield, ReturnFromExileToHand, ReturnFromGraveyardToHand, SacrificeObject, ExileObject
  - DelayedTriggerTiming enum: AtNextEndStep, AtOwnersNextEndStep, WhenSourceLeavesBattlefield, AtEndOfCombat
  - PendingTriggerKind::DelayedAction added (stubs.rs)
  - TriggerData::DelayedAction { action, target } added (stack.rs)
  - StackObjectKind::DelayedActionTrigger added (stack.rs)
  - Effect::ExileWithDelayedReturn + DelayedReturnDestination added (card_definition.rs)
  - Effect::CreateTokenCopy extended: except_not_legendary, gains_haste, delayed_action fields
  - EffectTarget::EquippedCreature added (card_definition.rs)
  - LayerModification::RemoveSuperType added (continuous_effect.rs)
  - GameObject flags: sacrifice_at_end_step, exile_at_end_step, return_to_hand_at_end_step
  - hash.rs: all new types hashed
  - effects/mod.rs: ExileWithDelayedReturn dispatch, EquippedCreature resolution, CreateTokenCopy mods
  - turn_actions.rs: end_step_actions processes delayed triggers + new flags
  - abilities.rs: flush_pending_triggers handles DelayedAction, check_triggers scans WhenSourceLeavesBattlefield
  - resolution.rs: DelayedActionTrigger resolution + countered case
  - engine.rs: check_and_flush_triggers cleans up WhenSourceLeavesBattlefield triggers
  - TUI stack_view.rs + replay-viewer view_model.rs: DelayedActionTrigger arms added
  - layers.rs: RemoveSuperType applied in type-change layer
- [x] 2. Card definition fixes
  - kiki_jiki_mirror_breaker.rs: activated ability with CreateTokenCopy + gains_haste + delayed sacrifice
  - the_fire_crystal.rs: activated ability with CreateTokenCopy + delayed sacrifice
  - helm_of_the_host.rs: triggered ability with CreateTokenCopy + EquippedCreature + except_not_legendary + gains_haste
  - miirym_sentinel_wyrm.rs: WheneverCreatureEntersBattlefield + CreateTokenCopy + except_not_legendary
  - the_eternal_wanderer.rs: +1 loyalty ability uses ExileWithDelayedReturn (AtOwnersNextEndStep)
  - brutal_cathar.rs: ETB trigger uses ExileWithDelayedReturn (WhenSourceLeavesBattlefield)
  - nezahal_primal_tide.rs: activated discard-3 with ExileWithDelayedReturn (AtNextEndStep, tapped)
  - chandra_flamecaller.rs: +1 creates Elementals with exile_at_end_step=true
  - voice_of_victory.rs: Mobilize tokens with sacrifice_at_end_step=true
  - zurgo_stormrender.rs: Mobilize token with sacrifice_at_end_step=true
  - the_locust_god.rs: WhenDies trigger uses SetReturnToHandAtEndStep
  - puppeteer_clique.rs: updated comment noting partial fix scope
  - mirage_phalanx.rs: AtBeginningOfCombat CreateTokenCopy + gains_haste + delayed exile AtEndOfCombat
  - thousand_faced_shadow.rs: new fields except_not_legendary/gains_haste/delayed_action added (defaults)
  - mist_syndicate_naga.rs: new fields added (defaults)
- [x] 3. New card definitions (if any)
  - No new card defs needed (all updates to existing defs)
- [x] 4. Unit tests
  - crates/engine/tests/delayed_triggers.rs (7 tests, all passing)
  - test_sacrifice_at_end_step_mobilize_token — CR 603.7b Mobilize pattern
  - test_exile_at_end_step_token — CR 603.7b Chandra Flamecaller pattern
  - test_return_to_hand_at_end_step — CR 603.7 Locust God pattern
  - test_delayed_trigger_fires_only_once — CR 603.7b AtNextEndStep fires once
  - test_create_token_copy_with_haste — CR 707.9a gains_haste
  - test_create_token_copy_not_legendary — CR 707.9b except_not_legendary
  - test_exile_until_source_leaves — CR 610.3 WhenSourceLeavesBattlefield
- [x] 5. Workspace build verification
  - cargo build --workspace: clean
  - cargo test --all: 2403 passing, 0 failed
  - cargo clippy -- -D warnings: clean
  - cargo fmt --check: clean
