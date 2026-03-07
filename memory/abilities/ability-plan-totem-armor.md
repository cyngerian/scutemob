# Ability Plan: Totem Armor (Umbra Armor)

**Generated**: 2026-03-07
**CR**: 702.89 (renamed from "Totem Armor" to "Umbra Armor" in Oracle updates)
**Priority**: P4
**Similar abilities studied**: Regenerate (replacement.rs, regenerate.rs tests)
**Discriminants**: KW 127, no new SOK needed, no new AbilDef needed (static ability)

## CR Rule Text

> **702.89. Umbra Armor**
>
> 702.89a Umbra armor is a static ability that appears on some Auras. "Umbra armor" means
> "If enchanted permanent would be destroyed, instead remove all damage marked on it and
> destroy this Aura."
>
> 702.89b Some older cards were printed with the ability "totem armor" or referenced that
> ability. The text of these cards has been updated in the Oracle card reference to refer to
> umbra armor instead.

Related destruction rules:

> **701.8a** To destroy a permanent, move it from the battlefield to its owner's graveyard.
>
> **701.8b** The only ways a permanent can be destroyed are as a result of an effect that uses
> the word "destroy" or as a result of the state-based actions that check for lethal damage
> (see rule 704.5g) or damage from a source with deathtouch (see rule 704.5h). If a permanent
> is put into its owner's graveyard for any other reason, it hasn't been "destroyed."
>
> **704.5f** If a creature has toughness 0 or less, it's put into its owner's graveyard.
> Regeneration can't replace this event. (NOT destruction -- umbra armor does NOT apply.)
>
> **704.5g** Lethal damage -- creature is destroyed. Umbra armor DOES apply.
>
> **704.5h** Deathtouch damage -- creature is destroyed. Umbra armor DOES apply.

## Key Edge Cases (from Hyena Umbra / Bear Umbra / Snake Umbra rulings)

1. **Mandatory effect**: Umbra armor is not optional. If the enchanted creature would be
   destroyed, the Aura must be destroyed instead. No player choice.
2. **NOT regeneration**: The creature does NOT tap and is NOT removed from combat. Effects
   that say "can't be regenerated" do NOT prevent umbra armor. This is a critical difference
   from the Regenerate implementation.
3. **Multiple umbra armor Auras**: If multiple Auras with umbra armor enchant the same
   creature, only ONE is destroyed. The creature's controller chooses which one. This
   produces a `NeedsChoice` replacement result when multiple apply.
4. **Simultaneous SBA destruction**: If a creature with deathtouch dealt lethal damage
   (704.5g + 704.5h would both destroy), umbra armor replaces ALL of them at once and
   saves the creature. Only one Aura is destroyed.
5. **Spell destroys both Aura and creature**: If a spell would destroy both the umbra armor
   Aura and its enchanted creature simultaneously, the umbra armor effect saves the creature.
   The Aura is destroyed by the spell AND by umbra armor simultaneously -- result is the same
   as destroying it once.
6. **Indestructible**: If the enchanted creature has indestructible, destruction simply has
   no effect. Umbra armor never triggers because there's nothing to replace.
7. **Does NOT protect against**: sacrifice, legend rule, toughness 0 or less (704.5f),
   exile, or any non-"destroy" removal.
8. **Damage removal is part of the replacement**: All damage is removed from the enchanted
   permanent as part of the replacement effect. `damage_marked = 0`, `deathtouch_damage = false`.
9. **Static ability on the Aura**: Umbra armor is a static ability that generates a
   continuous replacement effect while the Aura is on the battlefield. It is NOT a one-shot
   shield (unlike regeneration). As long as the Aura is on the battlefield, the replacement
   applies.

## Current State (from ability-wip.md)

- [ ] Step 1: Enum variant
- [ ] Step 2: Rule enforcement (replacement effect)
- [ ] Step 3: Trigger wiring (N/A -- static replacement, not a trigger)
- [ ] Step 4: Unit tests
- [ ] Step 5: Card definition
- [ ] Step 6: Game script
- [ ] Step 7: Coverage doc update

## Implementation Steps

### Step 1: Enum Variant

**File**: `crates/engine/src/state/types.rs`
**Action**: Add `KeywordAbility::UmbraArmor` variant (no parameters).
**Pattern**: Follow `KeywordAbility::Champion` at line ~1164 (the last KW variant).
**Line**: Add after Champion. Use the name `UmbraArmor` (the current CR name, not the old "TotemArmor").
**Note**: The engine names keywords per the current CR. The old name "Totem Armor" is mentioned only in 702.89b as errata history.

**File**: `crates/engine/src/state/hash.rs`
**Action**: Add `KeywordAbility::UmbraArmor => 127u8.hash_into(hasher)` to the KeywordAbility HashInto impl.
**Pattern**: Follow `KeywordAbility::Champion => 126u8.hash_into(hasher)` at line ~635.

**File**: `tools/replay-viewer/src/view_model.rs`
**Action**: Add arm `KeywordAbility::UmbraArmor => "Umbra Armor".to_string()` to the keyword display match.
**Pattern**: Follow existing keyword display arms around line ~708.
**Note**: No new `StackObjectKind` variant needed -- umbra armor is a static ability, not triggered.

**File**: `tools/tui/src/play/panels/stack_view.rs`
**Action**: No change needed -- no new `StackObjectKind` variant.

**File**: `crates/engine/src/cards/helpers.rs`
**Action**: No change needed -- `KeywordAbility` is already exported.

### Step 2: Rule Enforcement -- Replacement Effect

Umbra armor is a **static ability** on an Aura that creates a **continuous replacement effect**.
Unlike regeneration (which creates one-shot shields via `Effect::Regenerate`), umbra armor is
always active while the Aura is on the battlefield.

**Architecture Decision**: Umbra armor should NOT be implemented as a registered `ReplacementEffect`
in `state.replacement_effects`. Instead, it should be checked **inline** at each destruction site,
similar to how indestructible is checked. The reasons:

1. The replacement effect is tied to the Aura being on the battlefield -- it doesn't need
   registration/expiry tracking.
2. The Aura's `attached_to` field directly tells us which permanent it protects.
3. The check is simple: scan battlefield for Auras with `UmbraArmor` keyword attached to the
   permanent about to be destroyed.

**Implementation**:

#### Step 2a: Add helper function to `replacement.rs`

**File**: `crates/engine/src/rules/replacement.rs`
**Action**: Add a new function `check_umbra_armor` analogous to `check_regeneration_shield`.
**Location**: After `apply_regeneration` (~line 1695).

```
/// CR 702.89a: Check if an Aura with umbra armor can replace destruction.
///
/// Scans the battlefield for Auras with the UmbraArmor keyword that are
/// attached to the target permanent. Returns the ObjectId(s) of matching
/// Auras.
///
/// If exactly one Aura matches, it is auto-selected.
/// If multiple match, the enchanted creature's controller must choose (CR 616.1).
pub fn check_umbra_armor(state: &GameState, object_id: ObjectId) -> Vec<ObjectId>
```

This function:
1. Iterates `state.objects` for objects on the battlefield.
2. Filters for objects with `attached_to == Some(object_id)`.
3. Uses `calculate_characteristics` (layer-resolved) to check for `KeywordAbility::UmbraArmor`.
4. Returns the list of matching Aura ObjectIds.

#### Step 2b: Add apply function to `replacement.rs`

**File**: `crates/engine/src/rules/replacement.rs`
**Action**: Add `apply_umbra_armor` function.
**Location**: After `check_umbra_armor`.

```
/// CR 702.89a: Apply umbra armor replacement.
///
/// Instead of destroying the enchanted permanent:
/// 1. Remove all damage marked on the permanent (CR 702.89a).
/// 2. Clear deathtouch_damage flag.
/// 3. Destroy the Aura (move to owner's graveyard).
///
/// Unlike regeneration: the permanent is NOT tapped and NOT removed from combat.
pub fn apply_umbra_armor(
    state: &mut GameState,
    protected_id: ObjectId,
    aura_id: ObjectId,
) -> Vec<GameEvent>
```

This function:
1. Sets `damage_marked = 0` and `deathtouch_damage = false` on the protected permanent.
2. Calls `move_object_to_zone` to move the Aura to its owner's graveyard (this is destruction of the Aura).
3. Emits a new `GameEvent::UmbraArmorApplied { protected_id, aura_id }` event.
4. Emits `CreatureDied` (or appropriate event) for the Aura going to graveyard if applicable. Actually, the Aura is an enchantment, not a creature -- use the standard `move_object_to_zone` flow which handles zone-change events (including `AuraFellOff` or standard graveyard move). The aura is being destroyed, so standard zone-change replacement effects on IT should still apply (e.g., a commander Aura could redirect to command zone).

**Important**: The Aura's own destruction via umbra armor IS a "destroy" event (701.8a). If the Aura itself had umbra armor (impossible in practice since umbra armor only appears on Auras enchanting creatures, and Auras can't enchant themselves), it would NOT protect itself. But more relevantly: zone-change replacement effects (like commander redirect) DO apply to the Aura's movement to graveyard.

#### Step 2c: Wire into SBA destruction path

**File**: `crates/engine/src/rules/sba.rs`
**Action**: Add umbra armor check in the `dying` creature loop, AFTER the indestructible check
and AFTER the regeneration shield check, but BEFORE the zone-change replacement check.
**Location**: After line ~411 (after the regeneration `continue`).

The check order in the `for (id, is_destruction) in dying` loop should be:
1. Skip pending zone changes (existing)
2. Check regeneration shields (existing) -- if regeneration applies, `continue`
3. **NEW**: Check umbra armor -- if `is_destruction` and umbra armor Auras exist, apply and `continue`
4. Proceed with zone-change replacement effects and actual graveyard move (existing)

**CR justification**: Both regeneration and umbra armor are replacement effects that apply to
"would be destroyed." They follow the standard replacement ordering (CR 616.1). However, for
simplicity and because a permanent is unlikely to have both a regeneration shield AND an umbra
armor Aura at the same time, checking them sequentially is acceptable. If both apply simultaneously,
technically the controller chooses which to apply first (CR 616.1) -- this is a refinement that
can be deferred.

**Key detail**: Only check umbra armor when `is_destruction == true`. Zero-toughness (704.5f)
is NOT destruction, so umbra armor does not apply.

**Multiple umbra armor Auras**: If `check_umbra_armor` returns more than one Aura, the
controller must choose. For now, auto-select the first one (simplification). Add a TODO for
the multi-choice path.

#### Step 2d: Wire into Effect::DestroyPermanent path

**File**: `crates/engine/src/effects/mod.rs`
**Action**: Add umbra armor check in the `Effect::DestroyPermanent` handler, AFTER the
indestructible check and AFTER the regeneration shield check.
**Location**: After line ~573 (after the regeneration `continue`).

Same pattern as SBA: check `check_umbra_armor`, if non-empty, call `apply_umbra_armor` with
the first Aura and `continue`.

#### Step 2e: New GameEvent variant

**File**: `crates/engine/src/rules/events.rs`
**Action**: Add `GameEvent::UmbraArmorApplied { protected_id: ObjectId, aura_id: ObjectId }`.
**Location**: After `Regenerated` event variant (~line 868).
**Discriminant**: Use the next available GameEvent discriminant. Check existing max in hash.rs.

**File**: `crates/engine/src/state/hash.rs`
**Action**: Add hash impl for `GameEvent::UmbraArmorApplied`.
**Discriminant**: Next available after the current max GameEvent discriminant.

### Step 3: Trigger Wiring

**N/A** -- Umbra armor is a static ability that creates a replacement effect. It has no triggers.
No `StackObjectKind` variant needed. No `PendingTriggerKind` variant needed.

### Step 4: Unit Tests

**File**: `crates/engine/tests/umbra_armor.rs` (new file)

**Tests to write** (modeled on `crates/engine/tests/regenerate.rs`):

1. **`test_umbra_armor_prevents_destruction_by_spell`** -- CR 702.89a, 701.8b
   - Creature enchanted with an Aura that has UmbraArmor keyword.
   - DestroyPermanent effect targets the creature.
   - Creature survives on battlefield with damage cleared.
   - Aura is in graveyard.
   - UmbraArmorApplied event emitted.

2. **`test_umbra_armor_prevents_sba_lethal_damage`** -- CR 702.89a, 704.5g
   - Creature with 2 toughness has 2+ damage marked, enchanted by UmbraArmor Aura.
   - SBA check: creature would be destroyed by lethal damage.
   - Umbra armor fires: creature survives, damage cleared, Aura destroyed.

3. **`test_umbra_armor_prevents_sba_deathtouch_damage`** -- CR 702.89a, 704.5h
   - Creature with deathtouch_damage flag, enchanted by UmbraArmor Aura.
   - SBA check: creature would be destroyed by deathtouch.
   - Umbra armor fires: creature survives, damage + deathtouch flag cleared, Aura destroyed.

4. **`test_umbra_armor_does_not_prevent_zero_toughness`** -- CR 702.89a, 704.5f
   - Creature with toughness 0, enchanted by UmbraArmor Aura.
   - SBA check: creature goes to graveyard (not destruction -- umbra armor does NOT apply).
   - Aura also falls off (SBA 704.5m).

5. **`test_umbra_armor_does_not_tap_or_remove_from_combat`** -- Ruling: "not regeneration"
   - Creature in combat (attacking), enchanted by UmbraArmor Aura.
   - Destruction is replaced by umbra armor.
   - Creature remains UNTAPPED and remains in combat (unlike regeneration).

6. **`test_umbra_armor_not_consumed_by_indestructible`** -- Ruling: indestructible priority
   - Creature with Indestructible keyword, enchanted by UmbraArmor Aura.
   - DestroyPermanent effect targets the creature.
   - Indestructible prevents destruction entirely. Umbra armor is NOT consumed.
   - Aura remains on battlefield.

7. **`test_umbra_armor_does_not_prevent_sacrifice`** -- Ruling: not destruction
   - Creature enchanted by UmbraArmor Aura is sacrificed.
   - Sacrifice is not "destroy" (CR 701.8b) -- umbra armor does not apply.
   - Both creature and Aura end up in graveyard.

8. **`test_umbra_armor_removes_all_damage`** -- CR 702.89a
   - Creature with 3 damage marked (toughness 5), enchanted by UmbraArmor Aura.
   - Destroy spell targets creature.
   - After umbra armor: creature on battlefield with damage_marked = 0.

9. **`test_umbra_armor_event_emitted`** -- GameEvent::UmbraArmorApplied
   - Verify the event contains correct protected_id and aura_id.

**Pattern**: Follow `crates/engine/tests/regenerate.rs` for structure. Use same helper
functions (`find_on_battlefield`, `destroy_effect`). Create Aura objects with
`.with_keyword(KeywordAbility::UmbraArmor)` and `.attached_to(creature_id)`.

**Note on test setup**: Aura objects need:
- `ObjectSpec::card(owner, name)` with `.with_types([CardType::Enchantment])`
- `.with_keyword(KeywordAbility::UmbraArmor)`
- `.in_zone(ZoneId::Battlefield)`
- After building state, manually set `attached_to` on the Aura and `attachments` on the creature.
  OR use `ObjectSpec` builder if it supports attachment setup.

### Step 5: Card Definition (later phase)

**Suggested card**: Hyena Umbra
- {W}, Enchantment -- Aura
- Enchant creature
- Enchanted creature gets +1/+1 and has first strike.
- Umbra armor

**File**: `crates/engine/src/cards/defs/hyena_umbra.rs`
**Card lookup**: Use `card-definition-author` agent.

### Step 6: Game Script (later phase)

**Suggested scenario**: Creature enchanted with Hyena Umbra survives a destroy spell.
**Subsystem directory**: `test-data/generated-scripts/replacement/`
**Sequence**: Cast Hyena Umbra targeting a creature. Opponent casts a destroy spell on the
creature. Umbra armor replaces destruction: creature survives, Hyena Umbra goes to graveyard.

## Interactions to Watch

1. **Regeneration vs Umbra Armor**: Both replace "would be destroyed." If both apply to the
   same destruction event, the controller chooses which to apply first (CR 616.1). After one
   is applied, the destruction event is fully replaced -- the other doesn't apply. Current
   implementation checks regeneration before umbra armor sequentially; a TODO should note
   the proper CR 616.1 ordering for when both coexist.

2. **Indestructible takes priority**: Indestructible is checked first (not a replacement effect,
   but a static effect that makes destruction impossible). If a permanent is indestructible,
   destruction never happens, so neither regeneration nor umbra armor fires.

3. **"Can't be regenerated" (e.g., Wrath of God)**: This does NOT prevent umbra armor.
   Umbra armor is explicitly not regeneration (ruling). The engine's current "can't be
   regenerated" flag (if it exists) must not block umbra armor checks.

4. **Aura falling off**: When umbra armor destroys the Aura, the Aura goes to its owner's
   graveyard. Standard Aura fall-off SBAs do NOT apply here because the Aura is being
   destroyed directly, not because its enchanted permanent left. However, the Aura's own
   zone-change replacement effects DO apply (e.g., commander Aura redirect).

5. **Multiple destruction sources in same SBA batch**: If both 704.5g (lethal damage) and
   704.5h (deathtouch) would destroy the creature in the same SBA check, umbra armor replaces
   all of them with a single application (ruling). The engine's SBA loop collects dying
   creatures once and processes them -- a single `continue` after umbra armor handles this
   correctly.

6. **Simultaneous destruction of Aura and creature**: If a spell says "destroy all permanents,"
   the creature AND the umbra armor Aura would both be destroyed. Per rulings, umbra armor
   saves the creature. The Aura is destroyed by the spell AND by umbra armor simultaneously,
   but the result is the same as destroying it once. Implementation note: in the ForEach loop
   over permanents, if the creature is processed before the Aura, umbra armor fires and
   destroys the Aura. If the Aura is processed first, it's already in the graveyard when the
   creature is processed -- but umbra armor wouldn't have been checked yet. **This needs careful
   handling in the `ForEach` / `DestroyAll` path.** The correct behavior: check umbra armor for
   each creature being destroyed, even if the Aura would also be destroyed by the same effect.
   The Aura is destroyed as part of the replacement, preempting its own destruction by the
   original effect.
