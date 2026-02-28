# Ability Plan: Treasure Tokens

**Generated**: 2026-02-26
**CR**: 111.10a
**Priority**: P2
**Similar abilities studied**: Commander's Sphere (sacrifice-as-cost + mana production) in `definitions.rs:142-166`, Mind Stone (tap+sacrifice for draw) in `definitions.rs:195-220`, existing `CreateToken` + `make_token` in `effects/mod.rs:292-306,1676-1735`

## CR Rule Text

### CR 111.10
> Some effects instruct a player to create a predefined token. These effects use the definition below to determine the characteristics the token is created with. The effect that creates a predefined token may also modify or add to the predefined characteristics.

### CR 111.10a
> A Treasure token is a colorless Treasure artifact token with "{T}, Sacrifice this token: Add one mana of any color."

### CR 605.1a (Mana Ability classification)
> An activated ability is a mana ability if it meets all of the following criteria: it doesn't require a target (see rule 115.6), it could add mana to a player's mana pool when it resolves, and it's not a loyalty ability.

### CR 605.3b (Mana abilities resolve immediately)
> An activated mana ability doesn't go on the stack, so it can't be targeted, countered, or otherwise responded to. Rather, it resolves immediately after it is activated.

### CR 111.7 (Tokens leaving battlefield)
> A token that's in a zone other than the battlefield ceases to exist. This is a state-based action; see rule 704. (Note that if a token changes zones, applicable triggered abilities will trigger before the token ceases to exist.)

## Key Edge Cases

1. **Treasure's sacrifice ability IS a mana ability (CR 605.1a)**: It has no target, it produces mana, and it's not a loyalty ability. This means it does NOT use the stack. It resolves immediately. It cannot be countered or responded to. Players can activate it while paying costs (e.g., mid-cast). This is the #1 design challenge: the current `ManaAbility` struct + `TapForMana` command only supports simple "tap to add mana" -- it has no concept of sacrifice-as-cost.

2. **Summoning sickness applies to Treasure tokens**: Treasure tokens are artifacts (not creatures), so summoning sickness does NOT prevent activating the {T} ability. CR 302.6 only restricts creatures. The mana.rs handler already correctly checks `card_types.contains(&CardType::Creature)` before applying the summoning sickness check.

3. **Token ceases to exist after sacrifice (CR 111.7)**: After a Treasure token is sacrificed (moved to graveyard), it briefly exists in the graveyard (triggering "whenever a permanent is put into a graveyard" etc.), then ceases to exist as an SBA. The engine already handles this via `is_token: true` on the `GameObject` and the SBA check for tokens in non-battlefield zones.

4. **"Sacrifice this artifact" vs "Sacrifice this token"**: Both phrasings appear in Treasure reminder text. Functionally identical. The engine uses `Cost::Sacrifice(TargetFilter::default())` which means "sacrifice self."

5. **Multiple Treasures can be activated in sequence**: A player can sacrifice multiple Treasures in one "mana ability window" (e.g., while paying for a spell) since each activation is a separate mana ability that resolves immediately (CR 605.3a).

6. **Multiplayer considerations**: Treasure tokens are controlled by the player who created them. No opponent can activate your Treasure's ability. The engine already validates controller in `handle_tap_for_mana`.

7. **Interaction with Stifle-type effects**: Because Treasure's sacrifice ability is a mana ability, it CANNOT be countered by Stifle (which only counters activated/triggered abilities on the stack). Mana abilities never go on the stack.

8. **Interaction with Collector Ouphe / Null Rod**: "Activated abilities of artifacts can't be activated." This DOES prevent Treasure activation because Treasure's ability is still an activated ability (it's just also a mana ability). The engine would need to check for such static effects, but this is a future concern beyond this implementation.

## Current State (from ability-wip.md)

- [ ] Step 1: Token definition (predefined token type)
- [ ] Step 2: Rule enforcement (sacrifice for mana activated ability)
- [ ] Step 3: Trigger wiring
- [ ] Step 4: Unit tests
- [ ] Step 5: Card definition (card that creates Treasure tokens)
- [ ] Step 6: Game script
- [ ] Step 7: Coverage doc update

## Design Decisions

### Decision 1: How to Model Treasure's Activated Ability

**Problem**: Treasure has `{T}, Sacrifice this artifact: Add one mana of any color.` This is a mana ability per CR 605.1a, but the current `ManaAbility` struct only models `produces: OrdMap<ManaColor, u32>` + `requires_tap: bool`. There's no way to express "also sacrifice self" and "add any color (player chooses)."

**Approach: Extend `ManaAbility` with sacrifice flag + any-color flag**

Add two new fields to `ManaAbility`:
- `sacrifice_self: bool` -- if true, the source is moved to the graveyard as part of activation
- `any_color: bool` -- if true, produces 1 mana of any color (player chooses, but for now defaults to colorless like `AddManaAnyColor`)

Update `handle_tap_for_mana` in `mana.rs` to:
1. Check `sacrifice_self` and move the object to the graveyard (like the sacrifice path in `abilities.rs:231-260`)
2. Check `any_color` and add 1 mana (defaulting to colorless, consistent with current `AddManaAnyColor` behavior at `effects/mod.rs:683-696`)

**Why not model as `ActivateAbility`?** Because then it would go on the stack, which violates CR 605.1a/605.3b. Treasure's ability MUST resolve immediately without using the stack.

**Why not add a new `Effect::CreateTreasure` variant?** Because the existing `Effect::CreateToken { spec: TokenSpec }` infrastructure works fine. We just need `TokenSpec` to be able to carry activated abilities (specifically mana abilities) that get populated on the resulting `GameObject`.

### Decision 2: How to Populate Abilities on Tokens

**Problem**: The current `TokenSpec` has `keywords: OrdSet<KeywordAbility>` but no field for `mana_abilities` or `activated_abilities`. When `make_token` creates a `GameObject`, it uses `..Characteristics::default()` which zeroes out all abilities.

**Approach: Add `mana_abilities` field to `TokenSpec`**

Add `pub mana_abilities: Vec<ManaAbility>` to `TokenSpec` (with empty default). Update `make_token` to populate `characteristics.mana_abilities` from the spec.

This is more general than a Treasure-specific hardcode and will also work for Gold tokens (CR 111.10c: sacrifice without tap to add any color) and Powerstone tokens (CR 111.10h: tap to add {C}).

### Decision 3: Predefined Token Helper

Add a helper function `treasure_token_spec(count: u32) -> TokenSpec` in `card_definition.rs` (next to the `TokenSpec` struct). This produces a correctly-configured `TokenSpec` with:
- name: "Treasure"
- power/toughness: 0/0 (non-creature artifact)
- card_types: {Artifact}
- subtypes: {Treasure}
- colors: {} (colorless)
- mana_abilities: [ManaAbility { requires_tap: true, sacrifice_self: true, any_color: true, produces: OrdMap::new() }]
- count: given count

Later, similar helpers for Food, Clue, Gold, etc. can follow this pattern.

## Implementation Steps

### Step 1: Extend `ManaAbility` Struct

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/game_object.rs`
**Action**: Add two fields to `ManaAbility`:
```rust
pub struct ManaAbility {
    pub produces: OrdMap<ManaColor, u32>,
    pub requires_tap: bool,
    pub sacrifice_self: bool,  // NEW: if true, source is sacrificed as part of activation
    pub any_color: bool,       // NEW: if true, produces 1 mana of any color (overrides `produces`)
}
```
**Pattern**: Follow existing `ManaAbility` at line 43-62
**Update `tap_for` constructor**: Set `sacrifice_self: false, any_color: false` in the existing `tap_for()` method.
**Add new constructor**: `ManaAbility::treasure()` -> `Self { produces: OrdMap::new(), requires_tap: true, sacrifice_self: true, any_color: true }`

### Step 2: Update Hash Implementation

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/hash.rs`
**Action**: Add `sacrifice_self` and `any_color` fields to the `HashInto` impl for `ManaAbility`.
**Pattern**: Find the existing `ManaAbility` hash impl (search for `ManaAbility` in hash.rs) and add the two bool fields.

### Step 3: Update `handle_tap_for_mana` in mana.rs

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/mana.rs`
**Action**: After the tap cost is paid (line 93), add sacrifice handling:
```rust
// Pay sacrifice cost if required (CR 111.10a: Treasure tokens).
if ability.sacrifice_self {
    let (is_creature, owner, pre_death_controller, pre_death_counters) = {
        let obj = state.object(source)?;
        (
            obj.characteristics.card_types.contains(&CardType::Creature),
            obj.owner,
            obj.controller,
            obj.counters.clone(),
        )
    };
    let (new_id, _) = state.move_object_to_zone(source, ZoneId::Graveyard(owner))?;
    if is_creature {
        events.push(GameEvent::CreatureDied {
            object_id: source,
            new_grave_id: new_id,
            controller: pre_death_controller,
            pre_death_counters,
        });
    } else {
        events.push(GameEvent::PermanentDestroyed {
            object_id: source,
            new_grave_id: new_id,
        });
    }
}
```
**CR**: CR 605.3b -- mana abilities resolve immediately. The sacrifice is part of the cost (CR 602.2c).

After sacrifice handling, add any-color mana production:
```rust
if ability.any_color {
    // CR 111.10a: "Add one mana of any color"
    // Simplified: add 1 colorless (consistent with AddManaAnyColor in effects/mod.rs:683-696).
    // Interactive color choice deferred to future milestone.
    let player_state = state.player_mut(player)?;
    player_state.mana_pool.add(ManaColor::Colorless, 1);
    events.push(GameEvent::ManaAdded {
        player,
        color: ManaColor::Colorless,
        amount: 1,
    });
} else {
    // Existing fixed-mana production path (lines 96-106).
    // ... existing code ...
}
```
**Note**: The `any_color` path should be checked AFTER sacrifice, since the sacrifice is a cost and costs are paid first. The object must be gone before mana is added (to match how sacrifice-for-mana works with the stack-based ActivateAbility path).

**Important**: After sacrificing, the source ObjectId is dead (CR 400.7). The mana still needs to be added to the player's pool. This is fine because we already have `player` captured.

### Step 4: Add `mana_abilities` to `TokenSpec`

**File**: `/home/airbaggie/scutemob/crates/engine/src/cards/card_definition.rs`
**Action**: Add field `pub mana_abilities: Vec<ManaAbility>` to `TokenSpec` struct (after `keywords` field, at ~line 637).
**Default**: `mana_abilities: Vec::new()` in `Default` impl (~line 655).
**Hash**: Update `TokenSpec` hash in `hash.rs` to include the new field.

Import `ManaAbility` at the top of `card_definition.rs`:
```rust
use crate::state::game_object::ManaAbility;
```

### Step 5: Update `make_token` to Populate Mana Abilities

**File**: `/home/airbaggie/scutemob/crates/engine/src/effects/mod.rs`
**Action**: In `make_token` (~line 1700), populate `characteristics.mana_abilities` from the spec:
```rust
let mut mana_abilities = im::Vector::new();
for ma in &spec.mana_abilities {
    mana_abilities.push_back(ma.clone());
}

let characteristics = Characteristics {
    name: spec.name.clone(),
    power: Some(spec.power),
    toughness: Some(spec.toughness),
    card_types,
    keywords,
    subtypes,
    colors,
    mana_abilities,  // NEW
    ..Characteristics::default()
};
```
**Pattern**: Follow how `keywords` are populated at lines 1685-1688.

### Step 6: Add `treasure_token_spec` Helper

**File**: `/home/airbaggie/scutemob/crates/engine/src/cards/card_definition.rs`
**Action**: Add a public helper function near `TokenSpec` (~after line 660):
```rust
/// CR 111.10a: Predefined Treasure token specification.
///
/// A colorless Treasure artifact token with "{T}, Sacrifice this token: Add one mana of any color."
pub fn treasure_token_spec(count: u32) -> TokenSpec {
    TokenSpec {
        name: "Treasure".to_string(),
        power: 0,
        toughness: 0,
        colors: OrdSet::new(),
        card_types: [CardType::Artifact].into_iter().collect(),
        subtypes: [SubType("Treasure".to_string())].into_iter().collect(),
        keywords: OrdSet::new(),
        mana_abilities: vec![ManaAbility::treasure()],
        count,
        tapped: false,
        mana_color: None,
    }
}
```

### Step 7: Add `Effect::CreateTreasure` Convenience Variant (Optional)

**Decision**: This is optional. Cards can use `Effect::CreateToken { spec: treasure_token_spec(N) }` directly. However, a dedicated `Effect::CreateTreasure { count: EffectAmount }` variant would be cleaner for card definitions and avoid repeating the spec. **Recommend: skip for now** -- use the helper function approach which requires no new Effect variant. This keeps the Effect enum small and the pattern consistent with how Beast Within/Swan Song use `CreateToken`.

### Step 8: Export New Types in lib.rs

**File**: `/home/airbaggie/scutemob/crates/engine/src/lib.rs`
**Action**: Ensure `treasure_token_spec` is re-exported so card definitions and tests can use it.
**Pattern**: Check how `TokenSpec` is currently exported.

### Step 9: Unit Tests

**File**: `/home/airbaggie/scutemob/crates/engine/tests/treasure_tokens.rs` (NEW)
**Tests to write**:

1. **`test_treasure_token_creation`** -- Create a Treasure token via `Effect::CreateToken` with `treasure_token_spec(1)`. Assert: token on battlefield, is_token=true, card_types={Artifact}, subtypes={Treasure}, has 1 mana ability with sacrifice_self=true and any_color=true.
   - CR 111.10a: Treasure is a colorless Treasure artifact token.

2. **`test_treasure_sacrifice_for_mana`** -- Place a Treasure token on the battlefield. Activate its mana ability via `TapForMana { ability_index: 0 }`. Assert: Treasure is in the graveyard (then ceases to exist via SBA), player's mana pool has 1 colorless mana, events include PermanentTapped + PermanentDestroyed + ManaAdded.
   - CR 605.1a, CR 111.10a: Treasure's ability is a mana ability.

3. **`test_treasure_sacrifice_produces_mana_immediately`** -- Verify the mana is available immediately (no stack involvement). Place a Treasure on battlefield. Tap and sacrifice it. Then use the mana to cast a spell. Assert: spell is cast successfully.
   - CR 605.3b: Mana abilities resolve immediately.

4. **`test_treasure_multiple_sacrifice`** -- Place 3 Treasure tokens on the battlefield. Sacrifice all 3 in sequence. Assert: all 3 in graveyard, player has 3 colorless mana.
   - CR 605.3a: Multiple mana abilities can be activated in sequence.

5. **`test_treasure_already_tapped_cannot_activate`** -- Place a tapped Treasure. Try to activate its mana ability. Assert: error `PermanentAlreadyTapped`.
   - CR 602.2b: Tap cost cannot be paid if already tapped.

6. **`test_treasure_not_affected_by_summoning_sickness`** -- Place a Treasure token (has summoning_sickness=true because it's a new token). Activate its mana ability. Assert: succeeds (no error). Treasure is an artifact, not a creature, so summoning sickness doesn't prevent {T} activation.
   - CR 302.6: Only creatures are affected by summoning sickness for {T} costs.

7. **`test_treasure_token_ceases_to_exist`** -- After sacrificing a Treasure, run SBAs. Assert: the token no longer exists in any zone (is_token=true token in graveyard ceases to exist per CR 111.7 / CR 704.5d).

8. **`test_treasure_cannot_be_activated_by_opponent`** -- Player A controls a Treasure. Player B tries `TapForMana` on it. Assert: error `NotController`.
   - CR 602.2: Only controller can activate abilities.

9. **`test_create_multiple_treasure_tokens`** -- Use `treasure_token_spec(3)` to create 3 Treasures at once. Assert: 3 distinct Treasure tokens on the battlefield with separate ObjectIds.
   - CR 111.10a: spec.count controls how many tokens are created.

**Pattern**: Follow `crates/engine/tests/activated_abilities.rs` for the sacrifice-as-cost test pattern (Mind Stone tests). Follow `crates/engine/tests/mana.rs` for TapForMana test pattern.

### Step 10: Card Definition

**Suggested card**: **Stimulus Package** ({2RG}, Enchantment) -- "When this enchantment enters, create two Treasure tokens. Sacrifice a Treasure: Create a 1/1 green and white Citizen creature token."

**Alternative simpler card**: A custom "Create two Treasure tokens" sorcery for testing purposes if we want something without interacting abilities. The simplest real card would be **Seize the Spoils** ({2R}, sorcery, discard a card, draw 2, create a Treasure), but the discard-as-additional-cost adds complexity.

**Recommended approach**: Define a simple test-purpose card or use **Captain Lannery Storm** ({2R}, Legendary Creature, Haste, "Whenever ~ attacks, create a Treasure token"). The attack trigger is already supported, and Haste is already implemented.

**Card lookup**: Use `card-definition-author` agent in step 5.

### Step 11: Game Script

**Suggested scenario**: A script where:
1. Player has Captain Lannery Storm (or a simple Treasure-creating card) on the battlefield
2. Player attacks, creating a Treasure token via ETB/attack trigger
3. Player sacrifices the Treasure token for mana
4. Player uses that mana to cast a spell

**Subsystem directory**: `test-data/generated-scripts/stack/` (since it involves mana and casting)

## Interactions to Watch

1. **Split Second interaction**: If a spell with Split Second is on the stack, mana abilities can still be activated (CR 702.61b exempts mana abilities). The current `handle_tap_for_mana` does NOT check for split second -- this is correct behavior. Verify that the sacrifice-enhanced path also doesn't check.

2. **Dies triggers from Treasure sacrifice**: When a Treasure is sacrificed, it moves to the graveyard. If any "whenever a permanent is put into a graveyard" triggers exist, they should fire. Since Treasure is not a creature, "whenever a creature dies" does NOT fire. The implementation must emit `PermanentDestroyed` (not `CreatureDied`) to get this right.

3. **Token SBA cleanup**: After sacrifice, the Treasure token briefly exists in the graveyard (long enough for triggers to fire per CR 111.7). The SBA check then removes it. This is already handled by the engine's token SBA.

4. **Interaction with `check_triggers` / `flush_pending_triggers`**: Per the command handler pattern gotcha, `TapForMana` currently does NOT call `check_triggers()` because mana abilities don't normally trigger anything. However, when a Treasure is sacrificed, the `PermanentDestroyed` event could trigger abilities. The mana.rs handler currently returns events but does NOT call trigger dispatch. **This is a potential gap**: if a card has "whenever an artifact is put into a graveyard from the battlefield" (like Disciple of the Vault), the trigger won't fire from a Treasure sacrifice. **For this implementation**: document this gap but do not fix it now. The trigger flush for mana abilities is a separate concern that affects ALL mana abilities with side effects.

5. **Multiplayer**: No special multiplayer considerations. Each player controls their own Treasures. The active player receives priority after mana ability activation (no change from current behavior).

## Files Modified Summary

| File | Change |
|------|--------|
| `crates/engine/src/state/game_object.rs` | Add `sacrifice_self`, `any_color` to `ManaAbility`; add `treasure()` constructor |
| `crates/engine/src/state/hash.rs` | Hash new `ManaAbility` fields + `TokenSpec.mana_abilities` |
| `crates/engine/src/rules/mana.rs` | Handle sacrifice-as-cost and any-color in `handle_tap_for_mana` |
| `crates/engine/src/cards/card_definition.rs` | Add `mana_abilities: Vec<ManaAbility>` to `TokenSpec`; add `treasure_token_spec()` helper |
| `crates/engine/src/effects/mod.rs` | Populate `mana_abilities` in `make_token` |
| `crates/engine/src/lib.rs` | Re-export `treasure_token_spec` if needed |
| `crates/engine/tests/treasure_tokens.rs` | NEW: 9 unit tests |

## Known Limitations / Future Work

1. **`any_color` defaults to colorless**: Like `AddManaAnyColor`, the player doesn't interactively choose a color yet. This is consistent with the engine's current state and will be addressed when interactive mana choices are implemented.

2. **Trigger dispatch from mana ability side effects**: `handle_tap_for_mana` does not call `check_triggers()`. This means abilities that trigger on "artifact goes to graveyard" won't fire when Treasure is sacrificed as a mana ability. This is a known gap that affects the broader mana ability infrastructure, not just Treasures.

3. **Other predefined tokens**: This implementation lays groundwork for Food (CR 111.10b: {2}, {T}, Sacrifice: gain 3 life -- NOT a mana ability), Clue (CR 111.10f: {2}, Sacrifice: draw a card -- NOT a mana ability), Gold (CR 111.10c: Sacrifice: add any color -- IS a mana ability but no tap cost). These will need `activated_abilities` on `TokenSpec` for the non-mana-ability tokens, and further `ManaAbility` variants for Gold (no tap cost).

4. **Collector Ouphe / Null Rod static effects**: These prevent activation of artifact activated abilities including mana abilities. Not implemented yet.
