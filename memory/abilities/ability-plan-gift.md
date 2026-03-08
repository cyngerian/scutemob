# Ability Plan: Gift

**Generated**: 2026-03-07
**CR**: 702.174
**Priority**: P4
**Similar abilities studied**: Bargain (KW, `casting.rs:2366-2408`, `Condition::WasBargained`), Offspring (KW 138, ETB trigger pattern in `resolution.rs:1311-1340`)

## CR Rule Text

702.174. Gift

702.174a Gift is a keyword that represents two abilities. It is written "Gift a [something]." The first ability is a static ability that functions while the card with gift is on the stack, and the second is either an ability that functions while the card with gift is on the stack or a triggered ability that functions while the card with gift is on the battlefield. The first ability is always "As an additional cost to cast this spell, you may choose an opponent." Paying a spell's gift cost follows the rules for paying additional costs in rules 601.2b and 601.2f-h. The second ability depends on the [something] listed as well as whether the object with the ability is a permanent or an instant or sorcery spell.

702.174b On a permanent, the second ability represented by gift is "When this permanent enters, if its gift cost was paid, [effect]." On an instant or sorcery spell, the second ability represented by gift is "If this spell's gift cost was paid, [effect]." The specific effect is defined by the [something] listed.

702.174c Some effects trigger whenever a player gives a gift. Such an ability triggers whenever an instant or sorcery spell that player controls whose gift cost was paid resolves. It also triggers whenever the gift triggered ability of a permanent that player controls resolves.

702.174d "Gift a Food" means the effect is "The chosen player creates a Food token."

702.174e "Gift a card" means the effect is "The chosen player draws a card."

702.174f "Gift a tapped Fish" means the effect is "The chosen player creates a tapped 1/1 blue Fish creature token."

702.174g "Gift an extra turn" means the effect is "The chosen player takes an extra turn after this one."

702.174h "Gift a Treasure" means the effect is "The chosen player creates a Treasure token."

702.174i "Gift an Octopus" means the effect is "The chosen player creates an 8/8 blue Octopus creature token."

702.174j For instant and sorcery spells, the effect of a gift ability always happens before any other spell abilities of the card. If the spell is countered or otherwise leaves the stack before resolving, the gift effect doesn't happen.

702.174k If a spell's controller declares the intention to pay a spell's gift cost, that spell's gift was promised.

702.174m If part of a spell's ability has its effect only if its gift was promised, and that part of the ability includes any targets, the spell's controller chooses those targets only if the gift was promised.

## Key Edge Cases

- **702.174a**: Gift cost is "choose an opponent" -- not a sacrifice, not mana. The additional cost is selecting an opponent (a player choice during casting).
- **702.174b**: Permanents use ETB triggered ability ("When this enters, if gift cost was paid..."); instants/sorceries use inline conditional ("If this spell's gift cost was paid...").
- **702.174j**: For instants/sorceries, the gift effect happens BEFORE any other spell effects. This means the opponent gets their gift first, then the caster's spell resolves its main effect.
- **702.174j**: If the spell is countered, the gift effect does NOT happen (no token/draw for the opponent).
- **702.174k**: "Promised" = declared intent to pay gift cost. This is the cast-time declaration.
- **702.174m**: Targeting conditional -- targets for the gift-conditional part are only chosen if gift was promised. This is a targeting optimization, not a resolution change.
- **Multiplayer**: The caster chooses ONE specific opponent (not "each opponent"). That opponent receives the gift.
- **Gift types**: Each gift type (Food, card, Fish, Treasure, Octopus, extra turn) has a specific effect. The card definition must specify which gift type.
- **702.174c**: "Whenever a player gives a gift" triggers fire after the gift effect resolves. Deferred (no cards with this trigger in scope).

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
**Action**: Add `KeywordAbility::Gift` variant (unit variant, discriminant 139).
**Pattern**: Follow `KeywordAbility::Bargain` at line ~942 (unit variant, no parameter).
**Doc comment**:
```
/// CR 702.174: Gift a [something] -- optional additional cost: choose an opponent.
///
/// "As an additional cost to cast this spell, you may choose an opponent."
/// If the gift cost was paid, the chosen opponent receives a gift (defined by
/// the card's AbilityDefinition::Gift variant) and the caster may get an
/// enhanced effect.
///
/// For permanents: ETB trigger "When this enters, if its gift cost was paid, [effect]."
/// For instants/sorceries: inline conditional "If this spell's gift cost was paid, [effect]."
///
/// Marker for quick presence-checking (`keywords.contains`).
/// The gift type is stored in `AbilityDefinition::Gift { gift_type }`.
/// The chosen opponent is provided via `CastSpell.gift_opponent`.
/// Cards check "if gift was given" via `Condition::GiftWasGiven`.
///
/// CR 702.174a: Multiple instances are redundant (only one opponent can be chosen).
///
/// Discriminant 139.
Gift,
```

**File**: `crates/engine/src/cards/card_definition.rs`
**Action 1**: Add `AbilityDefinition::Gift { gift_type: GiftType }` variant (discriminant 56).
**Doc comment**:
```
/// CR 702.174a: Gift a [something] -- two linked abilities.
///
/// First ability: "As an additional cost to cast this spell, you may choose an opponent."
/// Second ability (permanent): "When this enters, if its gift cost was paid, [effect]."
/// Second ability (instant/sorcery): "If this spell's gift cost was paid, [effect]."
///
/// The `gift_type` determines what the chosen opponent receives (CR 702.174d-i).
///
/// Discriminant 56.
Gift { gift_type: GiftType },
```

**Action 2**: Add `GiftType` enum in `card_definition.rs` (near `CumulativeUpkeepCost` or similar small enums):
```rust
/// CR 702.174d-i: The specific gift given to the chosen opponent.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum GiftType {
    /// 702.174d: "The chosen player creates a Food token."
    Food,
    /// 702.174e: "The chosen player draws a card."
    Card,
    /// 702.174f: "The chosen player creates a tapped 1/1 blue Fish creature token."
    TappedFish,
    /// 702.174h: "The chosen player creates a Treasure token."
    Treasure,
    /// 702.174i: "The chosen player creates an 8/8 blue Octopus creature token."
    Octopus,
    /// 702.174g: "The chosen player takes an extra turn after this one."
    ExtraTurn,
}
```

**Action 3**: Add `Condition::GiftWasGiven` variant in `card_definition.rs` Condition enum (discriminant 14 in hash.rs):
```
/// CR 702.174b: "if this spell's gift cost was paid" / "if its gift cost was paid"
/// True when gift_opponent was chosen at cast time. Checked at resolution time
/// for instants/sorceries; at ETB trigger resolution for permanents.
GiftWasGiven,
```

### Step 2: State Fields

**File**: `crates/engine/src/state/game_object.rs`
**Action**: Add two fields to `GameObject`:
```rust
/// CR 702.174a: Whether the gift cost was paid when this permanent was cast.
/// Used by the GiftETB trigger at resolution. Reset on zone changes (CR 400.7).
#[serde(default)]
pub gift_was_given: bool,
/// CR 702.174a: The opponent chosen to receive the gift when this permanent was cast.
/// Used by the GiftETB trigger to determine which player gets the gift.
/// Reset to None on zone changes (CR 400.7).
#[serde(default)]
pub gift_opponent: Option<PlayerId>,
```

**File**: `crates/engine/src/state/stack.rs`
**Action**: Add two fields to `StackObject`:
```rust
/// CR 702.174a: Whether the gift cost was paid (an opponent was chosen).
#[serde(default)]
pub gift_was_given: bool,
/// CR 702.174a: The opponent chosen to receive the gift.
#[serde(default)]
pub gift_opponent: Option<PlayerId>,
```

**File**: `crates/engine/src/state/stack.rs`
**Action**: Add `StackObjectKind::GiftETBTrigger` variant (discriminant 54):
```rust
/// CR 702.174b: Gift ETB trigger -- "When this permanent enters, if its gift cost
/// was paid, [gift effect for the chosen opponent]."
///
/// Fired at ETB time when gift_was_given == true AND the permanent has
/// KeywordAbility::Gift in layer-resolved characteristics.
///
/// Discriminant 54.
GiftETBTrigger {
    /// The ObjectId of the permanent that entered.
    source_object: ObjectId,
    /// The card ID for registry lookup (LKI fallback).
    source_card_id: CardId,
    /// The opponent chosen at cast time to receive the gift.
    gift_opponent: PlayerId,
},
```

**File**: `crates/engine/src/state/stubs.rs`
**Action**: Add `PendingTriggerKind::GiftETB` variant.

**File**: `crates/engine/src/effects/mod.rs`
**Action**: Add two fields to `EffectContext`:
```rust
/// CR 702.174b: If true, this spell was cast with its gift cost paid.
pub gift_was_given: bool,
/// CR 702.174a: The opponent chosen to receive the gift.
pub gift_opponent: Option<PlayerId>,
```

### Step 3: Hash Updates

**File**: `crates/engine/src/state/hash.rs`
**Action**: Add hash entries for:
1. `GameObject`: `self.gift_was_given.hash_into(hasher)` and `self.gift_opponent.hash_into(hasher)` after `offspring_paid` hash.
2. `StackObject`: `self.gift_was_given.hash_into(hasher)` and `self.gift_opponent.hash_into(hasher)` after `offspring_paid` hash.
3. `KeywordAbility::Gift` match arm (discriminant 139) in the KW hash function.
4. `AbilityDefinition::Gift { gift_type }` match arm (discriminant 56) in the AbilDef hash function. Also hash the GiftType (impl HashInto for GiftType with Food=0, Card=1, TappedFish=2, Treasure=3, Octopus=4, ExtraTurn=5).
5. `StackObjectKind::GiftETBTrigger` match arm (discriminant 54) in the SOK hash function.
6. `Condition::GiftWasGiven` match arm (discriminant 14) in the Condition hash function.
7. `PendingTriggerKind::GiftETB` match arm in the PTK hash function.

### Step 4: Zone Change Resets

**File**: `crates/engine/src/state/mod.rs`
**Action**: In BOTH `move_object_to_zone` sites, add:
```rust
// CR 702.174a / CR 400.7: gift status is not preserved across zone changes.
gift_was_given: false,
gift_opponent: None,
```

**File**: `crates/engine/src/state/builder.rs`
**Action**: In the object initialization, add:
```rust
// CR 702.174a: test-placed objects are not cast spells; gift_was_given is false.
gift_was_given: false,
gift_opponent: None,
```

### Step 5: Casting Integration

**File**: `crates/engine/src/rules/casting.rs`
**Action 1**: Add `gift_opponent: Option<PlayerId>` parameter to `handle_cast_spell`.
**Action 2**: Add validation block (follow Bargain pattern at line ~2366):
```rust
// CR 702.174a / CR 601.2b: Gift -- validate the chosen opponent.
// Gift is optional: the player MAY choose an opponent as an additional cost.
let gift_chosen_opponent: Option<PlayerId> = if let Some(opponent) = gift_opponent {
    // Validate the spell has Gift keyword.
    if !chars.keywords.contains(&KeywordAbility::Gift) {
        return Err(GameStateError::InvalidCommand(
            "spell does not have gift (CR 702.174a)".into(),
        ));
    }
    // Validate the chosen player is an opponent.
    if opponent == player {
        return Err(GameStateError::InvalidCommand(
            "gift: must choose an opponent, not yourself (CR 702.174a)".into(),
        ));
    }
    // Validate the chosen player is in the game.
    if !state.active_players().contains(&opponent) {
        return Err(GameStateError::InvalidCommand(
            "gift: chosen opponent is not in the game (CR 702.174a)".into(),
        ));
    }
    Some(opponent)
} else {
    None
};
```
**Action 3**: Set `gift_was_given` and `gift_opponent` on the StackObject during creation (line ~3246):
```rust
// CR 702.174a: Record whether gift cost was paid and who was chosen.
was_gift_given: gift_chosen_opponent.is_some(),
gift_opponent: gift_chosen_opponent,
```
**Action 4**: Set `gift_was_given: false` and `gift_opponent: None` in all other StackObject creation sites (trigger/copy/etc.) -- follow the `was_bargained: false` pattern at lines ~3413, ~3485, ~3558, ~3625, ~3694.

### Step 6: Resolution Integration

**File**: `crates/engine/src/rules/resolution.rs`
**Action 1**: For instant/sorcery spells (line ~320), add gift effect execution BEFORE the main spell effect (CR 702.174j):
```rust
// CR 702.174j: For instant/sorcery spells, gift effect happens BEFORE other effects.
if stack_obj.gift_was_given {
    if let Some(opponent) = stack_obj.gift_opponent {
        execute_gift_effect(state, &stack_obj, opponent, &mut events)?;
    }
}
// Then set gift context for Condition::GiftWasGiven:
ctx.gift_was_given = stack_obj.gift_was_given;
ctx.gift_opponent = stack_obj.gift_opponent;
```
Create helper function `execute_gift_effect` that looks up the card's `AbilityDefinition::Gift { gift_type }` and produces the appropriate effect (create Food/Treasure/Fish/Octopus token, draw card, or grant extra turn) for the chosen opponent.

**Action 2**: For permanents (line ~430), propagate gift fields to the permanent:
```rust
// CR 702.174a: Transfer gift status from stack to permanent for ETB trigger.
obj.gift_was_given = stack_obj.gift_was_given;
obj.gift_opponent = stack_obj.gift_opponent;
```

**Action 3**: For permanents, add GiftETB trigger creation (follow Offspring pattern at line ~1311):
```rust
// CR 702.174b: Gift ETB trigger -- "When this permanent enters, if its
// gift cost was paid, [give the gift to the chosen opponent]."
let has_gift = {
    let chars = crate::rules::layers::calculate_characteristics(state, new_id);
    chars.map(|c| c.keywords.contains(&KeywordAbility::Gift)).unwrap_or(false)
};
let permanent_gift_was_given = state.objects.get(&new_id)
    .map(|o| o.gift_was_given).unwrap_or(false);
let permanent_gift_opponent = state.objects.get(&new_id)
    .and_then(|o| o.gift_opponent);
if has_gift && permanent_gift_was_given {
    if let Some(opponent) = permanent_gift_opponent {
        state.pending_triggers.push_back(PendingTrigger {
            source: new_id,
            ability_index: 0,
            controller: stack_obj.controller,
            kind: PendingTriggerKind::GiftETB,
            triggering_event: None,
            entering_object_id: None,
            // ... other fields None
        });
    }
}
```

**Action 4**: Add `StackObjectKind::GiftETBTrigger` resolution arm (follow Offspring pattern at line ~3631). The resolution reads the card's `AbilityDefinition::Gift { gift_type }` from the registry, determines the gift effect, and executes it for the stored `gift_opponent`.

**Action 5**: Add `GiftETBTrigger` to the countering catch-all arm (line ~6023).

### Step 7: Effect Context Propagation

**File**: `crates/engine/src/effects/mod.rs`
**Action 1**: Add `gift_was_given` and `gift_opponent` fields to `EffectContext` (after `was_bargained`).
**Action 2**: Initialize to `false`/`None` in all `EffectContext` creation sites.
**Action 3**: Add `Condition::GiftWasGiven` match arm in `evaluate_condition`:
```rust
Condition::GiftWasGiven => ctx.gift_was_given,
```

### Step 8: Gift Effect Helper

**File**: `crates/engine/src/rules/resolution.rs` (or a new helper)
**Action**: Create `execute_gift_effect` that reads the `GiftType` from the card's `AbilityDefinition::Gift` and produces the effect for the chosen opponent:
- `GiftType::Food` -- create a Food token (artifact token with "2, T, Sacrifice: Gain 3 life") controlled by the opponent
- `GiftType::Card` -- draw a card for the opponent
- `GiftType::TappedFish` -- create a tapped 1/1 blue Fish creature token controlled by the opponent
- `GiftType::Treasure` -- create a Treasure token controlled by the opponent
- `GiftType::Octopus` -- create an 8/8 blue Octopus creature token controlled by the opponent
- `GiftType::ExtraTurn` -- grant an extra turn to the opponent

For the initial implementation, focus on `GiftType::Card` (draw) and `GiftType::Treasure` (token) as the simplest to implement with existing infrastructure. Food/Fish/Octopus require token templates; ExtraTurn requires extra-turn infrastructure.

### Step 9: Exhaustive Match Updates

**File**: `tools/replay-viewer/src/view_model.rs`
**Action**: Add match arms for:
1. `KeywordAbility::Gift` in the keyword display function
2. `StackObjectKind::GiftETBTrigger` in `stack_kind_info()`

**File**: `tools/tui/src/play/panels/stack_view.rs`
**Action**: Add match arm for `StackObjectKind::GiftETBTrigger`.

### Step 10: CastSpell Command Routing

**File**: `crates/engine/src/rules/engine.rs`
**Action**: Add `gift_opponent` parameter to `Command::CastSpell` variant and route it through to `handle_cast_spell`. Follow the pattern for `bargain_sacrifice` routing.

### Step 11: Unit Tests

**File**: `crates/engine/tests/gift.rs`
**Tests to write**:
- `test_gift_basic_instant_card_draw` -- Cast an instant with Gift a card, choose an opponent, verify opponent draws a card BEFORE the spell's main effect resolves (CR 702.174j). Verify `Condition::GiftWasGiven` evaluates to true.
- `test_gift_not_paid_instant` -- Cast an instant with Gift but do NOT choose an opponent. Verify the gift effect does NOT happen. Verify `Condition::GiftWasGiven` evaluates to false.
- `test_gift_permanent_etb_trigger` -- Cast a creature with Gift a Treasure, choose an opponent. Verify the ETB trigger fires and the opponent gets a Treasure token.
- `test_gift_permanent_not_paid` -- Cast a creature with Gift but do NOT choose an opponent. Verify no ETB trigger fires.
- `test_gift_countered_no_effect` -- Cast a spell with Gift, choose an opponent, but counter the spell before resolution. Verify no gift effect happens (CR 702.174j).
- `test_gift_invalid_self_rejected` -- Attempt to choose yourself as the gift opponent. Verify error.
- `test_gift_multiplayer_choose_specific_opponent` -- In 4-player game, cast with Gift and choose a specific opponent (not just "any"). Verify only that opponent receives the gift.
**Pattern**: Follow tests in `crates/engine/tests/offspring.rs` for structure (card definition setup, cast, resolve, assert).

### Step 12: Helpers Export

**File**: `crates/engine/src/cards/helpers.rs`
**Action**: Add `GiftType` to the re-exports if card definitions need it.

### Step 13: Card Definition (later phase)

**Suggested card**: A simple Gift a card or Gift a Treasure creature. The MCP database did not return specific Foundation set Gift cards, so the card-definition-author agent should look up a card with Gift keyword and author it.
**Card lookup**: use `card-definition-author` agent with a Foundations Gift card.

### Step 14: Game Script (later phase)

**Suggested scenario**: 4-player game where P1 casts a creature with Gift a Treasure, chooses P3 as the gift opponent. Assert P3 gets a Treasure token on the battlefield after the ETB trigger resolves. Assert P1's creature is on the battlefield.
**Subsystem directory**: `test-data/generated-scripts/stack/` or `test-data/generated-scripts/baseline/`

## Interactions to Watch

- **Gift + Panharmonicon**: Gift's ETB trigger is "When this permanent enters" -- Panharmonicon would double it if the permanent is a creature. The doubled trigger should fire twice, giving the opponent two gifts. However, `SelfEntersBattlefield` triggers are NOT doubled by `doubler_applies_to_trigger` per MEMORY.md. This is a known gap; document but do not fix.
- **Gift + Stifle**: The GiftETBTrigger goes on the stack and can be countered (Stifle/Trickbind). If countered, no gift is given. Handle via the catch-all countering arm.
- **Gift + Humility/Dress Down**: If the permanent loses Gift before the ETB trigger resolves, the intervening-if check fails and no gift is given (CR 603.4). Follow the Offspring pattern for this check.
- **Gift + Copy**: A copy of a Gift spell on the stack should inherit `gift_was_given` and `gift_opponent` if the original was cast with gift paid. However, copies are not cast (CR 707.10), so `gift_was_given` should be false for copies. Set `gift_was_given: false` in copy creation sites.
- **Instant/sorcery gift ordering**: CR 702.174j mandates the gift effect happens FIRST. The implementation must execute the gift effect before calling `execute_effect` for the spell's main effect.
- **702.174c "Whenever a player gives a gift"**: This is a separate trigger that fires when the gift effect resolves. Defer implementation -- no cards in scope use this trigger pattern.
