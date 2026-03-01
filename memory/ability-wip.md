# Ability WIP: Ninjutsu

ability: Ninjutsu
cr: 702.49
priority: P4
started: 2026-03-01
phase: closed
plan_file: memory/abilities/ability-plan-ninjutsu.md

## Step Checklist
- [x] 1. Enum variant — types.rs:745-760, card_definition.rs:264-279, hash.rs (KeywordAbility 87/88, AbilityDefinition 22/23), view_model.rs:719-720
- [x] 2. Rule enforcement — command.rs:ActivateNinjutsu, abilities.rs:handle_ninjutsu+get_ninjutsu_cost, resolution.rs:NinjutsuAbility arm, stack.rs:NinjutsuAbility, hash.rs:discriminant 26
- [x] 3. Trigger wiring — n/a (ninjutsu is an activated ability, not a triggered ability; ETB triggers are wired via fire_when_enters_triggered_effects in resolution)
- [x] 4. Unit tests — tests/ninjutsu.rs (12 tests: all pass)
- [x] 5. Card definition — crates/engine/src/cards/defs/ninja_of_the_deep_hours.rs (Ninjutsu {1}{U}, {3}{U} 1/1 Human Ninja)
- [x] 6. Game script — test-data/generated-scripts/combat/125_ninjutsu_ninja_of_the_deep_hours.json
- [x] 7. Coverage doc update — docs/mtg-engine-ability-coverage.md (P4 19/88 now +Commander Ninjutsu, 111 total validated)

## Review
findings: 2 HIGH + 2 LOW
verdict: needs-fix → fixed
review_file: memory/abilities/ability-review-ninjutsu.md

## Fix Phase
- [x] Finding 1 (HIGH): abilities.rs:910 — replaced `.unwrap()` with `.ok_or_else(|| GameStateError::InvalidCommand(...))?`
- [x] Finding 2 (HIGH): abilities.rs:929,933 — replaced `.unwrap().expect()` with `.and_then(...).ok_or_else(|| GameStateError::InvalidCommand(...))?`
- [ ] Finding 3 (LOW): deferred — reveal tracking
- [ ] Finding 4 (LOW): deferred — missing negative test for wrong-controller case
