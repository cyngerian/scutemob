# Ability Plan: Ward

**Generated**: 2026-02-24
**CR**: 702.21
**Priority**: P1
**Similar abilities studied**: Hexproof (`rules/protection.rs`, `rules/mod.rs:L43-65`),
Shroud (`rules/protection.rs:L117-140`), CounterSpell effect (`effects/mod.rs:L506-540`),
MayPayOrElse effect (`effects/mod.rs:L961-963`), check_triggers dispatch (`rules/abilities.rs:L259-339`)

## CR Rule Text

> **702.21.** Ward
>
> **702.21a** Ward is a triggered ability. Ward [cost] means "Whenever this permanent
> becomes the target of a spell or ability an opponent controls, counter that spell or
> ability unless that player pays [cost]."
>
> **702.21b** Some ward abilities include an X in their cost and state what X is equal
> to. This value is determined at the time the ability resolves, not locked in as the
> ability triggers.

## Key Edge Cases

From CR 702.21 and card rulings:

1. **Ward triggers per-permanent**: If a spell targets multiple permanents with ward,
   each ward triggers separately. If any ward cost is not paid, the spell is countered.
   (Ruling: Adrix and Nev, Purple Worm, Sedgemoor Witch)

2. **Multiple ward abilities on one permanent**: If a creature has two ward abilities
   (e.g., its own + Plate Armor's), both trigger separately. The spell is countered if
   either cost is not paid. (Ruling: Plate Armor, Leather Armor, Rith Liberated Primeval,
   Winter Cursed Rider)

3. **"Can't be countered" interacts with ward**: Ward still triggers for spells that
   can't be countered. The opponent may still choose to pay the ward cost. But if they
   don't pay, the spell is NOT countered (it still resolves). Ward's counter effect
   simply has no effect. (Rulings: Raze to the Ground, Abrupt Decay, Void Rend,
   Out Cold, Lithomantic Barrage)

4. **Ward triggers only when becoming a target**: If a permanent gains ward after
   already being targeted, the ward ability does NOT trigger retroactively. Ward only
   fires at the moment the permanent becomes a target. (Ruling: Hall of Storm Giants)

5. **Ward is an opponent-only restriction**: Ward only triggers for spells/abilities
   controlled by opponents, not by the permanent's controller.

6. **Ward costs can be mana or non-mana**: Most common is mana (ward {2}, ward {1}),
   but ward can also require paying life, discarding, or sacrificing. For initial
   implementation, support mana costs (ward {N}). Non-mana ward costs are deferred.

7. **702.21b -- Variable X costs**: X is determined at resolution time, not trigger
   time. Deferred to a future phase; initial implementation handles fixed mana costs.

8. **Ward triggers on both spells AND abilities**: Not just spells -- activated and
   triggered abilities that target the warded permanent also trigger ward.

9. **Multiplayer**: Ward checks "an opponent" -- in Commander, any player other than
   the controller is an opponent. The controller targeting their own warded permanent
   does NOT trigger ward.

## Current State (from ability-wip.md)

- [ ] 1. Enum variant -- `KeywordAbility::Ward` exists at `state/types.rs:L128`,
      hashed at `state/hash.rs:L287`. **But Ward needs a cost parameter.**
- [ ] 2. Rule enforcement -- no "becomes the target" trigger infrastructure exists
- [ ] 3. Trigger wiring -- no `SelfBecomesTargetByOpponent` TriggerEvent
- [ ] 4. Unit tests
- [ ] 5. Card definition
- [ ] 6. Game script
- [ ] 7. Coverage doc update

## Implementation Steps

### Step 1: Parameterize the KeywordAbility::Ward Variant

**File**: `crates/engine/src/state/types.rs`
**Action**: Change `Ward` to `Ward(WardCost)` where `WardCost` encodes the cost.
Since Ward's cost is most commonly a mana amount (ward {1}, ward {2}), and the
engine already has `ManaCost` for mana costs, use `ManaCost` directly:

```rust
/// CR 702.21a: Ward [cost] -- counter targeting spell/ability unless opponent pays [cost].
Ward(ManaCost),
```

**Pattern**: Follow `ProtectionFrom(ProtectionQuality)` at `types.rs:L122` -- this is
the existing pattern for parameterized keyword variants.

**Import**: `ManaCost` is already imported in `types.rs` indirectly via other modules;
add `use super::game_object::ManaCost;` if needed.

**Hash**: Update `state/hash.rs:L287` -- change from `21u8.hash_into(hasher)` to:
```rust
KeywordAbility::Ward(cost) => {
    21u8.hash_into(hasher);
    cost.hash_into(hasher);
}
```

**Match arms**: Grep for all `KeywordAbility::Ward` patterns in the codebase and
update them. Known locations:
- `state/hash.rs:L287` (hash impl)
- Any display/debug matches (none expected beyond derive)

**Note**: `ManaCost` already implements `HashInto` (at `hash.rs`), `Clone`, `Debug`,
`PartialEq`, `Eq`, `Serialize`, `Deserialize`, and `Default`. It does NOT implement
`Hash` or `Ord` -- but `KeywordAbility` derives `Hash` and `Ord`, so `ManaCost` must
also implement them. Check whether `ManaCost` has these derives. If not, either:
(a) add `#[derive(Hash, Ord, PartialOrd)]` to `ManaCost`, or
(b) use a simpler representation like `WardCost(u32)` for the generic mana amount.

**Recommendation**: Use a `u32` directly: `Ward(u32)` representing the generic mana
cost (covers ward {1}, ward {2}, etc.). This avoids derive issues. Non-generic ward
costs (life, discard) can be extended later with an enum:

```rust
Ward(u32),  // ward {N} -- generic mana cost
```

This is simpler and covers all current card definitions. `u32` derives `Hash`, `Ord`,
`Copy` etc. trivially.

### Step 2: Add TriggerEvent::SelfBecomesTargetByOpponent

**File**: `crates/engine/src/state/game_object.rs`
**Action**: Add a new variant to `TriggerEvent`:

```rust
/// Triggers when this permanent becomes the target of a spell or ability
/// an opponent controls (CR 702.21a). Used by Ward.
/// `targeting_stack_id` is the ObjectId of the stack object (spell/ability)
/// that is targeting this permanent.
SelfBecomesTargetByOpponent,
```

**Hash**: Update `state/hash.rs` `HashInto for TriggerEvent` (around L861-870) to add:
```rust
TriggerEvent::SelfBecomesTargetByOpponent => 6u8.hash_into(hasher),
```

### Step 3: Add TriggerCondition::WhenBecomesTargetByOpponent

**File**: `crates/engine/src/cards/card_definition.rs`
**Action**: Add a new variant to `TriggerCondition`:

```rust
/// "Whenever this permanent becomes the target of a spell or ability an
/// opponent controls" (CR 702.21a). Used by Ward.
WhenBecomesTargetByOpponent,
```

**Hash**: Update `state/hash.rs` `HashInto for TriggerCondition` (around L1787-1814)
to add a new discriminant:
```rust
TriggerCondition::WhenBecomesTargetByOpponent => 17u8.hash_into(hasher),
```

### Step 4: Add GameEvent for Targeting

**File**: `crates/engine/src/rules/events.rs`
**Action**: Add a new event variant to `GameEvent` to notify when a permanent becomes
the target of a spell or ability. This event drives the ward trigger dispatch:

```rust
/// A permanent became the target of a spell or ability (CR 702.21a).
///
/// Emitted after a spell is cast or ability activated with targets. Used to
/// fire ward triggers. `target_id` is the permanent being targeted.
/// `targeting_stack_id` is the stack object whose spell/ability targets it.
/// `targeting_controller` is the player who controls the targeting spell/ability.
PermanentTargeted {
    target_id: ObjectId,
    targeting_stack_id: ObjectId,
    targeting_controller: PlayerId,
},
```

### Step 5: Emit PermanentTargeted Events from Casting and Ability Activation

**File**: `crates/engine/src/rules/casting.rs`
**Action**: After `SpellCast` is pushed (around L224-228), iterate over the validated
targets and emit a `PermanentTargeted` event for each `Target::Object` that resolves
to a battlefield permanent:

```rust
// CR 702.21a: Emit PermanentTargeted for each object target (drives ward triggers).
for target in &validated_targets {
    if let Target::Object(id) = &target.target {
        if let Some(obj) = state.objects.get(id) {
            if obj.zone == ZoneId::Battlefield {
                events.push(GameEvent::PermanentTargeted {
                    target_id: *id,
                    targeting_stack_id: stack_entry_id,
                    targeting_controller: player,
                });
            }
        }
    }
}
```

**File**: `crates/engine/src/rules/abilities.rs`
**Action**: After `AbilityActivated` is pushed in `handle_activate_ability` (around
L235-239), emit `PermanentTargeted` events similarly for each `Target::Object`:

```rust
// CR 702.21a: Emit PermanentTargeted for each object target (drives ward triggers).
for t in &targets {
    if let Target::Object(id) = t {
        if let Some(obj) = state.objects.get(id) {
            if obj.zone == ZoneId::Battlefield {
                events.push(GameEvent::PermanentTargeted {
                    target_id: *id,
                    targeting_stack_id: stack_id,
                    targeting_controller: player,
                });
            }
        }
    }
}
```

### Step 6: Wire Ward Trigger Dispatch in check_triggers

**File**: `crates/engine/src/rules/abilities.rs`
**Action**: Add a new match arm in `check_triggers` for `GameEvent::PermanentTargeted`:

```rust
GameEvent::PermanentTargeted {
    target_id,
    targeting_stack_id,
    targeting_controller,
} => {
    // CR 702.21a: Ward triggers when this permanent becomes the target
    // of a spell or ability an opponent controls.
    collect_triggers_for_event(
        state,
        &mut triggers,
        TriggerEvent::SelfBecomesTargetByOpponent,
        Some(*target_id), // Only check the targeted permanent
        None,             // No entering_object for this event type
    );
}
```

**Important**: The `collect_triggers_for_event` function currently does not check
controller relationships (opponent vs. self). Ward's "an opponent controls" check
must be done either:
(a) Inside `collect_triggers_for_event` with an extra parameter, or
(b) After collecting, by filtering triggers where the targeting controller is an
    opponent of the triggered ability's controller.

**Recommendation**: Option (b) is simpler and doesn't change the existing function
signature. In `check_triggers`, after `collect_triggers_for_event` returns, filter
the results:

```rust
// Filter: Ward only triggers for opponent-controlled spells/abilities.
triggers.retain(|t| {
    if t.triggering_event == Some(TriggerEvent::SelfBecomesTargetByOpponent) {
        t.controller != *targeting_controller
    } else {
        true
    }
});
```

Wait -- this is a problem because `collect_triggers_for_event` pushes into the shared
`triggers` vec. Better approach: do the opponent check inline:

```rust
GameEvent::PermanentTargeted {
    target_id,
    targeting_controller,
    targeting_stack_id,
} => {
    // CR 702.21a: Ward triggers on the targeted permanent itself.
    // Only triggers if the targeting player is an opponent (not the controller).
    if let Some(obj) = state.objects.get(target_id) {
        if obj.controller != *targeting_controller {
            collect_triggers_for_event(
                state,
                &mut triggers,
                TriggerEvent::SelfBecomesTargetByOpponent,
                Some(*target_id),
                None,
            );
        }
    }
}
```

**Additional data for resolution**: The ward trigger, when it resolves, needs to know
WHICH stack object to counter. The `PendingTrigger` struct needs a way to carry this
context. Currently `PendingTrigger` has:
- `source`: ObjectId (the permanent with ward)
- `ability_index`: usize
- `controller`: PlayerId
- `triggering_event`: Option<TriggerEvent>
- `entering_object_id`: Option<ObjectId>

We need the `targeting_stack_id` to survive from trigger time to resolution time. Two
approaches:
(a) Add a field `context_object_id: Option<ObjectId>` to `PendingTrigger` and
    `StackObjectKind::TriggeredAbility` to carry the targeting stack object ID.
(b) Store the targeting stack ID on the `StackObject.targets` when flushing the ward
    trigger to the stack (the ward trigger targets the spell/ability it wants to
    counter).

**Recommendation**: Option (b) is more aligned with MTG rules -- ward's triggered
ability implicitly "targets" the spell/ability that triggered it (though technically
ward doesn't use the word "target"). In the engine, set the ward trigger's
`StackObject.targets` to contain `Target::Object(targeting_stack_id)` when flushing
the trigger to the stack. This requires modifying `flush_pending_triggers` or adding
a new field to `PendingTrigger`.

**Simpler approach**: Add `targeting_stack_id: Option<ObjectId>` to `PendingTrigger`.
When flushing triggers, if this field is `Some`, set it as the ward trigger's target
on the stack.

### Step 7: Carry Targeting Context Through PendingTrigger

**File**: `crates/engine/src/state/stubs.rs`
**Action**: Add `targeting_stack_id: Option<ObjectId>` to `PendingTrigger`:

```rust
pub struct PendingTrigger {
    pub source: ObjectId,
    pub ability_index: usize,
    pub controller: PlayerId,
    pub triggering_event: Option<TriggerEvent>,
    pub entering_object_id: Option<ObjectId>,
    /// CR 702.21a: The stack object that targeted this permanent (for Ward).
    /// When present, this is passed to the triggered ability's StackObject as
    /// a target so the ward resolution can counter the correct spell/ability.
    pub targeting_stack_id: Option<ObjectId>,
}
```

**Hash**: Update `HashInto for PendingTrigger` to include the new field.

**File**: `crates/engine/src/rules/abilities.rs`
**Action**: When creating `PendingTrigger` entries in `collect_triggers_for_event`,
set `targeting_stack_id: None` by default. For the `PermanentTargeted` handler, pass
`targeting_stack_id` through.

This requires either:
(a) Passing an extra parameter to `collect_triggers_for_event`, or
(b) Setting the field on the returned triggers after `collect_triggers_for_event`.

**Recommendation**: Option (b) -- after `collect_triggers_for_event` returns, loop
over the newly added triggers and set `targeting_stack_id`:

```rust
let pre_len = triggers.len();
collect_triggers_for_event(
    state,
    &mut triggers,
    TriggerEvent::SelfBecomesTargetByOpponent,
    Some(*target_id),
    None,
);
// Tag ward triggers with the targeting stack object ID.
for t in &mut triggers[pre_len..] {
    t.targeting_stack_id = Some(*targeting_stack_id);
}
```

**File**: `crates/engine/src/rules/abilities.rs` (`flush_pending_triggers`)
**Action**: When creating `StackObject` for a triggered ability that has
`targeting_stack_id`, populate the `targets` field:

```rust
let stack_targets = if let Some(tsid) = trigger.targeting_stack_id {
    vec![SpellTarget {
        target: Target::Object(tsid),
        zone_at_cast: None, // stack objects don't have a "zone"
    }]
} else {
    vec![]
};

let stack_obj = StackObject {
    // ...
    targets: stack_targets,
    // ...
};
```

### Step 8: Ward Resolution Effect

**File**: `crates/engine/src/rules/abilities.rs` (or `rules/ward.rs` as new module)
**Action**: When a ward trigger resolves (in `resolution.rs`), the triggered ability's
effect is: "counter that spell or ability unless that player pays [cost]."

This maps to `Effect::MayPayOrElse`:
- cost = the ward mana cost (from the keyword)
- payer = controller of the targeting spell/ability
- or_else = `Effect::CounterSpell { target: DeclaredTarget { index: 0 } }`

The ward cost comes from the permanent's `KeywordAbility::Ward(n)` on the
battlefield. At resolution time, look up the source permanent's ward cost.

**Approach**: Instead of encoding ward's effect in `TriggeredAbilityDef.effect` (which
would require carrying the dynamic cost), implement ward resolution as special-case
logic in `resolution.rs` when resolving a `TriggeredAbility` whose
`triggering_event` is `SelfBecomesTargetByOpponent`.

Better approach: When creating the triggered ability def from the Ward keyword during
`enrich_spec_from_def` or object construction, encode the effect directly:

**File**: `crates/engine/src/testing/replay_harness.rs` (enrich_spec_from_def)
and any other place that translates `KeywordAbility::Ward(n)` into a
`TriggeredAbilityDef`.

**File**: `crates/engine/src/state/builder.rs` (ObjectSpec::build or
GameStateBuilder::finalize)
**Action**: When building a `GameObject` from an `ObjectSpec` that has
`KeywordAbility::Ward(n)`, automatically add a corresponding `TriggeredAbilityDef`:

```rust
if let Some(KeywordAbility::Ward(cost)) = keywords.iter().find(|k| matches!(k, KeywordAbility::Ward(_))) {
    characteristics.triggered_abilities.push(TriggeredAbilityDef {
        trigger_on: TriggerEvent::SelfBecomesTargetByOpponent,
        intervening_if: None,
        description: format!("Ward {{{}}} -- counter unless opponent pays", cost),
        effect: Some(Effect::MayPayOrElse {
            cost: Cost::Mana(ManaCost { generic: *cost, ..Default::default() }),
            payer: PlayerTarget::DeclaredTarget { index: 0 },
            or_else: Box::new(Effect::CounterSpell {
                target: EffectTarget::DeclaredTarget { index: 0 },
            }),
        }),
    });
}
```

**Problem**: The `payer` of MayPayOrElse needs to be the controller of the targeting
spell/ability. At resolution time, `DeclaredTarget { index: 0 }` resolves to the
stack object being targeted. We need the payer to be the CONTROLLER of that stack
object, not the stack object itself.

**Solution**: Use `PlayerTarget::ControllerOf(Box::new(EffectTarget::DeclaredTarget { index: 0 }))`.
Wait -- `ControllerOf` takes an `EffectTarget` and finds the controller of that
permanent. But the target is a stack object, not a permanent. Need to check if
`ControllerOf` handles stack objects.

Actually, looking at `MayPayOrElse` implementation (`effects/mod.rs:L961-963`):
```rust
Effect::MayPayOrElse { or_else, .. } => {
    // M9+: interactive choice to pay or not. For M7, don't pay -> apply or_else.
    execute_effect_inner(state, or_else, ctx, events);
}
```

The current implementation ALWAYS applies `or_else` (deterministic fallback -- payment
is not interactive yet). This means ward will ALWAYS counter the spell in the
deterministic engine. This is correct behavior for the non-interactive engine: if you
can't interactively pay, ward's cost is not paid, and the spell is countered.

For the initial ward implementation, this is acceptable. The `MayPayOrElse` will be
made interactive in M10+.

**Revised approach**: Since MayPayOrElse always fires or_else, ward's effect simplifies
to `CounterSpell { target: DeclaredTarget { index: 0 } }` for now. But we should still
use `MayPayOrElse` to encode the correct semantics for future interactive play.

**Counter vs cant_be_countered**: Ward's counter effect must respect
`cant_be_countered`. The existing `Effect::CounterSpell` already checks this flag
(at `effects/mod.rs:L519-522`). So this interaction is already handled.

### Step 9: Wire Ward Keyword to TriggeredAbilityDef in Object Construction

There are TWO places where keywords get mapped to runtime triggered abilities:

**A. Builder path** (`crates/engine/src/state/builder.rs`):
When `ObjectSpec` is built into a `GameObject`, the builder copies keywords from the
spec into `characteristics.keywords`. It does NOT auto-generate triggered abilities
from keywords. We need to add ward-to-trigger translation here.

Find where `ObjectSpec` keywords are copied to `characteristics.keywords` in the
`build_object` or similar method, and after that point, add:

```rust
// CR 702.21a: Ward keyword generates a triggered ability.
for kw in &characteristics.keywords {
    if let KeywordAbility::Ward(cost) = kw {
        characteristics.triggered_abilities.push(TriggeredAbilityDef {
            trigger_on: TriggerEvent::SelfBecomesTargetByOpponent,
            intervening_if: None,
            description: format!("Ward {{{}}} (CR 702.21a)", cost),
            effect: Some(Effect::MayPayOrElse {
                cost: Cost::Mana(ManaCost { generic: *cost, ..Default::default() }),
                payer: PlayerTarget::DeclaredTarget { index: 0 },
                or_else: Box::new(Effect::CounterSpell {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                }),
            }),
        });
    }
}
```

**B. CardDefinition path** (`crates/engine/src/cards/definitions.rs`):
Card definitions that have ward should include `AbilityDefinition::Keyword(KeywordAbility::Ward(N))`.
The `enrich_spec_from_def` function in `replay_harness.rs` copies keyword abilities
from card definitions to object specs. It may also need to generate the triggered
ability. Check `enrich_spec_from_def`.

**File**: `crates/engine/src/testing/replay_harness.rs`
**Action**: In `enrich_spec_from_def`, when populating keywords from card definitions,
also auto-generate the ward triggered ability. Or rely on the builder path (A) doing
the translation.

**Preferred**: Do the translation in the builder path (A) so it applies universally.
The builder already processes keywords -- just add the ward-to-trigger translation at
that point.

### Step 10: Handle targeting_stack_id in StackObject Resolution

**File**: `crates/engine/src/rules/resolution.rs`
**Action**: Ward's triggered ability resolves like any other triggered ability. Its
effect is `MayPayOrElse { ..., or_else: CounterSpell { ... } }`. The `CounterSpell`
effect needs to find the stack object to counter via `DeclaredTarget { index: 0 }`.

The ward trigger's `StackObject.targets[0]` contains `Target::Object(targeting_stack_id)`.
When `CounterSpell` resolves via `execute_effect`, it calls `resolve_effect_target_list`
which resolves `DeclaredTarget { index: 0 }` to the targeting stack object's ID.

The existing `CounterSpell` implementation (effects/mod.rs:L506-540) finds the stack
object by looking for a `StackObjectKind::Spell { source_object }` matching the ID.
But the ward trigger targets the `targeting_stack_id` which is the `StackObject.id`,
not the `source_object` ID.

**Problem**: `CounterSpell` searches `state.stack_objects` by `source_object == id`,
but ward passes the `StackObject.id` (stack entry ID), not the source card ID. These
are different IDs.

**Fix**: Either:
(a) Change the ward target to use the source_object ID instead of the stack_object ID, or
(b) Modify `CounterSpell` to also check `so.id == id`.

**Recommendation**: Option (b) is more correct. The `CounterSpell` effect should find
a stack object by either its `id` or its `source_object`:

```rust
let pos = state.stack_objects.iter().position(|so| {
    so.id == id  // Direct stack object ID match (for ward-style targeting)
    || matches!(&so.kind, StackObjectKind::Spell { source_object } if *source_object == id)
});
```

This change is backward-compatible -- existing CounterSpell uses via source_object
still work.

**Alternative**: Use `resolution::counter_stack_object(state, targeting_stack_id)`
directly as a new effect variant instead of `CounterSpell`. But using the existing
`CounterSpell` with the fix above is simpler.

### Step 11: Unit Tests

**File**: `crates/engine/tests/ward.rs` (new file)
**Tests to write**:

1. **`test_ward_basic_counter_on_targeting`**
   - CR 702.21a: Creature with ward {2} on battlefield. Opponent casts a targeting spell.
   - Ward triggers, resolves, counters the spell (deterministic: always counters).
   - Assert: spell is countered (SpellCountered event), creature survives.
   - Pattern: Follow `test_702_18_shroud_prevents_targeting` at `tests/keywords.rs:L369`.

2. **`test_ward_does_not_trigger_for_controller`**
   - CR 702.21a: Controller targets their own warded creature.
   - Ward does NOT trigger. Spell resolves normally.
   - Assert: no SpellCountered event, spell resolves.

3. **`test_ward_does_not_trigger_for_non_targeting`**
   - CR 702.21a: Opponent casts "destroy all creatures" (no targeting).
   - Ward does NOT trigger. Global effect applies.
   - Assert: creature is destroyed.

4. **`test_ward_triggers_for_activated_ability_targeting`**
   - CR 702.21a: Ward triggers for abilities too, not just spells.
   - Opponent activates an ability targeting the warded permanent.
   - Assert: ward trigger fires.

5. **`test_ward_cant_be_countered_spell`**
   - CR 101.6 + ward rulings: A "can't be countered" spell that targets a warded
     creature still triggers ward. Ward resolves and tries to counter, but the counter
     has no effect. The spell resolves normally.
   - Assert: ward triggers, but spell still resolves (not countered).

6. **`test_ward_multiple_targets_each_trigger`**
   - A spell targets two different creatures with ward. Each ward triggers separately.
   - Assert: two ward triggers on the stack.

7. **`test_ward_multiplayer_opponent_check`**
   - In a 4-player game, player B targets player A's warded creature. Ward triggers.
   - Player A targets their own warded creature. Ward does NOT trigger.
   - Assert: ward fires for B's spell only.

**Pattern**: Follow the hexproof test at `tests/keywords.rs:L425-478` for setup
(GameStateBuilder, ObjectSpec::creature with keyword, CastSpell command, assert on
result or events).

### Step 12: Card Definition (later phase)

**Suggested card**: Adrix and Nev, Twincasters
- Oracle: "Ward {2}. If one or more tokens would be created under your control,
  twice that many of those tokens are created instead."
- Simple ward {2} creature, good test subject.
- The token-doubling replacement effect can be omitted for ward-only testing.

**Alternative simpler card**: Any basic ward creature (e.g., a test-only creature
with just ward {2} and no other abilities).

### Step 13: Game Script (later phase)

**Suggested scenario**: "Opponent targets warded creature, ward counters the spell"
- P1 has a creature with ward {2} on the battlefield.
- P2 casts Lightning Bolt targeting P1's creature.
- Ward triggers, goes on the stack above Bolt.
- Both players pass, ward resolves, counters Bolt.
- Assert: creature is alive, Bolt is in graveyard.

**Subsystem directory**: `test-data/generated-scripts/stack/`
(Ward resolution involves the stack and trigger ordering.)

## Interactions to Watch

1. **Ward + Hexproof/Shroud**: If a creature has both ward and hexproof, hexproof
   prevents targeting entirely (at cast time). Ward would never trigger because the
   spell can't legally target the creature. No conflict.

2. **Ward + Protection**: Similar to hexproof -- protection prevents targeting from
   matching sources. Ward only triggers if the targeting is legal. No conflict.

3. **Ward + "Can't be countered"**: The `cant_be_countered` flag on `StackObject` must
   be checked by the `CounterSpell` effect. The existing implementation already does
   this. Ward triggers but the counter has no effect.

4. **Ward + Storm copies**: Storm copies are not cast (CR 702.40c). They DO have
   targets. If a storm copy targets a warded permanent, ward should trigger. Check
   whether storm copies emit `PermanentTargeted` events.

5. **Ward + Stack ordering**: Ward triggers go on the stack above the targeting
   spell/ability. They resolve first. If ward counters the spell, it leaves the stack
   before resolution. Multiple ward triggers from different permanents stack in APNAP
   order.

6. **MayPayOrElse deterministic behavior**: The current engine always applies `or_else`
   (the counter). This means ward is effectively "counter the spell" in the
   non-interactive engine. This is acceptable -- interactive payment is M10+.

## File Summary

| Step | File | Change Type |
|------|------|-------------|
| 1 | `crates/engine/src/state/types.rs` | Modify `Ward` to `Ward(u32)` |
| 1 | `crates/engine/src/state/hash.rs` | Update Ward hash arm |
| 2 | `crates/engine/src/state/game_object.rs` | Add `SelfBecomesTargetByOpponent` to `TriggerEvent` |
| 2 | `crates/engine/src/state/hash.rs` | Hash new TriggerEvent variant |
| 3 | `crates/engine/src/cards/card_definition.rs` | Add `WhenBecomesTargetByOpponent` to `TriggerCondition` |
| 3 | `crates/engine/src/state/hash.rs` | Hash new TriggerCondition variant |
| 4 | `crates/engine/src/rules/events.rs` | Add `PermanentTargeted` event |
| 5 | `crates/engine/src/rules/casting.rs` | Emit `PermanentTargeted` after SpellCast |
| 5 | `crates/engine/src/rules/abilities.rs` | Emit `PermanentTargeted` after AbilityActivated |
| 6 | `crates/engine/src/rules/abilities.rs` | Handle `PermanentTargeted` in `check_triggers` |
| 7 | `crates/engine/src/state/stubs.rs` | Add `targeting_stack_id` to `PendingTrigger` |
| 7 | `crates/engine/src/state/hash.rs` | Hash new PendingTrigger field |
| 7 | `crates/engine/src/rules/abilities.rs` | Pass targeting_stack_id, set targets on flush |
| 8 | N/A (uses existing MayPayOrElse + CounterSpell) | Conceptual only |
| 9 | `crates/engine/src/state/builder.rs` | Ward keyword -> TriggeredAbilityDef translation |
| 10 | `crates/engine/src/effects/mod.rs` | Fix CounterSpell to match by stack object ID |
| 11 | `crates/engine/tests/ward.rs` | New test file with 7 tests |
