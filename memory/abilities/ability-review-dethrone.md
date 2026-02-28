# Ability Review: Dethrone

**Date**: 2026-02-27
**Reviewer**: ability-impl-reviewer (Opus)
**CR**: 702.105
**Files reviewed**:
- `crates/engine/src/state/types.rs:372-378` (KeywordAbility::Dethrone)
- `crates/engine/src/state/game_object.rs:180-185` (TriggerEvent::SelfAttacksPlayerWithMostLife)
- `crates/engine/src/state/hash.rs:415-416` (KeywordAbility discriminant 55)
- `crates/engine/src/state/hash.rs:1047-1048` (TriggerEvent discriminant 14)
- `crates/engine/src/state/builder.rs:464-486` (Dethrone TriggeredAbilityDef auto-generation)
- `crates/engine/src/rules/abilities.rs:966-1001` (AttackersDeclared dethrone handler)
- `tools/replay-viewer/src/view_model.rs:637` (format_keyword arm)
- `crates/engine/tests/dethrone.rs` (8 tests, 679 lines)

## Verdict: clean

The Dethrone implementation is correct, complete, and well-tested. All CR 702.105 subrules
are implemented faithfully. The life-total comparison logic correctly compares the defending
player's life against the maximum across all active (non-eliminated) players. The dedicated
`TriggerEvent::SelfAttacksPlayerWithMostLife` variant cleanly separates the conditional
trigger dispatch from the generic `SelfAttacks` path. Planeswalker attacks are excluded by
the `if let AttackTarget::Player(def_pid)` guard. Multiple instances each produce separate
`TriggeredAbilityDef` entries via the `for kw in spec.keywords.iter()` loop in builder.rs,
satisfying CR 702.105b. Hash discriminants are unique and contiguous. All 8 tests pass and
cover positive, negative, multiplayer, tied-for-most, planeswalker, multiple-instance, and
self-life-edge-case scenarios. No HIGH or MEDIUM findings.

## Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | LOW | `abilities.rs:977` | **unwrap_or(i32::MIN) for defending_life is safe but fragile.** If a player is being attacked, they should always exist in `state.players`. The fallback to `i32::MIN` silently hides a bug (missing player). **Fix:** Consider logging or debug-asserting that the player exists, though this is a defensive pattern consistent with the rest of the codebase. No action required. |
| 2 | LOW | `dethrone.rs:551` | **Planeswalker test uses `ObjectSpec::card` without `enrich_spec_from_def`.** The test creates a "Test Planeswalker" via `ObjectSpec::card(p2, "Test Planeswalker").with_types(...)` which is the standard test-only approach. This works because the test only needs the planeswalker to exist as an attack target, not to function as a real planeswalker (no loyalty, no abilities). Documented pattern per `memory/gotchas-infra.md`. No action required. |
| 3 | LOW | `dethrone.rs` | **Missing test for eliminated-player exclusion from max-life calculation.** The plan mentions that eliminated players should not count in the life comparison, and the implementation correctly filters them with `!p.has_lost && !p.has_conceded`. However, no test verifies that an eliminated player with a high life total (before elimination) is excluded from the max. **Fix:** Add a test where a 4-player game has an eliminated player at 50 life, and verify that attacking a player with 30 life (now the true max among active players) triggers dethrone. Low priority since the filter logic is straightforward and used elsewhere. |

### Finding Details

#### Finding 1: unwrap_or(i32::MIN) defensive fallback

**Severity**: LOW
**File**: `crates/engine/src/rules/abilities.rs:977`
**CR Rule**: 702.105a -- "Whenever this creature attacks the player with the most life"
**Issue**: The `defending_life` lookup uses `.unwrap_or(i32::MIN)` as a fallback when the
defending player is not found in `state.players`. In practice, if a player is being attacked,
they must exist in the players map (the `DeclareAttackers` command would have failed
validation otherwise). The fallback silently converts a should-never-happen state into
"defending player has minimum life," which would cause dethrone to not trigger -- a correct
but misleading failure mode. This pattern is consistent with other code in the engine (e.g.,
the `max_life` computation on the next line), so it is not an inconsistency.
**Fix**: No action required. If desired, a `debug_assert!` could be added, but this is
consistent with existing codebase patterns.

#### Finding 2: Planeswalker test pattern

**Severity**: LOW
**File**: `crates/engine/tests/dethrone.rs:551`
**CR Rule**: Ruling 2023-07-28 -- "Dethrone doesn't trigger if the creature attacks a
planeswalker"
**Issue**: The test creates a planeswalker using `ObjectSpec::card()` with manually set types.
This is the standard approach for test objects that don't need full CardDefinition enrichment.
The test correctly validates that attacking a planeswalker does not fire dethrone.
**Fix**: No action required. The test correctly validates the behavior.

#### Finding 3: Missing eliminated-player exclusion test

**Severity**: LOW
**File**: `crates/engine/tests/dethrone.rs`
**CR Rule**: 702.105a -- "player with the most life" implies only active game participants
**Issue**: The implementation at `abilities.rs:981` filters eliminated players from the
max-life calculation with `.filter(|p| !p.has_lost && !p.has_conceded)`. This is correct.
However, no test verifies this specific edge case -- all tests use only active players.
A test with an eliminated high-life player would increase confidence in this path.
**Fix**: Optionally add a test: 4 players where P4 has the highest life (e.g., 50) but
`has_lost = true`. P1 attacks P2 (who has 30 life, the highest among active players).
Assert that dethrone triggers. Low priority.

## CR Coverage Check

| CR Subrule | Implemented? | Tested? | Notes |
|------------|-------------|---------|-------|
| 702.105a "attacks the player with the most life" | Yes | Yes | test_dethrone_basic_attacks_player_with_most_life |
| 702.105a "or tied for most life" | Yes | Yes | test_dethrone_tied_for_most_life |
| 702.105a "put a +1/+1 counter on this creature" | Yes | Yes | All positive tests verify counter + P/T |
| 702.105a negative: not most life | Yes | Yes | test_dethrone_does_not_trigger_against_lower_life |
| 702.105a multiplayer: all players compared | Yes | Yes | test_dethrone_multiplayer_four_player_most_life |
| 702.105a multiplayer negative | Yes | Yes | test_dethrone_multiplayer_not_most_life |
| 702.105a planeswalker exclusion | Yes | Yes | test_dethrone_does_not_trigger_on_planeswalker_attack |
| 702.105a self-life edge case | Yes | Yes | test_dethrone_attacker_has_most_life_attacks_lower |
| 702.105b multiple instances | Yes | Yes | test_dethrone_multiple_instances_trigger_separately |
| 702.105a eliminated player exclusion | Yes | No | Implementation filters; no dedicated test (LOW finding 3) |
