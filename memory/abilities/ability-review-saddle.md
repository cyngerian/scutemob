# Ability Review: Saddle

**Date**: 2026-03-07
**Reviewer**: ability-impl-reviewer (Opus)
**CR**: 702.171
**Files reviewed**:
- `crates/engine/src/state/types.rs:1305-1316`
- `crates/engine/src/state/hash.rs:670-674` (KW), `hash.rs:893-895` (GameObject), `hash.rs:2031-2035` (SOK)
- `crates/engine/src/state/game_object.rs:658-666`
- `crates/engine/src/state/mod.rs:387-388`, `mod.rs:555-556`
- `crates/engine/src/state/builder.rs:1002-1003`
- `crates/engine/src/state/stack.rs:1195-1205`
- `crates/engine/src/rules/command.rs:548-566`
- `crates/engine/src/rules/engine.rs:380-399`
- `crates/engine/src/rules/abilities.rs:6295-6561`
- `crates/engine/src/rules/resolution.rs:4056-4076`
- `crates/engine/src/rules/turn_actions.rs:1295-1311`
- `crates/engine/src/effects/mod.rs:2922-2924`
- `crates/engine/src/testing/replay_harness.rs:931-944`
- `tools/replay-viewer/src/view_model.rs:592-594`, `view_model.rs:879`
- `tools/tui/src/play/panels/stack_view.rs:201-203`
- `crates/engine/tests/saddle.rs` (865 lines, 14 tests)
- `crates/engine/src/rules/copy.rs` (verified no `is_saddled` reference)

## Verdict: clean

The Saddle implementation is correct and thorough. All three CR 702.171 subrules are faithfully implemented. The sorcery-speed restriction (the key difference from Crew) is enforced with three separate checks (active player, main phase, empty stack). The `is_saddled` boolean flag approach correctly models the "designation" nature of saddled status per CR 702.171b, with proper cleanup at end-of-turn and zone-change clearing via CR 400.7. Hash coverage is complete for the new field, keyword variant, and SOK variant. All exhaustive match arms are handled. The 14 tests provide strong coverage of positive and negative cases. One LOW finding exists for a missing test sub-case.

## Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | LOW | `tests/saddle.rs:600` | **Missing test for stack-not-empty sorcery-speed sub-case.** Test 10 covers "not active player" and "not main phase" but omits "stack not empty." **Fix:** Add a sub-case (c) that puts a spell on the stack and verifies SaddleMount is rejected with an appropriate error message. |

### Finding Details

#### Finding 1: Missing test for stack-not-empty sorcery-speed sub-case

**Severity**: LOW
**File**: `crates/engine/tests/saddle.rs:600`
**CR Rule**: 702.171a -- "Activate only as a sorcery."
**Issue**: The `test_saddle_sorcery_speed_only` test covers two of three sorcery-speed restriction sub-cases: (a) not active player's turn and (b) not a main phase. However, the third condition -- (c) stack is not empty -- is not tested. The enforcement code at `abilities.rs:6359-6363` correctly checks `!state.stack_objects.is_empty()`, so the implementation is correct; only the test coverage has this gap.
**Fix**: Add a case (c) to `test_saddle_sorcery_speed_only` (or a separate test) that manually pushes a stack object, then attempts `SaddleMount` and asserts rejection with an error mentioning "stack" or "empty".

## CR Coverage Check

| CR Subrule | Implemented? | Tested? | Notes |
|------------|-------------|---------|-------|
| 702.171a (activated ability definition) | Yes | Yes | tests 1-8, 10-13 |
| 702.171a (sorcery-speed: active player) | Yes | Yes | test 10 case (a) |
| 702.171a (sorcery-speed: main phase) | Yes | Yes | test 10 case (b) |
| 702.171a (sorcery-speed: empty stack) | Yes | No | Code at abilities.rs:6359 is correct; missing test case (LOW) |
| 702.171a (total power >= N) | Yes | Yes | tests 1-3 |
| 702.171a ("other" -- self-exclusion) | Yes | Yes | test 4 |
| 702.171a (untapped creatures) | Yes | Yes | test 6 |
| 702.171a (creatures only) | Yes | Yes | test 7 |
| 702.171a (you control) | Yes | Yes | test 13 |
| 702.171b (designation, not type change) | Yes | Yes | test 1 verifies boolean flag, not type |
| 702.171b (until end of turn) | Yes | Yes | test 9 |
| 702.171b (leaves battlefield) | Yes | Yes | test 14 |
| 702.171b (not copiable) | Yes | N/A | Automatic: is_saddled is on GameObject, not Characteristics; copy.rs confirmed clean |
| 702.171c (creatures "saddle" as tapped) | Yes | Yes | tests 1, 3 verify tapping |
| Ruling: already-saddled is legal | Yes | Yes | test 8 |
| Ruling: summoning sickness allows saddling | Yes | Yes | test 5 |
| CR 602.2: priority required | Yes | Yes | test 11 |
| CR 702.61a: split second blocks | Yes | N/A | Code present; no dedicated test (consistent with other abilities) |
| Duplicate creature rejection | Yes | Yes | test 12 |
