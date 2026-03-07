# Ability Review: Living Metal

**Date**: 2026-03-07
**Reviewer**: ability-impl-reviewer (Opus)
**CR**: 702.161
**Files reviewed**:
- `crates/engine/src/state/types.rs:1182-1185` (KeywordAbility::LivingMetal)
- `crates/engine/src/state/hash.rs:638-639` (hash arm)
- `crates/engine/src/rules/layers.rs:112-130` (Layer 4 inline enforcement)
- `tools/replay-viewer/src/view_model.rs:845` (display arm)
- `crates/engine/tests/living_metal.rs` (7 tests, 319 lines)

## Verdict: clean

The implementation correctly models CR 702.161a. The Layer 4 inline block adds `CardType::Creature` only when the active player is the permanent's controller and the permanent is on the battlefield. The pattern matches the existing Impending inline check. All required infrastructure (enum variant, hash arm, display arm) is present. Tests cover positive cases, negative cases, off-battlefield, distinct P/T, multiplayer rotation, and non-Living-Metal artifacts. No HIGH or MEDIUM findings.

## Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | LOW | `tests/living_metal.rs:48` | **Vehicle subtype not set in test spec.** Comment at line 10 claims "Vehicle subtype" is retained, but the spec never sets any subtypes. **Fix:** Add `spec.subtypes = vec![SubType::Vehicle];` to `living_metal_vehicle_spec` and assert subtype retention in test 3. |
| 2 | LOW | `tests/living_metal.rs` | **No hand/exile zone test.** Test 4 covers graveyard but not hand or exile. CR 611.3b applies to all non-battlefield zones. **Fix:** Add a brief assertion for `ZoneId::Hand` in test 4, or add a separate test for hand. Low risk since the implementation checks `zone == ZoneId::Battlefield` which excludes all other zones uniformly. |

### Finding Details

#### Finding 1: Vehicle subtype not set in test spec

**Severity**: LOW
**File**: `crates/engine/tests/living_metal.rs:48`
**CR Rule**: 702.161a -- "in addition to its other types"
**Issue**: The `living_metal_vehicle_spec` helper sets `card_types = vec![CardType::Artifact]` but does not set any subtypes. The module-level doc comment claims "Vehicle subtype" retention is tested, but no test actually asserts `SubType::Vehicle` is present. This is a documentation-vs-reality mismatch. The engine behavior is correct (insert only adds Creature, doesn't remove subtypes), but the test doesn't verify it.
**Fix**: Add `spec.subtypes = vec![SubType::Vehicle];` to the helper and add `assert!(chars.subtypes.contains(&SubType::Vehicle))` in test 3 (`test_living_metal_retains_other_types`).

#### Finding 2: Only graveyard tested for off-battlefield

**Severity**: LOW
**File**: `crates/engine/tests/living_metal.rs:178-199`
**CR Rule**: 611.3b -- static abilities function only on the battlefield
**Issue**: Test 4 only checks the graveyard zone. While the implementation uses `zone == ZoneId::Battlefield` which uniformly excludes all other zones, a test for at least one more zone (hand) would strengthen coverage.
**Fix**: In test 4, add a second state with the vehicle in `ZoneId::Hand(p1)` and assert Creature is not added. Or add a parameterized loop over `[Graveyard, Hand, Exile]`.

## CR Coverage Check

| CR Subrule | Implemented? | Tested? | Notes |
|------------|-------------|---------|-------|
| 702.161a (creature during your turn) | Yes | Yes | test 1, test 3 |
| 702.161a (not creature during opponent's turn) | Yes | Yes | test 2 |
| 702.161a ("in addition to its other types") | Yes | Yes | test 1 (Artifact retained), test 3 |
| 702.161a (P/T from printed stats) | Yes | Yes | test 1, test 5 |
| 611.3b (battlefield only) | Yes | Partial | test 4 (graveyard only; Finding 2) |
| Multiplayer (N players) | Yes | Yes | test 6 (4-player rotation) |
| Negative (non-LM artifact) | N/A | Yes | test 7 |

## Notes

- **Phased-out permanents**: The inline check does not filter `is_phased_in()`, but this is consistent with existing inline checks (Changeling at line 68, Impending at line 93, Devoid at line 82). Phased-out filtering is handled at call sites per the codebase convention. Not a finding.
- **Humility interaction**: Correctly handled by layer ordering. Living Metal adds Creature at Layer 4 before Humility strips keywords at Layer 6. The plan documents this and the implementation is consistent.
- **No TUI impact**: Living Metal introduces no new `StackObjectKind` variant (static ability), so `stack_view.rs` needs no update. Confirmed no `KeywordAbility` match in TUI code.
- **Hash coverage**: Discriminant 128 correctly added as the last arm in the hash match. Chain is contiguous (127 UmbraArmor, 128 LivingMetal).
