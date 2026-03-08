# Ability Plan: Offspring

**Generated**: 2026-03-07
**CR**: 702.175 (NOT 702.167 -- that is Craft; batch plan CR number was wrong)
**Priority**: P4
**Similar abilities studied**: Squad (CR 702.157) -- same pattern of additional cost at cast + ETB token creation trigger

## CR Rule Text

702.175. Offspring

702.175a Offspring represents two abilities. "Offspring [cost]" means "You may pay an additional [cost] as you cast this spell" and "When this permanent enters, if its offspring cost was paid, create a token that's a copy of it, except it's 1/1."

702.175b If a spell has multiple instances of offspring, each is paid separately and triggers based on the payments made for it, not any other instances of offspring.

## Key Edge Cases

From card rulings (Flowerfoot Swordmaster, 2024-07-26):

- **You can pay offspring only once per cast.** Unlike Squad which can be paid N times, Offspring is binary (paid or not). You cannot pay it multiple times to get more tokens.
- **Token is a copy "except it's 1/1."** The token copies exactly what was printed on the original creature, except the P/T is overridden to 1/1. This is a Layer 1 copy-except effect (CR 707.9d).
- **If the creature loses the offspring ability before the ETB trigger fires, no token is created.** Intervening-if check (CR 603.4) -- same pattern as Squad.
- **If the creature leaves the battlefield before the offspring trigger resolves, you still create the token.** This is explicitly stated in the rulings and differs from Squad's implementation which skips. The ruling says "you'll still create a token copy of it." This means Offspring must use last-known information (LKI) of the source creature.
- **Token is NOT cast.** Abilities that trigger on casting don't fire for the copy.
- **"As [this creature] enters" and "enters with" abilities work on the token.** ETB replacement effects apply normally.
- **If the original is copying something else, the token copies what the original copied, except 1/1.** The token uses copiable values (CR 707.2) with the P/T exception.
- **If the spell is countered, no trigger and no token.** The permanent must actually enter the battlefield.
- **Multiple instances of offspring (CR 702.175b)**: Each is paid separately. Each triggers independently. This is an edge case for later -- initial implementation supports one instance.

## Current State (from ability-wip.md)

- [ ] Step 1: Enum variant
- [ ] Step 2: Rule enforcement
- [ ] Step 3: Trigger wiring
- [ ] Step 4: Unit tests
- [ ] Step 5: Card definition
- [ ] Step 6: Game script
- [ ] Step 7: Coverage doc update

## Implementation Steps

### Step 1: Enum Variant

**File**: `crates/engine/src/state/types.rs`
**Action**: Add `KeywordAbility::Offspring` variant after `Squad` (line ~1273)
**Discriminant**: 138
**Doc comment**: CR 702.175a reference, note binary (paid/not paid), token is 1/1 copy

**File**: `crates/engine/src/cards/card_definition.rs`
**Action**: Add `AbilityDefinition::Offspring { cost: ManaCost }` variant after `Squad` (line ~607)
**Discriminant**: 55
**Pattern**: Follows `AbilityDefinition::Squad { cost: ManaCost }` exactly -- stores the offspring cost

**File**: `crates/engine/src/state/stack.rs`
**Action**: Add `StackObjectKind::OffspringTrigger { source_object: ObjectId }` variant after `SquadTrigger` (line ~1139)
**Discriminant**: 53
**Note**: No `squad_count` field needed -- Offspring always creates exactly 1 token

**File**: `crates/engine/src/state/stubs.rs`
**Action**: Add `PendingTriggerKind::OffspringETB` variant after `SquadETB` (line ~123)
**Note**: Add before the "Add new trigger kinds here" comment

**File**: `crates/engine/src/state/game_object.rs`
**Action**: Add `pub offspring_paid: bool` field to `GameObject` (after `squad_count`, line ~642)
**Default**: `false`
**Note**: Boolean flag (not u32 like squad_count) -- Offspring is binary

**File**: `crates/engine/src/state/stack.rs` (StackObject)
**Action**: Add `pub offspring_paid: bool` field to `StackObject` (after `squad_count`, line ~297)
**Default**: `false` (serde default)

**File**: `crates/engine/src/state/hash.rs`
**Action**: Add hash entries for:
1. `self.offspring_paid.hash_into(hasher)` in GameObject HashInto (after `squad_count`)
2. `self.offspring_paid.hash_into(hasher)` in StackObject HashInto (after `squad_count`)
3. `StackObjectKind::OffspringTrigger { source_object }` arm (discriminant 53u8)
4. `AbilityDefinition::Offspring { cost }` arm (discriminant 55u8)

**File**: `crates/engine/src/state/builder.rs`
**Action**: Add `offspring_paid: false` to the GameObject construction (after `squad_count: 0`, line ~996)

**File**: `crates/engine/src/state/mod.rs`
**Action**: Add `offspring_paid: false` in BOTH `move_object_to_zone` sites (after `squad_count: 0`, lines ~381 and ~542)
**CR**: 400.7 -- offspring_paid is not preserved across zone changes

**File**: `crates/engine/src/cards/helpers.rs`
**Action**: No changes needed -- ManaCost already exported

### Step 2: Rule Enforcement -- Casting (additional cost)

**File**: `crates/engine/src/rules/command.rs`
**Action**: Add `offspring_paid: bool` field to `Command::CastSpell` (after `squad_count`, line ~304)
**Default**: `#[serde(default)]`
**CR**: 702.175a -- "You may pay an additional [cost] as you cast this spell"

**File**: `crates/engine/src/rules/casting.rs`
**Action 1**: After the Squad cost validation block (line ~1926-1965), add an Offspring cost block:
- If `offspring_paid` is true:
  - Validate the spell has `KeywordAbility::Offspring` in layer-resolved characteristics
  - Look up the offspring cost from `AbilityDefinition::Offspring { cost }` via a `get_offspring_cost()` helper
  - Add the offspring cost to the total mana cost
  - If validation fails, return `InvalidCommand` error
**Pattern**: Follows `get_squad_cost()` at line ~5202. Create `get_offspring_cost()` similarly.
**CR**: 702.175a, 601.2b, 601.2f-h

**Action 2**: Propagate `offspring_paid` from `Command::CastSpell` to `StackObject` construction (after `squad_count` at line ~3242)

### Step 3: Trigger Wiring -- ETB trigger + resolution

**File**: `crates/engine/src/rules/resolution.rs`
**Action 1**: Transfer `offspring_paid` from StackObject to GameObject at resolution (after `obj.squad_count = stack_obj.squad_count`, line ~442):
```
obj.offspring_paid = stack_obj.offspring_paid;
```
**CR**: 702.175a -- transfer cast-time flag to the permanent for trigger-time checking

**Action 2**: After the Squad ETB trigger block (line ~1244-1280), add an Offspring ETB trigger block:
- Check: `offspring_paid == true` AND permanent has `KeywordAbility::Offspring` in layer-resolved characteristics (intervening-if, CR 603.4)
- If both true, queue a `PendingTrigger` with `kind: PendingTriggerKind::OffspringETB`
- No `squad_count` field needed on the PendingTrigger
**Pattern**: Follows Squad ETB trigger at line ~1244
**CR**: 702.175a, 603.4

**Action 3**: Token creation on trigger resolution. After `StackObjectKind::SquadTrigger` resolution block (line ~3451-3562), add `StackObjectKind::OffspringTrigger` resolution:
- **KEY DIFFERENCE FROM SQUAD**: Offspring rulings say "if the creature leaves the battlefield before the offspring ability resolves, you'll still create a token copy of it." This means we must NOT skip when source is gone. Instead, use LKI (characteristics from the source at trigger-queue time, or fall back to card registry).
- Build token using copiable values from source (same as Squad), BUT override power/toughness to 1/1
- The token is a copy-except effect: apply CopyOf continuous effect (Layer 1) AND a separate Layer 7b P/T-setting effect that sets base P/T to 1/1
- Alternative simpler approach: copy characteristics from source, then directly set `token_obj.characteristics.power = Some(1)` and `token_obj.characteristics.toughness = Some(1)` on the token BEFORE adding the CopyOf effect. The CopyOf effect in Layer 1 sets copiable values, but the "except 1/1" is part of the copy instruction (CR 707.9d), so the CopyOf registration should include the P/T override.
- Emit `TokenCreated` and `PermanentEnteredBattlefield` events
- Token has `offspring_paid: false`, `squad_count: 0`, `is_token: true`
**CR**: 702.175a, 707.2, 707.9d, 111.10

**File**: `crates/engine/src/rules/abilities.rs`
**Action**: In `flush_pending_triggers` (around line ~5718-5726), add arm for `PendingTriggerKind::OffspringETB`:
```
PendingTriggerKind::OffspringETB => {
    StackObjectKind::OffspringTrigger {
        source_object: trigger.source,
    }
}
```
**Pattern**: Follows `PendingTriggerKind::SquadETB` at line ~5718

**File**: `crates/engine/src/rules/resolution.rs` (counter/fizzle arm)
**Action**: Add `StackObjectKind::OffspringTrigger { .. }` to the counter/fizzle match arm (line ~5723)
**Pattern**: Same arm as `SquadTrigger { .. }`

### Step 4: Exhaustive Match Arms (view_model + stack_view)

**File**: `tools/replay-viewer/src/view_model.rs`
**Action 1**: Add `KeywordAbility::Offspring` arm in the keyword display match (return "Offspring")
**Action 2**: Add `StackObjectKind::OffspringTrigger { .. }` arm in `stack_kind_info()` (return appropriate display info)

**File**: `tools/tui/src/play/panels/stack_view.rs`
**Action**: Add `StackObjectKind::OffspringTrigger { .. }` arm in the exhaustive match

### Step 5: Additional Integration Points

**File**: `crates/engine/src/rules/resolution.rs` (token creation in effects/mod.rs)
**Action**: Add `offspring_paid: false` to ANY existing token-creation `GameObject` construction in resolution.rs (grep for `squad_count: 0` to find all sites)

**File**: `crates/engine/src/effects/mod.rs`
**Action**: Add `offspring_paid: false` to token creation sites (grep for `squad_count: 0`)

**File**: `crates/engine/src/testing/replay_harness.rs`
**Action 1**: Add `offspring_paid: false` default to existing `cast_spell` action type
**Action 2**: Add `cast_spell_offspring` action type that sets `offspring_paid: true`
**Pattern**: Follows `cast_spell_squad` at line ~1905

### Step 6: Unit Tests

**File**: `crates/engine/tests/offspring.rs`
**Tests to write**:

1. `test_offspring_not_paid` -- Cast a creature with Offspring but don't pay the cost (`offspring_paid: false`). Verify: creature enters battlefield, no OffspringTrigger on stack, no token created.
   **CR**: 702.175a intervening-if

2. `test_offspring_basic_paid` -- Cast with `offspring_paid: true`, resolve spell, resolve trigger. Verify: 2 permanents on battlefield (original + 1/1 token copy). Token has same name, types as original but P/T = 1/1.
   **CR**: 702.175a, 707.2, 707.9d

3. `test_offspring_token_is_1_1` -- After resolving, verify the token's layer-resolved P/T is (1, 1) even though the original creature has different P/T. Use `calculate_characteristics`.
   **CR**: 702.175a "except it's 1/1"

4. `test_offspring_rejected_without_keyword` -- Cast a creature without Offspring keyword but with `offspring_paid: true`. Verify: error returned.
   **CR**: 702.175a

5. `test_offspring_tokens_not_cast` -- Verify `spells_cast_this_turn` does not increase when the token is created.
   **Ruling**: 2024-07-26 "The token created by the offspring ability isn't 'cast'"

6. `test_offspring_source_leaves_still_creates_token` -- Cast with offspring, resolve spell, then remove the source creature from battlefield (e.g., bounce or destroy) before resolving the OffspringTrigger. Verify: token IS still created (unlike Squad which skips).
   **Ruling**: 2024-07-26 "If the spell resolves but the creature with offspring leaves the battlefield before the offspring ability resolves, you'll still create a token copy of it."

**Pattern**: Follow tests in `crates/engine/tests/squad.rs`

### Step 7: Card Definition (later phase)

**Suggested card**: Flowerfoot Swordmaster (Offspring {2}, simple 1/2 Mouse Soldier, {W} cost)
**Card lookup**: `mcp__mtg-rules__lookup_card("Flowerfoot Swordmaster")`
**Note**: Also has Valiant (not yet implemented), but Offspring can be tested independently

### Step 8: Game Script (later phase)

**Suggested scenario**: Cast Flowerfoot Swordmaster paying Offspring {2}, resolve spell, resolve trigger, verify 2 permanents on battlefield (original 1/2 + token 1/1 copy)
**Subsystem directory**: `test-data/generated-scripts/baseline/`

## Interactions to Watch

- **LKI for source leaving battlefield**: The rulings explicitly say you still create the token if the source leaves. This is a key difference from Squad (which skips). Implementation must capture source characteristics at trigger-queue time or use the card registry as fallback. Consider storing a snapshot of characteristics on the PendingTrigger or using the card_id from the trigger's source to look up the CardDefinition.
- **Copy-except P/T override (CR 707.9d)**: The "except it's 1/1" is part of the copy instruction, not a separate effect. The engine's `create_copy_effect` function may need a parameter for P/T override, OR the resolution code can apply a separate Layer 7b continuous effect that sets base P/T to 1/1. The simpler approach: after calling `create_copy_effect`, register an additional ContinuousEffect at Layer 7b (SetPowerToughness) that sets the token's P/T to 1/1. This is cleaner than modifying the copy infrastructure.
- **Panharmonicon doubling**: OffspringETB is a "When this permanent enters" trigger. `doubler_applies_to_trigger` should match it IF it matches AnyPermanentEntersBattlefield. Check whether the current implementation correctly categorizes OffspringETB for doubling purposes.
- **Multiple Offspring instances (CR 702.175b)**: Deferred for initial implementation. Each instance would need its own paid/not-paid flag and cost. Can be revisited when a card with multiple Offspring instances is authored.

## Discriminant Summary

| Type | Variant | Discriminant |
|------|---------|-------------|
| KeywordAbility | Offspring | 138 |
| AbilityDefinition | Offspring { cost } | 55 |
| StackObjectKind | OffspringTrigger { source_object } | 53 |
| PendingTriggerKind | OffspringETB | (enum, no explicit discriminant) |
