# Ability WIP: Overload

ability: Overload
cr: 702.96
priority: P3
started: 2026-02-28
phase: closed
plan_file: memory/abilities/ability-plan-overload.md

## Step Checklist
- [x] 1. Enum variant — types.rs:588, card_definition.rs:255+822, stack.rs:127, command.rs:157, hash.rs (KeywordAbility 70, Condition 9, AbilityDefinition 21)
- [x] 2. Rule enforcement — casting.rs (handle_cast_spell + get_overload_cost), engine.rs, effects/mod.rs (EffectContext + Condition::WasOverloaded), resolution.rs (ctx.was_overloaded), hash.rs (StackObject.was_overloaded), replay_harness.rs (cast_spell_overload), view_model.rs (format_keyword)
- [x] 3. Trigger wiring — n/a (Overload is alternative cost, not a trigger)
- [x] 4. Unit tests — crates/engine/tests/overload.rs (11 tests, all pass)
- [x] 5. Card definition — crates/engine/src/cards/defs/vandalblast.rs
- [x] 6. Game script — test-data/generated-scripts/baseline/108_vandalblast_overload.json
- [x] 7. Coverage doc update

## Review
findings: 3 MEDIUM, 1 LOW — MEDIUM fixes applied; LOW deferred
review_file: memory/abilities/ability-review-overload.md
