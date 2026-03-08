# Ability Plan: Blood Tokens

**Generated**: 2026-03-08
**CR**: 111.10g
**Priority**: P4
**Similar abilities studied**: Food tokens (CR 111.10b), Clue tokens (CR 111.10f)

## CR Rule Text

CR 111.10g: "A Blood token is a colorless Blood artifact token with '{1}, {T}, Discard a card, Sacrifice this token: Draw a card.'"

Parent rule CR 111.10: "Some effects instruct a player to create a predefined token. These effects use the definition below to determine the characteristics the token is created with. The effect that creates a predefined token may also modify or add to the predefined characteristics."

## Key Edge Cases

- Blood token's activated ability costs `{1}`, `{T}`, discard a card, AND sacrifice self -- all four costs must be paid simultaneously as part of activation (CR 602.2).
- Unlike Food ({2},{T},Sacrifice) and Clue ({2},Sacrifice -- no tap), Blood requires BOTH tap AND discard AND sacrifice AND mana.
- The discard is a COST, not an effect. It happens before the ability goes on the stack.
- Summoning sickness does NOT affect artifact tokens (CR 302.6 only restricts creatures). Blood tokens can be activated the turn they are created.
- Some triggered abilities trigger "whenever you sacrifice a Blood token" -- these trigger regardless of why the Blood token was sacrificed (ruling 2025-01-24).
- You can't sacrifice a Blood token to pay multiple costs (ruling 2025-01-24).
- If an effect refers to a Blood token, it means any artifact token with the subtype Blood, even if it has gained other subtypes (ruling 2025-01-24).
- Tokens cease to exist in non-battlefield zones as an SBA (CR 704.5d).

## Infrastructure Gap: `ActivationCost.discard_card`

**This is the key finding.** The `ActivationCost` struct at `crates/engine/src/state/game_object.rs:95-108` currently has:
- `requires_tap: bool`
- `mana_cost: Option<ManaCost>`
- `sacrifice_self: bool`
- `forage: bool`

It does NOT have a `discard_card: bool` field. The `Cost::DiscardCard` variant exists in the spell-level `Cost` enum at `card_definition.rs:723`, but `cost_to_activation_cost()` at `replay_harness.rs:2829` explicitly ignores it:
```
Cost::PayLife(_) | Cost::DiscardCard => {} // no ActivationCost representation yet
```

Furthermore, `handle_activate_ability()` in `abilities.rs` processes costs in order: tap -> mana -> sacrifice -> forage. There is NO discard-a-card cost processing.

**Resolution**: Add `discard_card: bool` to `ActivationCost`, wire the discard cost processing into `handle_activate_ability()` (between mana and sacrifice), update `cost_to_activation_cost()`, and update `hash.rs`. The discard is deterministic: the harness must accept a `discard_card_name` parameter on the `activate_ability` action, and the engine must find and discard that card from the player's hand.

## Current State (from ability-wip.md)

- [ ] Step 1: Token spec function (`blood_token_spec`)
- [ ] Step 2: ActivationCost.discard_card + handle_activate_ability discard processing
- [ ] Step 3: Harness wiring (activate_ability with discard_card)
- [ ] Step 4: Unit tests
- [ ] Step 5: Card definition (Voldaren Epicure)
- [ ] Step 6: Game script

## Discriminants

**No new KeywordAbility, AbilityDefinition, or StackObjectKind discriminants are needed.**

Blood tokens are a predefined token type (like Food, Clue, Treasure). They use existing infrastructure:
- `Effect::CreateToken { spec: blood_token_spec(N) }` -- no new Effect variant
- Activated ability uses the existing `ActivatedAbility` struct and generic `StackObjectKind::ActivatedAbility` path
- No new keyword ability variant needed

Current chain (after B14.3 Reconfigure): KW 143, AbilDef 58, SOK 58. These remain unchanged.

## Implementation Steps

### Step 1: Add `blood_token_spec` Function

**File**: `crates/engine/src/cards/card_definition.rs`
**Action**: Add `pub fn blood_token_spec(count: u32) -> TokenSpec` after `clue_token_spec` (line ~1555).
**Pattern**: Follow `food_token_spec` at line 1486 and `clue_token_spec` at line 1524.

The function creates a `TokenSpec` with:
- `name: "Blood"`
- `power: 0, toughness: 0`
- `colors: OrdSet::new()` (colorless)
- `card_types: [Artifact]`
- `subtypes: [SubType("Blood")]`
- `keywords: OrdSet::new()`
- `mana_abilities: vec![]`
- `activated_abilities: vec![ActivatedAbility { ... }]` with:
  - `cost: ActivationCost { requires_tap: true, mana_cost: Some(ManaCost { generic: 1, .. }), sacrifice_self: true, discard_card: true, forage: false }`
  - `effect: Some(Effect::DrawCards { player: PlayerTarget::Controller, count: EffectAmount::Fixed(1) })`
  - `sorcery_speed: false`

**Export**: Add `blood_token_spec` to:
- `crates/engine/src/cards/mod.rs` (re-export from card_definition)
- `crates/engine/src/lib.rs` (public export alongside `food_token_spec`, `clue_token_spec`, etc. at line 9)

### Step 2: Add `discard_card: bool` to `ActivationCost`

**File**: `crates/engine/src/state/game_object.rs`
**Action**: Add `pub discard_card: bool` field (with `#[serde(default)]`) to `ActivationCost` struct at line ~107, between `sacrifice_self` and `forage`.
**Pattern**: Follow `forage: bool` field.

**File**: `crates/engine/src/state/hash.rs`
**Action**: Add `self.discard_card.hash_into(hasher);` to the `ActivationCost` HashInto impl. Search for `forage.hash_into` to find the exact location.

**File**: `crates/engine/src/rules/abilities.rs` (`handle_activate_ability`)
**Action**: Add discard cost processing AFTER mana cost payment (line ~323) and BEFORE sacrifice cost (line ~326). The logic:
1. Check `ability_cost.discard_card`
2. If true, find a card in the player's hand to discard. For deterministic behavior (no player choice in the engine), discard the card with the smallest ObjectId. (The harness will specify which card via a command parameter; the engine's `handle_activate_ability` needs a `discard_card: Option<ObjectId>` parameter or similar mechanism.)

**IMPORTANT DESIGN DECISION**: The `Command::ActivateAbility` struct currently takes `(player, source, ability_index, targets)`. It does NOT have a `discard_card` parameter. Two approaches:

**Option A (recommended -- simpler)**: Add `discard_card: Option<ObjectId>` to `Command::ActivateAbility`. This follows the pattern of `CastSpell` which has many optional cost fields. The engine validates the card is in the player's hand and discards it as a cost.

**Option B (deterministic fallback)**: Don't change the Command enum; instead, when `discard_card` cost is required, auto-select the card with smallest ObjectId in hand. This is how forage works. However, this is less realistic for Blood tokens since the player should choose what to discard.

**Go with Option A.** Add `discard_card: Option<ObjectId>` to `Command::ActivateAbility`.

Files to update for Option A:
- `crates/engine/src/state/mod.rs` (or wherever `Command` enum is defined) -- add field
- `crates/engine/src/state/hash.rs` -- hash the new field
- `crates/engine/src/rules/engine.rs` -- destructure the new field in the match arm
- `crates/engine/src/rules/abilities.rs` -- accept and process the new parameter
- ALL existing `Command::ActivateAbility` construction sites must add `discard_card: None`

**File**: `crates/engine/src/testing/replay_harness.rs`
**Action**: Update `cost_to_activation_cost()` at line 2829 to handle `Cost::DiscardCard`:
```rust
Cost::DiscardCard => ac.discard_card = true,
```

### Step 3: Harness Wiring

**File**: `crates/engine/src/testing/script_schema.rs`
**Action**: The `discard_card: Option<String>` field already exists on `PlayerAction` (line 279). It's currently used only for `cast_spell_jump_start`. It can be reused for `activate_ability` actions.

**File**: `crates/engine/src/testing/replay_harness.rs`
**Action**: Update the `"activate_ability"` arm (line 640) to:
1. Read `discard_card_name` from the action
2. If present, find the card in the player's hand: `find_in_hand(state, player, discard_name)?`
3. Pass the ObjectId as `discard_card: Some(card_id)` in `Command::ActivateAbility`

### Step 4: Update All Existing `Command::ActivateAbility` Sites

**Action**: Grep for `Command::ActivateAbility` across the entire codebase. Every construction site must add `discard_card: None`. This includes:
- `rules/engine.rs` (destructuring)
- `testing/replay_harness.rs` (all action types that produce ActivateAbility)
- Any test files constructing ActivateAbility commands
- `tools/tui/` and `tools/replay-viewer/` if they construct or match on this variant

### Step 5: Existing Food Token Spec Sites

**Action**: Update all existing `ActivationCost` construction sites to include `discard_card: false`. Grep for `ActivationCost {` or `ActivationCost::default()`. The `Default` impl should already handle this (bool defaults to false), but explicit construction sites (like `food_token_spec`, `clue_token_spec`, and test files) need the field.

Since `ActivationCost` derives `Default` and `discard_card: bool` defaults to `false`, AND uses `#[serde(default)]`, existing code that uses `..Default::default()` or doesn't specify the field will work. But struct literal construction sites without `..Default::default()` will fail to compile. Check:
- `food_token_spec` at `card_definition.rs:1497-1505` -- uses explicit fields, needs `discard_card: false`
- `clue_token_spec` at `card_definition.rs:1534-1543` -- same
- `food_spec` in `tests/food_tokens.rs:64` -- same
- All other `ActivationCost { ... }` construction sites

### Step 6: Unit Tests

**File**: `crates/engine/tests/blood_tokens.rs`
**Tests to write** (follow `tests/food_tokens.rs` pattern):
- `test_blood_token_spec_characteristics` -- verify name, types, subtypes, colors, activated ability
- `test_blood_token_activation_basic` -- create a Blood token, activate it (pay {1}, tap, discard a card, sacrifice), verify draw a card
- `test_blood_token_activation_no_mana` -- insufficient mana, activation fails
- `test_blood_token_activation_already_tapped` -- tapped Blood token, activation fails
- `test_blood_token_activation_no_cards_in_hand` -- empty hand, activation fails (no card to discard)
- `test_blood_token_activation_sacrifice_removes_from_battlefield` -- Blood token goes to graveyard (then ceases to exist as SBA)
- `test_blood_token_sba_ceases_to_exist` -- token in graveyard ceases to exist
- `test_blood_token_not_affected_by_summoning_sickness` -- can activate same turn created (artifact, not creature)
- `test_blood_token_only_controller_can_activate` -- opponent cannot activate your Blood token
- `test_blood_token_create_via_effect` -- verify `Effect::CreateToken { spec: blood_token_spec(1) }` creates correct token
- `test_blood_token_discard_is_cost` -- the discard happens at activation (before stack), not at resolution
**Pattern**: Follow `tests/food_tokens.rs` structure closely.

### Step 7: Card Definition (later phase)

**Suggested card**: Voldaren Epicure
- `{R}`, Creature -- Vampire, 1/1
- "When this creature enters, it deals 1 damage to each opponent. Create a Blood token."
- ETB trigger: deal 1 damage to each opponent + create a Blood token
- Uses: `Effect::Sequence([DealDamage to each opponent, CreateToken { spec: blood_token_spec(1) }])`

**Card lookup** confirms oracle text (MCP):
> When this creature enters, it deals 1 damage to each opponent. Create a Blood token.

### Step 8: Game Script (later phase)

**Suggested scenario**: Voldaren Epicure ETB creates Blood token; player activates Blood token (pay {1}, tap, discard a card from hand, sacrifice) to draw a card.
**Subsystem directory**: `test-data/generated-scripts/stack/`

## Interactions to Watch

- **Discard-as-cost**: The discard happens at activation time (CR 602.2), not resolution. This means madness can trigger from the discard. The existing madness infrastructure (if any) should fire from this discard site.
- **"Whenever you sacrifice a Blood token" triggers**: Future cards may trigger on Blood sacrifice. The existing `PermanentDestroyed`/`CreatureDied` events from sacrifice-as-cost should suffice for trigger dispatch.
- **Token doubling**: Doubling Season, Anointed Procession, etc. affect `CreateToken` generically. Blood tokens benefit automatically.
- **Command::ActivateAbility change is CROSS-CUTTING**: Adding `discard_card: Option<ObjectId>` touches the Command enum, which affects engine.rs, all test files, TUI, replay-viewer, and simulator. This is the largest risk. Grep thoroughly for all `ActivateAbility` occurrences.

## Files Modified Summary

1. `crates/engine/src/cards/card_definition.rs` -- add `blood_token_spec()` function
2. `crates/engine/src/cards/mod.rs` -- re-export
3. `crates/engine/src/lib.rs` -- public export
4. `crates/engine/src/state/game_object.rs` -- add `discard_card: bool` to `ActivationCost`
5. `crates/engine/src/state/hash.rs` -- hash new field
6. `crates/engine/src/state/mod.rs` (or command location) -- add `discard_card: Option<ObjectId>` to `Command::ActivateAbility`
7. `crates/engine/src/rules/engine.rs` -- destructure new field, pass to handler
8. `crates/engine/src/rules/abilities.rs` -- process discard cost in `handle_activate_ability`
9. `crates/engine/src/testing/replay_harness.rs` -- update `cost_to_activation_cost()`, update `activate_ability` arm
10. `crates/engine/src/testing/script_schema.rs` -- no change needed (reuse `discard_card` field)
11. `tools/replay-viewer/src/view_model.rs` -- no new KW/SOK variants, but check Command match
12. `tools/tui/src/play/panels/stack_view.rs` -- no new SOK variants, but check Command match
13. ALL existing `Command::ActivateAbility` construction sites -- add `discard_card: None`
14. ALL existing `ActivationCost { ... }` struct literals -- add `discard_card: false`
15. `crates/engine/tests/blood_tokens.rs` -- new test file

## Risk Assessment

- **Medium risk**: The `Command::ActivateAbility` change is cross-cutting. Must grep thoroughly.
- **Low risk**: The `ActivationCost` struct literal change is mechanical but touches many files.
- **Low risk**: The token spec function itself is straightforward (follows Food/Clue pattern exactly).
