# Ability Review: Haunt

**Date**: 2026-03-08
**Reviewer**: ability-impl-reviewer (Opus)
**CR**: 702.55
**Files reviewed**:
- `crates/engine/src/state/types.rs` (KeywordAbility::Haunt, disc 142)
- `crates/engine/src/state/game_object.rs` (haunting_target field, TriggerEvent::HauntedCreatureDies)
- `crates/engine/src/state/hash.rs` (haunting_target hash, SOK disc 57/58, event disc 107, TriggerCondition disc 23, TriggerEvent disc 21)
- `crates/engine/src/state/stubs.rs` (PendingTriggerKind::HauntExile/HauntedCreatureDies, haunt_source fields)
- `crates/engine/src/state/stack.rs` (HauntExileTrigger SOK disc 57, HauntedCreatureDiesTrigger SOK disc 58)
- `crates/engine/src/state/mod.rs` (move_object_to_zone -- haunting_target: None)
- `crates/engine/src/state/builder.rs` (haunting_target: None)
- `crates/engine/src/rules/events.rs` (GameEvent::HauntExiled disc 107)
- `crates/engine/src/rules/abilities.rs` (CreatureDied handler: HauntExile + HauntedCreatureDies triggers; flush_pending_triggers SOK construction)
- `crates/engine/src/rules/resolution.rs` (HauntExileTrigger + HauntedCreatureDiesTrigger resolution; counter-spell catch-all)
- `crates/engine/src/cards/card_definition.rs` (TriggerCondition::HauntedCreatureDies)
- `crates/engine/src/effects/mod.rs` (token creation: haunting_target: None)
- `tools/replay-viewer/src/view_model.rs` (SOK + KW match arms)
- `tools/tui/src/play/panels/stack_view.rs` (SOK match arm)
- `crates/engine/tests/haunt.rs` (8 tests)

## Verdict: needs-fix

Implementation is structurally sound and covers the creature Haunt path (CR 702.55a creature path) correctly. The two-trigger architecture, exile-zone scanning, CR 400.7 identity handling, and multiplayer controller semantics are all correct. However, there is one MEDIUM finding related to the haunt card remaining in exile with a stale haunting_target after the haunted creature dies (allowing a spurious re-trigger if a new object reuses the ObjectId), and two LOW findings.

## Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | MEDIUM | `resolution.rs:4409-4414` | **Haunting_target not cleared after HauntedCreatureDiesTrigger resolves.** Haunt effect fires once per haunted-creature death, but the stale haunting_target persists on the exiled card. |
| 2 | LOW | `resolution.rs:4307-4319` | **MVP auto-target selects deterministically, not by controller choice.** CR 702.55a says "target creature" -- the haunt card's controller should choose. |
| 3 | LOW | `tests/haunt.rs:93-96` | **Test card uses GainLife(0) instead of a meaningful effect.** HauntedCreatureDiesTrigger resolution test does not verify the effect actually executed. |

### Finding Details

#### Finding 1: Haunting_target not cleared after HauntedCreatureDiesTrigger resolves

**Severity**: MEDIUM
**File**: `crates/engine/src/rules/resolution.rs:4409-4416`
**CR Rule**: 702.55c -- "Triggered abilities of cards with haunt that refer to the haunted creature can trigger in the exile zone."
**Issue**: After the HauntedCreatureDiesTrigger resolves successfully (lines 4409-4414), the haunt card stays in exile (correct per CR) but retains its `haunting_target` field pointing to the now-dead creature's old ObjectId. This is stated as "harmless" in the plan, but it creates a concrete risk: if a new object is ever assigned the same ObjectId (ObjectId recycling), the scan at abilities.rs:4477-4496 would spuriously fire a second HauntedCreatureDiesTrigger for an unrelated creature's death. While ObjectId recycling is unlikely in the current im-rs implementation (monotonic counter), this is an unnecessary correctness risk.

Additionally, per CR 702.55 the haunt effect should fire exactly once -- when the haunted creature dies. The haunt card should remain in exile but the haunting relationship is consumed. Clearing `haunting_target` to `None` after the trigger fires is the correct behavior.

**Fix**: After executing the haunt effect (line 4414), clear `haunting_target` on the exiled haunt card:
```rust
if let Some(haunt_obj) = state.objects.get_mut(&haunt_source) {
    haunt_obj.haunting_target = None;
}
```

#### Finding 2: MVP auto-target selects deterministically

**Severity**: LOW
**File**: `crates/engine/src/rules/resolution.rs:4307-4319`
**CR Rule**: 702.55a -- "exile it haunting target creature"
**Issue**: The HauntExileTrigger resolution auto-selects the first creature returned by `.iter().next()`. This is documented as MVP behavior and is acceptable for now, but the selection is HashMap-iteration-order-dependent (non-deterministic across runs). For replay determinism, a stable ordering would be preferable.
**Fix**: Deferred -- document as a known simplification. When interactive target selection is implemented, this becomes moot.

#### Finding 3: Test effect is no-op, cannot verify execution

**Severity**: LOW
**File**: `crates/engine/tests/haunt.rs:93-96`
**CR Rule**: 702.55c
**Issue**: The test card definition uses `Effect::GainLife { amount: Fixed(0) }` as the haunt effect. This means the full lifecycle test (test_haunt_full_lifecycle) cannot verify that the effect actually executed -- gaining 0 life produces no observable state change. The test only verifies that the trigger goes on the stack and resolves without errors, not that the effect fires.
**Fix**: Change the test card's haunt effect to `Effect::GainLife { player: PlayerTarget::Controller, amount: EffectAmount::Fixed(2) }` and assert P1's life total increased by 2 after HauntedCreatureDiesTrigger resolution in the full lifecycle test.

## CR Coverage Check

| CR Subrule | Implemented? | Tested? | Notes |
|------------|-------------|---------|-------|
| 702.55a (creature path) | Yes | Yes | test_haunt_creature_dies_puts_haunt_exile_trigger_on_stack, test_haunt_exile_trigger_resolution_exiles_card_with_haunting_target |
| 702.55a (spell path) | No | No | Documented as stretch goal in plan; out of scope for creature-only MVP |
| 702.55b (haunting relationship) | Yes | Yes | haunting_target field set on exiled card; test_haunt_exile_trigger_resolution checks haunting_target == target_id |
| 702.55b ("regardless of creature type") | Partial | No | No test for haunted object losing creature type; implementation is correct by default (haunting_target is ObjectId, not filtered by type) |
| 702.55c (trigger from exile) | Yes | Yes | test_haunt_haunted_creature_dies_fires_trigger_from_exile |
| No creatures fizzle | Yes | Yes | test_haunt_no_creatures_available_fizzles |
| Card removed from exile | Yes | Yes | test_haunt_card_removed_from_exile_no_trigger |
| Direct exile no trigger | Yes | Yes | test_haunt_creature_exiled_directly_does_not_trigger_haunt |
| Full lifecycle | Yes | Yes | test_haunt_full_lifecycle |
| Multiplayer controller | Yes | Yes | test_haunt_multiplayer_controller_of_trigger |
| Multiple haunt on same creature | No | No | Missing test; implementation supports it (scan finds all matching exiled cards) |
| haunting_target cleared after trigger | No (Finding 1) | No | MEDIUM -- should clear after trigger fires |
| Object identity (CR 400.7) | Yes | Yes | move_object_to_zone resets haunting_target; pre_death_object_id used for matching |
