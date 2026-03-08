# Ability Review: Blood Tokens

**Date**: 2026-03-08
**Reviewer**: ability-impl-reviewer (Opus)
**CR**: 111.10g
**Files reviewed**:
- `crates/engine/src/cards/card_definition.rs:1559-1599` (blood_token_spec)
- `crates/engine/src/cards/mod.rs:17` (re-export)
- `crates/engine/src/lib.rs:9` (public export)
- `crates/engine/src/state/game_object.rs:90-113` (ActivationCost.discard_card)
- `crates/engine/src/state/hash.rs:1404-1411` (ActivationCost hash)
- `crates/engine/src/rules/command.rs:331-341` (Command::ActivateAbility.discard_card)
- `crates/engine/src/rules/engine.rs:158-182` (destructure + pass-through)
- `crates/engine/src/rules/abilities.rs:52-58,326-354` (handle_activate_ability signature + discard processing)
- `crates/engine/src/testing/replay_harness.rs:640-653,2829-2834` (harness activate_ability + cost_to_activation_cost)
- `crates/engine/tests/blood_tokens.rs` (14 tests, 909 lines)

## Verdict: clean

The implementation is correct and thorough. The `blood_token_spec` function matches CR 111.10g exactly (colorless, artifact, Blood subtype, {1}/{T}/discard/sacrifice: draw). The `discard_card` cost infrastructure is properly wired through all layers: `ActivationCost` field, hash coverage, `Command::ActivateAbility` field, `handle_activate_ability` enforcement (zone validation, move to graveyard, CardDiscarded event), harness wiring, and all construction sites updated. Tests cover the positive path, all four cost components (mana, tap, discard, sacrifice), negative cases (no mana, tapped, no hand card, wrong zone, wrong controller), stack usage, SBA token cleanup, and summoning sickness non-applicability. No HIGH or MEDIUM findings.

## Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | LOW | `tests/blood_tokens.rs` | **Missing test: multiple Blood tokens independently activatable.** No test verifies two Blood tokens can each be activated separately. **Fix:** Add test creating 2 Blood tokens, activating one, verifying the other remains on battlefield and is independently activatable. |
| 2 | LOW | `tests/blood_tokens.rs` | **Missing test: discard from opponent's hand rejected.** The engine correctly validates `ZoneId::Hand(player)` at abilities.rs:340, but no test supplies a card from another player's hand as the discard target. **Fix:** Add test where p1 tries to discard a card in p2's hand as Blood activation cost, assert failure. |
| 3 | LOW | `rules/abilities.rs:330` | **No validation that discard_card is None when ability doesn't require discard.** If a caller passes `discard_card: Some(id)` for an ability with `discard_card: false`, the extra parameter is silently ignored. Not a correctness bug (no card is discarded), but could mask caller errors. **Fix:** Consider adding a debug_assert or warning, or accept as intentional leniency. Low priority. |

### Finding Details

#### Finding 1: Missing test -- multiple Blood tokens independently activatable

**Severity**: LOW
**File**: `crates/engine/tests/blood_tokens.rs`
**CR Rule**: 111.10g -- "A Blood token is a colorless Blood artifact token..."
**Issue**: The plan called for verifying that "Multiple Blood tokens can each be activated independently." While `test_blood_token_create_via_effect` checks `blood_token_spec(3).count == 3`, no test creates two Blood tokens on the battlefield and activates them separately.
**Fix**: Add a test that places 2 Blood tokens on the battlefield, activates one (verify it leaves, hand discards, draw occurs), then verifies the second is still on the battlefield and can be activated independently.

#### Finding 2: Missing test -- discard from opponent's hand rejected

**Severity**: LOW
**File**: `crates/engine/tests/blood_tokens.rs`
**CR Rule**: 602.2 -- activated ability costs are paid by the controller
**Issue**: Test 14 validates that a card in the graveyard cannot be used as discard cost. However, no test validates that a card in another player's hand is also rejected. The enforcement at abilities.rs:340 checks `ZoneId::Hand(player)` which would correctly reject `ZoneId::Hand(opponent)`, but this path is untested.
**Fix**: Add a test where player 1 controls a Blood token and tries to activate it with `discard_card: Some(card_in_p2_hand)`. Assert it returns an error.

#### Finding 3: Silent acceptance of unnecessary discard_card parameter

**Severity**: LOW
**File**: `crates/engine/src/rules/abilities.rs:330`
**CR Rule**: 602.2 -- costs must match the ability's cost specification
**Issue**: The `if ability_cost.discard_card { ... }` block only runs when the ability requires a discard. If `discard_card: Some(id)` is passed for a Food token activation (which has `discard_card: false`), the parameter is silently ignored. This is not a correctness bug but could mask command construction errors.
**Fix**: Optionally add `debug_assert!(discard_card.is_none(), ...)` when `!ability_cost.discard_card` to catch caller mistakes during testing. Low priority -- accept as-is if preferred.

## CR Coverage Check

| CR Subrule | Implemented? | Tested? | Notes |
|------------|-------------|---------|-------|
| 111.10g (token characteristics) | Yes | Yes | test_blood_token_spec_characteristics, test_blood_token_has_activated_ability |
| 111.10g ({1} mana cost) | Yes | Yes | test_blood_token_activation_basic, test_blood_token_activation_no_mana |
| 111.10g ({T} tap cost) | Yes | Yes | test_blood_token_activation_already_tapped |
| 111.10g (discard a card cost) | Yes | Yes | test_blood_token_discard_is_cost, test_blood_token_activation_no_cards_in_hand, test_blood_token_discard_must_be_from_hand |
| 111.10g (sacrifice self cost) | Yes | Yes | test_blood_token_activation_sacrifice_removes_from_battlefield |
| 111.10g (draw a card effect) | Yes | Yes | test_blood_token_activation_basic |
| 602.2 (costs paid at activation) | Yes | Yes | test_blood_token_discard_is_cost (discard before resolve) |
| 602.2 (controller activates) | Yes | Yes | test_blood_token_only_controller_can_activate |
| 602.2 (uses the stack) | Yes | Yes | test_blood_token_uses_stack |
| 302.6 (summoning sickness - creatures only) | Yes | Yes | test_blood_token_not_affected_by_summoning_sickness |
| 704.5d (token ceases in non-battlefield) | Yes | Yes | test_blood_token_sba_ceases_to_exist |
