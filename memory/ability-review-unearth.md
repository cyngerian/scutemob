# Ability Review: Unearth

**Date**: 2026-02-27
**Reviewer**: ability-impl-reviewer (Opus)
**CR**: 702.84
**Files reviewed**:
- `crates/engine/src/state/types.rs` (KeywordAbility::Unearth)
- `crates/engine/src/cards/card_definition.rs` (AbilityDefinition::Unearth)
- `crates/engine/src/state/game_object.rs` (was_unearthed field)
- `crates/engine/src/state/stack.rs` (UnearthAbility, UnearthTrigger)
- `crates/engine/src/state/stubs.rs` (is_unearth_trigger on PendingTrigger)
- `crates/engine/src/state/hash.rs` (all hash impls)
- `crates/engine/src/state/mod.rs` (move_object_to_zone -- was_unearthed reset)
- `crates/engine/src/state/builder.rs` (was_unearthed default)
- `crates/engine/src/rules/command.rs` (Command::UnearthCard)
- `crates/engine/src/rules/engine.rs` (UnearthCard dispatch)
- `crates/engine/src/rules/abilities.rs` (handle_unearth_card, get_unearth_cost, flush dispatch)
- `crates/engine/src/rules/resolution.rs` (UnearthAbility + UnearthTrigger resolution, counter arms)
- `crates/engine/src/rules/turn_actions.rs` (end_step_actions)
- `crates/engine/src/rules/replacement.rs` (was_unearthed zone-change replacement)
- `crates/engine/src/testing/replay_harness.rs` (unearth_card action)
- `tools/replay-viewer/src/view_model.rs` (UnearthAbility, UnearthTrigger arms)
- `crates/engine/tests/unearth.rs` (12 tests)

## Verdict: needs-fix

The implementation is structurally sound and covers CR 702.84a thoroughly. The UnearthAbility and UnearthTrigger stack kinds, the was_unearthed flag, the zone-change replacement, the delayed trigger wiring, and the sorcery-speed enforcement are all correct. However, there is one HIGH finding (missing hash field on PendingTrigger) and one MEDIUM finding (silent cost bypass when card definition is missing or has no Unearth cost). Both must be fixed before closing.

## Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | **HIGH** | `state/hash.rs:978` | **Missing hash: is_unearth_trigger on PendingTrigger.** The field is not hashed. **Fix:** Add `self.is_unearth_trigger.hash_into(hasher);` after the miracle fields. |
| 2 | MEDIUM | `rules/abilities.rs:676` | **Silent cost bypass when get_unearth_cost returns None.** If the CardRegistry has no AbilityDefinition::Unearth, the ability activates for free. **Fix:** Return an error when `unearth_cost` is `None`. |
| 3 | LOW | `rules/replacement.rs:586` | **Sentinel ReplacementId(u64::MAX) reused.** Previously flagged as MR-M8-05. Acceptable for now; document it. |
| 4 | LOW | `tests/unearth.rs` | **Missing "delayed trigger countered" test.** Plan test #11 was replaced with mana payment test. Acceptable: Stifle mechanism not yet implemented. |
| 5 | LOW | `tests/unearth.rs:852` | **Redundant test: test_unearth_creature_has_haste overlaps test_unearth_basic_return_to_battlefield.** Test 1 already asserts haste and was_unearthed. |

### Finding Details

#### Finding 1: Missing hash field -- is_unearth_trigger on PendingTrigger

**Severity**: HIGH
**File**: `crates/engine/src/state/hash.rs:978`
**CR Rule**: Architecture invariant -- hash coverage for all fields
**Issue**: The `PendingTrigger::is_unearth_trigger` field (added in `stubs.rs:133`) is not included in the `HashInto for PendingTrigger` implementation. The hash function ends at line 977 after `self.miracle_cost.hash_into(hasher)` and does not include the new `is_unearth_trigger` boolean. This means two PendingTrigger instances that differ only in `is_unearth_trigger` will hash identically, breaking the deterministic state verification system. Per CLAUDE.md architecture invariants: all new fields MUST have hash coverage.
**Fix**: Add `self.is_unearth_trigger.hash_into(hasher);` as the last line of the `HashInto for PendingTrigger` impl, before the closing brace at line 978. Add a comment: `// CR 702.84a: is_unearth_trigger -- unearth delayed exile trigger marker`.

#### Finding 2: Silent cost bypass when get_unearth_cost returns None

**Severity**: MEDIUM
**File**: `crates/engine/src/rules/abilities.rs:676`
**CR Rule**: 702.84a -- "Unearth [cost]" requires paying the cost; 602.2b -- activation costs must be paid
**Issue**: If `get_unearth_cost()` returns `None` (card has no `AbilityDefinition::Unearth { cost }` in the registry, or no CardId), the `if let Some(ref cost) = unearth_cost` block at line 676 is skipped entirely, and the ability proceeds to the stack without paying any mana. While the architecture invariant (#9) says every card must have a definition, defensive coding requires erroring here. Step 4 already verified the keyword is present, but the keyword marker (`KeywordAbility::Unearth`) and the cost definition (`AbilityDefinition::Unearth { cost }`) are separate -- one could exist without the other if a card definition is incomplete.
**Fix**: After line 672 (`let unearth_cost = get_unearth_cost(...)`), add a guard:
```rust
let unearth_cost = match unearth_cost {
    Some(cost) => cost,
    None => {
        return Err(GameStateError::InvalidCommand(
            "UnearthCard: no unearth cost found in card definition (CR 702.84a)".into(),
        ));
    }
};
```
Then simplify lines 676-688 to use `unearth_cost` directly (no `Option` wrapping).

#### Finding 3: Sentinel ReplacementId(u64::MAX) reused

**Severity**: LOW
**File**: `crates/engine/src/rules/replacement.rs:586`
**CR Rule**: N/A -- code quality (MR-M8-05 prior finding)
**Issue**: The unearth zone-change replacement uses `ReplacementId(u64::MAX)` as a sentinel value for the `applied_id` and `effect_id` fields. This pattern was previously flagged in MR-M8-05, which introduced `CommanderZoneRedirect` as a proper variant to avoid the sentinel. Since the unearth replacement fires as an early return before the registered-effect pipeline, the sentinel doesn't cause correctness issues, but it is inconsistent with the established fix pattern. Acceptable to defer.
**Fix**: When future refactoring addresses zone-change replacement architecture, introduce a proper `UnearthExileRedirect` event variant analogous to `CommanderZoneRedirect`.

#### Finding 4: Missing "delayed trigger countered" test

**Severity**: LOW
**File**: `crates/engine/tests/unearth.rs`
**CR Rule**: 702.84a ruling -- delayed trigger can be countered by Stifle; replacement effect persists independently
**Issue**: The plan specified test #11 as "test_unearth_delayed_trigger_countered" to verify that countering the end-step delayed trigger does not remove the replacement effect. The implementation replaced this with `test_unearth_requires_mana_payment` (mana validation). The substitution is reasonable since the engine lacks a general "counter triggered ability" mechanism (Stifle is not yet implemented). The mana payment test is useful. However, the interaction documented in the ruling (countered trigger + persistent replacement) remains untested.
**Fix**: Add this test when Stifle or a general triggered-ability-counter mechanism is implemented. No action needed now.

#### Finding 5: Redundant test -- haste check duplicated

**Severity**: LOW
**File**: `crates/engine/tests/unearth.rs:852`
**CR Rule**: N/A -- test quality
**Issue**: `test_unearth_creature_has_haste` (test 9) performs the same setup as `test_unearth_basic_return_to_battlefield` (test 1) and checks the same assertions (haste keyword present, was_unearthed flag set). Test 1 already verifies both at lines 195-206. Test 9 adds no new coverage. This is minor duplication, not a correctness issue.
**Fix**: No action required. The test is harmless and provides focused naming for the haste check, which aids in locating failures.

## CR Coverage Check

| CR Subrule | Implemented? | Tested? | Notes |
|------------|-------------|---------|-------|
| 702.84a: activated ability from graveyard | Yes | Yes | test 1 (basic), test 2 (zone check) |
| 702.84a: "[Cost]: Return this card" | Yes | Yes | test 1 (activation), test 12 (insufficient mana) |
| 702.84a: "It gains haste" | Yes | Yes | test 1, test 9 |
| 702.84a: "Exile at beginning of next end step" | Yes | Yes | test 3 (end-step exile via delayed trigger) |
| 702.84a: "If it would leave the battlefield, exile instead" | Yes | Yes | test 4 (bounce), test 5 (destroy), test 6 (actual exile not replaced) |
| 702.84a: "Activate only as a sorcery" | Yes | Yes | test 2 (combat, opponent turn, wrong zone), test 11 (multiplayer) |
| Ruling: card removed before resolution | Yes | Yes | test 7 (graveyard emptied before resolve) |
| Ruling: not a cast | Yes | Yes | test 8 (no SpellCast, no spells_cast increment) |
| Ruling: exile effects not abilities on creature | Yes | Yes | test 10 (keywords cleared, replacement still fires) |
| Ruling: delayed trigger countered | Partial | No | Engine lacks Stifle mechanism; replacement path tested independently |
| Ruling: flicker resets was_unearthed | Yes | No | Covered by CR 400.7 in move_object_to_zone (was_unearthed: false) |
| Ruling: multiple unearth abilities | Yes | No | Handled implicitly -- get_unearth_cost finds first; multiple activations work |
| CR 400.7: zone change = new object | Yes | Yes | Verified in state/mod.rs lines 289, 370 |
| CR 702.61a: split second blocks activation | Yes | No | Code present in handle_unearth_card line 612; no test |

## Previous Findings (re-review only)

N/A -- first review.
