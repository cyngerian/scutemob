# Ability Plan: Clue Tokens

**Generated**: 2026-02-28
**CR**: 111.10f
**Priority**: P3
**Similar abilities studied**: Food tokens (`food_token_spec()` at `crates/engine/src/cards/card_definition.rs:L815-845`, tests at `crates/engine/tests/food_tokens.rs`)

## CR Rule Text

**CR 111.10f**: A Clue token is a colorless Clue artifact token with "{2}, Sacrifice this token: Draw a card."

**CR 701.16** (Investigate keyword action -- separate ability, not implemented here):
- 701.16a: "Investigate" means "Create a Clue token." See rule 111.10f.

**CR 111.10** (parent): Some effects instruct a player to create a predefined token. These effects use the definition below to determine the characteristics the token is created with. The effect that creates a predefined token may also modify or add to the predefined characteristics.

## Key Edge Cases

- **No tap cost**: Unlike Food tokens ("{2}, {T}, Sacrifice"), Clue tokens cost only "{2}, Sacrifice this token" -- there is NO tap requirement. This means `requires_tap: false` in the `ActivationCost`, which is the critical difference from Food. A tapped Clue can still be activated.
- **Clue is an artifact type** (ruling from Thraben Inspector, 2016-04-08): "Clue is an artifact type. Even though it appears on some cards with other permanent types, it's never a creature type, a land type, or anything but an artifact type."
- **Cannot sacrifice for multiple costs** (ruling 2016-04-08): "You can't sacrifice a Clue to pay multiple costs. For example, you can't sacrifice a Clue token to activate its own ability and also to activate Alquist Proft, Master Sleuth's ability." This is automatically enforced by the engine's sacrifice-as-cost mechanism (once sacrificed, the object is gone).
- **Non-token Clues exist** (ruling 2016-04-08): "If an effect refers to a Clue, it means any Clue artifact, not just a Clue artifact token." This is relevant for future card interactions but does not affect token creation.
- **Token ceases to exist after sacrifice** (CR 704.5d): After the Clue is sacrificed (moved to graveyard as a cost), it ceases to exist as an SBA. Already implemented for all tokens.
- **Draw a card effect**: The activated ability draws exactly 1 card. If the controller's library is empty, the draw attempts to draw from an empty library (which is a loss condition under CR 104.3c, enforced by SBAs).
- **Multiplayer**: Any player can activate their own Clue tokens. The draw is for the controller, not any other player.

## Current State (from ability-wip.md)

- [ ] Step 1: Token type / `clue_token_spec()` helper
- [ ] Step 2: Rule enforcement (reuses existing `Effect::CreateToken` + `ActivateAbility`)
- [ ] Step 3: Trigger wiring (n/a -- Clue is just a token, no triggers)
- [ ] Step 4: Unit tests
- [ ] Step 5: Card definition
- [ ] Step 6: Game script
- [ ] Step 7: Coverage doc update

## Implementation Steps

### Step 1: `clue_token_spec()` Helper Function

**File**: `crates/engine/src/cards/card_definition.rs`
**Action**: Add `pub fn clue_token_spec(count: u32) -> TokenSpec` immediately after `food_token_spec()` (after line 845).
**Pattern**: Follow `food_token_spec()` at lines 815-845.
**CR**: 111.10f

The function returns a `TokenSpec` with:
- `name`: `"Clue"`
- `power`: 0, `toughness`: 0
- `colors`: empty (colorless)
- `card_types`: `[CardType::Artifact]`
- `subtypes`: `[SubType("Clue")]`
- `keywords`: empty
- `mana_abilities`: empty (the sacrifice-draw ability is NOT a mana ability)
- `activated_abilities`: one `ActivatedAbility` with:
  - `cost.requires_tap`: **`false`** (CRITICAL difference from Food -- Clue does NOT require tapping)
  - `cost.mana_cost`: `Some(ManaCost { generic: 2, ..Default::default() })`
  - `cost.sacrifice_self`: `true`
  - `description`: `"{2}, Sacrifice this token: Draw a card."`
  - `effect`: `Some(Effect::DrawCards { player: PlayerTarget::Controller, count: EffectAmount::Fixed(1) })`
  - `sorcery_speed`: `false` (can be activated at instant speed)
- `count`: parameter
- `tapped`: `false`
- `mana_color`: `None`

### Step 2: Export `clue_token_spec`

**File**: `crates/engine/src/cards/mod.rs` (line 16)
**Action**: Add `clue_token_spec` to the `pub use card_definition::{...}` list alongside `food_token_spec` and `treasure_token_spec`.

**File**: `crates/engine/src/lib.rs` (line 9)
**Action**: Add `clue_token_spec` to the `pub use cards::{...}` list alongside `food_token_spec` and `treasure_token_spec`.

**File**: `crates/engine/src/cards/definitions.rs` (line 29)
**Action**: Add `clue_token_spec` to the `use super::card_definition::{...}` import alongside `food_token_spec` and `treasure_token_spec`.

### Step 3: No Hash Changes Needed

**File**: `crates/engine/src/state/hash.rs`
**Action**: None. `clue_token_spec()` returns a `TokenSpec`, which already has `HashInto` implemented (line 2062). The `Effect::CreateToken { spec }` variant already hashes the spec (line 2364). No new enum variants are introduced.

### Step 4: No Trigger Wiring Needed

Clue tokens are just predefined tokens with an activated ability. They do not have triggered abilities. The "Investigate" keyword action (CR 701.16) that creates Clue tokens will be implemented as a separate ability. For now, card definitions that create Clue tokens will use `Effect::CreateToken { spec: clue_token_spec(N) }` directly.

### Step 5: Unit Tests

**File**: `crates/engine/tests/clue_tokens.rs` (new file)
**Pattern**: Follow `crates/engine/tests/food_tokens.rs` exactly.

**Tests to write** (11 tests, mirroring food_tokens.rs structure):

1. **`test_clue_token_spec_characteristics`** -- CR 111.10f: `clue_token_spec(1)` produces a spec for a colorless Clue artifact token with exactly one non-mana activated ability. Verify: name "Clue", power 0, toughness 0, colorless, Artifact type, Clue subtype, no mana abilities, 1 activated ability with `requires_tap: false`, `sacrifice_self: true`, `mana_cost.generic == 2`.

2. **`test_clue_token_has_activated_ability`** -- CR 111.10f: A Clue token placed on the battlefield via `ObjectSpec` is an artifact token with the correct subtype and exactly one activated ability. Verify `requires_tap: false` (unlike Food).

3. **`test_clue_activate_draw_card`** -- CR 111.10f + CR 602.2: Activating Clue's ability ({2}, sacrifice) puts the ability on the stack. After all players pass, the controller draws 1 card. Verify hand size increases by 1.

4. **`test_clue_uses_stack_not_mana_ability`** -- CR 602.2 / CR 605: Clue's ability is NOT a mana ability. After activation, the stack must contain the ability.

5. **`test_clue_sacrifice_is_cost_not_effect`** -- CR 602.2b / CR 601.2h: Sacrifice is a cost. The Clue token is gone from the battlefield immediately after activation, before the ability resolves. Hand size should not have changed yet.

6. **`test_clue_tapped_can_still_activate`** -- CR 111.10f: Unlike Food, Clue does NOT require {T}. A tapped Clue can still have its ability activated. This test is the INVERSE of the Food test (test 6 in food_tokens.rs). Build a tapped Clue, activate it, assert success.

7. **`test_clue_not_affected_by_summoning_sickness`** -- CR 602.5a / CR 302.6: Summoning sickness only restricts creatures with {T} abilities. Clue is an artifact (not a creature) and does not require tapping. Activation should succeed.

8. **`test_clue_token_ceases_to_exist_after_sba`** -- CR 704.5d: After a Clue is sacrificed and SBAs run, the token ceases to exist entirely.

9. **`test_clue_opponent_cannot_activate`** -- CR 602.2: Only the controller can activate their own Clue.

10. **`test_clue_insufficient_mana_cannot_activate`** -- CR 602.2b: Activating Clue requires {2} generic mana. With only 1 mana, activation fails.

11. **`test_clue_create_via_effect`** -- CR 111.10f: Using `clue_token_spec(1)` with `Effect::CreateToken` creates a Clue token on the battlefield whose `activated_abilities` are correctly propagated by `make_token`.

**Helper function** (in test file):
```rust
fn clue_spec(owner: PlayerId, name: &str) -> ObjectSpec {
    ObjectSpec::artifact(owner, name)
        .with_subtypes(vec![SubType("Clue".to_string())])
        .with_activated_ability(ActivatedAbility {
            cost: ActivationCost {
                requires_tap: false,  // KEY DIFFERENCE from Food
                mana_cost: Some(ManaCost {
                    generic: 2,
                    ..ManaCost::default()
                }),
                sacrifice_self: true,
            },
            description: "{2}, Sacrifice this token: Draw a card.".to_string(),
            effect: Some(Effect::DrawCards {
                player: PlayerTarget::Controller,
                count: EffectAmount::Fixed(1),
            }),
            sorcery_speed: false,
        })
        .token()
}
```

### Step 6: Card Definition (later phase)

**Suggested card**: Thraben Inspector
- **Mana cost**: {W}
- **Type**: Creature -- Human Soldier
- **P/T**: 1/2
- **Oracle text**: "When this creature enters, investigate. (Create a Clue token. It's an artifact with '{2}, Sacrifice this token: Draw a card.')"
- **Color identity**: W
- **Implementation**: `AbilityDefinition::Triggered` with `trigger_condition: TriggerCondition::WhenSelfEntersBattlefield`, `effect: Effect::CreateToken { spec: clue_token_spec(1) }`, `intervening_if: None`.

This is the simplest "create a Clue" card -- a single ETB trigger. No landfall, no "first time each turn" tracking. The `card-definition-author` agent should handle this using the existing ETB trigger pattern (similar to Solemn Simulacrum or other ETB creatures in definitions.rs).

**File**: `crates/engine/src/cards/definitions.rs`
**Import**: Add `clue_token_spec` to the `use super::card_definition::{...}` import at line 29.
**Pattern**: Follow Bake into a Pie (line 823) which uses `food_token_spec(1)` in a spell effect, but adapt to use a triggered ability (ETB) instead.

### Step 7: Game Script (later phase)

**Suggested scenario**: "Thraben Inspector ETB creates Clue, then activate Clue to draw"
**Subsystem directory**: `test-data/generated-scripts/stack/`
**Steps**:
1. P1 has Thraben Inspector in hand, {W} + {2} mana available.
2. P1 casts Thraben Inspector (cost {W}).
3. All pass, Inspector resolves, ETB trigger goes on stack.
4. All pass, ETB resolves -> creates a Clue token on P1's battlefield.
5. P1 adds {2} mana, activates Clue ability (sacrifice Clue, pay {2}).
6. All pass, ability resolves -> P1 draws 1 card.
7. Assert: P1's hand size increased by 1. Clue is no longer on battlefield.

## Interactions to Watch

- **No tap requirement**: The most important difference from Food. Tests MUST verify a tapped Clue can still be activated (test 6 above).
- **DrawCards on empty library**: If the controller's library is empty when the ability resolves, the draw triggers a loss-condition SBA (CR 104.3c). This is already handled by the engine's `DrawCards` effect -- no new code needed, but a test could verify it (deferred to a later edge-case pass).
- **Token doubling effects** (Doubling Season, Parallel Lives, Anointed Procession): These replace "create a token" events. When implemented, they would double Clue creation too. No special handling needed for Clue -- the replacement effect applies generically to all token creation.
- **Investigate (CR 701.16)**: The keyword action "Investigate" is just shorthand for "Create a Clue token." It will be implemented as a separate ability (likely as `Effect::Investigate` or by having card definitions use `Effect::CreateToken { spec: clue_token_spec(1) }` directly). For this plan, Thraben Inspector's ETB simply uses `CreateToken` with `clue_token_spec(1)`.
- **Sacrifice triggers**: Cards like Tireless Tracker ("Whenever you sacrifice a Clue, put a +1/+1 counter on this creature") trigger on Clue sacrifice. The engine already supports sacrifice triggers via `GameEvent::PermanentSacrificed`. Future cards can filter on subtype "Clue". No implementation needed now.
- **Multiplayer**: Clue tokens work identically in multiplayer. Each player controls their own Clues and can only activate their own (CR 602.2). The draw affects only the activating player's library.
