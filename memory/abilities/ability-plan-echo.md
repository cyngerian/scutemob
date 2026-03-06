# Ability Plan: Echo

**Generated**: 2026-03-06
**CR**: 702.30
**Priority**: P4
**Similar abilities studied**: Fading (CR 702.32) -- upkeep trigger + sacrifice pattern in `rules/turn_actions.rs:L168`, `rules/abilities.rs:L3803`, `rules/resolution.rs:L1215`; Dredge (CR 702.52) -- player choice command pattern in `rules/command.rs:L396`, `rules/events.rs:L745`

## CR Rule Text

702.30. Echo

702.30a Echo is a triggered ability. "Echo [cost]" means "At the beginning of your upkeep, if this permanent came under your control since the beginning of your last upkeep, sacrifice it unless you pay [cost]."

702.30b Urza block cards with the echo ability were printed without an echo cost. These cards have been given errata in the Oracle card reference; each one now has an echo cost equal to its mana cost.

**Note**: The user prompt cited CR 702.31, but that is Horsemanship (see gotchas-rules.md #36). MCP lookup confirms Echo is 702.30.

## Key Edge Cases

- **Payment is optional** (Karmic Guide ruling 2021-06-18): "Paying for echo is always optional. When the echo triggered ability resolves, if you can't pay the echo cost or choose not to, you sacrifice that permanent." This means player choice is required at resolution -- cannot auto-pay.
- **Trigger condition**: "came under your control since the beginning of your last upkeep" -- fires on the first upkeep after the permanent enters or the player gains control. The `echo_pending` flag (set at ETB, cleared after trigger resolution) models this correctly.
- **Control change**: If a player steals an Echo creature, the flag should be set (since it "came under their control" since their last upkeep). However, the `echo_pending` flag from the original ETB only tracks initial entry. For stolen creatures, we would need to set the flag on control change too. For initial implementation, the ETB-only flag is sufficient for the most common case (creatures you cast). Control-change tracking can be deferred as an edge case.
- **Echo cost can differ from mana cost** (CR 702.30b): Most Urza block cards have echo cost = mana cost, but later cards have different echo costs (e.g., Karmic Guide's {3}{W}{W}). The `KeywordAbility::Echo(ManaCost)` variant carries the cost.
- **Multiple instances**: Each instance of Echo triggers separately (standard for triggered abilities). Unusual but theoretically possible.
- **Sacrifice bypasses indestructible** (CR 701.17a): Same as Fading.
- **Permanent left battlefield**: If the permanent is no longer on the battlefield when the trigger resolves, do nothing (CR 400.7).
- **Multiplayer**: "At the beginning of YOUR upkeep" -- only fires for the current controller's upkeep, not all players.
- **Trigger can be Stifled**: Echo is a triggered ability on the stack. Countering it means the player neither pays nor sacrifices -- the permanent stays. But `echo_pending` should be cleared (the trigger fired and was handled). Actually, if Stifled, the flag should remain set so it triggers again next upkeep. This is a subtlety: the flag should only clear when the trigger RESOLVES (pay or sacrifice), not when it's countered. Implementation: clear the flag inside the resolution handler, not when the trigger is put on the stack.

## Current State (from ability-wip.md)

- [ ] Step 1: Enum variant
- [ ] Step 2: Rule enforcement
- [ ] Step 3: Trigger wiring
- [ ] Step 4: Unit tests
- [ ] Step 5: Card definition
- [ ] Step 6: Game script

## Implementation Steps

### Step 1: Enum Variant

**File**: `crates/engine/src/state/types.rs`
**Action**: Add `KeywordAbility::Echo(ManaCost)` variant after `Fading(u32)` (around line 1026).

```
/// CR 702.30a: Echo [cost] -- "At the beginning of your upkeep, if this permanent
/// came under your control since the beginning of your last upkeep, sacrifice it
/// unless you pay [cost]."
///
/// The ManaCost parameter is the echo cost. For most Urza block cards, this equals
/// the card's mana cost (CR 702.30b). Later cards may have different echo costs.
///
/// CR 702.30a: Each instance triggers separately (standard triggered ability rule).
Echo(ManaCost),
```

**Hash**: Add to `state/hash.rs` -- KeywordAbility discriminant **114** (next after Fading=113).
```
// Echo (discriminant 114) -- CR 702.30
KeywordAbility::Echo(cost) => {
    114u8.hash_into(hasher);
    cost.hash_into(hasher);
}
```

**Match arms**: Grep for all `KeywordAbility` exhaustive match expressions and add `Echo(..)` arm. Key locations:
- `state/hash.rs` (HashInto impl)
- `cards/builder.rs` (if any match on keywords)
- `rules/layers.rs` (calculate_characteristics -- likely no special handling needed, just pass-through)

### Step 2: AbilityDefinition Variant

**File**: `crates/engine/src/cards/card_definition.rs`
**Action**: Add `AbilityDefinition::Echo { cost: ManaCost }` variant after `Fading { count: u32 }` (around line 480).

```
/// CR 702.30a: Echo [cost] -- triggered ability that fires on the controller's
/// first upkeep after the permanent enters.
///
/// Cards with this ability should also include
/// `AbilityDefinition::Keyword(KeywordAbility::Echo(cost))` for quick
/// presence-checking without scanning all abilities.
///
/// `cost` is the echo cost (ManaCost). For Urza block cards, this equals
/// the card's mana cost (CR 702.30b).
Echo { cost: ManaCost },
```

**Hash**: Add to `state/hash.rs` -- AbilityDefinition discriminant **43** (next after Fading=42).
```
// Echo (discriminant 43) -- CR 702.30
AbilityDefinition::Echo { cost } => {
    43u8.hash_into(hasher);
    cost.hash_into(hasher);
}
```

### Step 3: echo_pending Flag on GameObject

**File**: `crates/engine/src/state/game_object.rs`
**Action**: Add `echo_pending: bool` field to `GameObject` struct (after `decayed_sacrifice_at_eoc` around line 421).

```
/// CR 702.30a: If true, this permanent has Echo and has not yet had its echo
/// trigger resolve. Set to true when the permanent enters the battlefield with
/// the Echo keyword. Cleared when the echo trigger resolves (either paid or
/// sacrificed). If the trigger is countered (e.g., Stifle), the flag remains
/// set so the trigger fires again on the next upkeep.
///
/// This models the "came under your control since the beginning of your last
/// upkeep" condition from CR 702.30a.
#[serde(default)]
pub echo_pending: bool,
```

**Initialization sites** (set `echo_pending: false`):
- `state/builder.rs` -- in GameStateBuilder object creation
- `effects/mod.rs` -- in token creation
- `rules/resolution.rs` -- in permanent creation from spell resolution

**Hash**: Add to `state/hash.rs` in the GameObject HashInto impl.

**ETB sites** (set `echo_pending: true` for permanents with Echo keyword):
- `rules/resolution.rs` -- in the ETB block where Vanishing/Fading counters are placed (around line 448-475). After the Fading counter placement, add Echo flag setting.
- `rules/lands.rs` -- in the ETB block where Vanishing/Fading counters are placed (around line 148-177). Same pattern.

Pattern (both sites):
```
// CR 702.30a: Mark permanents with Echo as pending their echo trigger.
if obj.characteristics.keywords.iter().any(|kw| matches!(kw, KeywordAbility::Echo(_))) {
    obj.echo_pending = true;
}
```

### Step 4: PendingTriggerKind and StackObjectKind

**File**: `crates/engine/src/state/stubs.rs`
**Action**: Add `PendingTriggerKind::EchoUpkeep` variant (before the "Add new trigger kinds" comment, around line 80).

```
/// CR 702.30a: Echo upkeep trigger -- sacrifice unless you pay [cost].
EchoUpkeep,
```

**File**: `crates/engine/src/state/stack.rs`
**Action**: Add `StackObjectKind::EchoTrigger` variant (after FadingTrigger, around line 911).

```
/// CR 702.30a: "At the beginning of your upkeep, if this permanent came under
/// your control since the beginning of your last upkeep, sacrifice it unless
/// you pay [cost]." Discriminant 40.
EchoTrigger {
    source_object: ObjectId,
    echo_permanent: ObjectId,
    echo_cost: ManaCost,
},
```

**Hash**: Add to `state/hash.rs` -- StackObjectKind discriminant **40** (next after FadingTrigger=39).
```
// EchoTrigger (discriminant 40) -- CR 702.30a
StackObjectKind::EchoTrigger {
    source_object,
    echo_permanent,
    echo_cost,
} => {
    40u8.hash_into(hasher);
    source_object.hash_into(hasher);
    echo_permanent.hash_into(hasher);
    echo_cost.hash_into(hasher);
}
```

**TUI**: Add arm in `tools/tui/src/play/panels/stack_view.rs` for `StackObjectKind::EchoTrigger { .. }`.

### Step 5: Upkeep Trigger Queueing

**File**: `crates/engine/src/rules/turn_actions.rs`
**Action**: Add Echo trigger queueing in `upkeep_actions()` after the Fading block (around line 230+). Follow the Fading pattern exactly.

```
// CR 702.30a: Queue upkeep triggers for all Echo permanents with echo_pending.
// "At the beginning of your upkeep, if this permanent came under your control
// since the beginning of your last upkeep, sacrifice it unless you pay [cost]."
//
// Only fires for permanents controlled by the active player (CR 702.30a: "your upkeep").
// Intervening-if: echo_pending must be true (models "came under your control since
// the beginning of your last upkeep").
// Multiple instances of Echo each trigger separately.
let echo_permanents: Vec<(ObjectId, Vec<ManaCost>)> = state
    .objects
    .values()
    .filter(|obj| {
        obj.zone == ZoneId::Battlefield
            && obj.controller == active
            && obj.echo_pending
    })
    .map(|obj| {
        let echo_costs: Vec<ManaCost> = obj
            .characteristics
            .keywords
            .iter()
            .filter_map(|kw| {
                if let KeywordAbility::Echo(cost) = kw {
                    Some(cost.clone())
                } else {
                    None
                }
            })
            .collect();
        (obj.id, echo_costs)
    })
    .filter(|(_, costs)| !costs.is_empty())
    .collect();

for (obj_id, costs) in echo_permanents {
    for cost in costs {
        state.pending_triggers.push_back(PendingTrigger {
            source: obj_id,
            ability_index: 0,
            controller: active,
            kind: PendingTriggerKind::EchoUpkeep,
            // ... all other fields None (follow Fading pattern)
            echo_cost: Some(cost),
        });
    }
}
```

**Note**: `PendingTrigger` needs a new field `echo_cost: Option<ManaCost>` to carry the cost through to stack object creation. Add this field to the struct in `stubs.rs`.

### Step 6: Trigger-to-Stack Conversion

**File**: `crates/engine/src/rules/abilities.rs`
**Action**: Add `PendingTriggerKind::EchoUpkeep` arm in the trigger-to-stack conversion match (after `FadingUpkeep`, around line 3810).

```
PendingTriggerKind::EchoUpkeep => {
    // CR 702.30a: Echo upkeep trigger.
    // "At the beginning of your upkeep, if this permanent came under your
    // control since the beginning of your last upkeep, sacrifice it unless
    // you pay [cost]."
    StackObjectKind::EchoTrigger {
        source_object: trigger.source,
        echo_permanent: trigger.source,
        echo_cost: trigger.echo_cost.clone().unwrap_or_default(),
    }
}
```

### Step 7: Player Choice -- Command and Event

**File**: `crates/engine/src/rules/command.rs`
**Action**: Add `Command::PayEcho` variant.

```
/// Choose whether to pay the echo cost for a permanent (CR 702.30a).
///
/// Sent in response to an `EchoPaymentRequired` event. If `pay` is true,
/// the player pays the echo cost (mana is deducted). If `pay` is false
/// (or the player cannot afford the cost), the permanent is sacrificed.
PayEcho {
    player: PlayerId,
    permanent: ObjectId,
    pay: bool,
},
```

**File**: `crates/engine/src/rules/events.rs`
**Action**: Add `GameEvent::EchoPaymentRequired` variant.

```
/// CR 702.30a: An echo trigger has resolved. The controller must choose
/// whether to pay the echo cost or sacrifice the permanent.
///
/// The engine pauses until a `Command::PayEcho` is received.
EchoPaymentRequired {
    player: PlayerId,
    permanent: ObjectId,
    cost: ManaCost,
},
```

**Hash**: Add to `state/hash.rs` -- GameEvent hash for `EchoPaymentRequired`.

### Step 8: Trigger Resolution

**File**: `crates/engine/src/rules/resolution.rs`
**Action**: Add `StackObjectKind::EchoTrigger` resolution handler (after `FadingTrigger` handler, around line 1290+).

The resolution handler should:
1. Check if permanent is still on the battlefield (CR 400.7). If not, do nothing.
2. Emit `EchoPaymentRequired` event with the echo cost.
3. Set `state.pending_echo_payment` (or a similar mechanism) to pause the game.

**Alternative approach** (simpler, matching Fading's self-contained resolution): Resolve the trigger entirely at resolution time. Since the engine already has `can_pay_cost`, the resolution handler can:
1. Check if permanent is on the battlefield.
2. If not, do nothing.
3. If yes, emit `EchoPaymentRequired` event.
4. The engine pauses for player command (like DredgeChoiceRequired).

**File**: `crates/engine/src/rules/engine.rs`
**Action**: Add `Command::PayEcho` handler in `process_command()`. Pattern: follow `ChooseDredge`.

```
Command::PayEcho { player, permanent, pay } => {
    // Validate: permanent must be on the battlefield, controlled by player.
    // Validate: if pay=true, player must be able to afford the echo cost.
    if pay {
        // Deduct mana via pay_cost().
        // Clear echo_pending on the permanent.
    } else {
        // Sacrifice the permanent (zone change to graveyard).
        // Check replacement effects (commander redirect, etc.).
        // Clear echo_pending on the permanent.
    }
}
```

Also add `EchoTrigger` to the "counter abilities" catch-all arm in `resolution.rs` (around line 3879-3895).

### Step 9: Pending Choice Tracking

**File**: `crates/engine/src/state/mod.rs` (GameState)
**Action**: Add a field to track pending echo payment choice, similar to `pending_commander_returns` or `pending_dredge_choice`.

```
/// CR 702.30a: Pending echo payment choices. When an echo trigger resolves,
/// the controller must choose to pay or sacrifice. The game pauses until
/// a `Command::PayEcho` is received for each entry.
#[serde(default)]
pub pending_echo_payments: Vector<(PlayerId, ObjectId, ManaCost)>,
```

Or simpler: use a single `Option` if only one can be pending at a time (echo triggers resolve one at a time from the stack).

**Hash**: Add to `state/hash.rs` in the GameState HashInto impl.

### Step 10: Unit Tests

**File**: `crates/engine/tests/echo.rs`
**Tests to write**:
- `test_echo_basic_etb_sets_pending` -- CR 702.30a: permanent enters battlefield with Echo, `echo_pending` is true
- `test_echo_upkeep_trigger_fires` -- CR 702.30a: advancing to upkeep queues the echo trigger for a pending Echo permanent
- `test_echo_pay_cost_keeps_permanent` -- CR 702.30a: paying the echo cost keeps the permanent, clears echo_pending
- `test_echo_decline_payment_sacrifices` -- CR 702.30a: declining payment sacrifices the permanent
- `test_echo_no_trigger_after_paid` -- CR 702.30a: after paying echo, the next upkeep does NOT trigger echo again (echo_pending is false)
- `test_echo_different_cost` -- CR 702.30b: echo cost can differ from mana cost (e.g., Karmic Guide)
- `test_echo_permanent_left_battlefield` -- CR 400.7: if permanent left battlefield before trigger resolves, do nothing
- `test_echo_multiplayer_only_controller_upkeep` -- CR 702.30a: echo only triggers on the controller's upkeep, not other players'

**Pattern**: Follow tests in `crates/engine/tests/fading.rs` -- same setup pattern (start at Untap, pass to Upkeep), same helper functions.

### Step 11: Card Definition (later phase)

**Suggested card**: Avalanche Riders ({3}{R}, Creature -- Human Nomad, 2/2, Haste, Echo {3}{R}, ETB destroy target land)
- Simpler alternative: Goblin War Buggy ({1}{R}, Creature -- Goblin, 2/2, Haste, Echo {1}{R}) -- no ETB, pure vanilla+echo+haste
- Complex alternative: Karmic Guide ({3}{W}{W}, Creature -- Angel Spirit, 2/2, Flying, Protection from black, Echo {3}{W}{W}, ETB return creature from graveyard)

**Card lookup**: use `card-definition-author` agent

### Step 12: Game Script (later phase)

**Suggested scenario**: Player casts Goblin War Buggy, passes to next upkeep, is prompted to pay echo or sacrifice. Two variants: one where player pays, one where player declines.
**Subsystem directory**: `test-data/generated-scripts/stack/`

## Interactions to Watch

- **Commander redirect**: If the echo permanent is a commander and the player declines to pay, the sacrifice triggers the commander zone return SBA (CR 903.9a). The resolution handler must use `check_zone_change_replacement` like Fading does.
- **Stifle/counter**: If the echo trigger is countered, `echo_pending` should remain true so the trigger fires again next upkeep. This is handled by only clearing the flag in the resolution handler, not in the counter handler.
- **Humility**: If Humility removes all abilities (including Echo), the upkeep trigger scanning in `turn_actions.rs` checks `echo_pending` AND `KeywordAbility::Echo(_)` in the resolved characteristics. If Echo is removed by Humility, the trigger should not fire (the permanent no longer has the ability). But `echo_pending` is a flag, not an ability check. The scan must check BOTH: `echo_pending == true` AND the permanent currently has `KeywordAbility::Echo(_)` in its layer-resolved characteristics. This is important.
- **Flickering**: If the permanent is flickered (exiled and returned), it gets a new ObjectId (CR 400.7). The new object should have `echo_pending: true` set again at its new ETB (since it re-enters with Echo). The old pending trigger on the stack references a dead ObjectId and does nothing.
- **Panharmonicon**: Echo is a triggered ability, but it triggers at beginning of upkeep, not on ETB. Panharmonicon ("Whenever a nontoken permanent enters the battlefield") does not double echo triggers.

## Discriminant Summary

| Type | Variant | Discriminant |
|------|---------|-------------|
| KeywordAbility | Echo(ManaCost) | 114 |
| AbilityDefinition | Echo { cost: ManaCost } | 43 |
| StackObjectKind | EchoTrigger | 40 |
| PendingTriggerKind | EchoUpkeep | (enum, no disc needed) |
| GameEvent | EchoPaymentRequired | (next available in hash.rs) |
| Command | PayEcho | (enum, no disc needed) |
