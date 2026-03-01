# Ability Review: Training

**Date**: 2026-03-01
**Reviewer**: ability-impl-reviewer (Opus)
**CR**: 702.149
**Files reviewed**:
- `crates/engine/src/state/types.rs` (lines 693-700)
- `crates/engine/src/state/game_object.rs` (lines 206-211)
- `crates/engine/src/state/hash.rs` (lines 493-494, 1187-1190)
- `crates/engine/src/state/builder.rs` (lines 489-511)
- `crates/engine/src/rules/abilities.rs` (lines 1508-1550)
- `tools/replay-viewer/src/view_model.rs` (line 706)
- `crates/engine/tests/training.rs` (full file, 670 lines)

## Verdict: clean

The Training implementation is correct and complete for all three CR 702.149 subrules.
The enforcement logic faithfully implements the trigger condition (strictly greater power
co-attacker), the effect (single +1/+1 counter via AddCounter on Source), the lack of
intervening-if (per ruling 2021-11-19), and multiple-instance behavior. The architectural
pattern closely follows the proven Dethrone template with a dedicated TriggerEvent variant,
condition checked at trigger-collection time, and standard TriggeredAbility stack resolution.
All seven tests pass and cover positive, negative, multi-instance, multi-creature, and
multiplayer scenarios. No HIGH or MEDIUM findings.

## Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | LOW | `abilities.rs:1516-1550` | **Power check iterates attackers per-attacker.** For N attackers, `calculate_characteristics` is called O(N^2) times across Training checks. Not a correctness issue but could be optimized by pre-computing attacker powers once outside the loop if performance becomes a concern. **Fix:** Defer unless benchmarks show regression. |
| 2 | LOW | `tests/training.rs` | **No test for Training creature being the strongest attacker while another Training creature triggers.** In test 6, the 4/4 Powerhouse is vanilla. A scenario where Trainee B (2/2 Training) triggers because Trainee A (1/1 Training) has a co-attacker with greater power is implicitly tested (4/4 > 2), but there is no test where one Training creature is also the "greater power ally" for another Training creature. This is implicitly covered (the power check does not care about keywords on the co-attacker), but an explicit test would document the interaction. **Fix:** Consider adding a test where a 3/3 Training creature and a 1/1 Training creature attack together -- the 1/1 should trigger (3 > 1) and the 3/3 should not (1 < 3). Low priority. |

### Finding Details

#### Finding 1: O(N^2) power calculations in Training trigger check

**Severity**: LOW
**File**: `crates/engine/src/rules/abilities.rs:1516-1550`
**CR Rule**: 702.149a
**Issue**: The Training trigger check is inside the per-attacker loop (line 1381). For each
attacker, it calls `calculate_characteristics` on itself (line 1519) and then iterates all
attackers again via `.any()` (line 1524), calling `calculate_characteristics` for each other
attacker (line 1526). With N attackers, this is O(N^2) calls to `calculate_characteristics`.
In typical Commander games, N is small (usually 1-5 attackers), so this is not a practical
concern. The Dethrone block above has a similar pattern (it checks all players' life totals
per attacker).
**Fix**: No immediate fix needed. If profiling reveals this as a hotspot (unlikely with
typical N), pre-compute a `Vec<(ObjectId, i32)>` of attacker powers once before the loop
and look up values from that cache.

#### Finding 2: Missing test for Training creature as greater-power ally

**Severity**: LOW
**File**: `crates/engine/tests/training.rs`
**CR Rule**: 702.149a -- "at least one other creature with power greater than this creature's power"
**Issue**: Test 6 covers two Training creatures triggering from a shared vanilla Powerhouse.
However, there is no test where a Training creature itself serves as the "greater power ally"
for another Training creature. For example: a 3/3 Training creature attacking alongside a
1/1 Training creature. The 1/1 should trigger (3 > 1), the 3/3 should not (1 < 3, no other
creature with greater power). This scenario is logically covered by the existing power
comparison code (which does not filter by keywords), but an explicit test would serve as
documentation.
**Fix**: Optionally add a test where both attackers have Training and only the weaker one
triggers. Low priority -- the existing tests adequately verify the power comparison logic.

## CR Coverage Check

| CR Subrule | Implemented? | Tested? | Notes |
|------------|-------------|---------|-------|
| 702.149a (trigger definition) | Yes | Yes | test_702_149a_training_basic_attacks_with_greater_power |
| 702.149a (strictly greater) | Yes | Yes | test_702_149a_training_does_not_trigger_equal_power |
| 702.149a (counter on self) | Yes | Yes | All positive tests verify counter on Training creature |
| 702.149a (at least one other) | Yes | Yes | test_702_149a_training_does_not_trigger_alone |
| 702.149a (lower power negative) | Yes | Yes | test_702_149a_training_does_not_trigger_lower_power |
| 702.149a (multiple creatures) | Yes | Yes | test_702_149a_training_two_training_creatures_both_trigger |
| 702.149a (multiplayer) | Yes | Yes | test_702_149a_training_multiplayer_four_player |
| 702.149b (multiple instances) | Yes | Yes | test_702_149b_training_multiple_instances |
| 702.149c ("when trains") | N/A | N/A | Deferred -- requires separate trigger event; documented in plan |

## Architectural Checks

| Check | Status | Notes |
|-------|--------|-------|
| Hash coverage for KeywordAbility::Training | OK | Discriminant 82, no collision |
| Hash coverage for TriggerEvent::SelfAttacksWithGreaterPowerAlly | OK | Discriminant 19, no collision |
| All exhaustive matches covered | OK | types.rs, hash.rs (x2), view_model.rs -- compiles clean |
| No .unwrap() in library code | OK | Uses .unwrap_or(0) and .and_then() for safe fallback |
| CR citations in doc comments | OK | All new code cites CR 702.149/702.149a/702.149b |
| No new StackObjectKind variant needed | OK | Uses standard TriggeredAbility path |
| TUI stack_view.rs | OK | No new StackObjectKind, no update needed |
| Multiplayer correctness | OK | Power check is within single player's attackers batch |
| Layer-aware power comparison | OK | Uses calculate_characteristics, not raw base P/T |
| No intervening-if (per ruling) | OK | intervening_if: None -- counter placed unconditionally at resolution |
| defending_player_id tagging | OK | Consistent with Dethrone/SelfAttacks pattern |
