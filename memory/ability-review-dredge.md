# Ability Review: Dredge

**Date**: 2026-02-26
**Reviewer**: ability-impl-reviewer (Opus)
**CR**: 702.52
**Files reviewed**:
- `crates/engine/src/state/types.rs` (line 202-205)
- `crates/engine/src/state/hash.rs` (lines 318-322, 1646-1666)
- `crates/engine/src/rules/events.rs` (lines 672-695)
- `crates/engine/src/rules/command.rs` (lines 187-201)
- `crates/engine/src/rules/replacement.rs` (lines 403-416, 432-514, 1369-1550)
- `crates/engine/src/rules/engine.rs` (lines 200-218)
- `crates/engine/src/rules/turn_actions.rs` (lines 108-122)
- `crates/engine/src/effects/mod.rs` (lines 1532-1546)
- `tools/replay-viewer/src/view_model.rs` (line 593)
- `crates/engine/tests/dredge.rs` (835 lines, 11 tests)

## Verdict: needs-fix

The core Dredge implementation is correct and well-structured. CR 702.52a and 702.52b are
faithfully implemented. The replacement-effect architecture (DrawAction::DredgeAvailable +
Command::ChooseDredge) correctly models the player choice, the mill-then-return sequence,
and the "not a draw" semantics. Hash coverage is complete. No `.unwrap()` in engine code.

However, two MEDIUM-severity test gaps exist: (1) no test for dredge during effect-based
draws (only draw-step draws are tested, despite the implementation correctly handling both
paths), and (2) no test for a freshly-milled dredge card becoming available for the next
draw in a multi-draw sequence. Both are explicitly called out in card rulings and the
implementation plan but were not included in the test file.

## Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | MEDIUM | `tests/dredge.rs` | **Missing test: dredge during effect-based draw.** Plan test #6 not implemented. |
| 2 | MEDIUM | `tests/dredge.rs` | **Missing test: milled dredge card available for next draw.** Plan test #7 not implemented. |
| 3 | LOW | `rules/events.rs:708-720` | **`Dredged`/`CardMilled` not in `reveals_hidden_info`.** Pre-existing gap. |
| 4 | LOW | `tests/dredge.rs` | **Missing test: wrong-player ChooseDredge.** Plan test #10 not implemented. |
| 5 | LOW | `rules/replacement.rs:1428-1460` | **`has_drawn_for_turn` not set on dredge.** Correct per CR but may confuse future code. |

### Finding Details

#### Finding 1: Missing test for dredge during effect-based draws

**Severity**: MEDIUM
**File**: `crates/engine/tests/dredge.rs`
**CR Rule**: 702.52a -- "if you would draw a card, you may instead mill N cards and return this card from your graveyard to your hand."
**Ruling**: "Dredge can replace any card draw, not only the one during your draw step." (multiple cards, 2024-01-12)
**Issue**: All 11 tests exercise dredge during the draw step turn-based action (via `pass_all` advancing from Upkeep to Draw). No test validates that dredge works during effect-based draws (e.g., DrawCards effect from a spell). The code path through `effects/mod.rs::draw_one_card` has the `DrawAction::DredgeAvailable` arm (line 1542), but it is untested. If this code path broke, no test would catch it.
**Fix**: Add test `test_dredge_during_effect_draw_not_just_draw_step` per plan test #6. Set up a player with a dredge card in graveyard, then execute a `DrawCards(2)` effect (or cast a spell with draw-2). Verify `DredgeChoiceRequired` is emitted for each draw in the sequence.

#### Finding 2: Missing test for freshly-milled dredge card available for next draw

**Severity**: MEDIUM
**File**: `crates/engine/tests/dredge.rs`
**CR Rule**: 614.11a -- "If an effect replaces a draw within a sequence of card draws, all actions required by the replacement are completed, if possible, before resuming the sequence."
**Ruling**: "if you're instructed to draw two cards and you replace the first draw with a dredge ability, another card with a dredge ability (including one that was milled by the first dredge ability) may be used to replace the second draw." (multiple cards, 2024-01-12)
**Issue**: This is one of the most important dredge interactions and is explicitly cited in rulings for 14+ dredge cards. A dredge card milled by the first dredge in a multi-draw sequence should appear in the options for the second draw's `DredgeChoiceRequired`. No test validates this behavior. The implementation should handle this correctly (each draw re-scans the graveyard via `check_would_draw_replacement`), but without a test, regressions would be silent.
**Fix**: Add test `test_dredge_milled_card_available_for_second_draw` per plan test #7. Set up: Player has Dredge-3 card in graveyard; a second Dredge-2 card is positioned near the top of library (within the first 3 cards). Execute a draw-2 effect. First draw: choose to dredge the Dredge-3 card (milling 3, which puts the Dredge-2 card into graveyard). Second draw: verify `DredgeChoiceRequired` includes the newly-milled Dredge-2 card as an option.

#### Finding 3: `Dredged` and `CardMilled` not flagged in `reveals_hidden_info`

**Severity**: LOW
**File**: `crates/engine/src/rules/events.rs:708-720`
**CR Rule**: N/A (architecture invariant #7 -- hidden information enforcement)
**Issue**: The `reveals_hidden_info()` method returns `false` for `Dredged` and `CardMilled` events. Milling moves cards from library (hidden zone) to graveyard (public zone), which reveals their identity. `Dredged` also reveals which card moved from graveyard to hand. However, `CardMilled` was already not flagged before the dredge implementation, so this is a pre-existing gap. The `Dredged` event itself doesn't reveal new hidden info beyond what `CardMilled` already reveals (graveyard is public; hand is private but the player already knows their own hand).
**Fix**: No action required for dredge specifically. If `reveals_hidden_info` is tightened in a future pass (M10 network layer), add `CardMilled` to the true-returning list. `Dredged` can remain false since its information is redundant with the `CardMilled` events.

#### Finding 4: Missing test for wrong-player ChooseDredge command

**Severity**: LOW
**File**: `crates/engine/tests/dredge.rs`
**CR Rule**: N/A (error handling)
**Issue**: Plan test #10 (`test_dredge_invalid_command_wrong_player`) was not implemented. The engine uses `validate_player_exists` (not `validate_player_active`) for `ChooseDredge`, which is correct -- dredge can happen during any player's draw. In the Command design, the `player` field is self-reported, and authentication is the network layer's responsibility. A completely invalid player would fail `validate_player_exists`. A valid but wrong player would fail `handle_choose_dredge`'s graveyard-zone check (the card wouldn't be in the wrong player's graveyard). So the engine is safe, but an explicit test would document the behavior.
**Fix**: Add test `test_dredge_invalid_command_wrong_player`. Have Player B send `ChooseDredge { player: p2, card: Some(p1_dredge_card) }` and assert error (card not in p2's graveyard).

#### Finding 5: `has_drawn_for_turn` not set when dredging during draw step

**Severity**: LOW
**File**: `crates/engine/src/rules/replacement.rs:1428-1460`
**CR Rule**: 702.52a -- dredge replaces the draw, so no draw occurred.
**Issue**: When a player dredges during the draw step, `handle_choose_dredge` (lines 1428-1460) does not set `has_drawn_for_turn = true`. This is technically correct per the CR (dredge is not drawing). However, `has_drawn_for_turn` is currently a write-only field -- it's set by `draw_card` and `draw_card_skipping_dredge` but never read by any game logic. If future code uses this flag to prevent duplicate draw-step draws, a dredge during the draw step would leave the flag as `false`, potentially allowing the draw-step action to fire again.
**Fix**: No immediate action required. The flag is write-only and does not affect game behavior. If the flag is ever used for draw-step gating, revisit whether dredge should set it. Add a comment at line ~1453: `// Note: has_drawn_for_turn intentionally NOT set -- dredge replaces the draw (CR 702.52a).`

## CR Coverage Check

| CR Subrule | Implemented? | Tested? | Notes |
|------------|-------------|---------|-------|
| 702.52a (dredge definition) | Yes | Yes | test_dredge_draw_step_emits_choice_required, test_dredge_mills_and_returns_card_to_hand |
| 702.52a ("may instead") | Yes | Yes | test_dredge_decline_draws_normally, test_dredge_decline_does_not_reoffer |
| 702.52a ("functions only while in graveyard") | Yes | Yes | test_dredge_card_on_battlefield_not_offered |
| 702.52b (library size check) | Yes | Yes | test_dredge_insufficient_library_not_offered, test_dredge_exact_library_count_is_eligible |
| 702.52a "any card draw" (not just draw step) | Yes | **No** | Code path exists in effects/mod.rs but untested (Finding 1) |
| 614.11a (per-draw replacement in sequence) | Yes (implicit) | **No** | Each draw re-scans graveyard, but no multi-draw sequence test (Finding 2) |
| 616.1 (multiple replacements, player chooses) | Partial | No | Dredge offered first; decline falls through to WouldDraw replacements. Correct simplification but no test with both dredge + WouldDraw replacement |
| CR 400.7 (zone-change identity) | Yes | Yes | Dredged event reports card_new_id; test_dredge_mills_and_returns_card_to_hand verifies card in hand by name |
| "Not a draw" semantics | Yes | Yes | test_dredge_does_not_increment_cards_drawn_counter |
| Multiple dredge options | Yes | Yes | test_dredge_multiple_options_both_offered |
| Error handling (invalid card) | Yes | Yes | test_dredge_invalid_command_card_not_in_graveyard |
| Error handling (wrong player) | Yes (implicit) | **No** | validate_player_exists + graveyard check covers it, but no explicit test (Finding 4) |

## Architecture Quality Notes

- **DrawAction::DredgeAvailable pattern**: Clean extension of the existing DrawAction enum. Follows the same early-return-with-event pattern as Skip and NeedsChoice. No architectural concerns.
- **draw_card_skipping_dredge**: Good solution to the "decline dredge but don't re-offer" problem. Duplicates some logic from `turn_actions::draw_card` but this is acceptable given the need to skip the dredge re-check.
- **Hash coverage**: Complete. KeywordAbility::Dredge(n) at discriminant 29, GameEvent::DredgeChoiceRequired at discriminant 72, GameEvent::Dredged at discriminant 73. All fields hashed.
- **Trigger flush**: The engine.rs ChooseDredge arm correctly calls check_triggers + flush_pending_triggers after dredge completes, following the command handler pattern.
- **Loop detection reset**: Correctly resets on ChooseDredge (meaningful player choice per CR 104.4b).
- **view_model.rs**: Exhaustive match updated with `KeywordAbility::Dredge(n) => format!("Dredge {n}")`.
