# Ability WIP: Bolster

ability: Bolster
cr: 701.39
priority: P3
started: 2026-02-28
phase: closed
plan_file: memory/abilities/ability-plan-bolster.md

## Step Checklist
- [x] 1. Enum variant — `Effect::Bolster` in `crates/engine/src/cards/card_definition.rs:388`; hash in `crates/engine/src/state/hash.rs:2695`
- [x] 2. Rule enforcement — execution arm in `crates/engine/src/effects/mod.rs:971` (Counters section)
- [x] 3. Trigger wiring — N/A (Bolster is an effect payload, not a triggered keyword)
- [x] 4. Unit tests — `crates/engine/tests/bolster.rs` (8 tests, all passing)
- [x] 5. Card definition — `crates/engine/src/cards/defs/cached_defenses.rs`
- [x] 6. Game script — `test-data/generated-scripts/baseline/104_cached_defenses_bolster.json`
- [x] 7. Coverage doc update

## Review
findings: 2 MEDIUM, 1 LOW — MEDIUM fixes applied; LOW deferred
review_file: memory/abilities/ability-review-bolster.md
