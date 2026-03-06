# Ability Review: Cumulative Upkeep

**Date**: 2026-03-06
**Reviewer**: ability-impl-reviewer (Opus)
**CR**: 702.24
**Files reviewed**:
- `crates/engine/src/state/types.rs` (CumulativeUpkeepCost enum, KeywordAbility variant, CounterType::Age)
- `crates/engine/src/state/hash.rs` (HashInto impls for CumulativeUpkeepCost, CounterType::Age, KeywordAbility, AbilityDefinition, StackObjectKind, GameEvent, GameState)
- `crates/engine/src/state/mod.rs` (pending_cumulative_upkeep_payments field, re-exports)
- `crates/engine/src/state/stack.rs` (StackObjectKind::CumulativeUpkeepTrigger)
- `crates/engine/src/state/stubs.rs` (PendingTriggerKind::CumulativeUpkeep, cumulative_upkeep_cost field)
- `crates/engine/src/cards/card_definition.rs` (AbilityDefinition::CumulativeUpkeep)
- `crates/engine/src/cards/helpers.rs` (CumulativeUpkeepCost re-export)
- `crates/engine/src/lib.rs` (CumulativeUpkeepCost re-export)
- `crates/engine/src/rules/turn_actions.rs` (upkeep trigger queueing)
- `crates/engine/src/rules/abilities.rs` (trigger-to-stack conversion)
- `crates/engine/src/rules/resolution.rs` (CumulativeUpkeepTrigger resolution, countered trigger arm)
- `crates/engine/src/rules/engine.rs` (PayCumulativeUpkeep command handler, handle_pay_cumulative_upkeep, multiply_mana_cost)
- `crates/engine/src/rules/command.rs` (Command::PayCumulativeUpkeep)
- `crates/engine/src/rules/events.rs` (CumulativeUpkeepPaymentRequired, CumulativeUpkeepPaid)
- `crates/engine/tests/cumulative_upkeep.rs` (8 tests)
- `tools/replay-viewer/src/view_model.rs` (StackObjectKind arm)
- `tools/tui/src/play/panels/stack_view.rs` (StackObjectKind arm)

## Verdict: needs-fix

The implementation is structurally sound and correctly models the CR 702.24a/b mechanics:
age counter placement during trigger resolution, escalating costs, separation of multiple
instances, and the sacrifice-on-decline path. However, there is one HIGH issue (`.expect()`
in library code) and one MEDIUM issue (life loss not tracked for Spectacle interaction).

## Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | **HIGH** | `abilities.rs:3862` | **`.expect()` in engine library code.** Panics if `cumulative_upkeep_cost` is None. **Fix:** use `.unwrap_or` with a default, or return a fallback StackObjectKind, matching the Echo pattern (line 3848: `.unwrap_or_default()`). |
| 2 | MEDIUM | `engine.rs:733-734` | **Life loss from CU payment not tracked in `life_lost_this_turn`.** Spectacle and similar mechanics will not see this life loss. **Fix:** add `ps.life_lost_this_turn += total_life;` before the life_total subtraction, matching the pattern in `effects/mod.rs:344` and `combat.rs:1439`. |
| 3 | LOW | `cumulative_upkeep.rs:623-674` | **Test 8 (702.24b) incomplete.** Only checks two triggers exist on the stack; does not resolve both and verify each counts all age counters independently. **Fix:** extend test to resolve both triggers sequentially and assert 2 age counters total, with two separate payment-required events each reflecting the correct age_counter_count at resolution time. |

### Finding Details

#### Finding 1: `.expect()` in engine library code

**Severity**: HIGH
**File**: `crates/engine/src/rules/abilities.rs:3862`
**CR Rule**: N/A -- architecture invariant (conventions.md: "never `unwrap()` or `expect()` in engine logic")
**Issue**: The trigger-to-stack conversion for `PendingTriggerKind::CumulativeUpkeep` uses `.expect("CumulativeUpkeep trigger must have cost")`, which will panic if a `PendingTrigger` with `kind == CumulativeUpkeep` but `cumulative_upkeep_cost == None` reaches this code. While this "should never happen" in practice (the trigger queueing in `turn_actions.rs` always sets the cost), the Echo trigger at line 3848 demonstrates the correct pattern: `.unwrap_or_default()`. A malformed PendingTrigger from a future code path or deserialization could trigger a panic.
**Fix**: Replace `.expect("CumulativeUpkeep trigger must have cost")` with `.unwrap_or(CumulativeUpkeepCost::Mana(ManaCost::default()))` or equivalent safe fallback, consistent with the Echo pattern on line 3848.

#### Finding 2: Life loss not tracked for Spectacle

**Severity**: MEDIUM
**File**: `crates/engine/src/rules/engine.rs:733-734`
**CR Rule**: CR 118.4 -- "If a cost or effect allows a player to pay an amount of life greater than 0 [...] the payment is a loss of life"
**Issue**: When a player pays life for cumulative upkeep (`CumulativeUpkeepCost::Life`), the handler subtracts from `life_total` and emits `GameEvent::LifeLost`, but does not increment `life_lost_this_turn`. Every other life-loss path in the engine (combat.rs:1439, effects/mod.rs:200, effects/mod.rs:344, effects/mod.rs:376) updates this counter. This means paying life for CU (e.g., Glacial Chasm) would not enable Spectacle costs for the paying player's opponents, which is incorrect per CR 118.4.
**Fix**: Add `p.life_lost_this_turn += total_life;` inside the `if let Some(p) = state.players.get_mut(&player)` block at line 733, before or after the `life_total` subtraction.

#### Finding 3: Test 8 (702.24b) does not verify counter sharing behavior

**Severity**: LOW
**File**: `crates/engine/tests/cumulative_upkeep.rs:623-674`
**CR Rule**: 702.24b -- "each cumulative upkeep ability will count the total number of age counters on the permanent at the time that ability resolves"
**Issue**: The test creates a permanent with two CU instances and verifies two triggers appear on the stack, but stops there. It does not resolve the triggers to verify: (a) each trigger adds one age counter (2 total after both resolve), (b) the second trigger to resolve counts both age counters (the one from its own resolution plus the one from the first trigger's resolution). This is the core behavior of 702.24b.
**Fix**: Extend the test to pass priority twice more (resolving both triggers), then assert `age_counters_on == 2` and verify the second `CumulativeUpkeepPaymentRequired` event has `age_counter_count == 2`.

## CR Coverage Check

| CR Subrule | Implemented? | Tested? | Notes |
|------------|-------------|---------|-------|
| 702.24a (trigger fires on upkeep) | Yes | Yes | test 1 (basic_age_counter_added) |
| 702.24a (age counter added first) | Yes | Yes | test 1, test 4 |
| 702.24a (pay cost x counters, keep) | Yes | Yes | test 2 (pay_mana_keeps_permanent) |
| 702.24a (decline = sacrifice) | Yes | Yes | test 3 (decline_payment_sacrifices) |
| 702.24a (escalating cost) | Yes | Yes | test 4 (escalating_cost) -- 2 upkeeps verified |
| 702.24a (no partial payments) | Yes | Implicit | Engine only offers pay-all-or-nothing via bool |
| 702.24a (life cost variant) | Yes | Yes | test 5 (pay_life_cost) |
| 702.24a ("your upkeep" only) | Yes | Yes | test 7 (multiplayer_only_controller_upkeep) |
| 702.24b (multiple instances separate) | Yes | Partial | test 8 checks stack count only (LOW finding) |
| 702.24b (share age counters) | Yes | No | Not verified at resolution time (LOW finding) |
| CR 400.7 (permanent left battlefield) | Yes | Yes | test 6 (permanent_left_battlefield) |
| Stifle/counter interaction | Yes | No | Noted in countered-trigger arm but no test |
| Commander sacrifice redirect | Yes | No | Uses check_zone_change_replacement but no test |

verdict: needs-fix
