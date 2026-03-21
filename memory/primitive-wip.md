# Primitive WIP: PB-22 S6 -- Emblem Creation (CR 114)

batch: PB-22
session: S6
title: Emblem Creation (CR 114)
cards_affected: 11 (Ajani Sleeper Agent + ~10 planeswalker cards)
started: 2026-03-21
phase: implement
plan_file: memory/primitives/pb-plan-22-s6.md

## Deferred from Prior PBs
- Emblem creation (PB-14) — 11 planeswalker cards blocked on emblem infrastructure

## Step Checklist
- [x] 1. Engine changes: Effect::CreateEmblem (emblem game object in command zone)
  - Added `is_emblem: bool` to `GameObject` (game_object.rs) with `#[serde(default)]`
  - Added `Effect::CreateEmblem { triggered_abilities, static_effects }` (card_definition.rs, disc 66)
  - Added `GameEvent::EmblemCreated { player, object_id }` (events.rs, disc 124)
  - Added hash support (hash.rs) — Effect disc 66, GameEvent disc 124, is_emblem field
  - Implemented CreateEmblem dispatch (effects/mod.rs) — creates emblem in command zone, registers static CEs, emits event
  - Added `collect_emblem_triggers_for_event` helper in abilities.rs; called from SpellCast handler (CR 113.6p, CR 114.4)
  - Exported `TriggeredAbilityDef`, `TriggerEvent`, `ETBTriggerFilter`, `InterveningIf` in helpers.rs
- [x] 2. Emblem zone placement + SBA immunity (CR 114.1-114.4)
  - Emblems use `is_token: false` — token SBA does not fire for emblems
  - `ZoneId::Command(ctrl)` used for emblem placement (verified no other SBAs target command zone)
  - Static CEs registered with `EffectDuration::Indefinite` (emblems never leave command zone)
- [x] 3. Card definition fixes (Ajani Sleeper Agent + other planeswalker emblem abilities)
  - ajani_sleeper_agent.rs: -6 TODO replaced with Effect::CreateEmblem (AnySpellCast trigger)
  - basri_ket.rs: new card def (5 abilities, emblem on -6)
  - kaito_bane_of_nightmares.rs: new card def (4 abilities, emblem with static P/T on +1)
  - tyvar_kell.rs: new card def (3 abilities, emblem on -6)
  - wrenn_and_realmbreaker.rs: new card def (3 abilities, emblem on -7 with TODO for play-from-graveyard)
  - wrenn_and_seven.rs: new card def (4 abilities, emblem with NoMaxHandSize static on -8)
- [x] 4. Unit tests (5+ tests)
  - 7 tests in crates/engine/tests/emblem_tests.rs — all passing
  - test_emblem_creation_basic, test_emblem_triggered_ability_fires, test_emblem_survives_board_wipe
  - test_emblem_not_removed_by_token_sba, test_multiple_emblems_stack, test_emblem_static_effect
  - test_emblem_persists_after_source_removed
- [x] 5. Workspace build verification
  - cargo test --all: all tests pass (7 new emblem tests + all prior tests)
  - cargo clippy -- -D warnings: clean
  - cargo build --workspace: clean
  - cargo fmt --check: clean
