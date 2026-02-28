# Ability Review: Escape

**Date**: 2026-02-27
**Reviewer**: ability-impl-reviewer (Opus)
**CR**: 702.138
**Files reviewed**:
- `crates/engine/src/state/types.rs` (lines 385-397)
- `crates/engine/src/cards/card_definition.rs` (lines 202-219)
- `crates/engine/src/state/hash.rs` (lines 390-391, 543, 1207, 2495-2509)
- `crates/engine/src/state/stack.rs` (lines 93-103)
- `crates/engine/src/state/game_object.rs` (lines 334-343)
- `crates/engine/src/rules/command.rs` (lines 122-140)
- `crates/engine/src/rules/casting.rs` (full file, focus on lines 97-413, 449-458, 670-689, 747-768, 1068-1151)
- `crates/engine/src/rules/engine.rs` (lines 71-112)
- `crates/engine/src/rules/resolution.rs` (lines 218-254, 385-391, 776-786)
- `crates/engine/src/rules/copy.rs` (lines 182-186, 355-358)
- `crates/engine/src/testing/replay_harness.rs` (escape-related lines)
- `crates/engine/src/testing/script_schema.rs` (lines 238-242)
- `crates/engine/tests/escape.rs` (full file, 14 tests)

## Verdict: needs-fix

The implementation is comprehensive and handles most CR 702.138 requirements correctly. The enum variants, hash coverage, StackObject/GameObject fields, cost payment, resolution propagation, and EscapeWithCounter are all well-implemented. However, there is one HIGH finding: the engine incorrectly rejects escape when the card also has flashback (contradicting the Glimpse of Freedom / Ox of Agonas ruling 2020-01-24 that says "you choose which one to apply"). There are also two MEDIUM findings: a missing test for sorcery-with-escape resolving to graveyard (not exile), and a missing test for opponent's-graveyard exile rejection. Several LOW findings exist around missing edge case tests and a minor CR 702.138d gap.

## Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | **HIGH** | `casting.rs:340` | **Escape+Flashback mutual exclusion prevents legal game action.** The engine auto-detects flashback and then rejects escape, making it impossible to use escape on a card that also has flashback. **Fix:** suppress flashback auto-detection when `cast_with_escape: true`. |
| 2 | MEDIUM | `tests/escape.rs` | **Missing test: sorcery with escape resolves to graveyard (not exile).** Plan test #4 was not implemented. **Fix:** add `test_escape_sorcery_resolves_to_graveyard`. |
| 3 | MEDIUM | `tests/escape.rs` | **Missing test: exile card from opponent's graveyard rejected.** Only tests hand-card rejection, not opponent-graveyard rejection. **Fix:** add `test_escape_exile_from_opponent_graveyard_rejected`. |
| 4 | LOW | `tests/escape.rs` | **Missing test: sorcery-speed timing enforcement from graveyard.** Plan test #11 was not implemented. |
| 5 | LOW | `tests/escape.rs` | **Missing test: cannot exile the escape card itself.** The `apply_escape_exile_cost` function checks for self-exile at line 1125, but no test covers this path. |
| 6 | LOW | `tests/escape.rs` | **Missing test: escape + evoke mutual exclusion.** Mutual exclusion with flashback is tested but not with evoke. |
| 7 | LOW | `tests/escape.rs:1091` | **Test 10 asserts wrong behavior.** The test expects escape+flashback to be rejected, but per the ruling the player should be allowed to choose escape. This test needs to be rewritten after Finding 1 is fixed. |
| 8 | LOW | `card_definition.rs` | **702.138d ("escapes with [ability]") not implemented.** Only EscapeWithCounter (702.138c) exists. Acceptable for now since no cards in the registry need it. |

### Finding Details

#### Finding 1: Escape+Flashback mutual exclusion prevents legal game action

**Severity**: HIGH
**File**: `crates/engine/src/rules/casting.rs:340`
**CR Rule**: 702.138a / 118.9a
**Ruling**: Glimpse of Freedom (2020-01-24): "If a card has multiple abilities giving you permission to cast it, such as two escape abilities or an escape ability and a flashback ability, you choose which one to apply. The others have no effect."
**Issue**: When a card in the graveyard has both the Flashback and Escape keywords, `casting_with_flashback` is auto-detected as `true` at line 105-109. Then when `cast_with_escape: true`, the check at line 340 fires:
```rust
if casting_with_flashback {
    return Err(GameStateError::InvalidCommand(
        "cannot combine escape with flashback (CR 118.9a: only one alternative cost)".into(),
    ));
}
```
This rejects the command even though the player is NOT combining two alternative costs -- they are choosing escape INSTEAD of flashback. The result is that escape can never be used on any card that also has flashback.
**Fix**: When `cast_with_escape: true`, suppress the flashback auto-detection. Change line 105-109 to account for the explicit escape flag:
```rust
let casting_with_flashback = casting_from_graveyard
    && card_obj.characteristics.keywords.contains(&KeywordAbility::Flashback)
    && !cast_with_escape; // Player explicitly chose escape over flashback
```
Then remove the `casting_with_flashback` conflict check at line 340 (which is now unreachable when `cast_with_escape: true`). Also update test 10 (`test_escape_cannot_combine_with_flashback`) to expect SUCCESS when `cast_with_escape: true` on a card with both keywords, and add a separate test that both flashback and escape cannot be used simultaneously via some other mechanism (which the current engine doesn't support anyway since flashback has no explicit flag).

#### Finding 2: Missing test for sorcery with escape resolving to graveyard

**Severity**: MEDIUM
**File**: `crates/engine/tests/escape.rs`
**CR Rule**: 702.138a -- Escape does not change the spell's resolution destination. Unlike flashback (CR 702.34a), escape does not exile the card on resolution. An instant or sorcery cast with escape goes to the graveyard after resolution.
**Issue**: The plan specified test #4 (`test_escape_sorcery_goes_to_graveyard`) but it was not implemented. Test 3 only verifies creatures enter the battlefield. The critical difference from flashback (instant/sorcery going to graveyard vs. exile) is not tested for escape.
**Fix**: Add a test that casts a sorcery via escape, resolves it, and asserts the card is in the graveyard (not exile). This requires a sorcery card definition with only Escape (not Flashback) to avoid the Finding 1 issue.

#### Finding 3: Missing test for opponent's graveyard exile rejection

**Severity**: MEDIUM
**File**: `crates/engine/tests/escape.rs`
**CR Rule**: 702.138a -- "Exile [N] other cards from your graveyard" -- cards must be from the caster's own graveyard.
**Issue**: Test 9 (`test_escape_exile_card_not_in_graveyard_rejected`) tests a card from the player's hand, not from an opponent's graveyard. The enforcement logic at `apply_escape_exile_cost` line 1134 checks `obj.zone != ZoneId::Graveyard(player)` which correctly rejects opponent's graveyard cards, but this path has no test coverage.
**Fix**: Add a test that places a card in `ZoneId::Graveyard(p2)` (opponent's graveyard) and attempts to use it in `escape_exile_cards`. Verify the cast is rejected with an appropriate error.

#### Finding 4: Missing test for sorcery-speed timing enforcement

**Severity**: LOW
**File**: `crates/engine/tests/escape.rs`
**CR Rule**: 702.138a (ruling 2020-01-24) -- Escape doesn't change timing restrictions.
**Issue**: The plan specified test #11 (`test_escape_timing_restriction_applies`) but it was not implemented. The enforcement at casting.rs line 525 correctly does NOT exempt escape from sorcery-speed timing (unlike madness and miracle which are exempted), but there is no test exercising this path.
**Fix**: Add a test that attempts to cast a creature with escape during an opponent's turn or during a non-main phase, and verify the cast is rejected.

#### Finding 5: Missing test for self-exile prevention

**Severity**: LOW
**File**: `crates/engine/tests/escape.rs`
**CR Rule**: 702.138a -- "other cards from your graveyard" -- the card being cast cannot be exiled as part of its own escape cost.
**Issue**: `apply_escape_exile_cost` at line 1125 checks `if id == escape_card_id` and rejects self-exile. Since costs are paid before the card moves to the stack in this engine, the escape card is still in the graveyard at that point, making self-inclusion in the exile list a realistic attack vector. No test covers this defensive check.
**Fix**: Add a test that includes the escape card's own ObjectId in `escape_exile_cards` and verify the cast is rejected.

#### Finding 6: Missing test for escape+evoke mutual exclusion

**Severity**: LOW
**File**: `crates/engine/tests/escape.rs`
**CR Rule**: 118.9a -- Only one alternative cost per spell.
**Issue**: The flashback mutual exclusion is tested (test 10), but there is no test for escape+evoke, escape+bestow, or escape+miracle mutual exclusion. The enforcement logic at lines 346-365 covers all these, but only the flashback path is tested.
**Fix**: Add at least one test for `cast_with_escape: true` + `cast_with_evoke: true` and verify the error.

#### Finding 7: Test 10 asserts wrong behavior

**Severity**: LOW
**File**: `crates/engine/tests/escape.rs:1091`
**CR Rule**: Ruling 2020-01-24 for Glimpse of Freedom / Ox of Agonas.
**Issue**: `test_escape_cannot_combine_with_flashback` expects `result.is_err()` when casting with `cast_with_escape: true` on a card that has both Escape and Flashback. Per the ruling, this should succeed (player chose escape over flashback). The test is asserting the current buggy behavior.
**Fix**: After fixing Finding 1, rewrite this test to expect success. Add a separate positive test that casts via escape on a dual-keyword card and verifies was_escaped=true, and that flashback exile-on-resolution does NOT apply.

#### Finding 8: 702.138d not implemented

**Severity**: LOW
**File**: `crates/engine/src/cards/card_definition.rs`
**CR Rule**: 702.138d -- "escapes with [ability]" means "If this permanent escaped, it has [ability]."
**Issue**: Only `EscapeWithCounter` (702.138c) is implemented as an `AbilityDefinition` variant. The `EscapeWithAbility` variant (702.138d) is not implemented. This is acceptable for the initial implementation since no current card definitions require it, but it should be tracked.
**Fix**: No immediate fix required. Track as a future enhancement. When a card definition needs "escapes with [ability]", add `AbilityDefinition::EscapeWithAbility { ability: KeywordAbility }` and handle it in resolution.rs similar to EscapeWithCounter.

## CR Coverage Check

| CR Subrule | Implemented? | Tested? | Notes |
|------------|-------------|---------|-------|
| 702.138a (cast from graveyard) | Yes | Yes | test_escape_basic_cast_from_graveyard |
| 702.138a (exile N other cards) | Yes | Yes | test_escape_exile_cost_events, test_escape_insufficient_exile_cards_rejected |
| 702.138a (alternative cost / CR 118.9) | Yes | Partial | Flashback exclusion tested but buggy (Finding 1); evoke/bestow/miracle not tested |
| 702.138a ("other cards" = not self) | Yes | No | Self-exile check exists in code but no test (Finding 5) |
| 702.138a ("your graveyard" = caster's) | Yes | Partial | Hand-card rejection tested; opponent's graveyard not tested (Finding 3) |
| 702.138a (timing unchanged) | Yes | No | Code correct; no test (Finding 4) |
| 702.138a (all card types) | Yes | Yes | Creature test (Uro, Ox); sorcery defined (Loot) but resolution not tested (Finding 2) |
| 702.138b (was_escaped flag) | Yes | Yes | test_escape_was_escaped_flag_on_permanent |
| 702.138c (escapes with counter) | Yes | Yes | test_escape_with_counter, test_escape_with_counter_not_applied_when_not_escaped |
| 702.138d (escapes with ability) | No | No | Not implemented; tracked as LOW (Finding 8) |
| CR 118.9a (one alt cost) | Yes | Partial | Flashback tested (buggy); others not tested (Finding 6) |
| CR 118.9c (mana value unchanged) | Yes | Yes | test_escape_mana_value_unchanged |
| CR 118.9d (additional costs apply) | Yes | No | Commander tax applies correctly in code path; no dedicated test |
| CR 400.7 (zone change identity) | Yes | Yes | test_escape_exile_cards_get_new_ids_in_exile |
| CR 601.2h (cost payment) | Yes | Yes | Exile during cost payment; events emitted |

## Previous Findings (re-review only)

N/A -- this is the first review.
