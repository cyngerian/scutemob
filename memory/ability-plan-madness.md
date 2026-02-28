# Ability Plan: Madness

**Generated**: 2026-02-27
**CR**: 702.35
**Priority**: P3
**Similar abilities studied**: Flashback (CR 702.34) — `rules/casting.rs`, `state/stack.rs`, `state/types.rs`, `state/hash.rs`, `tests/flashback.rs`

## CR Rule Text

**702.35.** Madness

**702.35a** Madness is a keyword that represents two abilities. The first is a static ability that functions while the card with madness is in a player's hand. The second is a triggered ability that functions when the first ability is applied. "Madness [cost]" means "If a player would discard this card, that player discards it, but exiles it instead of putting it into their graveyard" and "When this card is exiled this way, its owner may cast it by paying [cost] rather than paying its mana cost. If that player doesn't, they put this card into their graveyard."

**702.35b** Casting a spell using its madness ability follows the rules for paying alternative costs in rules 601.2b and 601.2f-h.

**702.35c** After resolving a madness triggered ability, if the exiled card wasn't cast and was moved to a public zone, effects referencing the discarded card can find that card. See rule 400.7k.

**400.7k** After resolving a madness triggered ability (see rule 702.35), if the exiled card wasn't cast and was moved to a public zone, effects referencing the discarded card can find that object.

## Key Edge Cases

1. **The discard replacement is a static ability, not a replacement effect on the permanent.** It functions while the card is in the player's hand. When discarded, the card goes to exile instead of graveyard. This is CR 702.35a's first ability.

2. **The madness triggered ability fires "when this card is exiled this way."** This is the second ability. It goes on the stack and when it resolves, the owner may cast the card by paying the madness cost. If they decline, the card goes to the graveyard.

3. **Madness ignores timing restrictions (CR ruling).** "Casting a spell with madness ignores the timing rules based on the card's card type." A sorcery with madness can be cast during an opponent's turn, or during combat, etc. This is a critical difference from flashback (which respects timing).

4. **The card counts as having been discarded (CR ruling on Fiery Temper).** "A card with madness that's discarded counts as having been discarded even though it's put into exile rather than a graveyard." Discard triggers still fire. The `CardDiscarded` event must still be emitted.

5. **Madness works from ANY discard source (CR ruling).** Cleanup step hand-size discard, paying a discard cost (cycling), spell/ability forcing discard -- all of these trigger madness.

6. **Madness trigger resolves before the spell that caused the discard (CR ruling).** "If you discard a card with madness to pay the cost of a spell or activated ability, that card's madness triggered ability (and the spell that card becomes, if you choose to cast it) will resolve before the spell or ability the discard paid for."

7. **If not cast, goes to graveyard -- no second chance (CR ruling).** "If you choose not to cast a card with madness when the madness triggered ability resolves, it's put into your graveyard. Madness doesn't give you another chance to cast it later."

8. **Madness cast resolves normally (CR ruling).** "A spell cast for its madness cost is put onto the stack like any other spell. It can be countered, copied, and so on. As it resolves, it's put onto the battlefield if it's a permanent card or into its owner's graveyard if it's an instant or sorcery card."

9. **Mana value is based on printed cost (CR 118.9c / CR ruling).** "The mana value of the spell is determined by only its mana cost, no matter what the total cost to cast that spell was."

10. **Multiplayer**: Madness works the same in multiplayer. The owner of the discarded card is the player who gets the option to cast. No APNAP ordering issues since it is a single player's trigger.

## Current State (from ability-wip.md)

- [ ] Step 1: Enum variant
- [ ] Step 2: Rule enforcement
- [ ] Step 3: Trigger wiring
- [ ] Step 4: Unit tests
- [ ] Step 5: Card definition
- [ ] Step 6: Game script
- [ ] Step 7: Coverage doc update

## Implementation Steps

### Step 1: Enum Variant & AbilityDefinition

Madness requires changes in three type files:

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/types.rs`
**Action**: Add `KeywordAbility::Madness` variant after `LivingWeapon` (line ~360)
**Pattern**: Follow `KeywordAbility::Flashback` at line 213-219

```rust
/// CR 702.35: Madness [cost] -- when discarded, exile instead of graveyard;
/// owner may cast for madness cost from exile.
///
/// Marker for quick presence-checking (`keywords.contains`).
/// The madness cost itself is stored in `AbilityDefinition::Madness { cost }`.
///
/// Two sub-abilities: (1) static replacement on discard (hand -> exile instead of graveyard),
/// (2) triggered ability "when exiled this way, may cast for [cost] or put into graveyard."
/// Casting ignores timing restrictions (sorceries can be cast at instant speed via madness).
Madness,
```

**File**: `/home/airbaggie/scutemob/crates/engine/src/cards/card_definition.rs`
**Action**: Add `AbilityDefinition::Madness { cost: ManaCost }` variant after `Bestow` (line ~183)
**Pattern**: Follow `AbilityDefinition::Flashback { cost }` at line 138-145

```rust
/// CR 702.35: Madness [cost]. When this card is discarded, it is exiled instead
/// of going to the graveyard. Then a triggered ability fires: the owner may cast
/// it by paying [cost] (an alternative cost, CR 118.9). If they decline, it goes
/// to the graveyard.
///
/// Cards with this ability should also include
/// `AbilityDefinition::Keyword(KeywordAbility::Madness)` for quick
/// presence-checking without scanning all abilities.
Madness { cost: ManaCost },
```

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/hash.rs`
**Action 1**: Add `KeywordAbility::Madness` hash arm (discriminant 48) after `LivingWeapon` (line ~385)

```rust
// Madness (discriminant 48) -- CR 702.35
KeywordAbility::Madness => 48u8.hash_into(hasher),
```

**Action 2**: Add `AbilityDefinition::Madness { cost }` hash arm (discriminant 13) after `Bestow` (line ~2424)

```rust
// Madness (discriminant 13) -- CR 702.35
AbilityDefinition::Madness { cost } => {
    13u8.hash_into(hasher);
    cost.hash_into(hasher);
}
```

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/stack.rs`
**Action 1**: Add `cast_with_madness: bool` field to `StackObject` after `was_bestowed` (line ~75)

```rust
/// CR 702.35a: If true, this spell was cast via madness from exile. The card
/// was exiled during a discard, and the owner chose to cast it by paying the
/// madness cost. Unlike flashback, madness does NOT change where the card goes
/// on resolution -- it resolves normally (permanent to battlefield, instant/sorcery
/// to graveyard).
///
/// Must always be false for copies (`is_copy: true`) -- copies are not cast.
#[serde(default)]
pub cast_with_madness: bool,
```

**Action 2**: Add `cast_with_madness` to the `StackObject` hash in `state/hash.rs` after `was_bestowed` hash (line ~1157)

```rust
// Madness (CR 702.35a) -- spell was cast via madness from exile
self.cast_with_madness.hash_into(hasher);
```

**Action 3**: Add `MadnessTrigger` variant to `StackObjectKind` enum after `EvokeSacrificeTrigger` (line ~152)

```rust
/// CR 702.35a: Madness triggered ability on the stack.
///
/// When a card with madness is discarded and exiled by the madness static
/// ability, this trigger fires: "When this card is exiled this way, its
/// owner may cast it by paying [cost] rather than paying its mana cost.
/// If that player doesn't, they put this card into their graveyard."
///
/// `exiled_card` is the ObjectId of the card in exile (new ID after zone move).
/// `madness_cost` is captured at trigger time from the card definition.
MadnessTrigger {
    source_object: ObjectId,
    exiled_card: ObjectId,
    madness_cost: crate::cards::card_definition::ManaCost,
    owner: PlayerId,
},
```

**Action 4**: Add `MadnessTrigger` to `StackObjectKind` hash in `state/hash.rs` (find the existing `StackObjectKind` hash impl and add after the `EvokeSacrificeTrigger` arm)

**Match arms**: Grep for all `StackObjectKind::EvokeSacrificeTrigger` match arms in `resolution.rs`, `hash.rs`, and any view-model code. Add the new `MadnessTrigger` arm to each.

### Step 2: Discard Replacement (Static Ability)

**CR 702.35a**: "If a player would discard this card, that player discards it, but exiles it instead of putting it into their graveyard."

This must be implemented at all 3 discard sites:

#### Site 1: Effect-based discard

**File**: `/home/airbaggie/scutemob/crates/engine/src/effects/mod.rs`
**Function**: `discard_cards` (line ~1906)
**Action**: Before `move_object_to_zone(card_id, ZoneId::Graveyard(player))`, check if the card has `KeywordAbility::Madness`. If so, exile instead. After exiling, push a `MadnessTrigger` onto the stack.

```rust
// CR 702.35a: Madness -- if the card has madness, exile it instead of graveyard.
let has_madness = state.object(card_id)
    .map(|obj| obj.characteristics.keywords.contains(&KeywordAbility::Madness))
    .unwrap_or(false);

let destination = if has_madness {
    ZoneId::Exile
} else {
    ZoneId::Graveyard(player)
};

if let Ok((new_id, _)) = state.move_object_to_zone(card_id, destination) {
    // CR ruling: "A card with madness that's discarded counts as having been discarded"
    events.push(GameEvent::CardDiscarded {
        player,
        object_id: card_id,
        new_id,
    });

    if has_madness {
        // CR 702.35a: Queue the madness triggered ability.
        // Look up the madness cost from the card definition.
        let madness_cost = get_madness_cost_from_state(state, card_id);
        push_madness_trigger(state, new_id, player, madness_cost);
    }
}
```

**Note**: The `CardDiscarded` event MUST still be emitted even when the card goes to exile (per ruling). Discard triggers fire normally.

#### Site 2: Cycling discard

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/abilities.rs`
**Function**: `handle_cycle_card` (line ~448)
**Action**: Same pattern. Before `state.move_object_to_zone(card, ZoneId::Graveyard(owner))`, check for madness keyword. If present, move to exile instead and queue the madness trigger.

**Important**: The `CardCycled` event should still fire regardless (cycling + madness both apply).

#### Site 3: Cleanup step hand-size discard

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/turn_actions.rs`
**Function**: `cleanup_actions` (line ~213)
**Action**: Same pattern. Check for madness before `state.move_object_to_zone(discard_id, graveyard_zone)`. If madness, move to exile. Still emit `DiscardedToHandSize` (the card was discarded).

**Note**: The `DiscardedToHandSize` event tracks the hand-size discard semantics. When madness applies, the `zone_to` field should be `ZoneId::Exile` instead of `ZoneId::Graveyard` to correctly reflect where the card went. Alternatively, add a new event variant -- but the simpler approach is to change the `zone_to` field.

#### Helper function

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/abilities.rs` (or a shared location)
**Action**: Add a helper to look up the madness cost from a card's definition, similar to `get_flashback_cost` in `casting.rs` (line ~660).

```rust
/// CR 702.35a: Look up the madness cost from the card's AbilityDefinition.
fn get_madness_cost(
    card_id: &Option<CardId>,
    registry: &CardRegistry,
) -> Option<ManaCost> {
    card_id.as_ref().and_then(|cid| {
        registry.get(cid.clone()).and_then(|def| {
            def.abilities.iter().find_map(|a| {
                if let AbilityDefinition::Madness { cost } = a {
                    Some(cost.clone())
                } else {
                    None
                }
            })
        })
    })
}
```

**Problem**: At the discard sites in `effects/mod.rs` and `turn_actions.rs`, the card registry may not be directly available. The `state.objects` has the card's `card_id` field. The registry is on `GameState` (check `state.registry` or however the registry is accessed at those sites).

**Action**: Check how `get_flashback_cost` accesses the registry. In `casting.rs`, the registry is passed as a parameter to `handle_cast_spell`. For the discard sites, the registry needs to be accessible from `GameState`. Grep for `registry` usage in `effects/mod.rs` and `turn_actions.rs` to determine the access pattern.

### Step 3: Madness Trigger Resolution

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/resolution.rs`
**Action**: Add a `StackObjectKind::MadnessTrigger` arm in `resolve_top_of_stack` (after `EvokeSacrificeTrigger`, line ~507).

When the madness trigger resolves:

1. Check if the exiled card is still in exile (CR 400.7 -- if it changed zones, the trigger does nothing).
2. The owner gets a choice: cast for madness cost, or put into graveyard.
3. **For the initial implementation**: Auto-cast if the owner has enough mana. If not, auto-decline (put into graveyard). A future Command variant (`ChooseMadness`) can add player choice later.
4. If casting: call `handle_cast_spell` with the card in exile, paying the madness cost as an alternative cost. The spell ignores timing restrictions.
5. If declining: `move_object_to_zone(exiled_card, ZoneId::Graveyard(owner))`.

```rust
StackObjectKind::MadnessTrigger {
    source_object: _,
    exiled_card,
    madness_cost,
    owner,
} => {
    let controller = stack_obj.controller;

    // CR 702.35a: Check if the card is still in exile (CR 400.7).
    let still_in_exile = state.objects.get(&exiled_card)
        .map(|obj| obj.zone == ZoneId::Exile)
        .unwrap_or(false);

    if still_in_exile {
        // For now: auto-decline (put into graveyard).
        // Future: ChooseMadness command for player agency.
        if let Ok((new_id, _)) = state.move_object_to_zone(exiled_card, ZoneId::Graveyard(owner)) {
            events.push(GameEvent::ObjectMovedZone {
                object_id: exiled_card,
                new_id,
                from: ZoneId::Exile,
                to: ZoneId::Graveyard(owner),
            });
        }
    }

    events.push(GameEvent::AbilityResolved {
        controller,
        stack_object_id: stack_obj.id,
    });
}
```

**Important design decision**: The full madness cast-from-exile flow requires either:

**(A) Auto-cast approach (simpler, MVP)**: When the trigger resolves, if the owner can pay the madness cost, automatically cast the spell. No player choice needed.

**(B) Command-based approach (correct, full)**: Add a `Command::CastSpellMadness` or reuse `CastSpell` with a new `cast_with_madness: bool` flag. The trigger resolution puts the game into a "waiting for madness choice" state. The owner submits a CastSpell command (from exile) or a DeclineMadness command.

**Recommended approach**: **(B)** -- reuse `Command::CastSpell` but modify `handle_cast_spell` in `casting.rs` to also allow casting from exile when:
- The card is in `ZoneId::Exile`
- The card has `KeywordAbility::Madness`
- There is a pending madness trigger (or we track a `madness_eligible` flag on the exiled object)

This mirrors how flashback works: the engine checks the card's zone and keyword to determine if the cast is legal. The difference is that madness casts from exile (not graveyard) and ignores timing restrictions.

### Step 3b: Casting from Exile via Madness

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/casting.rs`
**Action**: Modify the zone check in `handle_cast_spell` (line ~83-97) to also allow casting from exile when madness applies.

Currently:
```rust
if card_obj.zone != ZoneId::Hand(player)
    && !casting_from_command_zone
    && !casting_with_flashback
{
    return Err(...);
}
```

Add:
```rust
let casting_from_exile = card_obj.zone == ZoneId::Exile;
let casting_with_madness = casting_from_exile
    && card_obj.characteristics.keywords.contains(&KeywordAbility::Madness);

if card_obj.zone != ZoneId::Hand(player)
    && !casting_from_command_zone
    && !casting_with_flashback
    && !casting_with_madness
{
    return Err(...);
}
```

**Cost selection**: When `casting_with_madness`, use the madness cost instead of the mana cost. Add to the cost-determination block (line ~159-166 area):

```rust
// CR 702.35b: Madness -- pay madness cost instead of mana cost.
let casting_with_madness_cost = if casting_with_madness {
    get_madness_cost(&card_id, &registry)
} else {
    None
};
```

Then in the cost pipeline (after flashback/evoke):
```rust
// Madness cost (alternative cost, CR 118.9 -- mutually exclusive with flashback/evoke)
if let Some(madness_cost) = casting_with_madness_cost {
    base_cost = madness_cost;
}
```

**Timing override**: CR ruling says "Casting a spell with madness ignores the timing rules based on the card's card type." Modify the timing check (line ~142-156 area):

```rust
// CR 702.35 ruling: Madness ignores timing restrictions.
// A sorcery cast via madness can be cast any time, like an instant.
if casting_with_madness {
    // Skip sorcery-speed timing check entirely.
} else if !is_instant_speed && !is_active_player_main_phase_empty_stack {
    return Err(...);
}
```

**Stack object flag**: Set `cast_with_madness: true` on the `StackObject` (line ~493):

```rust
cast_with_madness: casting_with_madness,
```

**All other `StackObject` construction sites**: Grep for all existing `cast_with_flashback:` assignments and add `cast_with_madness: false` alongside each. These include:
- `casting.rs` line ~493, ~593, ~629
- `abilities.rs` line ~334, ~486, ~1126, ~1466
- `copy.rs` line ~174, ~346
- `resolution.rs` (if any StackObject construction exists there)

### Step 3c: Madness Trigger Queuing

When the discard replacement fires (Step 2), a `MadnessTrigger` must be pushed onto the stack. This needs to happen properly in the trigger system.

**Option 1 (direct push)**: After the discard replacement moves the card to exile, directly push a `MadnessTrigger` StackObject onto `state.stack_objects`. This is how `EvokeSacrificeTrigger` is handled.

**Option 2 (via check_triggers)**: Add a new `GameEvent` variant for madness exile and wire it through `check_triggers`. This is cleaner but more complex.

**Recommended**: Option 1 (direct push) for simplicity, matching the Evoke pattern.

**Implementation**: At each discard site (Step 2), after moving the card to exile:

```rust
// CR 702.35a: Push madness trigger onto the stack.
let trigger_id = state.next_object_id();
state.stack_objects.push_back(StackObject {
    id: trigger_id,
    controller: owner, // The owner of the card
    kind: StackObjectKind::MadnessTrigger {
        source_object: new_id, // The exiled card
        exiled_card: new_id,
        madness_cost: madness_cost.clone(),
        owner,
    },
    targets: vec![],
    cant_be_countered: false,
    is_copy: false,
    cast_with_flashback: false,
    kicker_times_paid: 0,
    was_evoked: false,
    was_bestowed: false,
    cast_with_madness: false,
});
```

### Step 3d: Madness Trigger Resolution (Full Flow)

When the `MadnessTrigger` resolves in `resolution.rs`:

1. Verify the exiled card is still in exile.
2. The game needs the owner to decide: cast or decline.
3. **For testability**: Introduce a `Command::ChooseMadness` command variant, or reuse the existing `CastSpell` flow.

**Simplest correct approach**: When the `MadnessTrigger` resolves, give the owner priority. The owner can then:
- Submit `Command::CastSpell { card: exiled_card, ... }` to cast via madness, OR
- Submit `Command::PassPriority` which will cause the card to go to graveyard.

This approach requires: when the madness trigger resolves AND the owner does not immediately cast, the card goes to graveyard as part of the resolution cleanup. But this is awkward because PassPriority advances the game past the trigger.

**Better approach**: Add a `Command::DeclineMadness { player, card }` command. When the `MadnessTrigger` resolves:
- Record a `pending_madness_choice` on the GameState (similar to `pending_zone_changes`).
- The engine waits for either `CastSpell` from exile or `DeclineMadness`.
- If `DeclineMadness`, move the card to graveyard.

**Simplest testable approach for MVP**: Auto-decline (put into graveyard). The test will verify:
1. Card with madness is exiled when discarded (not graveyard).
2. CardDiscarded event still fires.
3. Madness trigger is pushed onto stack.
4. When trigger resolves, card goes to graveyard (auto-decline).

Then a separate test verifies casting from exile by directly calling `CastSpell` on the exiled card (with madness keyword) before the trigger resolves. This tests the casting pathway.

**Final recommendation**: Implement both paths:
1. Auto-decline in `MadnessTrigger` resolution (card to graveyard).
2. Allow `CastSpell` from exile for madness cards in `casting.rs`.
3. Tests verify both paths: the discard-exile-trigger-graveyard flow, and the cast-from-exile flow.

The missing piece (player choice during trigger resolution) is a general engine limitation, not specific to madness. Future work can add interactive choice during resolution.

### Step 4: Unit Tests

**File**: `/home/airbaggie/scutemob/crates/engine/tests/madness.rs`
**Pattern**: Follow `tests/flashback.rs` structure

**Tests to write**:

1. **`test_madness_discard_goes_to_exile`** -- CR 702.35a
   - Card with madness in hand is discarded via `Effect::DiscardCards`.
   - Assert card is in exile (not graveyard).
   - Assert `CardDiscarded` event is still emitted.

2. **`test_madness_discard_without_madness_goes_to_graveyard`** -- CR 702.35a (negative)
   - Card without madness in hand is discarded.
   - Assert card is in graveyard (not exile).

3. **`test_madness_trigger_on_stack_after_discard`** -- CR 702.35a
   - Card with madness is discarded.
   - Assert a `MadnessTrigger` is on `state.stack_objects`.

4. **`test_madness_cast_from_exile`** -- CR 702.35a, CR 702.35b
   - Card with madness is in exile with the Madness keyword.
   - Player casts it via `CastSpell` from exile, paying madness cost.
   - Assert spell is on the stack.
   - Assert madness cost was charged (not mana cost).

5. **`test_madness_ignores_sorcery_timing`** -- CR ruling
   - Sorcery with madness in exile during opponent's turn.
   - Player casts it via `CastSpell` from exile.
   - Assert cast succeeds (sorcery timing ignored).

6. **`test_madness_decline_goes_to_graveyard`** -- CR 702.35a, CR ruling
   - Card with madness is exiled via discard.
   - Madness trigger resolves (auto-decline in MVP).
   - Assert card is in graveyard.

7. **`test_madness_mana_value_unchanged`** -- CR 118.9c
   - Card with madness cast from exile.
   - Assert mana value on stack is the printed mana cost, not the madness cost.

8. **`test_madness_cycling_triggers_madness`** -- CR 702.35a + cycling
   - Card with both madness and cycling in hand.
   - Player cycles it (discard as cost).
   - Assert card is in exile (not graveyard) because madness applies to the cycling discard.
   - Assert both `CardDiscarded` and `CardCycled` events fire.

9. **`test_madness_cleanup_discard_triggers_madness`** -- CR ruling
   - Card with madness discarded during cleanup (hand size).
   - Assert card goes to exile (not graveyard).

10. **`test_madness_cast_with_madness_flag_set_on_stack`** -- CR 702.35
    - Cast via madness from exile.
    - Assert `stack_obj.cast_with_madness == true`.

11. **`test_madness_non_madness_card_in_exile_cannot_cast`** -- Negative
    - Card without madness in exile.
    - Assert `CastSpell` from exile is rejected.

### Step 5: Card Definition (later phase)

**Suggested card**: Fiery Temper
- Oracle: "Fiery Temper deals 3 damage to any target. Madness {R}"
- Mana cost: {1}{R}{R}
- Type: Instant
- This is an instant (no timing restriction issues) with a simple effect (damage) and a much cheaper madness cost ({R} vs {1}{R}{R}), making it ideal for testing the cost difference.

**Alternative suggested card**: Big Game Hunter
- Oracle: "When this creature enters, destroy target creature with power 4 or greater. Madness {B}"
- Mana cost: {1}{B}{B}
- Type: Creature -- Human Rebel Assassin (1/1)
- Tests madness on a creature (permanent) -- verifies it goes to battlefield on resolution.

**Card lookup**: Use `card-definition-author` agent.

### Step 6: Game Script (later phase)

**Suggested scenario**: "Discard to madness, cast from exile"
- Player has Fiery Temper in hand.
- An effect forces a discard (e.g., opponent casts a spell that forces discard, or cleanup hand size).
- Fiery Temper goes to exile (madness replacement).
- Madness trigger goes on stack.
- Player casts Fiery Temper from exile for {R} (madness cost), targeting opponent.
- Spell resolves, deals 3 damage.

**Subsystem directory**: `test-data/generated-scripts/stack/`

## Interactions to Watch

1. **Cycling + Madness**: Both cycling's discard-as-cost and madness's exile replacement must work together. The card is discarded (cycling cost paid), but goes to exile (madness). Both `CardDiscarded` and `CardCycled` events fire. The cycling draw ability AND the madness trigger both go on the stack.

2. **Flashback + Madness on same card**: Unusual but possible. Flashback allows casting from graveyard; madness allows casting from exile. These are different zones and don't conflict, but the card should not have both applied simultaneously (you either discard with madness or cast with flashback from graveyard later).

3. **Commander zone-change SBA**: If a commander with madness is discarded, madness exiles it first (replacement). Then the commander SBA (CR 903.9) checks: the commander is in exile, so the owner may choose to move it to the command zone. If they do, the madness trigger finds the card no longer in exile and does nothing. This interaction is correct without special handling.

4. **Rest in Peace + Madness**: Rest in Peace replaces "would go to graveyard" with "exile." If a card with madness is discarded, madness already exiles it. If Rest in Peace is also in play, both want to exile -- they agree, so no conflict. The madness trigger fires normally.

5. **Leyline of the Void + Madness**: Similar to Rest in Peace. Leyline exiles opponent's cards that would go to graveyard. Madness also exiles on discard. Same destination -- no conflict.

6. **Split Second + Madness**: Split second prevents casting spells while it's on the stack. But the madness trigger is a triggered ability (uses the stack), and casting the madness spell happens during trigger resolution. The ruling says "If you discard a card with madness while a spell or ability is resolving, it moves immediately to exile. Continue resolving that spell or ability... Its madness triggered ability will be placed onto the stack once that spell or ability has completely resolved." So the madness trigger goes on the stack normally. However, if Split Second is still on the stack above the madness trigger, the player cannot cast spells. The madness trigger would resolve (since it's a trigger, not a spell), and the player would have to decline casting (can't cast with split second active) -- the card goes to graveyard.

7. **`cast_with_madness: false` on all existing StackObject constructions**: Must add this field everywhere a StackObject is constructed (at least 9 sites identified in Step 3b). Missing any site causes a compilation error (struct field not provided).

8. **`next_object_id()` availability**: The madness trigger push needs a new ObjectId. Verify `state.next_object_id()` is accessible at all 3 discard sites. It is used extensively throughout the codebase, so this should not be an issue.

## File Change Summary

| File | Action |
|------|--------|
| `crates/engine/src/state/types.rs` | Add `KeywordAbility::Madness` |
| `crates/engine/src/cards/card_definition.rs` | Add `AbilityDefinition::Madness { cost }` |
| `crates/engine/src/state/stack.rs` | Add `cast_with_madness: bool` to `StackObject`; add `MadnessTrigger` to `StackObjectKind` |
| `crates/engine/src/state/hash.rs` | Hash arms for new `KeywordAbility`, `AbilityDefinition`, `StackObject` field, `StackObjectKind` |
| `crates/engine/src/effects/mod.rs` | Modify `discard_cards` for madness replacement |
| `crates/engine/src/rules/abilities.rs` | Modify `handle_cycle_card` for madness replacement; add `get_madness_cost` helper |
| `crates/engine/src/rules/turn_actions.rs` | Modify `cleanup_actions` for madness replacement |
| `crates/engine/src/rules/casting.rs` | Allow casting from exile with madness; madness cost as alternative; timing override |
| `crates/engine/src/rules/resolution.rs` | Add `MadnessTrigger` resolution arm; add to fizzle/counter match arms |
| `crates/engine/src/rules/copy.rs` | Add `cast_with_madness: false` to StackObject constructions |
| `crates/engine/src/testing/replay_harness.rs` | Add `cast_spell_madness` action type; add `find_in_exile` helper |
| `crates/engine/tests/madness.rs` | 11 unit tests |
| `tools/replay-viewer/src/view_model.rs` | Add `MadnessTrigger` to StackObjectKind serialization |

## Risk Assessment

**Medium complexity**: Madness touches 3 discard sites + casting + resolution. Each site needs the same replacement logic. The main risk is missing a discard site or not handling the `cast_with_madness` field at all StackObject construction sites.

**Testing strategy**: Test the replacement (discard -> exile) separately from the casting (exile -> stack). This isolates bugs to one subsystem.

**MVP scope**: Auto-decline on trigger resolution is acceptable for P3 priority. Full player choice can be added when the engine supports interactive choices during trigger resolution (a general capability, not madness-specific).
