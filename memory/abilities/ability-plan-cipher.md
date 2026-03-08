# Ability Plan: Cipher

**Generated**: 2026-03-08
**CR**: 702.99
**Priority**: P4
**Similar abilities studied**: Champion (exile-link on GameObject), Storm/Casualty (copy-spell-on-stack via `copy::copy_spell_on_stack`), Ingest/Renown/Poisonous (combat-damage-to-player trigger dispatch in `abilities.rs:4546+`)

## CR Rule Text

702.99. Cipher

702.99a Cipher appears on some instants and sorceries. It represents two abilities. The first is a spell ability that functions while the spell with cipher is on the stack. The second is a static ability that functions while the card with cipher is in the exile zone. "Cipher" means "If this spell is represented by a card, you may exile this card encoded on a creature you control" and "For as long as this card is encoded on that creature, that creature has 'Whenever this creature deals combat damage to a player, you may copy the encoded card and you may cast the copy without paying its mana cost.'"

702.99b The term "encoded" describes the relationship between the card with cipher while in the exile zone and the creature chosen when the spell represented by that card resolves.

702.99c The card with cipher remains encoded on the chosen creature as long as the card with cipher remains exiled and the creature remains on the battlefield. The card remains encoded on that object even if it changes controller or stops being a creature, as long as it remains on the battlefield.

## Key Edge Cases

From card rulings (Hands of Binding, Last Thoughts, Call of the Nightwing):

1. **Encoding happens at resolution, after spell effects.** The card goes directly from the stack to exile. It never goes to the graveyard. (ruling 2013-04-15)
2. **Encoding is optional ("you may").** Player chooses a creature they control at resolution time. Not a target -- no targeting restrictions apply. (ruling 2013-04-15)
3. **Creature must be a creature BEFORE resolution.** A Keyrune that can become a creature must already be one. (ruling 2013-04-15)
4. **If spell doesn't resolve (fizzle/counter), cipher doesn't happen.** Card goes to graveyard normally. (ruling 2013-04-15)
5. **Copy is created in and cast from exile.** (ruling 2013-04-15)
6. **Cast the copy during trigger resolution.** Ignore timing restrictions (sorcery can be cast at instant speed). (ruling 2013-04-15)
7. **If you don't cast the copy, it ceases to exist at next SBA check.** No later chance. (ruling 2013-04-15)
8. **If another player gains control of the creature, THAT player controls the trigger and may cast the copy.** (ruling 2013-04-15)
9. **If creature deals combat damage to multiple players simultaneously, trigger fires once per player.** Each creates a separate copy. (ruling 2013-04-15)
10. **If creature loses the granted ability, trigger doesn't fire, but card stays encoded.** (ruling 2013-04-15)
11. **If creature leaves the battlefield, card stays exiled but is no longer encoded on anything.** (ruling 2013-04-15)
12. **Copies are NOT cast in the traditional sense for storm purposes** -- but per the ruling, "you may cast the copy," so it IS cast (triggers "whenever you cast" abilities). This differs from Storm copies.
13. **CR 702.99c: Encoding persists even if the object stops being a creature or changes controller**, as long as it remains on the battlefield.
14. **Flashback/jump-start override cipher.** If the spell was cast with flashback, CR 702.34a says "exile instead of putting it anywhere else" -- flashback takes priority. Same for jump-start (CR 702.133a). The cipher encoding choice should NOT be offered when cast with flashback/jump-start.
15. **Multiplayer:** Controller of creature controls the trigger. If creature changes controllers, the new controller casts the copy.

## Current State (from ability-wip.md)

- [ ] Step 1: Enum variant
- [ ] Step 2: Rule enforcement
- [ ] Step 3: Trigger wiring
- [ ] Step 4: Unit tests

## Implementation Steps

### Step 1: Enum Variant

**File**: `crates/engine/src/state/types.rs`
**Action**: Add `KeywordAbility::Cipher` variant (unit, no parameter)
**Discriminant**: 141 (after Saddle(u32) = 140)
**Pattern**: Follow `KeywordAbility::Gift` at the end of the enum (line ~1304)

```
/// CR 702.99a: Cipher -- two linked abilities. At resolution, exile this card
/// encoded on a creature you control. That creature gains "Whenever this creature
/// deals combat damage to a player, copy the encoded card and cast the copy
/// without paying its mana cost."
///
/// Discriminant 141.
Cipher,
```

**Also add**:

1. **`AbilityDefinition::Cipher`** in `crates/engine/src/cards/card_definition.rs`
   - Discriminant 57 (after Gift = 56)
   - No parameters needed -- cipher has no variable cost. Just a marker.
   - Pattern: Follow `AbilityDefinition::Gift` at line ~653

2. **`StackObjectKind::CipherTrigger`** in `crates/engine/src/state/stack.rs`
   - Discriminant 56 (after SaddleAbility = 55)
   - Fields: `source_creature: ObjectId` (the creature that dealt combat damage), `encoded_card_id: CardId` (the card definition to copy), `encoded_object_id: ObjectId` (the exiled card object)
   - Pattern: Follow `StackObjectKind::CasualtyTrigger` (copy-spell trigger)

3. **`PendingTriggerKind::CipherCombatDamage`** in `crates/engine/src/state/stubs.rs`
   - Add after `GiftETB` (line ~135)
   - This is the trigger kind for "deals combat damage to player" cipher triggers

4. **New field on `PendingTrigger`**: `cipher_encoded_card_id: Option<CardId>` and `cipher_encoded_object_id: Option<ObjectId>`
   - Only meaningful when `kind == PendingTriggerKind::CipherCombatDamage`
   - Pattern: Follow `champion_exiled_card: Option<ObjectId>` at line ~367

5. **New field on `GameObject`**: `encoded_cards: im::Vector<(ObjectId, CardId)>`
   - Tracks which exiled cipher cards are encoded on this permanent
   - `ObjectId` = the exiled card object; `CardId` = the card definition (needed to create copies)
   - Initialize to empty in `builder.rs`, `effects/mod.rs` (token creation), `resolution.rs`
   - Reset to empty in `move_object_to_zone` (CR 400.7)
   - Hash in `hash.rs`
   - Pattern: Follow `champion_exiled_card: Option<ObjectId>` on GameObject

### Step 2: Rule Enforcement (Resolution Path)

**File**: `crates/engine/src/rules/resolution.rs`
**Location**: Lines ~1608-1632 (the `else` branch for instant/sorcery resolution)
**Action**: Before the destination zone decision, check if the spell has Cipher and was not cast with flashback/jump-start. If so, and the controller has creatures on the battlefield, offer the cipher encoding choice.

**CR**: 702.99a -- "If this spell is represented by a card, you may exile this card encoded on a creature you control"

**Logic**:
```
// After effects execute, before zone move (line ~1608):
// 1. Check: spell has KeywordAbility::Cipher in characteristics
// 2. Check: NOT is_copy (copies are not real cards)
// 3. Check: NOT cast_with_flashback/jump-start (overrides cipher)
// 4. Check: controller has at least one creature on the battlefield
// 5. If all pass: deterministic MVP -- auto-encode on first available creature
//    (interactive choice is a future TODO, like Champion creature selection)
// 6. Move card to ZoneId::Exile instead of graveyard
// 7. Add (exiled_object_id, card_id) to the chosen creature's encoded_cards
// 8. Grant the creature a SelfDealsCombatDamageToPlayer triggered ability
// 9. Emit a GameEvent::CipherEncoded event
```

**Important**: The cipher encoding happens as part of the spell's resolution. The card goes directly from the stack to exile -- it never touches the graveyard. This modifies the existing destination logic at line ~1617.

**New GameEvent variant**: `CipherEncoded { card: ObjectId, creature: ObjectId, card_id: CardId }`
- Add to `crates/engine/src/rules/events.rs`
- Hash in `events.rs` HashInto impl

### Step 3: Trigger Wiring (Combat Damage)

**File**: `crates/engine/src/rules/abilities.rs`
**Location**: Lines ~4546-4563 (`GameEvent::CombatDamageDealt` handler)
**Action**: After the existing `collect_triggers_for_event` call for `SelfDealsCombatDamageToPlayer`, add cipher trigger dispatch.

**CR**: 702.99a -- "Whenever this creature deals combat damage to a player, you may copy the encoded card and you may cast the copy without paying its mana cost."

**Logic**:
```
// Inside the CombatDamageDealt handler, after Ingest/Renown/Poisonous:
// For each assignment where target is Player and amount > 0:
//   1. Look up the source creature on the battlefield
//   2. Check if creature has encoded_cards (non-empty)
//   3. For each (exiled_obj_id, card_id) in encoded_cards:
//      a. Verify the exiled card still exists in ZoneId::Exile (CR 702.99c)
//      b. Push a PendingTrigger with kind = CipherCombatDamage
//         - cipher_encoded_card_id = Some(card_id)
//         - cipher_encoded_object_id = Some(exiled_obj_id)
//         - controller = creature's current controller (NOT the original caster)
```

**File**: `crates/engine/src/rules/resolution.rs`
**Location**: After the existing `StackObjectKind` match arms (around line ~1800+)
**Action**: Add resolution for `StackObjectKind::CipherTrigger`

**Resolution logic**:
```
// CipherTrigger resolution:
// 1. Verify encoded_object_id still exists in exile (CR 702.99c)
// 2. Create a copy of the spell on the stack using card_id (NOT copy_spell_on_stack --
//    that copies an existing stack object; cipher creates a new spell from a card def)
// 3. The copy IS cast (ruling: "you may cast the copy") -- unlike Storm copies
// 4. Cast without paying mana cost (free cast)
// 5. If controller chooses not to cast, copy ceases to exist (SBA)
//
// MVP: Auto-cast the copy (deterministic). Target selection for targeted copies
// is a future TODO (same as Cascade's auto-cast pattern).
```

**Pattern reference**: Look at how Cascade creates and casts a spell from exile in `resolution.rs` `StackObjectKind::CascadeTrigger`. Cascade finds a card and casts it for free -- cipher does the same but from a known exiled card.

**File**: `crates/engine/src/rules/resolution.rs` (flush_pending_triggers)
**Action**: Add arm for `PendingTriggerKind::CipherCombatDamage` to create `StackObjectKind::CipherTrigger`

### Step 3b: Cleanup on Creature Leaving Battlefield

**File**: `crates/engine/src/state/mod.rs` (`move_object_to_zone`)
**Action**: When a permanent with `encoded_cards` leaves the battlefield, the encoded cards remain exiled but are no longer encoded (CR 702.99c). The `encoded_cards` field is reset to empty by the zone-change reset (CR 400.7). No additional cleanup needed for the exiled cards themselves -- they stay in exile.

**Also**: When an exiled cipher card leaves exile (e.g., pulled back by another effect), any creature that has it in `encoded_cards` should have that entry removed. This is an SBA or a check at trigger time. **MVP approach**: Check at trigger resolution time that the encoded card still exists in exile. If not, the trigger fizzles (no copy created). No SBA needed.

### Step 4: Hash and View Model Updates

**File**: `crates/engine/src/state/hash.rs`
- Add `encoded_cards` to `GameObject` HashInto impl (after `champion_exiled_card`)
- Add `CipherTrigger` arm to `StackObjectKind` HashInto (discriminant 56)
- Add `CipherEncoded` arm to `GameEvent` HashInto
- Add `Cipher` arm to `KeywordAbility` HashInto (discriminant 141)
- Add `Cipher` arm to `AbilityDefinition` HashInto (discriminant 57)
- Add `CipherCombatDamage` to `PendingTriggerKind` HashInto

**File**: `tools/replay-viewer/src/view_model.rs`
- Add `CipherTrigger` arm to `stack_kind_info()` (line ~592)
- Add `Cipher` arm to keyword display (line ~879)

**File**: `tools/tui/src/play/panels/stack_view.rs`
- Add `CipherTrigger` arm (line ~201)

**File**: `crates/engine/src/cards/helpers.rs`
- No changes needed unless new types are introduced (encoded_cards uses existing ObjectId/CardId)

### Step 5: Unit Tests

**File**: `crates/engine/tests/cipher.rs` (new file)
**Tests to write**:

1. **`test_cipher_basic_encode_on_creature`** -- CR 702.99a
   - Cast a cipher spell, verify it goes to exile (not graveyard)
   - Verify the chosen creature has a non-empty `encoded_cards`
   - Verify the exiled card exists in ZoneId::Exile

2. **`test_cipher_combat_damage_triggers_copy`** -- CR 702.99a
   - Creature with encoded card deals combat damage to a player
   - Verify a CipherTrigger appears on the stack
   - After resolution, verify the spell effect executes (e.g., draw a card)

3. **`test_cipher_no_creatures_goes_to_graveyard`** -- CR 702.99a ("you may")
   - Cast cipher spell when controller has no creatures
   - Verify card goes to graveyard (encoding is impossible, not offered)

4. **`test_cipher_creature_leaves_encoding_broken`** -- CR 702.99c
   - Encode on creature, then remove creature from battlefield
   - Verify encoded card stays in exile but no longer triggers

5. **`test_cipher_flashback_overrides_cipher`** -- CR 702.34a
   - Cast cipher spell with flashback
   - Verify card goes to exile (flashback exile), NOT encoded on a creature

6. **`test_cipher_copy_not_encodable`** -- CR 702.99a ("represented by a card")
   - Copy of cipher spell resolves
   - Verify no encoding happens (copies are not real cards)

7. **`test_cipher_controller_change`** -- ruling 2013-04-15
   - Encode on creature, then change creature's controller
   - New controller should control the trigger and cast the copy

8. **`test_cipher_multiple_encoded_cards`** -- Multiple cipher spells on same creature
   - Each deals-combat-damage event fires a separate trigger for each encoded card

9. **`test_cipher_no_combat_damage_no_trigger`** -- CR 603.2g
   - Creature with encoded card blocked by creature with protection; no damage dealt
   - Verify no cipher trigger fires

**Pattern**: Follow tests for Ingest (combat damage trigger) in `crates/engine/tests/` and Champion (exile-link state) for the encoding state verification.

### Step 6: Card Definition (later phase)

**Suggested card**: Last Thoughts
- Simple: "Draw a card. Cipher."
- Effect is straightforward (Effect::DrawCards)
- Good for testing the full pipeline without targeting complexity
- Mana cost: {3}{U}, Sorcery

**Alternative**: Hands of Binding
- Has a targeted effect: "Tap target creature an opponent controls. That creature doesn't untap during its controller's next untap step."
- More complex due to targeting on the copy
- Better for testing targeted cipher copies

### Step 7: Game Script (later phase)

**Suggested scenario**: Cast Last Thoughts targeting no one (sorcery, no targets), encode on a creature, then have that creature deal combat damage to trigger the copy.
**Subsystem directory**: `test-data/generated-scripts/stack/`

## Interactions to Watch

1. **Flashback/Jump-Start/Retrace/Aftermath override cipher.** All of these have "exile instead of putting anywhere else" clauses. Cipher encoding must NOT be offered when any of these alternative costs were used.

2. **Buyback interaction**: Buyback returns to hand; cipher exiles to encode. If both are present, player would need to choose. CR 702.27a says buyback returns to hand "instead of putting it into your graveyard." Cipher says "you may exile this card encoded on a creature." Since cipher is optional, the player could choose cipher (exile+encode) or decline cipher (buyback returns to hand). **MVP**: If buyback was paid, skip cipher (buyback takes priority). This can be refined later.

3. **Storm + Cipher**: Storm copies are not cast and are not real cards. Storm copies cannot be encoded (CR 702.99a: "represented by a card"). No interaction issue.

4. **Cascade + Cipher**: Cascade cast IS a real cast. If the cascaded spell has cipher, cipher should work normally at resolution.

5. **Copy of encoded creature**: If the creature is copied (e.g., Clone), the copy does NOT have the encoded cards -- `encoded_cards` is game state on the specific object, not a copiable characteristic. The copy gets no cipher triggers.

6. **Panharmonicon**: Cipher trigger is a combat damage trigger, not an ETB trigger. Panharmonicon does not double it.

7. **Multiple players damaged simultaneously** (e.g., redirected combat damage in multiplayer): Each player damaged creates a separate trigger instance. Each trigger creates its own copy. (ruling 2013-04-15)

## Discriminant Summary

| Type | Variant | Discriminant |
|------|---------|-------------|
| KeywordAbility | Cipher | 141 |
| AbilityDefinition | Cipher | 57 |
| StackObjectKind | CipherTrigger | 56 |

## Files Modified (Summary)

| File | Change |
|------|--------|
| `crates/engine/src/state/types.rs` | Add `KeywordAbility::Cipher` (disc 141) |
| `crates/engine/src/cards/card_definition.rs` | Add `AbilityDefinition::Cipher` (disc 57) |
| `crates/engine/src/state/stack.rs` | Add `StackObjectKind::CipherTrigger` (disc 56) |
| `crates/engine/src/state/stubs.rs` | Add `PendingTriggerKind::CipherCombatDamage` + fields on PendingTrigger |
| `crates/engine/src/state/game_object.rs` | Add `encoded_cards: im::Vector<(ObjectId, CardId)>` to GameObject |
| `crates/engine/src/state/hash.rs` | Hash new fields + enum variants |
| `crates/engine/src/state/mod.rs` | Reset `encoded_cards` in `move_object_to_zone` |
| `crates/engine/src/rules/resolution.rs` | Cipher encoding at instant/sorcery resolution + CipherTrigger resolution |
| `crates/engine/src/rules/abilities.rs` | Cipher trigger dispatch in CombatDamageDealt handler |
| `crates/engine/src/rules/events.rs` | Add `GameEvent::CipherEncoded` |
| `crates/engine/src/cards/builder.rs` | Initialize `encoded_cards` to empty |
| `crates/engine/src/effects/mod.rs` | Initialize `encoded_cards` on token creation |
| `tools/replay-viewer/src/view_model.rs` | Add CipherTrigger + Cipher arms |
| `tools/tui/src/play/panels/stack_view.rs` | Add CipherTrigger arm |
| `crates/engine/tests/cipher.rs` | New test file (9 tests) |
