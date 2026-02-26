# Ability Plan: Flashback

**Generated**: 2026-02-26
**CR**: 702.34
**Priority**: P2 (listed as P2 in ability coverage doc; top P2 gap)
**Similar abilities studied**: Commander zone casting (`rules/casting.rs:L58-87` -- cast from non-hand zone), Storm/Cascade (`rules/casting.rs:L268-326` -- keyword-triggered on-cast behavior), `is_copy` flag on StackObject (`state/stack.rs:L43-44` -- per-stack-object metadata that modifies resolution behavior)

## CR Rule Text

**702.34. Flashback**

> 702.34a Flashback appears on some instants and sorceries. It represents two static abilities: one that functions while the card is in a player's graveyard and another that functions while the card is on the stack. "Flashback [cost]" means "You may cast this card from your graveyard if the resulting spell is an instant or sorcery spell by paying [cost] rather than paying its mana cost" and "If the flashback cost was paid, exile this card instead of putting it anywhere else any time it would leave the stack." Casting a spell using its flashback ability follows the rules for paying alternative costs in rules 601.2b and 601.2f-h.

**Related rules:**

- **601.2b**: "A player can't apply two alternative methods of casting or two alternative costs to a single spell." Flashback cost is an alternative cost. Commander tax is an additional cost (applies on top).
- **601.2f**: "The total cost is the mana cost or alternative cost (as determined in rule 601.2b), plus all additional costs and cost increases, and minus all cost reductions."
- **118.9**: "An alternative cost is a cost listed in a spell's text, or applied to it from another effect, that its controller may pay rather than paying the spell's mana cost."
- **118.9a**: "Only one alternative cost can be applied to any one spell as it's being cast."
- **118.9c**: "An alternative cost doesn't change a spell's mana cost, only what its controller has to pay to cast it."
- **118.9d**: "If an alternative cost is being paid to cast a spell, any additional costs, cost increases, and cost reductions that affect that spell are applied to that alternative cost."

## Key Edge Cases

1. **Exile on ANY stack departure** (CR 702.34a): "Exile this card instead of putting it anywhere else any time it would leave the stack." This means exile on:
   - Resolution (instead of going to graveyard for instants/sorceries)
   - Being countered (instead of going to graveyard)
   - Fizzling (all targets illegal -- instead of going to graveyard)
   - Any other effect that would move it off the stack

2. **Alternative cost, not mana cost** (CR 118.9c): The spell's mana value remains based on its printed mana cost, not the flashback cost. Effects asking for "mana cost" see the original.

3. **Only instants and sorceries** (CR 702.34a): "if the resulting spell is an instant or sorcery spell." Flashback only works on instants and sorceries. If a card somehow has flashback but is not an instant or sorcery, it cannot be cast via flashback.

4. **Timing restrictions still apply** (Think Twice ruling 2024-11-08): "You must still follow any timing restrictions and permissions, including those based on the card's type. For instance, you can cast a sorcery using flashback only when you could normally cast a sorcery."

5. **Multiple flashback instances** (rulings across many cards): "If a card has multiple instances of flashback, you may choose any of its flashback costs to pay." For our engine, a single `Flashback(ManaCost)` is sufficient per instance; multiple instances would mean multiple `KeywordAbility::Flashback(cost)` entries.

6. **No mana cost = no flashback cost** (Katilda and Lier ruling 2023-04-14): "If a card with no mana cost gains flashback, it has no flashback cost. It can't be cast this way." Not relevant for cards with printed flashback costs, but important if flashback is granted by another effect.

7. **Can be cast without prior cast** (Faithless Looting ruling 2021-03-19): "You can cast a spell using flashback even if it was somehow put into your graveyard without having been cast." No "previously cast" requirement.

8. **Cost reductions apply to flashback cost** (CR 118.9d, Think Twice ruling 2024-11-08): "start with the mana cost or alternative cost (such as a flashback cost) you're paying, add any cost increases, then apply any cost reductions." Commander tax applies on top of flashback cost if applicable (though unlikely since flashback is instants/sorceries, not commanders).

9. **Multiplayer**: No special multiplayer rules. Flashback works identically in multiplayer -- any player can flashback their own cards from their own graveyard during appropriate timing windows.

## Current State (from ability-wip.md)

- [ ] Step 1: Enum variant
- [ ] Step 2: Rule enforcement
- [ ] Step 3: Trigger wiring
- [ ] Step 4: Unit tests
- [ ] Step 5: Card definition
- [ ] Step 6: Game script
- [ ] Step 7: Coverage doc update

No prior work exists. `Flashback` does not appear anywhere in the engine source.

## Implementation Steps

### Step 1: Enum Variant and Card Definition Support

**Overview**: Flashback needs three things in the type system: (A) the keyword enum variant with its alternative cost, (B) the `AbilityDefinition` variant to declare flashback on a card, and (C) a flag on `StackObject` to track whether a spell was cast via flashback.

#### Step 1a: KeywordAbility variant

**File**: `crates/engine/src/state/types.rs`
**Action**: Add `Flashback(ManaCost)` variant to `KeywordAbility` enum after `Cascade` (line 187).
**Pattern**: Follow `Ward(u32)` at line 157 -- a keyword that carries data.

```rust
/// CR 702.34: Flashback [cost] -- "You may cast this card from your graveyard
/// by paying [cost] rather than paying its mana cost" and "If the flashback
/// cost was paid, exile this card instead of putting it anywhere else any time
/// it would leave the stack."
Flashback(ManaCost),
```

**Note**: `ManaCost` is already imported in `types.rs` indirectly via the module system. Check if a direct import is needed -- `ManaCost` is defined in `state/game_object.rs`. The `types.rs` file currently does not import it, so this requires adding `use super::game_object::ManaCost;` OR defining Flashback to hold `ManaCost` by full path. The pattern for similar data-carrying variants like `Ward(u32)` uses a primitive. Since `ManaCost` is more complex, follow the existing pattern where `ManaCost` is used in other structs (e.g., `Characteristics` in `game_object.rs`). **Decision: use the full path `crate::state::game_object::ManaCost` in the enum variant, or add an import at the top of `types.rs`.**

Actually, checking the file more carefully: `types.rs` does not import anything from `game_object`. To keep `types.rs` self-contained (it currently has no imports beyond `serde`), use a simpler representation. **Alternative approach: store the flashback cost as an `Option<ManaCost>` on the `AbilityDefinition` level instead, and make the `KeywordAbility` variant bare (`Flashback`), similar to `Storm` and `Cascade`.**

**Revised design**: The `KeywordAbility::Flashback` variant is **bare** (no data). The flashback cost is stored in a new `AbilityDefinition::Flashback` variant (see Step 1b). This follows the pattern of Storm/Cascade: the keyword appears in `keywords` for quick presence-checking, while the detailed behavior is encoded in `AbilityDefinition`. The engine checks for `KeywordAbility::Flashback` to know the card has flashback, then looks up the cost from the `CardDefinition`.

```rust
/// CR 702.34: Flashback -- card may be cast from graveyard for its flashback cost.
/// The flashback cost itself is stored in `AbilityDefinition::Flashback { cost }`.
/// This variant serves as a marker for quick presence-checking.
Flashback,
```

#### Step 1b: AbilityDefinition variant

**File**: `crates/engine/src/cards/card_definition.rs`
**Action**: Add `Flashback { cost: ManaCost }` variant to `AbilityDefinition` enum after `TriggerDoubling` (currently the last variant).
**Pattern**: Follow `AbilityDefinition::Keyword(KeywordAbility)` which is the simplest data-carrying variant.

```rust
/// CR 702.34: Flashback [cost]. The card may be cast from its owner's graveyard
/// by paying this cost instead of its mana cost. If cast via flashback, the card
/// is exiled instead of going anywhere else when it leaves the stack.
///
/// Cards with this ability should also have `AbilityDefinition::Keyword(KeywordAbility::Flashback)`
/// so the engine can quickly check for flashback presence without scanning all abilities.
Flashback {
    cost: ManaCost,
},
```

#### Step 1c: StackObject flag for flashback-cast tracking

**File**: `crates/engine/src/state/stack.rs`
**Action**: Add `pub cast_with_flashback: bool` field to `StackObject` struct, after `is_copy`.
**Pattern**: Follow `is_copy: bool` at line 43-44 -- a boolean flag with `#[serde(default)]`.
**CR**: 702.34a -- "If the flashback cost was paid, exile this card instead of putting it anywhere else any time it would leave the stack."

```rust
/// CR 702.34a: If true, this spell was cast via flashback. When it leaves the
/// stack (resolves, is countered, or fizzles), it is exiled instead of going
/// to any other zone. Set at cast time in `handle_cast_spell`.
#[serde(default)]
pub cast_with_flashback: bool,
```

#### Step 1d: Hash updates

**File**: `crates/engine/src/state/hash.rs`
**Actions** (3 additions):

1. **KeywordAbility::Flashback hash** (after `Cascade => 26u8`): Add `KeywordAbility::Flashback => 27u8.hash_into(hasher)` at approximately line 313.

2. **StackObject hash** (after `self.is_copy.hash_into(hasher)`): Add `self.cast_with_flashback.hash_into(hasher)` at approximately line 1050.

3. **AbilityDefinition::Flashback hash** (after `TriggerDoubling` discriminant 7): Add discriminant 8 for `AbilityDefinition::Flashback { cost }`:
```rust
AbilityDefinition::Flashback { cost } => {
    8u8.hash_into(hasher);
    cost.hash_into(hasher);
}
```

#### Step 1e: Match arm exhaustiveness

After adding `KeywordAbility::Flashback`, grep for all `match` expressions on `KeywordAbility` and add the new arm. Key locations:

```
Grep pattern="KeywordAbility::" path="crates/engine/src/" output_mode="files_with_matches"
```

Likely files:
- `state/hash.rs` (covered above)
- `state/builder.rs` (if it matches on keywords for Ward/Prowess translation to triggered abilities)
- `rules/combat.rs` (matches on evasion keywords)
- `rules/protection.rs` (matches on protection keywords)
- Any file that exhaustively matches `KeywordAbility` will need a `Flashback => {}` arm (no-op in most cases since flashback is handled entirely in `casting.rs` and `resolution.rs`).

Similarly, after adding `AbilityDefinition::Flashback { .. }`, grep for all `match` on `AbilityDefinition` and add the new arm.

### Step 2: Rule Enforcement -- Casting from Graveyard

**File**: `crates/engine/src/rules/casting.rs`
**Action**: Extend `handle_cast_spell` to allow casting from the graveyard when the card has flashback, paying the flashback cost instead of the regular mana cost.
**CR**: 702.34a, 601.2b, 601.2f, 118.9

The current zone validation (lines 58-67) checks:
```rust
if card_obj.zone != ZoneId::Hand(player) && !casting_from_command_zone {
    return Err(...);
}
```

**Changes required:**

1. **Zone validation**: Add a third allowed zone: `ZoneId::Graveyard(player)`, gated by the card having flashback. Determine the flashback cost at this point.

```rust
let casting_from_graveyard = card_obj.zone == ZoneId::Graveyard(player);
let casting_with_flashback = casting_from_graveyard && has_flashback(&card_obj, &state.card_registry, &card_id);

// Validation: card must be in hand, command zone, or graveyard (with flashback)
if card_obj.zone != ZoneId::Hand(player) && !casting_from_command_zone && !casting_with_flashback {
    return Err(GameStateError::InvalidCommand("card is not in your hand".into()));
}
```

2. **Flashback cost lookup**: Add a helper function `get_flashback_cost` that looks up the `AbilityDefinition::Flashback { cost }` from the card's `CardDefinition`.

```rust
fn get_flashback_cost(
    card_id: &Option<CardId>,
    registry: &CardRegistry,
) -> Option<ManaCost> {
    card_id.as_ref()
        .and_then(|cid| registry.get(cid.clone()))
        .and_then(|def| {
            def.abilities.iter().find_map(|a| {
                if let AbilityDefinition::Flashback { cost } = a {
                    Some(cost.clone())
                } else {
                    None
                }
            })
        })
}
```

3. **Cost substitution**: When `casting_with_flashback` is true, use the flashback cost instead of the card's mana cost. This replaces the `base_mana_cost` (currently read from `card_obj.characteristics.mana_cost`):

```rust
let mana_cost: Option<ManaCost> = if casting_with_flashback {
    // CR 702.34a: Pay flashback cost instead of mana cost
    let fb_cost = get_flashback_cost(&card_id, &state.card_registry);
    // CR 118.9d: Additional costs (like commander tax) still apply to alternative costs
    // Commander tax is unlikely here (flashback is instants/sorceries, not commanders)
    // but handle it for correctness
    fb_cost
} else if casting_from_command_zone {
    // existing commander tax logic
    ...
} else {
    base_mana_cost
};
```

4. **Set flashback flag on StackObject**: When creating the `StackObject` (line 208-217), set `cast_with_flashback: casting_with_flashback`:

```rust
let stack_obj = StackObject {
    id: stack_entry_id,
    controller: player,
    kind: StackObjectKind::Spell { source_object: new_card_id },
    targets: spell_targets,
    cant_be_countered,
    is_copy: false,
    cast_with_flashback: casting_with_flashback,  // NEW
};
```

5. **Type validation**: CR 702.34a says "if the resulting spell is an instant or sorcery spell." The existing casting code already reads `chars.card_types`. Add a validation after determining `casting_with_flashback`:

```rust
if casting_with_flashback {
    let is_instant_or_sorcery = chars.card_types.contains(&CardType::Instant)
        || chars.card_types.contains(&CardType::Sorcery);
    if !is_instant_or_sorcery {
        return Err(GameStateError::InvalidCommand(
            "flashback can only be used on instants and sorceries".into(),
        ));
    }
}
```

6. **Replay harness**: In `crates/engine/src/testing/replay_harness.rs`, add a new action type `"cast_spell_flashback"` that finds the card in the graveyard instead of hand. This mirrors the existing `cast_spell` action but calls a `find_in_graveyard` helper.

**File**: `crates/engine/src/testing/replay_harness.rs`
**Action**: Add `find_in_graveyard` helper and `"cast_spell_flashback"` action handler.

```rust
"cast_spell_flashback" => {
    let card_id = find_in_graveyard(state, player, card_name?)?;
    let target_list = resolve_targets(targets, state, players);
    Some(Command::CastSpell {
        player,
        card: card_id,
        targets: target_list,
    })
}
```

```rust
fn find_in_graveyard(state: &GameState, player: PlayerId, name: &str) -> Option<ObjectId> {
    state.objects.iter().find_map(|(&id, obj)| {
        if obj.characteristics.name == name && obj.zone == ZoneId::Graveyard(player) {
            Some(id)
        } else {
            None
        }
    })
}
```

**Note**: The `Command::CastSpell` itself is unchanged -- it takes `player`, `card` (ObjectId), and `targets`. The engine determines whether this is a flashback cast by checking the card's zone at cast time. No new Command variant is needed.

### Step 3: Rule Enforcement -- Exile on Stack Departure (Replacement Effect)

**File**: `crates/engine/src/rules/resolution.rs`
**Action**: Modify resolution to exile flashback-cast spells instead of sending them to the graveyard.
**CR**: 702.34a -- "If the flashback cost was paid, exile this card instead of putting it anywhere else any time it would leave the stack."

This is conceptually a replacement effect, but per CR 702.34a it is a static ability that functions while the card is on the stack. The simplest and most correct implementation is to check the `cast_with_flashback` flag at every point where a spell leaves the stack, and redirect to exile.

**Three stack departure points to modify:**

#### 3a: Normal resolution (instant/sorcery goes to graveyard)

**Location**: `resolution.rs` lines 232-242 (the `else` branch for non-permanent spells)

Current code:
```rust
} else {
    // CR 608.2n: Instant/sorcery -- card moves to owner's graveyard.
    let (new_id, _old) = state.move_object_to_zone(source_object, ZoneId::Graveyard(owner))?;
    ...
}
```

**Change**: Check `stack_obj.cast_with_flashback` before choosing the destination zone:
```rust
} else {
    // CR 608.2n / CR 702.34a: Instant/sorcery moves to graveyard,
    // or exile if cast with flashback.
    let destination = if stack_obj.cast_with_flashback {
        ZoneId::Exile  // CR 702.34a
    } else {
        ZoneId::Graveyard(owner)
    };
    let (new_id, _old) = state.move_object_to_zone(source_object, destination)?;
    ...
}
```

#### 3b: Fizzle (all targets illegal)

**Location**: `resolution.rs` lines 59-84 (the fizzle path)

Current code:
```rust
let owner = state.object(source_object)?.owner;
let (new_id, _old) = state.move_object_to_zone(source_object, ZoneId::Graveyard(owner))?;
```

**Change**: Same pattern -- check `stack_obj.cast_with_flashback`:
```rust
let destination = if stack_obj.cast_with_flashback {
    ZoneId::Exile
} else {
    ZoneId::Graveyard(owner)
};
let (new_id, _old) = state.move_object_to_zone(source_object, destination)?;
```

#### 3c: Countered

**Location**: `resolution.rs` `counter_stack_object` function, lines 441-452

Current code:
```rust
StackObjectKind::Spell { source_object } => {
    let controller = stack_obj.controller;
    let owner = state.object(source_object)?.owner;
    let (new_id, _old) = state.move_object_to_zone(source_object, ZoneId::Graveyard(owner))?;
    ...
}
```

**Change**: Check `stack_obj.cast_with_flashback`:
```rust
StackObjectKind::Spell { source_object } => {
    let controller = stack_obj.controller;
    let owner = state.object(source_object)?.owner;
    let destination = if stack_obj.cast_with_flashback {
        ZoneId::Exile  // CR 702.34a
    } else {
        ZoneId::Graveyard(owner)
    };
    let (new_id, _old) = state.move_object_to_zone(source_object, destination)?;
    ...
}
```

**Note on `is_copy` interaction**: The fizzle path at line 59 already checks `stack_obj.is_copy` and skips the zone move for copies. Flashback is not relevant for copies (`cast_with_flashback` should never be true for copies since copies are not cast), but the `is_copy` check happens before the flashback check, so no conflict.

### Step 4: Trigger Wiring

**Not applicable for Flashback.** Flashback is NOT a triggered ability. Per CR 702.34a, it represents two static abilities:
1. A static ability that functions while the card is in the graveyard (permission to cast)
2. A static ability that functions while the card is on the stack (exile replacement)

No triggers need to be wired. No changes to `rules/abilities.rs` or `check_triggers`.

### Step 5: Unit Tests

**File**: `crates/engine/tests/flashback.rs` (new file)
**Pattern**: Follow `crates/engine/tests/prowess.rs` for structure, helpers, and assertion patterns.

**Card definitions needed for tests:**

1. `think_twice_def()` -- Instant, {1}{U}, "Draw a card. Flashback {2}{U}"
2. `faithless_looting_def()` -- Sorcery, {R}, "Draw two cards, then discard two cards. Flashback {2}{R}"
3. `counterspell_def()` -- Instant, {U}{U}, "Counter target spell." (for testing flashback + countered)

**Tests to write:**

#### `test_flashback_basic_cast_from_graveyard`
- CR 702.34a: A card with flashback in the graveyard can be cast by paying the flashback cost.
- Setup: Think Twice in p1's graveyard, p1 has {2}{U} mana. Main phase.
- Action: CastSpell with the graveyard ObjectId.
- Assert: SpellCast event emitted. Card moved to stack. Flashback cost deducted from pool.

#### `test_flashback_exile_on_resolution`
- CR 702.34a: When a flashback spell resolves, it is exiled (not put into graveyard).
- Setup: Think Twice in graveyard, cast via flashback, both pass.
- Assert: After resolution, the card is in exile zone (not graveyard).

#### `test_flashback_exile_on_counter`
- CR 702.34a: When a flashback spell is countered, it is exiled (not put into graveyard).
- Setup: Think Twice cast via flashback, opponent casts Counterspell.
- Assert: After counter resolves, Think Twice is in exile (not graveyard).

#### `test_flashback_exile_on_fizzle`
- CR 702.34a: When a flashback spell fizzles (all targets illegal), it is exiled.
- Setup: A flashback spell with a target (use Ancient Grudge targeting an artifact). Remove the artifact before resolution.
- Assert: After fizzle, the card is in exile.

#### `test_flashback_sorcery_timing`
- CR 702.34a + Think Twice ruling: Sorcery-speed flashback cards can only be cast at sorcery speed.
- Setup: Faithless Looting in graveyard. Try to cast during opponent's turn or during combat.
- Assert: Error returned (InvalidCommand or equivalent).

#### `test_flashback_instant_timing`
- CR 702.34a: Instant-speed flashback cards can be cast at instant speed, even from graveyard.
- Setup: Think Twice in graveyard. Cast during opponent's priority window (p2 active, p1 has priority with something on stack).
- Assert: Cast succeeds.

#### `test_flashback_card_not_in_graveyard_fails`
- Negative: A card with flashback that is in hand cannot be cast "via flashback" -- it must be cast normally from hand.
- Setup: Think Twice in hand. Verify it casts normally (from hand, goes to graveyard on resolution, not exiled).
- Assert: Normal cast behavior; card in graveyard after resolution.

#### `test_flashback_card_without_flashback_in_graveyard_fails`
- Negative: A non-flashback card in the graveyard cannot be cast.
- Setup: Lightning Bolt (no flashback) in graveyard. Try to cast.
- Assert: Error returned ("card is not in your hand").

#### `test_flashback_pays_flashback_cost_not_mana_cost`
- CR 702.34a, 118.9: The flashback cost is paid instead of the mana cost.
- Setup: Think Twice (mana cost {1}{U}) with flashback {2}{U}. Player has exactly {2}{U} (enough for flashback but NOT for mana cost if it had been {3}{U}).
- Assert: Cast succeeds with flashback cost deducted.

#### `test_flashback_mana_value_unchanged`
- CR 118.9c: The spell's mana value is still based on its printed mana cost.
- Setup: Think Twice (mana cost {1}{U}, MV=2) cast via flashback ({2}{U}).
- Assert: The card on the stack has mana_cost equal to {1}{U} (not {2}{U}). Characteristics.mana_cost is unchanged.

**Test file structure:**
```rust
//! Flashback keyword ability tests (CR 702.34).
//!
//! Flashback is a static ability that allows instants and sorceries to be cast
//! from the graveyard by paying an alternative cost. When a flashback spell
//! leaves the stack, it is exiled instead of going anywhere else.
//!
//! Key rules verified:
//! - Cast from graveyard by paying flashback cost (CR 702.34a).
//! - Exiled on resolution, counter, or fizzle (CR 702.34a).
//! - Timing restrictions still apply (sorcery speed for sorceries).
//! - Only instants and sorceries can use flashback.
//! - Mana value is based on printed mana cost, not flashback cost (CR 118.9c).
```

### Step 6: Card Definition (later phase)

**Suggested card**: Think Twice
- Simple instant, easy to test (draws a card).
- Flashback cost is different from mana cost ({1}{U} vs {2}{U}).
- No targets, so no fizzle complexity in the basic card def.
- Commander staple.

**Additional suggested card**: Faithless Looting
- Sorcery (tests sorcery-speed flashback).
- Commander staple (banned in Modern but legal and common in Commander).

**Card lookup**: Use `card-definition-author` agent for both.

### Step 7: Game Script (later phase)

**Suggested scenario**: "Flashback from Graveyard"
- Player 1 casts Think Twice from hand (goes to graveyard).
- Later, Player 1 casts Think Twice from graveyard via flashback.
- On resolution, Think Twice is exiled (not returned to graveyard).
- Verify exile zone count.

**Subsystem directory**: `test-data/generated-scripts/stack/` (flashback involves casting and stack resolution)

**Script actions needed:**
1. `cast_spell` -- Think Twice from hand (normal cast)
2. `pass_priority` x N -- resolve to graveyard
3. `cast_spell_flashback` -- Think Twice from graveyard
4. `pass_priority` x N -- resolve; verify exile

## Interactions to Watch

### Casting system
- **Zone validation in `handle_cast_spell`**: The current code checks `ZoneId::Hand(player)` and `ZoneId::Command(player)`. Adding `ZoneId::Graveyard(player)` must be carefully gated by flashback presence. A card in the graveyard without flashback must NOT be castable.
- **Commander tax interaction**: Commander tax is an additional cost (CR 903.8). If a commander somehow had flashback (unlikely but possible via Snapcaster Mage-like effects), the tax would apply on top of the flashback cost per CR 118.9d. The current code structure handles this naturally since commander tax is applied after the base cost is determined.

### Resolution system
- **Three exit points**: Resolution (lines 232-242), fizzle (lines 59-84), and counter (`counter_stack_object`). All three must check `cast_with_flashback` and redirect to exile.
- **`is_copy` check**: The fizzle path already handles `is_copy` before zone moves. Flashback should not conflict -- a copy is never cast with flashback. But ensure `cast_with_flashback: false` is always set for copies in `copy.rs`.

### Copy system
- **Storm/Cascade copies**: When `copy_spell_on_stack` creates copies, ensure `cast_with_flashback: false` on the copy. Copies are not cast, so they should never have the flashback flag.
- **File**: `crates/engine/src/rules/copy.rs` -- check where `StackObject` is created for copies and ensure `cast_with_flashback` defaults to false (which `#[serde(default)]` handles, but explicit is safer).

### Prowess interaction
- Casting a spell via flashback IS casting a spell. Prowess triggers ("Whenever you cast a noncreature spell") should fire normally. The `SpellCast` event is emitted by `handle_cast_spell` regardless of how the spell was cast, so no changes needed to prowess.

### Stack object display
- The replay viewer's `StateViewModel` serializes `StackObject`. The new `cast_with_flashback` field will appear in serialization automatically. No viewer changes needed (it already passes through all fields).

## File Change Summary

| File | Change Type | Description |
|------|-------------|-------------|
| `crates/engine/src/state/types.rs` | Add variant | `KeywordAbility::Flashback` |
| `crates/engine/src/cards/card_definition.rs` | Add variant | `AbilityDefinition::Flashback { cost: ManaCost }` |
| `crates/engine/src/state/stack.rs` | Add field | `StackObject::cast_with_flashback: bool` |
| `crates/engine/src/state/hash.rs` | Add hash arms | KeywordAbility (27), AbilityDefinition (8), StackObject field |
| `crates/engine/src/rules/casting.rs` | Major changes | Zone validation, cost lookup, flashback flag setting |
| `crates/engine/src/rules/resolution.rs` | Modify 3 paths | Exile instead of graveyard for flashback spells |
| `crates/engine/src/rules/copy.rs` | Minor | Ensure `cast_with_flashback: false` on copies |
| `crates/engine/src/testing/replay_harness.rs` | Add action | `cast_spell_flashback` + `find_in_graveyard` helper |
| `crates/engine/tests/flashback.rs` | New file | 10 unit tests |
| Match arm additions | Multiple files | Add `Flashback` arm to exhaustive matches |

## Risks and Mitigations

1. **Risk: Missing a stack departure point.** If any code path moves a spell off the stack without checking `cast_with_flashback`, the spell goes to the wrong zone.
   - **Mitigation**: Grep for all `move_object_to_zone` calls where the source is a stack object. There are exactly 3 (resolution, fizzle, counter). All are in `resolution.rs`.

2. **Risk: `types.rs` import cycle.** Adding `ManaCost` to the keyword variant could create a circular dependency if `types.rs` imports from `game_object.rs`.
   - **Mitigation**: Use the bare `Flashback` variant on `KeywordAbility` (no data). Store the cost in `AbilityDefinition::Flashback { cost }`. This avoids the import entirely.

3. **Risk: `cast_with_flashback` not propagated on copy.** Storm/cascade copies might inherit the flag incorrectly.
   - **Mitigation**: Explicitly set `cast_with_flashback: false` when creating copies in `copy.rs`. Add a test that verifies storm copies of a flashback spell are NOT exiled.
