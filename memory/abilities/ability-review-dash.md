# Ability Review: Dash

**Date**: 2026-03-01
**Reviewer**: ability-impl-reviewer (Opus)
**CR**: 702.109
**Files reviewed**:
- `crates/engine/src/state/types.rs` (KeywordAbility::Dash)
- `crates/engine/src/cards/card_definition.rs` (AbilityDefinition::Dash)
- `crates/engine/src/state/stack.rs` (StackObjectKind::DashReturnTrigger, StackObject.was_dashed)
- `crates/engine/src/state/game_object.rs` (GameObject.was_dashed)
- `crates/engine/src/state/stubs.rs` (PendingTrigger.is_dash_return_trigger)
- `crates/engine/src/state/hash.rs` (discriminants 95/31/28, field hashing)
- `crates/engine/src/state/builder.rs` (was_dashed init)
- `crates/engine/src/state/mod.rs` (was_dashed zone-change reset)
- `crates/engine/src/effects/mod.rs` (was_dashed in create_base_token)
- `crates/engine/src/rules/command.rs` (CastSpell.cast_with_dash)
- `crates/engine/src/rules/engine.rs` (dispatch)
- `crates/engine/src/rules/casting.rs` (validation, get_dash_cost, cost selection, StackObject.was_dashed)
- `crates/engine/src/rules/resolution.rs` (was_dashed ETB transfer, haste grant, DashReturnTrigger arm, fizzle arm)
- `crates/engine/src/rules/turn_actions.rs` (end_step_actions dash scan)
- `crates/engine/src/rules/abilities.rs` (flush_pending_triggers DashReturnTrigger, is_dash_return_trigger at all sites)
- `crates/engine/src/rules/copy.rs` (was_dashed: false on copies)
- `crates/engine/src/testing/replay_harness.rs` (cast_spell_dash action, cast_with_dash: false at all sites)
- `tools/replay-viewer/src/view_model.rs` (DashReturnTrigger match arm)
- `tools/tui/src/play/panels/stack_view.rs` (DashReturnTrigger match arm)
- `crates/engine/tests/dash.rs` (7 tests)

## Verdict: clean

The Dash implementation is correct and thorough. All CR 702.109 subrules are faithfully
implemented. The alternative cost path, haste grant, delayed return trigger, zone-change
reset, copy exclusion, and commander tax interaction all match the CR text. Hash
discriminants are present for all new types and fields. All PendingTrigger construction
sites across the codebase include `is_dash_return_trigger: false`. The test suite covers
the core lifecycle, negative cases, mutual exclusion, and commander tax. Two test gaps
are noted (both LOW) -- the "creature dies after trigger queued but before resolution"
scenario and the mana value unchanged test from the plan. No HIGH or MEDIUM findings.

## Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | LOW | `tests/dash.rs` | **Missing test: creature removed after trigger queued.** Plan test 3 specified testing when creature dies before end-step trigger resolves (not just before end step). Current test 4 only covers creature leaving before end step. **Fix:** Add a test where the creature is on the battlefield at end step (trigger gets queued), then is removed before the trigger resolves, and verify the trigger does nothing. |
| 2 | LOW | `tests/dash.rs` | **Missing test: mana value unchanged (CR 118.9c).** Plan test 6 specified verifying the spell's mana value on the stack is the original mana cost, not the dash cost. Not implemented in the 7 tests. **Fix:** Add a test that casts with dash and checks `stack_objects[0].mana_cost` equals the card's original mana cost, not the dash cost. |
| 3 | LOW | `tests/dash.rs:534` | **Test 5 title/intent mismatch.** Named `test_dash_alternative_cost_exclusivity_with_flashback` but actually tests casting a non-dash card with `cast_with_dash: true`. Does not test the flashback+dash combination. The evoke+dash mutual exclusion is tested separately in test 6. **Fix:** Either rename to `test_dash_rejected_for_card_without_dash_ability` or add a true flashback+dash exclusivity assertion (would require a card in graveyard with both abilities). |
| 4 | LOW | `resolution.rs:290-292` | **Haste grant is baked into base characteristics, not modeled as a continuous effect.** CR 702.109a says "As long as this permanent's dash cost was paid, it has haste" -- this is a static ability that should function through the layer system. The current approach inserts Haste into base keywords at resolution time. This works correctly in the common case and under Humility (Humility removes abilities in Layer 6, base haste is correctly removed; if Humility leaves, base haste reappears). However, an effect that specifically removes haste without removing the Dash ability (extremely niche) would not be re-granted. This matches the existing Suspend pattern (also bakes in haste) and is acceptable as a V1 simplification. **Fix:** No action needed. Document as a known simplification if a future layer refactor touches this area. |

### Finding Details

#### Finding 1: Missing test -- creature removed after trigger queued

**Severity**: LOW
**File**: `crates/engine/tests/dash.rs`
**CR Rule**: 702.109a / CR 400.7 -- "return the permanent this spell becomes to its owner's hand at the beginning of the next end step"
**Issue**: The plan specified 9 tests; the implementation has 7. One gap is the scenario where the DashReturnTrigger is already on the stack (creature was on the battlefield when end_step_actions fired), but then the creature is removed before the trigger resolves. The resolution code at `resolution.rs:1020-1025` correctly handles this (checks `obj.zone == ZoneId::Battlefield`), but no test exercises this path. Test 4 only covers the creature leaving BEFORE end step (so no trigger is queued at all).
**Fix**: Add a test that: (1) casts with dash, (2) advances to end step so the trigger is queued, (3) manually removes the creature from the battlefield while the trigger is on the stack, (4) resolves the trigger, (5) asserts the creature is NOT returned to hand.

#### Finding 2: Missing test -- mana value unchanged

**Severity**: LOW
**File**: `crates/engine/tests/dash.rs`
**CR Rule**: 118.9c -- "An alternative cost doesn't change a spell's mana cost, only what its controller has to pay to cast it."
**Issue**: The plan specified `test_dash_mana_value_unchanged` (test 6 in the plan). This test was not implemented. The dash cost selection code in `casting.rs:851-854` correctly uses `get_dash_cost` to determine the payment amount while the spell's `mana_cost` field on the StackObject retains the original value. However, this is untested.
**Fix**: Add a test that casts Goblin Raider with dash ({R}), then checks `state.stack_objects[0].mana_cost` still equals `ManaCost { generic: 1, red: 1, .. }` (the original {1}{R}, not the dash {R}).

#### Finding 3: Test 5 title/intent mismatch

**Severity**: LOW
**File**: `crates/engine/tests/dash.rs:534`
**CR Rule**: 118.9a -- "Only one alternative cost can be applied to any one spell as it's being cast."
**Issue**: The test is named `test_dash_alternative_cost_exclusivity_with_flashback` but does not actually test combining dash with flashback. Instead, it tests that a card without the Dash ability cannot be cast with `cast_with_dash: true`. This is a valid test (validates `get_dash_cost` returning None), but the name is misleading. The actual dash+evoke exclusivity is tested in test 6.
**Fix**: Rename to `test_dash_rejected_for_card_without_dash_ability` for accuracy. Optionally, add a separate test that creates a card with both Dash and Flashback keywords (in graveyard) and attempts to cast with both `cast_with_dash: true` and flashback -- should be rejected with CR 118.9a error.

#### Finding 4: Haste grant as base characteristic

**Severity**: LOW
**File**: `crates/engine/src/rules/resolution.rs:290-292`
**CR Rule**: 702.109a -- "As long as this permanent's dash cost was paid, it has haste."
**Issue**: The implementation inserts `KeywordAbility::Haste` directly into the object's `characteristics.keywords` at resolution time. Per CR 702.109a, this should be a static ability that continuously grants haste as long as `was_dashed` is true. The implementation works correctly for all practical scenarios including Humility interaction (Layer 6 removes base haste, Humility removal restores it). The only theoretical gap is an effect that removes haste specifically without removing the Dash ability -- in this case, the haste should be re-granted by the static ability but won't be under the current approach. This is a known V1 simplification consistent with the existing Suspend haste implementation.
**Fix**: No action needed. This is an extremely niche edge case with no commonly played cards that would exercise it.

## CR Coverage Check

| CR Subrule | Implemented? | Tested? | Notes |
|------------|-------------|---------|-------|
| 702.109a -- alternative cost cast | Yes | Yes | test_dash_basic_cast_with_dash_cost |
| 702.109a -- haste grant | Yes | Yes | test_dash_basic_cast_with_dash_cost (asserts Haste keyword) |
| 702.109a -- return to hand at end step | Yes | Yes | test_dash_return_to_hand_at_end_step |
| 702.109a -- was_dashed flag transfer | Yes | Yes | test_dash_basic_cast_with_dash_cost (asserts was_dashed on permanent) |
| 608.3g -- delayed trigger creation timing | Yes | Yes | Trigger queued by end_step_actions, not at ETB |
| 118.9a -- only one alternative cost | Yes | Partial | test_dash_cannot_combine_with_evoke; other combos enforced but only evoke tested |
| 118.9c -- mana value unchanged | Yes | No | Finding 2: cost selection uses dash cost but mana_cost field is unchanged |
| 118.9d -- commander tax on top | Yes | Yes | test_dash_commander_tax_applies |
| 400.7 -- zone change resets | Yes | Implicit | move_object_to_zone resets was_dashed; tested indirectly via test 4 |
| 400.7 -- creature left before trigger | Yes | Yes | test_dash_creature_left_battlefield_before_end_step |
| Ruling -- normal cast no return | Yes | Yes | test_dash_normal_cast_no_return |
| Ruling -- copies don't inherit | Yes (copy.rs) | No | was_dashed: false in copy paths; plan test 9 not implemented |
| Ruling -- no forced attack | N/A | N/A | No enforcement needed (absence of attack requirement is correct) |
| Ruling -- cost reduction applies | N/A | N/A | Inherent from alternative cost framework |
| Multiplayer -- dash works for N | Yes | Partial | end_step_actions scans all objects; commander tax test uses 2-player |
