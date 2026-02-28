# Ability Plan: Riot

**Generated**: 2026-02-27
**CR**: 702.136
**Priority**: P3
**Similar abilities studied**: Evoke (ETB flag transfer pattern, `was_evoked`), Escape with counters (inline ETB counter placement in `resolution.rs:230-254`), self-ETB replacement effects (`replacement.rs:837-870`, `EntersWithCounters` / `EntersTapped`)

## CR Rule Text

702.136. Riot

702.136a Riot is a static ability. "Riot" means "You may have this permanent enter with an additional +1/+1 counter on it. If you don't, it gains haste."

702.136b If a permanent has multiple instances of riot, each works separately.

## Key Rulings (from Scryfall/Gatherer)

1. **Riot is a replacement effect** (CR 614.1c). Players can't respond to the choice of +1/+1 counter or haste, and they can't take actions while the creature is on the battlefield without one or the other. (Spider-Punk, 2025-09-19)

2. **Haste is gained indefinitely.** If you choose haste, the creature gains haste permanently -- it won't lose it as the turn ends or as another player gains control of it. (Zhur-Taa Goblin, 2019-01-25)

3. **Can't receive counter => gains haste.** If a creature entering the battlefield has riot but can't have a +1/+1 counter put onto it, it gains haste. (Multiple cards, 2019-01-25)

4. **Multiple instances each work separately** (CR 702.136b). If a creature enters the battlefield with two instances of riot, the controller may choose to have it get two +1/+1 counters, one +1/+1 counter and haste, or two instances of haste. Multiple instances of haste are redundant. (Rhythm of the Wild, 2019-01-25)

5. **Once entered, keeps the choice.** Once a creature with riot has entered the battlefield, it keeps its +1/+1 counter or haste even if it loses riot. (Rhythm of the Wild, 2019-01-25)

6. **Noncreature entering as creature gets riot; creature entering as noncreature doesn't.** (Rhythm of the Wild, 2019-01-25)

## Key Edge Cases

- **No player interaction during the choice.** This is a replacement effect, not a triggered ability. The choice is made as part of the ETB event being replaced, before priority is passed.
- **Haste is permanent, not until-end-of-turn.** The keyword is added to the permanent's keywords set directly, not via a continuous effect with duration.
- **Multiple Riot instances**: Each instance of Riot on the permanent generates a separate choice. The controller can choose independently for each (counter, haste, or mix).
- **Can't receive counter fallback**: If the creature can't receive +1/+1 counters (e.g., due to some static effect), it must gain haste. This is currently an edge case that can be deferred -- the engine doesn't yet model "can't have counters placed on it" restrictions.
- **Multiplayer**: Riot is controller's choice, not owner's. In a Commander game, the controller of the permanent makes the riot choice (relevant if someone gains control of a spell on the stack).

## Design Decision: Choice Mechanism

### Problem

Riot requires a player choice (counter vs haste) at ETB time, which is a replacement effect. The engine currently has no mechanism for player choices within replacement effects -- all existing ETB replacements (`EntersTapped`, `EntersWithCounters`) are deterministic.

### Options Considered

1. **New `ReplacementModification::RiotChoice`** -- add a Riot-specific variant to `ReplacementModification` and handle it in `emit_etb_modification`. This tightly couples the enum to a specific keyword.

2. **Automatic default choice** -- always choose +1/+1 counter (or haste), no player choice. Simple but incorrect.

3. **Inline processing in `apply_self_etb_from_definition`** with a new `ReplacementModification::EntersWithKeywordOrCounters` -- generic enough to reuse.

4. **Inline processing at the ETB site in `resolution.rs`** (like Escape's counter placement at lines 230-254) -- check the permanent's keywords for `Riot`, apply the default choice. This follows the established Escape pattern and avoids modifying the replacement effect infrastructure.

### Chosen Approach: Option 4 (Inline at ETB site)

**Rationale**: This matches the Escape pattern exactly. Escape's "enters with counters" is also a replacement effect per the CR, but the engine implements it as inline logic at the ETB site in `resolution.rs` (lines 230-254) rather than through the `ReplacementModification` system. Riot follows the same pattern:

- After the permanent enters the battlefield and status flags are transferred (line 228)
- Before self-ETB replacements from card definition are applied (line 329)
- Check if the permanent has `KeywordAbility::Riot` in its keywords
- For each instance of Riot: default to +1/+1 counter (deterministic for testing)
- Add the counter or haste keyword directly to the permanent

**Default choice for deterministic testing**: The engine will default to choosing +1/+1 counter. This is the safe default because:
- Tests are deterministic
- Counter is the more common choice in competitive play
- The choice can be overridden by a future `Command::ChooseRiot` when player-interactive choices are implemented

**Future enhancement** (not in this plan): Add a `Command::ChooseRiot` variant that allows the player to specify their choice. For now, Riot always chooses the counter. A TODO comment will note this simplification. When the engine adds generic "choose on ETB" infrastructure (CR 614.12a), it can be retrofitted.

**Note on haste permanence**: Per rulings, if haste is chosen, it is gained **indefinitely** -- not until end of turn. This means we insert `KeywordAbility::Haste` directly into `characteristics.keywords`, NOT via a continuous effect with duration. The keyword persists through zone changes only if the permanent stays on the battlefield.

## Current State (from ability-wip.md)

- [ ] Step 1: Enum variant
- [ ] Step 2: Rule enforcement
- [ ] Step 3: Trigger wiring (n/a -- replacement effect, no triggers)
- [ ] Step 4: Unit tests
- [ ] Step 5: Card definition
- [ ] Step 6: Game script
- [ ] Step 7: Coverage doc update

## Implementation Steps

### Step 1: Enum Variant

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/types.rs`
**Action**: Add `KeywordAbility::Riot` variant after `Dethrone` (line ~378).
**Doc comment**:
```rust
/// CR 702.136: Riot -- "You may have this permanent enter with an additional
/// +1/+1 counter on it. If you don't, it gains haste."
///
/// Replacement effect (CR 614.1c). Applied inline at the ETB site in
/// `resolution.rs` (like Escape's counter placement). Each instance
/// of Riot on a permanent generates a separate choice (CR 702.136b).
///
/// Default: always chooses +1/+1 counter for deterministic testing.
/// Future: player-interactive choice via a new Command variant.
Riot,
```

**Hash**: `/home/airbaggie/scutemob/crates/engine/src/state/hash.rs`
**Action**: Add `KeywordAbility::Riot => 56u8.hash_into(hasher),` after `Dethrone` (line ~417). Use discriminant 56 (next available after Dethrone=55).

**Match arms**: After adding the variant, grep for all exhaustive matches on `KeywordAbility` and add the `Riot` arm:

1. `/home/airbaggie/scutemob/crates/engine/src/state/hash.rs` -- `hash_into` impl (line ~417)
2. `/home/airbaggie/scutemob/tools/replay-viewer/src/view_model.rs` -- `format_keyword` function (line ~637). Add: `KeywordAbility::Riot => "Riot".to_string(),`

### Step 2: Rule Enforcement (Inline at ETB site)

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/resolution.rs`
**Action**: Add Riot processing in the permanent ETB path, immediately after the Escape counter placement block (after line ~254) and before the bestow type transformation (line ~260).

**Pattern**: Follow the Escape counter placement pattern at lines 230-254.

**Code to add** (after the Escape block, before the bestow block):

```rust
// CR 702.136a: Riot -- "You may have this permanent enter with an
// additional +1/+1 counter on it. If you don't, it gains haste."
// CR 702.136b: Multiple instances each work separately.
// CR 614.1c: This is a replacement effect -- applied inline before
// PermanentEnteredBattlefield is emitted, not a triggered ability.
//
// Implementation: For each instance of Riot on the permanent,
// default to choosing +1/+1 counter (deterministic testing).
// TODO: Add Command::ChooseRiot for player-interactive choice.
{
    let riot_count = state
        .objects
        .get(&new_id)
        .map(|obj| {
            obj.characteristics
                .keywords
                .iter()
                .filter(|kw| matches!(kw, KeywordAbility::Riot))
                .count()
        })
        .unwrap_or(0);

    for _ in 0..riot_count {
        // Default choice: +1/+1 counter (CR 702.136a).
        // Each Riot instance adds one +1/+1 counter.
        if let Some(obj) = state.objects.get_mut(&new_id) {
            let current = obj
                .counters
                .get(&CounterType::PlusOnePlusOne)
                .copied()
                .unwrap_or(0);
            obj.counters = obj
                .counters
                .update(CounterType::PlusOnePlusOne, current + 1);
        }
        events.push(GameEvent::CounterAdded {
            object_id: new_id,
            counter: CounterType::PlusOnePlusOne,
            count: 1,
        });
    }
}
```

**CR**: 702.136a -- "You may have this permanent enter with an additional +1/+1 counter on it. If you don't, it gains haste."
**CR**: 702.136b -- "If a permanent has multiple instances of riot, each works separately."
**CR**: 614.1c -- This is a replacement effect.

**Important**: Riot uses `OrdSet<KeywordAbility>`, and `Riot` has no parameters, so multiple instances of `KeywordAbility::Riot` will be deduplicated in the `OrdSet`. This means we cannot count multiple Riot instances via keywords alone.

**REVISED APPROACH**: Since `OrdSet` deduplicates, we need to count Riot instances from the **card definition** (not from the permanent's keywords). This is the same approach used for Afterlife(N) and Annihilator(N), but Riot doesn't carry a count parameter. The count of Riot instances comes from:
- The card's own definition (e.g., a card printed with Riot has 1 instance)
- External sources granting Riot (e.g., Rhythm of the Wild)

For the MVP, we count from the card definition:

```rust
// CR 702.136a/b: Count riot instances from card definition.
// OrdSet deduplicates KeywordAbility::Riot, so we count from the def.
let riot_count = card_id
    .as_ref()
    .and_then(|cid| registry.get(cid.clone()))
    .map(|def| {
        def.abilities
            .iter()
            .filter(|a| matches!(a, AbilityDefinition::Keyword(KeywordAbility::Riot)))
            .count()
    })
    .unwrap_or(0);
```

**Note on OrdSet deduplication**: `KeywordAbility::Riot` (a unit variant with `Eq` + `Ord`) will be deduplicated in the `OrdSet`. A creature with two Riot instances from different sources still shows only one `Riot` in `keywords`. The count must come from the definition. For the "Rhythm of the Wild grants Riot" case, the engine would need to count continuous effects granting Riot -- this is a future concern. For MVP, the card-definition count is sufficient.

**Second ETB site**: `/home/airbaggie/scutemob/crates/engine/src/rules/lands.rs`
Riot does NOT need to be added to `lands.rs`. Riot only applies to creatures. Lands cannot be creatures at the time they are played via `PlayLand` (they enter as lands, not as creatures). If a land somehow gains Riot and enters as a creature, it would enter via `resolution.rs` (cast as a spell), not `lands.rs`. This is consistent with the Rhythm of the Wild ruling: "A noncreature card that happens to be entering the battlefield as a creature will have riot."

### Step 3: Trigger Wiring

**N/A** -- Riot is a replacement effect (CR 614.1c), not a triggered ability. No trigger wiring needed. The choice is made inline during ETB processing, not on the stack.

### Step 4: Unit Tests

**File**: `/home/airbaggie/scutemob/crates/engine/tests/riot.rs`
**Tests to write**:

1. **`test_riot_enters_with_counter`** -- CR 702.136a
   - Setup: Create a creature with `KeywordAbility::Riot` on the battlefield via spell resolution.
   - Assert: The creature has 1 +1/+1 counter after entering.
   - Assert: A `CounterAdded` event was emitted.
   - Pattern: Follow Escape counter tests; use `GameStateBuilder`, `CardRegistry` with a test card definition, `CastSpell` command.

2. **`test_riot_creature_has_correct_stats`** -- CR 702.136a
   - Setup: Cast a 2/2 creature with Riot.
   - Assert: After entering, the creature has 3/3 stats (base 2/2 + one +1/+1 counter).
   - Uses `calculate_characteristics` to verify P/T.

3. **`test_riot_keyword_present_on_permanent`** -- CR 702.136a
   - Setup: Cast a creature with Riot.
   - Assert: The permanent has `KeywordAbility::Riot` in its keywords set.

4. **`test_riot_multiple_instances`** -- CR 702.136b
   - Setup: Create a card definition with TWO `AbilityDefinition::Keyword(KeywordAbility::Riot)` entries.
   - Assert: The creature enters with 2 +1/+1 counters (one per Riot instance).
   - Assert: Two `CounterAdded` events were emitted.

5. **`test_riot_does_not_affect_noncreatures`** -- Ruling: "creature card entering as noncreature won't have riot"
   - Setup: Cast a noncreature permanent spell whose definition has Riot.
   - Assert: No counters added, no haste. (This test verifies that Riot only fires for creatures, though per CR 702.136a it applies to "this permanent" -- if a permanent has riot it enters with the choice regardless of type. This test may be deferred or refined based on whether noncreatures can have riot in the card pool.)

**Pattern**: Follow test structure in `/home/airbaggie/scutemob/crates/engine/tests/evoke.rs` and `/home/airbaggie/scutemob/crates/engine/tests/persist_undying.rs` for ETB-related keyword tests.

**Test helper card**: Define an inline test card named "Riot Test Creature" with:
```rust
CardDefinition {
    card_id: cid("riot-test-creature"),
    name: "Riot Test Creature".to_string(),
    mana_cost: Some(ManaCost { generic: 1, red: 1, ..Default::default() }),
    types: creature_types(&["Goblin"]),
    oracle_text: "Riot".to_string(),
    power: Some(2),
    toughness: Some(2),
    abilities: vec![
        AbilityDefinition::Keyword(KeywordAbility::Riot),
    ],
}
```

And for the multiple-instances test, "Double Riot Creature":
```rust
CardDefinition {
    card_id: cid("double-riot-creature"),
    name: "Double Riot Creature".to_string(),
    mana_cost: Some(ManaCost { generic: 2, red: 1, green: 1, ..Default::default() }),
    types: creature_types(&["Beast"]),
    oracle_text: "Riot, Riot".to_string(),
    power: Some(3),
    toughness: Some(3),
    abilities: vec![
        AbilityDefinition::Keyword(KeywordAbility::Riot),
        AbilityDefinition::Keyword(KeywordAbility::Riot),
    ],
}
```

### Step 5: Card Definition (later phase)

**Suggested card**: Zhur-Taa Goblin
- **Name**: Zhur-Taa Goblin
- **Mana Cost**: {R}{G}
- **Type**: Creature -- Goblin Berserker
- **Oracle Text**: "Riot (This creature enters with your choice of a +1/+1 counter or haste.)"
- **P/T**: 2/2
- **Keywords**: Riot
- **Color Identity**: R, G

**Why this card**: Simplest possible Riot creature -- no other abilities, clean test case. Gruul Spellbreaker has Trample + conditional Hexproof which adds complexity.

**Card lookup**: Use `card-definition-author` agent with card name "Zhur-Taa Goblin".

### Step 6: Game Script (later phase)

**Suggested scenario**: "Riot creature enters with +1/+1 counter"
- Player 1 has mana available ({R}{G})
- Player 1 casts Zhur-Taa Goblin
- All players pass priority
- Zhur-Taa Goblin enters the battlefield with a +1/+1 counter (default Riot choice)
- Assertions: creature on battlefield with 1 +1/+1 counter, effective P/T is 3/3

**Subsystem directory**: `test-data/generated-scripts/replacement/`

## Interactions to Watch

1. **Riot + Humility**: If Humility is on the battlefield, it removes all abilities (Layer 6) including Riot. But Riot is a replacement effect that applies "as the permanent enters" -- before Humility can affect the permanent's abilities (CR 614.12: check characteristics as the permanent would exist on the battlefield). However, per CR 614.12, the check considers "continuous effects that already exist and would apply to the permanent" -- so Humility's ability removal WOULD apply to the entering permanent, meaning it would NOT have Riot. This is a complex interaction that can be deferred.

2. **Riot + Panharmonicon**: Panharmonicon doubles ETB triggers, not replacement effects. Riot is a replacement effect, so Panharmonicon does NOT double the Riot choice. Correct behavior falls out naturally from the implementation (no trigger dispatch involved).

3. **Riot + Doubling Season**: Doubling Season doubles +1/+1 counters placed on permanents. If the Riot choice is +1/+1 counter, Doubling Season would double it to 2. This interaction is correct IF the counter is placed through the standard counter-placement infrastructure. Since we use direct `obj.counters.update()`, Doubling Season won't apply. This is a known gap in the engine's counter infrastructure (not Riot-specific).

4. **Riot + "can't have counters" effects**: Per ruling, if the creature can't receive +1/+1 counters, it gains haste instead. The engine doesn't currently model "can't have counters" restrictions, so this edge case is naturally deferred.

5. **Multiple Riot sources (Rhythm of the Wild)**: When Rhythm of the Wild grants Riot to a creature that already has Riot, the creature has two instances. The `OrdSet` deduplication means we can't count instances from `keywords` -- we must count from definitions + continuous effects. For MVP, counting from the card definition is sufficient.

## Files Modified Summary

| File | Action |
|------|--------|
| `crates/engine/src/state/types.rs` | Add `KeywordAbility::Riot` variant |
| `crates/engine/src/state/hash.rs` | Add hash discriminant 56 for `Riot` |
| `crates/engine/src/rules/resolution.rs` | Add inline Riot processing at ETB site |
| `tools/replay-viewer/src/view_model.rs` | Add `Riot` arm to `format_keyword` |
| `crates/engine/tests/riot.rs` | New test file with 4-5 tests |
