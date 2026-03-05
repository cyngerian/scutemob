# Ability Review: Spectacle

**Date**: 2026-03-04
**Reviewer**: ability-impl-reviewer (Opus)
**CR**: 702.137
**Files reviewed**:
- `crates/engine/src/state/player.rs` (field: `life_lost_this_turn`)
- `crates/engine/src/state/types.rs` (`AltCostKind::Spectacle`, `KeywordAbility::Spectacle`)
- `crates/engine/src/cards/card_definition.rs` (`AbilityDefinition::Spectacle`)
- `crates/engine/src/state/hash.rs` (hash coverage for all new fields/variants)
- `crates/engine/src/state/builder.rs` (builder default)
- `crates/engine/src/rules/turn_actions.rs` (reset in `reset_turn_state`)
- `crates/engine/src/effects/mod.rs` (3 life-loss tracking sites)
- `crates/engine/src/rules/combat.rs` (1 life-loss tracking site)
- `crates/engine/src/rules/casting.rs` (validation, mutual exclusion, cost substitution, helper)
- `crates/engine/src/testing/replay_harness.rs` (`cast_spell_spectacle` action)
- `tools/replay-viewer/src/view_model.rs` (display arm)
- `crates/engine/tests/spectacle.rs` (10 tests)
- `crates/engine/src/state/stack.rs` (verified no new fields needed)
- `crates/engine/src/rules/resolution.rs` (verified `cast_alt_cost` chain)

## Verdict: needs-fix

The core Spectacle implementation is CR-correct and well-structured. The life-loss
tracking infrastructure (`life_lost_this_turn`) is properly placed at all 4 life-decrement
sites, correctly excludes infect damage, and resets at turn boundaries. Hash coverage is
complete. Mutual exclusion with all 17 other alternative costs is comprehensive.

Two MEDIUM findings require attention: the mutual exclusion test does not actually test
mutual exclusion (it asserts success, not rejection), and the commander tax test specified
in the plan was not implemented. One LOW finding notes an edge case with eliminated players
in the opponent check.

## Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | MEDIUM | `tests/spectacle.rs:382` | **Mutual exclusion test does not test mutual exclusion.** Test asserts cast succeeds instead of verifying rejection when combining alt costs. **Fix:** rewrite to actually combine spectacle with another alt cost (e.g., use overload card with spectacle attempt). |
| 2 | MEDIUM | `tests/spectacle.rs` | **Missing commander tax test.** Plan specified test #8 (`test_spectacle_commander_tax_applies`, CR 118.9d) but it was not implemented. **Fix:** add test that casts a commander with spectacle + commander tax and verifies total cost includes both. |
| 3 | LOW | `casting.rs:1398` | **Opponent check does not exclude eliminated players.** Check uses `*pid != player` but doesn't filter `ps.has_lost`. An eliminated player who lost life earlier in the turn would still enable spectacle. CR 702.137a says "an opponent" -- eliminated players are no longer opponents (CR 800.4a). In practice this rarely matters (the life loss happened while they were still an opponent). **Fix:** add `&& !ps.has_lost && !ps.has_conceded` to the filter. |

### Finding Details

#### Finding 1: Mutual exclusion test does not test mutual exclusion

**Severity**: MEDIUM
**File**: `crates/engine/tests/spectacle.rs:382`
**CR Rule**: 118.9a -- "A player can't apply two alternative costs to a single spell."
**Issue**: The test `test_spectacle_mutual_exclusion_rejects_combined_costs` is named as if
it verifies CR 118.9a mutual exclusion, but its body only asserts that a valid spectacle
cast succeeds (`result.is_ok()`). The lengthy comment acknowledges the difficulty of
exercising the mutual-exclusion code path but then settles for a success assertion. This
leaves the mutual exclusion enforcement entirely untested.
**Fix**: Rewrite the test to actually exercise mutual exclusion. The simplest approach: create
a card that has both Spectacle and Overload (or Spectacle and Dash) keywords, then attempt
to cast with `alt_cost: Some(AltCostKind::Spectacle)` from a context where a competing alt
cost flag would also be set. Alternatively, test the reverse: try to cast with
`alt_cost: Some(AltCostKind::Dash)` on a card that also has spectacle, and verify it works
normally (the dash block's `cast_with_spectacle` check is false since alt_cost is Dash).
The most practical test: since `alt_cost` is a single value, true mutual exclusion testing
is structurally difficult. At minimum, rename the test to match what it actually verifies
(`test_spectacle_valid_cast_with_preconditions_met`).

#### Finding 2: Missing commander tax test

**Severity**: MEDIUM
**File**: `crates/engine/tests/spectacle.rs` (missing)
**CR Rule**: 118.9d -- "If a spell has an alternative cost, additional costs, cost
increases, and cost reductions that apply to it are applied to that alternative cost."
**Issue**: The plan explicitly listed test #8 (`test_spectacle_commander_tax_applies`) to
verify that commander tax is added on top of the spectacle cost when casting from the
command zone. This test was not implemented. While the code path (casting.rs lines
1546-1570) handles this correctly via the generic commander tax mechanism, the absence of
a test means a regression could go undetected.
**Fix**: Add `test_spectacle_commander_tax_applies` with a commander card that has spectacle,
cast it once from the command zone (to establish tax), then attempt a second cast with
`alt_cost: Some(AltCostKind::Spectacle)` and verify the total cost includes the spectacle
cost plus the {2} commander tax.

#### Finding 3: Opponent check does not exclude eliminated players

**Severity**: LOW
**File**: `crates/engine/src/rules/casting.rs:1398`
**CR Rule**: 800.4a -- "If a player leaves the game, all objects owned by that player leave
the game and any effects which give that player control of any objects or players end."
CR 102.3 -- "An opponent is someone a player is playing against."
**Issue**: The spectacle precondition check iterates all players and checks
`*pid != player && ps.life_lost_this_turn > 0`. This includes players who have already
lost the game (`ps.has_lost == true`). An eliminated player is no longer an opponent per
CR 800.4a. In practice this is extremely unlikely to matter (the life loss happened while
they were still an active opponent, and the engine retains eliminated player state for
history), but it is technically imprecise.
**Fix**: Add `&& !ps.has_lost && !ps.has_conceded` to the filter predicate at line 1398:
```rust
.any(|(pid, ps)| *pid != player && !ps.has_lost && !ps.has_conceded && ps.life_lost_this_turn > 0);
```

## CR Coverage Check

| CR Subrule | Implemented? | Tested? | Notes |
|------------|-------------|---------|-------|
| 702.137a (spectacle cost as alternative cost) | Yes | Yes | test_spectacle_basic_cast_after_opponent_life_loss |
| 702.137a (opponent life loss precondition) | Yes | Yes | test_spectacle_rejected_when_no_opponent_lost_life |
| 702.137a (optional -- normal cast allowed) | Yes | Yes | test_spectacle_normal_cast_without_spectacle |
| 702.137a (multiplayer -- any opponent) | Yes | Yes | test_spectacle_multiplayer_any_opponent_enables |
| 702.137a (caster life loss not sufficient) | Yes | Yes | test_spectacle_own_life_loss_does_not_enable |
| 702.137a (turn-scoped reset) | Yes | Yes | test_spectacle_life_lost_counter_resets_on_turn_boundary |
| 702.137b (cost calculation -- commander tax) | Yes | No | Missing test (Finding 2) |
| 702.137c (cost in ability definition) | Yes | Yes | get_spectacle_cost helper + validation |
| CR 118.9a (mutual exclusion) | Yes | No | Test exists but doesn't test mutual exclusion (Finding 1) |
| CR 702.90b (infect does not enable) | Yes | Yes | test_spectacle_life_lost_counter_not_set_for_infect |
| Life loss from LoseLife effect | Yes | Yes | test_spectacle_life_lost_counter_tracks_lose_life_effect |
| Life loss from DealDamage effect | Yes | No | Simulated, not exercised through engine |
| Life loss from DrainLife effect | Yes | No | Tracking added but not tested |
| Life loss from combat damage | Yes | No | Tracking added but not tested |
| Damage prevention interaction | Yes (structural) | No | `final_dmg > 0` guard prevents tracking for prevented damage |
| No keyword rejects spectacle | Yes | Yes | test_spectacle_no_keyword_rejects |
