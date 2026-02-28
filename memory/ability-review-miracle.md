# Ability Review: Miracle

**Date**: 2026-02-27
**Reviewer**: ability-impl-reviewer (Opus)
**CR**: 702.94
**Files reviewed**:
- `crates/engine/src/state/types.rs` (lines 373-384)
- `crates/engine/src/cards/card_definition.rs` (lines 193-201)
- `crates/engine/src/state/hash.rs` (lines 388-389, 961-964, 1142-1154, 1199-1201, 1836-1846, 2484-2488)
- `crates/engine/src/state/stack.rs` (lines 85-92, 185-200)
- `crates/engine/src/state/stubs.rs` (lines 109-127)
- `crates/engine/src/rules/events.rs` (lines 704-715)
- `crates/engine/src/rules/command.rs` (lines 112-121, 261-275)
- `crates/engine/src/rules/miracle.rs` (entire file, 183 lines)
- `crates/engine/src/rules/casting.rs` (lines 63, 86, 115-142, 291-314, 341-351, 415, 630)
- `crates/engine/src/rules/resolution.rs` (lines 648-673, 770)
- `crates/engine/src/rules/engine.rs` (lines 20, 81, 97, 240-258)
- `crates/engine/src/rules/abilities.rs` (lines 339, 511-513, 643-645, 916-918, 960-962, 1079-1081, 1198-1205, 1224)
- `crates/engine/src/rules/turn_actions.rs` (lines 160-164)
- `crates/engine/src/effects/mod.rs` (lines 1900-1905)
- `crates/engine/src/rules/replacement.rs` (lines 1566-1572)
- `crates/engine/src/rules/mod.rs` (line 17)
- `crates/engine/src/testing/replay_harness.rs` (lines 335-374)
- `tools/replay-viewer/src/view_model.rs` (lines 437-438, 622)
- `crates/engine/tests/miracle.rs` (entire file, 856 lines)

## Verdict: needs-fix

The implementation is structurally sound and closely follows the Madness pattern. All hash
fields are covered, all match arms are exhaustive, all draw sites are wired, and the
alternative cost / timing override / mutual exclusion logic in casting.rs is correct. However,
there is one MEDIUM finding related to the `cards_drawn_this_turn` counter semantics in
multiplayer that causes miracle to silently fail for non-active players across consecutive
opponent turns. There is also one MEDIUM finding for a missing test covering the
"card leaves hand before trigger resolves" edge case (test 9 in the plan was listed but the
implementation omits the actual validation). All other issues are LOW.

## Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | MEDIUM | `rules/turn_actions.rs:353` | **cards_drawn_this_turn not reset for non-active players.** Counter only resets for active player; in multiplayer, non-active players who drew on a previous opponent's turn have stale counters, causing miracle to silently fail. **Fix:** Reset all players' `cards_drawn_this_turn` to 0 at start of each game turn (not just active player). |
| 2 | MEDIUM | `tests/miracle.rs:656-768` | **test_miracle_cannot_combine_with_flashback does not actually test mutual exclusion.** The test casts with `cast_with_miracle: true` alone and asserts success, but never sends both `cast_with_miracle: true` AND `cast_with_flashback` (from graveyard) to trigger the mutual exclusion error. The test comment acknowledges this ("the actual exclusion check is in casting.rs"). **Fix:** Add a second assertion that attempts `CastSpell` with both flags and verifies the error. |
| 3 | LOW | `rules/miracle.rs:104-106` | **Dead code in source computation.** `card_id_opt.map(\|_\| card).unwrap_or(card)` always evaluates to `card`. **Fix:** Replace with `let source = card;`. |
| 4 | LOW | `tests/miracle.rs` | **No test for card leaving hand before MiracleTrigger resolves.** The plan (Step 7, test 9) specified `test_miracle_card_leaves_hand_before_resolution` but it was not implemented. The cast validation at casting.rs:118 handles this (returns error if card not in hand), but there is no explicit test. **Fix:** Add a test that moves the miracle card out of hand (e.g., discard) after revealing but before casting, then verify CastSpell with miracle fails. |
| 5 | LOW | `tests/miracle.rs:446-448` | **Direct mana pool mutation in test.** Uses `state.players.get_mut(&p1)` to add mana instead of using a mana-producing command/helper. This is a pattern used in other test files but bypasses the normal engine flow. Consistent with existing test conventions, so LOW. |
| 6 | LOW | `tests/miracle.rs:467` | **Fragile priority override.** `state.turn.priority_holder = Some(p1)` bypasses the priority system. The ChooseMiracle handler resets priority to the active player, but when testing miracle casting on another player's turn, directly setting priority_holder is fragile. Consistent with existing test conventions, so LOW. |

### Finding Details

#### Finding 1: cards_drawn_this_turn not reset for non-active players at turn boundaries

**Severity**: MEDIUM
**File**: `crates/engine/src/rules/turn_actions.rs:353`
**CR Rule**: 702.94a -- "the first card you've drawn this turn"
**Issue**: `reset_turn_state()` is called only for the active player at the start of their turn (engine.rs:350, 510, 576, 712). The field `cards_drawn_this_turn` is therefore only reset for one player per turn. In a 4-player Commander game, consider: Player B draws on Player C's turn (counter becomes 1). Player D's turn starts -- Player B's counter is NOT reset. If Player B draws on Player D's turn, their counter is now 2 and `check_miracle_eligible` returns `None` because `cards_drawn != 1`. This means miracle fails to trigger even though it's the first card Player B drew during Player D's turn.

The CR says "the first card you've drawn this turn" where "this turn" refers to the current game turn (the active player's turn), not the drawing player's own turn. Each game turn is a distinct time period during which each player's draw count should be independently tracked.

Note: This is a pre-existing issue with how `cards_drawn_this_turn` is managed. It was designed for Sylvan Library and Storm which only care about the active player's own draws/casts. The miracle implementation correctly uses the counter but the counter's reset semantics are wrong for non-active players.

**Fix**: In `reset_turn_state`, reset `cards_drawn_this_turn` for ALL players, not just the active player. Alternatively, add a separate loop in the turn-advance code to reset all players' draw counters. Be careful not to also reset `spells_cast_this_turn` for all players (storm cares about casts during the storm player's own turn). The safest fix is to add a new loop at engine.rs:350 (and the other advance_turn sites) that resets all players' `cards_drawn_this_turn`:

```rust
// Reset draw counter for all players at turn boundary (CR 702.94a: "this turn")
for player_state in state.players.values_mut() {
    player_state.cards_drawn_this_turn = 0;
}
```

Then verify Sylvan Library (CC#33) is not broken by this change -- Sylvan Library says "each turn" which means the Library owner's own turn, and the Library triggers at the beginning of the owner's draw step. Since the counter is now reset for ALL players at each turn boundary, it correctly tracks "draws during this game turn" which is what both Sylvan Library and miracle need.

#### Finding 2: test_miracle_cannot_combine_with_flashback does not exercise mutual exclusion

**Severity**: MEDIUM
**File**: `crates/engine/tests/miracle.rs:656-768`
**CR Rule**: 118.9a -- "Only one alternative cost can be applied to any one spell as it's being cast."
**Issue**: The test creates a card with both Miracle and Flashback but only casts it from hand with `cast_with_miracle: true`. Since the card is in hand (not graveyard), flashback never activates, so the mutual exclusion in casting.rs:304-309 is never tested. The test's assertion (`result.is_ok()`) verifies casting works, not that the exclusion is enforced. The comment at line 740-746 acknowledges this gap.

The defense-in-depth mutual exclusion code at casting.rs:291-314 is correct and would reject a command with both flags set, but it has ZERO test coverage.

**Fix**: Add a focused test that places the miracle card in the graveyard with both Miracle and Flashback keywords, attempts `CastSpell` with both `cast_with_miracle: true` and the card in graveyard (which triggers `casting_with_flashback`), and asserts the error message contains "cannot combine miracle with flashback." Alternatively, the simplest approach: directly test the error path by manually constructing a scenario where the miracle trigger is on stack and the card is in graveyard.

#### Finding 3: Dead code in source computation

**Severity**: LOW
**File**: `crates/engine/src/rules/miracle.rs:104-106`
**CR Rule**: N/A (code quality)
**Issue**: The expression `card_id_opt.map(|_| card).unwrap_or(card)` always returns `card` regardless of whether `card_id_opt` is `Some(_)` or `None`. The `.map(|_| card)` discards the inner value and returns `card`, and `.unwrap_or(card)` returns `card` for `None`. This is functionally `let source = card;`.
**Fix**: Replace lines 104-106 with `let source = card;`.

#### Finding 4: Missing test for card leaving hand before resolution

**Severity**: LOW
**File**: `crates/engine/tests/miracle.rs`
**CR Rule**: 702.94a + 400.7 (zone change identity)
**Issue**: The plan specified `test_miracle_card_leaves_hand_before_resolution` as test 9. This is an important edge case: if the miracle card is discarded (or bounced to library) after reveal but before the player casts it, `CastSpell` with `cast_with_miracle: true` should fail because the card is no longer in hand. The casting.rs validation at line 118 handles this correctly, but there is no test exercising this path.
**Fix**: Add a test that: (1) draws a miracle card (first draw), (2) reveals it (ChooseMiracle reveal: true), (3) moves the card out of hand (e.g., via a discard effect), (4) attempts CastSpell with cast_with_miracle: true and asserts the error.

## CR Coverage Check

| CR Subrule | Implemented? | Tested? | Notes |
|------------|-------------|---------|-------|
| 702.94a (static + triggered) | Yes | Yes | test_miracle_first_draw_emits_choice_event, test_miracle_reveal_puts_trigger_on_stack |
| 702.94a (first card drawn this turn) | Yes | Yes | test_miracle_second_draw_no_choice_event; but see Finding 1 for multiplayer reset gap |
| 702.94a (reveal choice) | Yes | Yes | test_miracle_decline_reveal_no_trigger |
| 702.94a (cast for miracle cost) | Yes | Yes | test_miracle_cast_for_miracle_cost |
| 702.94a (timing override - sorcery at instant speed) | Yes | Yes | test_miracle_sorcery_ignores_timing |
| 702.94a (trigger resolves, card stays in hand) | Yes | Yes | test_miracle_trigger_resolves_without_cast |
| 702.94b (card remains revealed until trigger leaves stack) | No | No | "Revealed" status tracking not implemented; LOW priority |
| 118.9a (only one alternative cost) | Yes | Partial | Mutual exclusion code exists in casting.rs:291-314 but test does not exercise the error path (Finding 2) |
| 118.9c (mana value unchanged) | Yes | Yes | test_miracle_mana_value_unchanged |
| 118.9d (additional costs apply on top) | Yes | N/A | Commander tax logic in casting.rs:356-367 applies; no specific miracle test needed |
| Any turn, not just own turn | Yes | Yes | test_miracle_opponent_turn_first_draw |
| Card leaves hand before resolution | Yes (validation) | No | Casting.rs:118 validates; no test (Finding 4) |
| Non-miracle card negative test | Yes | Yes | test_miracle_non_miracle_card_no_choice |
| Draw sites: turn_actions | Yes | Yes | Tested via test_miracle_first_draw_emits_choice_event |
| Draw sites: effects/mod.rs | Yes | No | Wired but no dedicated test for effect-based draw |
| Draw sites: replacement.rs (dredge decline) | Yes | No | Wired but no dedicated test for post-dredge-decline miracle |
