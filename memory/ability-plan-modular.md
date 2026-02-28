# Ability Plan: Modular

**Generated**: 2026-02-28
**CR**: 702.43
**Priority**: P3
**Similar abilities studied**: Riot (ETB counters, `resolution.rs:256-307`), Afterlife (parameterized dies trigger, `builder.rs:555-585`), Persist/Undying (dies trigger with `pre_death_counters`, `builder.rs:489-553`), Exploit (dedicated `StackObjectKind` for keyword trigger, `stack.rs:247-252`)

## CR Rule Text

```
702.43. Modular

702.43a Modular represents both a static ability and a triggered ability. "Modular N"
means "This permanent enters with N +1/+1 counters on it" and "When this permanent is
put into a graveyard from the battlefield, you may put a +1/+1 counter on target artifact
creature for each +1/+1 counter on this permanent."

702.43b If a creature has multiple instances of modular, each one works separately.
```

Additional relevant rule:

```
115.1e Some keyword abilities, such as equip and modular, represent targeted activated
or triggered abilities, and some keyword abilities, such as mutate, cause spells to have
targets. In those cases, the phrase "target [something]" appears in the rule for that
keyword ability rather than in the ability itself.
```

## Key Edge Cases

- **Last-known information for counter count (Arcbound Worker ruling 2006-09-25)**: "If this
  creature gets enough -1/-1 counters put on it to cause its toughness to be 0 or less (or
  the damage marked on it to be lethal), modular will put a number of +1/+1 counters on the
  target artifact creature equal to the number of +1/+1 counters on this creature before it
  left the battlefield." The count uses `pre_death_counters[PlusOnePlusOne]`, NOT the static
  N value from `Modular(N)`. If extra +1/+1 counters were added (e.g., by Arcbound Ravager's
  activated ability), the modular trigger moves ALL of them, not just N.

- **"You may" -- optional trigger**: The dies trigger says "you may put" -- it is optional.
  Deterministic default: always put counters (same as Extort's "you may pay" defaulting to
  always paying). If no valid target exists (no artifact creatures on the battlefield), the
  trigger has no legal targets and does not go on the stack at all (CR 603.3d -- if a
  triggered ability requires a target and there are no legal targets, it is removed from the
  stack on resolution / not placed).

- **Target is "target artifact creature"**: Must be both an Artifact AND a Creature on the
  battlefield. The target must be legal at trigger time (when placed on the stack) and
  re-checked at resolution time (fizzle rule CR 608.2b). Cannot target itself (it is in
  the graveyard when the trigger goes on the stack).

- **Multiple instances (CR 702.43b)**: Each Modular instance works separately. A creature
  with Modular 1 and Modular 2 enters with 3 counters (1+2) and triggers twice on death.
  Each trigger uses the same `pre_death_counters` value (the total +1/+1 counters on the
  creature at death). NOTE: both triggered instances independently target an artifact creature
  and each places the full counter count. This means two Modular triggers from the same death
  would each place the FULL count on their respective targets (or the same target).

- **Multiplayer**: Multiple Modular creatures dying simultaneously follow APNAP ordering
  for trigger placement (CR 603.3). Each trigger independently targets an artifact creature.

- **No artifact creature to target**: If the dying creature's controller controls no artifact
  creatures (or no artifact creatures exist on the battlefield at all), the modular trigger
  cannot be placed on the stack (no legal targets). The counters are simply lost.

## Current State (from ability-wip.md)

- [x] Step 1: Enum variant -- exists at `types.rs:489` as `Modular(u32)`, hash.rs disc 59, view_model.rs
- [ ] Step 2: ETB counter placement (static ability part)
- [ ] Step 3: Dies trigger wiring (triggered ability part)
- [ ] Step 4: `StackObjectKind::ModularTrigger` + resolution
- [ ] Step 5: Unit tests
- [ ] Step 6: Card definition
- [ ] Step 7: Game script

## Implementation Steps

### Step 1: Enum Variant (DONE)

**File**: `crates/engine/src/state/types.rs` line 489
**Status**: `Modular(u32)` already exists
**Hash**: `crates/engine/src/state/hash.rs` discriminant 59 already exists
**View model**: `tools/replay-viewer/src/view_model.rs` already handles `Modular(n)`

No action needed.

### Step 2: ETB Counter Placement

**File**: `crates/engine/src/rules/resolution.rs`
**Action**: Add a Modular ETB counter block immediately after the Riot block (line ~307).
**Pattern**: Follow Riot pattern at lines 256-307, but instead of counting Riot instances,
iterate over `Modular(n)` abilities from the card definition and sum the N values.
**CR**: 702.43a -- "This permanent enters with N +1/+1 counters on it"

Detailed implementation:

```rust
// CR 702.43a: Modular N -- "This permanent enters with N +1/+1 counters on it."
// CR 702.43b: Multiple instances each work separately (Modular 1 + Modular 2 = 3 counters).
// Unlike Riot (which is deduplicated by OrdSet), Modular(n) with different N values
// are distinct KeywordAbility variants and are NOT deduplicated. However, for safety,
// count from the card definition like Riot does.
{
    let modular_total: u32 = card_id
        .as_ref()
        .and_then(|cid| registry.get(cid.clone()))
        .map(|def| {
            def.abilities
                .iter()
                .filter_map(|a| match a {
                    crate::cards::card_definition::AbilityDefinition::Keyword(
                        KeywordAbility::Modular(n),
                    ) => Some(*n),
                    _ => None,
                })
                .sum()
        })
        .unwrap_or(0);

    if modular_total > 0 {
        if let Some(obj) = state.objects.get_mut(&new_id) {
            let current = obj
                .counters
                .get(&CounterType::PlusOnePlusOne)
                .copied()
                .unwrap_or(0);
            obj.counters = obj
                .counters
                .update(CounterType::PlusOnePlusOne, current + modular_total);
        }
        events.push(GameEvent::CounterAdded {
            object_id: new_id,
            counter: CounterType::PlusOnePlusOne,
            count: modular_total,
        });
    }
}
```

**Note on lands.rs**: Modular only appears on artifact creatures. Artifact creatures always
enter via `resolution.rs` (spell resolution), never via `lands.rs` (land playing). No change
to `lands.rs` is needed.

### Step 3: Dies Trigger Wiring in builder.rs

**File**: `crates/engine/src/state/builder.rs`
**Action**: Add a `Modular(n)` match arm after the Afterlife block (~line 585). Generate a
`TriggeredAbilityDef` with `trigger_on: TriggerEvent::SelfDies` and no intervening-if.
**Pattern**: Follow Afterlife at builder.rs lines 555-585, but with `effect: None` because
the effect is handled by the dedicated `ModularTrigger` resolution path (the counter count
is dynamic, not static).
**CR**: 702.43a -- "When this permanent is put into a graveyard from the battlefield..."

```rust
// CR 702.43a: Modular N -- "When this permanent is put into a graveyard from
// the battlefield, you may put a +1/+1 counter on target artifact creature
// for each +1/+1 counter on this permanent."
// Each Modular instance generates one TriggeredAbilityDef (CR 702.43b).
// The effect is NOT encoded here because the counter count is dynamic
// (based on pre_death_counters, not the static N). Resolution is handled
// by StackObjectKind::ModularTrigger.
if let KeywordAbility::Modular(_n) = kw {
    triggered_abilities.push(TriggeredAbilityDef {
        trigger_on: TriggerEvent::SelfDies,
        intervening_if: None,
        description: format!(
            "Modular (CR 702.43a): When this permanent dies, you may put a \
             +1/+1 counter on target artifact creature for each +1/+1 counter \
             on this permanent."
        ),
        effect: None, // Handled by ModularTrigger resolution
    });
}
```

### Step 4a: PendingTrigger Fields

**File**: `crates/engine/src/state/stubs.rs`
**Action**: Add two new fields to `PendingTrigger`:

```rust
/// CR 702.43a: If true, this pending trigger is a Modular trigger.
///
/// When flushed to the stack, creates a `StackObjectKind::ModularTrigger`
/// instead of the normal `StackObjectKind::TriggeredAbility`. The
/// `modular_counter_count` carries the +1/+1 counter count from last-known
/// information (pre_death_counters).
#[serde(default)]
pub is_modular_trigger: bool,
/// CR 702.43a: Number of +1/+1 counters on the creature at death time.
///
/// Only meaningful when `is_modular_trigger` is true. Captured from
/// `CreatureDied.pre_death_counters[PlusOnePlusOne]` at trigger-check time.
#[serde(default)]
pub modular_counter_count: Option<u32>,
```

### Step 4b: Tag Modular Triggers in check_triggers

**File**: `crates/engine/src/rules/abilities.rs`
**Action**: In the `CreatureDied` arm of `check_triggers` (line ~1140-1197), after
pushing the `PendingTrigger`, detect Modular triggers and tag them.
**CR**: 702.43a -- the counter count is captured at trigger time from `pre_death_counters`.

The detection: check the `trigger_def.description` contains "Modular" or, more robustly,
check the source object's keywords for `Modular(_)`. The cleanest approach is to check
if the object in the graveyard (`new_grave_id`) has `Modular` in its keywords.

After the existing push of the `PendingTrigger` (line ~1172-1195), add:

```rust
// CR 702.43a: Tag Modular triggers with the +1/+1 counter count
// from last-known information. The trigger carries the count so
// ModularTrigger resolution can add that many counters.
let is_modular = obj
    .characteristics
    .keywords
    .iter()
    .any(|k| matches!(k, KeywordAbility::Modular(_)));
if is_modular && trigger_def.trigger_on == TriggerEvent::SelfDies
    && trigger_def.description.contains("Modular")
{
    if let Some(last) = triggers.last_mut() {
        last.is_modular_trigger = true;
        last.modular_counter_count = Some(
            pre_death_counters
                .get(&CounterType::PlusOnePlusOne)
                .copied()
                .unwrap_or(0),
        );
    }
}
```

**Important**: Also update the `PendingTrigger` struct literal in this arm to include the
new fields with their defaults (`is_modular_trigger: false`, `modular_counter_count: None`).

### Step 4c: StackObjectKind::ModularTrigger

**File**: `crates/engine/src/state/stack.rs`
**Action**: Add a new variant after `ExploitTrigger`:

```rust
/// CR 702.43a: Modular triggered ability on the stack.
///
/// "When this permanent is put into a graveyard from the battlefield,
/// you may put a +1/+1 counter on target artifact creature for each
/// +1/+1 counter on this permanent."
///
/// `counter_count` is the number of +1/+1 counters on the creature at
/// death time (last-known information from pre_death_counters). The target
/// artifact creature is in `StackObject.targets[0]`.
///
/// If no legal artifact creature target exists at trigger time, the trigger
/// is not placed on the stack.
ModularTrigger {
    source_object: ObjectId,
    counter_count: u32,
},
```

**Hash**: Add to `crates/engine/src/state/hash.rs` StackObjectKind impl after ExploitTrigger
(discriminant 11):

```rust
// ModularTrigger (discriminant 11) -- CR 702.43a
StackObjectKind::ModularTrigger {
    source_object,
    counter_count,
} => {
    11u8.hash_into(hasher);
    source_object.hash_into(hasher);
    counter_count.hash_into(hasher);
}
```

**View model**: Add to `tools/replay-viewer/src/view_model.rs` in the StackObjectKind
match (follow the ExploitTrigger pattern).

### Step 4d: Flush Modular Triggers

**File**: `crates/engine/src/rules/abilities.rs`
**Action**: In `flush_pending_triggers` (line ~1460-1503), add a branch for
`is_modular_trigger` in the `kind` construction chain. This branch:

1. Finds the first artifact creature on the battlefield (deterministic target selection).
2. If no legal target exists, skips placing the trigger on the stack.
3. Otherwise creates `StackObjectKind::ModularTrigger` with the counter count and sets
   the target as `SpellTarget { target: Target::Object(artifact_creature_id), zone_at_cast: Some(ZoneId::Battlefield) }`.

```rust
} else if trigger.is_modular_trigger {
    // CR 702.43a: Modular trigger -- "target artifact creature"
    // Deterministic default: select first artifact creature on battlefield
    // (by ObjectId ascending) controlled by any player.
    // If no legal target exists, skip this trigger entirely (CR 603.3d).
    let target_id = state
        .objects
        .iter()
        .filter(|(_, obj)| {
            obj.zone == ZoneId::Battlefield
                && obj.characteristics.card_types.contains(&CardType::Artifact)
                && obj.characteristics.card_types.contains(&CardType::Creature)
        })
        .map(|(id, _)| *id)
        .next();

    match target_id {
        Some(tid) => {
            // Override trigger_targets for this specific trigger
            // (set below in the StackObject construction)
        }
        None => continue, // No legal target -- trigger not placed (CR 603.3d)
    }

    StackObjectKind::ModularTrigger {
        source_object: trigger.source,
        counter_count: trigger.modular_counter_count.unwrap_or(0),
    }
```

The target must be set on the `StackObject.targets` field. Since the existing code
constructs `trigger_targets` before the `kind` match, the Modular branch needs special
handling. The cleanest approach: override `trigger_targets` for modular triggers specifically.
Move the target selection into the `trigger_targets` construction block (before the `kind`
block) or set it after. The implementer should check the exact flow and wire it appropriately.

### Step 4e: ModularTrigger Resolution

**File**: `crates/engine/src/rules/resolution.rs`
**Action**: Add a resolution arm for `ModularTrigger` after the `ExploitTrigger` arm
(~line 912).
**CR**: 702.43a -- put +1/+1 counters on the targeted artifact creature.

```rust
// CR 702.43a: Modular trigger resolves -- put +1/+1 counters on
// target artifact creature equal to the counter_count (last-known
// information from pre_death_counters).
StackObjectKind::ModularTrigger {
    source_object: _,
    counter_count,
} => {
    let controller = stack_obj.controller;

    // CR 608.2b: Fizzle check -- verify target is still legal.
    // Target must still be an artifact creature on the battlefield.
    let target_legal = stack_obj
        .targets
        .first()
        .and_then(|t| match &t.target {
            Target::Object(id) => state.objects.get(id).map(|obj| {
                obj.zone == ZoneId::Battlefield
                    && obj
                        .characteristics
                        .card_types
                        .contains(&CardType::Artifact)
                    && obj
                        .characteristics
                        .card_types
                        .contains(&CardType::Creature)
            }),
            _ => None,
        })
        .unwrap_or(false);

    if target_legal && counter_count > 0 {
        if let Some(Target::Object(target_id)) =
            stack_obj.targets.first().map(|t| &t.target)
        {
            if let Some(obj) = state.objects.get_mut(target_id) {
                let current = obj
                    .counters
                    .get(&CounterType::PlusOnePlusOne)
                    .copied()
                    .unwrap_or(0);
                obj.counters = obj
                    .counters
                    .update(CounterType::PlusOnePlusOne, current + counter_count);
            }
            events.push(GameEvent::CounterAdded {
                object_id: *target_id,
                counter: CounterType::PlusOnePlusOne,
                count: counter_count,
            });
        }
    }
    // If target illegal (fizzled) or counter_count == 0, do nothing.

    events.push(GameEvent::AbilityResolved {
        controller,
        stack_object_id: stack_obj.id,
    });
}
```

**Also update**: The `counter_stack_object` function (~line 1003-1015) must include
`ModularTrigger` in the countering match arm:

```rust
| StackObjectKind::ModularTrigger { .. } => {
    // Countering abilities is non-standard; just remove from stack.
}
```

### Step 5: Unit Tests

**File**: `crates/engine/tests/modular.rs` (new file)
**Tests to write**:

1. **`test_modular_etb_counters`** -- CR 702.43a: Creature with Modular 1 enters the
   battlefield with 1 +1/+1 counter. Verify counter is present.
   Pattern: Follow Riot ETB counter test in `tests/riot.rs`.

2. **`test_modular_etb_counters_n`** -- CR 702.43a: Creature with Modular 3 enters with
   3 +1/+1 counters. Verify count.

3. **`test_modular_dies_transfers_counters`** -- CR 702.43a: Creature with Modular 1 dies
   (via lethal damage SBA). Modular trigger fires, targeting an artifact creature. That
   artifact creature receives +1/+1 counters equal to the dying creature's counter count.
   Pattern: Follow Afterlife basic test in `tests/afterlife.rs`.

4. **`test_modular_dies_extra_counters`** -- CR 702.43a + Arcbound Worker ruling: Creature
   with Modular 1 that had additional +1/+1 counters (e.g., 3 total) dies. The trigger
   moves ALL 3 counters (not just the static 1) to the target artifact creature.

5. **`test_modular_dies_no_artifact_creature_target`** -- CR 603.3d: Creature with Modular
   dies but no artifact creature exists on the battlefield. The trigger is not placed on
   the stack. Counters are lost.

6. **`test_modular_dies_zero_counters`** -- Edge case: Creature with Modular 1 had its
   +1/+1 counters removed before dying (e.g., by a -1/-1 counter cancelling it). The
   trigger fires but counter_count is 0, so no counters are moved.

7. **`test_modular_multiple_instances`** -- CR 702.43b: Creature with Modular 1 and
   Modular 2 enters with 3 counters (1+2). On death, two separate triggers fire, each
   placing the full counter count on target artifact creatures.

8. **`test_modular_0_0_base_stats`** -- Verify that a 0/0 creature with Modular 1 enters
   with 1 +1/+1 counter and does NOT die immediately to SBA (toughness is 0+1=1 after
   counter).

**Pattern**: Follow tests in `crates/engine/tests/afterlife.rs` and
`crates/engine/tests/persist.rs` for:
- `GameStateBuilder` setup with `ObjectSpec::creature().with_keyword(KeywordAbility::Modular(N))`
- `pass_all` helper for priority cycling
- Checking `GameEvent::CreatureDied` and `GameEvent::CounterAdded` events
- Verifying counter state via `state.objects.get(&id).unwrap().counters`

### Step 6: Card Definition (later phase)

**Suggested card**: Arcbound Worker (simplest Modular card)
- Mana cost: {1}
- Type: Artifact Creature -- Construct
- Oracle: Modular 1
- P/T: 0/0
- Color identity: colorless

**Alternative**: Arcbound Ravager (has activated ability: sacrifice artifact to add +1/+1 counter)
- More complex but tests the "extra counters beyond N" edge case

**Card lookup**: use `card-definition-author` agent

### Step 7: Game Script (later phase)

**Suggested scenario**: Arcbound Worker enters with 1 +1/+1 counter, takes lethal damage,
dies, modular trigger places counter on another artifact creature.

**Setup**: Player 1 has Arcbound Worker on battlefield (with 1 +1/+1 counter from ETB),
another artifact creature (e.g., Frogmite or a generic 2/2 artifact creature). Opponent
deals damage to Arcbound Worker. Worker dies, modular trigger fires, target artifact
creature gets the counter.

**Subsystem directory**: `test-data/generated-scripts/stack/` (triggered ability resolution)

## Interactions to Watch

- **SBA timing with 0/0 base stats**: Arcbound-style creatures are 0/0 with Modular N.
  The ETB counters must be applied BEFORE SBAs check (they are -- resolution.rs applies
  counters inline before emitting `PermanentEnteredBattlefield`). If counters are applied
  after the first SBA check, the creature would die immediately.

- **Counter annihilation with -1/-1 counters**: If a Modular creature has both +1/+1 and
  -1/-1 counters, SBA 704.5q annihilates pairs. The `pre_death_counters` should reflect
  the state AFTER annihilation (since annihilation happens as an SBA before the death SBA).
  Actually, counter annihilation and lethal-toughness SBAs happen simultaneously as a batch.
  The `pre_death_counters` are captured from the object at the time of the death SBA, which
  is BEFORE `move_object_to_zone` but DURING the SBA batch. If counter annihilation and
  death both fire in the same SBA pass, the implementation order within `check_and_apply_sbas`
  determines which snapshot `pre_death_counters` captures. This is a subtle edge case that
  should be tested.

- **Replacement effects on death**: If Rest in Peace or similar exile replacement applies,
  the creature is exiled instead of going to the graveyard. The Modular trigger says "put
  into a graveyard from the battlefield" -- if the creature goes to exile instead, the
  trigger does NOT fire. The `TriggerEvent::SelfDies` check already handles this correctly
  because `CreatureDied` is only emitted for graveyard destinations.

- **Wither interaction**: If a creature with Wither deals damage to a Modular creature,
  the damage becomes -1/-1 counters (per Wither). These interact with the +1/+1 counters
  via SBA 704.5q annihilation, reducing the `pre_death_counters[PlusOnePlusOne]` count.
  This is a natural interaction that should work correctly with the existing implementation.

## Files Modified (Summary)

1. `crates/engine/src/rules/resolution.rs` -- ETB counter block + ModularTrigger resolution + counter_stack_object update
2. `crates/engine/src/state/builder.rs` -- TriggeredAbilityDef for Modular dies trigger
3. `crates/engine/src/state/stubs.rs` -- PendingTrigger fields: `is_modular_trigger`, `modular_counter_count`
4. `crates/engine/src/rules/abilities.rs` -- Tag modular triggers in check_triggers + flush_pending_triggers ModularTrigger branch
5. `crates/engine/src/state/stack.rs` -- StackObjectKind::ModularTrigger variant
6. `crates/engine/src/state/hash.rs` -- HashInto for ModularTrigger (discriminant 11)
7. `tools/replay-viewer/src/view_model.rs` -- ModularTrigger display
8. `crates/engine/tests/modular.rs` -- New test file (8 tests)
