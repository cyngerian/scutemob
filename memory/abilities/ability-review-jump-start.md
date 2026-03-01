# Ability Review: Jump-Start

**Date**: 2026-03-01
**Reviewer**: ability-impl-reviewer (Opus)
**CR**: 702.133
**Files reviewed**:
- `crates/engine/src/state/types.rs` (lines 771-782)
- `crates/engine/src/state/stack.rs` (lines 135-141)
- `crates/engine/src/state/hash.rs` (lines 515-516, 1588-1589)
- `crates/engine/src/rules/command.rs` (lines 177-190)
- `crates/engine/src/rules/engine.rs` (lines 98-123)
- `crates/engine/src/rules/casting.rs` (lines 70-71, 104, 117-125, 205-253, 305-316, 800-828, 1083-1096, 1148-1149, 1255, 1299)
- `crates/engine/src/rules/resolution.rs` (lines 83-96, 538-548, 1461-1462, 2288-2294)
- `crates/engine/src/effects/mod.rs` (lines 804-811)
- `crates/engine/src/rules/copy.rs` (lines 194-195, 372-373)
- `crates/engine/src/rules/abilities.rs` (lines 345, 583, 757, 1010, 3093, 3271, 3624)
- `crates/engine/src/testing/replay_harness.rs` (lines 245-247, 791-819)
- `crates/engine/src/testing/script_schema.rs` (lines 274-279)
- `tools/replay-viewer/src/view_model.rs` (line 726)
- `crates/engine/tests/jump_start.rs` (all 12 tests)

## Verdict: needs-fix

One MEDIUM finding: the jump-start discard does not handle the Madness interaction
(CR 702.35a). When the discarded card has Madness, it should go to exile and trigger
a MadnessTrigger, but the current implementation always sends it to graveyard. The
plan explicitly marked this as "CRITICAL" in Step 3e and provided the code pattern
from the cycling discard handler (abilities.rs:461-543), but the implementation
did not replicate it. Two LOW findings for missing test coverage of specific
departure paths.

## Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | **MEDIUM** | `casting.rs:1087-1096` | **Missing Madness interaction on jump-start discard.** The discarded card always goes to graveyard; should check for Madness keyword and route to exile + queue MadnessTrigger per CR 702.35a. **Fix:** replicate the madness-check pattern from abilities.rs:461-543. |
| 2 | LOW | `jump_start.rs` | **No fizzle departure test.** Fizzle path is implemented correctly (resolution.rs:90-91) but untested for jump-start specifically. **Fix:** add a test with a targeted jump-start spell where the target becomes illegal before resolution. |
| 3 | LOW | `jump_start.rs` | **No jump-start + buyback interaction test.** Implementation correctly exiles over buyback (resolution.rs:542) but no test verifies this. **Fix:** add a test casting a spell with both buyback and jump-start, verify exile wins. |

### Finding Details

#### Finding 1: Missing Madness Interaction on Jump-Start Discard

**Severity**: MEDIUM
**File**: `crates/engine/src/rules/casting.rs:1087-1096`
**CR Rule**: 702.35a -- "If a player would discard a card with madness, they exile it
instead of putting it into their graveyard."
**Issue**: The jump-start discard code always moves the discarded card to
`ZoneId::Graveyard(discard_owner)`:

```rust
// casting.rs:1087-1096
if let Some(discard_id) = jump_start_card_to_discard {
    let discard_owner = state.object(discard_id)?.owner;
    let (new_discard_id, _) =
        state.move_object_to_zone(discard_id, ZoneId::Graveyard(discard_owner))?;
    events.push(GameEvent::CardDiscarded {
        player,
        object_id: discard_id,
        new_id: new_discard_id,
    });
}
```

This does not check whether the discarded card has `KeywordAbility::Madness`. Per
CR 702.35a, if the discarded card has madness, it should be exiled instead of going
to graveyard, and a `MadnessTrigger` should be queued. The cycling discard handler
in `abilities.rs:461-543` correctly implements this pattern. The plan explicitly
identified this as "CRITICAL" in Step 3e and provided the full code pattern to
follow.

Since jump-start allows discarding ANY card (unlike retrace which only discards
lands), madness interaction is a realistic game scenario. A player casting a
jump-start spell might discard a card with madness to get two spells from one action.

**Fix**: Replace the discard block at `casting.rs:1087-1096` with the madness-aware
pattern from `abilities.rs:461-543`:

1. Before `move_object_to_zone`, check `state.object(discard_id)?.characteristics.keywords.contains(&KeywordAbility::Madness)`.
2. If madness: route to `ZoneId::Exile` instead of `ZoneId::Graveyard(discard_owner)`.
3. After the zone move, if madness: look up the madness cost from the card registry and queue a `PendingTrigger` with `is_madness_trigger: true`, `madness_exiled_card: Some(new_discard_id)`, and `madness_cost`. Follow the exact field pattern from `abilities.rs:503-543`.
4. The `GameEvent::CardDiscarded` event should still be emitted regardless (CR 701.8 -- discard is always announced, even when going to exile via madness).

#### Finding 2: No Fizzle Departure Test

**Severity**: LOW
**File**: `crates/engine/tests/jump_start.rs`
**CR Rule**: 702.133a -- "exile this card instead of putting it anywhere else any time
it would leave the stack" + 608.2b (fizzle rule)
**Issue**: The test suite covers exile-on-resolution (test 2) and exile-on-counter
(test 3), but not exile-on-fizzle. A targeted jump-start spell where all targets
become illegal before resolution should fizzle and go to exile (not graveyard).
The implementation handles this correctly at `resolution.rs:90-91`, but the fizzle
path is not tested for jump-start specifically.
**Fix**: Add a test `test_jump_start_exile_on_fizzle` using a targeted jump-start spell
(e.g., the sorcery that targets a player) where the target player somehow becomes
invalid, or use a simulated scenario where the spell's target leaves the battlefield.
Verify the card ends up in exile.

#### Finding 3: No Jump-Start + Buyback Interaction Test

**Severity**: LOW
**File**: `crates/engine/tests/jump_start.rs`
**CR Rule**: 702.133a -- "exile this card instead of putting it anywhere else any time
it would leave the stack" overrides 702.27a (buyback return to hand)
**Issue**: The code correctly prioritizes jump-start exile over buyback return-to-hand
at `resolution.rs:542-548` (jump-start check comes before buyback check in the
if-else chain). However, no test verifies this interaction. A card with both
jump-start and buyback cast via jump-start with buyback paid should still be exiled.
**Fix**: Add a test `test_jump_start_overrides_buyback` with a card that has both
JumpStart keyword and a buyback cost, cast via jump-start with buyback paid. Verify
the card goes to exile on resolution (not hand).

## CR Coverage Check

| CR Subrule | Implemented? | Tested? | Notes |
|------------|-------------|---------|-------|
| 702.133a (graveyard casting permission) | Yes | Yes | test_jump_start_basic_cast_from_graveyard |
| 702.133a (discard additional cost) | Yes | Yes | test_jump_start_basic_cast_from_graveyard, test_jump_start_discard_required |
| 702.133a (normal mana cost, not alt) | Yes | Yes | test_jump_start_pays_normal_mana_cost |
| 702.133a (instants/sorceries only) | Yes | Partial | Type check exists; no explicit non-instant/sorcery negative test, but the keyword itself only appears on I/S |
| 702.133a (exile on resolve) | Yes | Yes | test_jump_start_exile_on_resolution |
| 702.133a (exile on counter) | Yes | Yes | test_jump_start_exile_on_counter |
| 702.133a (exile on fizzle) | Yes | No | Implementation correct (resolution.rs:90-91), no test (Finding 2) |
| 702.133a (exile overrides buyback) | Yes | No | Implementation correct (resolution.rs:542), no test (Finding 3) |
| 702.133a (timing restrictions) | Yes | Yes | test_jump_start_sorcery_timing |
| 702.133a (any card for discard) | Yes | Yes | test_jump_start_discard_any_card |
| 702.133a (discard must be in hand) | Yes | Yes | test_jump_start_discard_must_be_in_hand |
| 702.133a (cast_with_jump_start flag) | Yes | Yes | test_jump_start_flag_set_on_stack |
| 702.133a (normal cast not exiled) | Yes | Yes | test_jump_start_normal_hand_cast_not_exiled |
| 702.133a (insufficient mana rejected) | Yes | Yes | test_jump_start_insufficient_mana_rejected |
| 702.133a (card without keyword rejected) | Yes | Yes | test_jump_start_non_jump_start_card_cannot_cast |
| 702.133a (flashback suppression) | Yes | No | Handled in casting.rs:125 but no dedicated test |
| 702.35a (madness on discard) | **No** | No | **Finding 1 -- MEDIUM** |
| 601.2b/601.2f-h (additional cost rules) | Yes | Yes | Discard happens during cost payment, before stack |
| CR copies (707.10) | Yes | N/A | copy.rs sets cast_with_jump_start: false |
| Hash coverage | Yes | N/A | KeywordAbility 90u8 + StackObject field |
| All StackObject sites | Yes | N/A | 12 engine sites + 10 harness sites all set false |
