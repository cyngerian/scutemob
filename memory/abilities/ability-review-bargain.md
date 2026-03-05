# Ability Review: Bargain

**Date**: 2026-03-03
**Reviewer**: ability-impl-reviewer (Opus)
**CR**: 702.166
**Files reviewed**:
- `crates/engine/src/state/types.rs` (lines 891-903)
- `crates/engine/src/state/hash.rs` (lines 535-536, 713-714, 1662-1663, 2734-2735)
- `crates/engine/src/state/stack.rs` (lines 193-201)
- `crates/engine/src/state/game_object.rs` (lines 518-523)
- `crates/engine/src/state/mod.rs` (lines 308-309, 429-430)
- `crates/engine/src/state/builder.rs` (line 952)
- `crates/engine/src/cards/card_definition.rs` (lines 950-955)
- `crates/engine/src/rules/command.rs` (lines 164-174)
- `crates/engine/src/rules/casting.rs` (lines 69, 1368-1413, 1750-1763, 1832-1833)
- `crates/engine/src/rules/resolution.rs` (lines 208-209, 281-283)
- `crates/engine/src/rules/engine.rs` (lines 95, 114)
- `crates/engine/src/rules/copy.rs` (lines 211-213, 403-404)
- `crates/engine/src/rules/abilities.rs` (10 construction sites, all `was_bargained: false`)
- `crates/engine/src/effects/mod.rs` (lines 70-73, 86, 104, 1351, 1370, 2479, 2777)
- `crates/engine/src/testing/replay_harness.rs` (lines 249-252, 960-980, plus 15 `bargain_sacrifice: None` sites)
- `crates/engine/src/testing/script_schema.rs` (lines 280-285)
- `crates/engine/tests/script_replay.rs` (lines 153, 175)
- `crates/engine/tests/bargain.rs` (full file, 885 lines)

## Verdict: clean

The Bargain implementation is thorough and correct. All three CR 702.166 subrules are
implemented faithfully. The implementation follows established patterns for additional
costs (kicker, retrace, jump-start). The data flow from `CastSpell.bargain_sacrifice`
through `StackObject.was_bargained` to `EffectContext.was_bargained` and
`GameObject.was_bargained` is complete and consistent. Hash coverage is complete across
all four sites (KeywordAbility, Condition, StackObject, GameObject). Copy propagation
correctly inherits bargained status from the original spell (CR 707.2). Zone-change
reset correctly clears `was_bargained` at both `move_object_to_zone` sites in
`state/mod.rs` (CR 400.7). All 10 tests are well-structured, cite CR rules, and cover
the key positive and negative cases. Two LOW findings exist but neither affects
correctness.

## Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | LOW | `stack.rs:197-199` | **Contradictory doc comment on copy behavior.** Lines 197 says "Must always be false for copies" but lines 198-199 correctly note copies inherit bargained status. The code in `copy.rs:213` is correct. **Fix:** Change line 197 to "Not set directly on copies -- copies inherit via the copy system." or remove the contradictory sentence entirely. |
| 2 | LOW | `bargain.rs` (all tests) | **No test for copy interaction.** No test verifies that a copy of a bargained spell also has `was_bargained = true`. The copy propagation code is correct (`copy.rs:213`), but there is no test validating it. **Fix:** Add a test `test_bargain_copy_inherits_bargained_status` that copies a bargained spell (e.g., via storm) and asserts `was_bargained == true` on the copy. |

### Finding Details

#### Finding 1: Contradictory doc comment on copy behavior

**Severity**: LOW
**File**: `crates/engine/src/state/stack.rs:197-199`
**CR Rule**: 702.166b / 707.2
**Issue**: The doc comment on `StackObject.was_bargained` contains two contradictory
statements. Line 197 says "Must always be false for copies (`is_copy: true`) -- copies
are not cast." Lines 198-199 then say "Copies of a bargained spell are also bargained
(CR 707.2), so this should be propagated to copies in the copy system." The actual code
at `copy.rs:213` correctly propagates `was_bargained: original.was_bargained`, making
line 197 factually wrong for this field. This is a pre-existing pattern (many StackObject
fields have the same "must always be false for copies" boilerplate), but for `was_bargained`
it is specifically contradicted by the subsequent clarification.
**Fix**: Replace lines 197-199 with a single accurate statement:
"Copies inherit this from the original (CR 707.2 -- copies copy choices made during casting)."

#### Finding 2: No test for copy interaction

**Severity**: LOW
**File**: `crates/engine/tests/bargain.rs` (missing test)
**CR Rule**: 707.2 -- "The copy acquires the color(s), mana cost, mana value, name, types,
supertypes, subtypes, rules text, power, toughness, and loyalty of the original."
/ 702.166b -- copies of bargained spells are also bargained
**Issue**: The plan explicitly lists copy interactions as an edge case to watch
("Copies of a bargained spell are also bargained -- CR 707.2"). The implementation in
`copy.rs:213` correctly handles this by setting `was_bargained: original.was_bargained`.
However, there is no unit test validating this behavior. If the copy propagation were
accidentally removed, no test would catch it.
**Fix**: Add test `test_bargain_copy_inherits_bargained_status` -- set up a bargained
spell with storm, verify the copy on the stack also has `was_bargained == true`.

## CR Coverage Check

| CR Subrule | Implemented? | Tested? | Notes |
|------------|-------------|---------|-------|
| 702.166a (optional additional cost) | Yes | Yes | Tests 1, 2, 3, 4, 5 (positive); test 6 (invalid creature rejection) |
| 702.166a (artifact valid) | Yes | Yes | Test 3 |
| 702.166a (enchantment valid) | Yes | Yes | Test 4 |
| 702.166a (token valid) | Yes | Yes | Tests 1, 5 (creature token) |
| 702.166a (non-token creature rejected) | Yes | Yes | Test 6 |
| 702.166a (must control sacrifice) | Yes | Yes | Test 7 |
| 702.166a (spell must have Bargain) | Yes | Yes | Test 8 |
| 702.166b (spell "has been bargained") | Yes | Yes | Tests 1, 2 (resolution with effect branch) |
| 702.166b (permanent was_bargained propagation) | Yes | Yes | Tests 9, 10 |
| 702.166c (multiple instances redundant) | Yes (implicit) | No | Engine validates Bargain keyword presence, not count. Redundant instances naturally handled. No explicit test, but behavior is trivially correct. |
| Copy interaction (CR 707.2) | Yes | No | `copy.rs:213` propagates. No test -- Finding 2. |
| Zone-change reset (CR 400.7) | Yes | No | `state/mod.rs:309,430` resets to false. Implicitly tested by zone-change integrity tests. |
| Cascade free-cast not bargained | Yes | No | `copy.rs:404` sets false. No explicit test but consistent with pattern. |
