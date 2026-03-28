# Primitive Batch Review: PB-34 -- Mana Production (Filter Lands, Devotion, Conditional)

**Date**: 2026-03-27
**Reviewer**: primitive-impl-reviewer (Opus)
**CR Rules**: 605.1a (activated mana abilities), 605.3b (mana abilities don't use stack), 602.2 (activation costs), 700.5 (devotion)
**Engine files reviewed**: `crates/engine/src/cards/card_definition.rs` (lines 1107-1115), `crates/engine/src/effects/mod.rs` (lines 1361-1385), `crates/engine/src/state/hash.rs` (lines 4505-4515), `crates/engine/src/testing/replay_harness.rs` (lines 1881-1888, 2886-2915)
**Card defs reviewed**: 7 (fetid_heath, rugged_prairie, twilight_mire, flooded_grove, cascade_bluffs, sunken_ruins, graven_cairns)

## Verdict: clean

All engine changes are correct and well-structured. The new `Effect::AddManaFilterChoice` variant is properly defined, executed, hashed, and recognized in the replay harness. All 7 filter land card definitions match their oracle text exactly (type, abilities, colors). The `AddManaScaled` orphan bug fix correctly extends `try_as_tap_mana_ability` to register scaled-mana abilities. Hash discriminant 73 is unique within the Effect match block. Tests cover the positive case (mana production), negative case (tapped land rejection), all 7 cards parametrically, and the orphan bug fix across 6 affected cards. No TODOs remain in any modified card def. No HIGH or MEDIUM findings.

## Engine Change Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | **LOW** | `hash.rs:4867,4995` | **Pre-existing hash discriminant collision (70).** `ExileWithDelayedReturn` and `PreventCombatDamageFromOrTo` both use discriminant 70 in the Effect hash block. Not introduced by PB-34 but worth noting. **Fix:** Assign unique discriminant to one of these in a future cleanup pass. |
| 2 | **LOW** | `replay_harness.rs:2888` | **AddManaFilterChoice arm in try_as_tap_mana_ability is currently unreachable.** Filter lands use `Cost::Sequence`, not `Cost::Tap`, so the guard at line 1860 prevents this path. The code is defensive/future-proof and harmless. No fix needed. |

## Card Definition Findings

| # | Severity | Card | Description |
|---|----------|------|-------------|
| (none) | | | All 7 card defs match oracle text exactly. |

## CR Coverage Check

| CR Rule | Implemented? | Tested? | Notes |
|---------|-------------|---------|-------|
| 605.1a (activated mana ability criteria) | Yes | Yes | test_filter_land_produces_two_mana_fetid_heath, test_all_filter_lands_produce_correct_colors |
| 605.3b (mana ability doesn't use stack) | Partial | No | Filter lands use Cost::Sequence so they go through ActivateAbility (stack). Pre-existing architectural limitation shared with Phyrexian Tower. Documented in plan. |
| 602.2 (activation costs must be paid) | Yes | Yes | test_filter_land_tap_required verifies tap requirement. Hybrid mana cost enforcement is a pre-existing gap (documented in test file). |
| 700.5 (devotion) | Deferred | N/A | Nykthos/Three Tree City deferred to M10 (requires Command::ChooseColor). AddManaScaled + DevotionTo execution verified existing in effects/mod.rs. |

## Card Def Summary

| Card | Oracle Match | TODOs Remaining | Game State Correct | Notes |
|------|-------------|-----------------|-------------------|-------|
| fetid_heath | Yes | 0 | Yes | W/B filter, produces 1W+1B |
| rugged_prairie | Yes | 0 | Yes | R/W filter, produces 1R+1W |
| twilight_mire | Yes | 0 | Yes | B/G filter, produces 1B+1G |
| flooded_grove | Yes | 0 | Yes | G/U filter, produces 1G+1U |
| cascade_bluffs | Yes | 0 | Yes | U/R filter, produces 1U+1R |
| sunken_ruins | Yes | 0 | Yes | U/B filter, produces 1U+1B |
| graven_cairns | Yes | 0 | Yes | B/R filter, produces 1B+1R |

### Finding Details

#### Finding 1: Pre-existing hash discriminant collision (70)

**Severity**: LOW
**File**: `crates/engine/src/state/hash.rs:4867` and `crates/engine/src/state/hash.rs:4995`
**CR Rule**: N/A (architecture invariant -- hash uniqueness)
**Issue**: `ExileWithDelayedReturn` (line 4867) and `PreventCombatDamageFromOrTo` (line 4995) both use discriminant `70u8` within the `impl HashInto for Effect` match block. This means two structurally different Effect variants produce the same hash prefix, which could cause hash collisions in state comparison. This is a pre-existing issue from PB-33 or earlier, not introduced by PB-34.
**Fix**: In a future cleanup pass, assign a unique discriminant (74 or higher) to `PreventCombatDamageFromOrTo` to eliminate the collision.

#### Finding 2: Unreachable AddManaFilterChoice arm in try_as_tap_mana_ability

**Severity**: LOW
**File**: `crates/engine/src/testing/replay_harness.rs:2888`
**CR Rule**: N/A (code quality)
**Issue**: The `AddManaFilterChoice` recognition arm in `try_as_tap_mana_ability` is never reached because all current filter land card defs use `Cost::Sequence([Mana, Tap])`, not `Cost::Tap`. The function is only called for `Cost::Tap` abilities (line 1860). The code is defensive and correctly handles the hypothetical case of a filter-style effect with just a tap cost.
**Fix**: No action needed. The arm serves as future-proofing.
