# Ability WIP: Provoke

ability: Provoke
cr: 702.39
priority: P4
started: 2026-03-01
phase: closed
plan_file: memory/abilities/ability-plan-provoke.md

## Step Checklist
- [x] 1. Enum variant — types.rs:659, hash.rs:479, view_model.rs:695
- [x] 2. Rule enforcement — combat.rs:55 (forced_blocks field), combat.rs:630 (enforcement), hash.rs:1487
- [x] 3. Trigger wiring — stack.rs:469, builder.rs:743, abilities.rs (provoke tagging + flush), resolution.rs (ProvokeTrigger arm), stubs.rs:269, hash.rs
- [x] 4. Unit tests — crates/engine/tests/provoke.rs (7 tests, all passing)
- [x] 5. Card definition — crates/engine/src/cards/defs/goblin_grappler.rs (Goblin Grappler, {R}, 1/1, Provoke)
- [x] 6. Game script — test-data/generated-scripts/combat/117_goblin_grappler_provoke_forced_block.json
- [x] 7. Coverage doc update — docs/mtg-engine-ability-coverage.md (P4 10/88, 102 total validated)
