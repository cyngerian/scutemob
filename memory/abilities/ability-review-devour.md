# Ability Review: Devour

**Date**: 2026-03-06
**Reviewer**: ability-impl-reviewer (Opus)
**CR**: 702.82
**Files reviewed**:
- `crates/engine/src/state/types.rs:1135-1143`
- `crates/engine/src/state/hash.rs:624-628` (KeywordAbility), `:815-816` (GameObject), `:1907-1909` (StackObject)
- `crates/engine/src/rules/command.rs:247-259`
- `crates/engine/src/rules/casting.rs:78,2740-2793`
- `crates/engine/src/rules/engine.rs:104`
- `crates/engine/src/state/stack.rs:258-265`
- `crates/engine/src/state/game_object.rs:552-559`
- `crates/engine/src/rules/resolution.rs:840-1018`
- `crates/engine/src/rules/lands.rs:356-358`
- `crates/engine/src/testing/replay_harness.rs:1559-1595`
- `tools/replay-viewer/src/view_model.rs:832`
- `crates/engine/tests/devour.rs` (full file, 996 lines)
- `crates/engine/src/state/builder.rs:963`
- `crates/engine/src/state/mod.rs:365,503`
- `crates/engine/src/effects/mod.rs:2649`

## Verdict: clean

The Devour implementation is solid. All three CR 702.82 subrules are correctly implemented: the ETB replacement effect with optional sacrifice and counter placement (702.82a), the `creatures_devoured` tracking field (702.82b), and multiple-instance handling (702.82c by analogy). Hash coverage is complete for all new fields. Zone-change identity (CR 400.7) is properly handled with `creatures_devoured` reset at both `move_object_to_zone` sites. Replacement effects on sacrifice (Rest in Peace, commander zone redirect) are handled. Death events fire for sacrificed creatures. The only findings are LOW severity.

## Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | LOW | `casting.rs:2748` | **Missing Devour keyword check on spell.** Validation does not verify the spell has `KeywordAbility::Devour(_)`. **Fix:** Add a check at the top of the `if !devour_sacrifices.is_empty()` block that the spell's card definition includes at least one `Devour(n)` ability, returning an error otherwise. |
| 2 | LOW | `tests/devour.rs` | **No test for non-Devour spell with devour_sacrifices.** Missing negative test that passing `devour_sacrifices` on a spell without Devour is either rejected or silently ignored. **Fix:** Add a test that casts a non-Devour creature with populated `devour_sacrifices` and verifies the behavior (error or no sacrifice). |
| 3 | LOW | `resolution.rs:874` | **Controller lookup uses unwrap_or fallback.** `state.objects.get(&new_id).map(|o| o.controller).unwrap_or(stack_obj.controller)` -- the `unwrap_or` fallback is reasonable but the object should always exist at this point since it was just created by `move_object_to_zone`. **Fix:** Consider `debug_assert!` that the object exists, or leave as-is (defensive). |
| 4 | LOW | `replay_harness.rs:1560` | **Comment says "convoke_names (reused parameter slot)" for devour creature names.** The plan specified a `devour_creatures` JSON field but the implementation reuses `convoke_names`. This works but is confusing for script authors. **Fix:** Document in game-scripts.md that `cast_spell_devour` uses `convoke_names` for sacrifice targets. |

### Finding Details

#### Finding 1: Missing Devour keyword check on spell

**Severity**: LOW
**File**: `crates/engine/src/rules/casting.rs:2748`
**CR Rule**: 702.82a -- "Devour is a static ability."
**Issue**: The cast-time validation checks that each sacrifice target is valid (battlefield, controlled by caster, is a creature, no duplicates) but does not verify that the spell being cast actually has `KeywordAbility::Devour(_)`. If a non-Devour creature is cast with `devour_sacrifices` populated, validation passes and the sacrifices are silently ignored at resolution time (since `devour_instances` would be empty). No game state corruption occurs -- the creatures are not sacrificed -- but it's a lax validation that could mask client bugs.
**Fix**: Add a guard at the top of the `if !devour_sacrifices.is_empty()` block that checks the spell's card definition for at least one `Devour(n)` ability. Return `GameStateError::InvalidCommand` if none found. This matches the pattern used by other keyword validations (e.g., Amplify checks for the keyword before processing `amplify_reveals`).

#### Finding 2: No test for non-Devour spell with devour_sacrifices

**Severity**: LOW
**File**: `crates/engine/tests/devour.rs`
**CR Rule**: 702.82a
**Issue**: No negative test verifies what happens when `devour_sacrifices` is populated on a spell without the Devour keyword ability. This edge case should either be rejected (if Finding 1 is fixed) or verified to be harmless (if Finding 1 is deferred).
**Fix**: Add `test_devour_non_devour_spell_with_sacrifices` that constructs a CastSpell for a non-Devour creature with populated `devour_sacrifices` and asserts either an error or no sacrifice.

#### Finding 3: Controller lookup fallback

**Severity**: LOW
**File**: `crates/engine/src/rules/resolution.rs:874`
**CR Rule**: 702.82a
**Issue**: The controller lookup `state.objects.get(&new_id).map(|o| o.controller).unwrap_or(stack_obj.controller)` uses a silent fallback. The object should always exist since it was just created, so the fallback masks a potential bug if `move_object_to_zone` fails silently.
**Fix**: Add `debug_assert!(state.objects.contains_key(&new_id))` before the lookup, or leave as-is with a comment explaining the defensive fallback. This matches the existing pattern in Amplify/Bloodthirst blocks.

#### Finding 4: Harness parameter naming

**Severity**: LOW
**File**: `crates/engine/src/testing/replay_harness.rs:1560`
**CR Rule**: N/A (convention)
**Issue**: The `cast_spell_devour` harness action reuses the `convoke_names` parameter for devour sacrifice creature names. This is pragmatic (avoids adding a new parameter) but the comment acknowledges it's a "reused parameter slot". Script authors may be confused.
**Fix**: Document in `docs/mtg-engine-game-scripts.md` that `cast_spell_devour` uses the `convoke_names` JSON array for sacrifice targets.

## CR Coverage Check

| CR Subrule | Implemented? | Tested? | Notes |
|------------|-------------|---------|-------|
| 702.82a (sacrifice + counters) | Yes | Yes | Tests 1-5, 8 cover basic sacrifice, multiplier, zero sacrifice, no-eligible |
| 702.82a (optional -- "you may") | Yes | Yes | Test 4: zero sacrifice is valid |
| 702.82a (creatures only) | Yes | Yes | Validated at cast time (creature type check via layers) |
| 702.82a (controller only) | Yes | Yes | Test 6: opponent's creatures rejected |
| 702.82a (not self) | Yes | Yes | Test 7: self-sacrifice rejected (battlefield check catches this) |
| 702.82b (creatures_devoured tracking) | Yes | Yes | Test 10: creatures_devoured field set correctly |
| 702.82c (multiple instances) | Yes | Yes | Test 9: Devour 1 + Devour 2 = 6 counters for 2 sacrifices |
| 702.82c (Devour [quality] variant) | No | No | Not implemented -- deferred (no test cards with Devour [quality]) |
| Replacement effects on sacrifice | Yes | No | Code handles Rest in Peace / commander redirect but no test |
| Death triggers from sacrifice | Yes | Yes | Test 8 verifies CreatureDied events emitted |
| Zone change resets (CR 400.7) | Yes | No | creatures_devoured reset at both move_object_to_zone sites |

## Previous Findings (re-review only)

N/A -- first review.
