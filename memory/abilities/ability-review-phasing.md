# Ability Review: Phasing

**Date**: 2026-03-06
**Reviewer**: ability-impl-reviewer (Opus)
**CR**: 702.26
**Files reviewed**:
- `crates/engine/src/state/types.rs` (lines 1082-1094)
- `crates/engine/src/state/game_object.rs` (lines 534-561)
- `crates/engine/src/state/hash.rs` (lines 604, 788-790, 2700-2711)
- `crates/engine/src/state/mod.rs` (lines 362-363, 498-499)
- `crates/engine/src/state/builder.rs` (lines 960-961)
- `crates/engine/src/rules/turn_actions.rs` (lines 672-801)
- `crates/engine/src/rules/events.rs` (lines 961-983)
- `crates/engine/src/rules/sba.rs` (lines 129, 163, 643, 758, 1003)
- `crates/engine/src/rules/layers.rs` (lines 187, 216-227)
- `crates/engine/src/rules/combat.rs` (lines 78, 519)
- `crates/engine/src/rules/abilities.rs` (15 battlefield scans, no phased-out filter)
- `crates/engine/src/effects/mod.rs` (16 battlefield scans, no phased-out filter)
- `crates/engine/src/rules/casting.rs` (no phased-out filter)
- `crates/engine/src/rules/replacement.rs` (no phased-out filter)
- `crates/engine/src/rules/engine.rs` (no phased-out filter)
- `crates/engine/src/rules/resolution.rs` (init sites only)
- `tools/replay-viewer/src/view_model.rs` (line 819)
- `crates/engine/tests/phasing.rs` (15 tests)

## Verdict: needs-fix

The phasing implementation covers the core mechanics well -- enum variant, hash coverage,
new fields, SBA/layer/combat filtering, and events are all correctly implemented. However,
there is one HIGH finding (CR 502.1 simultaneous phasing violated by sequential
implementation) and two MEDIUM findings (CR 702.26h direct-vs-indirect handling is inverted,
and large swathes of battlefield scans in abilities.rs and effects/mod.rs lack phased-out
filtering per CR 702.26b). The HIGH must be fixed; the MEDIUMs should be addressed before
the ability is considered complete.

## Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | **HIGH** | `turn_actions.rs:686-793` | **CR 502.1 simultaneous phasing violated.** Phase-in and phase-out are done sequentially; a creature with Phasing that phases in immediately phases out again. **Fix:** snapshot both sets before mutating. |
| 2 | **HIGH** | `tests/phasing.rs:165-176` | **Test 2 asserts wrong outcome.** Asserts creature ends up phased out after phase-in + re-phase-out; CR 502.1 says it should end up phased IN. **Fix:** invert assertion after fixing #1. |
| 3 | MEDIUM | `turn_actions.rs:767-779` | **CR 702.26h direct+indirect resolution inverted.** Equipment with Phasing attached to Phasing creature phases out directly instead of indirectly. **Fix:** mark as indirect when in both sets. |
| 4 | MEDIUM | `abilities.rs` (15 sites) | **Missing phased-out filter on trigger source scans (CR 702.26b).** Phased-out permanents could fire triggers (Prowess, Evolve, etc.). **Fix:** add `&& obj.is_phased_in()` to all 15 battlefield filters. |
| 5 | MEDIUM | `effects/mod.rs` (16 sites) | **Missing phased-out filter on effect resolution scans (CR 702.26b).** Effects like "each creature" could include phased-out permanents. **Fix:** add `&& obj.is_phased_in()` or `&& !obj.status.phased_out` to all 16 battlefield filters. |
| 6 | MEDIUM | `tests/phasing.rs` | **Missing test for CR 702.26e (continuous effects exclusion).** Plan test #10 not implemented. **Fix:** add test with a global +1/+1 effect; verify phased-out creature is NOT buffed. |
| 7 | LOW | `casting.rs`, `engine.rs`, `replacement.rs` | **Missing phased-out filter on targeting/command validation (CR 702.26b).** These are lower risk because targeting phased-out permanents is unlikely in current card pool. **Fix:** add phased-out checks to battlefield filters in these files. |

### Finding Details

#### Finding 1: CR 502.1 simultaneous phasing violated

**Severity**: HIGH
**File**: `crates/engine/src/rules/turn_actions.rs:686-793`
**CR Rule**: 502.1 -- "all phased-in permanents with phasing that the active player controls phase out, and all phased-out permanents that the active player controlled when they phased out phase in. This all happens simultaneously."
**Issue**: The implementation performs phase-in (lines 686-733) and phase-out (lines 739-793) sequentially. This means a creature with the Phasing keyword that phases in during step 1 will be re-collected in step 2's phase-out scan (because it is now phased-in and has Phasing), causing it to immediately phase out again. Per CR 502.1, both sets should be determined simultaneously before any mutations occur, so the creature that phases in should NOT be in the phase-out set.
**Fix**: Before any mutations, snapshot both sets:
1. Collect `phase_in_ids` (phased-out permanents controlled by active, not indirect)
2. Collect `phase_out_ids` (phased-in permanents with Phasing controlled by active)
3. Then mutate: phase in all from set 1 (and their indirect attachments), phase out all from set 2 (and their indirect attachments)

This ensures a creature with Phasing that was phased out appears in set 1 (phase-in) but NOT in set 2 (because it was phased-out when the snapshot was taken).

#### Finding 2: Test 2 asserts wrong outcome

**Severity**: HIGH
**File**: `crates/engine/tests/phasing.rs:165-176`
**CR Rule**: 502.1 -- simultaneous phasing
**Issue**: The test comment says "creature phases in then immediately phases out (has Phasing keyword)" and asserts `is_phased_out == true`. Per CR 502.1, phasing in and out happen simultaneously. A creature with Phasing that was phased out phases in but does NOT also phase out in the same simultaneous event. On the NEXT untap step, it would phase out again. The correct assertion is `is_phased_out == false`.
**Fix**: After fixing Finding 1, change the assertion to `assert!(!is_phased_out(...))` and update the comment to explain CR 502.1 simultaneous phasing.

#### Finding 3: CR 702.26h direct+indirect resolution inverted

**Severity**: MEDIUM
**File**: `crates/engine/src/rules/turn_actions.rs:767-779`
**CR Rule**: 702.26h -- "If an object would simultaneously phase out directly and indirectly, it just phases out indirectly."
**Issue**: When an attachment (e.g., Equipment with Phasing) is in both `phase_out_direct` (has Phasing keyword) and is attached to a host that also has Phasing, the code at line 770 skips the indirect phase-out processing. The attachment is then processed later as a direct phase-out (lines 782-785, `phased_out_indirectly` remains false). CR 702.26h requires it to phase out **indirectly**, not directly. The practical consequence: when the host phases back in, the Equipment should phase in WITH it (indirect behavior), but since it's marked as direct, it would attempt to phase in independently on the next untap step.
**Fix**: When an attachment is in both `phase_out_direct` and is an attachment of another phasing-out host, mark it as phasing out indirectly (`phased_out_indirectly = true`). Remove it from the direct processing loop, or set the indirect flag when processing it as a host.

#### Finding 4: Missing phased-out filter on trigger source scans

**Severity**: MEDIUM
**File**: `crates/engine/src/rules/abilities.rs` (15 battlefield scan sites)
**CR Rule**: 702.26b -- "a phased-out permanent is treated as though it does not exist. It can't affect or be affected by anything else in the game."
**Issue**: All 15 `obj.zone == ZoneId::Battlefield` filters in abilities.rs lack `&& obj.is_phased_in()` checks. This means phased-out permanents with Prowess, Evolve, Renown, Training, Melee, Flanking, Bushido, etc. could incorrectly fire triggers. Lines affected: 2300, 2442, 2465, 2487, 2781, 2826, 2984, 3236, 3257, 3358, 3444, 3523, 3613, 3656, 3875.
**Fix**: Add `&& obj.is_phased_in()` (or `&& !obj.status.phased_out`) to each of these 15 filters.

#### Finding 5: Missing phased-out filter on effect resolution scans

**Severity**: MEDIUM
**File**: `crates/engine/src/effects/mod.rs` (16 battlefield scan sites)
**CR Rule**: 702.26b -- "a phased-out permanent is treated as though it does not exist."
**Issue**: All 16 `zone == ZoneId::Battlefield` filters in effects/mod.rs lack phased-out checks. This means effects like "destroy all creatures" or "each creature gets -X/-X" could incorrectly affect phased-out permanents. Lines affected: 476, 1016, 1420, 1573, 1627, 1839, 2044, 2187, 2195, 2202, 2753, 2756, 2761, 2816, 2825, 2835.
**Fix**: Add `&& !obj.status.phased_out` (or `&& obj.is_phased_in()`) to each of these 16 filters. Note: some of these (like line 476 which checks if an object is on the battlefield as a boolean) may need contextual review -- if the code is checking a specific object's zone, the phased-out filter may or may not be appropriate depending on whether the effect specifically mentions phased-out permanents.

#### Finding 6: Missing test for CR 702.26e

**Severity**: MEDIUM
**File**: `crates/engine/tests/phasing.rs`
**CR Rule**: 702.26e -- "a phased-out permanent won't be included in the set of affected objects"
**Issue**: The plan specified test #10 (`test_phasing_excluded_from_continuous_effects`) to verify that a global +1/+1 effect does not apply to phased-out creatures and that it applies when they phase back in. This test was not implemented, leaving a gap in coverage for a core CR subrule. The layer system implementation (layers.rs:216-227) does implement this filtering, but it is untested.
**Fix**: Add a test that creates a continuous effect (e.g., an enchantment that gives all creatures +1/+1), phases out a creature, and verifies via `calculate_characteristics` that the phased-out creature does NOT receive the bonus.

#### Finding 7: Missing phased-out filter in targeting, commands, replacement effects

**Severity**: LOW
**File**: `crates/engine/src/rules/casting.rs`, `crates/engine/src/rules/engine.rs`, `crates/engine/src/rules/replacement.rs`
**CR Rule**: 702.26b -- "It can't affect or be affected by anything else in the game."
**Issue**: The plan identified filter sites in casting.rs (target validation), engine.rs (command validation), and replacement.rs (source checks) that need phased-out filtering. None of these were implemented. The risk is lower because targeting a phased-out permanent requires constructing a specific scenario unlikely in the current card pool, but it represents an incomplete enforcement of CR 702.26b.
**Fix**: Add `!obj.status.phased_out` checks to battlefield filters in these three files at the sites identified in the plan: casting.rs (lines ~2364, ~4122, ~4341), engine.rs (lines ~561, ~727), replacement.rs (line ~231).

## CR Coverage Check

| CR Subrule | Implemented? | Tested? | Notes |
|------------|-------------|---------|-------|
| 702.26a (phase out on untap) | Yes | Yes | test_phasing_basic_phase_out_on_untap |
| 702.26a (phase in on untap) | Yes | Yes | test_phasing_phase_in_without_keyword |
| 702.26a (simultaneous) | **No** | **Wrong** | Finding #1/#2: sequential, not simultaneous |
| 702.26b (treated as nonexistent) | Partial | Yes | SBA/layer/combat filtered; abilities/effects/casting/engine NOT filtered |
| 702.26c (phase in status) | Yes | Yes | Implicit in phase-in tests |
| 702.26d (no zone change) | Yes | Yes | test_phasing_no_zone_change_preserves_object_id_and_counters |
| 702.26d (no ETB triggers) | Yes | Yes | test_phasing_no_etb_triggers_on_phase_in |
| 702.26d (tokens survive) | Yes | Yes | test_phasing_token_survives_phase_out |
| 702.26d (counters persist) | Yes | Yes | test_phasing_no_zone_change_preserves_object_id_and_counters |
| 702.26e (continuous effects) | Yes (layers.rs) | **No** | Finding #6: test missing |
| 702.26f (effects expire) | Deferred | No | "For as long as" duration tracking not tested |
| 702.26g (indirect phasing) | Yes | Yes | test_phasing_indirect_aura_phases_out_with_host |
| 702.26g (indirect phase-in) | Yes | Yes | test_phasing_indirect_phases_in_together |
| 702.26h (direct+indirect) | **Wrong** | No | Finding #3: inverted logic |
| 702.26i (direct Aura phase-in) | Deferred | No | Plan explicitly defers |
| 702.26j (no attach triggers) | Yes (implicit) | No | No attach/detach triggers exist in engine yet |
| 702.26k (player leaves) | Deferred | No | Plan explicitly defers |
| 702.26m (skip untap) | Not tested | No | Engine has step-skipping infra but no phasing test |
| 702.26n (multiplayer leave) | Deferred | No | Plan explicitly defers |
| 702.26p (redundant) | Yes | Yes | test_phasing_redundant_instances |
| 506.4 (removed from combat) | Partial | Partial | Attack/block declaration tested; mid-combat removal not tested |

verdict: needs-fix
