# Ability WIP: Offspring

ability: Offspring
cr: 702.175
priority: P4
started: 2026-03-07
phase: closed
plan_file: memory/abilities/ability-plan-offspring.md

## Step Checklist
- [x] 1. Enum variant — types.rs:1274, card_definition.rs:627, stack.rs:1143, stubs.rs:123, game_object.rs:643, state/mod.rs:381+542, builder.rs:996, hash.rs (all 4 sites)
- [x] 2. Rule enforcement — casting.rs:1966 (offspring cost validation), casting.rs:5249 (get_offspring_cost helper), engine.rs:109 (offspring_paid destructured), command.rs:305 (offspring_paid field)
- [x] 3. Trigger wiring — resolution.rs:442 (transfer stack→GO), resolution.rs:1357 (OffspringETB queue), resolution.rs:3565 (OffspringTrigger resolution), abilities.rs:5737 (flush OffspringETB), replay_harness.rs (cast_spell_offspring action)
- [x] 4. Unit tests — crates/engine/tests/offspring.rs (6 tests: not_paid, basic_paid, token_is_1_1, rejected_without_keyword, tokens_not_cast, source_leaves_still_creates_token)
- [x] 5. Card definition
- [x] 6. Game script
- [x] 7. Coverage doc update

## Review
findings: 2 (1 MEDIUM, 1 LOW)
verdict: needs-fix
review_file: memory/abilities/ability-review-offspring.md

## Fix Phase
- [x] MEDIUM: resolution.rs Layer 7b site — added TODO comment documenting CR 707.9b deviation and proper fix path
- [x] MEDIUM (doc test): offspring.rs — added `test_offspring_token_pt_is_layer7b_known_deviation` documenting the copiable-values gap
- LOW: offspring.rs CDA interaction test — deferred per review (finding 2 is LOW)
