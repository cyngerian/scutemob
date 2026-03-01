# Ability WIP: Training

ability: Training
cr: 702.149
priority: P4
started: 2026-03-01
phase: closed
plan_file: memory/abilities/ability-plan-training.md

## Step Checklist
- [x] 1. Enum variant — types.rs:692 (Training), game_object.rs:211 (SelfAttacksWithGreaterPowerAlly), hash.rs:493 (discriminant 82), hash.rs:1187 (discriminant 19), view_model.rs:705
- [x] 2. Rule enforcement — builder.rs:489 (Training TriggeredAbilityDef auto-generation)
- [x] 3. Trigger wiring — abilities.rs:1509 (Training trigger collection in AttackersDeclared handler)
- [x] 4. Unit tests — tests/training.rs (7 tests: basic, alone, equal, lower, multiple-instances, two-trainees, multiplayer)
- [x] 5. Card definition — crates/engine/src/cards/defs/gryff_rider.rs (Gryff Rider, {2}{W}, 2/1, Flying + Training)
- [x] 6. Game script — test-data/generated-scripts/combat/120_gryff_rider_training_counter.json
- [x] 7. Coverage doc update — docs/mtg-engine-ability-coverage.md (P4 13/88, 105 total validated, CR corrected to 702.149)
