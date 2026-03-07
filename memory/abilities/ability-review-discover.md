# Ability Review: Discover

**Date**: 2026-03-07
**Reviewer**: ability-impl-reviewer (Opus)
**CR**: 701.57
**Files reviewed**: `crates/engine/src/state/types.rs:1247-1263`, `crates/engine/src/state/hash.rs:663,2961-2979,3601`, `crates/engine/src/rules/copy.rs:509-718`, `crates/engine/src/rules/events.rs:660-692`, `crates/engine/src/effects/mod.rs:1854-1878`, `crates/engine/src/cards/card_definition.rs:937-958`, `tools/replay-viewer/src/view_model.rs:863`, `crates/engine/tests/discover.rs`

## Verdict: needs-fix

The implementation is largely correct and well-structured. The core MV <= N threshold (vs Cascade's < N) is implemented correctly, the hand fallback path exists, and the library-bottom cleanup works. However, there is one MEDIUM finding: the `DiscoverToHand` path is unreachable in normal operation (only fires on a move-to-stack failure), meaning the CR 701.57a "If you don't cast it, put that card into your hand" behavior is not exercisable even deterministically. There is also a doc comment inconsistency (LOW) on the `DiscoverExiled` event.

## Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | MEDIUM | `copy.rs:594-625` | **DiscoverToHand path unreachable in normal play.** The hand fallback only fires on move-to-stack Err. **Fix:** Add explicit DiscoverToHand for lands discovered (when library has only nonland with MV > N), or document deterministic limitation. |
| 2 | LOW | `events.rs:663` | **DiscoverExiled doc comment inaccurate.** Says "including the discovered card" but code pops it before emitting. **Fix:** Update doc comment. |
| 3 | LOW | `discover.rs` | **No test for DiscoverToHand event.** Plan test 5 acknowledged this gap but no workaround was implemented. **Fix:** Add test or accept as deferred. |

### Finding Details

#### Finding 1: DiscoverToHand path unreachable in normal play

**Severity**: MEDIUM
**File**: `crates/engine/src/rules/copy.rs:594-625`
**CR Rule**: 701.57a -- "If you don't cast it, put that card into your hand."
**Issue**: The current deterministic mode always casts the discovered card (line 594: "Deterministic fallback: always cast the discovered card"). The `DiscoverToHand` event is only emitted inside the `Err(_)` branch of `move_object_to_zone(exile_id, ZoneId::Stack)` (line 607), which is described as "unreachable in well-formed game state." This means `DiscoverToHand` is dead code in practice.

While the deterministic "always cast" policy is a reasonable M9.5 simplification (matching Cascade's behavior), the hand fallback is the defining behavioral difference between Discover and Cascade. The code currently has no reachable path to exercise it.

This is MEDIUM rather than HIGH because:
1. The deterministic "always cast" policy is explicitly documented and deferred to M10+.
2. The `DiscoverToHand` event and hand-move logic exist and are structurally correct.
3. No game state corruption results from the current behavior.

**Fix:** Accept as a known M10 deferral. Add a `// TODO(M10): When player choice is implemented, add a branch here for the "decline to cast" path that emits DiscoverToHand and moves the card to hand.` comment at line 594, and document in the test file that the DiscoverToHand path is not tested because the deterministic mode always casts.

#### Finding 2: DiscoverExiled doc comment says cards_exiled includes the discovered card

**Severity**: LOW
**File**: `crates/engine/src/rules/events.rs:663`
**CR Rule**: 701.57a
**Issue**: The doc comment on `DiscoverExiled.cards_exiled` says "in order exiled (including the discovered card if one was found)" and "last = the discovered card, if any qualifying card was found." But the code at `copy.rs:625` calls `exiled_ids.pop()` to remove the discovered card before the `DiscoverExiled` event is emitted at line 699. So the event does NOT include the discovered card when it was cast. This matches Cascade's behavior (`CascadeExiled` also excludes the cast card per line 476: "not including the cast card") but contradicts the doc comment.
**Fix:** Update the `DiscoverExiled` doc comment at `events.rs:662-669` to say "listing all exiled card IDs except the discovered card (which is tracked separately by DiscoverCast or DiscoverToHand)." Align with CascadeExiled's comment at line 476.

#### Finding 3: No test exercises DiscoverToHand

**Severity**: LOW
**File**: `crates/engine/tests/discover.rs`
**CR Rule**: 701.57a -- "If you don't cast it, put that card into your hand."
**Issue**: The plan identified test 5 as "test_discover_put_into_hand_fallback" but acknowledged the difficulty of testing this in deterministic mode. The test file has 7 tests, none of which exercise the `DiscoverToHand` path. This is acceptable for M9.5 but should be addressed when M10 adds player choice.
**Fix:** Add a comment in the test file noting this gap: "DiscoverToHand is not tested because the deterministic engine always casts the discovered card. When M10 adds player choice, add a test that declines to cast and verifies the card goes to hand."

## CR Coverage Check

| CR Subrule | Implemented? | Tested? | Notes |
|------------|-------------|---------|-------|
| 701.57a exile loop | Yes | Yes | test_discover_basic, test_discover_all_lands, test_discover_high_mv |
| 701.57a MV <= N threshold | Yes | Yes | test_discover_mv_equal_to_n, test_discover_vs_cascade_threshold |
| 701.57a cast without paying mana cost | Yes | Yes | test_discover_basic (SpellCast event, stack placement) |
| 701.57a "if you don't cast it, put into hand" | Partial | No | Only reachable on move-to-stack Err; deterministic always casts |
| 701.57a remaining to library bottom | Yes | Yes | test_discover_remaining_cards, test_discover_all_lands |
| 701.57a "random order" (deterministic: sorted) | Yes | Yes | Implicit in test_discover_remaining_cards |
| 701.57b "discovered" even if impossible | Yes | Yes | test_discover_empty_library |
| 701.57c "discovered card" identity | Yes | No | result_id returned but not asserted in tests; adequate for now |
