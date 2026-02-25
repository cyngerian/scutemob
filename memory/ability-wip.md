# Ability WIP: Ward

ability: Ward
cr: 702.21
priority: P1
started: 2026-02-24
phase: closed
plan_file: memory/ability-plan-ward.md

## Step Checklist
- [x] 1. Enum variant — `state/types.rs:135` Ward(u32), hash `state/hash.rs:287`, display `tools/replay-viewer/src/view_model.rs:585`
- [x] 2. Rule enforcement — `TriggerEvent::SelfBecomesTargetByOpponent` in `game_object.rs:118`; `TriggerCondition::WhenBecomesTargetByOpponent` in `card_definition.rs:509`; `GameEvent::PermanentTargeted` in `events.rs:605`; hash updated in `hash.rs`
- [x] 3. Trigger wiring — `PendingTrigger.targeting_stack_id` in `stubs.rs:49`; `check_triggers` handler in `abilities.rs`; `flush_pending_triggers` sets targets in `abilities.rs`; `PermanentTargeted` emitted in `casting.rs` and `abilities.rs`; Ward→TriggeredAbilityDef in `builder.rs:336`; CounterSpell fixed in `effects/mod.rs`; DeclaredTarget stack check in `effects/mod.rs`
- [x] 4. Unit tests — `crates/engine/tests/ward.rs` (7 tests, all pass)
- [x] 5. Card definition — Adrix and Nev, Twincasters (`cards/definitions.rs:1639`); Ward {2} encoded; token-doubling deferred
- [x] 6. Game script — `test-data/generated-scripts/stack/055_ward_counters_lightning_bolt.json` (8/8 assertions pass)
- [x] 7. Coverage doc update — Ward row: partial → validated; P1 validated 27→28

## Review
findings: 5 (2 MEDIUM, 3 LOW)
review_file: memory/ability-review-ward.md

## Fix Phase Results
- Finding 1 (MEDIUM) fixed: `effects/mod.rs` PlayerTarget::ControllerOf now checks stack_objects; `builder.rs:358` payer changed to ControllerOf(DeclaredTarget{index:0})
- Finding 2 (MEDIUM) fixed: `tests/ward.rs:590` removed spurious .clone() so ward trigger resolves on real state
- Findings 3, 4, 5 (LOW): deferred per plan
