# Ability WIP: Forecast

ability: Forecast
cr: 702.57
priority: P4
started: 2026-03-06
phase: closed

## Review
findings: 2 (0 HIGH, 0 MEDIUM, 2 LOW)
review_file: memory/abilities/ability-review-forecast.md
plan_file: memory/abilities/ability-plan-forecast.md

## Step Checklist
- [x] 1. Enum variant (types.rs:1076, card_definition.rs:508, hash.rs:601+3557, view_model.rs:815)
- [x] 2. Rule enforcement (command.rs:386, engine.rs:242, abilities.rs:669, stack.rs:944, resolution.rs:826, harness.rs:578)
- [x] 3. Trigger wiring (n/a — Forecast is an activated ability, not triggered)
- [x] 4. Unit tests (crates/engine/tests/forecast.rs — 9 tests all pass)
- [x] 5. Card definition — crates/engine/src/cards/defs/sky_hussar.rs
- [x] 6. Game script — test-data/generated-scripts/stack/154_forecast_sky_hussar.json
- [x] 7. Coverage doc update — docs/mtg-engine-ability-coverage.md (Forecast: validated)
