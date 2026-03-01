# Ability WIP: Afflict

ability: Afflict
cr: 702.130
priority: P4
started: 2026-03-01
phase: closed
plan_file: memory/abilities/ability-plan-afflict.md

## Step Checklist
- [x] 1. Enum variant — types.rs:673, hash.rs:483-487, view_model.rs:701
- [x] 2. Rule enforcement — builder.rs:763-780 (TriggeredAbilityDef with LoseLife effect)
- [x] 3. Trigger wiring — abilities.rs:1650-1700 (BlockersDeclared handler tags defending_player_id)
- [x] 4. Unit tests — crates/engine/tests/afflict.rs (6 tests: basic, unblocked, multi-blockers, multi-instances, multiplayer, life-not-damage)
- [x] 5. Card definition — crates/engine/src/cards/defs/khenra_eternal.rs (Khenra Eternal, {1}{B}, 2/2, Afflict 1)
- [x] 6. Game script — test-data/generated-scripts/combat/118_khenra_eternal_afflict_life_loss.json
- [x] 7. Coverage doc update — docs/mtg-engine-ability-coverage.md (P4 11/88, 103 total validated)
