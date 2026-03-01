# Ability Review: Enlist

**Date**: 2026-03-01
**Reviewer**: ability-impl-reviewer (Opus)
**CR**: 702.154
**Files reviewed**:
- `crates/engine/src/state/types.rs` (KeywordAbility::Enlist)
- `crates/engine/src/state/hash.rs` (discriminants 86, 25; PendingTrigger fields; CombatState.enlist_pairings)
- `crates/engine/src/state/stack.rs` (StackObjectKind::EnlistTrigger)
- `crates/engine/src/state/stubs.rs` (is_enlist_trigger, enlist_enlisted_creature)
- `crates/engine/src/state/combat.rs` (enlist_pairings field + init)
- `crates/engine/src/state/builder.rs` (Enlist placeholder TriggeredAbilityDef)
- `crates/engine/src/rules/command.rs` (DeclareAttackers.enlist_choices)
- `crates/engine/src/rules/engine.rs` (destructuring + pass-through)
- `crates/engine/src/rules/combat.rs` (10-check validation + tap + store)
- `crates/engine/src/rules/abilities.rs` (trigger post-processing + flush)
- `crates/engine/src/rules/resolution.rs` (EnlistTrigger resolution + counter arm)
- `crates/engine/src/testing/replay_harness.rs` (EnlistDeclaration parsing)
- `crates/engine/src/testing/script_schema.rs` (EnlistDeclaration struct)
- `crates/engine/tests/enlist.rs` (8 tests)
- `crates/engine/tests/script_replay.rs` (call site for translate_player_action)
- `crates/engine/src/effects/mod.rs` (PendingTrigger field updates)
- `crates/engine/src/rules/miracle.rs` (PendingTrigger field updates)
- `crates/engine/src/rules/turn_actions.rs` (PendingTrigger field updates)
- `tools/tui/src/play/panels/stack_view.rs` (EnlistTrigger match arm)
- `tools/replay-viewer/src/view_model.rs` (format_keyword + format_stack_kind arms)

## Verdict: clean

The Enlist implementation is thorough and correctly maps CR 702.154 and its subrules.
All 10 validation checks are present and correctly ordered. The trigger wiring follows
established patterns (Myriad-style tag/remove post-processing, Flanking-style
ContinuousEffect at resolution). Hash coverage is complete for all new fields.
The `enlisted_power != 0` guard correctly handles negative power (matching the plan's
recommendation). The 8 tests cover the core positive and negative cases well. Two
minor test gaps (0/negative power, multiple Enlist instances on one creature) and one
defensive-fallback style issue exist, but none rise to MEDIUM severity.

## Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | LOW | `resolution.rs:2038` | **0-power enlisted creature not tested.** Edge case 9 from plan (enlisted creature with 0 or negative power) has no test. |
| 2 | LOW | `tests/enlist.rs` | **CR 702.154d multiple instances not tested.** No test for a creature with two Enlist keywords enlisting two different creatures. |
| 3 | LOW | `abilities.rs:2989` | **Defensive fallback uses source as enlisted creature.** `unwrap_or(trigger.source)` would produce incorrect behavior if hit, but the code path is unreachable in practice. |
| 4 | LOW | `replay_harness.rs:560` | **Enlisted creature resolved without controller filter.** Uses `find_on_battlefield_by_name` (no controller check) instead of `find_on_battlefield` (controller-filtered). Validation in `combat.rs` catches illegal choices, so this is cosmetic. |

### Finding Details

#### Finding 1: 0-power enlisted creature not tested

**Severity**: LOW
**File**: `crates/engine/src/rules/resolution.rs:2038`
**CR Rule**: 702.154a -- "this creature gets +X/+0 until end of turn, where X is the tapped creature's power."
**Issue**: The plan identifies edge case 9 (enlisted creature with 0 or negative power at resolution time). The implementation correctly handles this with `if enlisted_power != 0` (applies both positive and negative modifiers, skips 0). However, no unit test exercises this path. A creature with 0 power is a valid enlist target, and the trigger should resolve with no effect.
**Fix**: Add a test `test_702_154a_enlist_zero_power_creature_no_buff` where P1 enlists a 0/1 creature and verifies the attacker's power is unchanged after resolution.

#### Finding 2: CR 702.154d multiple instances not tested

**Severity**: LOW
**File**: `crates/engine/tests/enlist.rs`
**CR Rule**: 702.154d -- "Multiple instances of enlist on a single creature function independently."
**Issue**: The module header lists "Multiple Enlist instances each tap a different creature (CR 702.154d)" but no test exercises this. Test 7 tests that a creature can only be enlisted once (by different attackers), but does not test a single creature with two Enlist keywords enlisting two different creatures. The validation logic (Check 9 in combat.rs) counts instances correctly, but this path is untested.
**Fix**: Add a test `test_702_154d_multiple_enlist_instances_on_one_creature` where P1 has a creature with two Enlist keywords, enlists two different creatures, and verifies both triggers resolve with separate power bonuses.

#### Finding 3: Defensive fallback uses source as enlisted creature

**Severity**: LOW
**File**: `crates/engine/src/rules/abilities.rs:2989`
**CR Rule**: 702.154a
**Issue**: In `flush_pending_triggers`, if `is_enlist_trigger` is true but `enlist_enlisted_creature` is `None`, the fallback `unwrap_or(trigger.source)` uses the attacker's own ObjectId as the enlisted creature. This would cause the attacker to receive a bonus equal to its own power, which is incorrect. However, this state is unreachable because `abilities.rs:1564-1565` always sets both `is_enlist_trigger = true` and `enlist_enlisted_creature = Some(enlisted_id)` together.
**Fix**: No code change required -- the fallback is defensive. Optionally, add a `debug_assert!(trigger.enlist_enlisted_creature.is_some())` before the unwrap_or to catch future regressions during testing.

#### Finding 4: Enlisted creature resolved without controller filter

**Severity**: LOW
**File**: `crates/engine/src/testing/replay_harness.rs:560`
**CR Rule**: 702.154a -- "you may tap up to one untapped creature **you control**"
**Issue**: The replay harness resolves the enlisted creature's name using `find_on_battlefield_by_name` which does not filter by controller. This means a script could name an opponent's creature and the harness would resolve it to an ObjectId. However, the engine's validation in `combat.rs` Check 3 (line 332-340) correctly rejects any enlisted creature not controlled by the declaring player, so illegal game states cannot result.
**Fix**: Optionally, use `find_on_battlefield(state, player, &edecl.enlisted)` instead of `find_on_battlefield_by_name` for consistency with the attacker resolution. This would catch script authoring errors earlier with a clearer error message.

## CR Coverage Check

| CR Subrule | Implemented? | Tested? | Notes |
|------------|-------------|---------|-------|
| 702.154a (static: optional cost to attack) | Yes | Yes | test_1 (basic), test_2 (no choice), combat.rs validation |
| 702.154a (triggered: +X/+0 until EOT) | Yes | Yes | test_1, test_5, test_8; resolution.rs ContinuousEffect |
| 702.154a (untapped non-attacking creature) | Yes | Yes | test_3 (attacking rejected), combat.rs Check 4+5 |
| 702.154a (haste or no summoning sickness) | Yes | Yes | test_4 (sickness rejected), test_5 (haste bypasses) |
| 702.154a (power at resolution time) | Yes | Partial | test_1 verifies correct final power; no test for power-change-between-tap-and-resolve |
| 702.154b (optional cost per 508.1g) | Yes | Yes | enlist_choices field on DeclareAttackers, #[serde(default)] |
| 702.154b (linked ability per 607.2h) | Yes | Yes | Placeholder trigger + tag/remove post-processing |
| 702.154c (cannot enlist self) | Yes | Yes | test_6, combat.rs Check 10 |
| 702.154d (multiple instances independent) | Yes | No | Validation counts instances (Check 9); no positive test |
| 702.154 (0/negative power) | Yes | No | Resolution handles `!= 0` correctly; no test |
| 702.154 (multiplayer) | Yes | Yes | test_8 (4-player) |
| 508.1g (optional cost timing) | Yes | Yes | enlist_choices in DeclareAttackers command |
| 607.2h (linked triggered ability) | Yes | Yes | Tag/remove in abilities.rs post-processing |

## Previous Findings (re-review only)

N/A -- first review.
