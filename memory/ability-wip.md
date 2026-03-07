# Ability WIP: Amass

ability: Amass
cr: 701.47
priority: P4
started: 2026-03-06
phase: closed

## Review
findings: 2 (0 HIGH, 1 MEDIUM, 1 LOW)
review_file: memory/abilities/ability-review-amass.md
plan_file: memory/abilities/ability-plan-amass.md

## Step Checklist
- [x] 1. Enum variant — `Effect::Amass { subtype, count }` in card_definition.rs:681; `army_token_spec()` helper at card_definition.rs:1279; re-exported from cards/mod.rs and lib.rs
- [x] 2. Rule enforcement — effects/mod.rs:1073 (CR 701.47a: find/create Army, add counters, add subtype); `GameEvent::Amassed` in events.rs:714; hash Effect disc 41, GameEvent disc 98 in hash.rs
- [x] 3. Trigger wiring — abilities.rs: `GameEvent::Amassed` arm added (no-op placeholder; no "whenever you amass" trigger card yet)
- [x] 4. Unit tests — crates/engine/tests/amass.rs: 7 tests, all passing (1613 total)
- [x] 5. Card definition — crates/engine/src/cards/defs/dreadhorde_invasion.rs
- [x] 6. Game script — test-data/generated-scripts/stack/161_amass_dreadhorde_invasion.json (cast/resolve; upkeep trigger gap is pre-existing AtBeginningOfYourUpkeep infra gap, Amass validated by 7 unit tests)
- [x] 7. Coverage doc update — docs/mtg-engine-ability-coverage.md (Amass: validated)
