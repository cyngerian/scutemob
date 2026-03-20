# Primitive WIP: PB-20 -- Additional Combat Phases

batch: PB-20
title: Additional Combat Phases
cards_affected: 10
started: 2026-03-19
phase: implement
plan_file: memory/primitives/pb-plan-20.md

## Deferred from Prior PBs
none

## Step Checklist
- [x] 1. Engine changes
  - Added `additional_phases: Vector<Phase>` to TurnState (replacing `extra_combats: u32`)
  - Added `Effect::AdditionalCombatPhase { followed_by_main: bool }` to card_definition.rs
  - Added `ForEachTarget::EachAttackingCreature` to card_definition.rs
  - Added `Condition::IsFirstCombatPhase` to card_definition.rs
  - Modified `advance_step()` in turn_structure.rs: EndOfCombat and PostCombatMain now check `additional_phases` queue (LIFO pop_back)
  - Added `execute_effect_inner` arm for `Effect::AdditionalCombatPhase` in effects/mod.rs
  - Added `collect_for_each` arm for `ForEachTarget::EachAttackingCreature` in effects/mod.rs
  - Added `check_condition` arm for `Condition::IsFirstCombatPhase` in effects/mod.rs
  - Added `GameEvent::AdditionalCombatPhaseCreated` to events.rs
  - Updated hash.rs: TurnState (additional_phases), Effect::AdditionalCombatPhase, ForEachTarget::EachAttackingCreature, Condition::IsFirstCombatPhase, GameEvent::AdditionalCombatPhaseCreated
  - Updated builder.rs: additional_phases: Vector::new()
  - Updated advance_turn() in turn_structure.rs: additional_phases = Vector::new()
- [x] 2. Card definition fixes
  - karlach_fury_of_avernus.rs: full triggered ability implemented (WhenAttacks + IsFirstCombatPhase + EachAttackingCreature untap + first strike + AdditionalCombatPhase)
- [x] 3. New card definitions
  - combat_celebrant.rs: authored (simplified -- Exert mechanic TODO)
  - breath_of_fury.rs: authored (DSL gap TODOs -- Aura re-attachment not yet supported)
- [x] 4. Unit tests
  - crates/engine/tests/additional_combat.rs: 9 tests written and passing
  - test_additional_combat_phase_basic
  - test_additional_combat_phase_with_main
  - test_additional_combat_lifo_ordering
  - test_additional_combat_not_on_opponents_turn
  - test_additional_combat_in_extra_combat_flag
  - test_is_first_combat_phase_condition_flag
  - test_additional_combat_phases_cleared_on_new_turn
  - test_additional_combat_event_emitted
  - test_no_extra_combats_normal_flow
- [x] 5. Workspace build verification
  - cargo test --all: 2184 tests passing (0 failed)
  - cargo clippy -- -D warnings: 0 warnings
  - cargo build --workspace: clean
  - cargo fmt --check: clean
