# Ability WIP: Forage

ability: Forage
cr: 701.61
priority: P4
started: 2026-03-07
phase: closed

## Review
findings: 0 HIGH/MEDIUM + 3 LOW (1 typo fixed inline, 2 deferred)
review_file: memory/abilities/ability-review-forage.md
plan_file: memory/abilities/ability-plan-forage.md

## Step Checklist
- [x] 1. Enum variant — `forage: bool` on `ActivationCost` (game_object.rs:103, hash.rs:1369)
- [x] 2. Rule enforcement — forage cost block in abilities.rs handle_activate_ability (abilities.rs:338)
- [x] 3. Trigger wiring — N/A (forage is a cost action, not a trigger)
- [x] 4. Unit tests — crates/engine/tests/forage.rs (7 tests: sacrifice_food, exile_three, insufficient_resources, requires_mana, food_is_artifact_subtype, non_food_rejected, prefers_food_when_both_available)
- [x] 5. Card definition — crates/engine/src/cards/defs/camellia_the_seedmiser.rs (Menace only; Forage activated TODO: Cost enum gap)
- [x] 6. Game script — test-data/generated-scripts/forage/182_food_token_sacrifice_gain_life.json
- [x] 7. Coverage doc update — docs/mtg-engine-ability-coverage.md: Forage → partial (Cost enum DSL gap noted)
