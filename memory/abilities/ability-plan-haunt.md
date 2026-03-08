# Ability Plan: Haunt

**Generated**: 2026-03-08
**CR**: 702.55
**Priority**: P4
**Similar abilities studied**: Cipher (CR 702.99 -- `types.rs:1317-1331`, `resolution.rs:1626-1699`, `abilities.rs:4885-4952`, `tests/cipher.rs`), Champion (CR 702.72 -- `types.rs:1153-1164`, `abilities.rs:4323-4377`, `tests/champion.rs`)

## CR Rule Text

> **702.55. Haunt**
>
> **702.55a** Haunt is a triggered ability. "Haunt" on a permanent means "When this
> permanent is put into a graveyard from the battlefield, exile it haunting target
> creature." "Haunt" on an instant or sorcery spell means "When this spell is put
> into a graveyard during its resolution, exile it haunting target creature."
>
> **702.55b** Cards that are in the exile zone as the result of a haunt ability "haunt"
> the creature targeted by that ability. The phrase "creature it haunts" refers to the
> object targeted by the haunt ability, regardless of whether or not that object is
> still a creature.
>
> **702.55c** Triggered abilities of cards with haunt that refer to the haunted creature
> can trigger in the exile zone.

## Key Edge Cases

1. **Creature vs. Spell Haunt (CR 702.55a)**: Creature Haunt triggers "when this creature
   dies" (battlefield -> graveyard). Spell Haunt triggers "when this spell is put into a
   graveyard during its resolution" (stack -> graveyard on resolution). These are two
   distinct trigger points.

2. **Haunt targets a creature (CR 702.55a)**: The haunt exile trigger targets a creature.
   This means it can be responded to, the target can become illegal (hexproof, leaves
   battlefield), and protection from the haunt card's colors prevents targeting.

3. **"Creature it haunts" is permanent (CR 702.55b)**: The phrase "creature it haunts"
   refers to the targeted object regardless of whether it is still a creature. If the
   haunted object loses creature type, the haunt relationship persists.

4. **Triggers from exile (CR 702.55c)**: The second triggered ability ("When the creature
   it haunts dies, [effect]") fires from exile. This is unusual -- most triggers only
   fire on the battlefield. The engine must check exiled haunt cards when a creature dies.

5. **Object identity on zone change**: When the haunt creature dies, it goes to the
   graveyard first (as a new object, CR 400.7), then the haunt trigger exiles it (another
   new object). The haunting relationship is on the exiled object.

6. **Haunted creature dies**: When the haunted creature dies, the haunt card's trigger
   fires from exile. The haunt card stays in exile after (it does not return or move).

7. **Riftsweeper/Pull from Eternity ruling**: If the haunt card leaves exile (e.g.,
   Riftsweeper puts it into library), the haunt relationship ends. No further triggers.

8. **Necromancer's Magemark ruling**: If the haunt creature never reaches the graveyard
   (e.g., replacement effect exiles it instead), the haunt trigger never fires.

9. **Protection from instants ruling (Petrified Wood-Kin)**: Haunt targets a creature,
   so protection prevents the haunt exile trigger from targeting that creature.

10. **Multiplayer**: The haunt creature's controller chooses which creature to haunt.
    The target can be any creature on the battlefield, including opponents' creatures.
    When the haunted creature dies, the haunt card's controller (its owner in exile)
    controls the triggered ability.

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
**Action**: Add `KeywordAbility::Haunt` variant after `Cipher` (line ~1331).
**Discriminant**: 142 (verified: Cipher = 141, last in enum)
**Pattern**: Follow `KeywordAbility::Cipher` at line 1331.
**Doc comment**:
```
/// CR 702.55a: Haunt -- two linked triggered abilities.
///
/// On a creature: "When this creature dies, exile it haunting target creature."
/// On a spell: "When this spell is put into a graveyard during its resolution,
/// exile it haunting target creature."
///
/// The exiled card then has a second trigger: "When the creature it haunts dies,
/// [effect]." This trigger fires from exile (CR 702.55c).
///
/// Discriminant 142.
Haunt,
```

**File**: `crates/engine/src/state/hash.rs`
**Action**: Add hash arm for `KeywordAbility::Haunt => 142u8.hash_into(hasher)` after the Cipher arm.
**Pattern**: Follow `KeywordAbility::Cipher => 141u8.hash_into(hasher)` at line 676.

**Match arms**: Grep for all exhaustive `KeywordAbility` matches and add `Haunt` arm:
- `tools/replay-viewer/src/view_model.rs` (keyword display function)
- Any other exhaustive matches in `state/`, `rules/`, `cards/`

### Step 2: AbilityDefinition Variant (if needed)

**Assessment**: Haunt does NOT need its own `AbilityDefinition` variant. Unlike Cipher
(which has special resolution-time behavior for encoding), Haunt's behavior is entirely
trigger-driven:
- Trigger 1 (creature dies or spell resolves): exile the card haunting target creature.
- Trigger 2 (haunted creature dies): fire the card's second ability from exile.

Both triggers are dispatched via `PendingTriggerKind` and `StackObjectKind`. The card's
second ability (the effect) is defined as a normal `TriggeredAbilityDef` on the card
definition (with `TriggerCondition::HauntedCreatureDies` or similar).

**Decision**: No AbilityDefinition variant needed. Use `AbilityDefinition::Keyword(KeywordAbility::Haunt)`.

### Step 3: State Fields

**File**: `crates/engine/src/state/game_object.rs`
**Action**: Add `haunting_target: Option<ObjectId>` field to `GameObject`.
**Purpose**: When a haunt card is exiled haunting a creature, this field records which
creature it haunts. The ObjectId refers to the haunted creature on the battlefield.
**CR**: 702.55b -- "Cards that are in the exile zone as the result of a haunt ability
'haunt' the creature targeted by that ability."
**Pattern**: Follow `champion_exiled_card: Option<ObjectId>` at line ~616.
**Default**: `None`
**Reset on zone change**: Yes -- `move_object_to_zone` should NOT carry this field forward
(a new object in a new zone has no haunt relationship). BUT: the field is set AFTER the
zone move to exile (similar to how `encoded_cards` is set on the creature after cipher
encoding). So the flow is: (1) card moves to exile (new ObjectId), (2) set
`haunting_target` on the new exiled object, (3) done.

**File**: `crates/engine/src/state/hash.rs`
**Action**: Add `haunting_target` to `GameObject` hash.
**Pattern**: Follow `champion_exiled_card` hashing.

**Initialization sites** (add `haunting_target: None`):
- `builder.rs` (GameStateBuilder)
- `effects/mod.rs` (token creation)
- `resolution.rs` (permanent ETB)
- Any other `GameObject { ... }` construction sites

### Step 4: PendingTriggerKind Variants

**File**: `crates/engine/src/state/stubs.rs`
**Action**: Add two new variants to `PendingTriggerKind`:

```rust
/// CR 702.55a: Haunt trigger (creature dies or spell resolves).
/// Exiles the haunt card targeting a creature.
HauntExile,
/// CR 702.55c: Haunted creature dies trigger.
/// Fires the haunt card's second ability from exile.
HauntedCreatureDies,
```

**Action**: Add `haunt_source_object_id: Option<ObjectId>` field to `PendingTrigger`.
**Purpose**: For `HauntedCreatureDies`, carries the ObjectId of the exiled haunt card
so the trigger resolution can look up which effect to apply.
**Pattern**: Follow `cipher_encoded_object_id: Option<ObjectId>` at line ~409.
**Default**: `None` in all existing PendingTrigger construction sites.

### Step 5: StackObjectKind Variants

**File**: `crates/engine/src/state/stack.rs`
**Action**: Add two new variants:

```rust
/// CR 702.55a: Haunt exile trigger -- "When this creature dies / this spell
/// resolves, exile it haunting target creature."
///
/// At resolution: exile the haunt card (from graveyard for creatures, from
/// graveyard for spells) haunting the target creature. Auto-selects first
/// legal creature (MVP -- interactive choice deferred).
///
/// Discriminant 57.
HauntExileTrigger {
    /// The ObjectId of the haunt card (in graveyard, about to be exiled).
    haunt_card: ObjectId,
    /// The CardId of the haunt card (for registry lookup).
    haunt_card_id: CardId,
},

/// CR 702.55c: Haunted creature dies trigger -- fires the haunt card's
/// effect from exile when the creature it haunts dies.
///
/// At resolution: execute the haunt card's second triggered ability effect.
/// The effect is looked up from the card registry.
///
/// Discriminant 58.
HauntedCreatureDiesTrigger {
    /// The ObjectId of the exiled haunt card.
    haunt_source: ObjectId,
    /// The CardId of the haunt card (for registry lookup to find the effect).
    haunt_card_id: CardId,
},
```

**File**: `crates/engine/src/state/hash.rs`
**Action**: Add hash arms for both new SOK variants (discriminants 57 and 58).
**Pattern**: Follow `CipherTrigger` hashing at line ~2046.

**Match arms**: Add arms in ALL exhaustive SOK matches:
- `resolution.rs` (main resolution match + counter-spell match)
- `tools/replay-viewer/src/view_model.rs` (stack_kind_info)
- `tools/tui/src/play/panels/stack_view.rs` (stack display)

### Step 6: GameEvent Variant

**File**: `crates/engine/src/rules/events.rs`
**Action**: Add `HauntExiled` event variant:

```rust
/// Emitted when a haunt card is exiled haunting a creature (CR 702.55a).
/// The haunt card moves from graveyard to exile and the haunting relationship
/// is established.
HauntExiled {
    /// The player who controls the haunt card.
    controller: PlayerId,
    /// The ObjectId of the exiled haunt card (new ID after zone move, CR 400.7).
    exiled_card: ObjectId,
    /// The ObjectId of the creature being haunted.
    haunted_creature: ObjectId,
},
```

**File**: `crates/engine/src/state/hash.rs`
**Action**: Add hash arm for `GameEvent::HauntExiled` (discriminant 107).
**Pattern**: Follow `GameEvent::CipherEncoded` at line ~3092.

### Step 7: Trigger Wiring -- Creature Haunt (CR 702.55a, creature path)

**File**: `crates/engine/src/rules/abilities.rs`
**Action**: In the `CreatureDied` event handler (search for `GameEvent::CreatureDied`),
add a check for Haunt keyword on the dying creature:

```
// CR 702.55a: Haunt -- "When this creature dies, exile it haunting target creature."
// Check if the dying creature has Haunt (use layer-resolved characteristics from
// pre-death state, or check card registry for the card_id).
// Note: Use state field NOT keyword presence for linked ability (CR 607.2a) --
// BUT Haunt's initial die-trigger IS the keyword trigger, so keyword presence
// is correct here (unlike Champion LTB which uses champion_exiled_card).
```

The trigger should:
1. Check if the dead creature had `KeywordAbility::Haunt` (using the graveyard object's
   characteristics or the card registry).
2. Create a `PendingTrigger` with `kind: PendingTriggerKind::HauntExile`.
3. Set `haunt_source_object_id` to the new graveyard ObjectId.

**Pattern**: Follow Champion LTB trigger in `CreatureDied` handler (line ~4323).
**CR**: 702.55a -- the trigger fires when the creature dies.

**Important**: Also check `PermanentDestroyed` and `ObjectExiled` events for non-creature
haunt permanents that leave the battlefield via other means. However, per CR 702.55a,
haunt on a permanent specifically says "put into a graveyard from the battlefield" --
so only the dies path triggers haunt. If the creature is exiled directly (not dying),
haunt does NOT trigger. This is different from Champion which fires on any LTB.

### Step 8: Trigger Wiring -- Spell Haunt (CR 702.55a, spell path) [STRETCH GOAL]

**File**: `crates/engine/src/rules/resolution.rs`
**Action**: In the instant/sorcery resolution path (near line ~1626 where Cipher is
checked), add a check for Haunt on the resolving spell:

```
// CR 702.55a: Haunt on an instant or sorcery -- "When this spell is put into
// a graveyard during its resolution, exile it haunting target creature."
// After the spell's effects resolve and the card moves to the graveyard,
// create a HauntExile pending trigger.
```

The trigger should fire AFTER the spell resolves and the card is in the graveyard.
This is a triggered ability, not a replacement (unlike Cipher which exiles directly).

**Pattern**: Cipher's resolution-time encoding at line ~1636.
**Note**: Unlike Cipher (which exiles the card directly during resolution), Haunt creates
a triggered ability that goes on the stack. The card goes to the graveyard first, then
the trigger exiles it.

### Step 9: HauntExileTrigger Resolution

**File**: `crates/engine/src/rules/resolution.rs`
**Action**: Add a resolution arm for `StackObjectKind::HauntExileTrigger`:

```
// CR 702.55a: Resolve haunt exile trigger.
// 1. Find the haunt card (should be in graveyard).
// 2. Find a target creature on the battlefield (MVP: auto-select first legal).
// 3. Move the haunt card from graveyard to exile.
// 4. Set haunting_target on the new exiled object to the target creature's ObjectId.
// 5. Emit HauntExiled event.
```

**Edge cases**:
- If the haunt card is no longer in the graveyard (e.g., shuffled back), the trigger
  fizzles (do nothing).
- If no legal creature target exists, the trigger fizzles (card stays in graveyard).

**Pattern**: Follow `CipherTrigger` resolution at line ~4166.

### Step 10: Trigger Wiring -- Haunted Creature Dies (CR 702.55c)

**File**: `crates/engine/src/rules/abilities.rs`
**Action**: In the `CreatureDied` event handler, add a check for exiled haunt cards
that haunt the dying creature:

```
// CR 702.55c: "When the creature it haunts dies, [effect]."
// Scan exile zone for cards with haunting_target == dying creature's pre-death ObjectId.
// For each such card, create a PendingTrigger with kind: HauntedCreatureDies.
```

**Critical**: The `haunting_target` on the exiled card stores the ObjectId of the haunted
creature. When that creature dies, the dying creature gets a NEW ObjectId in the graveyard.
We need to match against the ORIGINAL battlefield ObjectId. The `CreatureDied` event
provides `object_id` (the battlefield ObjectId before death) -- use this to match.

Wait -- `CreatureDied` provides `new_grave_id` (the graveyard ObjectId after death).
The original battlefield ObjectId is NOT directly available in the event. However:
- The `haunting_target` was set to the creature's battlefield ObjectId.
- When the creature dies, `move_object_to_zone` creates a new object.
- We need to match `haunting_target` against the OLD ObjectId.

**Solution**: The `CreatureDied` event includes `object_id` (the battlefield ObjectId)
or we can derive it. Check the event struct. If it only has `new_grave_id`, we need to
scan exile for `haunting_target` matching any creature that just left the battlefield.
Alternative: store the original battlefield ObjectId in the event.

Let me check the CreatureDied event fields:

The event likely has the pre-death ObjectId available somehow. If not, we can:
1. Store the pre-death battlefield ObjectId in the PendingTrigger.
2. Or: iterate all exiled objects with `haunting_target.is_some()` and check if that
   target ObjectId is no longer on the battlefield (it died).

**Simpler approach**: In the `CreatureDied` handler, the event carries enough info to
identify the dying creature. Scan exile for haunt cards whose `haunting_target` matches
the dying creature's battlefield ObjectId. The `CreatureDied` event should include the
pre-death ObjectId -- verify from `events.rs`.

**Pattern**: Follow the Champion LTB handler which also fires on `CreatureDied`.

### Step 11: HauntedCreatureDiesTrigger Resolution

**File**: `crates/engine/src/rules/resolution.rs`
**Action**: Add a resolution arm for `StackObjectKind::HauntedCreatureDiesTrigger`:

```
// CR 702.55c: Resolve haunted-creature-dies trigger.
// 1. Find the haunt card in exile (verify it still exists and still has haunting_target).
// 2. Look up the card's effect from the registry.
// 3. Execute the effect (the card's "When X enters or the creature it haunts dies, [effect]").
// 4. The haunt card stays in exile (it does NOT leave exile or lose its haunting status).
```

**Effect lookup**: The card definition has a triggered ability with the effect. For
creature Haunt cards like Blind Hunter, the effect is "target player loses 2 life and
you gain 2 life." This is the same effect as the ETB trigger.

**MVP approach**: Look up the card definition from registry, find the first triggered
ability that matches `TriggerCondition::WhenEntersOrHauntedDies` (or equivalent), and
execute its effect.

**Alternative simpler approach**: Store the effect directly on the `HauntedCreatureDiesTrigger`
SOK (as `embedded_effect: Option<Box<Effect>>`), captured from the card definition at
trigger creation time. This avoids registry lookup at resolution.

**Pattern**: Follow the `CipherTrigger` resolution which looks up effects.

### Step 12: Counter-spell Match Arms

**File**: `crates/engine/src/rules/resolution.rs`
**Action**: Add both `HauntExileTrigger` and `HauntedCreatureDiesTrigger` to the
counter-spell catch-all match (line ~6522).
**Pattern**: Follow `CipherTrigger { .. }` at line 6537.

### Step 13: Unit Tests

**File**: `crates/engine/tests/haunt.rs`
**Tests to write**:

1. `test_haunt_creature_dies_exiles_haunting_creature`
   - CR 702.55a -- creature with Haunt dies, trigger fires, card exiled haunting a creature.
   - Setup: Haunt creature on battlefield, another creature on battlefield.
   - Kill the Haunt creature (SBA zero toughness or damage).
   - Resolve the haunt exile trigger.
   - Assert: Haunt card in exile with `haunting_target` set.
   - Assert: `HauntExiled` event emitted.

2. `test_haunt_haunted_creature_dies_triggers_effect`
   - CR 702.55c -- when the haunted creature dies, the haunt card's effect fires.
   - Setup: Haunt card already in exile haunting a creature.
   - Kill the haunted creature.
   - Resolve the haunted-creature-dies trigger.
   - Assert: Effect executed (e.g., life drain for Blind Hunter).

3. `test_haunt_creature_dies_no_creatures_available`
   - Negative test: creature with Haunt dies but no other creatures on battlefield.
   - Assert: Haunt trigger fizzles, card stays in graveyard.

4. `test_haunt_card_leaves_exile_no_trigger`
   - Edge case: Haunt card is removed from exile before haunted creature dies.
   - Assert: No HauntedCreatureDies trigger fires.

5. `test_haunt_creature_not_dies_no_trigger`
   - Negative test: Haunt creature is exiled directly (not dying), haunt does NOT trigger.
   - CR 702.55a specifies "put into a graveyard from the battlefield."

6. `test_haunt_full_lifecycle`
   - Integration test: Haunt creature ETBs (ETB effect fires), dies (haunt exile trigger),
     haunted creature dies (haunt effect fires again).
   - Validates the complete two-phase lifecycle.

7. `test_haunt_multiplayer_target_opponent_creature`
   - Multiplayer: Haunt creature dies, controller exiles it haunting an opponent's creature.
   - When opponent's creature dies, the Haunt card's controller controls the trigger.

**Pattern**: Follow tests in `tests/cipher.rs` and `tests/champion.rs`.

### Step 14: Card Definition

**Suggested card**: Blind Hunter (Creature -- Bat, {2}{W}{B}, 2/2, Flying, Haunt)
- Oracle: "When this creature enters or the creature it haunts dies, target player
  loses 2 life and you gain 2 life."
- Simple effect (DrainLife 2), good for testing.
- Has Flying (already implemented).

**Alternative for spell Haunt (stretch goal)**: Cry of Contrition (Sorcery, {B})
- Oracle: "Target player discards a card. Haunt. When the creature this card haunts
  dies, target player discards a card."

### Step 15: Game Script (later phase)

**Suggested scenario**: Blind Hunter lifecycle
- Cast Blind Hunter (ETB: opponent loses 2, you gain 2).
- Kill Blind Hunter (damage or SBA).
- Resolve haunt exile trigger (exile haunting opponent's creature).
- Kill the haunted creature.
- Resolve haunted-creature-dies trigger (opponent loses 2, you gain 2 again).

**Subsystem directory**: `test-data/generated-scripts/stack/` (trigger-heavy interaction)

## Implementation Complexity Assessment

**Haunt is notably complex** because it involves:
1. **Two separate trigger points**: creature death AND haunted-creature death.
2. **Triggers from exile**: unusual zone for trigger origination (CR 702.55c).
3. **Cross-zone state tracking**: `haunting_target` on exiled objects pointing to
   battlefield objects.
4. **Object identity fragility**: battlefield ObjectId dies with the creature; the
   exiled card's `haunting_target` becomes a dangling reference to a dead ObjectId.

**Recommended approach for haunting_target staleness**:
When a creature dies, scan ALL exiled objects for `haunting_target` matching the dead
creature's pre-death ObjectId. This is O(exiled cards) per creature death, which is
acceptable. Do NOT try to maintain a reverse index -- it adds complexity for minimal
performance gain in a game that rarely has more than ~20 exiled cards.

**Key difference from Cipher**: Cipher stores `encoded_cards` on the CREATURE (the
creature knows what cards are encoded on it). Haunt stores `haunting_target` on the
EXILED CARD (the card knows what creature it haunts). This is because:
- Cipher needs the creature to trigger (creature deals damage -> check encoded_cards).
- Haunt needs the exiled card to trigger (creature dies -> scan exile for haunt cards
  targeting that creature).

## Interactions to Watch

1. **Object identity (CR 400.7)**: The haunted creature's ObjectId becomes invalid when
   it dies. The exiled haunt card's `haunting_target` is now a stale ObjectId. This is
   expected -- the trigger fires based on the pre-death ObjectId match, then the
   `haunting_target` is effectively orphaned (no cleanup needed since the haunt card
   stays in exile with a stale reference, which is harmless).

2. **Replacement effects (Necromancer's Magemark ruling)**: If a replacement effect
   prevents the haunt creature from reaching the graveyard (e.g., Rest in Peace exiles
   it instead), the haunt die-trigger never fires because "dies" means "put into a
   graveyard from the battlefield" (CR 700.4). Rest in Peace replaces the graveyard
   destination with exile, so the creature never "dies" -- no haunt trigger.

3. **Commander zone return**: If a commander with Haunt dies, the owner may choose to
   return it to the command zone (SBA CR 903.9a). If moved to command zone instead of
   staying in graveyard, the haunt trigger still fires (it triggered on the death event)
   but at resolution, the card is in the command zone, not the graveyard -- the exile
   step fizzles.

4. **Tokens with Haunt**: Tokens cease to exist when they leave the battlefield (SBA
   CR 704.5d). A token with Haunt would trigger the die-trigger, but by the time the
   trigger resolves, the token no longer exists -- the exile step fizzles.

5. **Multiple Haunt instances**: A creature can be haunted by multiple haunt cards.
   When the haunted creature dies, ALL haunt cards targeting it fire their triggers.

## Discriminant Summary (verified from codebase)

| Enum | Variant | Discriminant |
|------|---------|-------------|
| KeywordAbility | Haunt | 142 |
| StackObjectKind | HauntExileTrigger | 57 |
| StackObjectKind | HauntedCreatureDiesTrigger | 58 |
| GameEvent | HauntExiled | 107 |
| PendingTriggerKind | HauntExile | (no explicit disc, enum variant) |
| PendingTriggerKind | HauntedCreatureDies | (no explicit disc, enum variant) |

**AbilityDefinition**: No new variant needed (use `Keyword(KeywordAbility::Haunt)`).
