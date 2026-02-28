# Ability Plan: Combat Damage Trigger (WhenDealsCombatDamageToPlayer)

**Generated**: 2026-02-26
**CR**: 603.2 (trigger dispatch), 510.2 (simultaneous combat damage), 510.3a (trigger timing)
**Priority**: P1
**Similar abilities studied**: SelfAttacks (`rules/abilities.rs:452-462`, `testing/replay_harness.rs:446-460`), SelfDies (`rules/abilities.rs:506-545`, `testing/replay_harness.rs:420-437`)

## CR Rule Text

**CR 603.2**: Whenever a game event or game state matches a triggered ability's trigger event,
that ability automatically triggers. The ability doesn't do anything at this point.

**CR 603.2a**: Because they aren't cast or activated, triggered abilities can trigger even when
it isn't legal to cast spells and activate abilities. Effects that preclude abilities from
being activated don't affect them.

**CR 603.2c**: An ability triggers only once each time its trigger event occurs. However, it
can trigger repeatedly if one event contains multiple occurrences.

**CR 603.2g**: An ability triggers only if its trigger event actually occurs. An event that's
prevented or replaced won't trigger anything.

**CR 510.2**: Second, all combat damage that's been assigned is dealt simultaneously. This
turn-based action doesn't use the stack. No player has the chance to cast spells or activate
abilities between the time combat damage is assigned and the time it's dealt.

**CR 510.3a**: Any abilities that triggered on damage being dealt or while state-based actions
are performed afterward are put onto the stack before the active player gets priority; the
order in which they triggered doesn't matter. (See rule 603, "Handling Triggered Abilities.")

**CR 120.2a**: Damage may be dealt as a result of combat. Each attacking and blocking creature
deals combat damage equal to its power during the combat damage step.

## Key Edge Cases

- **CR 603.2g**: If all combat damage from a creature is prevented (protection, prevention
  shield), the trigger should NOT fire because the damage event was prevented (amount = 0).
- **CR 510.3a**: Combat damage triggers fire simultaneously with SBA triggers. Both are
  placed on the stack before the active player gets priority.
- **CR 603.10 (NOT applicable)**: Combat damage triggers do NOT look back in time. The
  creature must still be on the battlefield after combat damage is dealt for its "whenever
  ~ deals combat damage" trigger to fire. (Exception list in 603.10 does not include combat
  damage triggers.)
- **Multiplayer**: An unblocked creature attacking Player C deals combat damage to Player C.
  The trigger fires for the creature's controller. Multiple creatures attacking different
  players can each fire their own trigger independently.
- **First strike + double strike**: A creature with double strike deals combat damage in
  both the first-strike step and the regular combat damage step. Its trigger fires TWICE
  (once per step, per CR 603.2c -- two separate events).
- **Blocked creature with trample**: If a trampling creature deals excess damage to the
  player, that IS combat damage to a player. The trigger fires.
- **Blocked creature without trample**: A blocked creature deals damage to blockers only
  (not the player), so the trigger does NOT fire.
- **0 power**: A creature with 0 or less power assigns no combat damage (CR 510.1a), so
  the trigger does NOT fire.

## Current State (from ability-wip.md)

- [ ] Step 1: Enum variant -- `TriggerCondition::WhenDealsCombatDamageToPlayer` EXISTS at
  `cards/card_definition.rs:490`. `TriggerEvent::SelfDealsCombatDamageToPlayer` DOES NOT
  EXIST in `state/game_object.rs`.
- [ ] Step 2: Rule enforcement -- trigger dispatch for `CombatDamageDealt` event missing
  from `check_triggers` in `rules/abilities.rs`
- [ ] Step 3: Trigger wiring -- `enter_step` in `rules/engine.rs` does NOT call
  `check_triggers` on turn-based action events (the core bug)
- [ ] Step 4: Unit tests
- [ ] Step 5: Card definition
- [ ] Step 6: Game script
- [ ] Step 7: Coverage doc update

## Implementation Steps

### Step 1: Add TriggerEvent::SelfDealsCombatDamageToPlayer

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/game_object.rs`
**Action**: Add a new variant `SelfDealsCombatDamageToPlayer` to the `TriggerEvent` enum
after `SelfDies` (line 130).
**Pattern**: Follow `SelfAttacks` at line 115.
**Text**:
```rust
/// CR 603.2 / CR 510.3a: Triggers when this creature deals combat damage to a player.
/// The creature must still be on the battlefield after damage is dealt (CR 603.10 --
/// combat damage triggers do NOT look back in time).
SelfDealsCombatDamageToPlayer,
```

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/hash.rs`
**Action**: Add hash discriminant `9u8` for `SelfDealsCombatDamageToPlayer` in the
`HashInto for TriggerEvent` impl (line 899, after `SelfDies => 8u8`).
**Text**:
```rust
/// CR 510.3a: Combat damage to player trigger
TriggerEvent::SelfDealsCombatDamageToPlayer => 9u8.hash_into(hasher),
```

**Match arms**: Grep for exhaustive match expressions on `TriggerEvent` -- the only
exhaustive match is the `HashInto` impl. The `check_triggers` function uses
`collect_triggers_for_event` which compares `trigger_def.trigger_on != event_type` (no
exhaustive match). No other exhaustive matches exist.

### Step 2: Trigger Dispatch in check_triggers

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/abilities.rs`
**Action**: Add a `GameEvent::CombatDamageDealt { assignments }` match arm in `check_triggers`
(around line 547, before the `_ => {}` catch-all).
**Pattern**: Follow the `AttackersDeclared` pattern at lines 452-462, but iterate over
assignments that target a player and fire `SelfDealsCombatDamageToPlayer` for each
source creature.
**CR**: CR 510.3a -- abilities that triggered on damage being dealt are put onto the stack
before the active player gets priority.

**Logic**:
```rust
GameEvent::CombatDamageDealt { assignments } => {
    // CR 510.3a / CR 603.2: "Whenever ~ deals combat damage to a player" triggers
    // fire for each creature that dealt > 0 combat damage to a player.
    // CR 603.2g: prevented damage (amount == 0) does not trigger.
    // CR 603.10: NOT a look-back trigger -- creature must be on battlefield.
    for assignment in assignments {
        if assignment.amount == 0 {
            continue; // CR 603.2g: damage was fully prevented
        }
        if matches!(assignment.target, CombatDamageTarget::Player(_)) {
            collect_triggers_for_event(
                state,
                &mut triggers,
                TriggerEvent::SelfDealsCombatDamageToPlayer,
                Some(assignment.source), // Only the dealing creature
                None,
            );
        }
    }
}
```

**Important**: `collect_triggers_for_event` checks `obj.zone == ZoneId::Battlefield`
internally, so a creature that died before triggers are checked (impossible in normal flow
since SBAs haven't run yet at this point) would not fire. This is correct behavior per
CR 603.10 (no look-back for combat damage triggers).

**Deduplication note**: If a creature with trample deals damage to both a blocker AND a
player (two separate assignments), the trigger should fire once per player-targeting
assignment. With trample, there is exactly one player-targeting assignment per creature,
so no deduplication is needed. If double strike produces two `CombatDamageDealt` events
(one per step), the trigger correctly fires twice (two separate events).

### Step 3: Turn-Based Action Trigger Wiring in enter_step

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/engine.rs`
**Action**: After `execute_turn_based_actions` returns (line 271-272) and before SBA
checking (line 329), add a call to `check_triggers` on the turn-based action events.
**CR**: CR 510.3a -- abilities triggered on damage being dealt are put on the stack before
priority is granted. This call ensures `CombatDamageDealt` events (and any other turn-based
action events) get their triggers checked.

**Location**: Insert between line 272 (`events.extend(action_events);`) and line 274
(`if is_game_over(state) {`).

**Code**:
```rust
// Execute turn-based actions for this step
let action_events = turn_actions::execute_turn_based_actions(state)?;

// CR 510.3a: Check triggers from turn-based actions (e.g., CombatDamageDealt)
// BEFORE SBA checking. This ensures "whenever ~ deals combat damage to a player"
// triggers are queued alongside SBA-generated triggers.
let tba_triggers = abilities::check_triggers(state, &action_events);
for t in tba_triggers {
    state.pending_triggers.push_back(t);
}

events.extend(action_events);
```

**CRITICAL**: The `check_triggers` call MUST happen BEFORE `events.extend(action_events)`
so that the action_events reference is still valid (not moved). Alternatively, clone
`action_events` or call `check_triggers` before extending. The simplest pattern is to
call `check_triggers` on `&action_events` before consuming them into `events`.

**Risk assessment**: This change adds trigger checking to ALL turn-based action events,
not just combat damage. Other turn-based actions that produce events:
- `Step::Untap` -- produces `PermanentUntapped` events (could trigger "whenever a permanent
  becomes untapped" if such triggers existed; currently none are implemented)
- `Step::Draw` -- produces `CardDrawn` events (already handled by other systems)
- `Step::BeginningOfCombat` -- produces `CombatBegan` (no triggers currently)
- `Step::EndOfCombat` -- produces `CombatEnded` (no triggers currently)
- `Step::Cleanup` -- produces cleanup events (already has its own SBA+trigger loop)

None of these currently match any `check_triggers` arms that would cause regressions.
The change is safe.

### Step 4: Harness Enrichment

**File**: `/home/airbaggie/scutemob/crates/engine/src/testing/replay_harness.rs`
**Action**: Add a new enrichment block in `enrich_spec_from_def` that converts
`TriggerCondition::WhenDealsCombatDamageToPlayer` to
`TriggerEvent::SelfDealsCombatDamageToPlayer`. Insert after the `WhenBlocks` block
(after line 479).
**Pattern**: Follow the `WhenDies` block at lines 423-437 and the `WhenAttacks` block at
lines 446-460.

**Code**:
```rust
// CR 510.3a / CR 603.2: Convert "Whenever ~ deals combat damage to a player"
// card-definition triggers into runtime TriggeredAbilityDef entries so
// check_triggers can dispatch them via CombatDamageDealt events.
for ability in &def.abilities {
    if let AbilityDefinition::Triggered {
        trigger_condition: TriggerCondition::WhenDealsCombatDamageToPlayer,
        effect,
        intervening_if,
    } = ability
    {
        spec = spec.with_triggered_ability(TriggeredAbilityDef {
            trigger_on: TriggerEvent::SelfDealsCombatDamageToPlayer,
            intervening_if: intervening_if.clone(),
            description: "Whenever ~ deals combat damage to a player (CR 510.3a)".to_string(),
            effect: Some(effect.clone()),
        });
    }
}
```

**Note**: Unlike the `WhenDies` and `WhenAttacks` blocks, this block also propagates the
`intervening_if` clause from the card definition. The existing blocks use `None` for
`intervening_if`, which is an existing minor gap (they should also propagate it). For
this new block, propagate it correctly from the start.

### Step 5: Unit Tests

**File**: `/home/airbaggie/scutemob/crates/engine/tests/combat.rs`
**Tests to write** (add after the existing `test_603_self_attacks_trigger_fires` test
around line 805):

#### Test 1: `test_510_3a_combat_damage_trigger_fires_on_unblocked_attacker`
- **What it tests**: An unblocked creature with a `SelfDealsCombatDamageToPlayer` triggered
  ability attacks a player, deals combat damage, and the trigger fires.
- **Setup**: P1 has a 2/2 creature with `TriggerEvent::SelfDealsCombatDamageToPlayer`
  triggered ability. P2 has no blockers. Declare attackers targeting P2, advance through
  combat damage step.
- **Assertions**: `AbilityTriggered` event emitted for the creature. Stack has one object
  (the triggered ability). P2's life total decreased by 2.
- **CR**: CR 510.3a, CR 603.2

#### Test 2: `test_510_3a_combat_damage_trigger_does_not_fire_on_blocked_creature`
- **What it tests**: A blocked creature (without trample) deals damage to its blocker,
  not the player. The trigger should NOT fire.
- **Setup**: P1 has a 2/2 with the trigger. P2 has a 3/3 blocker. Declare attackers,
  declare blockers. Advance through combat damage step.
- **Assertions**: No `AbilityTriggered` event for the creature. P2's life total unchanged.
- **CR**: CR 510.1c (blocked creature damages blockers only)

#### Test 3: `test_510_3a_combat_damage_trigger_does_not_fire_when_damage_prevented`
- **What it tests**: If combat damage is fully prevented (e.g., damage = 0 after
  prevention), the trigger does not fire (CR 603.2g).
- **Setup**: P1 has a 0/2 creature (0 power) with the trigger. P2 has no blockers.
  Advance through combat damage.
- **Assertions**: No `AbilityTriggered` event. P2's life total unchanged.
- **CR**: CR 510.1a (0 power = no damage), CR 603.2g

#### Test 4: `test_510_3a_combat_damage_trigger_multiplayer_separate_targets`
- **What it tests**: In multiplayer, two creatures attacking different players each fire
  their triggers independently.
- **Setup**: P1 has creature A (attacks P2) and creature B (attacks P3), both with the
  trigger. Advance through combat damage.
- **Assertions**: Two `AbilityTriggered` events (one per creature). Two items on the stack.
  P2 and P3 both took damage.
- **CR**: CR 510.3a, CR 603.2c

**Pattern**: Follow `test_603_self_attacks_trigger_fires` at line 750 of
`/home/airbaggie/scutemob/crates/engine/tests/combat.rs`.

### Step 6: Card Definition

**Suggested card**: Scroll Thief
- **Oracle text**: "Whenever this creature deals combat damage to a player, draw a card."
- **Type**: Creature -- Merfolk Rogue, 1/3, {2}{U}
- **Why**: Simplest possible "deals combat damage to player" trigger with a concrete
  effect (DrawCards). No optional clause, no "that many" variable. Perfect for validating
  the trigger dispatch end-to-end.

The card definition should use:
```rust
AbilityDefinition::Triggered {
    trigger_condition: TriggerCondition::WhenDealsCombatDamageToPlayer,
    effect: Effect::DrawCards {
        target: EffectTarget::Controller,
        count: EffectAmount::Static(1),
    },
    intervening_if: None,
}
```

**Agent**: Use `card-definition-author` agent for Scroll Thief.

### Step 7: Game Script

**Suggested scenario**: Scroll Thief deals combat damage to an opponent, draws a card.
- P1 controls Scroll Thief (1/3 creature, {2}{U}).
- P2 has no blockers.
- P1 declares Scroll Thief attacking P2.
- P2 passes (no blockers).
- Combat damage step: Scroll Thief deals 1 damage to P2.
- Trigger fires: P1 draws a card.
- Assert: P2 life = 39. P1 hand increased by 1. Stack has triggered ability.

**Subsystem directory**: `test-data/generated-scripts/combat/`
**Agent**: Use `game-script-generator` agent.

## Interactions to Watch

- **SBAs after combat damage**: Creatures that receive lethal damage from combat die as
  SBAs. Their "when ~ dies" triggers fire in the same SBA check pass. The combat damage
  trigger should fire BEFORE SBAs (from the turn-based action events), and dies triggers
  fire DURING SBAs. Both end up in `pending_triggers` and are flushed together before
  priority is granted. This is correct per CR 510.3a.

- **Lifelink + combat damage trigger**: Lifelink gains are applied inside
  `apply_combat_damage` (lines 982-1004 of `combat.rs`). The trigger fires from the
  `CombatDamageDealt` event, which is emitted at line 991. Lifelink is resolved before
  triggers are checked. No interaction issue.

- **Commander damage tracking**: Commander damage is tracked inside `apply_combat_damage`
  (lines 943-969). This happens before the `CombatDamageDealt` event is emitted. No
  interaction issue with the trigger.

- **First strike / double strike**: First strike damage happens in `Step::FirstStrikeDamage`
  (a separate call to `apply_combat_damage`). The trigger wiring in `enter_step` handles
  this automatically because `execute_turn_based_actions` is called for each step. A
  double-strike creature fires the trigger twice (once per step). Correct per CR 603.2c.

- **Trample**: A trampling creature that deals excess damage to the player after lethal to
  the blocker generates a `CombatDamageAssignment` with `target: CombatDamageTarget::Player`.
  The trigger fires correctly for that assignment.

## Files Modified (Summary)

| File | Change |
|------|--------|
| `crates/engine/src/state/game_object.rs` | Add `TriggerEvent::SelfDealsCombatDamageToPlayer` variant |
| `crates/engine/src/state/hash.rs` | Add hash discriminant 9u8 for new variant |
| `crates/engine/src/rules/abilities.rs` | Add `CombatDamageDealt` match arm in `check_triggers` |
| `crates/engine/src/rules/engine.rs` | Add `check_triggers` call on turn-based action events in `enter_step` |
| `crates/engine/src/testing/replay_harness.rs` | Add enrichment block for `WhenDealsCombatDamageToPlayer` |
| `crates/engine/tests/combat.rs` | Add 4 unit tests |
| `crates/engine/src/cards/definitions.rs` | Scroll Thief card definition (Step 6) |
| `docs/mtg-engine-ability-coverage.md` | Update status to `validated` (Step 7) |
