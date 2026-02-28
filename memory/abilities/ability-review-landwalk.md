# Ability Review: Landwalk

**Date**: 2026-02-25
**Reviewer**: ability-impl-reviewer (Opus)
**CR**: 702.14
**Files reviewed**:
- `crates/engine/src/state/types.rs:60-76` (LandwalkType enum), `:133-137` (Landwalk variant)
- `crates/engine/src/state/hash.rs:261-271` (HashInto for LandwalkType), `:288-291` (Landwalk arm)
- `crates/engine/src/state/mod.rs:44` (re-export)
- `crates/engine/src/lib.rs:21` (re-export)
- `crates/engine/src/rules/combat.rs:17` (imports), `:484-509` (landwalk blocking restriction)
- `crates/engine/tests/keywords.rs:1136-1506` (7 tests)
- `tools/replay-viewer/src/view_model.rs:576` (display string)

## Verdict: clean

The Landwalk implementation is correct, well-structured, and matches CR 702.14a-e exactly.
The blocking restriction logic correctly checks only the defending player's lands using
post-layer `calculate_characteristics`, handles both basic-subtype landwalk and nonbasic
landwalk, and integrates cleanly with existing evasion checks (flying, intimidate,
CantBeBlocked, protection, menace). Hash coverage is complete. All 7 tests are well-designed
with proper CR citations, covering positive cases, negative cases, multiplayer defending-player
isolation, both LandwalkType variants, and multi-evasion independence. No HIGH or MEDIUM
findings.

## Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | LOW | `tests/keywords.rs:1136-1506` | **Missing edge case test: defender with zero lands.** No test verifies that landwalk does not prevent blocking when the defending player controls zero lands. The implementation is correct (`any()` returns false on empty iteration), but a test would document this edge case explicitly. **Fix:** Add a test where p2 has a creature but no lands; a swampwalk attacker should be blockable. |
| 2 | LOW | `tools/replay-viewer/src/view_model.rs:576` | **Debug-format display for landwalk type.** The viewer displays `"Landwalk (BasicType(SubType(\"Swamp\")))"` rather than a human-readable string like `"Swampwalk"`. Consistent with ProtectionFrom's treatment but not user-friendly. **Fix:** Match on `LandwalkType` variants to produce `"Swampwalk"`, `"Islandwalk"`, `"Nonbasic Landwalk"`, etc. |
| 3 | LOW | `crates/engine/src/rules/combat.rs:489-500` | **Performance: calculate_characteristics in inner loop.** For each landwalk keyword on each blocker, the code iterates all `state.objects.values()` and calls `calculate_characteristics` on each. This is O(objects * landwalk_keywords * blockers). Acceptable for Commander-scale games but could be optimized if profiling shows it matters. **Fix:** No immediate fix needed; note for future optimization. |

### Finding Details

#### Finding 1: Missing edge case test -- defender with zero lands

**Severity**: LOW
**File**: `crates/engine/tests/keywords.rs:1136-1506`
**CR Rule**: 702.14c -- "A creature with landwalk can't be blocked as long as the defending player controls at least one land with the specified land type."
**Issue**: The phrase "at least one land" implies that if the defender controls zero lands, landwalk does not apply and the creature can be blocked normally. The implementation handles this correctly via `state.objects.values().any(...)` returning `false` when no matching land exists. However, there is no test that explicitly covers the zero-lands scenario. All negative tests (tests 2, 4, 6) give the defender at least one land (of a non-matching type). Adding a zero-lands test would document the boundary condition.
**Fix**: Add `test_702_14_swampwalk_blockable_when_defender_has_no_lands` -- setup p2 with a creature but no lands on the battlefield. Assert DeclareBlockers succeeds.

#### Finding 2: Debug-format display for landwalk in replay viewer

**Severity**: LOW
**File**: `tools/replay-viewer/src/view_model.rs:576`
**CR Rule**: N/A (display concern)
**Issue**: The display string `format!("Landwalk ({lw:?})")` produces Rust Debug output like `"Landwalk (BasicType(SubType(\"Swamp\")))"` which is not human-readable. Other abilities like Flying and Menace display clean names. This is consistent with how `ProtectionFrom` is handled (line 579), so it's a broader pattern, not specific to landwalk.
**Fix**: Replace with a match that produces `"Swampwalk"`, `"Islandwalk"`, `"Forestwalk"`, `"Mountainwalk"`, `"Plainswalk"`, `"Nonbasic Landwalk"`, etc. Consider fixing the ProtectionFrom display at the same time.

#### Finding 3: Performance of inner-loop characteristic calculation

**Severity**: LOW
**File**: `crates/engine/src/rules/combat.rs:489-500`
**CR Rule**: N/A (performance)
**Issue**: The landwalk check iterates all objects in the game state and calls `calculate_characteristics` on each to check if it's a land controlled by the defending player with the matching type. In a Commander game with ~50-100 permanents, this is fast enough. However, if multiple blockers are declared against multiple attackers with landwalk, the same set of lands is recalculated each time.
**Fix**: No change needed now. If profiling reveals this as a bottleneck, cache the defending player's land characteristics once per DeclareBlockers call and reuse across all blocker checks.

## CR Coverage Check

| CR Subrule | Implemented? | Tested? | Notes |
|------------|-------------|---------|-------|
| 702.14a (generic term, type variants) | Yes | Yes | LandwalkType enum with BasicType(SubType) and Nonbasic variants; tested via swampwalk, islandwalk, nonbasic |
| 702.14b (evasion ability) | Yes | Yes | Blocking restriction in combat.rs; tested as blocking rejection |
| 702.14c (can't be blocked if defender has land) | Yes | Yes | Full implementation; 5 tests cover positive/negative for basic + nonbasic + multiplayer |
| 702.14c (defending player only) | Yes | Yes | `obj.controller == player` check; test_702_14_landwalk_checks_defending_player_only |
| 702.14c ("at least one land") | Yes | Partial | Implementation correct via `any()`; no explicit zero-lands test (Finding 1) |
| 702.14c (nonbasic variant) | Yes | Yes | LandwalkType::Nonbasic checks `!supertypes.contains(Basic)`; 2 tests |
| 702.14d (don't cancel each other) | Yes (automatic) | No | Automatic -- each creature's keywords checked independently; no specific test needed |
| 702.14e (redundant instances) | Yes (automatic) | No | Automatic -- `OrdSet<KeywordAbility>` deduplicates; no specific test needed |

## Notes

- **Layer system interaction**: The implementation correctly uses `calculate_characteristics(state, obj.id)` to get post-layer land types. This means Blood Moon turning all nonbasic lands into Mountains will automatically make mountainwalk unblockable against players who control former nonbasics. Spreading Seas adding Island subtype will enable islandwalk. No special code needed -- the layer system handles it.
- **702.14a exotic variants not yet supported**: "Artifact landwalk", "legendary landwalk", "snow swampwalk" are not representable with the current `LandwalkType` enum. The plan correctly identifies these as P3/P4 scope. When needed, additional enum variants can be added.
- **Naming**: `LandwalkType::BasicType(SubType)` means "a basic land subtype check" (Plains, Island, Swamp, Mountain, Forest). Despite the name, it does NOT check for the `Basic` supertype. The `Nonbasic` variant checks for the absence of `Basic` supertype. This naming distinction is clear in the doc comments but could be confusing at first glance. Not a bug, just a style observation.
