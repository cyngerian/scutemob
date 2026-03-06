# Ability WIP: Fading

ability: Fading
cr: 702.32
priority: P4
started: 2026-03-06
phase: closed

## Review
findings: 2 (0 HIGH, 0 MEDIUM, 2 LOW)
review_file: memory/abilities/ability-review-fading.md
plan_file: memory/abilities/ability-plan-fading.md

## Step Checklist
- [x] 1. Enum variant — types.rs:141,1022; hash.rs:232,563,1674,3386; card_definition.rs:471; stack.rs:907; stubs.rs:79; TUI stack_view.rs; replay-viewer view_model.rs
- [x] 2. Rule enforcement — resolution.rs ETB fading counters; lands.rs ETB fading counters
- [x] 3. Trigger wiring — stubs.rs:79 FadingUpkeep; stack.rs:907 FadingTrigger; turn_actions.rs upkeep loop; abilities.rs flush FadingUpkeep; resolution.rs FadingTrigger resolution
- [x] 4. Unit tests — crates/engine/tests/fading.rs (8 tests: ETB, upkeep removal, sacrifice at 0, full lifecycle, multiplayer, non-creature, fade vs time counter distinction)
- [x] 5. Card definition — crates/engine/src/cards/defs/blastoderm.rs
- [x] 6. Game script — test-data/generated-scripts/stack/150_fading_blastoderm.json
- [x] 7. Coverage doc update — docs/mtg-engine-ability-coverage.md (Fading: validated)
