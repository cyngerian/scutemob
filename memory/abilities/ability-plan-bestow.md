# Ability Plan: Bestow

**Generated**: 2026-02-27
**CR**: 702.103
**Priority**: P3
**Similar abilities studied**: Evoke (alternative cost + special resolution behavior; `crates/engine/src/rules/casting.rs`, `crates/engine/src/rules/resolution.rs`, `crates/engine/src/rules/abilities.rs`, `crates/engine/tests/evoke.rs`), Enchant (Aura targeting + attachment + SBA; `crates/engine/src/rules/casting.rs:288-334`, `crates/engine/src/rules/resolution.rs:193-243`, `crates/engine/src/rules/sba.rs:578-712`, `crates/engine/tests/enchant.rs`)

## CR Rule Text

702.103. Bestow

702.103a Bestow represents a static ability that functions in any zone from which you could play the card it's on. "Bestow [cost]" means "As you cast this spell, you may choose to cast it bestowed. If you do, you pay [cost] rather than its mana cost." Casting a spell using its bestow ability follows the rules for paying alternative costs (see 601.2b and 601.2f-h).

702.103b As a spell cast bestowed is put onto the stack, it becomes an Aura enchantment and gains enchant creature. It is a bestowed Aura spell, and the permanent it becomes as it resolves will be a bestowed Aura. These effects last until the spell or the permanent it becomes ceases to be bestowed (see rules 702.103e-g). Because the spell is an Aura spell, its controller must choose a legal target for that spell as defined by its enchant creature ability and rule 601.2c. See also rule 303.4.

702.103c If a bestowed Aura spell is copied, the copy is also a bestowed Aura spell. Any rule that refers to a spell cast bestowed applies to the copy as well.

702.103d When casting a spell bestowed, only its characteristics as modified by the bestow ability are evaluated to determine if it can be cast.

702.103e As a bestowed Aura spell begins resolving, if its target is illegal, it ceases to be bestowed and the effect making it an Aura spell ends. It continues resolving as a creature spell. See rule 608.3b.

702.103f If a bestowed Aura becomes unattached, it ceases to be bestowed. If a bestowed Aura is attached to an illegal object or player, it becomes unattached and ceases to be bestowed. This is an exception to rule 704.5m.

702.103g If a bestowed Aura phases in unattached, it ceases to be bestowed. See rule 702.26, "Phasing."

608.3b If the object that's resolving has a target, it checks whether the target is still legal, as described in 608.2b. If a spell with an illegal target is a bestowed Aura spell (see rule 702.103e) or a mutating creature spell (see rule 702.140b), it becomes a creature spell and will resolve as described in rule 608.3a. Otherwise, the spell doesn't resolve. It is removed from the stack and put into its owner's graveyard.

## Key Edge Cases

From card rulings (Boon Satyr, Nighthowler, Eidolon of Countless Battles):

1. **Bestow is an alternative cost** (CR 118.9). It cannot be combined with other alternative costs (flashback, evoke). Matches existing pattern from evoke.
2. **Target becomes illegal at resolution** -- CR 702.103e / 608.3b: The spell "falls back" to being a creature instead of fizzling. This is the core edge case. The spell enters the battlefield as an enchantment creature, NOT as an Aura.
3. **Enters battlefield without being cast** -- If a permanent with bestow enters by any method other than being cast (e.g., reanimation, blink), it enters as an enchantment creature, never as an Aura. The bestow choice only applies at cast time.
4. **Unattach reverts to creature** -- CR 702.103f: When a bestowed Aura becomes unattached (enchanted creature leaves battlefield, or Aura attached to illegal object), it ceases to be bestowed and stays on the battlefield as an enchantment creature. This is an **exception to CR 704.5m** -- normal Auras go to the graveyard when unattached; bestowed Auras become creatures instead.
5. **A permanent with bestow is EITHER a creature OR an Aura, never both** (ruling). While bestowed, it is an Aura enchantment (not a creature). When not bestowed, it is an enchantment creature.
6. **An Aura that's also a creature can't enchant anything** (CR 303.4d). This is a state-based action. Bestow avoids this by making the permanent NOT a creature while bestowed.
7. **Mana value unchanged** (CR 118.9c): The printed mana cost, not the bestow cost, determines mana value. Same as evoke.
8. **Summoning sickness tracking**: When a bestowed Aura becomes a creature (due to unattach), it can attack and use tap abilities on the turn it becomes a creature IF it has been under the controller's control continuously since their most recent turn began (even while it was an Aura).
9. **Multiplayer**: No special multiplayer considerations beyond standard Aura + creature rules. The bestow choice and fallback work identically in 1v1 and multiplayer.

## Current State (from ability-wip.md)

- [ ] Step 1: Enum variant
- [ ] Step 2: Rule enforcement
- [ ] Step 3: Trigger wiring (n/a -- bestow has no triggers)
- [ ] Step 4: Unit tests
- [ ] Step 5: Card definition
- [ ] Step 6: Game script
- [ ] Step 7: Coverage doc update

## Implementation Steps

### Step 1: Enum Variant and Data Model

#### 1a: KeywordAbility::Bestow

**File**: `crates/engine/src/state/types.rs`
**Action**: Add `Bestow` variant to `KeywordAbility` enum after `Fear` (line ~346).
**Pattern**: Follow `KeywordAbility::Evoke` at line 293 -- marker keyword with cost stored in `AbilityDefinition::Bestow`.
**CR**: 702.103a

```rust
/// CR 702.103: Bestow [cost] -- alternative cost; becomes Aura with enchant creature.
///
/// Marker for quick presence-checking (`keywords.contains`).
/// The bestow cost itself is stored in `AbilityDefinition::Bestow { cost }`.
///
/// When cast bestowed (CR 702.103b): spell becomes an Aura enchantment, gains
/// enchant creature, loses creature type. When unattached (CR 702.103f): ceases
/// to be bestowed, reverts to enchantment creature.
Bestow,
```

#### 1b: AbilityDefinition::Bestow

**File**: `crates/engine/src/cards/card_definition.rs`
**Action**: Add `Bestow { cost: ManaCost }` variant to `AbilityDefinition` enum after `Evoke` (line ~175).
**Pattern**: Follow `AbilityDefinition::Evoke { cost }` at line 175.
**CR**: 702.103a

```rust
/// CR 702.103: Bestow [cost]. The card may be cast by paying this cost instead
/// of its mana cost (alternative cost, CR 118.9). When cast bestowed, the spell
/// becomes an Aura enchantment with enchant creature (CR 702.103b).
///
/// Cards with this ability should also include
/// `AbilityDefinition::Keyword(KeywordAbility::Bestow)` for quick
/// presence-checking without scanning all abilities.
Bestow { cost: ManaCost },
```

#### 1c: StackObject::was_bestowed

**File**: `crates/engine/src/state/stack.rs`
**Action**: Add `was_bestowed: bool` field to `StackObject` struct (after `was_evoked` at line ~65).
**Pattern**: Follow `was_evoked` field at line 59-65.
**CR**: 702.103b

```rust
/// CR 702.103b: If true, this spell was cast by paying its bestow cost.
/// On the stack, this spell is an Aura enchantment (not a creature) with
/// enchant creature. At resolution, if the target is illegal, it ceases
/// to be bestowed and resolves as a creature (CR 702.103e / 608.3b).
///
/// Must always be false for copies (`is_copy: true`) -- copies inherit
/// bestowed status from the original (CR 702.103c).
#[serde(default)]
pub was_bestowed: bool,
```

**IMPORTANT**: Also add `was_bestowed: false` to EVERY existing `StackObject { ... }` literal in the codebase. Grep for `StackObject {` and `was_evoked:` to find all sites:
- `crates/engine/src/rules/casting.rs` (main CastSpell handler, storm trigger, cascade trigger)
- `crates/engine/src/rules/abilities.rs` (flush_pending_triggers, activated ability, crew ability)

#### 1d: GameObject::is_bestowed

**File**: `crates/engine/src/state/game_object.rs`
**Action**: Add `is_bestowed: bool` field to `GameObject` struct (after `was_evoked` at line ~322).
**Pattern**: Follow `was_evoked` field at line 316-322.
**CR**: 702.103b, 702.103f

```rust
/// CR 702.103b: If true, this permanent is currently bestowed. While bestowed,
/// it is an Aura enchantment (NOT a creature) with enchant creature.
/// CR 702.103f: When it becomes unattached, it ceases to be bestowed and
/// reverts to an enchantment creature -- this is an exception to CR 704.5m
/// (normal Auras go to graveyard when unattached; bestowed Auras become creatures).
///
/// Set during spell resolution when the permanent enters the battlefield
/// as a bestowed Aura. Reset to false when unattached (SBA) or on zone
/// changes (CR 400.7).
#[serde(default)]
pub is_bestowed: bool,
```

#### 1e: Hash updates

**File**: `crates/engine/src/state/hash.rs`
**Action**: Add `self.was_bestowed.hash_into(hasher)` to the `StackObject` HashInto impl (after `was_evoked` at line ~1151), and `self.is_bestowed.hash_into(hasher)` to the `GameObject` HashInto impl (after `was_evoked` at line ~529).
**Pattern**: Follow `was_evoked` hashing.

#### 1f: View model update

**File**: `tools/replay-viewer/src/view_model.rs`
**Action**: Add `KeywordAbility::Bestow => "Bestow".to_string()` to the `format_keyword` match (after `Fear` at line ~612).
**Pattern**: Follow the other keyword arms in `format_keyword`.

### Step 2: Rule Enforcement -- Casting

**File**: `crates/engine/src/rules/casting.rs`
**Action**: Integrate bestow as an alternative cost alongside evoke and flashback.
**CR**: 702.103a, 702.103b, 702.103d, 118.9a

#### 2a: Add `cast_with_bestow: bool` to CastSpell Command

**File**: `crates/engine/src/rules/command.rs`
**Action**: Add `cast_with_bestow: bool` field to `Command::CastSpell` (after `cast_with_evoke` at line ~105).

```rust
/// CR 702.103a: If true, cast this spell by paying its bestow cost instead
/// of its mana cost. This is an alternative cost (CR 118.9) -- cannot
/// combine with flashback, evoke, or other alternative costs.
/// Ignored for spells without bestow.
#[serde(default)]
cast_with_bestow: bool,
```

**IMPORTANT**: Also update all `Command::CastSpell { ... }` constructor sites to include `cast_with_bestow: false`:
- `crates/engine/src/testing/replay_harness.rs` (all existing cast_spell variants)
- `crates/engine/tests/` (every test that constructs `Command::CastSpell`)
- `crates/engine/src/rules/casting.rs` (if any CastSpell construction exists)

#### 2b: Add `cast_with_bestow` parameter to `handle_cast_spell`

**File**: `crates/engine/src/rules/casting.rs`
**Action**: Add `cast_with_bestow: bool` to the function signature (line ~56). Pass it from the Command handler.

**Validation logic (insert after evoke validation at ~line 180)**:

```rust
// Step 1b: Validate bestow (CR 702.103a / CR 118.9a).
let casting_with_bestow = if cast_with_bestow {
    if casting_with_flashback {
        return Err(GameStateError::InvalidCommand(
            "cannot combine bestow with flashback (CR 118.9a: only one alternative cost)".into(),
        ));
    }
    if casting_with_evoke {
        return Err(GameStateError::InvalidCommand(
            "cannot combine bestow with evoke (CR 118.9a: only one alternative cost)".into(),
        ));
    }
    if get_bestow_cost(&card_id, &state.card_registry).is_none() {
        return Err(GameStateError::InvalidCommand(
            "spell does not have bestow".into(),
        ));
    }
    true
} else {
    false
};
```

**Cost selection (modify Step 2 at ~line 183)**:

Add bestow to the base cost selection chain:

```rust
let base_cost_before_tax: Option<ManaCost> = if casting_with_evoke {
    get_evoke_cost(&card_id, &state.card_registry)
} else if casting_with_bestow {
    // CR 702.103a: Pay bestow cost instead of mana cost.
    get_bestow_cost(&card_id, &state.card_registry)
} else if casting_with_flashback {
    get_flashback_cost(&card_id, &state.card_registry)
} else {
    base_mana_cost
};
```

#### 2c: Bestow type transformation on the stack

**File**: `crates/engine/src/rules/casting.rs`
**Action**: After the spell is put on the stack (after line ~461), if `casting_with_bestow`, modify the stack object's source's characteristics on the stack. This implements CR 702.103b.

**CR**: 702.103b -- "As a spell cast bestowed is put onto the stack, it becomes an Aura enchantment and gains enchant creature."

```rust
// CR 702.103b: When cast bestowed, the spell becomes an Aura enchantment
// and gains enchant creature. It loses the Creature type while on the stack.
if casting_with_bestow {
    if let Some(stack_source) = state.objects.get_mut(&new_card_id) {
        // Remove Creature type; add Enchantment if not present
        stack_source.characteristics.card_types.remove(&CardType::Creature);
        stack_source.characteristics.card_types.insert(CardType::Enchantment);
        // Add "Aura" subtype
        stack_source.characteristics.subtypes.insert(SubType("Aura".to_string()));
        // Add enchant creature keyword
        stack_source.characteristics.keywords.insert(
            KeywordAbility::Enchant(EnchantTarget::Creature)
        );
    }
}
```

**CRITICAL**: Because we already transformed the characteristics, the Aura target validation code at lines 293-334 (which checks for Aura + Enchant keyword) will now apply correctly. However, there's a sequencing issue: the Aura target validation happens BEFORE the spell is put on the stack. We need to either:

(a) Move the bestow type transformation to happen BEFORE target validation (recommended), or
(b) Add separate target validation for bestow spells.

**Recommended approach**: Transform `chars` (the local characteristics copy used for validation) BEFORE the Aura target check. Insert at approximately line 288 (before the Aura target validation block):

```rust
// CR 702.103b: When cast bestowed, modify characteristics for validation.
// The spell is treated as an Aura enchantment with enchant creature.
if casting_with_bestow {
    chars.card_types.remove(&CardType::Creature);
    chars.card_types.insert(CardType::Enchantment);
    chars.subtypes.insert(SubType("Aura".to_string()));
    chars.keywords.insert(KeywordAbility::Enchant(EnchantTarget::Creature));
}
```

This way, the existing Aura target validation code at lines 293-334 will naturally enforce "enchant creature" targeting for bestow spells. The spell MUST have a target when cast bestowed.

**Non-bestowed cast of bestow cards**: When a bestow card is cast normally (without bestow), it is cast as an enchantment creature spell with no target needed (no Aura, no enchant creature). The existing code handles this correctly because the card's printed types include Enchantment and Creature but NOT Aura.

#### 2d: Add `get_bestow_cost` helper

**File**: `crates/engine/src/rules/casting.rs`
**Action**: Add after `get_evoke_cost` function (line ~615).
**Pattern**: Follow `get_evoke_cost` / `get_flashback_cost`.

```rust
/// CR 702.103a / CR 118.9: Look up the bestow cost from the card's `AbilityDefinition`.
fn get_bestow_cost(
    card_id: &Option<crate::state::CardId>,
    registry: &crate::cards::CardRegistry,
) -> Option<ManaCost> {
    card_id.as_ref().and_then(|cid| {
        registry.get(cid.clone()).and_then(|def| {
            def.abilities.iter().find_map(|a| {
                if let AbilityDefinition::Bestow { cost } = a {
                    Some(cost.clone())
                } else {
                    None
                }
            })
        })
    })
}
```

#### 2e: Set `was_bestowed` on StackObject

**File**: `crates/engine/src/rules/casting.rs`
**Action**: In the main StackObject construction (~line 451), set `was_bestowed: casting_with_bestow`.

#### 2f: Add `cast_spell_bestow` action to replay harness

**File**: `crates/engine/src/testing/replay_harness.rs`
**Action**: Add a `cast_spell_bestow` arm in `translate_player_action` (after `cast_spell_evoke` at ~line 285).
**Pattern**: Follow `cast_spell_evoke`.

```rust
// CR 702.103a: Cast a spell with bestow from the player's hand.
// The bestow cost (an alternative cost) is paid instead of the mana cost.
"cast_spell_bestow" => {
    let card_id = find_in_hand(state, player, card_name?)?;
    let target_list = resolve_targets(targets, state, players);
    Some(Command::CastSpell {
        player,
        card: card_id,
        targets: target_list,
        convoke_creatures: vec![],
        improvise_artifacts: vec![],
        delve_cards: vec![],
        kicker_times: 0,
        cast_with_evoke: false,
        cast_with_bestow: true,
    })
}
```

### Step 3: Rule Enforcement -- Resolution

**File**: `crates/engine/src/rules/resolution.rs`
**Action**: Implement CR 702.103e / 608.3b -- bestow "fall off" at resolution, and CR 702.103b -- entering as a bestowed Aura.

#### 3a: Replace fizzle with creature fallback for bestow

**CR**: 702.103e, 608.3b -- "As a bestowed Aura spell begins resolving, if its target is illegal, it ceases to be bestowed and the effect making it an Aura spell ends. It continues resolving as a creature spell."

In `resolve_top_of_stack`, the existing fizzle check (line ~48-92) returns early with `SpellFizzled` when all targets are illegal. For bestow spells, we must intercept this:

**Insert BEFORE the fizzle return** (inside the `legal_count == 0` block, approximately line 53):

```rust
if legal_count == 0 {
    // CR 702.103e / 608.3b: Bestowed Aura with illegal target
    // ceases to be bestowed and resolves as a creature spell.
    if stack_obj.was_bestowed {
        // Revert the source card's characteristics: remove Aura/enchant creature,
        // add Creature type back.
        if let Some(source_obj) = state.objects.get_mut(&source_object) {
            source_obj.characteristics.subtypes.remove(&SubType("Aura".to_string()));
            source_obj.characteristics.keywords.remove(&KeywordAbility::Enchant(EnchantTarget::Creature));
            source_obj.characteristics.card_types.insert(CardType::Creature);
        }
        // Fall through to normal permanent resolution (don't return early).
        // The spell resolves as a creature entering the battlefield.
        // Clear targets so the Aura attachment code doesn't fire.
        // We use a mutable stack_obj shadow for this.
        let mut stack_obj = stack_obj;
        stack_obj.targets.clear();
        stack_obj.was_bestowed = false;
        // Continue to permanent resolution below...
        // (Need to restructure: instead of returning, fall through.)
    } else {
        // Normal fizzle path (existing code)...
    }
}
```

**IMPORTANT -- structural consideration**: The current fizzle path uses `return Ok(events)`. For bestow, we need to NOT return and instead fall through to permanent resolution. This may require restructuring the fizzle block into a `let should_fizzle = ...` pattern that allows bestow to skip the return. The runner should restructure as:

```rust
let bestow_fallback = if !targets.is_empty() {
    let legal_count = targets.iter().filter(|t| is_target_legal(state, t)).count();
    if legal_count == 0 {
        if stack_obj.was_bestowed {
            // CR 702.103e: revert to creature
            // ... (revert characteristics on source_object)
            true // signal: we are doing bestow fallback
        } else {
            // Normal fizzle -- return early
            // ... (existing fizzle code)
            return Ok(events);
        }
    } else {
        false
    }
} else {
    false
};
```

Then later, skip the Aura attachment block if `bestow_fallback` is true.

#### 3b: Transfer is_bestowed to the permanent

**File**: `crates/engine/src/rules/resolution.rs`
**Action**: In the permanent resolution path (~line 187-191), after setting `kicker_times_paid` and `was_evoked`, also set `is_bestowed`:

```rust
if let Some(obj) = state.objects.get_mut(&new_id) {
    obj.controller = controller;
    obj.kicker_times_paid = stack_obj.kicker_times_paid;
    obj.was_evoked = stack_obj.was_evoked;
    obj.is_bestowed = stack_obj.was_bestowed && !bestow_fallback;
}
```

The `&& !bestow_fallback` is important: if the bestow target was illegal and we fell back to creature mode, `is_bestowed` must be false on the permanent.

### Step 4: Rule Enforcement -- SBA Exception

**File**: `crates/engine/src/rules/sba.rs`
**Action**: Modify `check_aura_sbas` to implement CR 702.103f.
**CR**: 702.103f -- "If a bestowed Aura becomes unattached, it ceases to be bestowed."

Currently, `check_aura_sbas` (line ~625) finds Auras with illegal/missing targets and moves them to the graveyard. For bestowed Auras, instead of moving to graveyard:

1. Set `is_bestowed = false` on the permanent.
2. Remove `attached_to` and clear from target's `attachments`.
3. Revert characteristics: remove Aura subtype, remove `Enchant(Creature)` keyword, add `Creature` card type.
4. Do NOT move to graveyard.

**Implementation**: Split the `illegal_auras` processing into two paths:

```rust
// Separate bestowed Auras from normal Auras
let (bestowed_auras, normal_auras): (Vec<ObjectId>, Vec<ObjectId>) =
    illegal_auras.into_iter().partition(|id| {
        state.objects.get(id)
            .map(|obj| obj.is_bestowed)
            .unwrap_or(false)
    });

// CR 702.103f: Bestowed Auras revert to creatures instead of going to graveyard
for aura_id in bestowed_auras {
    // Remove attachment links
    if let Some(obj) = state.objects.get(&aura_id) {
        if let Some(target_id) = obj.attached_to {
            if let Some(target) = state.objects.get_mut(&target_id) {
                target.attachments.retain(|id| *id != aura_id);
            }
        }
    }
    if let Some(obj) = state.objects.get_mut(&aura_id) {
        obj.attached_to = None;
        obj.is_bestowed = false;
        // Revert types: remove Aura, remove enchant creature, add Creature
        obj.characteristics.subtypes.remove(&SubType("Aura".to_string()));
        obj.characteristics.keywords.remove(&KeywordAbility::Enchant(EnchantTarget::Creature));
        obj.characteristics.card_types.insert(CardType::Creature);
    }
    events.push(GameEvent::BestowReverted {
        object_id: aura_id,
    });
}

// Normal Auras go to graveyard (existing code)
for id in normal_auras {
    // ... existing AuraFellOff code ...
}
```

**Note**: A new `GameEvent::BestowReverted { object_id: ObjectId }` event variant needs to be added to `crates/engine/src/rules/events.rs`. This event informs the UI/replay system that a bestowed permanent has reverted to creature form.

### Step 5: New GameEvent Variant

**File**: `crates/engine/src/rules/events.rs`
**Action**: Add `BestowReverted { object_id: ObjectId }` variant to `GameEvent` enum.
**Pattern**: Follow `AuraFellOff` event structure.

```rust
/// CR 702.103f: A bestowed Aura became unattached and reverted to an
/// enchantment creature. Unlike normal Auras (AuraFellOff), the permanent
/// stays on the battlefield.
BestowReverted {
    object_id: ObjectId,
},
```

**Also update**: The `GameEvent` match in `view_model.rs` (replay viewer) to handle `BestowReverted` if the match is exhaustive.

### Step 6: Trigger Wiring

**Not applicable**. Bestow has no triggered abilities. The only special behavior is:
- Alternative cost at cast time (Step 2)
- Type transformation on stack (Step 2c)
- Fallback to creature on illegal target (Step 3a)
- SBA exception for unattach (Step 4)

All of these are replacement/modification behaviors, not triggers.

### Step 7: Unit Tests

**File**: `crates/engine/tests/bestow.rs`
**Tests to write**:

1. **`test_bestow_cast_as_aura_basic`** -- CR 702.103a/b: Cast Boon Satyr-alike for bestow cost {3}{G}{G} targeting a creature. Verify: spell on stack has Aura type + enchant creature, creature type removed; target creature required. After resolution: permanent on battlefield with `is_bestowed = true`, attached to target creature, Aura subtype present, no Creature type.

2. **`test_bestow_cast_normally_as_creature`** -- CR 702.103: Cast the same card for its normal mana cost {1}{G}{G} without targets. Verify: spell is an enchantment creature on the stack, no Aura subtype. After resolution: enchantment creature on battlefield, `is_bestowed = false`, not attached to anything.

3. **`test_bestow_target_illegal_at_resolution_becomes_creature`** -- CR 702.103e / 608.3b: Cast bestowed targeting a creature. Before resolution, the target creature leaves the battlefield (simulated zone move). At resolution, spell does NOT fizzle; instead reverts to creature and enters battlefield as enchantment creature with `is_bestowed = false`.

4. **`test_bestow_unattach_reverts_to_creature`** -- CR 702.103f: Cast bestowed, resolve, creature enters as bestowed Aura. Then the enchanted creature leaves the battlefield. SBA check: bestowed Aura does NOT go to graveyard (exception to 704.5m). Instead, `is_bestowed` becomes false, permanent stays on battlefield as enchantment creature.

5. **`test_bestow_alternative_cost_pays_bestow_cost`** -- CR 702.103a / 118.9: Verify the bestow cost is deducted (not the printed mana cost). Give exactly the bestow cost in mana pool; should succeed. Mana value on stack is still the printed cost.

6. **`test_bestow_cannot_combine_with_flashback`** -- CR 118.9a: Attempt to combine bestow with flashback. Should return error mentioning alternative cost conflict.

7. **`test_bestow_cannot_combine_with_evoke`** -- CR 118.9a: Attempt to combine bestow with evoke. Should return error.

8. **`test_bestow_non_bestow_spell_rejected`** -- Engine validation: Setting `cast_with_bestow: true` on a non-bestow card should return error.

9. **`test_bestow_enters_without_casting_is_creature`** -- Card ruling: If a permanent with bestow enters the battlefield without being cast (e.g., moved from graveyard to battlefield directly), it enters as an enchantment creature, not an Aura.

10. **`test_bestow_grants_bonus_to_enchanted_creature`** -- Integration: Cast bestowed with a static continuous effect (e.g., "Enchanted creature gets +4/+2"). Verify the enchanted creature's P/T includes the bonus via the layer system.

**Pattern**: Follow `crates/engine/tests/evoke.rs` for structure (helper functions, pass_all, find_object_in_zone). Follow `crates/engine/tests/enchant.rs` for Aura-specific patterns.

**Mock card definitions for tests**:
- `mock_bestow_creature()`: Enchantment Creature {1}{G}{G}, Bestow {3}{G}{G}, 4/2. Static ability: "Enchanted creature gets +4/+2." (Boon Satyr analog)
- A standard creature for targeting ("Test Bear", 2/2)

### Step 8: Card Definition (later phase)

**Suggested card**: Boon Satyr
- Oracle text: "Flash\nBestow {3}{G}{G}\nEnchanted creature gets +4/+2."
- Enchantment Creature -- Satyr, 4/2, {1}{G}{G}
- Good test card: has Flash for additional coverage, simple static bonus

**Card lookup**: use `card-definition-author` agent

### Step 9: Game Script (later phase)

**Suggested scenario**: "Boon Satyr bestowed on a creature, then the creature dies. Boon Satyr reverts to a 4/2 creature."
**Subsystem directory**: `test-data/generated-scripts/stack/`

## Interactions to Watch

1. **Aura targeting validation at cast time**: The bestow type transformation (CR 702.103b) must happen BEFORE the existing Aura target validation code in `casting.rs`. If the transformation happens after, the spell won't be recognized as an Aura and won't require/accept a target.

2. **Layer system**: While bestowed, the permanent is an Aura enchantment (not a creature). Its static continuous effects should use `EffectFilter::AttachedCreature` to apply to the enchanted creature. When it reverts, these effects stop applying (source is no longer attached). The layer system should handle this naturally through `attached_to` being cleared.

3. **SBA ordering**: The bestow SBA exception must be checked WITHIN `check_aura_sbas`, not as a separate SBA. The existing function identifies illegal Auras; we intercept bestowed ones before they reach the "move to graveyard" path.

4. **Object identity (CR 400.7)**: When the enchanted creature dies and the bestowed Aura reverts, the Aura keeps its ObjectId (it didn't change zones). This is correct -- it stayed on the battlefield the entire time.

5. **`StackObject` literal sites**: Adding `was_bestowed` to `StackObject` will cause compiler errors at EVERY site that constructs a `StackObject`. All must be updated to include `was_bestowed: false` (or `casting_with_bestow` for the main cast path). Grep for `StackObject {` to find all sites. Expected sites:
   - `casting.rs` (3-4): main cast, storm trigger, cascade trigger
   - `abilities.rs` (3-4): flush_pending_triggers, activated ability handler, crew handler

6. **`Command::CastSpell` literal sites**: Adding `cast_with_bestow` will cause compiler errors at all CastSpell construction sites. This is a much larger surface area -- every test file that constructs a CastSpell command. The runner must update all of them with `cast_with_bestow: false`. Use `Grep pattern="Command::CastSpell"` to enumerate.

7. **Characteristics on the stack vs. on the battlefield**: The type transformation (Creature -> Aura) happens to the actual `GameObject` on the stack. When the spell resolves and the permanent enters the battlefield, `move_object_to_zone` creates a new object. The `enrich_spec_from_def` or equivalent re-population may reset the characteristics. The runner must verify that the bestow type modifications persist through resolution -- if `enrich_spec_from_def` is called on the new battlefield object, it may overwrite the bestowed characteristics with the card's printed types. If this happens, the resolution code must re-apply the bestow transformation after enrichment, or skip enrichment for bestowed spells.

8. **`enrich_spec_from_def` concern**: Check whether resolution calls `enrich_spec_from_def` on the newly created battlefield permanent. If it does, it will restore the printed types (Enchantment Creature) and undo the bestow transformation. The fix: after enrichment, re-apply the bestow type change if `was_bestowed`.
