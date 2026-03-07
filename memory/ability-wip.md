# Ability WIP: Outlast

ability: Outlast
cr: 702.107
priority: P4
started: 2026-03-06
phase: closed

## Review
findings: 2 (0 HIGH, 0 MEDIUM, 2 LOW)
review_file: memory/abilities/ability-review-outlast.md
plan_file: memory/abilities/ability-plan-outlast.md

## Step Checklist
- [x] 1. Enum variant — types.rs:1112 (KeywordAbility::Outlast, disc 121), card_definition.rs:526 (AbilityDefinition::Outlast, disc 48), hash.rs:612+3628
- [x] 2. Rule enforcement — replay_harness.rs:1711 (enrich_spec_from_def Outlast expansion: tap+mana+sorcery_speed+AddCounter)
- [x] 3. Trigger wiring — N/A (pure activated ability, no triggers)
- [x] 4. Unit tests — tests/outlast.rs (7 tests: basic, sorcery_speed, summoning_sickness, requires_mana, already_tapped, stacks_counters, not_a_cast)
- [x] 5. Card definition — crates/engine/src/cards/defs/ainok_bond_kin.rs
- [x] 6. Game script — test-data/generated-scripts/stack/158_outlast_ainok_bond_kin.json
- [x] 7. Coverage doc update — docs/mtg-engine-ability-coverage.md (Outlast: validated)
