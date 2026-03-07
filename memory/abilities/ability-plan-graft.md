# Ability Plan: Graft

**Generated**: 2026-03-06
**CR**: 702.58
**Priority**: P4
**Similar abilities studied**: Evolve (CR 702.100) in `types.rs:564-570`, `abilities.rs:2253-2401`, `resolution.rs:2112-2195`, `tests/evolve.rs`; Modular (CR 702.43) ETB counters in `resolution.rs:596-638`

## CR Rule Text

702.58. Graft

702.58a Graft represents both a static ability and a triggered ability. "Graft N" means "This permanent enters with N +1/+1 counters on it" and "Whenever another creature enters, if this permanent has a +1/+1 counter on it, you may move a +1/+1 counter from this permanent onto that creature."

702.58b If a permanent has multiple instances of graft, each one works separately.

## Key Edge Cases

- **Two components**: Graft is BOTH a static ability (ETB with N +1/+1 counters) AND a triggered ability (move counter when another creature enters). Both must be implemented.
- **Intervening-if (CR 603.4)**: "if this permanent has a +1/+1 counter on it" is checked at trigger time AND at resolution time. If the source has no +1/+1 counters at either check, the trigger does not fire / does not resolve.
- **"You may" -- optional trigger**: The counter move is optional ("you may move"). The controller chooses at resolution time whether to move the counter. This means the trigger goes on the stack, but at resolution the controller can decline.
- **"Another creature"**: The trigger fires for any creature entering the battlefield, not just creatures controlled by the graft permanent's controller. This is different from Evolve (which only fires for "a creature you control").
- **Non-creature permanents do NOT trigger graft**: Only creatures entering the battlefield trigger it. A land with Graft (Llanowar Reborn) entering does NOT trigger its own Graft -- it's not a creature. But another creature entering DOES trigger Llanowar Reborn's Graft.
- **Multiple instances (CR 702.58b)**: Each instance triggers separately, and each ETB places N counters independently. A creature with Graft 2 and Graft 3 enters with 5 +1/+1 counters and gets two separate triggers when another creature enters.
- **Counter movement direction**: The counter moves FROM the graft permanent TO the entering creature. This is a removal + addition, not a transfer effect. The graft permanent loses a +1/+1 counter and the entering creature gains one.
- **Land with Graft (Llanowar Reborn)**: A land enters with +1/+1 counters (which do nothing for non-creatures). Its triggered ability fires for other creatures entering. ETB counter placement must happen in BOTH `resolution.rs` AND `lands.rs` (per gotchas-infra.md ETB site pattern).
- **0/0 creatures with Graft**: Most Graft creatures are base 0/0. If all +1/+1 counters are moved away, SBAs will destroy them (CR 704.5f). This is expected behavior.
- **Multiplayer**: Graft triggers for ANY player's creature entering, not just the controller's. This differs from Evolve.

## Current State (from ability-wip.md)

- [ ] Step 1: Enum variant
- [ ] Step 2: Rule enforcement (ETB counters + trigger)
- [ ] Step 3: Trigger wiring
- [ ] Step 4: Unit tests
- [ ] Step 5: Card definition
- [ ] Step 6: Game script

## Implementation Steps

### Step 1: Enum Variant

**File**: `crates/engine/src/state/types.rs`
**Action**: Add `KeywordAbility::Graft(u32)` variant before the closing `}` of the enum, after `Phasing` (line 1095).
**Discriminant**: KW 119 (next available after Phasing = 118).
**Pattern**: Follow `KeywordAbility::Modular(u32)` at line 563 -- parameterized keyword with N value.
**Doc comment**:
```
/// CR 702.58: Graft N -- "This permanent enters with N +1/+1 counters on it"
/// and "Whenever another creature enters, if this permanent has a +1/+1
/// counter on it, you may move a +1/+1 counter from this permanent onto
/// that creature."
///
/// Represents both a static ability (ETB counters) and a triggered ability
/// (counter transfer). Each instance works separately (CR 702.58b).
///
/// Discriminant 119.
Graft(u32),
```

**Hash**: Add to `state/hash.rs` `HashInto` impl for `KeywordAbility`. Find the match arm block (search for `Phasing =>`) and add after it:
```rust
KeywordAbility::Graft(n) => {
    119u8.hash_into(hasher);
    n.hash_into(hasher);
}
```

**Match arms**: Grep for exhaustive `KeywordAbility` match expressions across the codebase and add the new arm. Key locations:
- `state/hash.rs` (HashInto impl)
- `state/builder.rs` (keyword processing -- add Graft trigger auto-generation here, see Step 3)
- Any display/format impls

### Step 2: Rule Enforcement -- ETB Counters (Static Ability)

**File 1**: `crates/engine/src/rules/resolution.rs`
**Action**: Add Graft ETB counter placement in the same section as Modular (after line ~638). Sum all Graft N values from the card definition and place that many +1/+1 counters on the entering permanent.
**CR**: 702.58a -- "This permanent enters with N +1/+1 counters on it"
**Pattern**: Follow Modular ETB counter placement at `resolution.rs:596-638` exactly:
```rust
// CR 702.58a: Graft N -- "This permanent enters with N +1/+1 counters on it."
// CR 702.58b: Multiple instances each work separately; their N values sum.
{
    let graft_total: u32 = card_id
        .as_ref()
        .and_then(|cid| registry.get(cid.clone()))
        .map(|def| {
            def.abilities
                .iter()
                .filter_map(|a| match a {
                    AbilityDefinition::Keyword(KeywordAbility::Graft(n)) => Some(*n),
                    _ => None,
                })
                .sum()
        })
        .unwrap_or(0);

    if graft_total > 0 {
        if let Some(obj) = state.objects.get_mut(&new_id) {
            let current = obj
                .counters
                .get(&CounterType::PlusOnePlusOne)
                .copied()
                .unwrap_or(0);
            obj.counters = obj
                .counters
                .update(CounterType::PlusOnePlusOne, current + graft_total);
        }
        events.push(GameEvent::CounterAdded {
            object_id: new_id,
            counter: CounterType::PlusOnePlusOne,
            count: graft_total,
        });
    }
}
```

**File 2**: `crates/engine/src/rules/lands.rs`
**Action**: Add Graft ETB counter placement for lands (Llanowar Reborn). Place after the existing Vanishing/Fading counter blocks.
**CR**: 702.58a -- same rule, but for lands entering via PlayLand.
**Pattern**: Follow the Vanishing/Fading counter blocks in `lands.rs:115-175`.

### Step 3: Trigger Wiring

#### Step 3a: PendingTriggerKind variant

**File**: `crates/engine/src/state/stubs.rs`
**Action**: Add `Graft` variant to `PendingTriggerKind` enum (after `Recover`, before the comment).
**Pattern**: Follow `Evolve` at line 39.

#### Step 3b: PendingTrigger field for entering creature

**File**: `crates/engine/src/state/stubs.rs`
**Action**: Add `pub graft_entering_creature: Option<ObjectId>` field to `PendingTrigger` struct. This stores the ObjectId of the creature that entered, needed at resolution time.
**Pattern**: Follow `evolve_entering_creature` field at line 194.
**Hash**: Add to `state/hash.rs` HashInto impl for PendingTrigger.

#### Step 3c: StackObjectKind variant

**File**: `crates/engine/src/state/stack.rs`
**Action**: Add `GraftTrigger` variant:
```rust
/// CR 702.58a: Graft triggered ability on the stack.
/// "Whenever another creature enters, if this permanent has a +1/+1 counter
/// on it, you may move a +1/+1 counter from this permanent onto that creature."
///
/// At resolution: re-check intervening-if (source has +1/+1 counter), then
/// controller may move a counter. The entering creature ID is needed so the
/// counter can be placed on it.
///
/// Discriminant 44.
GraftTrigger {
    source_object: ObjectId,
    entering_creature: ObjectId,
},
```
**Hash**: Add to `state/hash.rs` HashInto impl for StackObjectKind:
```rust
// GraftTrigger (discriminant 44) -- CR 702.58a
StackObjectKind::GraftTrigger {
    source_object,
    entering_creature,
} => {
    44u8.hash_into(hasher);
    source_object.hash_into(hasher);
    entering_creature.hash_into(hasher);
}
```

#### Step 3d: TUI stack_view.rs

**File**: `tools/tui/src/play/panels/stack_view.rs`
**Action**: Add match arm for `StackObjectKind::GraftTrigger`:
```rust
StackObjectKind::GraftTrigger { source_object, .. } => {
    ("Graft: ".to_string(), Some(*source_object))
}
```

#### Step 3e: Trigger collection in abilities.rs

**File**: `crates/engine/src/rules/abilities.rs`
**Action**: In the `PermanentEnteredBattlefield` match arm (around line 2004), after the Evolve trigger collection block, add a Graft trigger collection block.

**Key differences from Evolve**:
1. Graft fires for ANY player's creature entering (not just controller's creatures).
2. Graft has an intervening-if: source must have a +1/+1 counter.
3. Graft uses `AnyPermanentEntersBattlefield` trigger event.
4. The entering object must be a creature (non-creatures don't trigger Graft).
5. The graft permanent itself entering does NOT trigger its own Graft ("another creature").

```rust
// CR 702.58a: Graft -- "Whenever another creature enters, if this
// permanent has a +1/+1 counter on it, you may move a +1/+1 counter
// from this permanent onto that creature."
//
// CR 702.58b: Multiple instances each trigger separately.
// Differences from Evolve:
// - Fires for ANY player's creature, not just controller's
// - Has intervening-if: source must have a +1/+1 counter
// - "Another creature" -- source entering does NOT trigger itself
{
    // Check if the entering permanent is a creature
    let entering_is_creature = state
        .objects
        .get(object_id)
        .map(|obj| {
            let chars = crate::rules::layers::calculate_characteristics(state, *object_id)
                .unwrap_or_else(|| obj.characteristics.clone());
            chars.type_line.card_types.contains(&CardType::Creature)
        })
        .unwrap_or(false);

    if entering_is_creature {
        // Find all battlefield permanents with Graft that have +1/+1 counters
        // (intervening-if check at trigger time)
        let graft_sources: Vec<(ObjectId, PlayerId, u32)> = state
            .objects
            .iter()
            .filter(|(id, obj)| {
                obj.zone == ZoneId::Battlefield
                    && **id != *object_id  // "another creature"
                    && obj.is_phased_in()
                    && obj.counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0) > 0
                    && {
                        let chars = crate::rules::layers::calculate_characteristics(state, **id)
                            .unwrap_or_else(|| obj.characteristics.clone());
                        chars.keywords.iter().any(|kw| matches!(kw, KeywordAbility::Graft(_)))
                    }
            })
            .map(|(id, obj)| {
                let chars = crate::rules::layers::calculate_characteristics(state, *id)
                    .unwrap_or_else(|| obj.characteristics.clone());
                let graft_count = chars
                    .keywords
                    .iter()
                    .filter(|kw| matches!(kw, KeywordAbility::Graft(_)))
                    .count() as u32;
                (*id, obj.controller, graft_count)
            })
            .collect();

        for (graft_id, controller, graft_count) in graft_sources {
            for _ in 0..graft_count {
                triggers.push(PendingTrigger {
                    source: graft_id,
                    ability_index: 0,
                    controller,
                    kind: PendingTriggerKind::Graft,
                    triggering_event: Some(TriggerEvent::AnyPermanentEntersBattlefield),
                    entering_object_id: Some(*object_id),
                    targeting_stack_id: None,
                    triggering_player: None,
                    graft_entering_creature: Some(*object_id),
                    // ... other fields defaulted
                });
            }
        }
    }
}
```

#### Step 3f: Flush pending triggers -- PendingTriggerKind::Graft arm

**File**: `crates/engine/src/rules/abilities.rs`
**Action**: In the `flush_pending_triggers` match on `PendingTriggerKind`, add the `Graft` arm to create a `StackObjectKind::GraftTrigger`:
```rust
PendingTriggerKind::Graft => {
    StackObjectKind::GraftTrigger {
        source_object: trigger.source,
        entering_creature: trigger
            .graft_entering_creature
            .unwrap_or(trigger.source),
    }
}
```

#### Step 3g: Resolution handler

**File**: `crates/engine/src/rules/resolution.rs`
**Action**: Add resolution handler for `StackObjectKind::GraftTrigger`. Place near the `EvolveTrigger` handler (after line ~2195).

Resolution logic:
1. **Re-check intervening-if (CR 603.4)**: Source must still be on the battlefield AND have at least one +1/+1 counter.
2. **"You may"**: This is optional. For now, auto-accept (the engine currently auto-resolves "may" triggers -- consistent with Evolve). If a Command-based choice system exists, use it; otherwise always move the counter (which is the typical desired play).
3. **Move counter**: Remove one +1/+1 counter from source, add one +1/+1 counter to entering creature. Both must still be on the battlefield.
4. **Entering creature must still be on the battlefield**: If it left, the trigger fizzles.

```rust
StackObjectKind::GraftTrigger {
    source_object,
    entering_creature,
} => {
    // CR 702.58a: Graft trigger resolves.
    // CR 603.4: Re-check intervening-if -- source must have a +1/+1 counter.
    let source_has_counter = state
        .objects
        .get(&source_object)
        .map(|obj| {
            obj.zone == ZoneId::Battlefield
                && obj.counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0) > 0
        })
        .unwrap_or(false);

    let target_on_battlefield = state
        .objects
        .get(&entering_creature)
        .map(|obj| obj.zone == ZoneId::Battlefield)
        .unwrap_or(false);

    if source_has_counter && target_on_battlefield {
        // CR 702.58a: "you may move a +1/+1 counter" -- auto-accept for now.
        // Remove one +1/+1 counter from source.
        if let Some(obj) = state.objects.get_mut(&source_object) {
            let current = obj.counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0);
            if current > 1 {
                obj.counters = obj.counters.update(CounterType::PlusOnePlusOne, current - 1);
            } else {
                obj.counters = obj.counters.without(&CounterType::PlusOnePlusOne);
            }
        }
        events.push(GameEvent::CounterRemoved {
            object_id: source_object,
            counter: CounterType::PlusOnePlusOne,
            count: 1,
        });

        // Add one +1/+1 counter to entering creature.
        if let Some(obj) = state.objects.get_mut(&entering_creature) {
            let current = obj.counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0);
            obj.counters = obj.counters.update(CounterType::PlusOnePlusOne, current + 1);
        }
        events.push(GameEvent::CounterAdded {
            object_id: entering_creature,
            counter: CounterType::PlusOnePlusOne,
            count: 1,
        });
    }
    // If condition fails, trigger does nothing (fizzles silently).
}
```

Also add `StackObjectKind::GraftTrigger { .. }` to the non-spell-resolution catch-all arm (around line 4062) that skips post-resolution zone moves.

### Step 4: Unit Tests

**File**: `crates/engine/tests/graft.rs`
**Tests to write**:

1. `test_graft_etb_counters` -- CR 702.58a: A creature with Graft 2 enters with 2 +1/+1 counters. Verify counter count on the battlefield object.

2. `test_graft_trigger_moves_counter` -- CR 702.58a: Graft creature on battlefield, another creature enters, trigger resolves, one +1/+1 counter moves from graft creature to entering creature.

3. `test_graft_trigger_does_not_fire_without_counters` -- CR 702.58a intervening-if: If the graft permanent has no +1/+1 counters, the trigger does not fire when another creature enters.

4. `test_graft_trigger_does_not_fire_for_self` -- CR 702.58a "another creature": The graft creature entering the battlefield does not trigger its own graft.

5. `test_graft_trigger_fires_for_opponents_creatures` -- CR 702.58a: Unlike Evolve, Graft fires for any player's creature entering (including opponents').

6. `test_graft_noncreature_does_not_trigger` -- Entering a noncreature permanent (e.g., an artifact or enchantment) does not trigger Graft.

7. `test_graft_multiple_instances` -- CR 702.58b: A creature with Graft 2 and Graft 3 enters with 5 +1/+1 counters. When another creature enters, two separate triggers fire (one per instance).

8. `test_graft_zero_toughness_after_moving_all_counters` -- After moving all +1/+1 counters away from a 0/0 Graft creature, SBAs destroy it.

9. `test_graft_resolution_recheck_intervening_if` -- CR 603.4: If the source loses its last +1/+1 counter before the trigger resolves (e.g., from a first Graft trigger resolving), the second trigger does nothing.

**Pattern**: Follow `tests/evolve.rs` structure -- same helper functions (`find_object`, `pass_all`, `cast_and_resolve`), same card definition construction pattern, same assertion style.

### Step 5: Card Definition (later phase)

**Suggested card**: Simic Initiate (simplest Graft creature -- {G}, 0/0, Graft 1)
**Card lookup**: use `card-definition-author` agent
**Also consider**: Cytoplast Root-Kin ({2}{G}{G}, 0/0, Graft 4 + bonus ETB + activated ability)

### Step 6: Game Script (later phase)

**Suggested scenario**: Simic Initiate enters with Graft 1 counter, then another creature enters. Graft trigger resolves, counter moves. Simic Initiate dies to SBAs (0 toughness). Verify entering creature gained the counter.
**Subsystem directory**: `test-data/generated-scripts/baseline/` or `test-data/generated-scripts/stack/`

## Interactions to Watch

- **Graft + Modular**: Both place +1/+1 counters at ETB. If a creature somehow has both, both static abilities fire. The triggered abilities are different (Modular triggers on death, Graft on another creature entering).
- **Graft + Evolve**: Both can trigger on the same creature entering. Evolve only fires for controller's creatures; Graft fires for any creature. APNAP ordering applies for multiple triggers.
- **Graft + Panharmonicon**: Graft's trigger uses `AnyPermanentEntersBattlefield`, so Panharmonicon's `TriggerDoublerFilter::ArtifactOrCreatureETB` SHOULD double it (if the entering permanent is an artifact or creature AND the graft permanent is controlled by the Panharmonicon controller). This should work automatically with the existing doubler infrastructure.
- **SBA interaction**: Moving the last counter off a 0/0 creature kills it. This is a natural consequence of existing SBA checks and requires no special handling.
- **Llanowar Reborn (land with Graft)**: The ETB counter placement must work in `lands.rs` for PlayLand. The +1/+1 counter does nothing on a non-creature land but is important if the land becomes a creature later (e.g., via Nissa, Who Shakes the World).

## Discriminant Summary

| Type | Variant | Discriminant |
|------|---------|-------------|
| `KeywordAbility` | `Graft(u32)` | 119 |
| `StackObjectKind` | `GraftTrigger` | 44 |
| `PendingTriggerKind` | `Graft` | (no explicit discriminant, enum variant only) |
| `AbilityDefinition` | n/a (uses `Keyword(Graft(N))` only) | n/a |
