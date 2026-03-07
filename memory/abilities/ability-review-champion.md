# Ability Review: Champion

**Date**: 2026-03-07
**Reviewer**: ability-impl-reviewer (Opus)
**CR**: 702.72
**Files reviewed**:
- `crates/engine/src/state/types.rs` (lines 1153-1177)
- `crates/engine/src/state/hash.rs` (lines 635-650, 837, 1863-1879, 3737-3741)
- `crates/engine/src/state/game_object.rs` (lines 430-486)
- `crates/engine/src/state/stubs.rs` (lines 97-102, 106-300+)
- `crates/engine/src/state/stack.rs` (lines 1-80)
- `crates/engine/src/state/mod.rs` (lines 366-368, 507-509)
- `crates/engine/src/cards/card_definition.rs` (lines 532-540)
- `crates/engine/src/cards/helpers.rs` (line 11)
- `crates/engine/src/rules/abilities.rs` (lines 2372-2430, 3506-3555, 4066-4220, 4838-4854)
- `crates/engine/src/rules/resolution.rs` (lines 2751-3003, 4999-5000)
- `crates/engine/src/testing/replay_harness.rs` (lines 1777-1783)
- `tools/replay-viewer/src/view_model.rs` (lines 567-570, 843)
- `tools/tui/src/play/panels/stack_view.rs` (lines 177-181)
- `crates/engine/tests/champion.rs` (all 1192 lines)

## Verdict: needs-fix

Two MEDIUM findings: (1) inconsistent keyword guard on LTB triggers across zone-departure events creates a correctness gap when Champion keyword is removed before exile/bounce, and (2) the LTB return-to-battlefield path is entirely untested -- Test 3 only checks `champion_exiled_card` is set but never kills the champion and verifies the card returns.

## Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | **MEDIUM** | `abilities.rs:4122-4125,4177-4180` | **Inconsistent keyword guard on LTB triggers.** ObjectExiled and ObjectReturnedToHand check `KeywordAbility::Champion` but CreatureDied and PermanentDestroyed do not. **Fix:** remove keyword check. |
| 2 | **MEDIUM** | `champion.rs:547-664` | **LTB return path untested.** Test 3 never kills the champion; only checks `champion_exiled_card` is set. **Fix:** add a test that kills the champion and verifies the exiled card returns to battlefield. |
| 3 | LOW | `abilities.rs` | **Missing LTB trigger for tuck-to-library.** No `ObjectPutIntoLibrary` event exists. Engine-wide gap. |
| 4 | LOW | `champion.rs:1087-1095` | **Changeling test assertion is weak.** Accepts either exile or sacrifice outcome. |

### Finding Details

#### Finding 1: Inconsistent keyword guard on LTB triggers

**Severity**: MEDIUM
**File**: `crates/engine/src/rules/abilities.rs:4122-4125` and `crates/engine/src/rules/abilities.rs:4177-4180`
**CR Rule**: 603.10a -- "Some zone-change triggers look back in time. These are leaves-the-battlefield abilities [...]"
**Issue**: The `ObjectExiled` arm (line 4122-4125) and `ObjectReturnedToHand` arm (line 4177-4180) check whether the post-move object still has `KeywordAbility::Champion` in its keywords before firing the LTB trigger. The `CreatureDied` arm (line 3512) and `PermanentDestroyed` arm (line 4069) do NOT check this -- they only check `champion_exiled_card`. This inconsistency means:

1. If a champion has its Champion keyword removed (e.g., by Humility) and is then exiled or bounced, the LTB trigger will NOT fire despite `champion_exiled_card` being set. Per CR 603.10a, LTB triggers use last-known information, and the `champion_exiled_card` field being set is conclusive evidence that the permanent championed something.

2. If a champion with its keyword removed dies, the LTB DOES fire (from the CreatureDied arm), but if it's exiled, it does NOT fire. This is an internal inconsistency.

The `champion_exiled_card` field is sufficient for detecting LTB triggers. The keyword check adds nothing except a false-negative path.

**Fix**: Remove the `KeywordAbility::Champion` check from both the `ObjectExiled` arm (lines 4121-4125) and `ObjectReturnedToHand` arm (lines 4177-4180). Only check `champion_exiled_card.is_some()`, matching the pattern used in `CreatureDied` and `PermanentDestroyed`.

#### Finding 2: LTB return path untested

**Severity**: MEDIUM
**File**: `crates/engine/tests/champion.rs:547-664`
**CR Rule**: 702.72a -- "When this permanent leaves the battlefield, return the exiled card to the battlefield under its owner's control."
**Issue**: `test_champion_ltb_returns_exiled_card` (Test 3) sets up a champion that exiles a fodder creature, then verifies that `champion_exiled_card` is set on the champion object. However, the test NEVER kills the champion and verifies that the LTB trigger fires and returns the exiled card to the battlefield. The test comment at line 642-645 acknowledges this: "LTB return is validated by the game script." But no game script exists yet (step 6 is unchecked in ability-wip.md).

This means the entire `ChampionLTBTrigger` resolution path in `resolution.rs:2930-3003` -- including the owner-control assignment, the ETB pipeline for the returned card, and the exile-zone existence check -- is completely untested.

**Fix**: Extend the test (or add a new test) that:
1. Casts the champion, resolves ETB (exiles fodder)
2. Destroys/sacrifices the champion (e.g., via DealDamage with lethal damage + SBA check, or a direct `move_object_to_zone` if needed)
3. Resolves the LTB trigger
4. Asserts the fodder creature is back on the battlefield (not in exile)
5. Asserts the fodder is under its owner's control

#### Finding 3: Missing LTB trigger for tuck-to-library

**Severity**: LOW
**File**: `crates/engine/src/rules/abilities.rs`
**CR Rule**: 702.72a -- "When this permanent leaves the battlefield" includes being put into a library.
**Issue**: The engine has no `ObjectPutIntoLibrary` or `ObjectShuffledIntoLibrary` event. If a champion is tucked to the bottom of its owner's library (e.g., by Hinder, Terminus, Condemn), no LTB trigger will fire because there is no zone-departure event to hook into. This is an engine-wide gap affecting all LTB triggers, not specific to Champion.
**Fix**: Defer. When an `ObjectPutIntoLibrary` event is added to the engine, add a Champion LTB check in its arm (matching the `CreatureDied`/`PermanentDestroyed` pattern -- check `champion_exiled_card` only, no keyword guard).

#### Finding 4: Changeling test assertion is weak

**Severity**: LOW
**File**: `crates/engine/tests/champion.rs:1087-1095`
**CR Rule**: 702.72a + Changeling (702.73) -- a Changeling creature has all creature types, so it should satisfy "Champion a Faerie."
**Issue**: The test assertion accepts either outcome: (a) changeling exiled and champion on battlefield, or (b) champion sacrificed. The comment says "if Changeling didn't resolve to all subtypes via layers" the champion would sacrifice itself. A correct implementation MUST exile the Changeling. The weak assertion masks a potential bug where Changeling subtypes aren't resolved correctly through layers.
**Fix**: Tighten the assertion to require that the Changeling is exiled and the champion stays on the battlefield. If this fails, the underlying issue is that layer-resolved characteristics don't include Changeling subtypes for the `ChampionFilter::Subtype` check, which should be fixed in resolution.rs.

## CR Coverage Check

| CR Subrule | Implemented? | Tested? | Notes |
|------------|-------------|---------|-------|
| 702.72a ETB ("sacrifice unless exile") | Yes | Yes | Tests 1, 2, 5, 6, 7, 8 |
| 702.72a LTB ("return exiled card") | Yes | **No** | Resolution code exists but no test kills champion + verifies return |
| 702.72a "another [object]" | Yes | Yes | Test 8 (cannot target self) |
| 702.72a "under its owner's control" | Yes | **No** | Code sets `obj.controller = owner` but no test verifies ownership vs control distinction |
| 702.72b Linked abilities (607.2k) | Yes | Partial | `champion_exiled_card` tracking tested, but LTB return not tested |
| 702.72c "championed" designation | N/A | N/A | No game effect depends on this status word |
| Subtype filter | Yes | Yes | Tests 5, 6 |
| Changeling interaction | Yes | Weak | Test 7 has weak assertion |
| No-target sacrifice | Yes | Yes | Tests 2, 8 |
| LTB on CreatureDied | Yes | No | Code at abilities.rs:3512 |
| LTB on PermanentDestroyed | Yes | No | Code at abilities.rs:4069 |
| LTB on ObjectExiled | Yes (with bug) | No | Code at abilities.rs:4117 -- has extra keyword check |
| LTB on ObjectReturnedToHand | Yes (with bug) | No | Code at abilities.rs:4173 -- has extra keyword check |
| LTB on tuck-to-library | No | No | Engine-wide gap (no event) |
