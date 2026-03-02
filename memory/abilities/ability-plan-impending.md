# Ability Plan: Impending

**Generated**: 2026-03-02
**CR**: 702.176
**Priority**: P4 (Batch 5)
**Similar abilities studied**: Dash (alt cost + end step trigger), Blitz (alt cost + end step trigger), Suspend (time counter removal trigger), Devoid/Changeling (inline Layer 4 type modification in `calculate_characteristics`)

## CR Rule Text

702.176. Impending

702.176a Impending is a keyword that represents four abilities. The first is a static ability that functions while the spell with impending is on the stack. The second is static ability that creates a replacement effect that may apply to the permanent with impending as it enters the battlefield from the stack. The third is a static ability that functions on the battlefield. The fourth is a triggered ability that functions on the battlefield. "Impending N--[cost]" means "You may choose to pay [cost] rather than pay this spell's mana cost," "If you chose to pay this permanent's impending cost, it enters with N time counters on it," "As long as this permanent's impending cost was paid and it has a time counter on it, it's not a creature," and "At the beginning of your end step, if this permanent's impending cost was paid and it has a time counter on it, remove a time counter from it." Casting a spell for its impending cost follows the rules for paying alternative costs in rules 601.2b and 601.2f-h.

## Key Edge Cases

1. **Still a creature spell on the stack** (ruling 2024-09-20): "If you choose to pay the impending cost of a creature spell, it's still a creature spell on the stack. You can cast that spell for its impending cost only when you could normally cast that creature spell." The impending cost does NOT change the timing -- sorcery-speed creature cards still require main phase + empty stack. The spell's types on the stack are unchanged.

2. **Copy does not inherit impending status** (ruling 2024-09-20): "If an object enters as a copy of a permanent that was cast with its impending cost, it won't enter with time counters, and it will be a creature." The `cast_alt_cost` field is NOT part of copiable values (CR 707). Copies bypass the replacement effect entirely.

3. **Type removal is conditional on BOTH impending cost paid AND time counters present**: The permanent only loses the Creature type while BOTH conditions are true: (a) `cast_alt_cost == Some(AltCostKind::Impending)` and (b) the object has at least one time counter (`counters.get(CounterType::Time) > 0`). When the last time counter is removed, the type-removal effect stops applying automatically -- no separate trigger needed.

4. **End step counter removal is controller's end step only** (CR 702.176a): "At the beginning of YOUR end step" -- the trigger only fires for the controller of the permanent, on their own end step. In multiplayer, this means each player's impending permanents tick down only on that player's own end step.

5. **Counter removal is a triggered ability** (goes on the stack, can be countered by Stifle, etc.): If the counter-removal trigger is countered, the permanent retains its time counter(s) longer -- it stays as a non-creature enchantment for at least one more turn cycle.

6. **Intervening-if check** (CR 603.4): "if this permanent's impending cost was paid and it has a time counter on it" -- checked both at trigger time AND at resolution time. If the permanent loses its time counters between trigger and resolution (e.g., via Clockspinning or Vampire Hexmage), the trigger does nothing.

7. **Zone changes reset cast_alt_cost** (CR 400.7): If an impending permanent is blinked (leaves and re-enters), the new object has `cast_alt_cost: None`. The time counters are also gone. The permanent re-enters as a normal creature.

8. **Multiplayer**: End step scanning must only check permanents whose controller matches the active player (the player whose end step it is).

9. **cast_alt_cost mutual exclusion** (CR 118.9a): Impending is an alternative cost. Cannot combine with other alternative costs (Flashback, Evoke, Dash, Blitz, etc.). CAN combine with Prototype (which is NOT an alternative cost per ruling 2022-10-14) -- but this interaction is unlikely on actual Impending cards (all Duskmourn Overlords are Enchantment Creatures, not artifact creatures with prototype).

## Current State (from ability-wip.md)

- [ ] Step 1: Enum variant
- [ ] Step 2: Rule enforcement
- [ ] Step 3: Trigger wiring
- [ ] Step 4: Unit tests
- [ ] Step 5: Card definition
- [ ] Step 6: Game script
- [ ] Step 7: Coverage doc update

## Implementation Steps

### Step 1: Enum Variants & Type Definitions

#### 1a: AltCostKind::Impending

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/types.rs`
**Action**: Add `Impending` variant to `AltCostKind` enum at line 112 (replacing the "Future" comment).
**Pattern**: Follow `AltCostKind::Blitz` at line 110.
**CR**: 702.176a -- "You may choose to pay [cost] rather than pay this spell's mana cost" (alternative cost per CR 601.2b).

```rust
// Before:
    Plot,
    // Future: Prototype, Impending (add as implemented)
}

// After:
    Plot,
    Impending,
}
```

#### 1b: KeywordAbility::Impending

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/types.rs`
**Action**: Add `Impending` variant after `KeywordAbility::Prototype` (line 882).
**Pattern**: Follow `KeywordAbility::Blitz` at line 850.
**Discriminant**: 99 in hash.rs (next after Prototype=98).
**CR**: 702.176 -- marker for quick presence-checking.

```rust
/// CR 702.176: Impending N--[cost]. Alternative cost; enters with N time counters;
/// not a creature while it has time counters and was cast for impending cost;
/// remove a time counter at beginning of controller's end step.
///
/// Marker for quick presence-checking (`keywords.contains`).
/// The impending cost and counter count are stored in
/// `AbilityDefinition::Impending { cost, count }`.
Impending,
```

#### 1c: AbilityDefinition::Impending

**File**: `/home/airbaggie/scutemob/crates/engine/src/cards/card_definition.rs`
**Action**: Add `Impending { cost: ManaCost, count: u32 }` variant after `AbilityDefinition::Prototype`.
**Pattern**: Follow `AbilityDefinition::Dash { cost }` at line 341 (alt cost pattern) and `AbilityDefinition::Suspend { cost, time_counters }` at line 254 (parameterized cost+count).
**Discriminant**: 32 in hash.rs (next after Prototype=31).
**CR**: 702.176a -- stores the alternative cost and the number of time counters.

```rust
/// CR 702.176: Impending N--[cost]. You may cast this spell by paying [cost]
/// rather than its mana cost. If you do, it enters with N time counters and
/// isn't a creature while it has time counters. At the beginning of your end
/// step, remove a time counter from it.
///
/// Cards with this ability should also include
/// `AbilityDefinition::Keyword(KeywordAbility::Impending)` for quick
/// presence-checking without scanning all abilities.
Impending { cost: ManaCost, count: u32 },
```

#### 1d: StackObject::was_impended

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/stack.rs`
**Action**: Add `was_impended: bool` field to `StackObject` after `was_prototyped` (line 184).
**Pattern**: Follow `was_dashed: bool` at line 158.
**CR**: 702.176a -- tracks whether impending cost was paid on the stack.

```rust
/// CR 702.176a: If true, this spell was cast by paying its impending cost
/// (an alternative cost). When the permanent enters the battlefield, it
/// enters with N time counters and is not a creature while it has time
/// counters.
///
/// Must always be false for copies (`is_copy: true`) -- copies are not cast.
#[serde(default)]
pub was_impended: bool,
```

#### 1e: PendingTriggerKind::ImpendingCounter

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/stubs.rs`
**Action**: Add `ImpendingCounter` variant to `PendingTriggerKind` after `BlitzSacrifice` (line 71).
**Pattern**: Follow `PendingTriggerKind::SuspendCounter` at line 43.
**CR**: 702.176a -- "At the beginning of your end step, if this permanent's impending cost was paid and it has a time counter on it, remove a time counter from it."

```rust
/// CR 702.176a: Impending end-step counter-removal trigger.
ImpendingCounter,
```

#### 1f: StackObjectKind::ImpendingCounterTrigger

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/stack.rs`
**Action**: Add `ImpendingCounterTrigger` variant to `StackObjectKind` after `BlitzSacrificeTrigger` (line 745).
**Pattern**: Follow `StackObjectKind::SuspendCounterTrigger` (has `source_object` and `impending_permanent` fields).
**Discriminant**: 33 in hash.rs (next after BlitzSacrificeTrigger=32).
**CR**: 702.176a -- triggered ability: remove one time counter from the impending permanent.

```rust
/// CR 702.176a: Impending end-step counter-removal trigger.
///
/// "At the beginning of your end step, if this permanent's impending cost
/// was paid and it has a time counter on it, remove a time counter from it."
///
/// When this trigger resolves:
/// 1. Re-check intervening-if: permanent must still be on battlefield,
///    must have `cast_alt_cost == Some(AltCostKind::Impending)`, and must
///    have at least one time counter (CR 603.4).
/// 2. If yes, remove one time counter.
/// 3. If no (conditions no longer met), do nothing.
///
/// Unlike Suspend, there is no follow-up trigger when the last counter is
/// removed -- the permanent simply becomes a creature because the type-
/// removal effect in Layer 4 stops applying.
ImpendingCounterTrigger {
    source_object: ObjectId,
    impending_permanent: ObjectId,
},
```

### Step 2: Hash Updates

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/hash.rs`
**Action**: Add hash arms for all new types.

#### 2a: KeywordAbility::Impending hash

After `KeywordAbility::Prototype => 98u8.hash_into(hasher),` (line 532):
```rust
// Impending (discriminant 99) -- CR 702.176
KeywordAbility::Impending => 99u8.hash_into(hasher),
```

#### 2b: AbilityDefinition::Impending hash

After the `AbilityDefinition::Prototype` block (around line 3205):
```rust
// Impending (discriminant 32) -- CR 702.176
AbilityDefinition::Impending { cost, count } => {
    32u8.hash_into(hasher);
    cost.hash_into(hasher);
    count.hash_into(hasher);
}
```

#### 2c: StackObjectKind::ImpendingCounterTrigger hash

After the `StackObjectKind::BlitzSacrificeTrigger` block (around line 1574):
```rust
// ImpendingCounterTrigger (discriminant 33) -- CR 702.176a
StackObjectKind::ImpendingCounterTrigger {
    source_object,
    impending_permanent,
} => {
    33u8.hash_into(hasher);
    source_object.hash_into(hasher);
    impending_permanent.hash_into(hasher);
}
```

#### 2d: StackObject::was_impended hash

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/hash.rs`
**Action**: Add `self.was_impended.hash_into(hasher);` to the `StackObject` HashInto impl, after `self.was_prototyped.hash_into(hasher);` (around line 1644).

### Step 3: Casting Flow (Alternative Cost)

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/casting.rs`
**Action**: Wire impending as an alternative cost, following the Dash/Blitz pattern.

#### 3a: Derive boolean from alt_cost

After `let cast_with_plot = alt_cost == Some(AltCostKind::Plot);` (line 82):
```rust
let cast_with_impending = alt_cost == Some(AltCostKind::Impending);
```

#### 3b: Validate mutual exclusion

Add a new validation block (like "Step 1k") after the plot validation block (around line 955):
```rust
// Step 1k: Validate impending mutual exclusion (CR 702.176a / CR 118.9a).
// Impending is an alternative cost -- cannot combine with other alternative costs.
let casting_with_impending = if cast_with_impending {
    if casting_with_flashback {
        return Err(GameStateError::InvalidCommand(
            "cannot combine impending with flashback (CR 118.9a: only one alternative cost)".into(),
        ));
    }
    // ... (same mutual exclusion checks as Dash/Blitz for all other alt costs)
    if get_impending_cost(&card_id, &state.card_registry).is_none() {
        return Err(GameStateError::InvalidCommand(
            "card has no impending cost defined".into(),
        ));
    }
    true
} else {
    false
};
```
**Important**: Also add `casting_with_impending` to the mutual exclusion checks of ALL existing alt costs (Dash, Blitz, Evoke, Bestow, Miracle, Escape, Foretell, Aftermath, Overload, Plot).

#### 3c: Cost determination

In the cost determination section (around line 1073), add an `else if casting_with_impending` branch:
```rust
} else if casting_with_impending {
    // CR 702.176a: Pay impending cost instead of mana cost.
    get_impending_cost(&card_id, &state.card_registry)
```

#### 3d: StackObject construction

In the StackObject construction (around line 1646), add:
```rust
was_impended: casting_with_impending,
```

#### 3e: get_impending_cost helper

Add a new helper function at the end of `casting.rs` (after `get_dash_cost`):
```rust
/// CR 702.176a: Look up the impending cost from the card's `AbilityDefinition`.
fn get_impending_cost(
    card_id: &Option<crate::state::CardId>,
    registry: &crate::cards::CardRegistry,
) -> Option<ManaCost> {
    card_id.as_ref().and_then(|cid| {
        registry.get(cid.clone()).and_then(|def| {
            def.abilities.iter().find_map(|a| {
                if let AbilityDefinition::Impending { cost, .. } = a {
                    Some(cost.clone())
                } else {
                    None
                }
            })
        })
    })
}
```

#### 3f: get_impending_count helper

Add another helper to look up the N value (time counter count):
```rust
/// CR 702.176a: Look up the impending counter count from the card's `AbilityDefinition`.
fn get_impending_count(
    card_id: &Option<crate::state::CardId>,
    registry: &crate::cards::CardRegistry,
) -> Option<u32> {
    card_id.as_ref().and_then(|cid| {
        registry.get(cid.clone()).and_then(|def| {
            def.abilities.iter().find_map(|a| {
                if let AbilityDefinition::Impending { count, .. } = a {
                    Some(*count)
                } else {
                    None
                }
            })
        })
    })
}
```

### Step 4: Resolution (ETB with Time Counters)

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/resolution.rs`
**Action**: Transfer `was_impended` to `cast_alt_cost` and add time counters at ETB.

#### 4a: Transfer cast_alt_cost at resolution

In the `cast_alt_cost` transfer chain (line 284-294), add an arm for impending:
```rust
} else if stack_obj.was_impended {
    Some(AltCostKind::Impending)
```

#### 4b: Add time counters at ETB

After the `cast_alt_cost` transfer and the existing Dash/Blitz handling (around line 326), add:
```rust
if stack_obj.was_impended {
    // CR 702.176a: "If you chose to pay this permanent's impending cost,
    // it enters with N time counters on it."
    let impending_count = get_impending_count(&obj.card_id, &state.card_registry)
        .unwrap_or(0);
    if impending_count > 0 {
        let current = obj.counters.get(&CounterType::Time).copied().unwrap_or(0);
        obj.counters = obj.counters.update(CounterType::Time, current + impending_count);
    }
}
```

**Note**: Import `get_impending_count` from `casting.rs` or duplicate as a module-level helper. Since `casting.rs` helpers are private, duplicating (or extracting to a shared location) may be needed. Follow the existing pattern -- check if other resolution.rs code calls casting helpers or re-implements them inline.

#### 4c: All other StackObject construction sites

Grep for all sites that construct `StackObject` and add `was_impended: false`:
- `resolution.rs` SuspendCastTrigger (line ~1671): add `was_impended: false`
- `resolution.rs` CascadeTrigger (around line ~2529): add `was_impended: false`
- `resolution.rs` StormTrigger: add `was_impended: false`
- `resolution.rs` EmbalmAbility token creation: `was_impended: false`
- Any other StackObject construction sites

#### 4d: Countered stack object match arm

In the countered-abilities match (around line 3229), add the new variant:
```rust
| StackObjectKind::ImpendingCounterTrigger { .. }
```

### Step 5: Layer 4 Type-Removal (calculate_characteristics)

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/layers.rs`
**Action**: Add inline Layer 4 type-removal for impending, following the Changeling/Devoid inline pattern.
**CR**: 702.176a -- "As long as this permanent's impending cost was paid and it has a time counter on it, it's not a creature."

In `calculate_characteristics`, inside the `for &layer in &layers_in_order` loop, after the Devoid check (line 83) and before gathering `layer_effects`:

```rust
// CR 702.176a: Impending -- "As long as this permanent's impending cost was
// paid and it has a time counter on it, it's not a creature."
// Applied at Layer 4 (TypeChange) inline, similar to Changeling/Devoid CDAs.
// Unlike CDAs, this is a static ability that only functions on the battlefield.
if layer == EffectLayer::TypeChange {
    if let Some(obj_ref) = state.objects.get(&object_id) {
        if obj_ref.zone == ZoneId::Battlefield
            && obj_ref.cast_alt_cost == Some(AltCostKind::Impending)
            && obj_ref.counters.get(&CounterType::Time).copied().unwrap_or(0) > 0
        {
            chars.card_types.remove(&CardType::Creature);
            // Also remove creature subtypes? No -- CR 702.176a only says
            // "it's not a creature." The subtypes remain (they become
            // non-creature subtypes temporarily, which is unusual but legal).
            // When it becomes a creature again, the subtypes apply normally.
        }
    }
}
```

**Important**: This check should run AFTER the Changeling CDA (which adds all creature types) but BEFORE gathering layer_effects. Place it between the Changeling block (line 72) and the Devoid block, or after both. Since Devoid is Layer 5 and this is Layer 4, place it adjacent to the Changeling check (both Layer 4).

**Note on subtypes**: The CR says "it's not a creature" but does NOT say it loses creature subtypes. However, creature subtypes on a non-creature permanent are technically meaningless per CR 205.3a ("Subtypes are correlated with card types"). We should NOT remove subtypes -- when the last counter is removed and it becomes a creature again, the subtypes need to be there. The layer system handles this correctly: the subtypes persist, they just have no mechanical effect while it's not a creature.

### Step 6: End-Step Counter-Removal Trigger

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/turn_actions.rs`
**Action**: Add impending counter-removal trigger scanning to `end_step_actions`.
**Pattern**: Follow the Dash/Blitz end-step trigger pattern (lines 208-300).
**CR**: 702.176a -- "At the beginning of your end step, if this permanent's impending cost was paid and it has a time counter on it, remove a time counter from it."

In `end_step_actions()`, after the Blitz section (around line 300):

```rust
// CR 702.176a: Queue counter-removal triggers for all impending permanents.
// "At the beginning of your end step, if this permanent's impending cost was
// paid and it has a time counter on it, remove a time counter from it."
//
// Only fires for permanents whose controller matches the active player
// (the player whose end step it is). The trigger has an intervening-if
// condition (CR 603.4): must still have cast_alt_cost == Impending AND
// at least one time counter.
let active = state.turn.active_player;
let impending_permanents: Vec<ObjectId> = state
    .objects
    .values()
    .filter(|obj| {
        obj.zone == ZoneId::Battlefield
            && obj.controller == active
            && obj.cast_alt_cost == Some(AltCostKind::Impending)
            && obj.counters.get(&CounterType::Time).copied().unwrap_or(0) > 0
    })
    .map(|obj| obj.id)
    .collect();

for obj_id in impending_permanents {
    state.pending_triggers.push_back(PendingTrigger {
        source: obj_id,
        ability_index: 0,
        controller: active,
        kind: PendingTriggerKind::ImpendingCounter,
        triggering_event: None,
        entering_object_id: None,
        targeting_stack_id: None,
        triggering_player: None,
        exalted_attacker_id: None,
        defending_player_id: None,
        madness_exiled_card: None,
        madness_cost: None,
        miracle_revealed_card: None,
        miracle_cost: None,
        modular_counter_count: None,
        evolve_entering_creature: None,
        suspend_card_id: None,
        hideaway_count: None,
        partner_with_name: None,
        ingest_target_player: None,
        flanking_blocker_id: None,
        rampage_n: None,
        provoke_target_creature: None,
        renown_n: None,
        poisonous_n: None,
        poisonous_target_player: None,
        enlist_enlisted_creature: None,
        encore_activator: None,
    });
}
```

### Step 7: Trigger Resolution

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/resolution.rs`
**Action**: Add resolution handler for `StackObjectKind::ImpendingCounterTrigger`.
**Pattern**: Follow `StackObjectKind::SuspendCounterTrigger` resolution (lines 1520-1601).
**CR**: 702.176a + CR 603.4 (intervening-if re-check at resolution).

Add a new match arm in `resolve_top_of_stack` for `ImpendingCounterTrigger`:

```rust
// CR 702.176a: Impending counter-removal trigger resolves.
//
// "At the beginning of your end step, if this permanent's impending cost
// was paid and it has a time counter on it, remove a time counter from it."
//
// Intervening-if re-check (CR 603.4): permanent must still be on the
// battlefield, must have cast_alt_cost == Impending, and must have at
// least one time counter.
StackObjectKind::ImpendingCounterTrigger {
    source_object: _,
    impending_permanent,
} => {
    let controller = stack_obj.controller;

    // CR 603.4: Re-check intervening-if condition at resolution.
    let current_counters = state
        .objects
        .get(&impending_permanent)
        .filter(|obj| {
            obj.zone == ZoneId::Battlefield
                && obj.cast_alt_cost == Some(AltCostKind::Impending)
        })
        .and_then(|obj| obj.counters.get(&CounterType::Time).copied());

    if let Some(count) = current_counters {
        if count > 0 {
            // Remove one time counter (CR 702.176a).
            if let Some(obj) = state.objects.get_mut(&impending_permanent) {
                let new_count = count - 1;
                if new_count == 0 {
                    obj.counters.remove(&CounterType::Time);
                } else {
                    obj.counters.insert(CounterType::Time, new_count);
                }
            }
            events.push(GameEvent::CounterRemoved {
                object_id: impending_permanent,
                counter: CounterType::Time,
                count: 1,
            });
            // No follow-up trigger when last counter removed -- the
            // permanent simply becomes a creature because the Layer 4
            // type-removal effect in calculate_characteristics stops
            // applying (no time counters => condition is false).
        }
    }
    // If not on battlefield, or no impending status, or no counters,
    // the trigger does nothing (CR 603.4).

    events.push(GameEvent::AbilityResolved {
        controller,
        stack_object_id: stack_obj.id,
    });
}
```

### Step 8: Trigger Flush (PendingTriggerKind -> StackObjectKind)

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/abilities.rs`
**Action**: Add flush handler for `PendingTriggerKind::ImpendingCounter`.
**Pattern**: Follow the `SuspendCounter` flush pattern.

Find the match on `PendingTriggerKind` in `flush_pending_triggers` and add:

```rust
PendingTriggerKind::ImpendingCounter => {
    StackObjectKind::ImpendingCounterTrigger {
        source_object: trigger.source,
        impending_permanent: trigger.source,
    }
}
```

### Step 9: Builder & Token Construction Sites

#### 9a: builder.rs

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/builder.rs`
**Action**: Initialize `cast_alt_cost: None` is already set at line 909. No change needed for `cast_alt_cost`. But verify there are no impending-specific fields to initialize.

#### 9b: state/mod.rs zone-move resets

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/mod.rs`
**Action**: Verify `cast_alt_cost: None` is already set in both `move_object_to_zone` sites (lines 278-280 and 401-403). This ensures zone changes reset impending status. Already handled -- no change needed.

#### 9c: effects/mod.rs token creation

**File**: `/home/airbaggie/scutemob/crates/engine/src/effects/mod.rs`
**Action**: Verify `cast_alt_cost: None` is set in token creation (line 2455). Already handled -- no change needed.

### Step 10: TUI stack_view.rs

**File**: `/home/airbaggie/scutemob/tools/tui/src/play/panels/stack_view.rs`
**Action**: Add match arm for `StackObjectKind::ImpendingCounterTrigger`.
**Pattern**: Follow `StackObjectKind::SuspendCounterTrigger` (line 68).

```rust
StackObjectKind::ImpendingCounterTrigger { impending_permanent, .. } => {
    ("Impending tick: ".to_string(), Some(*impending_permanent))
}
```

### Step 11: Replay Viewer view_model.rs

**File**: `/home/airbaggie/scutemob/tools/replay-viewer/src/view_model.rs`
**Action**: Add match arms for new types in `stack_kind_info` and `keyword_display_name`.

#### 11a: stack_kind_info

After `StackObjectKind::BlitzSacrificeTrigger` (line 519):
```rust
StackObjectKind::ImpendingCounterTrigger { impending_permanent, .. } => {
    ("impending_counter_trigger", Some(*impending_permanent))
}
```

#### 11b: keyword_display_name

After `KeywordAbility::Prototype => "Prototype".to_string(),` (line 755):
```rust
KeywordAbility::Impending => "Impending".to_string(),
```

### Step 12: helpers.rs Prelude

**File**: `/home/airbaggie/scutemob/crates/engine/src/cards/helpers.rs`
**Action**: Verify that `AltCostKind` and any new types needed by card definitions are exported. Card definitions for impending cards will need `AbilityDefinition::Impending { cost, count }` which uses `ManaCost` (already exported) and `u32` (primitive). No new exports needed in helpers.rs.

### Step 13: Unit Tests

**File**: `/home/airbaggie/scutemob/crates/engine/tests/impending.rs` (new file)
**Tests to write**:

#### test_impending_cast_basic
- **CR**: 702.176a
- **What**: Cast a creature spell for its impending cost. Verify: (a) correct mana deducted, (b) on resolution the permanent enters with N time counters, (c) layer calculation shows it is NOT a creature (no `CardType::Creature` in characteristics), (d) it IS an enchantment (or whatever its other types are).

#### test_impending_counter_removal_end_step
- **CR**: 702.176a (fourth ability)
- **What**: Set up a state with an impending permanent on the battlefield with time counters. Advance to the controller's end step. Verify the counter-removal trigger fires and removes one time counter.

#### test_impending_becomes_creature_last_counter
- **CR**: 702.176a
- **What**: Set up a permanent with `cast_alt_cost: Some(AltCostKind::Impending)` and 1 time counter. Remove the counter. Verify `calculate_characteristics` now includes `CardType::Creature` and the creature's P/T are correct.

#### test_impending_not_a_creature_multiple_counters
- **CR**: 702.176a (third ability)
- **What**: Permanent with 3 time counters. After one end step tick (2 counters remain), verify it is still not a creature. After two more ticks (0 counters), verify it IS a creature.

#### test_impending_cast_normal_cost
- **CR**: 702.176a (negative test)
- **What**: Cast the same card for its normal mana cost (no alt_cost). Verify: (a) no time counters, (b) it IS a creature immediately, (c) no end-step counter removal triggers fire.

#### test_impending_stifle_counter_removal
- **CR**: 702.176a + CR 603.4
- **What**: Counter-removal trigger is a triggered ability on the stack. If countered (conceptually -- test by removing the trigger from the stack before resolution), the time counter is NOT removed. Verify permanent retains its time counter.

#### test_impending_intervening_if
- **CR**: 603.4
- **What**: Queue the counter-removal trigger, then remove all time counters from the permanent before the trigger resolves (e.g., via direct state manipulation). Verify the trigger does nothing at resolution.

#### test_impending_zone_change_resets
- **CR**: 400.7
- **What**: A permanent with `cast_alt_cost: Some(AltCostKind::Impending)` and time counters leaves the battlefield and re-enters. Verify the new object has `cast_alt_cost: None`, no time counters, and IS a creature.

#### test_impending_copy_no_counters
- **CR**: Ruling 2024-09-20
- **What**: An object enters as a copy of an impending permanent. Verify the copy has no time counters, `cast_alt_cost: None`, and IS a creature.

#### test_impending_alt_cost_mutual_exclusion
- **CR**: 118.9a
- **What**: Attempt to cast with both `alt_cost: Some(AltCostKind::Impending)` and flashback. Verify the engine rejects the command with an appropriate error.

#### test_impending_multiplayer_controller_end_step
- **CR**: 702.176a ("your end step")
- **What**: In a 4-player game, player B has an impending permanent. Advance through player A's end step -- verify no trigger fires for B's permanent. Advance to player B's end step -- verify the trigger fires.

**Pattern**: Follow tests for Dash/Blitz in `crates/engine/tests/` for alt-cost casting flow; follow tests for Suspend in `crates/engine/tests/suspend.rs` for time counter mechanics.

### Step 14: Card Definition (later phase)

**Suggested card**: Overlord of the Hauntwoods
- Type: Enchantment Creature -- Avatar Horror
- Mana cost: {3}{G}{G}
- Impending 4--{1}{G}{G}
- P/T: 6/5
- Also has an ETB/attack trigger (create a land token) which can be deferred for the impending-specific test card.

**Simpler alternative**: Create a minimal test-only card definition for the unit tests:
```rust
CardDefinition {
    card_id: CardId("test-impending-creature".into()),
    name: "Test Impending Creature".into(),
    mana_cost: Some(ManaCost { generic: 3, green: 2, ..Default::default() }),
    types: TypeLine {
        card_types: ordset![CardType::Enchantment, CardType::Creature],
        ..Default::default()
    },
    oracle_text: "Impending 4--{1}{G}{G}".into(),
    abilities: vec![
        AbilityDefinition::Keyword(KeywordAbility::Impending),
        AbilityDefinition::Impending {
            cost: ManaCost { generic: 1, green: 2, ..Default::default() },
            count: 4,
        },
    ],
    power: Some(6),
    toughness: Some(5),
}
```

**Card lookup**: use `card-definition-author` agent for Overlord of the Hauntwoods.

### Step 15: Game Script (later phase)

**Suggested scenario**: Cast Overlord of the Hauntwoods for its impending cost. Verify it enters as a non-creature enchantment with 4 time counters. Advance through 4 end steps to remove all counters. Verify it becomes a 6/5 creature.

**Subsystem directory**: `test-data/generated-scripts/stack/` (casting alt cost pattern)

## Interactions to Watch

1. **Layer system interaction**: The Layer 4 type removal must be applied AFTER Changeling (which adds all creature types in Layer 4). If a creature with both Changeling and Impending is cast for impending cost, the Changeling subtypes are added first, then the Creature card type is removed. The subtypes persist but are mechanically meaningless while it's not a creature. When counters are gone, the Creature type returns and the Changeling subtypes apply normally.

2. **Bestow interaction**: Bestow also changes creature type (removes it on the stack for bestowed Auras). The two shouldn't interact -- Bestow is a stack modification and Impending is a battlefield type-removal. But verify that `is_bestowed` and `cast_alt_cost == Impending` are mutually exclusive (they should be, since both are alternative costs under CR 118.9a).

3. **Humility interaction**: If Humility removes all abilities from an impending permanent, what happens? The `cast_alt_cost` field is NOT an ability -- it's a game-state marker. The Layer 4 inline check uses `cast_alt_cost` (a field on GameObject, not an ability), so Humility does not affect it. The permanent would still be "not a creature" while it has time counters, even under Humility. This is correct per the rules -- the static ability that makes it "not a creature" is granted by the impending keyword, but the check uses the `cast_alt_cost` marker, which persists independently. However: after Humility removes all keywords (Layer 6), the Impending keyword is gone. But our implementation checks `cast_alt_cost`, not the keyword. This is intentional -- per CR 702.176a, the third ability ("it's not a creature") is a static ability of the PERMANENT, not a keyword ability that can be removed. It persists as long as both conditions are met.

4. **SBA interaction**: While not a creature, the permanent's P/T is irrelevant (SBAs 704.5f/g only check creatures). The permanent is an enchantment (or artifact, etc.) and follows those rules instead.

5. **Combat interaction**: While not a creature, the permanent cannot attack, block, or be targeted by "target creature" effects. It CAN be targeted by "target enchantment" or "target permanent" effects.

6. **Proliferate interaction**: Proliferate can add time counters to the impending permanent, keeping it in non-creature state longer. This is intentional per the rules.

7. **Clockspinning / Vampire Hexmage**: Can remove all time counters at once, immediately making the permanent a creature. This works correctly with our implementation (Layer 4 check sees 0 time counters => does not remove Creature type).

## Files Modified Summary

| File | Action |
|------|--------|
| `crates/engine/src/state/types.rs` | Add `AltCostKind::Impending`, `KeywordAbility::Impending` |
| `crates/engine/src/cards/card_definition.rs` | Add `AbilityDefinition::Impending { cost, count }` |
| `crates/engine/src/state/stack.rs` | Add `was_impended: bool` to `StackObject`, `ImpendingCounterTrigger` to `StackObjectKind` |
| `crates/engine/src/state/stubs.rs` | Add `PendingTriggerKind::ImpendingCounter` |
| `crates/engine/src/state/hash.rs` | Add hash arms for all new enum variants + `was_impended` field |
| `crates/engine/src/rules/casting.rs` | Wire impending as alt cost, mutual exclusion checks, cost/count helpers |
| `crates/engine/src/rules/resolution.rs` | Transfer `was_impended` to `cast_alt_cost`, add time counters at ETB, resolution handler for `ImpendingCounterTrigger` |
| `crates/engine/src/rules/layers.rs` | Inline Layer 4 type removal in `calculate_characteristics` |
| `crates/engine/src/rules/turn_actions.rs` | End step scanning for impending permanents |
| `crates/engine/src/rules/abilities.rs` | Flush handler for `PendingTriggerKind::ImpendingCounter` |
| `tools/tui/src/play/panels/stack_view.rs` | Add match arm for `ImpendingCounterTrigger` |
| `tools/replay-viewer/src/view_model.rs` | Add match arms for new `StackObjectKind` and `KeywordAbility` |
| `crates/engine/tests/impending.rs` | New test file with 11 unit tests |
