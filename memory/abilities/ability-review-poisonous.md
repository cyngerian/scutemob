# Ability Review: Poisonous

**Date**: 2026-03-01
**Reviewer**: ability-impl-reviewer (Opus)
**CR**: 702.70
**Files reviewed**:
- `crates/engine/src/state/types.rs` (lines 712-722)
- `crates/engine/src/state/hash.rs` (lines 497-501, 1480-1490, 1142-1145)
- `crates/engine/src/state/stubs.rs` (lines 307-325)
- `crates/engine/src/state/stack.rs` (lines 534-558)
- `crates/engine/src/state/builder.rs` (lines 532-536)
- `crates/engine/src/rules/abilities.rs` (lines 2336-2431, 2890-2899, all `is_poisonous_trigger: false` sites)
- `crates/engine/src/rules/resolution.rs` (lines 1967-2003, 2117)
- `tools/replay-viewer/src/view_model.rs` (lines 491-493, 714)
- `tools/tui/src/play/panels/stack_view.rs` (lines 98-100)
- `crates/engine/tests/poisonous.rs` (all 706 lines, 6 tests)

## Verdict: clean

The implementation correctly encodes all of CR 702.70 (both subrules a and b). The trigger dispatch, flush, and resolution logic follow the established Ingest/Renown pattern with no deviations. All PendingTrigger construction sites across 7 files include the new fields. Hash coverage is complete for both `KeywordAbility::Poisonous(u32)` and `StackObjectKind::PoisonousTrigger`. The counter-spell catch-all arm includes PoisonousTrigger. The builder.rs correctly skips `TriggeredAbilityDef` registration to avoid double-triggering. Tests cover all 6 scenarios from the plan. Two LOW findings (incorrect CR citation, minor event doc comment) -- no HIGH or MEDIUM.

## Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | LOW | `stack.rs:551-553` | **Incorrect CR citation for source-independent resolution.** Cites CR 603.10 but should cite CR 113.7a. **Fix:** Change "CR 603.10" to "CR 113.7a" in the doc comment. |
| 2 | LOW | `resolution.rs:1970` | **Same incorrect CR citation in resolution comment.** Same issue as Finding 1. **Fix:** Change "CR 603.10" to "CR 113.7a". |

### Finding Details

#### Finding 1: Incorrect CR citation for source-independent resolution

**Severity**: LOW
**File**: `crates/engine/src/state/stack.rs:551-553`
**CR Rule**: 113.7a -- "Once activated or triggered, an ability exists on the stack independently of its source. Destruction or removal of the source after that time won't affect the ability."
**Issue**: The doc comment on `PoisonousTrigger` states "CR 603.10: The source creature does NOT need to be on the battlefield at resolution time." CR 603.10 is about "looking back in time" to determine whether a trigger fires at all (e.g., leaves-the-battlefield triggers). The correct rule for "triggered abilities resolve independently of their source" is CR 113.7a. The behavior is correct; only the citation is wrong.
**Fix**: Replace the two lines citing CR 603.10 with:
```
/// CR 113.7a: Once triggered, this ability exists on the stack independently
/// of its source. The poison counters are given regardless of whether the
/// source creature is still on the battlefield at resolution time.
```

#### Finding 2: Same incorrect CR citation in resolution comment

**Severity**: LOW
**File**: `crates/engine/src/rules/resolution.rs:1970`
**CR Rule**: 113.7a (same as Finding 1)
**Issue**: The resolution handler comment also cites "CR 603.10" for the same incorrect reason.
**Fix**: Change line 1970 from:
```
// CR 603.10: The source creature does NOT need to be on the battlefield
```
to:
```
// CR 113.7a: The source creature does NOT need to be on the battlefield
```

## CR Coverage Check

| CR Subrule | Implemented? | Tested? | Notes |
|------------|-------------|---------|-------|
| 702.70a (trigger definition) | Yes | Yes | `test_702_70a_poisonous_basic_gives_poison_counter` -- fires on combat damage to player, gives N poison counters |
| 702.70a (N is fixed, not damage-based) | Yes | Yes | `test_702_70a_poisonous_amount_independent_of_damage` -- 5/5 creature with Poisonous 1 gives exactly 1 counter |
| 702.70a (only fires on player damage) | Yes | Yes | `test_702_70a_poisonous_blocked_no_trigger` -- blocked creature does not trigger |
| 702.70a (multiplayer: correct target) | Yes | Yes | `test_702_70a_poisonous_multiplayer_correct_player` -- 4-player, only attacked player gets counters |
| 702.70b (multiple instances) | Yes | Yes | `test_702_70b_poisonous_multiple_instances_trigger_separately` -- Poisonous 1 + Poisonous 2 = 3 counters |
| 104.3d / 704.5c (10+ poison SBA) | Pre-existing | Yes | `test_702_70a_poisonous_kills_via_sba` -- 9 + 1 = 10 counters triggers loss |
| 702.70a (additive, not replacement) | Yes | Yes | `test_702_70a_poisonous_basic_gives_poison_counter` -- P2 loses life AND gets poison counter |
| 702.70a (zero damage = no trigger) | Pre-existing guard | No (negative test gap) | The `assignment.amount == 0` guard at abilities.rs:2134 covers this, but no dedicated test with a prevention effect exists. Edge case relies on existing combat infrastructure. |

## Notes

- The discriminant values differ from the plan (plan: 83/23, actual: 84/24) because the Melee ability was implemented between plan creation and Poisonous implementation, taking the planned discriminant slots. This is correct and expected.
- The `unwrap_or` fallbacks in the flush branch (line 2897: `unwrap_or(trigger.controller)` for target_player, line 2898: `unwrap_or(1)` for poisonous_n) are defensive defaults for the case where `is_poisonous_trigger` is true but the Option fields are None. This should be unreachable in practice since the dispatch always sets both to `Some(...)`. The fallback values are reasonable (controller as default target, 1 as default N), matching the pattern established by other triggers.
- The `PoisonCountersGiven` event reuse is correct. The event's doc comment in events.rs says "infect damage" but the field semantics (`player`, `amount`, `source`) are generic. This is a pre-existing LOW issue (stale doc comment), not introduced by this change.
