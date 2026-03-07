# Ability Review: Fabricate

**Date**: 2026-03-07
**Reviewer**: ability-impl-reviewer (Opus)
**CR**: 702.123
**Files reviewed**: `crates/engine/src/state/types.rs:1213-1221`, `crates/engine/src/state/hash.rs:650-654`, `tools/replay-viewer/src/view_model.rs:852`, `crates/engine/src/rules/replacement.rs:971-1042`, `crates/engine/src/rules/lands.rs:365-368`, `crates/engine/tests/fabricate.rs`

## Verdict: needs-fix

The core implementation is correct: CR 702.123a counter placement, CR 702.123b multiple-instance handling, hash coverage, view_model arm, Servo token spec, and the fallback-to-tokens path are all properly implemented. One MEDIUM finding: the token fallback path (permanent no longer on battlefield) has no test coverage, despite being explicitly called out in the plan as test 6. No HIGH findings.

## Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | MEDIUM | `tests/fabricate.rs` | **Missing test for token fallback path.** The plan specified test 6 (`test_fabricate_permanent_left_battlefield_creates_tokens`) but it was not implemented. The `else` branch at `replacement.rs:1014-1039` that creates Servo tokens when the permanent is gone is untested. **Fix:** Add a test that manually removes the Fabricate creature from the battlefield before the inline trigger fires (or directly calls into the fallback path). The test should verify N Servo tokens are created with correct characteristics (1/1, colorless, Artifact Creature -- Servo). |
| 2 | LOW | `tests/fabricate.rs:430` | **ObjectSpec only has one Fabricate keyword for Double Fabricate test.** The `ObjectSpec` for the double-fabricate test only calls `.with_keyword(KeywordAbility::Fabricate(1))` -- it does not add `Fabricate(2)`. The test still passes because `fire_when_enters_triggered_effects` reads abilities from the `CardDefinition` (via the registry), not from the `ObjectSpec`. This is correct behavior but the test setup is misleading. **Fix:** Either add both keywords to the ObjectSpec for clarity, or add a comment explaining that the CardDefinition (not the ObjectSpec) is the source of truth for Fabricate dispatch. |
| 3 | LOW | `replacement.rs:971-977` | **Inline ETB approximation not documented in code.** The plan correctly identifies that Fabricate is a triggered ability that uses the stack (CR 702.123a: "When this permanent enters"), but the implementation fires it inline during ETB resolution for bot simplicity. The plan documents this, but the code comments do not explicitly note this as a known approximation that will need to change for interactive play. **Fix:** Add a brief comment like `// NOTE: Fires inline for bot play. In interactive play, this should go on the stack (CR 702.123a is a triggered ability).` |

### Finding Details

#### Finding 1: Missing test for token fallback path

**Severity**: MEDIUM
**File**: `crates/engine/tests/fabricate.rs`
**CR Rule**: 702.123a -- "When this permanent enters, you may put N +1/+1 counters on it. If you don't, create N 1/1 colorless Servo artifact creature tokens."
**Ruling**: 2016-09-20 -- "If you can't put +1/+1 counters on the creature for any reason as fabricate resolves (for instance, if it's no longer on the battlefield), you just create Servo tokens."
**Issue**: The `else` branch at `replacement.rs:1014-1039` creates Servo tokens when the permanent is no longer on the battlefield. This path is completely untested. The plan explicitly listed this as test 6 (`test_fabricate_permanent_left_battlefield_creates_tokens`) but it was not implemented. The Servo token spec (1/1 colorless Artifact Creature -- Servo) is also untested -- if the token characteristics were wrong (e.g., wrong P/T, wrong types, missing subtype), no test would catch it.
**Fix**: Add `test_fabricate_permanent_left_battlefield_creates_tokens`. Since the inline trigger fires immediately during ETB (the permanent is always on the battlefield at that point), the test will need to either: (a) manually remove the object from the battlefield and re-invoke the fabricate logic, or (b) directly unit-test the token creation path by setting up a state where the permanent's zone is not Battlefield before the fabricate block runs. If neither approach is feasible with the current architecture, document this as a deferred test gap with a tracking comment.

#### Finding 2: ObjectSpec only has one Fabricate keyword for Double Fabricate test

**Severity**: LOW
**File**: `crates/engine/tests/fabricate.rs:430`
**Issue**: The ObjectSpec for the "Double Fabricate Test" creature only adds `.with_keyword(KeywordAbility::Fabricate(1))` but the CardDefinition has both `Fabricate(1)` and `Fabricate(2)`. The test passes because `fire_when_enters_triggered_effects` reads from the `CardDefinition`, not the object's runtime keywords. While correct, this is misleading -- a reader might think only one instance is being tested.
**Fix**: Add `.with_keyword(KeywordAbility::Fabricate(2))` to the ObjectSpec, or add a comment clarifying that the CardDefinition is the authoritative source.

#### Finding 3: Inline ETB approximation not documented in code

**Severity**: LOW
**File**: `crates/engine/src/rules/replacement.rs:971-977`
**CR Rule**: 702.123a -- "When this permanent enters" is triggered ability language (uses the stack).
**Issue**: The code fires Fabricate inline during ETB processing, bypassing the stack. The plan documents this as intentional for bot play, but the code comments do not flag this as a known approximation.
**Fix**: Add a comment noting the inline approximation and that interactive play will need stack-based resolution.

## CR Coverage Check

| CR Subrule | Implemented? | Tested? | Notes |
|------------|-------------|---------|-------|
| 702.123a (counters path) | Yes | Yes | Tests 1, 4, 7 cover N=1, N=2, N=3 |
| 702.123a (tokens path) | Yes | **No** | replacement.rs:1014-1039 untested (Finding 1) |
| 702.123a (Servo spec: 1/1 colorless Artifact Creature Servo) | Yes | **No** | Token characteristics not validated by any test |
| 702.123b (multiple instances) | Yes | Yes | Test 5 covers Fabricate 1 + Fabricate 2 = 3 counters |
| Bot choice documented | Yes | Yes | All counter tests confirm bot always picks counters |
| Hash coverage | Yes | N/A | hash.rs:650-654 hashes discriminant 132 + n |
| view_model.rs arm | Yes | N/A | view_model.rs:852 |
| Negative test (non-Fabricate) | N/A | Yes | Test 6 confirms no counters/tokens on plain creature |
| Multiplayer | Yes | Yes | Test 8 covers 4-player game |
