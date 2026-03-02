# Ability WIP: Dash

ability: Dash
cr: 702.109
priority: P4
started: 2026-03-01
phase: closed
plan_file: memory/abilities/ability-plan-dash.md

## Step Checklist
- [x] 1. Enum variant — types.rs:KeywordAbility::Dash, card_definition.rs:AbilityDefinition::Dash, stack.rs:StackObjectKind::DashReturnTrigger, game_object.rs:was_dashed, stubs.rs:is_dash_return_trigger, hash.rs
- [x] 2. Rule enforcement — casting.rs:cast_with_dash param+validation+get_dash_cost, command.rs:CastSpell.cast_with_dash, engine.rs dispatch, resolution.rs:was_dashed+haste ETB transfer
- [x] 3. Trigger wiring — turn_actions.rs:end_step_actions dash scan, abilities.rs:flush_pending_triggers DashReturnTrigger, resolution.rs:DashReturnTrigger arm
- [x] 4. Unit tests — crates/engine/tests/dash.rs: 7 tests (basic cast, normal cast, return to hand, creature left BF, alt-cost exclusivity, combine-with-evoke, commander tax)
- [x] 5. Card definition — Zurgo Bellstriker (crates/engine/src/cards/defs/zurgo_bellstriker.rs)
- [x] 6. Game script — 132_dash_zurgo_bellstriker.json (stack/)
- [x] 7. Coverage doc update — Dash → validated (P4: 26/88)

## Review
findings: 4 (0 HIGH, 0 MEDIUM, 4 LOW)
review_file: memory/abilities/ability-review-dash.md
fix_applied: none — verdict clean, 4 LOW deferred
