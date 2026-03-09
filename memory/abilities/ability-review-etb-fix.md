# Ability Review: ETB Trigger Engine Correctness Fix

**Date**: 2026-03-08
**Reviewer**: ability-impl-reviewer (Opus)
**CR**: 603.2, 603.3, 603.4, 603.6d, 708.3
**Files reviewed**:
- `crates/engine/src/rules/replacement.rs` (lines 910-1200)
- `crates/engine/src/rules/resolution.rs` (lines 1655-1912, 2840-2860, 3660-3690, 5888-5906, 6106-6124, 6325-6343, 6560-6580)
- `crates/engine/src/rules/lands.rs` (lines 360-440)
- `crates/engine/src/rules/abilities.rs` (lines 5724-5810, 6374-6378)
- `crates/engine/tests/card_def_fixes.rs` (Rest in Peace ETB test)
- `crates/engine/tests/corrupted.rs` (6 tests with two-pass-priority pattern)
- `crates/engine/tests/discover.rs` (5 tests with two-pass-priority pattern)
- `crates/engine/tests/exploit.rs` (5 tests including exploit+ETB interaction)
- `crates/engine/tests/tribute.rs` (TributeNotPaid tests with two-pass-priority pattern)

## Verdict: needs-fix

The core architectural change (inline execution to PendingTrigger queuing) is correct
and well-implemented. The function correctly queues `WhenEntersBattlefield` and
`TributeNotPaid` triggers as `PendingTrigger` entries with `PendingTriggerKind::Normal`,
and the existing `flush_pending_triggers` APNAP-ordered pipeline places them on the
stack as `StackObjectKind::TriggeredAbility`. The face-down guard (CR 708.3), kicker
context propagation, and Fabricate inline approximation are all handled correctly.
However, one MEDIUM finding exists: intervening-if conditions are not re-checked at
resolution time, violating CR 603.4.

## Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | MEDIUM | `resolution.rs:1888` | **Intervening-if not re-checked at resolution.** CR 603.4 requires both trigger-time AND resolution-time checks. **Fix:** evaluate `_carddef_intervening_if` at resolution. |
| 2 | LOW | `replacement.rs:1104-1108` | **Fabricate inline approximation documented but not tracked.** Comment notes it should be stack-based. No tracking issue. |
| 3 | LOW | `replacement.rs:920` | **Stale comment reference.** Doc comment references "old `fire_when_enters_triggered_effects`" -- acceptable as historical note but could be cleaned up. |
| 4 | LOW | `resolution.rs:1230` | **Stale comment in Tribute section.** Still references `fire_when_enters_triggered_effects`. |

### Finding Details

#### Finding 1: Intervening-if not re-checked at resolution time

**Severity**: MEDIUM
**File**: `crates/engine/src/rules/resolution.rs:1872-1888`
**CR Rule**: 603.4 -- "When the trigger event occurs, the ability checks whether the
stated condition is true. The ability triggers only if it is; otherwise it does nothing.
If the ability triggers, it checks the stated condition again as it resolves. If the
condition isn't true at that time, the ability is removed from the stack and does nothing."

**Issue**: At line 1872, the `intervening_if` condition extracted from the CardDef registry
is bound to `_iif` (discarded). At line 1873, `None` is returned for the condition. At
line 1888, `condition_holds` is hardcoded to `true`. This means that if a Corrupted ETB
trigger fires (opponent had 3+ poison at trigger time), but the opponent's poison counter
count drops below 3 before resolution (e.g., via Leeches or Melira), the trigger will
still resolve and draw a card -- violating CR 603.4.

Currently, the only intervening-if condition used by CardDef ETB triggers is
`Condition::OpponentHasPoisonCounters`, and reducing poison counters is extremely rare in
practice. However, future CardDef triggers with intervening-if will inherit this bug.

**Fix**: At the CardDef-registry resolution path (line 1872-1888 in resolution.rs):
1. Pass the `intervening_if` through instead of discarding it: change `_iif` to `iif`
   and return `(Some(eff), iif)` instead of `(Some(eff), None)`.
2. At line 1888, evaluate the condition using `abilities::check_intervening_if()` instead
   of hardcoding `true`. If the condition fails, skip executing the effect (but still emit
   `AbilityResolved` -- the ability resolves with no effect per CR 603.4).

#### Finding 2: Fabricate inline approximation undocumented as tracking issue

**Severity**: LOW
**File**: `crates/engine/src/rules/replacement.rs:1104-1108`
**CR Rule**: 702.123a -- "When this permanent enters" is triggered ability language.
**Issue**: The comment correctly notes Fabricate should use the stack, but there is no
tracking issue or TODO tag in the codebase audit. This is a known bot-play approximation
that will need to be fixed before human player support.
**Fix**: No action required now. Consider adding to the LOW issues remediation doc.

#### Finding 3: Stale comment in doc header

**Severity**: LOW
**File**: `crates/engine/src/rules/replacement.rs:920`
**Issue**: References "old `fire_when_enters_triggered_effects`" -- acceptable as historical
context but creates slight confusion about whether the old function still exists.
**Fix**: Optional -- remove or rephrase the historical reference.

#### Finding 4: Stale comment in Tribute section

**Severity**: LOW
**File**: `crates/engine/src/rules/resolution.rs:1230`
**Issue**: Comment still says "via fire_when_enters_triggered_effects with TributeNotPaid
condition" -- this path no longer exists. The trigger is now queued via
`queue_carddef_etb_triggers`.
**Fix**: Update comment to reference `queue_carddef_etb_triggers`.

## CR Coverage Check

| CR Subrule | Implemented? | Tested? | Notes |
|------------|-------------|---------|-------|
| 603.2 (trigger on event) | Yes | Yes | Triggers fire when permanent enters |
| 603.3 (queue at priority) | Yes | Yes | PendingTrigger queued, flush_pending_triggers places on stack |
| 603.3a (controller = source controller) | Yes | Yes | `controller` param passed through |
| 603.3b (APNAP ordering) | Yes | Yes (corrupted multiplayer) | flush_pending_triggers sorts by APNAP |
| 603.4 (intervening-if trigger time) | Yes | Yes | OpponentHasPoisonCounters checked at queue time |
| 603.4 (intervening-if resolution time) | **No** | **No** | **F1: hardcoded true at resolution** |
| 603.6a (ETB trigger) | Yes | Yes | WhenEntersBattlefield handled |
| 603.6d (static "enters with") | N/A | N/A | Not part of this fix (handled elsewhere) |
| 708.3 (face-down no ETB) | Yes | Yes | morph.rs test_morph_face_down_no_etb |

## Correctness Verification Summary

1. **Face-down guard (CR 708.3)**: Correct. Checks `obj.status.face_down && obj.face_down_as.is_some()` at function entry. Tested in morph.rs.

2. **APNAP ordering**: Correct. Triggers are queued into `state.pending_triggers` and `flush_pending_triggers` sorts by APNAP order before placing on stack.

3. **PendingTriggerKind::Normal resolution path**: Correct. At abilities.rs:6374, Normal maps to `StackObjectKind::TriggeredAbility { source_object, ability_index }`. At resolution.rs:1844-1877, the CardDef registry fallback looks up `def.abilities[ability_index]` and extracts the effect. This is the same path used by B14 end-step/upkeep triggers.

4. **No remaining old inline path**: Confirmed. All 8 call sites (7 in resolution.rs, 1 in lands.rs) call `queue_carddef_etb_triggers`. The old function name appears only in comments.

5. **Keyword ETB triggers (Fabricate, Tribute) no double-fire**: Correct. Fabricate is handled inline inside `queue_carddef_etb_triggers` (lines 1099-1200) and is NOT also a `WhenEntersBattlefield` trigger condition -- it's `AbilityDefinition::Keyword(KeywordAbility::Fabricate(n))`, which the `WhenEntersBattlefield` match arm does not match. TributeNotPaid has its own match arm (line 1038-1093) and is correctly guarded by `!tribute_was_paid`.

6. **Kicker context**: Correct. The permanent receives `kicker_times_paid` from the stack object (resolution.rs:480). The ETB trigger resolution reads it from the permanent (resolution.rs:1893-1897) and passes it to `EffectContext::new_with_kicker`. Torch Slinger's kicked ETB will work.

7. **No double-trigger risk**: Confirmed. The old inline path is completely removed. Only the queue path exists. No risk of both executing.

8. **lands.rs path**: Correct. Calls `queue_carddef_etb_triggers` at line 423 with the correct arguments. Solemn Simulacrum's ETB trigger (when it has one defined) would be correctly queued.

9. **Test quality**: All modified tests correctly use the two-pass-priority pattern (first pass resolves the spell, second pass resolves the ETB trigger on the stack). The exploit test 5 (`test_exploit_and_etb_triggers_both_on_stack`) correctly verifies both triggers are on the stack simultaneously and the draw has NOT yet occurred.
