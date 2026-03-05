# Ability Plan: Bargain

**Generated**: 2026-03-03
**CR**: 702.166
**Priority**: P4
**Similar abilities studied**: Kicker (additional cost w/ conditional effect, CR 702.33), Retrace (non-mana additional cost at cast time, CR 702.81), Jump-Start (non-mana additional cost at cast time, CR 702.133)

## CR Rule Text

CR 702.166a: Bargain is a keyword found on some spells. It represents an optional additional cost. "Bargain" means "As an additional cost to cast this spell, you may sacrifice an artifact, enchantment, or token."

CR 702.166b: If a spell's controller declared the intention to pay a bargain cost for that spell, that spell has been "bargained." (See rule 601.2b.)

CR 702.166c: Multiple instances of bargain on the same spell are redundant.

Note: The rule text is reconstructed from authoritative sources (Wilds of Eldraine release notes, MTG CR appendix). The bare `\r` encoding of the local CR file prevented direct extraction. The CR number 702.166 is verified by two independent codebase references (`docs/mtg-engine-ability-coverage.md`, `docs/ability-batch-plan.md`).

## Key Edge Cases

- **Bargain is optional**: The player MAY sacrifice -- it is not mandatory. Cards check "if this spell was bargained" for enhanced effects.
- **Sacrifice types**: Only artifacts, enchantments, or tokens qualify. A creature token qualifies (it is a token). A non-token creature that is not an artifact/enchantment does NOT qualify.
- **The sacrifice is an additional cost (CR 118.8)**, not an alternative cost. The player still pays the normal mana cost. Bargain can combine with alternative costs (flashback, foretell, etc.).
- **The sacrifice happens at cast time (CR 601.2f-h)**, as part of paying costs. The sacrificed permanent is gone before anyone can respond to the spell.
- **Multiple instances are redundant (702.166c)**: Only one sacrifice is ever needed/allowed.
- **Token criteria**: Any token on the battlefield controlled by the caster qualifies, regardless of type (creature token, Clue, Food, Treasure, Blood, etc.).
- **"Bargained" is a designation on the spell**: Cards with bargain reference "if this spell was bargained" (like "if this spell was kicked"). This status must propagate from StackObject to both EffectContext (for spell resolution) and GameObject (for ETB triggers on permanents).
- **Permanent cards with bargain exist**: e.g., Hylda's Crown of Winter (artifact) has "When ~ enters, if it was bargained, tap up to two target nonland permanents." This means `was_bargained` must be on GameObject, not just StackObject.
- **Multiplayer**: No special multiplayer considerations -- the player sacrifices their own permanent.
- **Copy interactions**: Copies of a bargained spell are also bargained (same pattern as kicked copies -- CR 707.2: copies copy choices made during casting).

## Current State (from ability-wip.md)

- [ ] 1. Enum variant -- does not exist yet
- [ ] 2. Rule enforcement
- [ ] 3. Trigger wiring -- N/A (bargain is not a trigger)
- [ ] 4. Unit tests
- [ ] 5. Card definition
- [ ] 6. Game script
- [ ] 7. Coverage doc update

## Implementation Steps

### Step 1: Enum Variant

**File**: `crates/engine/src/state/types.rs`
**Action**: Add `KeywordAbility::Bargain` variant after `Impending` (line ~890)
**Pattern**: Follow `KeywordAbility::Kicker` at line 293 -- simple marker keyword, no parameters
**Doc comment**:
```rust
/// CR 702.166: Bargain -- optional additional cost: sacrifice an artifact,
/// enchantment, or token.
///
/// "As an additional cost to cast this spell, you may sacrifice an artifact,
/// enchantment, or token."
///
/// Marker for quick presence-checking (`keywords.contains`).
/// No `AbilityDefinition::Bargain` needed -- bargain has no per-card cost
/// to store. The sacrifice target is provided via `CastSpell.bargain_sacrifice`.
///
/// Cards check "if this spell was bargained" via `Condition::WasBargained`.
/// CR 702.166c: Multiple instances are redundant.
Bargain,
```

**Hash**: Add to `state/hash.rs` in `impl HashInto for KeywordAbility` match block:
```rust
// Bargain (discriminant 100) -- CR 702.166
KeywordAbility::Bargain => 100u8.hash_into(hasher),
```
at line ~534, after the `Impending => 99u8` arm.

**Match arms**: Grep for exhaustive `KeywordAbility` match expressions outside hash.rs (if any exist) and add new arm. The keyword is a simple marker, so most matches will have a wildcard `_ =>` that already covers it.

### Step 2: Condition Variant

**File**: `crates/engine/src/cards/card_definition.rs`
**Action**: Add `Condition::WasBargained` variant after `WasOverloaded` (line ~949)
**Pattern**: Follow `Condition::WasKicked` at line 943 and `Condition::WasOverloaded` at line 949
**Doc comment**:
```rust
/// CR 702.166b: "if this spell was bargained" -- true when
/// `was_bargained` is set on the EffectContext or StackObject.
///
/// Checked at resolution time. Used in card definitions to branch between
/// base and enhanced effects (analogous to WasKicked).
WasBargained,
```

**Hash**: Add to `state/hash.rs` in `impl HashInto for Condition` match block:
```rust
// Bargain condition (discriminant 10) -- CR 702.166b
Condition::WasBargained => 10u8.hash_into(hasher),
```
at line ~2728, after the `WasOverloaded => 9u8` arm.

### Step 3: CastSpell Command Extension

**File**: `crates/engine/src/rules/command.rs`
**Action**: Add `bargain_sacrifice: Option<ObjectId>` field to `Command::CastSpell` variant, after `prototype: bool` (line ~163)
**Pattern**: Follow `retrace_discard_land: Option<ObjectId>` at line 143
**Doc comment**:
```rust
/// CR 702.166a: The ObjectId of an artifact, enchantment, or token on the
/// battlefield to sacrifice as the bargain additional cost.
/// `None` means the player chose not to bargain (bargain is optional).
///
/// When `Some`, the identified permanent must be:
/// - On the battlefield, controlled by the caster
/// - An artifact OR an enchantment OR a token
/// - Not the spell being cast (it's on the stack, not the battlefield)
///
/// Validated in `handle_cast_spell`. Ignored for spells without bargain.
#[serde(default)]
bargain_sacrifice: Option<ObjectId>,
```

### Step 4: StackObject Extension

**File**: `crates/engine/src/state/stack.rs`
**Action**: Add `was_bargained: bool` field to `StackObject` struct, after `was_impended: bool` (line ~192)
**Pattern**: Follow `was_buyback_paid: bool` at line 118 (also an additional cost flag)
**Doc comment**:
```rust
/// CR 702.166b: If true, this spell was cast with its bargain cost paid
/// (sacrificed an artifact, enchantment, or token as an additional cost).
/// Used by `Condition::WasBargained` to check at resolution time.
///
/// Must always be false for copies (`is_copy: true`) -- copies are not cast.
/// Note: Copies of a bargained spell are also bargained (CR 707.2), so
/// this should be propagated to copies in the copy system.
#[serde(default)]
pub was_bargained: bool,
```

**Hash**: Add to `state/hash.rs` in `impl HashInto for StackObject` after `was_impended`:
```rust
// Bargain (CR 702.166b) -- spell was cast with bargain cost paid
self.was_bargained.hash_into(hasher);
```

### Step 5: GameObject Extension

**File**: `crates/engine/src/state/game_object.rs`
**Action**: Add `was_bargained: bool` field to `GameObject` struct, near `kicker_times_paid` (line ~349)
**Pattern**: Follow `kicker_times_paid: u32` at line 349 -- additional cost status propagated from stack to permanent
**Doc comment**:
```rust
/// CR 702.166b: If true, this permanent was cast with its bargain cost paid.
/// Used by ETB triggers that check "if this permanent was bargained" (e.g.,
/// Hylda's Crown of Winter). Propagated from `StackObject.was_bargained`
/// at resolution time when the permanent enters the battlefield.
#[serde(default)]
pub was_bargained: bool,
```

**Hash**: Add to `state/hash.rs` in `impl HashInto for GameObject` after `kicker_times_paid`:
```rust
// Bargain (CR 702.166b) -- permanent was cast with bargain
self.was_bargained.hash_into(hasher);
```

**Initialize to `false` in ALL creation sites**:
- `state/builder.rs` (GameStateBuilder object creation)
- `effects/mod.rs` (token creation -- multiple sites)
- `rules/resolution.rs` (token creation -- multiple sites, myriad tokens, encore tokens, etc.)

### Step 6: EffectContext Extension

**File**: `crates/engine/src/effects/mod.rs`
**Action**: Add `was_bargained: bool` field to `EffectContext` struct (line ~69, after `was_overloaded`)
**Pattern**: Follow `was_overloaded: bool` at line 69
**Doc comment**:
```rust
/// CR 702.166b: If true, this spell was cast with its bargain cost paid.
/// Used by `Condition::WasBargained`. Set from `StackObject.was_bargained`
/// at spell resolution.
pub was_bargained: bool,
```

**Update `EffectContext::new()`** (line ~74): add `was_bargained: false`
**Update `EffectContext::new_with_kicker()`** (line ~86): add `was_bargained: false`

### Step 7: Condition Check

**File**: `crates/engine/src/effects/mod.rs`
**Action**: Add `Condition::WasBargained` arm to `check_condition()` function (line ~2766, after `WasOverloaded`)
**Pattern**: Follow `Condition::WasOverloaded => ctx.was_overloaded` at line 2766
**Code**:
```rust
/// CR 702.166b: "if this spell was bargained" -- true when bargain cost was paid.
Condition::WasBargained => ctx.was_bargained,
```

### Step 8: Casting Enforcement

**File**: `crates/engine/src/rules/casting.rs`
**Action**: Add bargain handling to `handle_cast_spell()`. This involves:

1. **Function signature**: Add `bargain_sacrifice: Option<ObjectId>` parameter after `prototype: bool` (line ~68)

2. **Validation** (around line ~1290, after retrace/jump-start validation): Validate the bargain sacrifice:
   - Spell must have `KeywordAbility::Bargain`
   - Identified permanent must be on the battlefield
   - Must be controlled by the caster
   - Must be an artifact OR enchantment OR a token (`is_token: true`)
   - Must not be duplicated with another cost field

   **Pattern**: Follow retrace validation at lines 1290-1334
   ```rust
   // CR 702.166a / CR 601.2b,f: Bargain -- validate the sacrifice target.
   let bargain_sacrifice_id: Option<ObjectId> = if let Some(sac_id) = bargain_sacrifice {
       // Validate the spell has Bargain keyword
       if !chars.keywords.contains(&KeywordAbility::Bargain) {
           return Err(GameStateError::InvalidCommand(
               "spell does not have bargain (CR 702.166a)".into(),
           ));
       }
       // Validate the sacrifice target
       let sac_obj = state.object(sac_id)?;
       if sac_obj.zone != ZoneId::Battlefield {
           return Err(GameStateError::InvalidCommand(
               "bargain: sacrifice target must be on the battlefield".into(),
           ));
       }
       if sac_obj.controller != player {
           return Err(GameStateError::InvalidCommand(
               "bargain: sacrifice target must be controlled by the caster".into(),
           ));
       }
       let sac_chars = calculate_characteristics(state, sac_id);
       let is_artifact = sac_chars.card_types.contains(&CardType::Artifact);
       let is_enchantment = sac_chars.card_types.contains(&CardType::Enchantment);
       let is_token = sac_obj.is_token;
       if !is_artifact && !is_enchantment && !is_token {
           return Err(GameStateError::InvalidCommand(
               "bargain: sacrifice target must be an artifact, enchantment, or token (CR 702.166a)".into(),
           ));
       }
       Some(sac_id)
   } else {
       None
   };
   ```

3. **Sacrifice execution** (around line ~1620, after retrace/jump-start cost payment): Sacrifice the bargained permanent as part of cost payment.
   ```rust
   // CR 702.166a / CR 601.2f-h: Pay the bargain additional cost -- sacrifice
   // an artifact, enchantment, or token. The sacrifice is a real sacrifice
   // (CR 701.17): the permanent goes from battlefield to the owner's graveyard.
   // This happens as part of cost payment (CR 601.2h), after mana payment.
   if let Some(sac_id) = bargain_sacrifice_id {
       let sac_owner = state.object(sac_id)?.owner;
       state.move_object_to_zone(sac_id, ZoneId::Graveyard(sac_owner))?;
       events.push(GameEvent::PermanentSacrificed {
           player,
           object_id: sac_id,
       });
   }
   ```
   Note: Check if `PermanentSacrificed` event variant exists. If not, use appropriate existing event or add one. The important thing is the zone move.

4. **Set `was_bargained` on StackObject** (around line ~1730, in the StackObject construction):
   ```rust
   was_bargained: bargain_sacrifice_id.is_some(),
   ```

### Step 9: Resolution Propagation

**File**: `crates/engine/src/rules/resolution.rs`
**Action**: Propagate `was_bargained` from StackObject to GameObject at permanent-entry time, and to EffectContext at spell-resolution time.

1. **Permanent entry** (around line ~276, where `kicker_times_paid` is transferred):
   ```rust
   obj.was_bargained = stack_obj.was_bargained;
   ```

2. **Spell effect resolution** (around line ~199-207, where EffectContext is built):
   ```rust
   ctx.was_bargained = stack_obj.was_bargained;
   ```

### Step 10: Engine.rs Command Routing

**File**: `crates/engine/src/rules/engine.rs`
**Action**: Add `bargain_sacrifice` to the CastSpell destructuring (line ~82) and pass it through to `handle_cast_spell` (line ~99).

**Pattern**: Follow `prototype` at line 94/112.

Add `bargain_sacrifice` to the destructuring:
```rust
Command::CastSpell {
    player,
    card,
    targets,
    convoke_creatures,
    improvise_artifacts,
    delve_cards,
    kicker_times,
    alt_cost,
    escape_exile_cards,
    retrace_discard_land,
    jump_start_discard,
    prototype,
    bargain_sacrifice,  // NEW
} => {
```

Pass to `handle_cast_spell`:
```rust
let mut events = casting::handle_cast_spell(
    &mut state,
    player,
    card,
    targets,
    convoke_creatures,
    improvise_artifacts,
    delve_cards,
    kicker_times,
    alt_cost,
    escape_exile_cards,
    retrace_discard_land,
    jump_start_discard,
    prototype,
    bargain_sacrifice,  // NEW
)?;
```

### Step 11: Harness & Script Schema Extension

**File**: `crates/engine/src/testing/script_schema.rs`
**Action**: Add `bargain_sacrifice: Option<String>` field to `ScriptAction::PlayerAction` variant (near `kicked`, line ~253)
**Doc comment**:
```rust
/// CR 702.166a: For `cast_spell` with bargain. Name of the artifact,
/// enchantment, or token to sacrifice as the bargain additional cost.
/// `None` means the player chose not to bargain.
#[serde(default)]
bargain_sacrifice: Option<String>,
```

**File**: `crates/engine/src/testing/replay_harness.rs`
**Action**:
1. Add `bargain_sacrifice_name: Option<&str>` parameter to `translate_player_action()` function signature (after `discard_card_name`, line ~248)
2. In the `"cast_spell"` arm (line ~263), resolve the bargain sacrifice name to ObjectId:
   ```rust
   let bargain_sac_id = bargain_sacrifice_name
       .and_then(|name| find_on_battlefield(state, player, name));
   ```
3. Pass `bargain_sacrifice: bargain_sac_id` to the `Command::CastSpell` construction
4. Add `bargain_sacrifice: None` to all other CastSpell constructions (cast_spell_flashback, cast_spell_evoke, cast_spell_bestow, etc.)

**File**: `crates/engine/tests/script_replay.rs`
**Action**: Extract `bargain_sacrifice` from the `PlayerAction` destructuring and pass it to `translate_player_action()`.

### Step 12: Default Values for Existing StackObject/GameObject Construction Sites

**Action**: Grep for all existing `StackObject { ... }` and `GameObject { ... }` struct literals across the codebase and add `was_bargained: false` to each one.

**Key files with StackObject construction** (from existing patterns):
- `crates/engine/src/rules/casting.rs` (main spell cast, cascade copy, storm copy)
- `crates/engine/src/rules/resolution.rs` (encore tokens, embalm tokens, suspend cast trigger, etc.)
- `crates/engine/src/rules/copy.rs` (spell copies)
- `crates/engine/src/rules/abilities.rs` (triggered abilities pushed to stack)

**Key files with GameObject construction** (from existing patterns):
- `crates/engine/src/state/builder.rs` (initial object construction)
- `crates/engine/src/effects/mod.rs` (token creation)
- `crates/engine/src/rules/resolution.rs` (token creation, myriad, encore, embalm)

**Strategy**: Use `grep -r "was_impended" crates/engine/src/` to find all StackObject construction sites, then add `was_bargained: false` next to each `was_impended: false`.

### Step 13: Unit Tests

**File**: `crates/engine/tests/bargain.rs`
**Tests to write**:

- `test_bargain_basic_instant_with_sacrifice` -- CR 702.166a: Cast an instant with Bargain, provide a token to sacrifice. Verify the token is sacrificed (in graveyard/ceased to exist) and the spell resolves with the bargained effect branch.
- `test_bargain_basic_instant_without_sacrifice` -- CR 702.166a: Cast the same instant without providing a sacrifice. Verify the spell resolves with the non-bargained effect branch.
- `test_bargain_sacrifice_artifact` -- CR 702.166a: Sacrifice an artifact (non-token). Verify it's valid and the spell is bargained.
- `test_bargain_sacrifice_enchantment` -- CR 702.166a: Sacrifice an enchantment (non-token). Verify it's valid and the spell is bargained.
- `test_bargain_sacrifice_creature_token` -- CR 702.166a: Sacrifice a creature token. Verify it's valid (it is a token, even though it's not an artifact/enchantment).
- `test_bargain_sacrifice_invalid_creature` -- CR 702.166a: Attempt to sacrifice a non-token creature that is not an artifact/enchantment. Verify the command is rejected.
- `test_bargain_sacrifice_opponent_permanent` -- CR 702.166a: Attempt to sacrifice an opponent's artifact. Verify the command is rejected (must be controlled by the caster).
- `test_bargain_no_keyword_rejects_sacrifice` -- CR 702.166a: Attempt to provide a bargain sacrifice for a spell without the Bargain keyword. Verify the command is rejected.
- `test_bargain_permanent_etb_was_bargained` -- CR 702.166b: Cast a permanent with Bargain (e.g., an artifact), providing a sacrifice. Verify the permanent enters the battlefield with `was_bargained == true`, allowing ETB triggers that check `Condition::WasBargained` to fire.
- `test_bargain_permanent_etb_not_bargained` -- CR 702.166b: Cast the same permanent without bargaining. Verify `was_bargained == false` on the permanent.

**Pattern**: Follow buyback tests in `crates/engine/tests/buyback.rs` (additional cost with conditional effect)

### Step 14: Card Definition (later phase)

**Suggested card**: Torch the Tower (instant, {R}, bargain: "Deal 2 damage to target creature or planeswalker. If this spell was bargained, it deals 3 damage instead and the controller can't gain life this turn.")

This is a simple instant with a clear bargained/not-bargained branch, ideal for testing.

**Alternative simpler card**: Beseech the Mirror (sorcery, {1}{B}{B}{B}, bargain: various effects) -- but this is complex. Torch the Tower is better for the first card.

**Card lookup**: use `card-definition-author` agent

### Step 15: Game Script (later phase)

**Suggested scenario**: Player casts Torch the Tower targeting an opponent's creature. First without bargaining (2 damage), then in a second game state, with bargaining (sacrifice a Clue token, 3 damage). Assert life/creature status at each checkpoint.
**Subsystem directory**: `test-data/generated-scripts/stack/`

## Interactions to Watch

- **Bargain + Flashback/Escape/other alt costs**: Bargain is an additional cost, not an alternative cost. It should combine freely with any alternative cost. Verify no conflict in `casting.rs` logic.
- **Bargain + Kicker**: A spell could theoretically have both Bargain and Kicker. Both are additional costs. Both should be payable simultaneously without interference.
- **Sacrificed permanent's triggers**: When the bargained permanent is sacrificed as a cost, "when this dies" triggers on that permanent should fire after the spell is on the stack (same as other cost sacrifices like Emerge, Exploit). The sacrifice happens during cost payment (CR 601.2h).
- **Bargain + Convoke**: The sacrificed permanent cannot also be tapped for convoke (it's gone from the battlefield after sacrifice). But this is naturally handled because sacrifice happens during cost payment and convoke validation runs first.

## Files to Modify (complete list)

1. `crates/engine/src/state/types.rs` -- add `KeywordAbility::Bargain`
2. `crates/engine/src/cards/card_definition.rs` -- add `Condition::WasBargained`
3. `crates/engine/src/rules/command.rs` -- add `bargain_sacrifice: Option<ObjectId>` to CastSpell
4. `crates/engine/src/state/stack.rs` -- add `was_bargained: bool` to StackObject
5. `crates/engine/src/state/game_object.rs` -- add `was_bargained: bool` to GameObject
6. `crates/engine/src/effects/mod.rs` -- add `was_bargained: bool` to EffectContext + condition check
7. `crates/engine/src/rules/casting.rs` -- validate + execute bargain sacrifice + set flag
8. `crates/engine/src/rules/resolution.rs` -- propagate `was_bargained` to permanent + context
9. `crates/engine/src/rules/engine.rs` -- route new field from Command to handle_cast_spell
10. `crates/engine/src/state/hash.rs` -- hash `KeywordAbility::Bargain` (disc 100), `Condition::WasBargained` (disc 10), `StackObject.was_bargained`, `GameObject.was_bargained`
11. `crates/engine/src/state/builder.rs` -- initialize `was_bargained: false`
12. `crates/engine/src/testing/script_schema.rs` -- add `bargain_sacrifice` to PlayerAction
13. `crates/engine/src/testing/replay_harness.rs` -- add parameter + resolve bargain sacrifice name
14. `crates/engine/tests/script_replay.rs` -- extract + pass `bargain_sacrifice` field
15. `crates/engine/tests/bargain.rs` -- new test file
16. `crates/engine/src/cards/helpers.rs` -- no changes needed (no new types to export)
17. `tools/tui/src/play/panels/stack_view.rs` -- no changes needed (no new StackObjectKind)
18. `tools/replay-viewer/src/view_model.rs` -- no changes needed (no new StackObjectKind)

## Discriminant Summary

| Type | Variant | Discriminant |
|------|---------|-------------|
| `KeywordAbility` | `Bargain` | 100 |
| `Condition` | `WasBargained` | 10 |
| `StackObjectKind` | N/A (no new variant) | -- |
| `AbilityDefinition` | N/A (no new variant) | -- |
