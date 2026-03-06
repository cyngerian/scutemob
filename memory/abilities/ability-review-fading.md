# Ability Review: Fading

**Date**: 2026-03-06
**Reviewer**: ability-impl-reviewer (Opus)
**CR**: 702.32
**Files reviewed**: `crates/engine/src/state/types.rs`, `crates/engine/src/state/hash.rs`, `crates/engine/src/state/stack.rs`, `crates/engine/src/state/stubs.rs`, `crates/engine/src/cards/card_definition.rs`, `crates/engine/src/rules/resolution.rs`, `crates/engine/src/rules/turn_actions.rs`, `crates/engine/src/rules/abilities.rs`, `crates/engine/src/rules/lands.rs`, `tools/tui/src/play/panels/stack_view.rs`, `tools/replay-viewer/src/view_model.rs`, `crates/engine/tests/fading.rs`

## Verdict: clean

The Fading implementation correctly follows CR 702.32a. The single-trigger design (counter removal + sacrifice in one trigger) is the right approach per the CR text. ETB counter placement is present in both resolution.rs and lands.rs (the dual-site requirement). Hash discriminants are unique and properly ordered. The trigger queueing correctly fires unconditionally (no intervening-if), matching the CR text. The sacrifice path correctly handles replacement effects, commander zone redirection, and zone-change tracking. Tests cover the full lifecycle, multiplayer scoping, non-creature permanents, and fade-vs-time counter distinction. All 8 tests cite CR 702.32a. No HIGH or MEDIUM findings.

## Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | LOW | `tests/fading.rs:672` | **CreatureDied event for non-creature permanent.** Test correctly documents the engine behavior, but the comment "CreatureDied event should be emitted for non-creature Fading sacrifice" highlights that the engine uses `CreatureDied` for all permanent deaths including enchantments. This is a pre-existing engine-wide naming issue, not a Fading bug. **Fix:** No action needed for Fading. Track as part of the existing LOW issue for renaming `CreatureDied` to `PermanentDied`. |
| 2 | LOW | `tests/fading.rs:203` | **ETB test does not verify counter placement.** `test_fading_etb_places_fade_counters` only checks the keyword is present -- it does not place counters because ObjectSpec-built objects bypass the ETB resolution path. The actual ETB counter placement is tested by `test_fading_etb_counters_on_cast` (test 2), so this is a naming/clarity issue, not a gap. **Fix:** Consider renaming to `test_fading_keyword_present_on_battlefield` or adding a comment noting that counter placement is tested in the next test. |

### Finding Details

#### Finding 1: CreatureDied event for non-creature permanent

**Severity**: LOW
**File**: `crates/engine/tests/fading.rs:672`
**CR Rule**: 702.32a -- "sacrifice the permanent"
**Issue**: The test asserts `CreatureDied` for a non-creature enchantment sacrifice. This is technically correct per the current engine design (all permanent-to-graveyard events use `CreatureDied`), and the test documents it with a comment. However, if the engine later adds a `PermanentSacrificed` event, this test would need updating.
**Fix**: No immediate action. This is a pre-existing engine naming issue tracked in the LOW remediation doc.

#### Finding 2: ETB test naming vs content mismatch

**Severity**: LOW
**File**: `crates/engine/tests/fading.rs:203`
**CR Rule**: 702.32a -- "This permanent enters with N fade counters on it"
**Issue**: The test `test_fading_etb_places_fade_counters` is named as if it tests counter placement, but it only verifies the keyword is present on the object. Counter placement through the CastSpell flow is tested by the separate `test_fading_etb_counters_on_cast` test. The first test is not wrong, but its name promises more than it delivers.
**Fix**: Consider renaming to `test_fading_keyword_present_on_battlefield` or adding a brief comment explaining that ETB counter placement requires the cast flow (tested in the next test).

## CR Coverage Check

| CR Subrule | Implemented? | Tested? | Notes |
|------------|-------------|---------|-------|
| 702.32a (ETB counters) | Yes | Yes | `test_fading_etb_counters_on_cast` -- resolution.rs + lands.rs |
| 702.32a (upkeep counter removal) | Yes | Yes | `test_fading_upkeep_removes_counter` |
| 702.32a (sacrifice when can't remove) | Yes | Yes | `test_fading_sacrifice_when_no_counters` |
| 702.32a (full lifecycle N+1) | Yes | Yes | `test_fading_full_lifecycle` (Fading 2 = 3 upkeeps) |
| 702.32a ("your upkeep" multiplayer) | Yes | Yes | `test_fading_multiplayer_only_active_player` |
| 702.32a (non-creature permanent) | Yes | Yes | `test_fading_non_creature_sacrifice` |
| 702.32a (fade vs time counters) | Yes | Yes | `test_fading_uses_fade_counters_not_time` |
| 702.32a (no intervening-if) | Yes | Yes | Trigger queues unconditionally in turn_actions.rs; sacrifice-at-0 tested |
| Multiple Fading instances | Yes | No | turn_actions.rs counts instances; no dedicated test |

verdict: clean
