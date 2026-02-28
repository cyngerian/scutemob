# Ability Plan: Equip

**Generated**: 2026-02-26
**CR**: 702.6
**Priority**: P1
**Similar abilities studied**: Ward (CR 702.21) in `crates/engine/tests/ward.rs` and `rules/abilities.rs`; Mind Stone activated ability pattern in `rules/abilities.rs:46-274` and `testing/replay_harness.rs:384-406`

## CR Rule Text

702.6. Equip

  [702.6a] Equip is an activated ability of Equipment cards. "Equip [cost]" means
  "[Cost]: Attach this permanent to target creature you control. Activate only as a sorcery."

  [702.6b] For more information about Equipment, see rule 301, "Artifacts."

  [702.6c] Equip abilities may further restrict what creatures may be chosen as legal targets.
  Such restrictions usually appear in the form "Equip [quality]" or "Equip [quality] creature."
  These equip abilities may legally target only a creature that's controlled by the player
  activating the ability and that has the chosen quality. Additional restrictions for an equip
  ability don't restrict what the Equipment may be attached to.

  [702.6d] If a permanent has multiple equip abilities, any of its equip abilities may be
  activated.

  [702.6e] "Equip planeswalker" is a variant of the equip ability. "Equip planeswalker [cost]"
  means "[Cost]: Attach this permanent to target planeswalker you control as though that
  planeswalker were a creature. Activate only as a sorcery."

### Supporting Rules

**701.3 Attach**:
  [701.3a] To attach an Equipment to an object means to take it from where it currently is and
  put it onto that object. An Equipment can't be attached to an object it couldn't equip.

  [701.3b] If an effect tries to attach an Equipment to an object it can't be attached to, the
  Equipment doesn't move. If an effect tries to attach an Equipment to the object it's already
  attached to, the effect does nothing.

  [701.3c] Attaching an Equipment on the battlefield to a different object causes the Equipment
  to receive a new timestamp.

**301.5 Equipment**:
  [301.5] Some artifacts have the subtype "Equipment." An Equipment can be attached to a
  creature. It can't legally be attached to anything that isn't a creature.

  [301.5b] Equipment enter the battlefield like other artifacts. They don't enter the battlefield
  attached to a creature. Control of the creature matters only when the equip ability is
  activated and when it resolves.

  [301.5c] An Equipment that's also a creature can't equip a creature unless it has reconfigure.
  An Equipment can't equip itself. An Equipment can't equip more than one creature.

  [301.5d] An Equipment's controller is separate from the equipped creature's controller.

**602.5d**: Activated abilities that read "Activate only as a sorcery" mean the player must
follow the timing rules for casting a sorcery spell (main phase, stack empty, active player).

**704.5n**: If an Equipment is attached to an illegal permanent, it becomes unattached. (Already
implemented in `rules/sba.rs:662-763`.)

## Key Edge Cases

1. **Sorcery speed only** (CR 702.6a, CR 602.5d): Equip can only be activated during the
   controller's main phase when the stack is empty. This is NOT enforced today --
   `handle_activate_ability` has no sorcery-speed check.
2. **Target must be a creature you control** (CR 702.6a): The target must be a creature
   controlled by the activating player. Hexproof/shroud/protection checks apply (DEBT).
3. **Equipment can't equip itself** (CR 301.5c): If the equipment is somehow also a creature,
   it cannot target itself.
4. **Re-equip is legal** (CR 701.3b): Moving equipment from one creature to another is legal.
   If it's already attached to the target, the effect does nothing.
5. **Equipment detaches from previous creature** (CR 301.5c): "An Equipment can't equip more
   than one creature." On resolution, the equipment is detached from any previous creature
   before being attached to the new target.
6. **New timestamp on reattach** (CR 701.3c, CR 613.7e): When equipment is attached to a
   different object, it receives a new timestamp.
7. **Protection blocks equipping** (CR 702.16d): A creature with protection from a quality that
   matches the equipment cannot be equipped. The equip ability's target becomes illegal.
   (Existing target validation in `handle_activate_ability` covers this via
   `validate_target_protection`.)
8. **Lightning Greaves shroud paradox**: Equipment with shroud (Lightning Greaves) makes the
   equipped creature untargetable. To move Lightning Greaves to another creature, you must
   first move it off the current creature (equip a different creature). But the current creature
   has shroud, so... actually, Equip targets the NEW creature, not the current one. So equip a
   different creature to move the Greaves there. The old creature temporarily loses shroud.
9. **Resolution check** (CR 301.5b): "Control of the creature matters only when the equip
   ability is activated and when it resolves." If the target creature changes controller or
   leaves the battlefield between activation and resolution, the equip fizzles per the
   standard target legality check (CR 608.2b).
10. **Multiplayer**: Equipment controller is separate from equipped creature controller
    (CR 301.5d). Only the Equipment's controller can activate its equip ability.

## Current State (from ability-wip.md)

- [ ] Step 1: Enum variant -- `KeywordAbility::Equip` exists at `state/types.rs:L125`
- [ ] Step 2: Rule enforcement -- no activation logic, no `Effect::AttachEquipment`, no
  sorcery-speed enforcement in `handle_activate_ability`
- [ ] Step 3: Trigger wiring -- N/A (equip is an activated ability, not a trigger)
- [ ] Step 4: Unit tests
- [ ] Step 5: Card definition
- [ ] Step 6: Game script
- [ ] Step 7: Coverage doc update

## Implementation Steps

### Step 1: Enum Variant (DONE -- no changes needed)

`KeywordAbility::Equip` already exists at `crates/engine/src/state/types.rs:125`.
The enum variant is already in the `HashInto` impl at `crates/engine/src/state/hash.rs`.

No changes needed for this step.

### Step 2: Rule Enforcement

This is the core implementation step with three sub-tasks:

#### Step 2a: Add `Effect::AttachEquipment` to the Effect enum

**File**: `crates/engine/src/cards/card_definition.rs`
**Action**: Add a new variant to the `Effect` enum (around line 320, before `Nothing`):

```rust
/// CR 702.6a / CR 701.3a: Attach the source Equipment to the target creature.
///
/// Used as the effect of the Equip activated ability. On resolution:
/// 1. Detach Equipment from any previously equipped creature (CR 301.5c).
/// 2. Set `source.attached_to = target` and add source to `target.attachments`.
/// 3. Update Equipment timestamp (CR 701.3c, CR 613.7e).
///
/// If the target is no longer legal at resolution (left battlefield, no longer
/// a creature, no longer controlled by the activating player), the ability
/// fizzles via the standard target legality check in resolution.rs.
AttachEquipment {
    /// The equipment to attach. Should be `EffectTarget::Source`.
    equipment: EffectTarget,
    /// The creature to attach to. Should be `EffectTarget::DeclaredTarget { index: 0 }`.
    target: EffectTarget,
},
```

**Hash**: Add to `state/hash.rs` in the `HashInto for Effect` impl block. Find the
existing `Effect::Goad` discriminant and add the new variant after it with the next
available discriminant number.

#### Step 2b: Implement `Effect::AttachEquipment` execution

**File**: `crates/engine/src/effects/mod.rs`
**Action**: Add a match arm in `execute_effect` (around line 1001, near `Goad`):

```rust
Effect::AttachEquipment { equipment, target } => {
    // Resolve the equipment source
    let equip_ids = resolve_targets(state, &equipment, ctx);
    let target_ids = resolve_targets(state, &target, ctx);

    for equip_id in &equip_ids {
        for target_id in &target_ids {
            // Validate: target must be a creature on the battlefield controlled
            // by the ability controller.
            let target_valid = state.objects.get(target_id).map(|obj| {
                obj.zone == ZoneId::Battlefield
                    && obj.controller == ctx.controller
                    && obj.characteristics.card_types.contains(&CardType::Creature)
            }).unwrap_or(false);

            if !target_valid {
                continue; // CR 701.3b: can't attach to illegal target
            }

            // CR 301.5c: Equipment can't equip itself
            if equip_id == target_id {
                continue;
            }

            // CR 701.3b: Already attached to same target -- do nothing
            if state.objects.get(equip_id)
                .and_then(|o| o.attached_to)
                .map(|att| att == *target_id)
                .unwrap_or(false)
            {
                continue;
            }

            // Detach from previous creature (CR 301.5c: can't equip more than one)
            if let Some(prev_target) = state.objects.get(equip_id).and_then(|o| o.attached_to) {
                if let Some(prev) = state.objects.get_mut(&prev_target) {
                    prev.attachments.retain(|&x| x != *equip_id);
                }
            }

            // Attach to new target
            if let Some(equip_obj) = state.objects.get_mut(equip_id) {
                equip_obj.attached_to = Some(*target_id);
                // CR 701.3c / CR 613.7e: new timestamp on reattach
                equip_obj.timestamp = state.timestamp_counter;
                state.timestamp_counter += 1;
            }
            if let Some(target_obj) = state.objects.get_mut(target_id) {
                target_obj.attachments.push_back(*equip_id);
            }

            events.push(GameEvent::EquipmentAttached {
                equipment_id: *equip_id,
                target_id: *target_id,
                controller: ctx.controller,
            });
        }
    }
}
```

**Note**: The `resolve_targets` helper already exists in `effects/mod.rs` -- use the same
pattern as `TapPermanent` or `Goad`. For `EffectTarget::Source`, it resolves to `ctx.source`.
For `EffectTarget::DeclaredTarget { index: 0 }`, it resolves from `ctx.targets[0]`.

**Note**: The `timestamp_counter` field must exist on `GameState`. Check whether it exists
or if a different mechanism is used. If `next_object_id()` is used for timestamps, use
`state.next_object_id().0` for the timestamp value. Study how existing code assigns
timestamps (grep for `timestamp` in `state/`).

#### Step 2c: Add `GameEvent::EquipmentAttached` variant

**File**: `crates/engine/src/rules/events.rs`
**Action**: Add a new event variant (after `EquipmentUnattached` around line 262):

```rust
/// An Equipment was attached to a creature via the Equip ability (CR 702.6a).
/// Emitted when the equip effect resolves and the attachment state changes.
EquipmentAttached {
    /// The Equipment that was attached.
    equipment_id: ObjectId,
    /// The creature it was attached to.
    target_id: ObjectId,
    /// The player who activated the equip ability.
    controller: PlayerId,
},
```

**Hash**: Add to `state/hash.rs` in the `HashInto for GameEvent` impl. Use discriminant
**70** (next after 69 for `PermanentTargeted`):

```rust
GameEvent::EquipmentAttached {
    equipment_id,
    target_id,
    controller,
} => {
    70u8.hash_into(hasher);
    equipment_id.hash_into(hasher);
    target_id.hash_into(hasher);
    controller.hash_into(hasher);
}
```

#### Step 2d: Add sorcery-speed enforcement to `ActivatedAbility`

**File**: `crates/engine/src/state/game_object.rs`
**Action**: Add a `sorcery_speed: bool` field to `ActivatedAbility` (around line 93):

```rust
/// CR 602.5d: If true, this ability can only be activated at sorcery speed
/// (main phase, stack empty, active player only).
#[serde(default)]
pub sorcery_speed: bool,
```

**Hash**: Add to `state/hash.rs` in the `HashInto for ActivatedAbility` impl -- hash
`self.sorcery_speed` after the existing fields.

**File**: `crates/engine/src/rules/abilities.rs`
**Action**: After the controller/battlefield checks (around line 84), add sorcery-speed
validation before paying costs:

```rust
// CR 602.5d: Check sorcery-speed restriction before paying any costs.
{
    let obj = state.object(source)?;
    let ab = &obj.characteristics.activated_abilities[ability_index];
    if ab.sorcery_speed {
        // Must be active player's main phase with empty stack
        if state.turn.active_player != player {
            return Err(GameStateError::InvalidCommand(
                "sorcery-speed ability can only be activated during your own turn".into(),
            ));
        }
        if !matches!(
            state.turn.step,
            crate::state::turn::Step::PreCombatMain | crate::state::turn::Step::PostCombatMain
        ) {
            return Err(GameStateError::NotMainPhase);
        }
        if !state.stack_objects.is_empty() {
            return Err(GameStateError::StackNotEmpty);
        }
    }
}
```

#### Step 2e: Wire Equip keyword into activated ability enrichment

**File**: `crates/engine/src/testing/replay_harness.rs`
**Action**: In `enrich_spec_from_def()` (around line 386), after the existing activated
ability loop, add handling for `AbilityDefinition::Keyword(KeywordAbility::Equip)`:

The card definition for Equipment cards with Equip will use:
```rust
AbilityDefinition::Activated {
    cost: Cost::Mana(ManaCost { generic: N, ..Default::default() }),
    effect: Effect::AttachEquipment {
        equipment: EffectTarget::Source,
        target: EffectTarget::DeclaredTarget { index: 0 },
    },
    timing_restriction: Some(TimingRestriction::SorcerySpeed),
}
```

This is already a standard `Activated` ability definition, so the existing enrichment
loop at lines 386-406 handles it automatically. The `cost_to_activation_cost` function
(line 590) converts the cost. However, `sorcery_speed` is NOT propagated because
`cost_to_activation_cost` only handles `Cost` variants, not `timing_restriction`.

**Fix**: Modify the enrichment loop to propagate `timing_restriction`:

```rust
if let AbilityDefinition::Activated { cost, effect, timing_restriction } = ability {
    // ... existing mana ability filter ...
    if !is_tap_mana_ability {
        let mut activation_cost = cost_to_activation_cost(cost);
        let ab = ActivatedAbility {
            cost: activation_cost,
            description: String::new(),
            effect: Some(effect.clone()),
            sorcery_speed: matches!(timing_restriction, Some(TimingRestriction::SorcerySpeed)),
        };
        spec = spec.with_activated_ability(ab);
    }
}
```

### Step 3: Trigger Wiring

**N/A** -- Equip is an activated ability, not a triggered ability. No new trigger event
types are needed. The existing `check_triggers` and `flush_pending_triggers` infrastructure
is not involved.

However, note that equipping can trigger "whenever an equipment becomes attached" or
"whenever ~ becomes equipped" abilities on other cards. These would be separate trigger
events defined on those cards, not part of the Equip implementation itself. For now, the
`EquipmentAttached` event is emitted and can be used to wire such triggers in the future.

### Step 4: Unit Tests

**File**: `crates/engine/tests/equip.rs`

**Tests to write**:

1. **`test_equip_basic_attaches_to_creature`** (CR 702.6a)
   - Setup: Player has an Equipment with Equip {2} and a creature on the battlefield.
   - Action: Activate equip targeting the creature. Pass priority to resolve.
   - Assert: Equipment's `attached_to == Some(creature_id)`, creature's `attachments`
     contains equipment_id. `EquipmentAttached` event emitted.

2. **`test_equip_grants_keywords_to_equipped_creature`** (CR 702.6a + CR 604)
   - Setup: Lightning Greaves (haste + shroud) on battlefield, creature on battlefield.
   - Action: Equip creature. Resolve.
   - Assert: `calculate_characteristics(creature)` includes Haste and Shroud keywords
     (via the static continuous effect + AttachedCreature filter).

3. **`test_equip_sorcery_speed_only`** (CR 702.6a, CR 602.5d)
   - Setup: Equipment + creature, state at DeclareAttackers step (not main phase).
   - Action: Try to activate equip.
   - Assert: Returns `GameStateError::NotMainPhase`.

4. **`test_equip_sorcery_speed_stack_not_empty`** (CR 602.5d)
   - Setup: Equipment + creature + spell on the stack.
   - Action: Try to activate equip.
   - Assert: Returns `GameStateError::StackNotEmpty`.

5. **`test_equip_target_must_be_creature_you_control`** (CR 702.6a)
   - Setup: Equipment controlled by P1, creature controlled by P2.
   - Action: P1 activates equip targeting P2's creature.
   - Assert: Error (target validation rejects -- creature must be controlled by activator).

6. **`test_equip_reequip_detaches_from_previous`** (CR 301.5c)
   - Setup: Equipment attached to Creature A. Creature B also on battlefield.
   - Action: Equip targeting Creature B. Resolve.
   - Assert: Equipment `attached_to == Some(creature_b)`, Creature A's `attachments`
     no longer contains equipment, Creature B's `attachments` contains equipment.

7. **`test_equip_cannot_equip_self`** (CR 301.5c)
   - Setup: A creature that is also an Equipment (hypothetical test object).
   - Action: Try to equip targeting itself.
   - Assert: Effect does nothing (no state change).

8. **`test_equip_fizzles_if_target_leaves_battlefield`** (CR 608.2b)
   - Setup: Equipment + creature. Activate equip. Before resolving, destroy the creature.
   - Action: Resolve.
   - Assert: `SpellFizzled` or no attachment change. Equipment remains unattached.

9. **`test_equip_already_attached_to_same_target_no_op`** (CR 701.3b)
   - Setup: Equipment already attached to Creature A.
   - Action: Equip targeting Creature A again. Resolve.
   - Assert: State unchanged (no extra attachment, no error).

10. **`test_equip_protection_blocks_targeting`** (CR 702.16d via DEBT "E")
    - Setup: Equipment (red artifact), creature with protection from red.
    - Action: Try to equip targeting the protected creature.
    - Assert: Target validation error (protection blocks equipping/targeting).

11. **`test_equip_pays_mana_cost`** (CR 702.6a)
    - Setup: Equipment with Equip {2}, player has exactly 2 generic mana.
    - Action: Activate equip.
    - Assert: Mana pool reduced by 2. `ManaCostPaid` event.

12. **`test_equip_insufficient_mana_rejected`** (CR 702.6a)
    - Setup: Equipment with Equip {2}, player has only 1 mana.
    - Action: Try to activate equip.
    - Assert: `GameStateError::InsufficientMana`.

**Pattern**: Follow the test structure from `crates/engine/tests/ward.rs` for setup patterns
(GameStateBuilder, CardRegistry, pass_all helper, find_object helper).

### Step 5: Card Definition (later phase)

**Update existing cards**: Lightning Greaves and Swiftfoot Boots at
`crates/engine/src/cards/definitions.rs:1260-1310` currently lack the Equip activated ability.

**Lightning Greaves** (definition #45): Add the equip ability:
```rust
AbilityDefinition::Activated {
    cost: Cost::Mana(ManaCost { ..Default::default() }), // Equip {0}
    effect: Effect::AttachEquipment {
        equipment: EffectTarget::Source,
        target: EffectTarget::DeclaredTarget { index: 0 },
    },
    timing_restriction: Some(TimingRestriction::SorcerySpeed),
},
```

**Swiftfoot Boots** (definition #46): Add equip {1}:
```rust
AbilityDefinition::Activated {
    cost: Cost::Mana(ManaCost { generic: 1, ..Default::default() }), // Equip {1}
    effect: Effect::AttachEquipment {
        equipment: EffectTarget::Source,
        target: EffectTarget::DeclaredTarget { index: 0 },
    },
    timing_restriction: Some(TimingRestriction::SorcerySpeed),
},
```

**Suggested new card**: Sol Ring's Greaves or another commonly-used equipment. Best
candidate: **Skullclamp** (Equip {1}, equipped creature gets +1/-1, when equipped creature
dies draw 2) -- but this requires a dies trigger. Simpler candidate: **Whispersilk Cloak**
(Equip {2}, equipped creature has shroud and can't be blocked).

Use `card-definition-author` agent for any new card.

### Step 6: Game Script (later phase)

**Suggested scenario**: "Player casts Lightning Greaves, equips a creature, creature gains
haste and shroud, then re-equips to another creature."

**Subsystem directory**: `test-data/generated-scripts/layers/` (since it tests equipment
keyword grant through the layer system).

**Script outline**:
1. Initial state: P1 has Lightning Greaves on battlefield, two creatures (Bear, Bird).
2. P1 activates Equip targeting Bear (cost {0}).
3. All players pass priority, equip resolves.
4. Assert: Bear has haste + shroud keywords, Lightning Greaves attached.
5. P1 activates Equip targeting Bird (cost {0}).
6. All players pass priority, equip resolves.
7. Assert: Bird now has haste + shroud, Bear no longer has them, Greaves moved.

Use `game-script-generator` agent.

### Step 7: Coverage Doc Update

**File**: `docs/mtg-engine-ability-coverage.md`
**Action**: Update the Equip row from `partial` to `validated`:

```
| Equip | 702.6 | P1 | `validated` | `state/types.rs`, `effects/mod.rs`, `rules/abilities.rs` | Lightning Greaves, Swiftfoot Boots | `layers/` script | — | Full activation + attachment + sorcery speed |
```

Also update the P1 Gaps section to remove item #1 ("Equip activation").

## Interactions to Watch

1. **Layer system**: Equipment static continuous effects use `EffectFilter::AttachedCreature`
   which resolves via `source.attached_to` at characteristic-calculation time
   (`rules/layers.rs:225-238`). The `attached_to` field MUST be set correctly for keyword
   grants to work. Verify that after `AttachEquipment` resolves, `calculate_characteristics`
   for the equipped creature includes the granted keywords.

2. **SBA 704.5n**: Already implemented in `rules/sba.rs:662-763`. If the equipped creature
   dies or loses creature type, the SBA fires and unattaches the equipment. No changes needed.

3. **Protection (DEBT)**: The "E" in DEBT is "Enchanting/Equipping." Protection from a quality
   matching the equipment prevents equipping (CR 702.16d). The SBA also detaches an already-
   attached equipment if the creature gains protection. Already handled by
   `attachment_is_illegal_due_to_protection` in `rules/protection.rs:87-95` and the SBA check.

4. **Timestamp on reattach**: When equipment moves from one creature to another, it gets a new
   timestamp (CR 701.3c). This matters for layer ordering when multiple continuous effects
   compete at the same layer. The implementation in Step 2b must update the equipment's
   `timestamp` field.

5. **Existing tests**: `test_sba_704_5n_equipment_on_non_creature_unattaches` in
   `tests/sba.rs:507-554` and `test_protection_from_red_equipment_detaches` in
   `tests/protection.rs:459-506` both manually set `attached_to` to create test states.
   After implementing Equip, these tests should continue to pass (they don't use the Equip
   ability -- they directly set attachment state).

6. **Fizzle**: The standard target legality check in `resolution.rs:50-88` handles the case
   where the target creature leaves the battlefield or changes zones between activation and
   resolution. No special fizzle logic needed for Equip -- the `ActivatedAbility` resolution
   path at `resolution.rs:244-275` handles it via the standard `is_target_legal` check when
   targets exist on the stack object.

   **Important nuance**: The current resolution code for `ActivatedAbility` (line 244-275) does
   NOT check target legality before executing the effect. Only `Spell` resolution (line 50-88)
   does the fizzle check. This means activated abilities with targets (like Equip) currently
   never fizzle. This is a pre-existing gap. The Equip implementation should either:
   (a) Add a target legality check to the `ActivatedAbility` resolution path (preferred), or
   (b) Have the `AttachEquipment` effect itself validate the target (the implementation in
       Step 2b already includes `target_valid` checks, which serves as a partial workaround).

   Option (b) is sufficient for Equip specifically, but a proper fix would add the fizzle check
   to the `ActivatedAbility` resolution path for all activated abilities with targets. Log this
   as a follow-up task if not addressed here.

7. **`timestamp_counter`**: Verify that `GameState` has a `timestamp_counter` field or equivalent.
   If timestamps are derived from `next_object_id`, use that. Grep for `timestamp` assignments
   in `state/builder.rs` and `state/mod.rs` to find the pattern.

## Files Modified (Summary)

| File | Changes |
|------|---------|
| `crates/engine/src/cards/card_definition.rs` | Add `Effect::AttachEquipment` variant |
| `crates/engine/src/effects/mod.rs` | Add `execute_effect` match arm for `AttachEquipment` |
| `crates/engine/src/rules/events.rs` | Add `GameEvent::EquipmentAttached` variant |
| `crates/engine/src/state/game_object.rs` | Add `sorcery_speed: bool` to `ActivatedAbility` |
| `crates/engine/src/rules/abilities.rs` | Add sorcery-speed check in `handle_activate_ability` |
| `crates/engine/src/testing/replay_harness.rs` | Propagate `timing_restriction` to `sorcery_speed` |
| `crates/engine/src/state/hash.rs` | Hash new event variant (70), hash `sorcery_speed` field, hash `AttachEquipment` effect |
| `crates/engine/src/cards/definitions.rs` | Add Equip activated ability to Lightning Greaves + Swiftfoot Boots |
| `crates/engine/tests/equip.rs` | New test file: 12 unit tests |
| `docs/mtg-engine-ability-coverage.md` | Update Equip row to `validated` |
