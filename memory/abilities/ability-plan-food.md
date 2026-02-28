# Ability Plan: Food Tokens

**Generated**: 2026-02-28
**CR**: 111.10b
**Priority**: P3
**Similar abilities studied**: Treasure tokens (CR 111.10a) -- `crates/engine/src/cards/card_definition.rs:784-801`, `crates/engine/tests/treasure_tokens.rs`

## CR Rule Text

> **111.10.** Some effects instruct a player to create a predefined token. These effects
> use the definition below to determine the characteristics the token is created with. The
> effect that creates a predefined token may also modify or add to the predefined
> characteristics.
>
> **111.10b** A Food token is a colorless Food artifact token with "{2}, {T}, Sacrifice
> this token: You gain 3 life."

Related rules:

> **205.3g** Artifacts have their own unique set of subtypes; these subtypes are called
> artifact types. The artifact types are Attraction, Blood, Bobblehead, Clue, Contraption,
> Equipment, **Food**, Fortification, Gold, Incubator, Infinity, Junk, Lander, Map,
> Powerstone, Spacecraft, Stone, Treasure, and Vehicle.

> **602.2** To activate an ability is to put it onto the stack and pay its costs, so that
> it will eventually resolve and have its effect.

> **602.5a** A creature's activated ability with the tap symbol ({T}) in its activation
> cost can't be activated unless the creature has been under its controller's control since
> the start of their most recent turn. Ignore this rule for creatures with haste.

> **701.61a** To forage means "Exile three cards from your graveyard or sacrifice a Food."

## Key Edge Cases

- **Food is an artifact type, not a creature type** (CR 205.3g). A Food token is NOT a
  creature -- it's a 0/0 artifact. SBAs that check creature toughness (CR 704.5f) apply
  only to creatures; Food tokens with power/toughness 0/0 survive because they are not
  creatures.
- **The activated ability uses the stack** (CR 602.2). Unlike Treasure's mana ability
  (CR 605), Food's ability does NOT produce mana, so it is NOT a mana ability. It goes
  on the stack and can be responded to. This is a critical difference from Treasure tokens.
- **Summoning sickness does NOT prevent activation** (CR 602.5a). Food tokens are not
  creatures, so summoning sickness (which restricts `{T}` abilities on creatures only)
  does not apply.
- **Sacrifice is a cost, not an effect** (CR 602.2c). The Food token is sacrificed when
  the ability is activated (cost payment), not when it resolves. If the ability is
  countered, the Food is still gone.
- **The ability is "{2}, {T}, Sacrifice this token: You gain 3 life."** This means:
  - Cost: 2 generic mana + tap + sacrifice self
  - Effect: Controller gains 3 life
- **"Sacrifice a Food"** (in Forage, Trail of Crumbs, etc.) means any artifact with the
  Food subtype, not just a Food token. But for the purpose of Food token implementation,
  the token itself must have the Food artifact subtype.
- **Tokens cease to exist in non-battlefield zones** (CR 704.5d). After sacrifice, the
  Food token briefly exists in the graveyard (long enough for "when a Food is put into
  a graveyard" triggers), then ceases to exist as an SBA.
- **Multiplayer**: Each player independently controls their own Food tokens. No special
  multiplayer interactions beyond standard controller rules.

## Current State (from ability-wip.md)

- [ ] Step 1: Token type / spec function
- [ ] Step 2: Rule enforcement (TokenSpec activated_abilities field + make_token propagation)
- [ ] Step 3: Trigger wiring (n/a -- Food uses activated abilities, not triggers)
- [ ] Step 4: Unit tests
- [ ] Step 5: Card definition
- [ ] Step 6: Game script

## Implementation Steps

### Step 1: Add `activated_abilities` field to `TokenSpec`

**File**: `/home/airbaggie/scutemob/crates/engine/src/cards/card_definition.rs`
**Action**: Add an `activated_abilities` field to the `TokenSpec` struct.

Currently `TokenSpec` (line 746) has `mana_abilities: Vec<ManaAbility>` but no
`activated_abilities`. Food tokens need a non-mana activated ability, so this field
must be added.

```rust
// In TokenSpec struct, after mana_abilities field (line 763):
/// Non-mana activated abilities on the token (CR 602).
/// Used by Food tokens (CR 111.10b), Clue tokens (CR 111.10f),
/// Shard tokens (CR 111.10e), etc.
#[serde(default)]
pub activated_abilities: Vec<ActivatedAbility>,
```

**Pattern**: Follow how `mana_abilities` is declared with `#[serde(default)]` and `Vec<T>`.

**Import needed**: The `ActivatedAbility` type is in `crate::state::game_object`. Add to
the existing import block at the top of `card_definition.rs`:
```rust
use crate::state::game_object::{ActivatedAbility, ManaAbility};
```
(Check whether `ManaAbility` is already imported; if so, just add `ActivatedAbility` to
the same `use` statement.)

**Also update in the same file**:
- `Default for TokenSpec` impl (line 766): add `activated_abilities: Vec::new()`
- `treasure_token_spec()` (line 789): add `activated_abilities: vec![]`

### Step 1b: Update ALL existing `TokenSpec` literal construction sites

Adding a new field to `TokenSpec` will cause compile errors at every site that constructs
a `TokenSpec` literal WITHOUT `..Default::default()`. These sites ALL need
`activated_abilities: vec![]` added:

**File**: `/home/airbaggie/scutemob/crates/engine/src/cards/definitions.rs`
- Line 696: Beast token (Beast Within)
- Line 732: Elephant token (Generous Gift)
- Line 944: Bird token (Swan Song)
- Line 2161: Faerie Rogue token (Bitterblossom)

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/builder.rs`
- Line 570: Spirit token (Afterlife keyword)
- Line 621: Phyrexian Germ token (Living Weapon keyword)

**File**: `/home/airbaggie/scutemob/crates/engine/tests/effects.rs`
- Line 930: Beast token in test -- this one uses `..Default::default()`, so it is SAFE.

**Total**: 6 sites need `activated_abilities: vec![]` added. 1 site is safe.

### Step 2: Update `make_token` to propagate `activated_abilities`

**File**: `/home/airbaggie/scutemob/crates/engine/src/effects/mod.rs`
**Action**: In the `make_token` function (line 1986), copy `activated_abilities` from
the `TokenSpec` to the token's `Characteristics`.

Currently `make_token` copies `mana_abilities` (lines 2010-2013) but NOT
`activated_abilities`. Add the equivalent copy:

```rust
// After the mana_abilities loop (line 2013):
let activated_abilities: Vec<ActivatedAbility> = spec.activated_abilities.clone();
```

Then include it in the `Characteristics` struct initialization (line 2015):
```rust
let characteristics = Characteristics {
    // ... existing fields ...
    mana_abilities,
    activated_abilities,  // <-- NEW
    ..Characteristics::default()
};
```

**Import needed**: Add `ActivatedAbility` to the existing imports at the top of the
function or at the module level. Check what's already imported.

**CR**: 111.10b -- the token must have the defined activated ability.

### Step 3: Update `TokenSpec` hash implementation

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/hash.rs`
**Action**: Add `self.activated_abilities.hash_into(hasher);` to the `HashInto for TokenSpec`
implementation (line 2049).

Insert after `self.mana_abilities.hash_into(hasher);` (line 2061):
```rust
self.activated_abilities.hash_into(hasher);
```

**Note**: `ActivatedAbility` already has a `HashInto` impl (confirmed -- it's hashed as
part of `Characteristics` at line 551). Verify `Vec<ActivatedAbility>` hashes correctly
by checking the blanket `impl<T: HashInto> HashInto for Vec<T>` or equivalent.

### Step 4: Create `food_token_spec()` helper function

**File**: `/home/airbaggie/scutemob/crates/engine/src/cards/card_definition.rs`
**Action**: Add a `food_token_spec(count: u32) -> TokenSpec` function, right after
`treasure_token_spec()` (line 801).

```rust
/// CR 111.10b: Predefined Food token specification.
///
/// A colorless Food artifact token with "{2}, {T}, Sacrifice this token:
/// You gain 3 life."
pub fn food_token_spec(count: u32) -> TokenSpec {
    use crate::state::game_object::{ActivatedAbility, ActivationCost};
    TokenSpec {
        name: "Food".to_string(),
        power: 0,
        toughness: 0,
        colors: OrdSet::new(),
        card_types: [CardType::Artifact].into_iter().collect(),
        subtypes: [SubType("Food".to_string())].into_iter().collect(),
        keywords: OrdSet::new(),
        mana_abilities: Vec::new(),
        activated_abilities: vec![ActivatedAbility {
            cost: ActivationCost {
                requires_tap: true,
                mana_cost: Some(ManaCost {
                    generic: 2,
                    ..ManaCost::default()
                }),
                sacrifice_self: true,
            },
            description: "{2}, {T}, Sacrifice this token: You gain 3 life.".to_string(),
            effect: Some(Effect::GainLife {
                player: PlayerTarget::Controller,
                amount: EffectAmount::Fixed(3),
            }),
            sorcery_speed: false,
        }],
        count,
        tapped: false,
        mana_color: None,
    }
}
```

**CR**: 111.10b -- "{2}, {T}, Sacrifice this token: You gain 3 life."

### Step 5: Export `food_token_spec` through the public API

**File**: `/home/airbaggie/scutemob/crates/engine/src/cards/mod.rs`
**Action**: Add `food_token_spec` to the re-export list alongside `treasure_token_spec`
(line 16).

**File**: `/home/airbaggie/scutemob/crates/engine/src/lib.rs`
**Action**: Add `food_token_spec` to the `pub use cards::` re-export (line 9).

**Pattern**: Follow exactly how `treasure_token_spec` is exported at both locations.

### Step 6: Unit Tests

**File**: `/home/airbaggie/scutemob/crates/engine/tests/food_tokens.rs`
**Pattern**: Follow `crates/engine/tests/treasure_tokens.rs` closely. Key difference:
Food uses `Command::ActivateAbility` (stack), not `Command::TapForMana` (immediate).

**Imports needed**:
```rust
use mtg_engine::{
    check_and_apply_sbas, food_token_spec, process_command, ActivatedAbility,
    ActivationCost, CardType, Command, Effect, EffectAmount, GameEvent, GameState,
    GameStateBuilder, ManaCost, ManaColor, ObjectId, ObjectSpec, PlayerId,
    PlayerTarget, Step, SubType, Target, ZoneId,
};
```

**Tests to write**:

1. **`test_food_token_spec_characteristics`** -- CR 111.10b
   - Verify `food_token_spec(1)` produces correct name, types, subtypes, colors.
   - Verify it has 0 mana abilities (Food is NOT a mana ability).
   - Verify it has exactly 1 activated ability with correct cost and effect.
   - Verify activated ability has: `requires_tap: true`, `mana_cost: Some({generic: 2})`,
     `sacrifice_self: true`.

2. **`test_food_token_has_activated_ability`** -- CR 111.10b
   - Place a Food token on battlefield via the `food_spec()` helper.
   - Verify the token has the Food subtype and exactly 1 activated ability.

3. **`test_food_activate_gain_3_life`** -- CR 111.10b + CR 602.2
   - Place Food token on battlefield. Set player mana pool to include {2} generic.
   - Send `Command::ActivateAbility { player, source, ability_index: 0, targets: vec![] }`.
   - Pass priority for both players to resolve.
   - Verify player gained 3 life (40 -> 43).
   - Verify Food token was sacrificed (gone from battlefield).

4. **`test_food_uses_stack_not_mana_ability`** -- CR 602.2, NOT CR 605
   - Activate Food ability.
   - Verify the stack is NOT empty after activation (before resolution).
   - This is the critical behavioral difference from Treasure tokens.

5. **`test_food_sacrifice_is_cost_not_effect`** -- CR 602.2c
   - Activate Food ability.
   - After activation but before resolving: verify the Food token is already gone from
     battlefield (sacrifice is cost, paid before ability goes on stack).
   - Verify player has NOT gained life yet (still 40 -- effect hasn't resolved).

6. **`test_food_already_tapped_cannot_activate`** -- CR 602.2b
   - Place a tapped Food token. Try to activate. Should fail with
     `GameStateError::PermanentAlreadyTapped`.

7. **`test_food_not_affected_by_summoning_sickness`** -- CR 602.5a / CR 302.6
   - Food is an artifact, not a creature. Summoning sickness does not apply.
   - Verify the Food token is NOT a creature type.
   - Verify activation succeeds (unlike a creature with summoning sickness).

8. **`test_food_token_ceases_to_exist_after_sba`** -- CR 704.5d
   - Activate Food, resolve, run SBAs.
   - Verify `TokenCeasedToExist` event is emitted.
   - Verify Food token is completely gone (removed from all zones).

9. **`test_food_opponent_cannot_activate`** -- CR 602.2
   - Player 2 tries to activate Player 1's Food token.
   - Should fail with `GameStateError::NotController`.

10. **`test_food_insufficient_mana_cannot_activate`** -- CR 602.2b
    - Player has only 1 generic mana. Activating Food should fail (needs {2}).
    - Assert the error is related to insufficient mana.

11. **`test_food_create_via_effect`** -- CR 111.10b + Effect::CreateToken
    - Use `Effect::CreateToken { spec: food_token_spec(1) }` via the effect execution
      engine to create a Food token on the battlefield.
    - Verify the created token has the correct activated ability (1 ability, correct cost).
    - This tests the `make_token` -> `activated_abilities` propagation path.

**Helper function** for tests (analogous to `treasure_spec` in treasure_tokens.rs):
```rust
fn food_spec(owner: PlayerId, name: &str) -> ObjectSpec {
    ObjectSpec::artifact(owner, name)
        .with_subtypes(vec![SubType("Food".to_string())])
        .with_activated_ability(ActivatedAbility {
            cost: ActivationCost {
                requires_tap: true,
                mana_cost: Some(ManaCost { generic: 2, ..ManaCost::default() }),
                sacrifice_self: true,
            },
            description: "{2}, {T}, Sacrifice: You gain 3 life.".to_string(),
            effect: Some(Effect::GainLife {
                player: PlayerTarget::Controller,
                amount: EffectAmount::Fixed(3),
            }),
            sorcery_speed: false,
        })
        .token()
}
```

**Testing pattern for activated abilities (critical difference from Treasure)**:
```rust
// Give player 2 generic mana (e.g., via TapForMana on lands, or set pool directly).
// Activate the Food ability (goes on the stack):
let (state_after_activate, _) = process_command(
    state,
    Command::ActivateAbility {
        player: p1,
        source: food_id,
        ability_index: 0,
        targets: vec![],
    },
).unwrap();
// Stack should NOT be empty (ability is on the stack):
assert!(!state_after_activate.stack_objects.is_empty());
// Food is already sacrificed (cost paid):
assert_eq!(count_on_battlefield(&state_after_activate, "Food"), 0);
// Life not gained yet (effect on stack):
assert_eq!(state_after_activate.player(p1).unwrap().life, 40);

// Pass priority to resolve:
let (state_after_p1_pass, _) = process_command(
    state_after_activate,
    Command::PassPriority { player: p1 },
).unwrap();
let (state_after_resolve, _) = process_command(
    state_after_p1_pass,
    Command::PassPriority { player: p2 },
).unwrap();
// Life gained after resolution:
assert_eq!(state_after_resolve.player(p1).unwrap().life, 43);
```

**Note on mana setup**: The test needs the player to have {2} generic mana in their pool
before activating the Food token. Use the builder's `with_mana` method or `TapForMana` on
lands placed in the initial state. Check how existing ability tests (e.g., equip.rs,
abilities.rs) provide mana to the player's pool.

### Step 7: Card Definition (later phase)

**Suggested card**: Gilded Goose
- Mana cost: {G}
- Type: Creature -- Bird (0/2)
- Flying
- When this creature enters, create a Food token.
- {1}{G}, {T}: Create a Food token.
- {T}, Sacrifice a Food: Add one mana of any color.

**Why this card**: It creates Food tokens on ETB (tests the `food_token_spec` + `CreateToken`
pipeline), has a second ability that creates more Food, and has a third ability that
sacrifices Food for mana. It's a strong Commander staple in green.

**Note**: The third ability ("Sacrifice a Food") requires sacrificing a specific artifact
subtype, not just "sacrifice this." This is a more complex `Cost::Sacrifice(TargetFilter)`
that targets any controlled Food artifact. This may require verifying that the `Sacrifice`
cost variant + `TargetFilter` can express "sacrifice a Food" (not just "sacrifice self").
If not supported, defer the third ability and only implement ETB + second ability.

**Alternative simpler card**: Bake into a Pie ({2}{B}{B} instant, "Destroy target creature.
Create a Food token.") -- simpler, only creates a Food token as part of its effect. Good
fallback if Gilded Goose is too complex for initial implementation.

### Step 8: Game Script (later phase)

**Suggested scenario**: Player casts a spell that creates a Food token, then activates the
Food token to gain 3 life.

**Subsystem directory**: `test-data/generated-scripts/stack/`

**Script outline**:
1. Player 1 casts a card that creates a Food token (ETB trigger or spell effect).
2. Resolve the spell/trigger. Food token appears on battlefield.
3. Player 1 pays {2}, activates Food token ability (sacrifice + tap).
4. All players pass priority.
5. Ability resolves. Player 1 gains 3 life.
6. Assert: player 1 life = 43, Food token no longer on battlefield.

## Interactions to Watch

- **Food ability uses the stack**: Unlike Treasure (mana ability, immediate), Food's ability
  goes on the stack and can be countered. The `ActivateAbility` command handler already
  supports this flow (used by Equip, Mind Stone draw, etc.).
- **Sacrifice-as-cost timing**: The engine already handles `sacrifice_self: true` in
  `ActivationCost` (see `abilities.rs` line 128+). The Food token is sacrificed during
  cost payment, before the ability goes on the stack.
- **{2} mana cost in activation**: The `ActivationCost.mana_cost` field supports this.
  The engine pays the mana from the player's pool during activation.
- **CreateToken must propagate activated_abilities**: This is the main infrastructure change.
  Without it, `Effect::CreateToken { spec: food_token_spec(1) }` would create a Food token
  that has no abilities and cannot be activated.
- **Clue and Shard tokens also need this**: Adding `activated_abilities` to `TokenSpec` is
  not just for Food -- it's also needed for Clue tokens (CR 111.10f: "{2}, Sacrifice: Draw
  a card") and Shard tokens (CR 111.10e: "{2}, Sacrifice: Scry 1, then draw a card").
  This implementation unblocks all three predefined artifact token types.
- **Blood tokens need both tap AND discard in cost**: Blood (CR 111.10g) has
  "{1}, {T}, Discard a card, Sacrifice: Draw a card." The `ActivationCost` struct can
  represent {1}+{T}+sacrifice, but NOT the "discard a card" component. This is a known
  limitation -- Blood tokens will need `ActivationCost` to be extended later.
- **Multiplayer**: No special multiplayer considerations. Each player's Food tokens are
  independently controlled.
- **Priority reset**: `ActivateAbility` resets priority to the active player (per
  `process_command` flow in `engine.rs`). This is standard behavior and does not require
  special handling.

## File Change Summary

| File | Change |
|------|--------|
| `crates/engine/src/cards/card_definition.rs` | Add `activated_abilities: Vec<ActivatedAbility>` to `TokenSpec`; update `Default`; update `treasure_token_spec()`; add `food_token_spec()`; add import for `ActivatedAbility` |
| `crates/engine/src/cards/definitions.rs` | Add `activated_abilities: vec![]` to 4 existing `TokenSpec` literals (lines 696, 732, 944, 2161) |
| `crates/engine/src/state/builder.rs` | Add `activated_abilities: vec![]` to 2 existing `TokenSpec` literals (lines 570, 621) |
| `crates/engine/src/effects/mod.rs` | Update `make_token()` to copy `activated_abilities` from `TokenSpec` to `Characteristics` |
| `crates/engine/src/state/hash.rs` | Add `self.activated_abilities.hash_into(hasher)` to `HashInto for TokenSpec` impl |
| `crates/engine/src/cards/mod.rs` | Re-export `food_token_spec` |
| `crates/engine/src/lib.rs` | Re-export `food_token_spec` |
| `crates/engine/tests/food_tokens.rs` | New test file: 11 tests |

## Dependencies

- `ActivatedAbility` and `ActivationCost` from `state/game_object.rs` -- already exist
- `Effect::GainLife` -- already exists
- `handle_activate_ability` in `rules/abilities.rs` -- already supports tap + mana + sacrifice costs
- `Effect::CreateToken` in `effects/mod.rs` -- already exists, needs `activated_abilities` propagation
- `treasure_token_spec` pattern in `card_definition.rs` -- template for `food_token_spec`

## Risks

- **Low risk**: Adding a field to `TokenSpec` is a straightforward struct change. Since the
  field has `#[serde(default)]`, existing serialized data (JSON scripts) won't break.
- **Medium risk**: 6 existing `TokenSpec` literal construction sites enumerate all fields
  WITHOUT `..Default::default()`. Adding the new field will cause compile errors at these
  sites. The runner MUST add `activated_abilities: vec![]` to all 6 sites (listed in Step
  1b) to maintain compilation.
- **Low risk**: The `make_token` function change is additive -- existing tokens with empty
  `activated_abilities` will be unaffected.
