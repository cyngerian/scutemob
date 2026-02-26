# Ability Review: Combat Damage Trigger (SelfDealsCombatDamageToPlayer)

**Date**: 2026-02-26
**Reviewer**: ability-impl-reviewer (Opus)
**CR**: 603.2, 510.2, 510.3a, 603.10, 120.2a
**Files reviewed**:
- `crates/engine/src/state/game_object.rs` (TriggerEvent enum, line 131-135)
- `crates/engine/src/state/hash.rs` (HashInto for TriggerEvent, lines 900-901; HashInto for TriggerCondition, line 1847)
- `crates/engine/src/rules/abilities.rs` (CombatDamageDealt match arm, lines 547-567; collect_triggers_for_event, lines 584-633)
- `crates/engine/src/rules/engine.rs` (enter_step TBA trigger wiring, lines 273-280)
- `crates/engine/src/testing/replay_harness.rs` (WhenDealsCombatDamageToPlayer enrichment, lines 481-500)
- `crates/engine/tests/combat.rs` (4 new tests, lines 1537-1935)
- `crates/engine/src/rules/combat.rs` (apply_combat_damage, lines 660-1007 -- read-only context)
- `crates/engine/src/rules/turn_actions.rs` (execute_turn_based_actions, lines 15-26 -- read-only context)
- `crates/engine/src/rules/events.rs` (CombatDamageDealt event, lines 321-323; CombatDamageAssignment, lines 28-36)
- `crates/engine/src/cards/card_definition.rs` (TriggerCondition::WhenDealsCombatDamageToPlayer, line 490)
- `crates/engine/src/cards/definitions.rs` (Alela usage, lines 1619-1625)

## Verdict: clean

The implementation is correct and well-structured. The trigger dispatch in `check_triggers` accurately matches the CR rules: it filters for player-targeted assignments with amount > 0, uses `collect_triggers_for_event` which enforces the battlefield-presence check (CR 603.10), and is wired into the `enter_step` flow at exactly the right point (after TBA execution, before SBA checking). The hash discriminant is assigned correctly (9u8). All four tests are well-designed, cover the primary positive case, two negative cases, and multiplayer correctness. No HIGH or MEDIUM findings. Two LOW findings are noted below.

## Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | LOW | `replay_harness.rs:495` | **intervening_if not propagated from card def.** Hardcoded `None` instead of converting `Condition` to `InterveningIf`. |
| 2 | LOW | `tests/combat.rs:1537-1935` | **Missing trample + combat damage trigger test.** Trample edge case documented in plan but not covered by unit tests. |

### Finding Details

#### Finding 1: intervening_if not propagated from card definition

**Severity**: LOW
**File**: `crates/engine/src/testing/replay_harness.rs:495`
**CR Rule**: CR 603.4 -- "A triggered ability may read 'When/Whenever/At [trigger event], if [condition], [effect].' [...] The ability triggers only if [condition] is also true."
**Issue**: The enrichment block for `WhenDealsCombatDamageToPlayer` at line 495 hardcodes `intervening_if: None` rather than attempting to convert the card definition's `intervening_if: Option<Condition>` to `InterveningIf`. The implementation comment at lines 484-485 explains this is intentional: `Condition` and `InterveningIf` are separate type hierarchies and no conversion currently exists. This is consistent with the existing `WhenDies` (line 432), `WhenAttacks` (line 455), and `WhenBlocks` (line 475) enrichment blocks which also hardcode `None`. No cards in the card definitions currently use conditional combat damage triggers (Alela at `definitions.rs:1624` uses `intervening_if: None`).
**Fix**: Deferred. When a card with a conditional combat damage trigger is added, implement `Condition -> InterveningIf` conversion at this site and at the other three enrichment blocks (WhenDies, WhenAttacks, WhenBlocks). This is tracked as a known gap across all trigger enrichment paths.

#### Finding 2: Missing trample + combat damage trigger test

**Severity**: LOW
**File**: `crates/engine/tests/combat.rs:1537-1935`
**CR Rule**: CR 702.19b -- "If an attacking creature with trample is blocked, [...] its controller assigns damage to the blocking creature [...] and the remaining damage is assigned as its controller chooses among those combats' defending players and/or planeswalkers."
**Issue**: The plan documented the trample edge case: "If a trampling creature deals excess damage to the player after lethal to the blocker, the trigger fires." The implementation handles this correctly (the `apply_combat_damage` code generates a separate `CombatDamageTarget::Player` assignment for the trample-through damage, which `check_triggers` matches). However, no unit test verifies this path. The existing tests cover: unblocked (positive), fully blocked without trample (negative), zero power (negative), and multiplayer (positive). A trample test would strengthen confidence that the trigger fires exactly once (not once per assignment -- only the player-targeting assignment triggers).
**Fix**: Add a test `test_510_3a_combat_damage_trigger_fires_with_trample` with setup: P1 has a 5/5 trample creature with the trigger, P2 has a 2/2 blocker. After combat damage, verify: trigger fires exactly once, P2 takes 3 damage (5 - 2 lethal to blocker), one triggered ability on the stack.

## CR Coverage Check

| CR Subrule | Implemented? | Tested? | Notes |
|------------|-------------|---------|-------|
| CR 510.1a (0 power = no damage) | Yes | Yes | `test_510_3a_..._when_damage_is_zero` |
| CR 510.1b (unblocked creature damages player) | Yes | Yes | `test_510_3a_..._fires_on_unblocked_attacker` |
| CR 510.1c (blocked creature damages blockers only) | Yes | Yes | `test_510_3a_..._does_not_fire_on_blocked_creature` |
| CR 510.2 (simultaneous damage) | Yes | Yes | Implicit in all combat damage tests |
| CR 510.3a (triggers on damage placed on stack before priority) | Yes | Yes | All four tests verify stack state |
| CR 510.4 (first strike / double strike two-step) | Yes (structural) | No | Engine calls `apply_combat_damage` per step via `enter_step`; no explicit test for double-strike + trigger |
| CR 603.2 (trigger event matching) | Yes | Yes | Core dispatch logic |
| CR 603.2c (one trigger per event occurrence) | Yes | Yes | `test_510_3a_..._multiplayer_separate_targets` |
| CR 603.2g (prevented/zero damage no trigger) | Yes | Yes | `test_510_3a_..._when_damage_is_zero` |
| CR 603.4 (intervening-if) | Partial | No | `collect_triggers_for_event` checks it, but enrichment always sets `None` |
| CR 603.10 (NOT a look-back trigger) | Yes | No | `collect_triggers_for_event` checks `obj.zone == Battlefield`; no test where creature leaves BF before trigger check |
| CR 702.19b (trample excess to player) | Yes (structural) | No | `apply_combat_damage` generates `Player` assignment; no dedicated trigger test |
| CR 120.2a (combat damage = power) | Yes | Yes | Implicit via power-based damage assertions |

## Hash Coverage Check

| New Field/Variant | Hash Discriminant | Verified? |
|-------------------|-------------------|-----------|
| `TriggerEvent::SelfDealsCombatDamageToPlayer` | 9u8 | Yes -- `hash.rs:901` |
| `TriggerCondition::WhenDealsCombatDamageToPlayer` | 4u8 | Yes -- `hash.rs:1847` (pre-existing) |

## Structural Correctness Summary

1. **Trigger ordering (CR 510.3a)**: Correct. In `enter_step` (engine.rs:273-280), `check_triggers` is called on `action_events` (which includes `CombatDamageDealt`) BEFORE SBA checking (engine.rs:336-340). The resulting `PendingTrigger` entries are pushed into `state.pending_triggers`. After SBAs run (which may add more triggers, e.g., CreatureDied), all pending triggers are flushed together via `flush_pending_triggers` (engine.rs:349) in APNAP order before priority is granted. This matches CR 510.3a exactly: "abilities that triggered on damage being dealt or while state-based actions are performed afterward are put onto the stack before the active player gets priority."

2. **Not a look-back trigger (CR 603.10)**: Correct. The `collect_triggers_for_event` function (abilities.rs:606) checks `obj.zone != ZoneId::Battlefield` and skips objects not on the battlefield. Combat damage triggers are not in the CR 603.10 exception list (only zone-change, phase-out, unattach, control-change, counter, loss, and planeswalk triggers look back). A creature that somehow left the battlefield before trigger checking would not fire.

3. **Zero/prevented damage (CR 603.2g)**: Correct. The `apply_combat_damage` function builds `final_assignments` with post-prevention amounts (combat.rs:903-911). These can include `amount: 0` for fully prevented damage. The `check_triggers` code at abilities.rs:554 explicitly checks `assignment.amount == 0` and continues, preventing the trigger from firing on prevented damage.

4. **Trample deduplication**: Not an issue. A trampling creature blocked by one creature generates at most one `CombatDamageTarget::Player` assignment (combat.rs:776-783). Even with multiple blockers, the trample excess is consolidated into a single player assignment. No deduplication is needed.

5. **First strike / double strike**: Structurally correct. `execute_turn_based_actions` is called for each step (turn_actions.rs:20-21). For `Step::FirstStrikeDamage`, `first_strike_damage_step` calls `apply_combat_damage(state, true)`. For `Step::CombatDamage`, `combat_damage_step` calls `apply_combat_damage(state, false)`. Each produces its own `CombatDamageDealt` event, and `enter_step`'s trigger wiring runs for each step. A double-strike creature would fire the trigger twice (once per step), which is correct per CR 603.2c.

6. **Multiplayer correctness**: Verified. The implementation iterates all assignments in the event, not just one. Multiple creatures attacking different players each get their triggers collected independently. The multiplayer test (test_510_3a_..._multiplayer_separate_targets) confirms this with 3 players.
