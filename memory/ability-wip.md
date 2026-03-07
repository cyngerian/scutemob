# Ability WIP: Amplify

ability: Amplify
cr: 702.38
priority: P4
started: 2026-03-06
phase: closed

## Review
findings: 3 (0 HIGH, 0 MEDIUM, 3 LOW)
review_file: memory/abilities/ability-review-amplify.md
plan_file: memory/abilities/ability-plan-amplify.md

## Step Checklist
- [x] 1. Enum variant — types.rs:1117, hash.rs:613, view_model.rs:829
- [x] 2. Rule enforcement — resolution.rs:679, lands.rs:215
- [x] 3. Trigger wiring — N/A (static ETB replacement, no trigger)
- [x] 4. Unit tests — tests/amplify.rs (8 tests: basic, multiple revealed, no match, N multiplier, dual instances, empty hand, changeling, non-creature)
- [x] 5. Card definition — crates/engine/src/cards/defs/canopy_crawler.rs
- [x] 6. Game script — test-data/generated-scripts/stack/159_amplify_canopy_crawler.json
- [x] 7. Coverage doc update — docs/mtg-engine-ability-coverage.md (Amplify: validated)
