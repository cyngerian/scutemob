# Ability WIP: Toxic

ability: Toxic
cr: 702.164
priority: P4
started: 2026-03-01
phase: closed
plan_file: memory/abilities/ability-plan-toxic.md

## Step Checklist
- [x] 1. Enum variant — types.rs:722, hash.rs:502, view_model.rs:715
- [x] 2. Rule enforcement — combat.rs:1118-1183 (DamageAppInfo), combat.rs:1291-1311 (Player arm)
- [x] 3. Trigger wiring (N/A — Toxic is static)
- [x] 4. Unit tests — crates/engine/tests/toxic.rs (8 tests)
- [x] 5. Card definition — crates/engine/src/cards/defs/pestilent_syphoner.rs (Pestilent Syphoner, {1}{B}, 1/1, Flying + Toxic 1)
- [x] 6. Game script — test-data/generated-scripts/combat/123_pestilent_syphoner_toxic_inline_poison.json
- [x] 7. Coverage doc update — docs/mtg-engine-ability-coverage.md (P4 16/88, 108 total validated, CR corrected to 702.164)

## Review
findings: 1 MEDIUM + 2 LOW
verdict: fixed
review_file: memory/abilities/ability-review-toxic.md

## Fix Log
- MEDIUM (Finding 1): combat.rs — replaced CardDefinition registry path for source_toxic_total
  with layer-resolved chars.keywords iteration (same pattern as deathtouch/lifelink/wither/infect)
- LOW (Finding 3): toxic.rs:296 — added .with_keyword(KeywordAbility::Toxic(1)) to ObjectSpec
  so both Toxic(2) and Toxic(1) appear in layer-resolved keywords; test_cumulative now exercises
  the correct path
- LOW (Finding 2): deferred (OrdSet same-N deduplication — no real-world card triggers this)
