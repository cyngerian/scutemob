# Ability Review: Echo

**Date**: 2026-03-06
**Reviewer**: ability-impl-reviewer (Opus)
**CR**: 702.30
**Files reviewed**:
- `crates/engine/src/state/types.rs:1029-1037` (KeywordAbility::Echo)
- `crates/engine/src/cards/card_definition.rs:481-490` (AbilityDefinition::Echo)
- `crates/engine/src/state/game_object.rs:523-533` (echo_pending field)
- `crates/engine/src/state/hash.rs:573-577` (KeywordAbility), `:634-643` (ManaCost), `:1696-1705` (StackObjectKind), `:2564-2580` (GameEvent), `:3442-3446` (AbilityDefinition), `:756-759` (GameObject echo_pending), `:3546-3550` (GameState pending_echo_payments)
- `crates/engine/src/state/stubs.rs:80-81` (PendingTriggerKind::EchoUpkeep), `:259-264` (echo_cost field)
- `crates/engine/src/state/stack.rs:912-919` (StackObjectKind::EchoTrigger)
- `crates/engine/src/state/mod.rs:138-143` (pending_echo_payments)
- `crates/engine/src/rules/turn_actions.rs:239-310` (upkeep trigger queueing)
- `crates/engine/src/rules/abilities.rs:3826-3836` (trigger-to-stack conversion)
- `crates/engine/src/rules/resolution.rs:502-515` (ETB echo_pending), `:1357-1399` (EchoTrigger resolution), `:3953` (counter arm)
- `crates/engine/src/rules/engine.rs:429-450` (PayEcho command handler), `:461-605` (handle_pay_echo)
- `crates/engine/src/rules/events.rs:875-894` (EchoPaymentRequired, EchoPaid)
- `crates/engine/src/rules/command.rs:550-560` (Command::PayEcho)
- `crates/engine/src/rules/lands.rs:179-194` (land ETB echo_pending)
- `crates/engine/tests/echo.rs` (9 tests)
- `tools/tui/src/play/panels/stack_view.rs:155-157` (TUI arm)
- `tools/replay-viewer/src/view_model.rs:545-547` (replay viewer arm)

## Verdict: clean

The Echo implementation is correct and complete. All CR 702.30 subrules are faithfully implemented. The `echo_pending` flag correctly models the "came under your control since the beginning of your last upkeep" condition. The trigger fires only on the controller's upkeep, the payment choice is properly deferred to the player via `Command::PayEcho`, the sacrifice path uses `check_zone_change_replacement` for commander redirect, and countering the trigger correctly leaves `echo_pending` set. All hash coverage is present. Tests cover all meaningful scenarios including multiplayer and the CR 400.7 zone-change edge case. One LOW-severity defensive code pattern is noted below but does not affect correctness.

## Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | LOW | `abilities.rs:3834` | **Defensive unwrap_or_default on echo_cost.** The `unwrap_or_default()` would silently produce a zero-cost echo if the field were somehow None. **Fix:** Replace with `.expect("EchoUpkeep triggers always carry echo_cost")` or propagate an error, since the queueing site in `turn_actions.rs:300` always sets `Some(cost)`. |
| 2 | LOW | `ability-wip.md:4` | **WIP file cites wrong CR number.** The `cr:` field says `702.31` (Horsemanship) but Echo is CR 702.30. The plan file and all implementation code correctly use 702.30. **Fix:** Update `ability-wip.md` line 4 to `cr: 702.30`. |

### Finding Details

#### Finding 1: Defensive unwrap_or_default on echo_cost

**Severity**: LOW
**File**: `crates/engine/src/rules/abilities.rs:3834`
**CR Rule**: 702.30a -- "Echo [cost]" requires a specific cost to be carried through
**Issue**: The trigger conversion uses `trigger.echo_cost.clone().unwrap_or_default()`. If `echo_cost` were `None` (which should never happen since `turn_actions.rs` always sets `Some(cost)`), this would silently create a zero-mana echo cost instead of surfacing a bug. Per `memory/conventions.md`, engine library code should avoid silently swallowing impossible states.
**Fix**: Replace `unwrap_or_default()` with `.expect("EchoUpkeep triggers always carry echo_cost")` or convert to an error return. This is LOW because the code path is unreachable in practice -- `turn_actions.rs:300` always provides `Some(cost)`.

#### Finding 2: WIP file cites wrong CR number

**Severity**: LOW
**File**: `memory/ability-wip.md:4`
**CR Rule**: 702.30 (Echo), not 702.31 (Horsemanship)
**Issue**: The `cr:` metadata field says `702.31` but the correct rule number for Echo is `702.30`. The plan file header, all doc comments, and all implementation code correctly reference `702.30`. This is a cosmetic issue in the tracking file only.
**Fix**: Change line 4 of `memory/ability-wip.md` from `cr: 702.31` to `cr: 702.30`.

## CR Coverage Check

| CR Subrule | Implemented? | Tested? | Notes |
|------------|-------------|---------|-------|
| 702.30a (triggered ability definition) | Yes | Yes | test_echo_upkeep_trigger_fires |
| 702.30a ("your upkeep") | Yes | Yes | test_echo_multiplayer_only_controller_upkeep |
| 702.30a ("came under your control since...") | Yes | Yes | echo_pending flag; test_echo_etb_sets_pending, test_echo_no_trigger_after_paid |
| 702.30a ("sacrifice it unless you pay") | Yes | Yes | test_echo_pay_cost_keeps_permanent, test_echo_decline_payment_sacrifices |
| 702.30b (echo cost = mana cost for Urza block) | Yes | Yes | test_echo_different_cost verifies cost independence |
| CR 400.7 (permanent left battlefield) | Yes | Yes | test_echo_permanent_left_battlefield |
| Stifle interaction (counter preserves flag) | Yes (counter arm at resolution.rs:3953) | No (no explicit test) | Correct by design: flag cleared only in PayEcho handler |
| Commander redirect on sacrifice | Yes (check_zone_change_replacement) | No (no explicit test) | Follows established Fading pattern |

verdict: clean
