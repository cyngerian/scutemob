# Ability Review: Opponent-Casts Trigger

**Date**: 2026-02-26
**Reviewer**: ability-impl-reviewer (Opus)
**CR**: 603.2, 102.2, 102.3, 903.2
**Files reviewed**:
- `crates/engine/src/state/game_object.rs` (TriggerEvent::OpponentCastsSpell)
- `crates/engine/src/state/stubs.rs` (triggering_player field on PendingTrigger)
- `crates/engine/src/state/hash.rs` (TriggerEvent discriminant 10, PendingTrigger hash)
- `crates/engine/src/rules/abilities.rs` (dispatch at lines 440-466, flush at lines 728-740)
- `crates/engine/src/testing/replay_harness.rs` (WheneverOpponentCastsSpell enrichment at lines 502-521)
- `crates/engine/tests/effects.rs` (5 tests at lines 1016-1527)
- `crates/engine/src/cards/definitions.rs` (Rhystic Study card def at lines 1103-1133)
- `crates/engine/src/cards/card_definition.rs` (TriggerCondition::WheneverOpponentCastsSpell at line 492)

## Verdict: clean

The implementation correctly models the Opponent-Casts Trigger pattern per CR 603.2 and
CR 102.2/102.3. The opponent check (`controller != caster`) is applied at trigger-collection
time, matching the existing patterns for Prowess (controller match) and Ward (opponent
match). The `triggering_player` field on `PendingTrigger` correctly propagates the casting
player through to the stack entry's target list, enabling future `MayPayOrElse` interactive
payment resolution. Hash coverage is complete. All five tests pass and cover positive,
negative, multiplayer, and target-propagation scenarios. No HIGH or MEDIUM findings. Two
LOW findings noted below regarding a minor plan inaccuracy and a test gap.

## Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | LOW | `memory/ability-plan-opponent-casts-trigger.md:31` | **Plan incorrectly claims storm copies emit SpellCast.** Storm copies use `copy_spell_on_stack` which emits `SpellCopied`, not `SpellCast`. The concern about storm copies triggering `OpponentCastsSpell` is unfounded. No code fix needed -- the plan is a read-only reference document. |
| 2 | LOW | `crates/engine/tests/effects.rs` | **No test for Rhystic Study card-def enrichment path.** Tests manually construct `ObjectSpec` with `with_triggered_ability()` rather than using `enrich_spec_from_def()` with the actual Rhystic Study `CardDefinition`. The enrichment block in `replay_harness.rs:502-521` is untested by the unit tests (only exercised by game scripts). |
| 3 | LOW | `crates/engine/tests/effects.rs:1108` | **Priority pass loop uses hardcoded iteration count.** The drain loop `for _ in 0..6` (line 1108) and `for _ in 0..10` (line 1318, 1426) could silently succeed with a non-empty stack if the iteration count is too low. This is a pre-existing pattern, not introduced by this change. |

### Finding Details

#### Finding 1: Plan incorrectly claims storm copies emit SpellCast

**Severity**: LOW
**File**: `memory/ability-plan-opponent-casts-trigger.md:31`
**CR Rule**: 702.40a -- "Storm" / 707.10c -- copies are not cast
**Issue**: The plan states: "Storm copies emit `SpellCast` events and would incorrectly
trigger `OpponentCastsSpell`." This is incorrect. Inspecting `crates/engine/src/rules/copy.rs:197-211`,
`create_storm_copies` calls `copy_spell_on_stack`, which emits `GameEvent::SpellCopied`
(line 178), not `GameEvent::SpellCast`. The `SpellCast` emission at `copy.rs:347` is from
the cascade path, which IS a real cast (CR 702.85c). Storm copies never enter the
`check_triggers` SpellCast arm. No code fix is needed -- the plan is a historical document.
The implementation is correct.
**Fix**: No code change needed. If the plan is updated for reference, clarify that storm
copies emit `SpellCopied`, not `SpellCast`.

#### Finding 2: No test for card-def enrichment path

**Severity**: LOW
**File**: `crates/engine/tests/effects.rs`
**CR Rule**: 603.2 -- triggered abilities must be wired from card definitions
**Issue**: All five tests manually construct the triggered ability via
`ObjectSpec::card(...).with_triggered_ability(TriggeredAbilityDef { trigger_on: TriggerEvent::OpponentCastsSpell, ... })`.
None of them exercise the `enrich_spec_from_def()` path in `replay_harness.rs:502-521`
that converts `TriggerCondition::WheneverOpponentCastsSpell` into
`TriggerEvent::OpponentCastsSpell`. This enrichment path is only tested indirectly via
game scripts. A unit test using `enrich_spec_from_def()` with the Rhystic Study
`CardDefinition` would strengthen coverage.
**Fix**: Add a test that creates a Rhystic Study via `enrich_spec_from_def()` and verifies
the resulting `ObjectSpec` has a `TriggeredAbilityDef` with `trigger_on: TriggerEvent::OpponentCastsSpell`.
This is low priority -- the game script pipeline will cover this path.

#### Finding 3: Priority pass drain loops use hardcoded iteration counts

**Severity**: LOW
**File**: `crates/engine/tests/effects.rs:1108`
**CR Rule**: N/A (test quality)
**Issue**: The drain loops use `for _ in 0..6` or `for _ in 0..10` with an early break on
empty stack. If the stack never empties (e.g., due to a bug introducing an infinite trigger
loop), the test would silently pass with a non-empty stack. The assertion after the loop
checks hand size, which would catch most issues, but a direct `assert!(current.stack_objects.is_empty())`
after the loop would be more robust. This is a pre-existing pattern used throughout the
test file, not introduced by this change.
**Fix**: Optionally add `assert!(current.stack_objects.is_empty(), "stack should be fully resolved")`
after each drain loop. Low priority.

## CR Coverage Check

| CR Subrule | Implemented? | Tested? | Notes |
|------------|-------------|---------|-------|
| 603.2 (trigger on event) | Yes | Yes | test_rhystic_study_draws_card_when_opponent_casts |
| 603.2a (triggers even when can't cast) | N/A | N/A | Inherent in trigger system -- triggers don't check legality |
| 603.2c (triggers once per event; multiple sources each trigger) | Yes | Yes | test_opponent_casts_trigger_multiple_studies_each_trigger_independently |
| 603.2g (prevented/replaced events don't trigger) | N/A | N/A | SpellCast event only emitted on successful cast |
| 603.3a (controller = source's controller at trigger time) | Yes | Yes | test_opponent_casts_trigger_multiplayer_fires_for_correct_player asserts controller == p1 |
| 603.3b (APNAP ordering) | Yes | Implicit | flush_pending_triggers sorts by APNAP; not directly tested for opponent-casts but tested elsewhere |
| 102.2 (opponent = other player in two-player) | Yes | Yes | test_rhystic_study_draws_card_when_opponent_casts (2-player, p2 is opponent) |
| 102.2 (NOT opponent when self casts) | Yes | Yes | test_opponent_casts_trigger_does_not_fire_on_own_spell |
| 102.3 / 903.2 (FFA multiplayer = all others are opponents) | Yes | Yes | test_opponent_casts_trigger_multiplayer_fires_for_correct_player (4-player) |
| triggering_player propagation | Yes | Yes | test_opponent_casts_trigger_carries_casting_player_as_target |
| Hash coverage (TriggerEvent discriminant) | Yes | N/A | hash.rs:905 -- discriminant 10 |
| Hash coverage (PendingTrigger.triggering_player) | Yes | N/A | hash.rs:865 |
| Hash coverage (TriggerCondition::WheneverOpponentCastsSpell) | Yes | N/A | hash.rs:1852 -- discriminant 5 |
| Enrichment (WheneverOpponentCastsSpell -> OpponentCastsSpell) | Yes | No (indirect) | replay_harness.rs:502-521; tested only by game scripts |

## Previous Findings (re-review only)

N/A -- first review.
