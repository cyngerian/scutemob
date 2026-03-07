# Ability WIP: Discover

ability: Discover
cr: 701.57
priority: P4
started: 2026-03-07
phase: closed

## Review
findings: 1 MEDIUM + 2 LOW — all applied
review_file: memory/abilities/ability-review-discover.md
plan_file: memory/abilities/ability-plan-discover.md

## Step Checklist
- [x] 1. Enum variant — types.rs:1241, hash.rs:661, view_model.rs:862
- [x] 2. Rule enforcement — copy.rs:509 (resolve_discover), card_definition.rs:937 (Effect::Discover), effects/mod.rs:1854 (handler), events.rs:659 (3 variants), hash.rs (arms)
- [x] 3. Trigger wiring — N/A: Discover is a keyword action, not a triggered ability. Invoked via Effect::Discover during resolution of parent triggers.
- [x] 4. Unit tests — crates/engine/tests/discover.rs (7 tests: basic, mv_equal, empty_library, all_lands, remaining_to_bottom, vs_cascade_threshold, high_mv)
- [x] 5. Card definition — crates/engine/src/cards/defs/geological_appraiser.rs
- [x] 6. Game script — test-data/generated-scripts/etb-triggers/179_discover_geological_appraiser.json
- [x] 7. Coverage doc update — docs/mtg-engine-ability-coverage.md: Discover → validated
