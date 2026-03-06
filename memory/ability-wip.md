# Ability WIP: Cumulative Upkeep

ability: Cumulative Upkeep
cr: 702.24
priority: P4
started: 2026-03-06
phase: closed

## Review
findings: 3 (1 HIGH, 1 MEDIUM, 1 LOW)
review_file: memory/abilities/ability-review-cumulative-upkeep.md
plan_file: memory/abilities/ability-plan-cumulative-upkeep.md

## Step Checklist
- [x] 1. Enum variant (types.rs:1037,152,192; card_definition.rs:491; hash.rs; state/mod.rs)
- [x] 2. Rule enforcement (resolution.rs CumulativeUpkeepTrigger handler; engine.rs handle_pay_cumulative_upkeep)
- [x] 3. Trigger wiring (stubs.rs PendingTriggerKind::CumulativeUpkeep; stack.rs SOK 41; turn_actions.rs upkeep scan; abilities.rs trigger-to-stack; command.rs; events.rs)
- [x] 4. Unit tests (tests/cumulative_upkeep.rs — 8 tests: basic_age_counter_added, pay_mana_keeps_permanent, decline_payment_sacrifices, escalating_cost, pay_life_cost, permanent_left_battlefield, multiplayer_only_controller_upkeep, multiple_instances_share_counters)
- [x] 5. Card definition — crates/engine/src/cards/defs/mystic_remora.rs
- [x] 6. Game script — test-data/generated-scripts/stack/152_cumulative_upkeep_mystic_remora.json
- [x] 7. Coverage doc update — docs/mtg-engine-ability-coverage.md (Cumulative Upkeep: validated)
