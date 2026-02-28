# Ability Plan: Evoke

**Generated**: 2026-02-26
**CR**: 702.74
**Priority**: P2
**Similar abilities studied**: Kicker (CR 702.33) — `kicker_times_paid` flag on StackObject + GameObject, `AbilityDefinition::Kicker`, `Condition::WasKicked`; Flashback (CR 702.34) — alternative cost in `casting.rs`, `cast_with_flashback` on StackObject

## CR Rule Text

> **702.74. Evoke**
>
> **702.74a** Evoke represents two abilities: a static ability that functions in any zone
> from which the card with evoke can be cast and a triggered ability that functions on
> the battlefield. "Evoke [cost]" means "You may cast this card by paying [cost] rather
> than paying its mana cost" and "When this permanent enters, if its evoke cost was paid,
> its controller sacrifices it." Casting a spell for its evoke cost follows the rules for
> paying alternative costs in rules 601.2b and 601.2f-h.

### Supporting rules

- **118.9a**: Only one alternative cost can be applied to any one spell as it's being cast.
  Evoke CANNOT combine with flashback or other alternative costs.
- **118.9c**: An alternative cost doesn't change a spell's mana cost, only what its
  controller has to pay. Mana value is unchanged.
- **118.9d**: Additional costs (commander tax, kicker) and cost reductions still apply
  on top of the alternative cost.
- **601.2b**: The player announces the intention to pay an alternative cost (evoke) at
  cast announcement time.
- **601.2f**: Total cost = alternative cost + additional costs + cost increases - reductions.

## Key Edge Cases

1. **Evoke is an alternative cost (CR 118.9a)**: Cannot combine with flashback or any
   other alternative cost. CAN combine with additional costs (commander tax, kicker)
   and cost reductions (convoke, delve).
2. **ETB trigger and evoke sacrifice trigger are separate (Mulldrifter ruling)**: Both
   trigger simultaneously on ETB. The controller puts them on the stack in the order of
   their choice. If the card's ETB resolves first, the player gets the effect before
   the creature is sacrificed.
3. **Blinking/flickering an evoked creature (key interaction)**: If the creature is
   exiled and returned before the evoke sacrifice trigger resolves, it is a new object
   (CR 400.7). The sacrifice trigger fizzles (its target/source no longer exists).
   The creature survives. This is a well-known interaction but does NOT need special
   implementation -- it falls out naturally from CR 400.7 zone-change identity rules
   already implemented in the engine.
4. **Mana value unchanged (CR 118.9c)**: Mulldrifter's mana value is always 5, even
   when cast for its evoke cost of {2}{U}.
5. **Commander tax applies to evoke cost (CR 118.9d)**: If a commander with evoke is
   cast from the command zone using the evoke cost, commander tax ({2} per previous cast)
   is added on top.
6. **Evoke only applies to creature spells**: Evoke says "You may cast this card..." --
   the static ability functions from any zone the card can be cast from (hand, command
   zone). Only creature cards have evoke in practice (the rule text applies to
   "this permanent").
7. **Multiplayer**: No special multiplayer considerations. The sacrifice trigger fires
   normally. APNAP ordering for simultaneous ETB + evoke sacrifice triggers follows
   standard rules (controller orders their own triggers).

## Current State (from ability-wip.md)

- [ ] 1. Enum variant
- [ ] 2. Rule enforcement
- [ ] 3. Trigger wiring
- [ ] 4. Unit tests
- [ ] 5. Card definition
- [ ] 6. Game script
- [ ] 7. Coverage doc update

## Implementation Steps

### Step 1: Enum Variant and AbilityDefinition

**File**: `crates/engine/src/state/types.rs`
**Action**: Add `KeywordAbility::Evoke` variant after `Changeling` (line ~292).

```rust
/// CR 702.74: Evoke [cost] -- alternative cost; sacrifice on ETB if evoked.
///
/// Marker for quick presence-checking (`keywords.contains`).
/// The evoke cost itself is stored in `AbilityDefinition::Evoke { cost }`.
Evoke,
```

**Pattern**: Follows `KeywordAbility::Kicker` at line ~248, `KeywordAbility::Flashback` at line ~219 -- both are markers with cost stored in `AbilityDefinition`.

**File**: `crates/engine/src/cards/card_definition.rs`
**Action**: Add `AbilityDefinition::Evoke { cost: ManaCost }` variant.
**Location**: After `AbilityDefinition::Kicker` (around line ~163).

```rust
/// CR 702.74: Evoke [cost]. The card may be cast by paying this cost instead of
/// its mana cost (alternative cost, CR 118.9). When the permanent enters the
/// battlefield, if evoke was paid, its controller sacrifices it.
///
/// Cards with this ability should also include
/// `AbilityDefinition::Keyword(KeywordAbility::Evoke)` for quick
/// presence-checking without scanning all abilities.
Evoke { cost: ManaCost },
```

**File**: `crates/engine/src/state/hash.rs`
**Action 1**: Add `KeywordAbility::Evoke` arm to the `impl HashInto for KeywordAbility` match.
Find the existing match in hash.rs (search for `KeywordAbility::Changeling`), add:
```rust
KeywordAbility::Evoke => 25u8.hash_into(hasher),
```
(Use next sequential discriminant after `Changeling`.)

**Action 2**: Add `AbilityDefinition::Evoke { cost }` arm to the `impl HashInto for AbilityDefinition` match.
```rust
AbilityDefinition::Evoke { cost } => {
    <next_discriminant>u8.hash_into(hasher);
    cost.hash_into(hasher);
}
```

**Action 3**: Add `self.was_evoked.hash_into(hasher)` to `impl HashInto for StackObject` (after `kicker_times_paid` at line ~1118).

**Action 4**: Add `self.was_evoked.hash_into(hasher)` to `impl HashInto for GameObject` (after `kicker_times_paid` at line ~507).

**File**: `crates/engine/src/state/stack.rs`
**Action**: Add `was_evoked: bool` field to `StackObject`, after `kicker_times_paid` (line ~58).

```rust
/// CR 702.74a: If true, this spell was cast by paying its evoke cost
/// (an alternative cost). When the permanent enters the battlefield,
/// the evoke sacrifice trigger fires.
///
/// Must always be false for copies (`is_copy: true`) -- copies are not cast.
#[serde(default)]
pub was_evoked: bool,
```

**File**: `crates/engine/src/state/game_object.rs`
**Action**: Add `was_evoked: bool` field to `GameObject`, after `kicker_times_paid` (line ~310).

```rust
/// CR 702.74a: If true, this permanent was cast by paying its evoke cost.
/// The evoke sacrifice trigger checks this flag at ETB time.
///
/// Set during spell resolution when the permanent enters the battlefield.
/// Reset to false on zone changes (CR 400.7).
#[serde(default)]
pub was_evoked: bool,
```

**File**: `crates/engine/src/state/mod.rs`
**Action**: Add `was_evoked: false` to both `move_object_to_zone` sites where `kicker_times_paid: 0` appears (lines ~278 and ~348).

**File**: `crates/engine/src/state/builder.rs`
**Action**: Add `was_evoked: false` alongside `kicker_times_paid: 0` in the builder (line ~555).

**Match arm sweep**: Grep for all sites that construct `StackObject { ... }` or `GameObject { ... }` literals and add `was_evoked: false` / `was_evoked: true` as appropriate. Key sites:
- `casting.rs` (lines ~390-402, ~465-474, ~500-508): add `was_evoked` field to StackObject construction
- `abilities.rs` (lines ~330-336, ~480-486, ~1060-1066): add `was_evoked: false` to trigger StackObjects
- `copy.rs` (lines ~170-177, ~340-344): add `was_evoked: false` (copies are not cast with evoke)
- `effects/mod.rs` (line ~1739): add `was_evoked: false` to token GameObject construction

### Step 2: Cast-time Alternative Cost (Rule Enforcement)

**File**: `crates/engine/src/rules/casting.rs`
**Action**: Add evoke cost handling, similar to flashback.

**2a. Add `cast_with_evoke` parameter to `handle_cast_spell`**

Add a `cast_with_evoke: bool` parameter to `handle_cast_spell`. This follows the same pattern as `kicker_times`, `convoke_creatures`, and `delve_cards`.

**2b. Validate evoke is available**

After the flashback zone check (around line ~88), add evoke validation:

```rust
// CR 702.74a: Evoke is an alternative cost. Validate the spell has evoke
// and that no other alternative cost (flashback) is being used.
let casting_with_evoke = if cast_with_evoke {
    if casting_with_flashback {
        return Err(GameStateError::InvalidCommand(
            "cannot combine evoke with flashback (CR 118.9a: only one alternative cost)".into(),
        ));
    }
    // Verify the card has evoke ability
    if get_evoke_cost(&card_id, &state.card_registry).is_none() {
        return Err(GameStateError::InvalidCommand(
            "spell does not have evoke".into(),
        ));
    }
    true
} else {
    false
};
```

**2c. Apply evoke cost as alternative cost**

In the cost determination section (around line ~159), add evoke alongside flashback:

```rust
let mana_cost: Option<ManaCost> = if casting_with_flashback {
    get_flashback_cost(&card_id, &state.card_registry)
} else if casting_with_evoke {
    // CR 702.74a / 118.9: Pay evoke cost instead of mana cost.
    get_evoke_cost(&card_id, &state.card_registry)
} else if casting_from_command_zone {
    // ... existing commander tax logic
} else {
    base_mana_cost
};
```

Note: If casting from command zone AND with evoke, the evoke cost is the base alternative cost, and commander tax is added on top (CR 118.9d). The casting_from_command_zone tax application happens AFTER the base cost is determined, so the order should be:
1. evoke cost (alternative cost)
2. commander tax (additional cost, applied on top of alternative cost per CR 118.9d)
3. kicker (additional cost)
4. convoke/delve (cost reduction)

Adjust the cost determination pipeline so the `casting_from_command_zone` tax is applied AFTER the `casting_with_evoke` base, not as an alternative branch.

**2d. Set `was_evoked` on StackObject**

In the StackObject construction (around line ~396):

```rust
was_evoked: casting_with_evoke,
```

**2e. Add `get_evoke_cost` helper function**

After `get_kicker_cost` (line ~561), add:

```rust
/// CR 702.74a: Look up the evoke cost from the card's `AbilityDefinition`.
///
/// Returns the `ManaCost` stored in `AbilityDefinition::Evoke { cost }`, or `None`
/// if the card has no evoke ability.
fn get_evoke_cost(
    card_id: &Option<crate::state::CardId>,
    registry: &crate::cards::CardRegistry,
) -> Option<ManaCost> {
    card_id.as_ref().and_then(|cid| {
        registry.get(cid.clone()).and_then(|def| {
            def.abilities.iter().find_map(|a| {
                if let AbilityDefinition::Evoke { cost } = a {
                    Some(cost.clone())
                } else {
                    None
                }
            })
        })
    })
}
```

**File**: `crates/engine/src/rules/command.rs`
**Action**: Add `cast_with_evoke: bool` field to `Command::CastSpell` variant (after `kicker_times`, line ~88).

```rust
/// CR 702.74a: If true, cast this spell by paying its evoke cost instead
/// of its mana cost. This is an alternative cost (CR 118.9) -- cannot
/// combine with flashback or other alternative costs.
/// Ignored for spells without evoke.
#[serde(default)]
cast_with_evoke: bool,
```

**File**: `crates/engine/src/rules/engine.rs`
**Action**: Update the `Command::CastSpell` match arm (around line ~70) to destructure `cast_with_evoke` and pass it to `handle_cast_spell`.

### Step 3: Resolution-time Transfer and ETB Sacrifice Trigger

**File**: `crates/engine/src/rules/resolution.rs`
**Action 3a**: Transfer `was_evoked` from StackObject to GameObject at resolution.

After `obj.kicker_times_paid = stack_obj.kicker_times_paid;` (line ~187), add:

```rust
// CR 702.74a: Transfer evoked status from stack to permanent so
// ETB sacrifice trigger can check was_evoked.
obj.was_evoked = stack_obj.was_evoked;
```

**Action 3b**: Generate the evoke sacrifice trigger at ETB.

The evoke sacrifice trigger is a triggered ability defined by the evoke keyword itself
(CR 702.74a: "When this permanent enters, if its evoke cost was paid, its controller
sacrifices it."). This trigger should go on the stack like any other triggered ability,
allowing the controller to order it relative to the creature's own ETB trigger.

The best approach is to generate the sacrifice trigger in
`fire_when_enters_triggered_effects` in `replacement.rs` (which runs at both ETB sites).

**File**: `crates/engine/src/rules/replacement.rs`
**Location**: Inside `fire_when_enters_triggered_effects` (line ~880), after the loop over `def.abilities`.

```rust
// CR 702.74a: If the permanent was evoked, generate the sacrifice trigger.
// "When this permanent enters, if its evoke cost was paid, its controller
// sacrifices it."
let was_evoked = state
    .objects
    .get(&new_id)
    .map(|o| o.was_evoked)
    .unwrap_or(false);

if was_evoked {
    // The evoke sacrifice trigger goes on the stack as a triggered ability.
    // It sacrifices the source permanent when it resolves.
    let sacrifice_effect = Effect::SacrificeSource;
    let mut ctx = EffectContext::new(controller, new_id, vec![]);
    evts.extend(execute_effect(state, &sacrifice_effect, &mut ctx));
}
```

**IMPORTANT DESIGN DECISION**: The above approach executes the sacrifice INLINE (not
through the stack). This is simpler but does NOT allow the controller to order it
relative to the creature's own ETB trigger. Per the Mulldrifter ruling:

> "If you pay the evoke cost, you can have Mulldrifter's own triggered ability resolve
> before the evoke triggered ability."

This means the evoke sacrifice MUST go through the stack as a separate triggered ability,
not be executed inline.

**Revised approach**: Instead of executing inline, push a PendingTrigger for the evoke
sacrifice. This requires:

1. Adding the evoke sacrifice as a `TriggeredAbilityDef` on the creature when it enters
   (if `was_evoked` is true), OR
2. Generating a PendingTrigger directly in the check_triggers path.

**Best approach**: Handle it in `check_triggers` in `abilities.rs`. When a
`PermanentEnteredBattlefield` event fires, check if the entering permanent has
`was_evoked == true`. If so, push a PendingTrigger for the evoke sacrifice.

**File**: `crates/engine/src/rules/abilities.rs`
**Location**: In `check_triggers`, inside the `PermanentEnteredBattlefield` arm (line ~545), after the existing `collect_triggers_for_event` calls.

```rust
// CR 702.74a: If the permanent was evoked, generate the sacrifice trigger.
// This is a separate triggered ability that goes on the stack, allowing
// the controller to order it relative to other ETB triggers.
if let Some(obj) = state.objects.get(object_id) {
    if obj.was_evoked {
        let evoke_trigger = PendingTrigger {
            source: *object_id,
            ability_index: usize::MAX, // Sentinel: not an indexed ability
            controller: obj.controller,
            triggering_event: Some(TriggerEvent::SelfEntersBattlefield),
            entering_object_id: Some(*object_id),
            triggering_player: None,
        };
        triggers.push(evoke_trigger);
    }
}
```

**File**: `crates/engine/src/rules/abilities.rs`
**Location**: In `flush_pending_triggers` (where pending triggers become stack objects),
add special handling for `ability_index == usize::MAX` to create the sacrifice effect.

Find the section where `StackObjectKind::TriggeredAbility` is constructed from a
PendingTrigger and add:

```rust
// CR 702.74a: Evoke sacrifice trigger — ability_index == usize::MAX sentinel.
// Effect: sacrifice the source permanent.
let kind = if trigger.ability_index == usize::MAX {
    // Evoke sacrifice: embed the sacrifice effect directly
    StackObjectKind::TriggeredAbility {
        source_object: trigger.source,
        ability_index: usize::MAX,
    }
} else {
    // Normal triggered ability
    StackObjectKind::TriggeredAbility {
        source_object: trigger.source,
        ability_index: trigger.ability_index,
    }
};
```

Then at resolution time, when the TriggeredAbility with `ability_index == usize::MAX`
resolves, the engine needs to know it's an evoke sacrifice and execute the sacrifice.

**File**: `crates/engine/src/rules/resolution.rs`
**Location**: In the TriggeredAbility resolution path, add a special case for the evoke
sacrifice sentinel.

```rust
// CR 702.74a: Evoke sacrifice trigger — sentinel ability_index.
if ability_index == usize::MAX {
    // Sacrifice the source permanent.
    if let Some(obj) = state.objects.get(&source_object) {
        if obj.zone == ZoneId::Battlefield {
            let controller = obj.controller;
            let (_, _) = state.move_object_to_zone(source_object, ZoneId::Graveyard(obj.owner))?;
            events.push(GameEvent::CreatureDied {
                object_id: source_object,
                controller,
                owner: obj.owner,
                pre_death_counters: obj.counters.clone(),
            });
        }
        // If the source is not on the battlefield (blinked, bounced, etc.),
        // the sacrifice trigger does nothing (CR 400.7: new object).
    }
} else {
    // Normal triggered ability resolution...
}
```

**ALTERNATIVE (cleaner) DESIGN**: Instead of using `ability_index == usize::MAX` as a
sentinel, add a new `StackObjectKind::EvokeSacrificeTrigger { source_object }` variant.
This is cleaner but requires updating more match arms.

**RECOMMENDED DESIGN**: Use a new `StackObjectKind` variant. It is more explicit, avoids
sentinel values, and is self-documenting. The cost is adding match arms to:
- `stack_kind_info` in `view_model.rs`
- `HashInto for StackObjectKind` in `hash.rs`
- Resolution match in `resolution.rs`
- Any other pattern matches on `StackObjectKind`

```rust
/// CR 702.74a: Evoke sacrifice trigger on the stack.
///
/// When an evoked permanent enters the battlefield, this trigger fires:
/// "When this permanent enters, if its evoke cost was paid, its controller
/// sacrifices it." Resolves to sacrifice the source permanent.
///
/// If the source has left the battlefield by resolution time (blinked,
/// bounced, etc.), the trigger does nothing per CR 400.7.
EvokeSacrificeTrigger {
    source_object: ObjectId,
},
```

### Step 4: Replay Harness Support

**File**: `crates/engine/src/testing/replay_harness.rs`
**Action**: Add `"cast_spell_evoke"` action type in `translate_player_action` (after
`"cast_spell_flashback"`, line ~258).

```rust
// CR 702.74a: Cast a spell with evoke from the player's hand.
"cast_spell_evoke" => {
    let card_id = find_in_hand(state, player, card_name?)?;
    let target_list = resolve_targets(targets, state, players);
    Some(Command::CastSpell {
        player,
        card: card_id,
        targets: target_list,
        convoke_creatures: vec![],
        delve_cards: vec![],
        kicker_times: 0,
        cast_with_evoke: true,
    })
}
```

Also update the existing `"cast_spell"` action to pass `cast_with_evoke: false`.

### Step 5: Unit Tests

**File**: `crates/engine/tests/evoke.rs` (new file)
**Tests to write**:

1. **`test_evoke_basic_cast_with_evoke_cost`** -- CR 702.74a: Cast Mulldrifter for {2}{U}
   evoke cost. Verify it enters the battlefield, draw 2 triggers fire, then evoke
   sacrifice trigger fires and creature goes to graveyard.

2. **`test_evoke_basic_cast_without_evoke`** -- CR 702.74a: Cast Mulldrifter for {4}{U}
   normal cost. Verify it enters and stays on the battlefield. No sacrifice.

3. **`test_evoke_sacrifice_trigger_goes_through_stack`** -- Mulldrifter ruling: ETB draw
   trigger and evoke sacrifice trigger both go on the stack. Verify both are on the
   stack after ETB, allowing the controller to resolve draw first.

4. **`test_evoke_does_not_change_mana_value`** -- CR 118.9c: Mulldrifter's mana value
   remains 5 even when evoked for {2}{U}.

5. **`test_evoke_cannot_combine_with_flashback`** -- CR 118.9a: Attempting to cast with
   both evoke and flashback should fail with an error.

6. **`test_evoke_non_evoke_spell_rejected`** -- Engine validation: Setting
   `cast_with_evoke: true` on a spell without evoke should fail.

7. **`test_evoke_with_commander_tax`** -- CR 118.9d: Commander tax stacks on top of
   evoke cost. If a commander with evoke cost {2}{U} has been cast once before,
   the total cost is {4}{U} (evoke {2}{U} + tax {2}).

8. **`test_evoke_blink_saves_creature`** -- CR 400.7: If the evoked creature is
   exiled and returned before the sacrifice trigger resolves, the sacrifice trigger
   finds a new object and does nothing. (Test this by verifying the sacrifice effect
   checks the source's identity.)

**Pattern**: Follow the kicker tests in `crates/engine/tests/kicker.rs` for:
- Helper functions (`p()`, `find_object()`, `pass_all()`)
- Registry setup with `all_cards()` + filter
- ObjectSpec construction with `.with_keyword(KeywordAbility::Evoke)`
- Command construction with `cast_with_evoke: true`
- Post-resolution assertions

### Step 6: Card Definition (later phase)

**Suggested card**: Mulldrifter

**Oracle text**:
```
Flying
When this creature enters, draw two cards.
Evoke {2}{U}
```

**CardDefinition** (structure):
```rust
CardDefinition {
    name: "Mulldrifter".to_string(),
    mana_cost: Some(ManaCost { generic: 4, blue: 1, ..Default::default() }),
    types: TypeLine {
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: vec![SubType("Elemental".to_string())],
    },
    oracle_text: "Flying\nWhen this creature enters, draw two cards.\nEvoke {2}{U}".to_string(),
    abilities: vec![
        AbilityDefinition::Keyword(KeywordAbility::Flying),
        AbilityDefinition::Triggered {
            trigger_condition: TriggerCondition::WhenEntersBattlefield,
            effect: Effect::DrawCards {
                player: PlayerTarget::Controller,
                count: EffectAmount::Fixed(2),
            },
        },
        AbilityDefinition::Evoke {
            cost: ManaCost { generic: 2, blue: 1, ..Default::default() },
        },
        AbilityDefinition::Keyword(KeywordAbility::Evoke),
    ],
    power: Some(2),
    toughness: Some(2),
    ..Default::default()
}
```

**Card lookup**: Use `card-definition-author` agent for Mulldrifter.
**Secondary card**: Shriekmaw (targeted ETB + evoke) for testing targeted interaction.

### Step 7: Game Script (later phase)

**Suggested scenario**: Mulldrifter evoke -- cast for evoke cost, draw 2 cards, sacrifice.
**Subsystem directory**: `test-data/generated-scripts/stack/`
**Script outline**:
1. P1 has Mulldrifter in hand, {2}{U} in mana pool
2. P1 casts Mulldrifter with evoke
3. All players pass priority -- spell resolves
4. Mulldrifter enters battlefield
5. ETB draw trigger and evoke sacrifice trigger both go on stack
6. All players pass priority -- draw trigger resolves (P1 draws 2)
7. All players pass priority -- sacrifice trigger resolves (Mulldrifter dies)
8. Assert: P1 hand has 2 more cards, Mulldrifter in P1's graveyard

## Interactions to Watch

1. **Evoke + Commander tax (CR 118.9d)**: Alternative costs still receive additional costs.
   The cost pipeline in `casting.rs` must apply commander tax AFTER the evoke alternative
   cost is selected. Currently, the flashback/commander-zone cost branches are mutually
   exclusive -- evoke must integrate differently so tax is additive, not alternative.

2. **Evoke + Kicker (CR 118.9d)**: Additional costs (kicker) can be paid on top of evoke.
   A player could theoretically kick an evoked spell if the creature has kicker.
   The existing kicker pipeline already applies after base cost, so this should work
   automatically.

3. **Evoke + Convoke/Delve**: These are cost reductions, not alternative costs. They
   apply after total cost is determined. Should work automatically with evoke as the
   base alternative cost.

4. **Zone-change identity (CR 400.7)**: If an evoked creature leaves and re-enters the
   battlefield before the sacrifice trigger resolves, the trigger references the old
   object and does nothing. This falls out naturally from the engine's zone-change
   identity rules -- no special implementation needed. However, the sacrifice trigger
   resolution code MUST check that `source_object` is still on the battlefield.

5. **Evoke sacrifice is NOT an SBA**: It is a triggered ability that uses the stack.
   Players get priority between the ETB trigger and the sacrifice trigger. This is
   critical for the Mulldrifter/Grief interaction patterns.

6. **Copy effects**: Copies of evoked spells on the stack are NOT evoked (copies are
   not cast). Set `was_evoked: false` on copies.

## Files Modified (Summary)

| File | Changes |
|------|---------|
| `crates/engine/src/state/types.rs` | Add `KeywordAbility::Evoke` |
| `crates/engine/src/cards/card_definition.rs` | Add `AbilityDefinition::Evoke { cost }` |
| `crates/engine/src/state/stack.rs` | Add `was_evoked: bool` to `StackObject`, add `EvokeSacrificeTrigger` variant to `StackObjectKind` |
| `crates/engine/src/state/game_object.rs` | Add `was_evoked: bool` to `GameObject` |
| `crates/engine/src/state/hash.rs` | Hash `KeywordAbility::Evoke`, `AbilityDefinition::Evoke`, `was_evoked` on both `StackObject` and `GameObject`, `StackObjectKind::EvokeSacrificeTrigger` |
| `crates/engine/src/state/mod.rs` | Add `was_evoked: false` to zone-change constructors |
| `crates/engine/src/state/builder.rs` | Add `was_evoked: false` to builder |
| `crates/engine/src/rules/command.rs` | Add `cast_with_evoke: bool` to `Command::CastSpell` |
| `crates/engine/src/rules/engine.rs` | Destructure and pass `cast_with_evoke` |
| `crates/engine/src/rules/casting.rs` | Add parameter, evoke validation, evoke cost determination, `get_evoke_cost()` helper |
| `crates/engine/src/rules/resolution.rs` | Transfer `was_evoked` to permanent, handle `EvokeSacrificeTrigger` resolution |
| `crates/engine/src/rules/abilities.rs` | Generate evoke sacrifice `PendingTrigger` in `check_triggers` on `PermanentEnteredBattlefield` |
| `crates/engine/src/rules/replacement.rs` | No changes needed (sacrifice goes through stack, not inline) |
| `crates/engine/src/rules/copy.rs` | Add `was_evoked: false` to copy StackObjects |
| `crates/engine/src/effects/mod.rs` | Add `was_evoked: false` to token construction |
| `crates/engine/src/testing/replay_harness.rs` | Add `"cast_spell_evoke"` action, update existing actions with `cast_with_evoke: false` |
| `tools/replay-viewer/src/view_model.rs` | Add `EvokeSacrificeTrigger` arm to `stack_kind_info` |
| `crates/engine/tests/evoke.rs` | New test file with 8 tests |
