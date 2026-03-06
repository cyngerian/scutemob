# Ability Review: Forecast

**Date**: 2026-03-06
**Reviewer**: ability-impl-reviewer (Opus)
**CR**: 702.57
**Files reviewed**:
- `crates/engine/src/state/types.rs:1076-1081` (KeywordAbility::Forecast)
- `crates/engine/src/cards/card_definition.rs:508-516` (AbilityDefinition::Forecast)
- `crates/engine/src/state/hash.rs:601-603` (KW hash), `:1755-1763` (SOK hash), `:3697-3700` (GameState hash)
- `crates/engine/src/state/mod.rs:165-170` (forecast_used_this_turn field)
- `crates/engine/src/state/builder.rs:349` (field init)
- `crates/engine/src/rules/command.rs:387-401` (Command::ActivateForecast)
- `crates/engine/src/rules/engine.rs:242-261` (command handler dispatch)
- `crates/engine/src/rules/abilities.rs:669-870` (handle_activate_forecast)
- `crates/engine/src/state/stack.rs:944-956` (StackObjectKind::ForecastAbility)
- `crates/engine/src/rules/resolution.rs:827-846` (ForecastAbility resolution)
- `crates/engine/src/testing/replay_harness.rs:583-593` (activate_forecast action)
- `crates/engine/src/rules/turn_actions.rs:1051-1052` (reset_turn_state)
- `crates/engine/tests/forecast.rs` (9 tests)
- `tools/replay-viewer/src/view_model.rs:554-556,818` (view model arms)
- `tools/tui/src/play/panels/stack_view.rs:164-165` (TUI arm)

## Verdict: clean

The Forecast implementation correctly enforces all sub-rules of CR 702.57. The handler
validates priority, split second, upkeep step, owner's upkeep, hand zone, keyword presence,
and once-per-turn -- in the right order. The card stays in hand (no zone move), the ability
goes on the stack and resolves via the standard effect pipeline. Hash coverage is complete
for the new GameState field and the new StackObjectKind variant. All exhaustive match sites
(TUI, replay viewer, hash) have been updated. Tests cover positive activation, wrong step,
wrong player's upkeep, once-per-turn enforcement, turn reset, card-stays-in-hand after
resolution, effect execution, split second blocking, and missing keyword. Two LOW-severity
observations noted below; neither affects correctness.

## Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | LOW | `abilities.rs:752` | **Fragile once-per-turn guard for None card_id.** See details. |
| 2 | LOW | `forecast.rs:433` | **Turn reset test manually clears field instead of using engine reset.** See details. |

### Finding Details

#### Finding 1: Fragile once-per-turn guard for None card_id

**Severity**: LOW
**File**: `crates/engine/src/rules/abilities.rs:752`
**CR Rule**: 702.57b -- "only once each turn"
**Issue**: If `card_id_opt` is `None`, the once-per-turn check (step 7) and the mark-as-used
(step 10) are both silently skipped via `if let Some(...)`. In practice this is unreachable
because step 8's registry lookup also requires a `card_id` and would fail first, so no actual
bug exists. However, the defense-in-depth is incomplete -- a future refactor that changes the
lookup path could expose unlimited activations.
**Fix**: Add a guard at step 7: if `card_id_opt` is `None`, return an error
(`"ActivateForecast: card has no CardId; cannot track once-per-turn (CR 702.57b)"`).
This makes the once-per-turn enforcement self-contained rather than relying on a later step.

#### Finding 2: Turn reset test uses manual field clear

**Severity**: LOW
**File**: `crates/engine/tests/forecast.rs:433`
**CR Rule**: 702.57b -- "only once each turn"
**Issue**: `test_forecast_resets_each_turn` manually sets
`state.forecast_used_this_turn = im::OrdSet::new()` instead of exercising the actual
`reset_turn_state` path. This means the test would still pass even if someone removed
the reset from `turn_actions.rs`. The reset IS present (confirmed at `turn_actions.rs:1052`),
but the test does not validate it.
**Fix**: Ideally, advance the game to a new turn via engine commands (e.g., pass through all
phases) so `reset_turn_state` runs naturally. If that is too complex for a unit test, add a
comment explaining why the manual reset is used and add a separate assertion that
`reset_turn_state` clears the field (or rely on a future integration/script test).

## CR Coverage Check

| CR Subrule | Implemented? | Tested? | Notes |
|------------|-------------|---------|-------|
| 702.57a (activated ability from hand) | Yes | Yes | test 1 (basic), test 6 (stays in hand), test 9 (keyword required) |
| 702.57b (upkeep step only) | Yes | Yes | test 2 (wrong step) |
| 702.57b (owner's upkeep only) | Yes | Yes | test 3 (opponent's upkeep) |
| 702.57b (once each turn) | Yes | Yes | test 4 (second activation fails) |
| 702.57b (resets each turn) | Yes | Partial | test 5 (manual reset, not engine-driven) |
| 702.57b (card revealed) | No (cosmetic) | No | Plan correctly defers reveal state as informational |
| Split second interaction | Yes | Yes | test 8 |
| Effect resolution | Yes | Yes | test 7 (draw effect) |

verdict: clean
