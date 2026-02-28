# Ability Plan: Living Weapon

**Generated**: 2026-02-27
**CR**: 702.92
**Priority**: P3
**Similar abilities studied**: Afterlife (CR 702.135) in `builder.rs:530-560`, `tests/afterlife.rs`; AttachEquipment in `effects/mod.rs:1278-1383`

## CR Rule Text

702.92. Living Weapon

702.92a Living weapon is a triggered ability. "Living weapon" means "When this Equipment enters, create a 0/0 black Phyrexian Germ creature token, then attach this Equipment to it."

## Key Edge Cases

1. **Token enters as 0/0 but Equipment attaches before SBAs check** (Batterskull ruling 2020-08-07): "The Germ token enters the battlefield as a 0/0 creature and the Equipment becomes attached to it before state-based actions would cause the token to die. Abilities that trigger as the token enters the battlefield see that a 0/0 creature entered the battlefield." This means the create+attach must happen as a single atomic effect execution, NOT as two separate triggers/effects that allow SBA checks between them.

2. **Doubling Season interaction** (Batterskull ruling 2020-08-07): "If the living weapon trigger causes two Germs to be created (due to an effect such as that of Doubling Season), the Equipment becomes attached to one of them. The other will be put into your graveyard and subsequently cease to exist, unless another effect raises its toughness above 0." The engine must attach to exactly ONE token even if multiple are created.

3. **Equipment remains when Germ dies** (Batterskull ruling 2020-08-07): "If the Germ token is destroyed, the Equipment remains on the battlefield as with any other Equipment." Standard equipment behavior -- no special handling needed.

4. **Germ is a 0/0 black Phyrexian Germ creature token**: The token has:
   - Name: "Phyrexian Germ" (or just "Germ" -- oracle text says "Phyrexian Germ")
   - P/T: 0/0
   - Color: Black
   - Types: Creature
   - Subtypes: Phyrexian, Germ
   - No keywords

5. **Equipment can be re-equipped to other creatures** (Batterskull ruling 2020-08-07): "Like other Equipment, each Equipment with living weapon has an equip cost. You can pay this cost to attach an Equipment to another creature you control. Once the Germ token is no longer equipped, it will be put into your graveyard and subsequently cease to exist, unless another effect raises its toughness above 0." Standard equip + SBA behavior.

6. **Multiplayer**: No special multiplayer considerations beyond standard APNAP trigger ordering.

7. **CR 301.5b relevance**: "Equipment enter the battlefield like other artifacts. They don't enter the battlefield attached to a creature." Living Weapon's trigger fires AFTER the Equipment enters, and the attachment happens during trigger resolution. The Equipment briefly exists unattached.

## Current State (from ability-wip.md)

- [ ] Step 1: Enum variant
- [ ] Step 2: Rule enforcement (trigger wiring in builder.rs)
- [ ] Step 3: Effect infrastructure (CreateTokenAndAttach effect)
- [ ] Step 4: Unit tests
- [ ] Step 5: Card definition
- [ ] Step 6: Game script
- [ ] Step 7: Coverage doc update

## Implementation Steps

### Step 1: Enum Variant

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/types.rs`
**Action**: Add `KeywordAbility::LivingWeapon` variant after `Fear` (line ~353)
**Pattern**: Follow `KeywordAbility::BattleCry` (simple enum variant, no parameters)
**Doc comment**:
```rust
/// CR 702.92: Living Weapon -- "When this Equipment enters, create a 0/0
/// black Phyrexian Germ creature token, then attach this Equipment to it."
///
/// Implemented as a triggered ability. builder.rs auto-generates a
/// TriggeredAbilityDef from this keyword at object-construction time.
/// The trigger fires on SelfEntersBattlefield. The effect creates a Germ
/// token and attaches the source Equipment to it atomically.
LivingWeapon,
```

**Hash file**: `/home/airbaggie/scutemob/crates/engine/src/state/hash.rs`
**Action**: Add discriminant 47 after Fear (discriminant 46, line ~383):
```rust
// LivingWeapon (discriminant 47) -- CR 702.92
KeywordAbility::LivingWeapon => 47u8.hash_into(hasher),
```

**Match arms**: Grep for exhaustive `KeywordAbility` match expressions across the codebase. The runner must check:
- `hash.rs` (done above)
- `view_model.rs` in `tools/replay-viewer/` (keyword display name)
- Any other match on `KeywordAbility` that doesn't have a wildcard arm

### Step 2: New Effect Variant -- CreateTokenAndAttach

**File**: `/home/airbaggie/scutemob/crates/engine/src/cards/card_definition.rs`
**Action**: Add a new `Effect::CreateTokenAndAttachSource` variant after `AttachEquipment` (line ~415).

**Rationale**: A `Sequence([CreateToken, AttachEquipment])` pattern would NOT work because:
1. `CreateToken` does not expose the created token's ObjectId to subsequent effects in the sequence.
2. `EffectTarget` has no `LastCreatedToken` variant.
3. The CR ruling explicitly states the attachment happens before SBAs, meaning the create and attach must be atomic (no SBA check between them).

A dedicated effect variant keeps the implementation simple and correct.

```rust
/// CR 702.92a: Create a token and immediately attach the source Equipment to it.
///
/// Used by Living Weapon. The token creation and attachment happen as a single
/// atomic operation -- SBAs are not checked between token creation and attachment
/// (ruling: "The Germ token enters the battlefield as a 0/0 creature and the
/// Equipment becomes attached to it before state-based actions would cause the
/// token to die.").
///
/// If multiple tokens would be created (e.g., Doubling Season), the Equipment
/// attaches to the first one. The others are subject to SBAs normally.
CreateTokenAndAttachSource {
    spec: TokenSpec,
},
```

**File**: `/home/airbaggie/scutemob/crates/engine/src/effects/mod.rs`
**Action**: Add handler for `Effect::CreateTokenAndAttachSource` near the existing `CreateToken` handler (line ~332). The implementation:

```rust
Effect::CreateTokenAndAttachSource { spec } => {
    let mut first_token_id: Option<ObjectId> = None;
    for _ in 0..spec.count {
        let obj = make_token(spec, ctx.controller);
        if let Ok(id) = state.add_object(obj, ZoneId::Battlefield) {
            events.push(GameEvent::TokenCreated {
                player: ctx.controller,
                object_id: id,
            });
            events.push(GameEvent::PermanentEnteredBattlefield {
                player: ctx.controller,
                object_id: id,
            });
            if first_token_id.is_none() {
                first_token_id = Some(id);
            }
        }
    }
    // Attach source Equipment to the first created token (CR 702.92a).
    // If Doubling Season creates extras, only the first gets equipped (ruling).
    if let Some(token_id) = first_token_id {
        let equip_id = ctx.source;
        // Verify source is still on the battlefield and is an Equipment.
        let source_on_bf = state.objects.get(&equip_id)
            .map(|o| o.zone == ZoneId::Battlefield)
            .unwrap_or(false);
        if source_on_bf {
            // Detach from previous (should not be attached, but defensive).
            let prev_target_opt = state.objects.get(&equip_id).and_then(|o| o.attached_to);
            if let Some(prev_target) = prev_target_opt {
                if let Some(prev) = state.objects.get_mut(&prev_target) {
                    prev.attachments.retain(|&x| x != equip_id);
                }
            }
            // Attach to token.
            state.timestamp_counter += 1;
            let new_ts = state.timestamp_counter;
            if let Some(equip_obj) = state.objects.get_mut(&equip_id) {
                equip_obj.attached_to = Some(token_id);
                equip_obj.timestamp = new_ts;
            }
            if let Some(target_obj) = state.objects.get_mut(&token_id) {
                if !target_obj.attachments.contains(&equip_id) {
                    target_obj.attachments.push_back(equip_id);
                }
            }
            events.push(GameEvent::EquipmentAttached {
                equipment_id: equip_id,
                target_id: token_id,
                controller: ctx.controller,
            });
        }
    }
}
```

**CR**: 702.92a -- the "create ... then attach" is a single triggered ability effect.
**Note**: The attachment logic mirrors `Effect::AttachEquipment` (lines 1278-1383 of effects/mod.rs) but simplified since the target is always the just-created token (no need to resolve `EffectTarget`).

### Step 3: Trigger Wiring in builder.rs

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/builder.rs`
**Action**: Add Living Weapon trigger generation after the Extort block (line ~579). Pattern follows Afterlife (line 530-560).

```rust
// CR 702.92a: Living Weapon -- "When this Equipment enters, create a
// 0/0 black Phyrexian Germ creature token, then attach this Equipment
// to it."
// ETB trigger on the Equipment itself. Uses CreateTokenAndAttachSource
// to atomically create + attach before SBAs.
if matches!(kw, KeywordAbility::LivingWeapon) {
    triggered_abilities.push(TriggeredAbilityDef {
        trigger_on: TriggerEvent::SelfEntersBattlefield,
        intervening_if: None,
        description: "Living Weapon (CR 702.92a): When this Equipment enters, \
                      create a 0/0 black Phyrexian Germ creature token, then \
                      attach this Equipment to it."
            .to_string(),
        effect: Some(Effect::CreateTokenAndAttachSource {
            spec: crate::cards::card_definition::TokenSpec {
                name: "Phyrexian Germ".to_string(),
                power: 0,
                toughness: 0,
                colors: [Color::Black].into_iter().collect(),
                card_types: [CardType::Creature].into_iter().collect(),
                subtypes: [
                    SubType("Phyrexian".to_string()),
                    SubType("Germ".to_string()),
                ]
                .into_iter()
                .collect(),
                keywords: im::OrdSet::new(),
                count: 1,
                tapped: false,
                mana_color: None,
                mana_abilities: vec![],
            },
        }),
    });
}
```

**CR**: 702.92a -- trigger event is "when this Equipment enters" = `SelfEntersBattlefield`.

### Step 4: Unit Tests

**File**: `/home/airbaggie/scutemob/crates/engine/tests/living_weapon.rs`
**Pattern**: Follow `crates/engine/tests/afterlife.rs` structure.

**Tests to write**:

1. **`test_living_weapon_basic_creates_germ_and_attaches`** -- CR 702.92a
   - Create an Equipment with LivingWeapon on the battlefield (or cast it to trigger ETB).
   - More precisely: set up an Equipment with `LivingWeapon` keyword and lethal damage=0 (just placed on battlefield).
   - Use `with_keyword(KeywordAbility::LivingWeapon)` and an Equipment subtype.
   - After ETB trigger resolves: verify a 0/0 Phyrexian Germ token exists on the battlefield, and the Equipment's `attached_to` points to the token's ObjectId (and the token's `attachments` contains the Equipment).
   - Verify token characteristics: name "Phyrexian Germ", power 0, toughness 0, color Black, subtypes Phyrexian + Germ, card type Creature.
   - **Important**: The trigger fires on `PermanentEnteredBattlefield`, so the test must either:
     (a) Use a builder that places the Equipment on the battlefield with an ETB event (current pattern: place with damage; but Equipment doesn't die from damage), or
     (b) Cast the Equipment spell through the stack so it enters normally.
   - Best approach: cast the Equipment from hand. Set up a hand card with Equipment type, LivingWeapon keyword. Cast it, pass priority, it resolves and enters battlefield, LivingWeapon trigger goes on stack, pass priority, trigger resolves.
   - Simpler alternative for unit testing: Use `ObjectSpec` with `in_zone(ZoneId::Battlefield)` and manually fire the `SelfEntersBattlefield` trigger. However, the engine's builder does not auto-fire ETB triggers for objects placed directly on the battlefield -- they must enter through resolution. The afterlife tests avoid this by using death triggers, not ETB triggers.
   - **Recommended approach**: Create the Equipment in Hand, use `CastSpell` to put it on the stack, then `PassPriority` for all players to resolve it. The Equipment enters the battlefield, the LivingWeapon ETB trigger goes on the stack. Then `PassPriority` for all to resolve the trigger.

2. **`test_living_weapon_germ_survives_with_equipment_buff`** -- CR 702.92a + Batterskull ruling
   - Equipment has LivingWeapon + a static continuous effect giving equipped creature +4/+4.
   - After trigger resolves: Germ is 4/4 (0/0 + 4/4 from equipment), survives SBAs.
   - Pass priority again to confirm no SBA kills the Germ.

3. **`test_living_weapon_germ_dies_without_buff`** -- CR 702.92a + CR 704.5f
   - Equipment with LivingWeapon but NO P/T buff (or a buff that only gives +0/+0).
   - After trigger resolves: Germ is 0/0, SBA kills it. Equipment remains on battlefield unattached.
   - Verify Equipment's `attached_to` is `None` after Germ dies.

4. **`test_living_weapon_equip_to_other_creature`** -- Batterskull ruling
   - After LivingWeapon trigger resolves (Germ equipped), use the Equip ability to move Equipment to a different creature.
   - Germ loses equipment, becomes 0/0, dies via SBA.
   - Verify Equipment is now attached to the other creature.

5. **`test_living_weapon_equipment_stays_after_germ_dies`** -- Batterskull ruling
   - Germ dies (e.g., from damage or removal). Equipment remains on battlefield.
   - Verify Equipment is on battlefield, `attached_to` is `None`.

6. **`test_living_weapon_multiplayer`** -- CR 603.3
   - Two players each have Equipment with LivingWeapon entering simultaneously (e.g., both already placed with pending ETB -- or more realistically, test with a single player since simultaneous ETBs of different controllers' Equipment is unusual).
   - Alternative: one player's Equipment enters, verify the trigger is ordered correctly in APNAP.

**Note on test difficulty**: Testing ETB triggers requires the object to actually enter the battlefield through the resolution pipeline (not be pre-placed by the builder). This means the tests must cast the Equipment from hand. This requires:
- A `CardDefinition` for the Equipment (at least a minimal one via inline registration)
- Enough mana in the player's pool to cast it
- The full cast-resolve cycle

The runner should use the pattern from existing equipment tests if they exist, or create a minimal Equipment CardDefinition inline with `CardRegistry::new(vec![card_def])`.

### Step 5: Card Definition (later phase)

**Suggested card**: Batterskull
- 5 mana artifact Equipment
- Living weapon
- Equipped creature gets +4/+4 and has vigilance and lifelink
- {3}: Return Batterskull to its owner's hand
- Equip {5}

**Alternative simpler card**: Skinwing
- 4 mana artifact Equipment
- Living weapon
- Equipped creature gets +2/+2 and has flying
- Equip {6}

**Card lookup**: use `card-definition-author` agent

### Step 6: Game Script (later phase)

**Suggested scenario**: "Cast Batterskull, Living Weapon trigger creates Germ, Germ is 4/4 with vigilance and lifelink, attacks, deals combat damage, gains life."

**Subsystem directory**: `test-data/generated-scripts/combat/` or `test-data/generated-scripts/stack/`

## Interactions to Watch

1. **Equipment attachment + continuous effects (Layer system)**: When the Equipment attaches to the Germ via `CreateTokenAndAttachSource`, the static continuous effects (e.g., "+4/+4 and has vigilance and lifelink") must apply. The `EffectFilter::AttachedCreature` filter resolves via `source.attached_to`. The attachment must be set BEFORE `register_static_continuous_effects` runs for the trigger resolution. Since the attachment happens during effect execution (which is after resolution), the continuous effects should already be registered from the Equipment's own ETB resolution. Verify this works correctly.

2. **SBA timing**: The create+attach is atomic within the trigger resolution. After the trigger resolves, SBAs check. If the Equipment provides +X/+X, the Germ survives. If it does not provide toughness, SBA kills the 0/0 Germ. The Equipment detaches via SBA (CR 704.5n: "If an Equipment or Fortification is attached to an illegal permanent or to a player, it becomes unattached").

3. **Doubling Season / Parallel Lives**: These double token creation. The effect must handle multiple tokens gracefully -- attach to the first, let others die to SBA if they remain 0/0.

4. **Object identity (CR 400.7)**: The Germ token gets a new ObjectId when created. The Equipment's `attached_to` must reference this new ObjectId. When the Germ later dies, the token's old ObjectId is dead -- standard equipment unattach SBA handles this.

5. **Blink/flicker interaction**: If the Equipment is blinked (exiled and returned), it enters as a new object, triggering Living Weapon again. A new Germ is created. The old Germ (if still alive) loses equipment and dies to SBA if 0/0. This is standard behavior, no special handling.

6. **`view_model.rs` update**: The replay viewer's keyword display must handle `LivingWeapon`. Add a display name arm in the keyword match.
