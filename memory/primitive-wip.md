# Primitive WIP: PB-15 -- Saga & Class Mechanics (REVIEW-ONLY)

batch: PB-15
title: Saga & Class Mechanics
cards_affected: 2
mode: review-only
started: 2026-03-18
phase: done
plan_file: n/a (retroactive review -- no plan needed)
review_file: memory/primitives/pb-review-15.md

## Review Findings Summary
- 1 HIGH: Class level-up bypasses the stack (CR 716.2a) -- FIXED 2026-03-18
- 1 MEDIUM: Saga precombat main TBA missing is_phased_in() check -- FIXED 2026-03-18
- 2 MEDIUM: Card def TODOs (urzas_saga chapters I/II, druid_class levels 1/2/3) -- DSL gaps, deferred
- 1 LOW: Urza's Saga chapter III uses mana value instead of actual mana cost -- deferred

## Fixes Applied (2026-03-18)
- Finding 1 (HIGH): Added StackObjectKind::ClassLevelAbility { source_object, target_level } (disc 68)
  to state/stack.rs. handle_level_up_class in engine.rs now pushes to stack instead of resolving
  immediately. Resolution arm added to resolution.rs (sets class_level + registers continuous effects).
  Hash arm in hash.rs. TUI stack_view.rs and replay-viewer view_model.rs match arms added.
  Tests updated: class_level_up_from_1_to_2_cr716_2a and class_sequential_level_up_cr716_2a now
  pass priority after level-up to trigger stack resolution.
- Finding 2 (MEDIUM): Added && obj.is_phased_in() to Saga precombat main filter in turn_actions.rs:611.
- Finding 3 (LOW): Addressed by Finding 1 fix -- AbilityActivated now emits with actual stack_object_id.

## Test Results
- 2155 tests passing, 0 failed
- cargo clippy -- -D warnings: clean
- cargo build --workspace: clean
- cargo fmt --check: clean
