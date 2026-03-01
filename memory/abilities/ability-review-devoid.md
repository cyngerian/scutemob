# Ability Review: Devoid

**Date**: 2026-02-28
**Reviewer**: ability-impl-reviewer (Opus)
**CR**: 702.114
**Files reviewed**:
- `crates/engine/src/state/types.rs:609-615` (enum variant)
- `crates/engine/src/state/hash.rs:463-464` (hash discriminant 73)
- `crates/engine/src/rules/layers.rs:74-83` (Layer 5 CDA enforcement)
- `crates/engine/tests/devoid.rs:1-359` (8 unit tests)
- `tools/replay-viewer/src/view_model.rs:682` (display string)
- `crates/engine/src/rules/commander.rs:198-214` (color identity -- unchanged, verified correct)

## Verdict: clean

Devoid implementation is correct and complete. The CDA enforcement in Layer 5 follows
the exact same pattern as Changeling in Layer 4 and correctly handles all CR subrules.
The inline CDA check runs before `resolve_layer_order` gathers non-CDA Layer 5 effects,
which means color-adding effects (e.g., Painter's Servant) correctly override the Devoid
colorlessness. Layer ordering (L5 before L6) ensures that RemoveAllAbilities in Layer 6
does not undo Devoid's color clearing. Color identity in commander.rs is correctly
unaffected (it reads from CardDefinition mana cost symbols, not game-state colors).
Hash discriminant 73 is unique within the KeywordAbility impl. All 8 tests are well-structured,
cite CR rules, and cover the key edge cases including all-zones functionality, layer
interaction with ability removal, color-adding override, and protection interaction.
No HIGH or MEDIUM findings.

## Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | LOW | `tests/devoid.rs:344` | **Protection test uses manual match instead of engine API.** Test 8 manually matches on `ProtectionQuality::FromColor` rather than calling the engine's `has_protection_from_source` function. This validates the color set is empty but does not exercise the actual protection path. **Fix:** Optionally add a second test (or extend this one) that sets up both attacker and defender with protection and calls `has_protection_from_source` to confirm the full path. |
| 2 | LOW | `tests/devoid.rs` | **Missing test: Devoid + SetColors override.** Test 4 covers `AddColors` (which adds to the empty set), but does not cover `SetColors` (which replaces the entire set). Both should work correctly with the current implementation, but a `SetColors` test would improve coverage symmetry. **Fix:** Optionally add a test with `LayerModification::SetColors(OrdSet::from(vec![Color::Green]))` to confirm SetColors also overrides Devoid. |
| 3 | LOW | `tests/devoid.rs` | **Missing test: Devoid in command zone.** Tests 5 and 6 cover graveyard and hand zones (CR 604.3), but no test covers the command zone. Since Devoid is frequently found on Commander-legal Eldrazi, a command zone test would strengthen Commander-format confidence. **Fix:** Optionally add a test placing a Devoid card in `ZoneId::CommandZone(p1())`. |

### Finding Details

#### Finding 1: Protection test uses manual match instead of engine API

**Severity**: LOW
**File**: `crates/engine/tests/devoid.rs:344`
**CR Rule**: 702.16a -- "A permanent or player with protection ... can't be the target of spells with the stated quality..., can't be blocked by creatures with the stated quality..."
**Issue**: The test manually checks `source_chars.colors.contains(c)` against a `ProtectionQuality::FromColor` value. While this does verify the critical property (that Devoid makes the source colorless), it does not exercise the engine's `has_protection_from_source` function, which is the actual code path used during combat and targeting. If `has_protection_from_source` had a bug that ignored colors, this test would not catch it.
**Fix**: Consider adding a companion test or extending this test to call `has_protection_from_source(&state, defender_id, source_id)` with a defender that has `Protection(ProtectionQuality::FromColor(Color::Red))` and confirm it returns `false` for a Devoid source with Red mana cost.

#### Finding 2: Missing test for SetColors override

**Severity**: LOW
**File**: `crates/engine/tests/devoid.rs`
**CR Rule**: 613.3 -- "Within layers 2-6, apply effects from characteristic-defining abilities first, then all other effects in timestamp order."
**Issue**: Test 4 covers `AddColors` (which adds Blue to the empty color set after Devoid clears it), but `SetColors` (which replaces the entire color set) is not tested. Both `AddColors` and `SetColors` are handled in `apply_layer_modification` and both would work correctly, but the asymmetric coverage is a minor gap.
**Fix**: Optionally add a test using `LayerModification::SetColors(OrdSet::from(vec![Color::Green]))` to confirm that `SetColors` also correctly overrides Devoid's colorlessness.

#### Finding 3: Missing test for command zone (Commander format)

**Severity**: LOW
**File**: `crates/engine/tests/devoid.rs`
**CR Rule**: 604.3 -- "Characteristic-defining abilities function in all zones."
**Issue**: Tests cover battlefield, graveyard, and hand zones, but the command zone is not tested. Since the engine is Commander-first (architecture invariant #6) and Devoid creatures are often Eldrazi that can be commanders, a command zone test would increase confidence in Commander-format correctness.
**Fix**: Optionally add a test placing a Devoid card in `ZoneId::CommandZone(p1())` and verifying it is colorless after `calculate_characteristics`.

## CR Coverage Check

| CR Subrule | Implemented? | Tested? | Notes |
|------------|-------------|---------|-------|
| 702.114a "This object is colorless" | Yes | Yes | test_devoid_creature_is_colorless |
| 702.114a "functions everywhere" (= CR 604.3) | Yes | Yes | test_devoid_works_in_graveyard, test_devoid_works_in_hand |
| 604.3 CDAs function in all zones | Yes | Partial | Graveyard + hand tested; command zone not tested (LOW #3) |
| 604.3a CDA criteria | Yes (inherent) | N/A | Devoid meets all 5 criteria by definition |
| 613.3 CDAs apply first within layer | Yes | Yes | test_devoid_color_adding_effect_overrides |
| 613.1e Layer 5 (ColorChange) | Yes | Yes | Core enforcement in layers.rs:81-83 |
| 613.1f Layer 6 ordering vs Layer 5 | Yes (structural) | Yes | test_devoid_lose_all_abilities_still_colorless |
| Ruling: "loses devoid, still colorless" | Yes | Yes | test_devoid_lose_all_abilities_still_colorless |
| Ruling: "color-adding overrides devoid" | Yes | Yes | test_devoid_color_adding_effect_overrides |
| Color identity unaffected (CR 903.4) | Yes (by design) | No | commander.rs uses CardDefinition, not game-state colors; no test but correct by construction |
| Protection interaction (CR 702.16a) | Yes (by design) | Partial | test_devoid_protection_from_color_does_not_match verifies color set is empty; does not call has_protection_from_source (LOW #1) |
