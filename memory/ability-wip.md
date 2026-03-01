# Ability WIP: Aftermath

ability: Aftermath
cr: 702.127
priority: P4
started: 2026-03-01
phase: closed
plan_file: memory/abilities/ability-plan-aftermath.md

## Step Checklist
- [x] 1. Enum variant — KeywordAbility::Aftermath=91, AbilityDefinition::Aftermath disc 24, StackObject::cast_with_aftermath
- [x] 2. Rule enforcement — casting.rs zone/cost/exile; resolution.rs effect selection; harness fix for split card names
- [x] 3. Trigger wiring — n/a
- [x] 4. Unit tests — aftermath.rs (12 tests)
- [x] 5. Card definition — cut_ribbons.rs
- [x] 6. Game script — 128_aftermath_cut_ribbons.json (validated)
- [x] 7. Coverage doc update — P4 validated 21→22, total 113→114

## Review
findings: 0 HIGH + 0 MEDIUM + 3 LOW
verdict: clean
review_file: memory/abilities/ability-review-aftermath.md
