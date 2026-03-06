# Ability Review: Recover

**Date**: 2026-03-06
**Reviewer**: ability-impl-reviewer (Opus)
**CR**: 702.59
**Files reviewed**:
- `crates/engine/src/state/types.rs:1032-1040` (KeywordAbility::Recover)
- `crates/engine/src/cards/card_definition.rs:499-507` (AbilityDefinition::Recover)
- `crates/engine/src/state/stack.rs:930-943` (StackObjectKind::RecoverTrigger)
- `crates/engine/src/state/stubs.rs:84-88,279-288` (PendingTriggerKind::Recover, recover fields)
- `crates/engine/src/state/mod.rs:158-163` (pending_recover_payments)
- `crates/engine/src/state/builder.rs:348` (initialization)
- `crates/engine/src/state/hash.rs:600-601,1742-1752,3555-3558,2651-2682,3673-3678` (hash coverage)
- `crates/engine/src/rules/abilities.rs:648-667,2892-2963,3988-3997` (find_recover_cost, trigger wiring, flush)
- `crates/engine/src/rules/resolution.rs:1470-1509,4068` (RecoverTrigger resolution, counter arm)
- `crates/engine/src/rules/engine.rs:452-471,872-971` (Command dispatch, handle_pay_recover)
- `crates/engine/src/rules/command.rs:575-587` (Command::PayRecover)
- `crates/engine/src/rules/events.rs:918-943` (RecoverPaymentRequired, RecoverPaid, RecoverDeclined)
- `tools/tui/src/play/panels/stack_view.rs:161-163` (TUI match arm)
- `crates/engine/tests/recover.rs` (8 tests)

## Verdict: clean

The Recover implementation correctly matches CR 702.59a. The trigger fires from the graveyard when a creature enters the same player's graveyard from the battlefield; it offers a pay-or-exile choice via the pending payments infrastructure; it checks CR 400.7 at both resolution and payment time; hash coverage is complete for all new types; no `.unwrap()` in library code; all match arms covered (including TUI and counter/stifle arm). Two test gaps exist (multiple Recover cards, non-creature dying negative case) but the implementation logic is correct. All findings are LOW severity.

## Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | LOW | `abilities.rs:2934` | **Wrong TriggerEvent for non-self Recover.** All Recover PendingTriggers use `SelfDies` even when the trigger is on a different card. **Fix:** Use `None` or a new `TriggerEvent::CreatureDied` variant instead of `SelfDies`. Currently harmless (only `AnyPermanentEntersBattlefield` is checked by doubler logic). |
| 2 | LOW | `tests/recover.rs` | **Missing test: multiple Recover cards trigger independently.** Plan listed `test_recover_multiple_recover_cards_in_graveyard`. The scenario where two Recover cards in the same graveyard each trigger from one creature death, with only the first resolution succeeding (CR 400.7 fizzle for the second), is untested. **Fix:** Add a test with two Recover cards in graveyard, verify two RecoverTriggers on stack, resolve first (pay), verify second fizzles. |
| 3 | LOW | `tests/recover.rs` | **Missing test: non-creature permanent dying does not trigger Recover.** Plan listed `test_recover_noncreature_dying_does_not_trigger`. **Fix:** Add a test where a non-creature permanent (artifact/enchantment) goes to graveyard via SBA and verify no RecoverTrigger is queued. |

### Finding Details

#### Finding 1: Wrong TriggerEvent for non-self Recover

**Severity**: LOW
**File**: `crates/engine/src/rules/abilities.rs:2934`
**CR Rule**: 702.59a -- "When a creature is put into your graveyard from the battlefield"
**Issue**: The `triggering_event` field on Recover PendingTriggers is set to `Some(TriggerEvent::SelfDies)` for ALL Recover triggers, including those where the Recover card is a non-creature spell in the graveyard being triggered by a different creature dying. `SelfDies` implies the source object itself died, which is only true when a creature with Recover dies and triggers its own Recover. For a sorcery with Recover in the graveyard, the source did not "die" -- it was triggered by an external event.
**Fix**: Use `triggering_event: None` for Recover triggers, since the `PendingTriggerKind::Recover` already conveys the trigger type. This is cosmetic -- the field is only functionally checked for Panharmonicon doubling (which matches `AnyPermanentEntersBattlefield`), so `SelfDies` is never matched incorrectly.

#### Finding 2: Missing test for multiple Recover cards

**Severity**: LOW
**File**: `crates/engine/tests/recover.rs`
**CR Rule**: 702.59a + 400.7
**Issue**: The plan specified `test_recover_multiple_recover_cards_in_graveyard` to verify that when two Recover cards are in the same graveyard and a creature dies, both trigger independently. This also validates the CR 400.7 interaction where the first resolved trigger (returning card to hand) causes subsequent triggers for the same card to fizzle. The test was replaced with `test_recover_no_recover_card_in_graveyard_no_trigger` (a simpler negative case).
**Fix**: Add a test with two different Recover cards (or two instances of the same) in the graveyard, kill a creature, verify two RecoverTriggers are placed on the stack, resolve the first (pay), then verify the second card's trigger also resolves independently (both cards can be returned if both are paid).

#### Finding 3: Missing test for non-creature dying

**Severity**: LOW
**File**: `crates/engine/tests/recover.rs`
**CR Rule**: 702.59a -- "When a **creature** is put into your graveyard"
**Issue**: The plan specified `test_recover_noncreature_dying_does_not_trigger` to confirm that a non-creature permanent (e.g., artifact, enchantment) entering the graveyard does NOT trigger Recover. Since the trigger wiring is inside the `GameEvent::CreatureDied` arm (which only fires for creatures), this is inherently correct, but the negative test would guard against future regressions.
**Fix**: Add a test where a non-creature permanent goes to the graveyard (e.g., via SBA or destroy effect) and verify no RecoverTrigger is queued. Low priority since the implementation structure inherently prevents this.

## CR Coverage Check

| CR Subrule | Implemented? | Tested? | Notes |
|------------|-------------|---------|-------|
| 702.59a (trigger from graveyard) | Yes | Yes | test_recover_basic_creature_dies_triggers_recover |
| 702.59a (pay cost -> return to hand) | Yes | Yes | test_recover_pay_returns_to_hand |
| 702.59a (decline -> exile) | Yes | Yes | test_recover_decline_exiles_card |
| 702.59a (self-trigger on creature death) | Yes | Yes | test_recover_creature_with_recover_dies_triggers_self |
| 702.59a (only creature deaths trigger) | Yes | No | Inherent in implementation (CreatureDied arm), but no explicit negative test |
| 702.59a (own graveyard only) | Yes | Yes | test_recover_opponents_creature_death_no_trigger |
| 702.59a (multiple Recover cards) | Yes | No | Implementation handles it (all cards in GY scanned), but no test |
| 400.7 (card left graveyard) | Yes | Yes | test_recover_card_left_graveyard_fizzles |
| 400.7 (card left graveyard at payment time) | Yes | No | Checked in handle_pay_recover:909-921, but no explicit test |

verdict: clean
