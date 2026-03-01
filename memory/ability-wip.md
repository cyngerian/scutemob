# Ability WIP: Poisonous

ability: Poisonous
cr: 702.70
priority: P4
started: 2026-03-01
phase: closed
plan_file: memory/abilities/ability-plan-poisonous.md

## Step Checklist
- [x] 1. Enum variant — types.rs:701, hash.rs:497-501, view_model.rs:711, builder.rs:532-545
- [x] 2. Rule enforcement — stubs.rs:307-330, hash.rs:1142-1145, abilities.rs/effects/mod.rs/turn_actions.rs/resolution.rs/miracle.rs (all PendingTrigger sites)
- [x] 3. Trigger wiring — stack.rs:534-558, hash.rs:1471-1480, abilities.rs (dispatch+flush), resolution.rs (resolution+counter arm), view_model.rs:490-492, stack_view.rs:98-100
- [x] 4. Unit tests — crates/engine/tests/poisonous.rs (6 tests, all passing)
- [x] 5. Card definition — crates/engine/src/cards/defs/poisonous_viper.rs (test card: {2}{B}, 2/2 Snake, Poisonous 1)
- [x] 6. Game script — test-data/generated-scripts/combat/122_poisonous_viper_gives_poison_counter.json
- [x] 7. Coverage doc update — docs/mtg-engine-ability-coverage.md (P4 15/88, 107 total validated)

## Review
findings: 2 LOW (no HIGH/MEDIUM)
verdict: clean
review_file: memory/abilities/ability-review-poisonous.md
