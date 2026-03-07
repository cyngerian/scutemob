# Ability WIP: Scavenge

ability: Scavenge
cr: 702.97
priority: P4
started: 2026-03-06
phase: closed

## Review
findings: 1 (0 HIGH, 0 MEDIUM, 1 LOW)
review_file: memory/abilities/ability-review-scavenge.md
plan_file: memory/abilities/ability-plan-scavenge.md

## Step Checklist
- [x] 1. Enum variant — `KeywordAbility::Scavenge` (KW 120) in `state/types.rs:1112`; `AbilityDefinition::Scavenge` (AbilDef 47) in `cards/card_definition.rs:517`; `StackObjectKind::ScavengeAbility` (SOK 45) in `state/stack.rs:978`; hash arms in `state/hash.rs`; TUI `tools/tui/src/play/panels/stack_view.rs` + replay-viewer `tools/replay-viewer/src/view_model.rs` arms added
- [x] 2. Rule enforcement — `Command::ScavengeCard` in `rules/command.rs:542`; handler in `rules/engine.rs:430`; `handle_scavenge_card()` + `get_scavenge_cost()` in `rules/abilities.rs:4832`; resolution in `rules/resolution.rs:2313`; harness action `scavenge_card` in `testing/replay_harness.rs`; `target_creature` field added to `testing/script_schema.rs`
- [x] 3. Trigger wiring — n/a (activated ability, not triggered; SOK/resolution wiring done in step 2)
- [x] 4. Unit tests — `crates/engine/tests/scavenge.rs` — 10 tests, all passing
- [x] 5. Card definition — crates/engine/src/cards/defs/deadbridge_goliath.rs
- [x] 6. Game script — test-data/generated-scripts/stack/157_scavenge_deadbridge_goliath.json
- [x] 7. Coverage doc update — docs/mtg-engine-ability-coverage.md (Scavenge: validated)
