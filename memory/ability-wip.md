# Ability WIP: Retrace

ability: Retrace
cr: 702.81
priority: P4
started: 2026-03-01
phase: closed
plan_file: memory/abilities/ability-plan-retrace.md

## Step Checklist
- [x] 1. Enum variant — types.rs:761 (KeywordAbility::Retrace), hash.rs:513 (discriminant 89)
- [x] 2. Rule enforcement — command.rs:167 (retrace_discard_land field), casting.rs (detection + validation + cost payment), engine.rs (destructure + pass-through)
- [x] 3. Trigger wiring — n/a (Retrace is a static ability with no triggers)
- [x] 4. Unit tests — retrace.rs (11 tests: basic cast, graveyard return on resolve, graveyard return when countered, sorcery timing, discard-must-be-land, discard-must-be-in-hand, no-keyword-no-cast, normal-mana-cost, no-land-provided, hand-cast, recast-after-resolution)
- [x] 5. Card definition — crates/engine/src/cards/defs/flame_jab.rs (Flame Jab, {R} Sorcery, deal 1 damage, Retrace)
- [x] 6. Game script — test-data/generated-scripts/combat/126_retrace_flame_jab.json
- [x] 7. Coverage doc update — P4 validated 19→20, total 111→112

## Review
findings: 1 HIGH + 1 MEDIUM + 3 LOW
verdict: needs-fix → fixed
review_file: memory/abilities/ability-review-retrace.md

## Fix Phase
- [x] Finding 1 (HIGH): casting.rs:721 — replace .expect() with .ok_or_else()
- [x] Finding 2 (MEDIUM): casting.rs escape detection — add && !casting_with_retrace
- [ ] Finding 3 (LOW): deferred
- [ ] Finding 4 (LOW): deferred
- [ ] Finding 5 (LOW): deferred
