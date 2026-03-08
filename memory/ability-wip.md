# Ability WIP: Blood Tokens

ability: Blood Tokens
cr: N/A (predefined token type)
priority: P4
started: 2026-03-08
phase: closed
plan_file: memory/abilities/ability-plan-blood-tokens.md

## Step Checklist
- [x] 1. Enum variant — `blood_token_spec()` in `crates/engine/src/cards/card_definition.rs:1557`; exported from `cards/mod.rs:17` and `lib.rs:9`
- [x] 2. Rule enforcement — `discard_card: bool` added to `ActivationCost` in `state/game_object.rs:106`; discard processing in `rules/abilities.rs:326`; hash in `state/hash.rs:1409`; `Command::ActivateAbility.discard_card: Option<ObjectId>` in `rules/command.rs:337`; engine.rs destructures new field; all construction sites updated (simulator, TUI, 15 test files); `cost_to_activation_cost` handles `Cost::DiscardCard`; harness `activate_ability` reads `discard_card_name`
- [x] 3. Trigger wiring — n/a (Blood tokens use existing token/ability infrastructure; no new trigger kinds needed)
- [x] 4. Unit tests — 14 tests in `crates/engine/tests/blood_tokens.rs`; all passing
- [x] 5. Card definition (voldaren_epicure.rs — {R} 1/1 Vampire; ETB create Blood token)
- [x] 6. Game script — `test-data/generated-scripts/baseline/script_190_blood_tokens.json`; harness PASS (5/5)
- [x] 7. Coverage doc update

## Review
findings: 3 LOW (no HIGH/MEDIUM)
verdict: clean
review_file: memory/abilities/ability-review-blood-tokens.md
