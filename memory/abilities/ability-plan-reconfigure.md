# Ability Plan: Reconfigure

**Generated**: 2026-03-08
**CR**: 702.151
**Priority**: P4
**Similar abilities studied**: Equip (CR 702.6, `effects/mod.rs:2062`), Fortify (CR 702.67, `effects/mod.rs:2171`, `defs/darksteel_garrison.rs`), Bestow (CR 702.103, `game_object.rs:400`, `sba.rs:852`, `casting.rs:3326`)

## CR Rule Text

702.151. Reconfigure

702.151a Reconfigure represents two activated abilities. Reconfigure [cost] means "[Cost]: Attach this permanent to another target creature you control. Activate only as a sorcery" and "[Cost]: Unattach this permanent. Activate only if this permanent is attached to a creature and only as a sorcery."

702.151b Attaching an Equipment with reconfigure to another creature causes the Equipment to stop being a creature until it becomes unattached from that creature.

## Related Rules

- CR 301.5c: "An Equipment that's also a creature can't equip a creature unless that Equipment has reconfigure."
- CR 301.5c: "An Equipment can't equip itself."

## Key Edge Cases

From card rulings (2022-02-18, Lizard Blades):

1. **Reconfigure is NOT an equip ability** -- cards that reference "equip abilities" (Fighter Class, Leonin Shikari) do not interact with reconfigure.
2. **If a permanent with reconfigure somehow still a creature after attaching** (e.g., March of the Machines), it immediately becomes unattached. This is an SBA check.
3. **If an Equipment with reconfigure loses its abilities while attached**, the "not a creature" effect continues until it becomes unattached. The effect is locked in at attachment time.
4. **Tapped Equipment can still reconfigure** -- becoming attached does not untap it. Tapped state is irrelevant while attached (no combat relevance).
5. **When it stops being a creature, creature-only Auras/Equipment fall off** (SBA 704.5m/704.5n). Auras with "enchant creature" become illegal.
6. **Can be attached by other effects** (e.g., Brass Squire) -- not limited to its own reconfigure ability.
7. **Cannot attach to itself** (CR 301.5c).
8. **Creature subtypes are lost while attached** (ruling 2022-02-18: "It also loses any creature subtypes it had").

## Current State (from ability-wip.md)

- [ ] Step 1: Enum variant
- [ ] Step 2: Rule enforcement
- [ ] Step 3: Trigger wiring (N/A -- reconfigure is purely activated abilities + static type change)
- [ ] Step 4: Unit tests
- [ ] Step 5: Card definition
- [ ] Step 6: Game script

## Verified Discriminants

- **KeywordAbility**: highest = Haunt at 142. Next available: **143**
- **StackObjectKind**: highest = HauntedCreatureDiesTrigger at 58. Next available: **59** (but Reconfigure needs no new SOK -- both abilities go through standard ActivateAbility command)
- **AbilityDefinition**: highest = Cipher at 57. Next available: **58** (needed for Reconfigure cost storage)

## Implementation Steps

### Step 1: Enum Variant

**File**: `crates/engine/src/state/types.rs`
**Action**: Add `KeywordAbility::Reconfigure` variant after `Haunt` (line ~1342)
**Discriminant**: 143
**Doc comment**:
```
/// CR 702.151: Reconfigure [cost] -- two activated abilities.
/// "[Cost]: Attach this permanent to another target creature you control.
/// Activate only as a sorcery." and "[Cost]: Unattach this permanent.
/// Activate only if this permanent is attached to a creature and only
/// as a sorcery."
///
/// CR 702.151b: While attached, the Equipment stops being a creature
/// (and loses creature subtypes).
///
/// Marker for quick presence-checking (`keywords.contains`).
/// The reconfigure cost is stored in `AbilityDefinition::Reconfigure { cost }`.
///
/// Discriminant 143.
Reconfigure,
```

**File**: `crates/engine/src/cards/card_definition.rs`
**Action**: Add `AbilityDefinition::Reconfigure { cost: ManaCost }` variant after `Cipher` (line ~665)
**Discriminant**: 58
**Doc comment**:
```
/// CR 702.151a: Reconfigure [cost] -- the cost for both attach and unattach abilities.
///
/// Discriminant 58.
Reconfigure { cost: ManaCost },
```

**File**: `crates/engine/src/state/hash.rs`
**Action**: Add hash arms for both `KeywordAbility::Reconfigure` and `AbilityDefinition::Reconfigure { cost }`. Follow the pattern of existing parameterless keyword variants and cost-bearing AbilityDefinition variants.

**File**: `tools/replay-viewer/src/view_model.rs`
**Action**: Add arms for `KeywordAbility::Reconfigure` in the keyword display match. No new SOK variant needed, so no SOK match update required.

**File**: `tools/tui/src/play/panels/stack_view.rs`
**Action**: No update needed (no new SOK variant).

### Step 2: New Effect Variant -- DetachEquipment

**File**: `crates/engine/src/cards/card_definition.rs`
**Action**: Add new `Effect::DetachEquipment` variant. This is needed for the unattach ability of Reconfigure.
**Pattern**: Simpler than `AttachEquipment` -- takes only the equipment source.
```
/// CR 702.151a: Unattach an Equipment from its currently equipped creature.
/// The Equipment remains on the battlefield unattached.
DetachEquipment {
    equipment: EffectTarget,
},
```

**File**: `crates/engine/src/effects/mod.rs`
**Action**: Add handler for `Effect::DetachEquipment`. Logic:
1. Resolve the equipment target (should be `EffectTarget::Source`).
2. Check that equipment is on the battlefield and has `attached_to`.
3. Clear `attached_to` on the equipment.
4. Remove equipment from target's `attachments`.
5. Emit `GameEvent::EquipmentUnattached { object_id }`.

**File**: `crates/engine/src/state/hash.rs`
**Action**: Add hash arm for `Effect::DetachEquipment`.

### Step 3: Builder -- Auto-generate Activated Abilities from Reconfigure

**File**: `crates/engine/src/state/builder.rs`
**Action**: In the section that translates keyword abilities into activated abilities (where Equip, Fortify, etc. are handled), add a case for `AbilityDefinition::Reconfigure { cost }` that generates TWO activated abilities:

1. **Attach ability**: `AbilityDefinition::Activated { cost: Cost::Mana(cost), effect: Effect::AttachEquipment { equipment: Source, target: DeclaredTarget { index: 0 } }, timing_restriction: Some(TimingRestriction::SorcerySpeed) }`
2. **Unattach ability**: `AbilityDefinition::Activated { cost: Cost::Mana(cost), effect: Effect::DetachEquipment { equipment: Source }, timing_restriction: Some(TimingRestriction::SorcerySpeed) }`

**Pattern**: Follow how `AbilityDefinition::Activated` for Equip is generated from card definitions. The card definition stores `AbilityDefinition::Reconfigure { cost }` and builder.rs expands it into two activated abilities.

**CR**: 702.151a -- "Reconfigure [cost] means '[Cost]: Attach...' and '[Cost]: Unattach...'"

**NOTE**: The unattach ability has an additional activation restriction: "Activate only if this permanent is attached to a creature." This must be checked at activation time. Add a condition check in `abilities.rs:handle_activate_ability` for `DetachEquipment` effects (analogous to the Equip target validation).

### Step 4: Layer 4 Type Change -- "Not a Creature While Attached"

**File**: `crates/engine/src/rules/layers.rs`
**Action**: In `calculate_characteristics`, at the Layer 4 (TypeChange) section where CDAs like Changeling and Devoid are handled, add a check:

```
// CR 702.151b: Equipment with reconfigure that is attached to a creature
// stops being a creature (and loses creature subtypes).
if chars.keywords.contains(&KeywordAbility::Reconfigure) {
    if let Some(obj) = state.objects.get(&object_id) {
        if obj.attached_to.is_some() {
            chars.card_types.remove(&CardType::Creature);
            // Remove creature subtypes (ruling 2022-02-18).
            // Retain non-creature subtypes like "Equipment".
            chars.subtypes.retain(|st| {
                // Keep Equipment, Fortification, and other non-creature subtypes.
                // Remove creature subtypes.
                !ALL_CREATURE_TYPES.contains(st)
            });
        }
    }
}
```

**Important**: This is the key mechanic -- the Equipment stops being a creature while attached. Per ruling (2022-02-18), "if an Equipment with reconfigure loses its abilities while attached, the effect causing it to not be a creature continues to apply until it becomes unattached." This means the type removal is NOT tied to having the keyword at layer resolution time. Instead, it should use a flag-based approach similar to Bestow's `is_bestowed`.

**REVISED APPROACH**: Add an `is_reconfigured: bool` field to `GameObject` (set when attached via reconfigure or any other attach effect on an Equipment with reconfigure). The Layer 4 check should use `obj.is_reconfigured` rather than checking `keywords.contains(Reconfigure)`, so that ability removal (Humility, Dress Down) does not re-enable creature status while still attached.

### Step 5: New State Field -- `is_reconfigured`

**File**: `crates/engine/src/state/game_object.rs`
**Action**: Add `pub is_reconfigured: bool` field to `GameObject`.
**Doc comment**: `/// CR 702.151b: If true, this Equipment is attached via reconfigure. While true, the permanent is not a creature (ruling 2022-02-18: persists even if reconfigure keyword is removed).`

**Initialize to `false` in**:
- `crates/engine/src/state/builder.rs` (object construction)
- `crates/engine/src/effects/mod.rs` (token creation)
- `crates/engine/src/rules/resolution.rs` (permanent enters battlefield)

**Reset on zone change**: Add `is_reconfigured: false` at BOTH `move_object_to_zone` sites in `crates/engine/src/state/mod.rs` (per CR 400.7).

**Hash**: Add to `crates/engine/src/state/hash.rs` in the `GameObject` hasher.

### Step 6: Set `is_reconfigured` on Attach

**File**: `crates/engine/src/effects/mod.rs`
**Action**: In the `Effect::AttachEquipment` handler (line ~2139), after the attachment is set, check if the equipment has reconfigure. If so, set `is_reconfigured = true` on the equipment object.

```rust
// CR 702.151b: If the Equipment has reconfigure, it stops being a creature.
if let Some(equip_obj) = state.objects.get(&equip_id) {
    let has_reconfigure = crate::rules::layers::calculate_characteristics(state, equip_id)
        .map(|chars| chars.keywords.iter().any(|k| matches!(k, KeywordAbility::Reconfigure)))
        .unwrap_or(false);
    if has_reconfigure {
        if let Some(equip_obj) = state.objects.get_mut(&equip_id) {
            equip_obj.is_reconfigured = true;
        }
    }
}
```

### Step 7: Clear `is_reconfigured` on Detach

**File**: `crates/engine/src/effects/mod.rs`
**Action**: In the new `Effect::DetachEquipment` handler, after clearing `attached_to`, set `is_reconfigured = false`.

**File**: `crates/engine/src/rules/sba.rs`
**Action**: In `check_equipment_sbas` (line ~991), when an Equipment is unattached by SBA (illegal target), also set `is_reconfigured = false`.

### Step 8: SBA -- Reconfigured Equipment That Is Still a Creature

**File**: `crates/engine/src/rules/sba.rs`
**Action**: Add a check in `check_equipment_sbas`: if an Equipment with `is_reconfigured = true` is still a creature after layer resolution (e.g., due to March of the Machines), it becomes unattached (ruling 2022-02-18: "it immediately becomes unattached from the equipped creature").

This is an ADDITIONAL SBA check beyond the normal CR 704.5n logic:
```
// CR 702.151b + ruling 2022-02-18: If a reconfigured Equipment is somehow
// still a creature (e.g., March of the Machines), it becomes unattached.
if obj.is_reconfigured {
    let chars = chars_map.get(id);
    if let Some(c) = chars {
        if c.card_types.contains(&CardType::Creature) {
            // Still a creature despite being reconfigured -- unattach.
            return true;
        }
    }
}
```

### Step 9: Activation Validation for Unattach

**File**: `crates/engine/src/rules/abilities.rs`
**Action**: Add a pre-activation check for `DetachEquipment` effects (similar to the Equip target validation at line 140):

```rust
// CR 702.151a: Unattach ability -- "Activate only if this permanent is attached to a creature."
if matches!(&embedded_effect, Some(Effect::DetachEquipment { .. })) {
    let is_attached = state.objects.get(&source)
        .and_then(|obj| obj.attached_to)
        .is_some();
    if !is_attached {
        return Err(GameStateError::InvalidAction(
            "reconfigure unattach: permanent must be attached to a creature".into(),
        ));
    }
}
```

### Step 10: CR 301.5c Exemption for Reconfigure

**File**: `crates/engine/src/effects/mod.rs` (or `crates/engine/src/rules/abilities.rs`)
**Action**: The existing `AttachEquipment` handler and Equip validation do NOT currently check whether the equipment source is also a creature (CR 301.5c: "An Equipment that's also a creature can't equip a creature unless that Equipment has reconfigure"). This is a pre-existing gap, but for correctness with Reconfigure, the check should be:
- If the Equipment is a creature AND does NOT have reconfigure: reject the equip.
- If the Equipment is a creature AND HAS reconfigure: allow (this is what reconfigure exists to enable).

For MVP, the Reconfigure implementation should at minimum ensure its own attach ability works. The CR 301.5c enforcement for non-reconfigure Equipment-creatures is a separate LOW issue.

### Step 11: Unit Tests

**File**: `crates/engine/tests/reconfigure.rs`
**Tests to write**:

1. **`test_reconfigure_attach_removes_creature_type`** -- CR 702.151b
   - Set up a Reconfigure Equipment creature on the battlefield
   - Activate reconfigure (attach to target creature)
   - Assert: Equipment is attached, `is_reconfigured = true`
   - Assert: Layer-resolved characteristics do NOT include CardType::Creature
   - Assert: Creature subtypes removed, Equipment subtype retained

2. **`test_reconfigure_unattach_restores_creature_type`** -- CR 702.151a
   - Attach via reconfigure, then activate unattach
   - Assert: `attached_to = None`, `is_reconfigured = false`
   - Assert: CardType::Creature is back in characteristics
   - Assert: Creature subtypes restored

3. **`test_reconfigure_sorcery_speed_only`** -- CR 702.151a
   - Attempt to activate reconfigure during opponent's turn or with non-empty stack
   - Assert: Error / rejected

4. **`test_reconfigure_cant_attach_to_self`** -- CR 301.5c
   - Attempt to reconfigure targeting itself
   - Assert: Error / rejected

5. **`test_reconfigure_equipped_creature_leaves_battlefield`** -- CR 704.5n
   - Attach Equipment via reconfigure to a creature
   - Remove the creature from the battlefield
   - Assert: SBA unattaches the Equipment, `is_reconfigured = false`
   - Assert: Equipment is now a creature again

6. **`test_reconfigure_grants_ability_to_equipped_creature`** -- Lizard Blades test
   - Lizard Blades (double strike) attached to a vanilla creature
   - Assert: equipped creature has double strike (from static ability)
   - Assert: Lizard Blades itself is NOT a creature

7. **`test_reconfigure_negative_not_attached`** -- CR 702.151a
   - Attempt to activate unattach when not attached
   - Assert: Error / rejected

8. **`test_reconfigure_multiplayer_controller_check`** -- CR 702.6a analog
   - Attempt to reconfigure-attach to an opponent's creature
   - Assert: Error / rejected

**Pattern**: Follow Equipment tests. Use `ObjectSpec::creature()` for targets, `ObjectSpec::card()` with `enrich_spec_from_def()` for the Equipment creature.

### Step 12: Card Definition (later phase)

**Suggested card**: Lizard Blades
- `{1}{R}`, Artifact Creature -- Equipment Lizard, 1/1
- Double strike
- Equipped creature has double strike.
- Reconfigure {2}

**File**: `crates/engine/src/cards/defs/lizard_blades.rs`
**Agent**: Use `card-definition-author` agent

### Step 13: Game Script (later phase)

**Suggested scenario**: Lizard Blades enters as a creature, attacks with double strike, then reconfigures onto another creature which gains double strike. Later the equipped creature dies and Lizard Blades reverts to a creature.

**Subsystem directory**: `test-data/generated-scripts/abilities/`

## Interactions to Watch

1. **March of the Machines + Reconfigure**: March makes all artifacts creatures. A reconfigured Equipment would be made a creature again by March (Layer 4). The SBA should unattach it immediately (ruling 2022-02-18).

2. **Humility + Reconfigure**: Humility removes all abilities (Layer 6). Per ruling, the "not a creature" effect persists even when reconfigure keyword is removed -- hence `is_reconfigured` flag approach instead of checking keyword presence.

3. **Equip vs Reconfigure**: Reconfigure is NOT an "equip ability" (ruling 2022-02-18). Cards referencing equip abilities (Fighter Class, Leonin Shikari) do not affect reconfigure.

4. **Brass Squire / other attach effects**: A Reconfigure Equipment can be attached to creatures by effects other than its own reconfigure ability. When attached by any means, `is_reconfigured` should be set if the Equipment has reconfigure. This is handled in Step 6 (the `AttachEquipment` handler checks for reconfigure keyword).

5. **Auras with "enchant creature"**: When the Equipment stops being a creature via reconfigure, any creature-only Auras fall off (existing SBA 704.5m handles this automatically).

6. **Protection from artifacts**: If the target creature gains protection from artifacts while equipped, SBA 704.5n (via protection check) unattaches the Equipment.

7. **Layer ordering**: The type removal is in Layer 4. If another effect in Layer 4 adds Creature type (e.g., Animate Dead making it a creature), timestamp ordering determines the final result.

## Design Decision: Reuse AttachEquipment or New Command?

**Decision: Reuse `Effect::AttachEquipment` for attach, add `Effect::DetachEquipment` for unattach.**

Rationale:
- The attach mechanic is identical to Equip: target creature you control, sorcery speed, same attachment state changes.
- No new Command variant needed -- both abilities go through the existing `Command::ActivateAbility` path with `ability_index`.
- The unattach ability needs a new Effect variant (`DetachEquipment`) because no existing effect clears `attached_to` without moving the object to another zone.
- This approach keeps the infrastructure minimal and reuses proven attach logic.

## Layer 4 Implementation Detail

The `is_reconfigured` flag is checked in `calculate_characteristics` at Layer 4:
- `if obj.is_reconfigured { remove Creature type + creature subtypes }`
- This runs BEFORE Layer 6 (ability removal by Humility), so the keyword check is irrelevant -- the flag persists.
- The flag is cleared only when the Equipment becomes unattached (DetachEquipment effect, SBA unattach, or zone change).
