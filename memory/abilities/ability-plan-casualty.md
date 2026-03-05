# Ability Plan: Casualty

**Generated**: 2026-03-05
**CR**: 702.153 (NOT 702.154 -- the batch plan has a wrong number; 702.154 is Enlist)
**Priority**: P4
**Similar abilities studied**: Bargain (sacrifice as additional cost, `casting.rs`), Storm (triggered copy on cast, `casting.rs` + `copy.rs` + `resolution.rs`)

## CR Rule Text

702.153. Casualty

702.153a Casualty is a keyword that represents two abilities. The first is a static
ability that functions while the spell with casualty is on the stack. The second is a
triggered ability that functions while the spell with casualty is on the stack. Casualty N
means "As an additional cost to cast this spell, you may sacrifice a creature with power N
or greater," and "When you cast this spell, if a casualty cost was paid for it, copy it. If
the spell has any targets, you may choose new targets for the copy." Paying a spell's
casualty cost follows the rules for paying additional costs in rules 601.2b and 601.2f-h.

702.153b If a spell has multiple instances of casualty, each is paid separately and
triggers based on the payments made for it, not any other instance of casualty.

## Key Edge Cases

- **Casualty X** (Ob Nixilis, the Adversary): The caster chooses X at cast time. The
  sacrificed creature must have power >= X. For initial implementation, we support fixed N
  only (Casualty 1, Casualty 2, Casualty 3). Casualty X is deferred -- it requires a
  mechanism to choose X at cast time that does not yet exist in the engine.
- **Copy is NOT cast** (ruling 2022-04-29): The copy is created on the stack directly.
  "Whenever you cast a spell" triggers do NOT fire for the copy. This matches Storm's
  behavior (CR 702.40c) -- use `is_copy: true` on the copy's StackObject.
- **Copy resolves first** (ruling 2022-04-29): The copy is pushed onto the stack above the
  original, so it resolves before the original via LIFO.
- **Only one creature sacrificed per casualty instance** (ruling 2022-04-29): "You may
  sacrifice only one creature to pay a spell's casualty cost, and you copy the spell only
  once." This is inherent in the design -- one sacrifice field, one copy.
- **Power check uses layer-resolved characteristics** (gotcha from gotchas-infra.md):
  The sacrificed creature's power must be checked via `calculate_characteristics`, not
  from the card registry. This ensures Humility/Dress Down/pump effects are respected.
- **Multiple instances** (CR 702.153b): Each instance is independent. Deferred for initial
  implementation -- no current cards have multiple instances of casualty.
- **Multiplayer**: No special multiplayer considerations. The sacrifice is a cost paid by
  the caster; the copy targets can be redirected to any legal target.

## Current State (from ability-wip.md)

- [ ] Step 1: Enum variant
- [ ] Step 2: Rule enforcement (casting.rs validation + sacrifice)
- [ ] Step 3: Trigger wiring (CasualtyTrigger on stack -> copy on resolution)
- [ ] Step 4: Unit tests

## Implementation Steps

### Step 1: Enum Variant and Types

**File**: `crates/engine/src/state/types.rs`
**Action**: Add `KeywordAbility::Casualty(u32)` variant. The `u32` is the N value
(minimum power of sacrificed creature). This is a parameterized keyword like
`Annihilator(u32)` or `Afterlife(u32)`.
**Pattern**: Follow `KeywordAbility::Bargain` at approximately line 899-911, but with a
parameter like `KeywordAbility::Annihilator(u32)`.

**File**: `crates/engine/src/state/hash.rs`
**Action**: Add hash arm for `KeywordAbility::Casualty(n)` with discriminant **104**.
Hash both the discriminant and the N value.
**Pattern**: Follow `KeywordAbility::Annihilator` hash pattern (discriminant + n).

**File**: `crates/engine/src/state/hash.rs` (StackObjectKind section)
**Action**: Add hash arm for `StackObjectKind::CasualtyTrigger` with discriminant **34**.
Hash `source_object` and `original_stack_id`.

**File**: `crates/engine/src/state/stack.rs`
**Action**: Add `StackObjectKind::CasualtyTrigger` variant:
```rust
CasualtyTrigger {
    source_object: ObjectId,
    original_stack_id: ObjectId,
}
```
This is simpler than StormTrigger (no count -- always exactly 1 copy).

**File**: `crates/engine/src/state/stack.rs`
**Action**: Add `was_casualty_paid: bool` field to `StackObject`. Initialize to `false`.
This tracks whether the casualty cost was paid for this specific spell instance.
**NOTE**: Unlike Bargain, Casualty's triggered copy is automatic (built into the keyword),
so there is no `Condition::WasCasualtyPaid` needed on card definitions -- the copy is
always produced if the cost is paid, and no card text references "if casualty was paid"
for conditional effects beyond the copy itself.

**Wait -- reconsider**: Some Casualty cards DO have effects beyond the copy. For example,
Ob Nixilis modifies the copy (not legendary, starting loyalty X). However, the standard
Casualty keyword itself handles the copy automatically. Card-specific modifications to the
copy would need a separate mechanism. For now, the `was_casualty_paid` field is sufficient
for the triggered copy; if future cards need `Condition::WasCasualtyPaid`, add it then.

**File**: `crates/engine/src/state/hash.rs` (StackObject section)
**Action**: Add `self.was_casualty_paid.hash_into(hasher)` after `was_bargained`.

**File**: `crates/engine/src/state/mod.rs` (Command enum)
**Action**: Add `casualty_sacrifice: Option<ObjectId>` field to `Command::CastSpell`.
**Pattern**: Follow `bargain_sacrifice: Option<ObjectId>`.

**File**: `crates/engine/src/rules/engine.rs`
**Action**: Destructure `casualty_sacrifice` from `Command::CastSpell` and pass it to
`cast_spell()`.

**File**: `tools/tui/src/play/panels/stack_view.rs`
**Action**: Add match arm for `StackObjectKind::CasualtyTrigger { source_object, .. }` =>
`("Casualty: ".to_string(), Some(*source_object))`.

**File**: `tools/replay-viewer/src/view_model.rs`
**Action**: Add serialization arm for `StackObjectKind::CasualtyTrigger` if exhaustive
match exists there.

### Step 2: Casting Validation and Sacrifice

**File**: `crates/engine/src/rules/casting.rs`
**Action**: Add casualty validation block after the bargain validation block (~line 1866).

The validation logic:
1. If `casualty_sacrifice` is `Some(sac_id)`:
   - Verify the spell has `KeywordAbility::Casualty(n)` in its keywords.
   - Verify the sacrifice target is on the battlefield.
   - Verify the sacrifice target is controlled by the caster.
   - Verify the sacrifice target is a creature (has `CardType::Creature`).
   - Verify the creature's power (via `calculate_characteristics`) >= N.
   - Extract the casualty N value from the keyword.
2. Store `casualty_sacrifice_id: Option<ObjectId>`.

**CR**: 702.153a -- "sacrifice a creature with power N or greater"
**CR**: 601.2b, 601.2f-h -- paying additional costs

**File**: `crates/engine/src/rules/casting.rs` (cost payment section)
**Action**: Add casualty sacrifice execution after the bargain sacrifice block (~line 2252).
Move the sacrificed creature to graveyard and emit `ObjectPutInGraveyard` event.
**Pattern**: Identical to bargain sacrifice execution.

**File**: `crates/engine/src/rules/casting.rs` (StackObject construction)
**Action**: Set `was_casualty_paid: casualty_sacrifice_id.is_some()` on the StackObject.

### Step 3: Trigger Wiring (Casualty Copy)

**File**: `crates/engine/src/rules/casting.rs` (after storm trigger block, ~line 2447)
**Action**: Add casualty trigger creation. After the spell is placed on the stack, if
`was_casualty_paid` is true, push a `CasualtyTrigger` onto the stack above the spell.

```rust
// CR 702.153a: Casualty -- "When you cast this spell, if a casualty cost was paid
// for it, copy it." Create a triggered ability on the stack that will produce the copy.
if casualty_sacrifice_id.is_some() {
    let trigger_id = state.next_object_id();
    let trigger_obj = StackObject {
        id: trigger_id,
        controller: player,
        kind: StackObjectKind::CasualtyTrigger {
            source_object: new_card_id,
            original_stack_id: stack_entry_id,
        },
        targets: vec![],
        // ... all boolean fields false, etc.
    };
    state.stack_objects.push_back(trigger_obj);
    events.push(GameEvent::AbilityTriggered { ... });
}
```

**Pattern**: Follow `StormTrigger` creation at casting.rs ~line 2447-2486.

**File**: `crates/engine/src/rules/resolution.rs`
**Action**: Add resolution arm for `StackObjectKind::CasualtyTrigger`. When resolved:
1. Call `copy::copy_spell_on_stack(state, original_stack_id, controller, false)`.
2. This creates exactly ONE copy (unlike Storm which creates N copies).
3. Emit the `SpellCopied` event.
4. Emit `AbilityResolved`.

**Pattern**: Follow `StormTrigger` resolution at resolution.rs ~line 773-791, but
call `copy_spell_on_stack` once instead of `create_storm_copies`.

**CR**: 702.153a -- copy the spell; "you may choose new targets for the copy" is handled
by the existing copy infrastructure (target redirection is a future enhancement -- for now,
copies share the original's targets, matching current Storm behavior).

### Step 4: Harness Support

**File**: `crates/engine/src/testing/replay_harness.rs`
**Action**: Add `casualty_sacrifice_name: Option<&str>` parameter to
`translate_player_action`. Add `"cast_spell_casualty"` action type that looks up the
sacrifice creature on the battlefield by name.
**Pattern**: Follow `"cast_spell_bargain"` at ~line 981-1001.

**File**: `crates/engine/src/testing/script_schema.rs`
**Action**: Add `casualty_sacrifice` field to the script action schema if needed.

### Step 5: All StackObject Construction Sites

Every place that constructs a `StackObject` needs `was_casualty_paid: false`.

Grep for `was_bargained:` to find all sites. Known sites:
- `casting.rs` (main cast -- use `casualty_sacrifice_id.is_some()`)
- `casting.rs` (storm trigger, cascade trigger -- `false`)
- `resolution.rs` (token creation sites -- `false`)
- `resolution.rs` (suspend cast -- `false`)
- `copy.rs` (copy_spell_on_stack -- propagate from original? CR 707.2 says copies copy
  the original's characteristics, but `was_casualty_paid` is not a copiable value. Set to
  `false` for copies.)

### Step 6: Unit Tests

**File**: `crates/engine/tests/casualty.rs`
**Tests to write**:

1. `test_casualty_basic_copy` -- CR 702.153a: Cast a spell with Casualty 1, sacrifice a
   1/1 creature. Verify: (a) creature goes to graveyard, (b) CasualtyTrigger appears on
   stack, (c) after resolving the trigger, a copy of the spell exists on the stack,
   (d) copy resolves before original (LIFO).

2. `test_casualty_power_threshold` -- CR 702.153a: Attempt to sacrifice a creature with
   power less than N. Verify: error/rejection. Then sacrifice a creature with power
   exactly N. Verify: accepted.

3. `test_casualty_optional_no_sacrifice` -- CR 702.153a: Cast a spell with Casualty
   without providing a sacrifice. Verify: spell resolves normally with no copy.

4. `test_casualty_not_a_creature` -- Attempt to sacrifice a non-creature (e.g., artifact).
   Verify: error/rejection. Casualty requires sacrificing a creature, unlike Bargain.

5. `test_casualty_wrong_controller` -- Attempt to sacrifice an opponent's creature.
   Verify: error/rejection.

6. `test_casualty_spell_without_keyword` -- Attempt to provide a casualty sacrifice for
   a spell that does not have the Casualty keyword. Verify: error/rejection.

7. `test_casualty_copy_is_not_cast` -- Verify that the copy does NOT trigger "whenever
   you cast a spell" abilities. Check `spells_cast_this_turn` is NOT incremented by the
   copy.

8. `test_casualty_higher_power_accepted` -- Sacrifice a creature with power > N (e.g.,
   Casualty 1 with a 3/3 creature). Verify: accepted.

**Pattern**: Follow `crates/engine/tests/bargain.rs` for test structure (synthetic card
definitions, `find_object`, `pass_all`, `GameStateBuilder`).

### Step 7: Card Definition (later phase)

**Suggested card**: A Little Chat (Casualty 1, simple instant with no extra copy
modification logic)
**Oracle text**: "Casualty 1. Look at the top two cards of your library. Put one of them
into your hand and the other on the bottom of your library."
**Note**: The "look at top 2, choose 1" effect is complex (requires SearchLibrary-like
choice). For testing, use a synthetic card definition with a simpler effect (e.g.,
DealDamage or GainLife) that demonstrates the copy behavior clearly.

**Alternative suggested card**: Make Disappear (Casualty 1, counter target spell unless
its controller pays {2}). Simpler effect but requires counter infrastructure.

**Recommended**: Use a synthetic test card like Bargain does (e.g., "Casualty Bolt" --
{R}, Casualty 1, deal 2 damage to target creature or player). This avoids DSL gaps.

### Step 8: Game Script (later phase)

**Suggested scenario**: Player casts a Casualty 1 spell targeting an opponent, sacrificing
a 1/1 creature token. The casualty trigger resolves, creating a copy. The copy resolves
(dealing damage), then the original resolves (dealing damage again). Opponent takes double
damage.
**Subsystem directory**: `test-data/generated-scripts/stack/`

## Interactions to Watch

- **Casualty sacrifice is a COST, not an effect.** The creature is sacrificed as part of
  casting (CR 601.2h). If the spell is countered, the creature is still gone. This is the
  same as Bargain.
- **Copy targets**: The rule says "you may choose new targets for the copy." The current
  copy infrastructure (`copy_spell_on_stack`) copies targets from the original. Target
  redirection for copies is a general infrastructure gap (also affects Storm). For now,
  copies share the original's targets. This is acceptable for initial implementation.
- **Planeswalker copies**: Ob Nixilis makes the copy "not legendary and has starting
  loyalty X." This is card-specific text that modifies the copy, NOT part of the Casualty
  keyword itself. Implementing this requires a hook in the copy pipeline to apply
  card-specific modifications. Deferred to card authoring phase.
- **Sacrifice feeds death triggers**: The sacrificed creature dying can trigger
  "whenever a creature dies" abilities (Zulaport Cutthroat, Blood Artist). These triggers
  are independent of the casualty keyword -- they come from the sacrifice event.

## Discriminant Summary

| Type | Variant | Discriminant |
|------|---------|-------------|
| `KeywordAbility` | `Casualty(u32)` | 104 |
| `StackObjectKind` | `CasualtyTrigger` | 34 |
| `AbilityDefinition` | (none needed -- N stored in KeywordAbility) | -- |

## Differences from Bargain Pattern

| Aspect | Bargain | Casualty |
|--------|---------|----------|
| Sacrifice target | Artifact, enchantment, or token | Creature with power >= N |
| Effect of paying | Card-specific (Condition::WasBargained) | Automatic copy of spell |
| Keyword parameter | None | N (minimum power) |
| Triggered ability | None | CasualtyTrigger -> copy spell |
| Copy mechanism | None | copy_spell_on_stack (like Storm x1) |
| `was_X` field needed | `was_bargained` on StackObject + GameObject | `was_casualty_paid` on StackObject only (no need on GameObject -- no permanent cares if casualty was paid after resolution) |

## Files to Modify (complete list)

1. `crates/engine/src/state/types.rs` -- KeywordAbility::Casualty(u32)
2. `crates/engine/src/state/stack.rs` -- StackObjectKind::CasualtyTrigger, was_casualty_paid field
3. `crates/engine/src/state/hash.rs` -- hash arms for KeywordAbility, StackObjectKind, StackObject field
4. `crates/engine/src/state/mod.rs` -- Command::CastSpell casualty_sacrifice field
5. `crates/engine/src/rules/engine.rs` -- destructure + pass casualty_sacrifice
6. `crates/engine/src/rules/casting.rs` -- validate, sacrifice, trigger creation
7. `crates/engine/src/rules/resolution.rs` -- CasualtyTrigger resolution (copy_spell_on_stack)
8. `crates/engine/src/testing/replay_harness.rs` -- cast_spell_casualty action
9. `crates/engine/src/testing/script_schema.rs` -- casualty_sacrifice field (if needed)
10. `tools/tui/src/play/panels/stack_view.rs` -- CasualtyTrigger match arm
11. `tools/replay-viewer/src/view_model.rs` -- CasualtyTrigger serialization (if exhaustive match)
12. `crates/engine/tests/casualty.rs` -- new test file (8 tests)
