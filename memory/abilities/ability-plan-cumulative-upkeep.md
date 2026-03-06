# Ability Plan: Cumulative Upkeep

**Generated**: 2026-03-06
**CR**: 702.24
**Priority**: P4
**Similar abilities studied**: Echo (CR 702.30) -- upkeep trigger + pay-or-sacrifice pattern in `rules/turn_actions.rs:L239`, `rules/resolution.rs:L1357`, `rules/engine.rs:L429` (PayEcho handler at L461); Fading (CR 702.32) -- upkeep counter manipulation in `rules/turn_actions.rs:L170`

## CR Rule Text

702.24. Cumulative Upkeep

702.24a Cumulative upkeep is a triggered ability that imposes an increasing cost on a permanent. "Cumulative upkeep [cost]" means "At the beginning of your upkeep, if this permanent is on the battlefield, put an age counter on this permanent. Then you may pay [cost] for each age counter on it. If you don't, sacrifice it." If [cost] has choices associated with it, each choice is made separately for each age counter, then either the entire set of costs is paid, or none of them is paid. Partial payments aren't allowed.

702.24b If a permanent has multiple instances of cumulative upkeep, each triggers separately. However, the age counters are not connected to any particular ability; each cumulative upkeep ability will count the total number of age counters on the permanent at the time that ability resolves.

## Key Edge Cases

- **Age counter is added FIRST, then payment is required** (CR 702.24a). On the first upkeep: add 1 age counter, then must pay 1x cost. On the second upkeep: add another age counter (now 2 total), then must pay 2x cost. This escalating pattern is the defining feature.
- **Payment is always optional** (ruling 2008-10-01, confirmed across 15+ cards): "Paying cumulative upkeep is always optional. If it's not paid, the permanent with cumulative upkeep is sacrificed."
- **No partial payments** (CR 702.24a): "either the entire set of costs is paid, or none of them is paid." The player pays the full cost for ALL age counters or sacrifices. No "pay for some counters."
- **Multiple instances trigger separately, but share age counters** (CR 702.24b). If a permanent has two instances of cumulative upkeep (e.g., from two enchantments), each trigger fires separately. When each resolves, it counts ALL age counters on the permanent (not just "its" counters). Each trigger also adds one age counter -- so with two instances, the permanent gains 2 age counters per upkeep.
- **Cost varieties**: Most common is mana (Mystic Remora: `{1}`), but costs can also be life (Glacial Chasm: "Pay 2 life"), mana production (Braid of Fire: "Add {R}"), or complex actions (Herald of Leshrac: "Gain control of a land you don't control"). For initial implementation, support `Mana(ManaCost)` and `Life(u32)`. Complex action costs can use `Custom(String)` as a placeholder for future expansion.
- **Sacrifice bypasses indestructible** (CR 701.17a): Same as Echo and Fading.
- **Permanent left battlefield**: If the permanent is no longer on the battlefield when the trigger resolves, do nothing (CR 400.7).
- **Multiplayer**: "At the beginning of YOUR upkeep" -- only fires for the current controller's upkeep.
- **Trigger can be Stifled/countered**: Cumulative upkeep is a triggered ability on the stack. If countered, the age counter was never added (the counter addition is part of the trigger resolution, not the trigger being put on the stack), so the next upkeep will try again with the same counter count.
- **Proliferate interaction**: Proliferate can add extra age counters, increasing the cost. This works naturally since the trigger counts all age counters at resolution time.
- **Counter removal**: Effects that remove counters (e.g., Vampire Hexmage removing all counters) reduce the cost. The trigger still fires but the cost is 0x if 0 age counters remain after the add step... wait, no -- the trigger adds an age counter first, so even after Hexmage, you'd have 1 age counter and pay 1x cost.

## Current State (from ability-wip.md)

- [ ] Step 1: Enum variant
- [ ] Step 2: Rule enforcement
- [ ] Step 3: Trigger wiring
- [ ] Step 4: Unit tests
- [ ] Step 5: Card definition
- [ ] Step 6: Game script
- [ ] Step 7: Coverage doc update

## Implementation Steps

### Step 1: CumulativeUpkeepCost Enum

**File**: `crates/engine/src/state/types.rs`
**Action**: Add a `CumulativeUpkeepCost` enum near the other cost/keyword types (after `EnchantTarget`, around line 160).

```rust
/// CR 702.24a: The cost paid for each age counter on a permanent with
/// cumulative upkeep. Multiplied by the number of age counters at
/// resolution time.
///
/// Most common variants are mana costs (Mystic Remora: {1}) and life
/// costs (Glacial Chasm: "Pay 2 life"). Complex action-based costs
/// (Herald of Leshrac) are not yet supported.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum CumulativeUpkeepCost {
    /// Pay a mana cost for each age counter (most common).
    /// Example: Mystic Remora with "{1}" -- pay {1} per counter.
    Mana(ManaCost),
    /// Pay life for each age counter.
    /// Example: Glacial Chasm with "Pay 2 life" -- pay 2 life per counter.
    Life(u32),
}
```

**Note**: `ManaCost` is in `state/game_object.rs`. Import it in `types.rs` or use the full path. Check how `Echo(ManaCost)` handles this import.

### Step 2: KeywordAbility Variant

**File**: `crates/engine/src/state/types.rs`
**Action**: Add `KeywordAbility::CumulativeUpkeep(CumulativeUpkeepCost)` variant after `Echo(ManaCost)` (around the line after discriminant 114).

```rust
/// CR 702.24a: Cumulative upkeep [cost] -- "At the beginning of your upkeep,
/// if this permanent is on the battlefield, put an age counter on this
/// permanent. Then you may pay [cost] for each age counter on it. If you
/// don't, sacrifice it."
///
/// The CumulativeUpkeepCost parameter is the per-counter cost.
/// CR 702.24b: Each instance triggers separately, but all share age counters.
CumulativeUpkeep(CumulativeUpkeepCost),
```

**Hash**: Add to `state/hash.rs` -- KeywordAbility discriminant **115**.
```rust
// CumulativeUpkeep (discriminant 115) -- CR 702.24
KeywordAbility::CumulativeUpkeep(cost) => {
    115u8.hash_into(hasher);
    cost.hash_into(hasher);
}
```

Also add `HashInto` impl for `CumulativeUpkeepCost`:
```rust
impl HashInto for CumulativeUpkeepCost {
    fn hash_into(&self, hasher: &mut Hasher) {
        match self {
            CumulativeUpkeepCost::Mana(cost) => {
                0u8.hash_into(hasher);
                cost.hash_into(hasher);
            }
            CumulativeUpkeepCost::Life(amount) => {
                1u8.hash_into(hasher);
                amount.hash_into(hasher);
            }
        }
    }
}
```

**Match arms**: Grep for all `KeywordAbility` exhaustive match expressions and add `CumulativeUpkeep(..)` arm. Key locations:
- `state/hash.rs` (HashInto impl)
- `cards/builder.rs` (if any match on keywords)
- `rules/layers.rs` (calculate_characteristics -- pass-through, no special handling)
- Any other exhaustive matches found by the compiler

### Step 3: CounterType::Age

**File**: `crates/engine/src/state/types.rs`
**Action**: Add `Age` variant to `CounterType` enum (after `Fade`, around line 149).

```rust
/// CR 702.24a: Age counters are placed on permanents with cumulative upkeep.
/// One age counter is added at the beginning of each upkeep before the
/// payment check. The total number of age counters determines the total cost.
Age,
```

**Hash**: Add to `state/hash.rs` in the `CounterType` HashInto impl. Check existing discriminant pattern -- `Fade` likely has a discriminant; `Age` gets the next one.

### Step 4: AbilityDefinition Variant

**File**: `crates/engine/src/cards/card_definition.rs`
**Action**: Add `AbilityDefinition::CumulativeUpkeep { cost: CumulativeUpkeepCost }` variant after `Echo { cost: ManaCost }` (around discriminant 43).

```rust
/// CR 702.24a: Cumulative upkeep [cost] -- triggered ability that fires on
/// the controller's upkeep. Adds an age counter, then requires payment of
/// cost x age_count or sacrifice.
///
/// Cards with this ability should also include
/// `AbilityDefinition::Keyword(KeywordAbility::CumulativeUpkeep(cost))` for
/// quick presence-checking.
CumulativeUpkeep { cost: CumulativeUpkeepCost },
```

**Hash**: Add to `state/hash.rs` -- AbilityDefinition discriminant **44**.
```rust
// CumulativeUpkeep (discriminant 44) -- CR 702.24
AbilityDefinition::CumulativeUpkeep { cost } => {
    44u8.hash_into(hasher);
    cost.hash_into(hasher);
}
```

### Step 5: PendingTriggerKind and StackObjectKind

**File**: `crates/engine/src/state/stubs.rs`
**Action**: Add `PendingTriggerKind::CumulativeUpkeep` variant.

```rust
/// CR 702.24a: Cumulative upkeep trigger -- add age counter, then pay or sacrifice.
CumulativeUpkeep,
```

Also add a new field to `PendingTrigger` to carry the cumulative upkeep cost:
```rust
/// Only meaningful when `kind == PendingTriggerKind::CumulativeUpkeep`.
/// Carries the per-counter cost from trigger queueing to stack object creation.
#[serde(default)]
pub cumulative_upkeep_cost: Option<CumulativeUpkeepCost>,
```

**File**: `crates/engine/src/state/stack.rs`
**Action**: Add `StackObjectKind::CumulativeUpkeepTrigger` variant.

```rust
/// CR 702.24a: "At the beginning of your upkeep, if this permanent is on the
/// battlefield, put an age counter on this permanent. Then you may pay [cost]
/// for each age counter on it. If you don't, sacrifice it."
/// Discriminant 41.
CumulativeUpkeepTrigger {
    source_object: ObjectId,
    cu_permanent: ObjectId,
    per_counter_cost: CumulativeUpkeepCost,
},
```

**Hash**: Add to `state/hash.rs` -- StackObjectKind discriminant **41**.
```rust
// CumulativeUpkeepTrigger (discriminant 41) -- CR 702.24a
StackObjectKind::CumulativeUpkeepTrigger {
    source_object,
    cu_permanent,
    per_counter_cost,
} => {
    41u8.hash_into(hasher);
    source_object.hash_into(hasher);
    cu_permanent.hash_into(hasher);
    per_counter_cost.hash_into(hasher);
}
```

**TUI**: Add arm in `tools/tui/src/play/panels/stack_view.rs` for `StackObjectKind::CumulativeUpkeepTrigger { .. }`.

### Step 6: Upkeep Trigger Queueing

**File**: `crates/engine/src/rules/turn_actions.rs`
**Action**: Add cumulative upkeep trigger queueing in `upkeep_actions()` after the Echo block (around line 307+). Follow the Echo pattern.

```rust
// CR 702.24a: Queue upkeep triggers for all cumulative upkeep permanents.
// "At the beginning of your upkeep, if this permanent is on the battlefield,
// put an age counter on this permanent. Then you may pay [cost] for each age
// counter on it. If you don't, sacrifice it."
//
// Only fires for permanents controlled by the active player (CR 702.24a: "your upkeep").
// Must use layer-resolved characteristics to check for CumulativeUpkeep keyword.
// CR 702.24b: Each instance triggers separately.
let cu_permanents: Vec<(ObjectId, Vec<CumulativeUpkeepCost>)> = state
    .objects
    .values()
    .filter(|obj| {
        obj.zone == ZoneId::Battlefield && obj.controller == active
    })
    .map(|obj| {
        let cu_costs: Vec<CumulativeUpkeepCost> = obj
            .characteristics
            .keywords
            .iter()
            .filter_map(|kw| {
                if let KeywordAbility::CumulativeUpkeep(cost) = kw {
                    Some(cost.clone())
                } else {
                    None
                }
            })
            .collect();
        (obj.id, cu_costs)
    })
    .filter(|(_, costs)| !costs.is_empty())
    .collect();

for (obj_id, costs) in cu_permanents {
    for cost in costs {
        state.pending_triggers.push_back(PendingTrigger {
            source: obj_id,
            ability_index: 0, // unused for CU triggers
            controller: active,
            kind: PendingTriggerKind::CumulativeUpkeep,
            // ... all other fields default/None (follow Echo pattern)
            cumulative_upkeep_cost: Some(cost),
        });
    }
}
```

**Key difference from Echo**: No `echo_pending` flag needed. Cumulative upkeep fires EVERY upkeep, not just the first one. The age counters are the state tracker.

### Step 7: Trigger-to-Stack Conversion

**File**: `crates/engine/src/rules/abilities.rs`
**Action**: Add `PendingTriggerKind::CumulativeUpkeep` arm in the trigger-to-stack conversion match (after `EchoUpkeep`).

```rust
PendingTriggerKind::CumulativeUpkeep => {
    // CR 702.24a: Cumulative upkeep trigger.
    StackObjectKind::CumulativeUpkeepTrigger {
        source_object: trigger.source,
        cu_permanent: trigger.source,
        per_counter_cost: trigger.cumulative_upkeep_cost.clone()
            .expect("CumulativeUpkeep trigger must have cost"),
    }
}
```

### Step 8: Player Choice -- Command and Events

**File**: `crates/engine/src/rules/command.rs`
**Action**: Add `Command::PayCumulativeUpkeep` variant after `PayEcho`.

```rust
// ── Cumulative Upkeep (CR 702.24) ────────────────────────────────────
/// Choose whether to pay the cumulative upkeep cost for a permanent (CR 702.24a).
///
/// Sent in response to a `CumulativeUpkeepPaymentRequired` event. If `pay`
/// is true, the player pays the total cost (per-counter cost x age_count).
/// If `pay` is false, the permanent is sacrificed.
PayCumulativeUpkeep {
    player: PlayerId,
    permanent: ObjectId,
    /// True = pay the total cumulative upkeep cost. False = sacrifice.
    pay: bool,
},
```

**File**: `crates/engine/src/rules/events.rs`
**Action**: Add two new GameEvent variants after `EchoPaid`.

```rust
// ── Cumulative Upkeep events (CR 702.24) ─────────────────────────────
/// CR 702.24a: A cumulative upkeep trigger has resolved. The age counter
/// has been added. The controller must choose whether to pay the total
/// cost or sacrifice the permanent.
///
/// The engine pauses until a `Command::PayCumulativeUpkeep` is received.
CumulativeUpkeepPaymentRequired {
    player: PlayerId,
    permanent: ObjectId,
    /// The per-counter cost.
    per_counter_cost: CumulativeUpkeepCost,
    /// Number of age counters currently on the permanent (after adding one).
    age_counter_count: u32,
    /// The total cost to be paid (per_counter_cost x age_counter_count).
    /// For mana: multiply each component. For life: multiply the amount.
    total_description: String,
},

/// CR 702.24a: A player paid the cumulative upkeep cost for a permanent.
CumulativeUpkeepPaid {
    player: PlayerId,
    permanent: ObjectId,
    age_counter_count: u32,
},
```

**Hash**: Add to `state/hash.rs`:
- `CumulativeUpkeepPaymentRequired` -- GameEvent discriminant **91**
- `CumulativeUpkeepPaid` -- GameEvent discriminant **92**

### Step 9: Trigger Resolution

**File**: `crates/engine/src/rules/resolution.rs`
**Action**: Add `StackObjectKind::CumulativeUpkeepTrigger` resolution handler after `EchoTrigger` handler (around line 1399+).

The resolution handler does the UNIQUE work of cumulative upkeep:
1. Check if permanent is still on the battlefield (CR 400.7). If not, do nothing.
2. **Add one age counter** to the permanent (CR 702.24a: "put an age counter on this permanent").
3. Count total age counters on the permanent.
4. Emit `CumulativeUpkeepPaymentRequired` event with the per-counter cost and age count.
5. Add to `pending_cumulative_upkeep_payments` to pause the game.

```rust
StackObjectKind::CumulativeUpkeepTrigger {
    source_object: _,
    cu_permanent,
    per_counter_cost,
} => {
    let controller = stack_obj.controller;

    // Check if permanent is still on the battlefield (CR 400.7).
    let still_on_battlefield = state
        .objects
        .get(&cu_permanent)
        .map(|obj| obj.zone == ZoneId::Battlefield)
        .unwrap_or(false);

    if still_on_battlefield {
        // CR 702.24a: "put an age counter on this permanent"
        if let Some(obj) = state.objects.get_mut(&cu_permanent) {
            let current = obj.counters.get(&CounterType::Age).copied().unwrap_or(0);
            obj.counters.insert(CounterType::Age, current + 1);
        }

        // Count total age counters after adding.
        let age_count = state
            .objects
            .get(&cu_permanent)
            .and_then(|obj| obj.counters.get(&CounterType::Age).copied())
            .unwrap_or(0) as u32;

        // Emit payment required event.
        events.push(GameEvent::CumulativeUpkeepPaymentRequired {
            player: controller,
            permanent: cu_permanent,
            per_counter_cost: per_counter_cost.clone(),
            age_counter_count: age_count,
            total_description: format_cu_total(&per_counter_cost, age_count),
        });

        // Track pending payment.
        state
            .pending_cumulative_upkeep_payments
            .push_back((controller, cu_permanent, per_counter_cost));
    }

    events.push(GameEvent::AbilityResolved {
        controller,
        stack_object_id: stack_obj.id,
    });
}
```

**Helper function** `format_cu_total` (in `resolution.rs` or a shared utility):
```rust
fn format_cu_total(cost: &CumulativeUpkeepCost, count: u32) -> String {
    match cost {
        CumulativeUpkeepCost::Mana(mc) => {
            // Multiply each mana component by count
            format!("Pay {} total mana ({} x {})", /* ... */)
        }
        CumulativeUpkeepCost::Life(amount) => {
            format!("Pay {} life ({} x {})", amount * count, amount, count)
        }
    }
}
```

### Step 10: Payment Command Handler

**File**: `crates/engine/src/rules/engine.rs`
**Action**: Add `Command::PayCumulativeUpkeep` handler after the `PayEcho` handler. Follow the `handle_pay_echo` pattern closely.

```rust
// ── Cumulative Upkeep (CR 702.24) ───────────────────────────────
Command::PayCumulativeUpkeep {
    player,
    permanent,
    pay,
} => {
    validate_player_exists(&state, player)?;
    loop_detection::reset_loop_detection(&mut state);
    let mut events = handle_pay_cumulative_upkeep(&mut state, player, permanent, pay)?;
    let new_triggers = abilities::check_triggers(&state, &events);
    for t in new_triggers {
        state.pending_triggers.push_back(t);
    }
    let trigger_events = abilities::flush_pending_triggers(&mut state);
    events.extend(trigger_events);
    all_events.extend(events);
}
```

**New function** `handle_pay_cumulative_upkeep` in `engine.rs` (after `handle_pay_echo`):

```rust
fn handle_pay_cumulative_upkeep(
    state: &mut GameState,
    player: PlayerId,
    permanent: ObjectId,
    pay: bool,
) -> Result<Vec<GameEvent>, GameStateError> {
    let mut events = Vec::new();

    // Find and remove the pending payment entry.
    let payment_pos = state
        .pending_cumulative_upkeep_payments
        .iter()
        .position(|(p, obj, _)| *p == player && *obj == permanent);

    let per_counter_cost = if let Some(pos) = payment_pos {
        let (_, _, cost) = state.pending_cumulative_upkeep_payments.remove(pos);
        cost
    } else {
        return Err(GameStateError::InvalidCommand(format!(
            "No pending cumulative upkeep payment for player {:?} permanent {:?}",
            player, permanent
        )));
    };

    // Validate: permanent must still be on the battlefield.
    let source_info = state.objects.get(&permanent).and_then(|obj| {
        if obj.zone == ZoneId::Battlefield {
            Some((obj.owner, obj.controller, obj.counters.clone()))
        } else {
            None
        }
    });

    let Some((owner, controller, pre_death_counters)) = source_info else {
        return Ok(events);
    };

    // Count age counters (already incremented during trigger resolution).
    let age_count = state
        .objects
        .get(&permanent)
        .and_then(|obj| obj.counters.get(&CounterType::Age).copied())
        .unwrap_or(0) as u32;

    if pay {
        match &per_counter_cost {
            CumulativeUpkeepCost::Mana(mc) => {
                // Multiply mana cost by age_count.
                let total_cost = multiply_mana_cost(mc, age_count);
                let pool = &state.players.get(&player)
                    .ok_or(GameStateError::PlayerNotFound(player))?
                    .mana_pool;
                if !casting::can_pay_cost(pool, &total_cost) {
                    return Err(GameStateError::InvalidCommand(
                        format!("Player {:?} cannot afford cumulative upkeep cost", player)
                    ));
                }
                if let Some(p) = state.players.get_mut(&player) {
                    casting::pay_cost(&mut p.mana_pool, &total_cost);
                }
            }
            CumulativeUpkeepCost::Life(amount) => {
                let total_life = amount * age_count;
                // Deduct life.
                if let Some(p) = state.players.get_mut(&player) {
                    p.life_total -= total_life as i64;
                }
                // Emit life loss event.
                events.push(GameEvent::LifeLost {
                    player,
                    amount: total_life,
                });
            }
        }
        events.push(GameEvent::CumulativeUpkeepPaid {
            player,
            permanent,
            age_counter_count: age_count,
        });
    } else {
        // Sacrifice the permanent (follow handle_pay_echo sacrifice pattern).
        // Use check_zone_change_replacement for commander redirect.
        // Emit CreatureDied or PermanentLeftBattlefield as appropriate.
        // Follow the exact sacrifice pattern from handle_pay_echo lines 540-589.
    }

    Ok(events)
}
```

**Helper function** `multiply_mana_cost` (in `engine.rs` or a shared utility):
```rust
fn multiply_mana_cost(cost: &ManaCost, multiplier: u32) -> ManaCost {
    ManaCost {
        white: cost.white * multiplier,
        blue: cost.blue * multiplier,
        black: cost.black * multiplier,
        red: cost.red * multiplier,
        green: cost.green * multiplier,
        colorless: cost.colorless * multiplier,
        generic: cost.generic * multiplier,
    }
}
```

### Step 11: Pending Choice Tracking

**File**: `crates/engine/src/state/mod.rs` (GameState)
**Action**: Add a field to track pending cumulative upkeep payment choices, next to `pending_echo_payments`.

```rust
/// CR 702.24a: Pending cumulative upkeep payment choices. When a cumulative
/// upkeep trigger resolves (after adding the age counter), the controller
/// must choose to pay or sacrifice. The game pauses until a
/// `Command::PayCumulativeUpkeep` is received for each entry.
#[serde(default)]
pub pending_cumulative_upkeep_payments: im::Vector<(PlayerId, ObjectId, CumulativeUpkeepCost)>,
```

**Hash**: Add to `state/hash.rs` in the GameState HashInto impl.

### Step 12: Countered Trigger Handling

**File**: `crates/engine/src/rules/resolution.rs`
**Action**: Add `StackObjectKind::CumulativeUpkeepTrigger { .. }` to the "counter abilities" catch-all arm (around line 3953).

```rust
| StackObjectKind::CumulativeUpkeepTrigger { .. }
```

With note:
```rust
// Note: For CumulativeUpkeepTrigger, if countered (e.g. by Stifle), no age
// counter is added (the counter addition happens at resolution, not queueing).
// The trigger will fire again on the next upkeep with the same counter count.
```

### Step 13: builder.rs Trigger Registration

**File**: `crates/engine/src/cards/builder.rs`
**Action**: Add `CumulativeUpkeep` to the keyword-to-trigger builder logic (near the Echo/Fading patterns). The builder should recognize `KeywordAbility::CumulativeUpkeep(_)` and register the upkeep trigger for the card.

Check how Echo's builder registration works -- it likely adds an upkeep trigger through `AbilityDefinition::Echo`. Follow the same pattern for `AbilityDefinition::CumulativeUpkeep`.

### Step 14: ETB Counter Placement (NOT needed)

Unlike Echo (which needs `echo_pending` flag set at ETB), cumulative upkeep does NOT need any ETB work. There is no "pending" flag -- the trigger fires every upkeep unconditionally. Age counters start at 0 and are added during trigger resolution, not at ETB.

**No changes needed** in `resolution.rs` ETB block or `lands.rs` ETB block.

### Step 15: helpers.rs Export

**File**: `crates/engine/src/cards/helpers.rs`
**Action**: Add `CumulativeUpkeepCost` to the re-exports so card definitions can use it.

```rust
pub use super::super::state::types::CumulativeUpkeepCost;
```

### Step 16: Unit Tests

**File**: `crates/engine/tests/cumulative_upkeep.rs`
**Tests to write**:
- `test_cumulative_upkeep_basic_age_counter_added` -- CR 702.24a: advancing to upkeep adds an age counter to the permanent and triggers payment choice
- `test_cumulative_upkeep_pay_mana_keeps_permanent` -- CR 702.24a: paying the mana cost (cost x age_count) keeps the permanent
- `test_cumulative_upkeep_decline_payment_sacrifices` -- CR 702.24a: declining payment sacrifices the permanent
- `test_cumulative_upkeep_escalating_cost` -- CR 702.24a: cost increases each turn (turn 1: 1x, turn 2: 2x, turn 3: 3x)
- `test_cumulative_upkeep_pay_life_cost` -- CR 702.24a: life-based cost variant (Glacial Chasm pattern)
- `test_cumulative_upkeep_permanent_left_battlefield` -- CR 400.7: if permanent left battlefield before trigger resolves, do nothing
- `test_cumulative_upkeep_multiplayer_only_controller_upkeep` -- CR 702.24a: fires only on the controller's upkeep
- `test_cumulative_upkeep_multiple_instances_share_counters` -- CR 702.24b: two instances trigger separately but count all age counters

**Pattern**: Follow tests in `crates/engine/tests/echo.rs` -- same setup pattern, same helper functions.

### Step 17: Card Definition (later phase)

**Suggested card**: Mystic Remora ({U}, Enchantment, Cumulative upkeep {1}, "Whenever an opponent casts a noncreature spell, you may draw a card unless that player pays {4}.")
- Commander staple, simple mana-based cumulative upkeep cost
- The "opponent casts" trigger is a separate ability from the CU mechanic; can be stubbed

**Alternative**: Glacial Chasm (Land, Cumulative upkeep -- Pay 2 life, ETB sacrifice a land, can't attack, prevent all damage to you) -- tests the life-cost variant but has more complex other abilities.

**Card lookup**: use `card-definition-author` agent

### Step 18: Game Script (later phase)

**Suggested scenario**: Player controls Mystic Remora. Turn 1 upkeep: pay {1}. Turn 2 upkeep: pay {2}. Turn 3 upkeep: decline, sacrifice Mystic Remora.
**Subsystem directory**: `test-data/generated-scripts/stack/`

## Interactions to Watch

- **Commander redirect**: If the CU permanent is a commander and the player declines to pay, the sacrifice triggers the commander zone return SBA (CR 903.9a). The sacrifice handler must use `check_zone_change_replacement` like Echo does.
- **Stifle/counter**: If the CU trigger is countered, NO age counter is added (unlike Echo where echo_pending persists). The trigger simply fires again next upkeep. This is DIFFERENT from Echo's behavior where the flag persists.
- **Humility**: If Humility removes all abilities (including CumulativeUpkeep), the upkeep trigger scanning checks layer-resolved characteristics. If CU is removed, the trigger should not fire. But existing age counters remain on the permanent (they're not removed by Humility). If Humility later leaves, the CU trigger fires again and counts all existing age counters.
- **Proliferate**: Proliferate can add extra age counters, increasing the cost. Works naturally since the trigger counts all age counters at resolution time.
- **Flickering**: If the permanent is flickered (exiled and returned), it gets a new ObjectId (CR 400.7). Age counters are reset (new object has no counters). The old trigger on the stack references a dead ObjectId and does nothing.
- **Panharmonicon**: CU triggers at beginning of upkeep, NOT on ETB. Panharmonicon does not double it.
- **Solemnity**: Solemnity ("Counters can't be placed on...") prevents the age counter from being added. If no age counter can be added, the trigger still resolves -- but the count is 0 (or whatever was already there). With 0 age counters, the cost is 0 x [cost] = free. This is a known interaction -- Solemnity + Glacial Chasm = permanent Glacial Chasm with no upkeep cost. The engine handles this naturally if age counter placement checks for Solemnity (via replacement effects or counter prevention). For now, Solemnity is not implemented, so this interaction is deferred.

## Discriminant Summary

| Type | Variant | Discriminant |
|------|---------|-------------|
| KeywordAbility | CumulativeUpkeep(CumulativeUpkeepCost) | 115 |
| AbilityDefinition | CumulativeUpkeep { cost: CumulativeUpkeepCost } | 44 |
| StackObjectKind | CumulativeUpkeepTrigger | 41 |
| PendingTriggerKind | CumulativeUpkeep | (enum, no disc needed) |
| GameEvent | CumulativeUpkeepPaymentRequired | 91 |
| GameEvent | CumulativeUpkeepPaid | 92 |
| Command | PayCumulativeUpkeep | (enum, no disc needed) |
| CounterType | Age | (next after Fade in hash.rs) |

## Design Decisions

1. **CumulativeUpkeepCost enum vs ManaCost**: Unlike Echo which uses `ManaCost` directly, CU needs a flexible cost type because cards like Glacial Chasm pay life, not mana. The `CumulativeUpkeepCost` enum handles both. Future expansion: `AddMana(ManaColor)` for Braid of Fire, `Custom(String)` for complex costs like Herald of Leshrac.

2. **No pending flag (unlike Echo)**: Echo uses `echo_pending` because it only fires once ("came under your control since your last upkeep"). CU fires every upkeep unconditionally. Age counters track the state directly.

3. **Age counter added during trigger resolution, not queueing**: This matches CR 702.24a exactly and has the correct interaction with Stifle (counter the trigger = no age counter added). If the trigger is put on the stack but later countered, the permanent keeps its current age counter count.

4. **Counter multiplication at payment time**: The `multiply_mana_cost` helper creates a total ManaCost by multiplying each component. This is simple and correct for mana costs. For life costs, simple multiplication works directly.
