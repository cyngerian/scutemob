# Ability Review: Encore

**Date**: 2026-03-01
**Reviewer**: ability-impl-reviewer (Opus)
**CR**: 702.141
**Files reviewed**:
- `crates/engine/src/state/types.rs` (KeywordAbility::Encore)
- `crates/engine/src/cards/card_definition.rs` (AbilityDefinition::Encore)
- `crates/engine/src/state/game_object.rs` (encore_sacrifice_at_end_step, encore_must_attack)
- `crates/engine/src/state/stack.rs` (EncoreAbility, EncoreSacrificeTrigger)
- `crates/engine/src/state/stubs.rs` (is_encore_sacrifice_trigger, encore_activator)
- `crates/engine/src/state/hash.rs` (all 5 hash entries)
- `crates/engine/src/state/builder.rs` (encore fields init)
- `crates/engine/src/state/mod.rs` (zone change resets)
- `crates/engine/src/effects/mod.rs` (encore fields init)
- `crates/engine/src/rules/command.rs` (Command::EncoreCard)
- `crates/engine/src/rules/engine.rs` (EncoreCard handler)
- `crates/engine/src/rules/abilities.rs` (handle_encore_card, get_encore_cost, flush_pending_triggers arm)
- `crates/engine/src/rules/resolution.rs` (EncoreAbility + EncoreSacrificeTrigger resolution)
- `crates/engine/src/rules/turn_actions.rs` (end_step_actions encore trigger queueing)
- `crates/engine/src/testing/replay_harness.rs` (encore_card action)
- `tools/tui/src/play/panels/stack_view.rs` (EncoreAbility + EncoreSacrificeTrigger arms)
- `tools/replay-viewer/src/view_model.rs` (EncoreAbility + EncoreSacrificeTrigger arms)
- `crates/engine/tests/encore.rs` (10 tests)

## Verdict: needs-fix

One MEDIUM finding: the encore sacrifice trigger stores the token's current controller
at end-step time as `encore_activator`, instead of the original activator who activated
the encore ability. If a control-change effect takes the token before end step, the
sacrifice check (`current_controller == activator`) is bypassed, contradicting the
2020-11-10 ruling. All other aspects of the implementation are correct and thorough.

## Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | **MEDIUM** | `turn_actions.rs:255` | **Encore activator identity lost on control change.** The sacrifice trigger uses current controller instead of original activator. **Fix:** Store the activator PlayerId on the token (new field) or propagate it from the EncoreAbility resolution. |
| 2 | LOW | `turn_actions.rs:259` | **No `encore_must_attack` cleanup at end of turn.** If sacrifice is countered (Stifle), the token persists with stale mandatory attack data. **Fix:** Clear `encore_must_attack` in `cleanup_actions()`. |
| 3 | LOW | `resolution.rs:2684` | **Token abilities/triggered_abilities/activated_abilities empty.** Pre-existing systemic gap (documented in embalm review #2). Token uses `card_id` registry lookup as workaround. Not Encore-specific. |
| 4 | LOW | `resolution.rs:2653-2670` | **Token color derived from mana cost only.** Does not handle `color_indicator` (CR 202.2e). Pre-existing pattern across all token creators. Not Encore-specific. |
| 5 | LOW | `tests/encore.rs` | **No assertion that `encore_must_attack` is set correctly on tokens.** Module doc claims this is tested but no actual assertion exists. |
| 6 | LOW | `tests/encore.rs` | **No test for sacrifice trigger countered by Stifle.** Plan lists this as interaction #4. |

### Finding Details

#### Finding 1: Encore activator identity lost on control change

**Severity**: MEDIUM
**File**: `crates/engine/src/rules/turn_actions.rs:255`
**CR Rule**: 702.141a -- "Sacrifice them at the beginning of the next end step."
**Ruling**: 2020-11-10 -- "If one of the tokens is under another player's control as the delayed triggered ability resolves, you can't sacrifice that token."
**Issue**: In `end_step_actions()`, the sacrifice trigger is created with `encore_activator: Some(controller)`, where `controller` is the token's current controller at the time the end step begins. The comment says "at creation time, controller == activator", but this code runs at end step, not at token creation. If a control-change effect (e.g., Threaten, Act of Treason) has changed the token's controller between its creation (when `EncoreAbility` resolves) and the end step, the `encore_activator` will be the NEW controller. Then, at sacrifice trigger resolution (`resolution.rs:2819`), the check `current_controller == activator` compares the (still-changed) controller against the (wrong) activator, always succeeding. The token would be incorrectly sacrificed even though another player controls it.

The token has no field recording who originally activated encore. The `encore_sacrifice_at_end_step: bool` flag only indicates that the token should be sacrificed, but not by whom.

**Fix**: Either (a) add a new `encore_activated_by: Option<PlayerId>` field to `GameObject`, set it during `EncoreAbility` resolution at `resolution.rs:2727`, and read it in `turn_actions.rs:255` instead of using `controller`; or (b) use the existing `encore_must_attack` field's presence to identify encore tokens and read the activator from a separate field. Option (a) is cleanest. The field should be initialized to `None` in all existing `GameObject` construction sites, set to `Some(activator)` during encore token creation in `resolution.rs`, hashed in `hash.rs`, and reset on zone changes in `state/mod.rs`.

#### Finding 2: No encore_must_attack cleanup at end of turn

**Severity**: LOW
**File**: `crates/engine/src/rules/turn_actions.rs:259`
**CR Rule**: 702.141a -- "attacks that opponent this turn if able"
**Issue**: The `encore_must_attack` field is set to `Some(opponent_id)` when encore tokens are created. The CR says the attack obligation is "this turn" only. If the sacrifice trigger is countered (by Stifle), the token persists to the next turn with a stale `encore_must_attack` value. Since there is no cleanup of this field in `cleanup_actions()`, the token would still carry a mandatory attack requirement on subsequent turns -- which contradicts "this turn."

In practice, even if the sacrifice is countered, the token would need to survive cleanup step (it does, since it's a permanent), and then on the next turn the stale `encore_must_attack` would still be set. Since combat enforcement of `encore_must_attack` is not yet implemented, this has no actual impact currently. But it should be cleaned up for correctness.

**Fix**: In `turn_actions.rs` `cleanup_actions()`, add a loop to clear `encore_must_attack` on all battlefield objects: `for obj in state.objects.values_mut() { if obj.zone == ZoneId::Battlefield { obj.encore_must_attack = None; } }`.

#### Finding 3: Token abilities are empty vectors (pre-existing systemic gap)

**Severity**: LOW
**File**: `crates/engine/src/rules/resolution.rs:2684`
**CR Rule**: 707.2 -- "the copy acquires the copiable values of the original object's characteristics"
**Issue**: Token creation sets `abilities: im::Vector::new()`, `activated_abilities: Vec::new()`, `triggered_abilities: Vec::new()`. This means the token won't have non-keyword abilities from the card definition populated at the `Characteristics` level. The existing workaround is that `card_id` is set on the token, and `fire_when_enters_triggered_effects` uses the registry to fire ETB triggers. However, post-ETB triggered abilities (e.g., attack triggers, damage triggers) that require `triggered_abilities` to be populated on the object would not fire.

This is a known systemic gap documented in the Embalm review (ability-review-embalm.md finding #2). Encore follows the same pattern. No Encore-specific fix needed; the systemic fix (extracting builder ability conversion into a shared function) will resolve this for all token creators.

**Fix**: No Encore-specific fix. Track as part of the systemic token ability population improvement.

#### Finding 4: Token color derivation ignores color_indicator

**Severity**: LOW
**File**: `crates/engine/src/rules/resolution.rs:2653-2670`
**CR Rule**: 202.2e -- "An object may have a color indicator printed to the left of the type line."
**Issue**: Colors are derived solely from `def.mana_cost`. Cards with color indicators (e.g., back faces of DFCs) would not get the correct colors on their tokens. However, `CardDefinition` does not currently have a `color_indicator` field, so this cannot be fixed without schema changes. Pre-existing gap across Myriad, Embalm, and Eternalize token creators.

**Fix**: No Encore-specific fix. When `CardDefinition` gains a `color_indicator` field, all token creators should use it.

#### Finding 5: No test assertion for encore_must_attack field

**Severity**: LOW
**File**: `crates/engine/tests/encore.rs:15`
**Issue**: The module-level documentation claims `encore_must_attack = Some(opponent_id)` is tested, but no test actually asserts this field's value on the created tokens. The `test_encore_basic_4p` test checks token count and zone but does not inspect the `encore_must_attack` field.

**Fix**: Add assertions to `test_encore_basic_4p` (or a dedicated test) that verify each token's `encore_must_attack` is `Some(opponent_id)` with the correct opponent for each token.

#### Finding 6: No test for Stifle on sacrifice trigger

**Severity**: LOW
**File**: `crates/engine/tests/encore.rs`
**Issue**: The plan (ability-plan-encore.md, interaction #4) identifies "Encore + Stifle on sacrifice trigger" as a key interaction: "If the EncoreSacrificeTrigger is countered (Stifle, Disallow), the token remains on the battlefield permanently." No test covers this scenario. The `EncoreSacrificeTrigger` is in the countering skip-list at `resolution.rs:3023`, so countering it would remove it from the stack without effect -- the token persists.

**Fix**: Add a test that queues the EncoreSacrificeTrigger, then counters it (or removes it from the stack), and verifies the token remains on the battlefield.

## CR Coverage Check

| CR Subrule | Implemented? | Tested? | Notes |
|------------|-------------|---------|-------|
| 702.141a (activated ability from graveyard) | Yes | Yes | test_encore_basic_4p, test_encore_card_exiled_as_cost |
| 702.141a (exile as cost) | Yes | Yes | test_encore_card_exiled_as_cost |
| 702.141a (for each opponent) | Yes | Yes | test_encore_basic_4p (3 tokens), test_encore_2p_game (1 token) |
| 702.141a (token copy of card) | Yes | Partial | Tokens get correct name/P&T/types but triggered_abilities are empty (systemic gap) |
| 702.141a (attacks that opponent this turn if able) | Yes (field set) | No | encore_must_attack set but no assertion; combat enforcement deferred |
| 702.141a (tokens gain haste) | Yes | Yes | test_encore_tokens_have_haste |
| 702.141a (sacrifice at beginning of next end step) | Yes | Yes | test_encore_sacrifice_at_end_step |
| 702.141a (activate only as a sorcery) | Yes | Yes | test_encore_sorcery_speed_opponent_turn, test_encore_sorcery_speed_non_empty_stack |
| Ruling: exile is cost, can't respond | Yes | Yes | test_encore_card_exiled_as_cost |
| Ruling: eliminated opponents not counted | Yes | Yes | test_encore_eliminated_opponent |
| Ruling: token under another's control can't be sacrificed | Partial | No | encore_activator is set to current controller, not original activator (Finding 1) |
| Ruling: tokens copy only original card | Yes | Implicit | Token characteristics from CardDefinition, not battlefield state |
| Edge: card not in graveyard | Yes | Yes | test_encore_not_in_graveyard |
| Edge: card lacks Encore keyword | Yes | Yes | test_encore_no_keyword |
| Edge: 2-player game (1 opponent) | Yes | Yes | test_encore_2p_game |
