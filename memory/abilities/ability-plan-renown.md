# Ability Plan: Renown

**Generated**: 2026-03-01
**CR**: 702.112
**Priority**: P4
**Similar abilities studied**: Ingest (CR 702.115, custom StackObjectKind combat damage trigger), Dethrone (CR 702.105, +1/+1 counter placement on attack), Decayed (CR 702.147, boolean flag on GameObject)

## CR Rule Text

702.112. Renown

702.112a Renown is a triggered ability. "Renown N" means "When this creature deals combat damage to a player, if it isn't renowned, put N +1/+1 counters on it and it becomes renowned."

702.112b Renowned is a designation that has no rules meaning other than to act as a marker that the renown ability and other spells and abilities can identify. Only permanents can be or become renowned. Once a permanent becomes renowned, it stays renowned until it leaves the battlefield. Renowned is neither an ability nor part of the permanent's copiable values.

702.112c If a creature has multiple instances of renown, each triggers separately. The first such ability to resolve will cause the creature to become renowned, and subsequent abilities will have no effect. (See rule 603.4)

## Key Edge Cases

- **Intervening-if clause (CR 603.4 + CR 702.112a)**: The trigger checks "if it isn't renowned" both at trigger time AND at resolution time. If the creature becomes renowned between trigger and resolution (e.g., from the first of two renown triggers resolving), subsequent renown triggers on the stack are countered by the intervening-if failing at resolution.
- **Creature leaves battlefield before resolution (Ruling 2015-06-22)**: "If a renown ability triggers, but the creature leaves the battlefield before that ability resolves, the creature doesn't become renowned." The resolution must verify the source object is still on the battlefield.
- **Only players, not planeswalkers (Ruling 2015-06-22)**: "Renown won't trigger when a creature deals combat damage to a planeswalker or another creature. It also won't trigger when a creature deals noncombat damage to a player." This is inherent in the `CombatDamageTarget::Player` guard.
- **Redirected damage to controller (Ruling 2015-06-22)**: "If a creature with renown deals combat damage to its controller because that damage was redirected, renown will trigger." The trigger fires on combat damage to ANY player, not just opponents.
- **Renowned is NOT a copiable value (CR 702.112b)**: A copy of a renowned creature is NOT renowned. The `is_renowned` flag must NOT be part of the copiable characteristics. It is a designation on the game object, like the `decayed_sacrifice_at_eoc` flag -- not in `Characteristics`.
- **Renowned resets on zone change (CR 702.112b)**: "it stays renowned until it leaves the battlefield." When the object changes zones, `is_renowned` resets to false (CR 400.7).
- **Multiple instances (CR 702.112c)**: Each instance triggers separately. The first to resolve sets `is_renowned` and places counters. Subsequent triggers check the intervening-if at resolution, find the creature is now renowned, and do nothing (CR 603.4).
- **Renown N carries a value (CR 702.112a)**: Different cards have different N values (Renown 1, Renown 2, etc.). The keyword variant must carry `u32`. The trigger must convey the N value to the resolution handler.
- **Multiplayer**: Renown triggers whenever the creature deals combat damage to any player (not just opponents). In Commander, a 4-player game means any of the 3 opponents can be hit.

## Current State (from ability-wip.md)

- [ ] Step 1: Enum variant
- [ ] Step 2: Rule enforcement (trigger dispatch)
- [ ] Step 3: Trigger wiring (StackObjectKind, resolution)
- [ ] Step 4: Unit tests
- [ ] Step 5: Card definition
- [ ] Step 6: Game script

## Implementation Steps

### Step 1: Enum Variant

**File**: `crates/engine/src/state/types.rs`
**Action**: Add `KeywordAbility::Renown(u32)` variant after `Ingest` (line ~639)
**Discriminant**: 81 (per batch assignment: Flanking=76, Bushido=77, Rampage=78, Provoke=79, Afflict=80, Renown=81)
**Pattern**: Follow `Annihilator(u32)` at line 297 or `Modular(u32)` at line 503 -- keyword with associated value

```rust
/// CR 702.112: Renown N -- triggered ability.
/// "When this creature deals combat damage to a player, if it isn't renowned,
/// put N +1/+1 counters on it and it becomes renowned."
/// CR 702.112c: Multiple instances trigger separately.
///
/// Renowned is a designation tracked as `is_renowned` on `GameObject`
/// (CR 702.112b). Not a copiable value. Resets on zone change (CR 400.7).
Renown(u32),
```

**Hash**: Add to `state/hash.rs` in the `KeywordAbility` `HashInto` impl -- discriminant 81, hash the u32 value:
```rust
// Renown (discriminant 81) -- CR 702.112
KeywordAbility::Renown(n) => {
    81u8.hash_into(hasher);
    n.hash_into(hasher);
}
```

**Match arms**: Grep for exhaustive `KeywordAbility` match expressions and add the new arm. Known locations:
- `state/hash.rs` (HashInto for KeywordAbility)
- `rules/combat.rs` (evasion checks -- Renown is not an evasion keyword, so add a no-op or wildcard)
- `state/builder.rs` (keyword-to-triggered-ability translation -- NOT needed for Renown since we use custom dispatch, but builder.rs might have a match block)
- `cards/helpers.rs` (re-export if needed)

### Step 2: GameObject `is_renowned` Flag

**File**: `crates/engine/src/state/game_object.rs`
**Action**: Add `pub is_renowned: bool` field to `GameObject` after `decayed_sacrifice_at_eoc` (line ~408)
**CR**: 702.112b -- "Renowned is a designation... Only permanents can be or become renowned. Once a permanent becomes renowned, it stays renowned until it leaves the battlefield."

```rust
/// CR 702.112b: Renowned designation. Tracked as a boolean flag on the
/// permanent. Once set by a resolved Renown trigger, stays true until
/// the permanent leaves the battlefield (CR 400.7 resets it).
///
/// NOT a copiable value (CR 702.112b) -- copies start non-renowned.
/// NOT an ability -- persists even if abilities are removed (e.g., Humility).
#[serde(default)]
pub is_renowned: bool,
```

**Initialization sites** (all must set `is_renowned: false`):
1. `state/mod.rs` -- `move_object_to_zone` (TWO sites, lines ~286 and ~375) -- CR 400.7 reset
2. `state/builder.rs` -- `build()` object construction (line ~741)
3. `effects/mod.rs` -- token creation (line ~2458)
4. `rules/resolution.rs` -- myriad token creation (line ~1226)

**Hash**: Add to `state/hash.rs` in the `GameObject` `HashInto` impl, after `decayed_sacrifice_at_eoc`:
```rust
// Renowned (CR 702.112b) -- designation flag
self.is_renowned.hash_into(hasher);
```

### Step 3: PendingTrigger Fields

**File**: `crates/engine/src/state/stubs.rs`
**Action**: Add two fields to `PendingTrigger` after `ingest_target_player` (line ~242):

```rust
/// CR 702.112a: If true, this pending trigger is a Renown trigger.
///
/// When flushed to the stack, creates a `StackObjectKind::RenownTrigger`
/// instead of the normal `StackObjectKind::TriggeredAbility`.
/// The `renown_n` carries the N value (number of +1/+1 counters).
#[serde(default)]
pub is_renown_trigger: bool,
/// CR 702.112a: The N value from "Renown N" -- how many +1/+1 counters
/// to place on the creature when the trigger resolves.
///
/// Only meaningful when `is_renown_trigger` is true.
#[serde(default)]
pub renown_n: Option<u32>,
```

**Hash**: Add to `state/hash.rs` in the `PendingTrigger` `HashInto` impl:
```rust
// Renown (CR 702.112a) — trigger flag and counter count
self.is_renown_trigger.hash_into(hasher);
self.renown_n.hash_into(hasher);
```

**Also update all PendingTrigger construction sites** to include `is_renown_trigger: false` and `renown_n: None`. Grep for `PendingTrigger {` to find all sites. There are many (~15-20 sites across abilities.rs, turn_actions.rs, resolution.rs, miracle.rs, effects/mod.rs, lands.rs, replacement.rs).

### Step 4: StackObjectKind Variant

**File**: `crates/engine/src/state/stack.rs`
**Action**: Add `RenownTrigger` variant after `IngestTrigger` (line ~427):

```rust
/// CR 702.112a: Renown N triggered ability.
///
/// Resolution steps:
/// 1. Re-check intervening-if: is the source still on the battlefield
///    and NOT renowned? (CR 603.4)
/// 2. If yes, put N +1/+1 counters on the source creature.
/// 3. Set `is_renowned = true` on the source (CR 702.112b).
/// 4. If the source left the battlefield, do nothing (ruling 2015-06-22).
RenownTrigger {
    source_object: ObjectId,
    renown_n: u32,
},
```

**Hash**: Add to `state/hash.rs` in the `StackObjectKind` `HashInto` impl -- discriminant 19:
```rust
// RenownTrigger (discriminant 19) -- CR 702.112a
StackObjectKind::RenownTrigger {
    source_object,
    renown_n,
} => {
    19u8.hash_into(hasher);
    source_object.hash_into(hasher);
    renown_n.hash_into(hasher);
}
```

**TUI stack_view.rs**: Add arm to the exhaustive match in `tools/tui/src/play/panels/stack_view.rs` (after IngestTrigger arm at line ~81):
```rust
StackObjectKind::RenownTrigger { source_object, .. } => {
    ("Renown: ".to_string(), Some(*source_object))
}
```

**Replay viewer view_model.rs**: Add arm to the kind-mapping function in `tools/replay-viewer/src/view_model.rs` (after IngestTrigger arm at line ~474):
```rust
StackObjectKind::RenownTrigger { source_object, .. } => {
    ("renown_trigger", Some(*source_object))
}
```

### Step 5: Trigger Dispatch (abilities.rs)

**File**: `crates/engine/src/rules/abilities.rs`
**Action**: Add Renown trigger dispatch in the `CombatDamageDealt` handler, AFTER the Ingest block (line ~1837)
**CR**: 702.112a -- "When this creature deals combat damage to a player, if it isn't renowned, put N +1/+1 counters on it and it becomes renowned."
**Pattern**: Follow the Ingest dispatch block at lines 1757-1837

```rust
// CR 702.112a: Renown N -- "When this creature deals combat
// damage to a player, if it isn't renowned, put N +1/+1
// counters on it and it becomes renowned."
// CR 702.112c: Multiple instances trigger separately.
if let Some(obj) = state.objects.get(&assignment.source) {
    if obj.zone == ZoneId::Battlefield
        && !obj.is_renowned  // CR 603.4: intervening-if at trigger time
    {
        // Count Renown instances and collect N values from
        // card definition (CR 702.112c: each triggers separately).
        let renown_values: Vec<u32> = obj
            .card_id
            .as_ref()
            .and_then(|cid| state.card_registry.get(cid.clone()))
            .map(|def| {
                def.abilities
                    .iter()
                    .filter_map(|a| match a {
                        AbilityDefinition::Keyword(KeywordAbility::Renown(n)) => Some(*n),
                        _ => None,
                    })
                    .collect()
            })
            .unwrap_or_else(|| {
                // Fallback: check keywords on the object itself
                obj.characteristics
                    .keywords
                    .iter()
                    .filter_map(|kw| match kw {
                        KeywordAbility::Renown(n) => Some(*n),
                        _ => None,
                    })
                    .collect()
            });

        let controller = obj.controller;
        let source_id = obj.id;
        for renown_n in renown_values {
            triggers.push(PendingTrigger {
                source: source_id,
                ability_index: 0, // unused for renown triggers
                controller,
                triggering_event: Some(TriggerEvent::SelfDealsCombatDamageToPlayer),
                entering_object_id: None,
                targeting_stack_id: None,
                triggering_player: None,
                exalted_attacker_id: None,
                defending_player_id: None,
                is_evoke_sacrifice: false,
                is_madness_trigger: false,
                madness_exiled_card: None,
                madness_cost: None,
                is_miracle_trigger: false,
                miracle_revealed_card: None,
                miracle_cost: None,
                is_unearth_trigger: false,
                is_exploit_trigger: false,
                is_modular_trigger: false,
                modular_counter_count: None,
                is_evolve_trigger: false,
                evolve_entering_creature: None,
                is_myriad_trigger: false,
                is_suspend_counter_trigger: false,
                is_suspend_cast_trigger: false,
                suspend_card_id: None,
                is_hideaway_trigger: false,
                hideaway_count: None,
                is_partner_with_trigger: false,
                partner_with_name: None,
                is_ingest_trigger: false,
                ingest_target_player: None,
                is_renown_trigger: true,
                renown_n: Some(renown_n),
            });
        }
    }
}
```

### Step 6: Flush to Stack (abilities.rs)

**File**: `crates/engine/src/rules/abilities.rs`
**Action**: Add an `else if trigger.is_renown_trigger` branch in `flush_pending_triggers`, after the `is_ingest_trigger` branch (line ~2218)
**Pattern**: Follow the IngestTrigger flush at lines 2218-2226

```rust
} else if trigger.is_renown_trigger {
    // CR 702.112a: Renown trigger -- "When this creature deals combat
    // damage to a player, if it isn't renowned, put N +1/+1 counters
    // on it and it becomes renowned."
    StackObjectKind::RenownTrigger {
        source_object: trigger.source,
        renown_n: trigger.renown_n.unwrap_or(1),
    }
}
```

### Step 7: Resolution (resolution.rs)

**File**: `crates/engine/src/rules/resolution.rs`
**Action**: Add a resolution arm for `StackObjectKind::RenownTrigger` before the catch-all counter arm (line ~1779)
**CR**: 702.112a (counter placement + renowned designation), 702.112b (renowned persists until zone change), 603.4 (intervening-if re-check at resolution)

```rust
// CR 702.112a: Renown trigger resolves -- re-check the intervening-if
// (CR 603.4) and place N +1/+1 counters on the source creature, then
// set it as renowned (CR 702.112b).
//
// Ruling 2015-06-22: "If a renown ability triggers, but the creature
// leaves the battlefield before that ability resolves, the creature
// doesn't become renowned."
StackObjectKind::RenownTrigger {
    source_object,
    renown_n,
} => {
    let controller = stack_obj.controller;

    // CR 603.4: Re-check intervening-if at resolution time.
    // Source must still be on the battlefield AND not yet renowned.
    let should_resolve = state
        .objects
        .get(&source_object)
        .map(|obj| obj.zone == ZoneId::Battlefield && !obj.is_renowned)
        .unwrap_or(false);

    if should_resolve {
        // CR 702.112a: Place N +1/+1 counters.
        if let Some(obj) = state.objects.get_mut(&source_object) {
            let current = obj
                .counters
                .get(&CounterType::PlusOnePlusOne)
                .copied()
                .unwrap_or(0);
            obj.counters = obj
                .counters
                .update(CounterType::PlusOnePlusOne, current + renown_n);
            // CR 702.112b: Set the renowned designation.
            obj.is_renowned = true;
        }

        events.push(GameEvent::AbilityResolved {
            controller,
            stack_object_id: stack_obj.id,
        });
    } else {
        // CR 603.4: Intervening-if failed at resolution -- ability does nothing.
        events.push(GameEvent::AbilityResolved {
            controller,
            stack_object_id: stack_obj.id,
        });
    }
}
```

**Also add `RenownTrigger` to the counter arm** at line ~1779 (the `StackObjectKind::XxxTrigger { .. } => { // Countering ... }` catch-all):
```rust
| StackObjectKind::RenownTrigger { .. }
```

### Step 8: Unit Tests

**File**: `crates/engine/tests/renown.rs`
**Tests to write**:

1. `test_702_112a_renown_basic_counters_and_renowned` -- A creature with Renown 1 deals unblocked combat damage to a player; trigger fires, resolves, creature gets 1 +1/+1 counter and becomes renowned. Verifies `is_renowned == true` and counter count.
   **CR**: 702.112a

2. `test_702_112a_renown_n2_places_two_counters` -- A creature with Renown 2 deals combat damage; gets 2 +1/+1 counters. Verifies the N value is respected.
   **CR**: 702.112a

3. `test_702_112a_renown_no_trigger_when_already_renowned` -- A creature that is already renowned deals combat damage to a player. No trigger fires (intervening-if fails at trigger time).
   **CR**: 702.112a, 603.4

4. `test_702_112b_renown_resets_on_zone_change` -- A renowned creature leaves and re-enters the battlefield. After re-entry, `is_renowned` is false. (Use blink/flicker pattern or manual zone move.)
   **CR**: 702.112b, 400.7

5. `test_702_112c_renown_multiple_instances_first_resolves` -- A creature with two Keyword(Renown(1)) entries deals combat damage. Two triggers fire. The first to resolve places 1 counter and sets renowned. The second resolves but the intervening-if fails (already renowned) -- no additional counters.
   **CR**: 702.112c, 603.4

6. `test_702_112_renown_creature_leaves_before_resolution` -- A creature with Renown triggers, but is removed from the battlefield before the trigger resolves. The trigger resolves with no effect; the creature does not become renowned.
   **CR**: Ruling 2015-06-22, 603.4

7. `test_702_112a_renown_multiplayer_specific_player` -- In a 4-player game, creature deals combat damage to P3. Renown triggers correctly for the attacker. (Tests multiplayer context.)
   **CR**: 702.112a

**Pattern**: Follow tests in `crates/engine/tests/ingest.rs` for the combat damage trigger pattern (DeclareAttackers -> DeclareBlockers -> pass_all for damage -> pass_all for trigger resolution).

### Step 9: Card Definition (later phase)

**Suggested card**: Topan Freeblade ({1}{W}, 2/2, Vigilance + Renown 1)
- Simple, clean test card with an additional keyword (Vigilance) to verify keyword coexistence
- Oracle text: "Vigilance / Renown 1"
- File: `crates/engine/src/cards/defs/topan_freeblade.rs`

**Alternative card**: Consul's Lieutenant ({W}{W}, 2/1, First strike + Renown 1) -- also has a conditional attack trigger ("whenever this attacks, if renowned, other attackers get +1/+1"), but that secondary ability is more complex.

**Card lookup reference**:
- Topan Freeblade: {1}{W}, Creature -- Human Soldier, 2/2, Vigilance, Renown 1
- Rhox Maulers: {4}{G}, Creature -- Rhino Soldier, 4/4, Trample, Renown 2 (good for testing Renown N > 1)

### Step 10: Game Script (later phase)

**Suggested scenario**: "Topan Freeblade attacks unblocked, triggers Renown 1, gets +1/+1 counter, becomes renowned. Second attack does not trigger Renown again."
**Subsystem directory**: `test-data/generated-scripts/combat/`
**Sequence number**: next available in combat directory

## Interactions to Watch

- **Copy effects (CR 702.112b)**: Renowned is NOT a copiable value. A Clone of a renowned creature starts as non-renowned. The `is_renowned` flag is on `GameObject`, not in `Characteristics`, so copy effects that clone `Characteristics` will correctly not copy it.
- **Humility / ability removal**: Removing all abilities (Layer 6) removes the Renown keyword, preventing future triggers. However, if the creature is already renowned, it STAYS renowned -- renowned is a designation, not an ability (CR 702.112b). The `is_renowned` flag persists independently of abilities.
- **"Whenever a creature becomes renowned" triggers**: Some cards trigger when a creature becomes renowned. This is NOT implemented in the plan; it would require a new `GameEvent::CreatureBecameRenowned` event. Defer to card-specific implementation.
- **Ingest + Renown on same creature**: Both trigger on `SelfDealsCombatDamageToPlayer`. The `CombatDamageDealt` handler processes Ingest first, then Renown. Both fire independently. APNAP ordering applies (same controller, so controller chooses stack order via the pending trigger flush).
- **Prevention effects**: If combat damage is fully prevented (amount == 0), the `CombatDamageDealt` handler skips the assignment entirely (line 1745-1746). Renown does not trigger.
- **First strike / double strike**: A creature with first strike + renown deals damage in the first strike step. Renown triggers. If the trigger resolves before regular damage, the creature is now renowned and won't trigger again on regular damage. With double strike, the first damage step fires renown; the second damage step finds the creature already renowned (if the trigger resolved).

## Design Decision: Custom StackObjectKind vs Standard TriggeredAbility

**Chosen**: Custom `StackObjectKind::RenownTrigger` (like Ingest)

**Rationale**: Renown's resolution needs to both (1) place N counters AND (2) set the `is_renowned` designation flag. The standard `TriggeredAbility` resolution runs an `Effect`, but there is no `Effect::BecomeRenowned` variant, and adding one would be overly specialized. The custom StackObjectKind keeps the logic self-contained in `resolution.rs`, avoids adding a new Effect variant, and follows the established Ingest pattern for combat damage triggers with specialized resolution behavior.

**Alternative considered**: Standard TriggeredAbility with `InterveningIf::SourceNotRenowned` + `Effect::Sequence([AddCounter, BecomeRenowned])`. Rejected because it requires both a new `InterveningIf` variant AND a new `Effect` variant, adding more surface area for less clarity.
