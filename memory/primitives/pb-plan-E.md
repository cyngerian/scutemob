# Primitive Batch Plan: PB-E -- Mana Doubling

**Generated**: 2026-04-08
**Primitive**: Mana trigger interception system (triggered mana abilities + mana production replacement effects)
**CR Rules**: 605.1b, 605.4, 605.4a, 106.12, 106.12a, 106.12b, 106.6a
**Cards affected**: 9 (6 existing fixes + 3 new)
**Dependencies**: None (all prerequisite types exist)
**Deferred items from prior PBs**: A-38/A-42 "mana-doubling" blocked category

## Primitive Specification

Two distinct engine capabilities are needed, corresponding to two different CR mechanisms:

### Pattern 1: Triggered Mana Abilities (CR 605.1b, 605.4, 605.4a)

Cards like Mirari's Wake, Crypt Ghast, Wild Growth, Leyline of Abundance, Badgermole Cub, Zendikar Resurgent have "Whenever you tap a [type] for mana, add [mana]" abilities. These are **triggered mana abilities** per CR 605.1b: they trigger from the activation of an activated mana ability, they don't require a target, and they could add mana. Per CR 605.4a, they resolve immediately after the mana ability that triggered them, without waiting for priority.

**EXCEPTION**: Forbidden Orchard's "Whenever you tap this land for mana, target opponent creates a 1/1 Spirit" is NOT a mana ability because it targets (CR 605.5a). However, it still uses the same trigger condition ("tapped for mana"). Per CR 605.5a, it follows normal triggered ability rules (goes on the stack, can be responded to).

Implementation approach: After `handle_tap_for_mana` produces mana, scan the battlefield for permanents with mana-trigger abilities and immediately resolve qualifying triggered mana abilities (adding extra mana inline). For non-mana triggers like Forbidden Orchard (which target), queue them as normal pending triggers instead.

### Pattern 2: Mana Production Replacement Effects (CR 106.12b, 106.6a)

Cards like Nyxbloom Ancient ("produces three times as much") and Mana Reflection ("produces twice as much") are **replacement effects** that modify the mana production event while a tap-mana ability is resolving. Per CR 106.12b, they apply "if a permanent is tapped for mana" and modify the production.

Implementation approach: Add new `ReplacementTrigger::ManaWouldBeProduced` and `ReplacementModification::DoubleMana` / `TripleMana` variants. Apply these replacement effects in `handle_tap_for_mana` BEFORE adding mana to the pool, multiplying the `produces` amounts.

Key rulings from Nyxbloom Ancient:
- "If an ability triggers 'whenever you tap' something for mana and produces mana, that triggered mana ability won't be affected by Nyxbloom Ancient." -- Pattern 2 only affects the original mana ability, not Pattern 1 triggers.
- Multiple Nyxbloom Ancients stack multiplicatively (two = 9x, per ruling).
- Multiple Mana Reflections stack multiplicatively (two = 4x, per ruling).
- "You're 'tapping a permanent for mana' only if you're activating a mana ability that includes {T} in its cost." -- Only applies to `requires_tap: true` mana abilities.

## CR Rule Text

### CR 605.1b
A triggered ability is a mana ability if it meets all of the following criteria: it doesn't require a target (see rule 115.6), it triggers from the activation or resolution of an activated mana ability (see rule 605.1a) or from mana being added to a player's mana pool, and it could add mana to a player's mana pool when it resolves.

### CR 605.4
Triggered mana abilities follow all the rules for other triggered abilities (see rule 603, "Handling Triggered Abilities"), with the following exception:

### CR 605.4a
A triggered mana ability doesn't go on the stack, so it can't be targeted, countered, or otherwise responded to. Rather, it resolves immediately after the mana ability that triggered it, without waiting for priority.

### CR 605.5a
An ability with a target is not a mana ability, even if it could put mana into a player's mana pool when it resolves. The same is true for a triggered ability that could produce mana but triggers from an event other than activating a mana ability, or a triggered ability that triggers from activating a mana ability but couldn't produce mana. These follow the normal rules for activated or triggered abilities, as appropriate.

### CR 106.12
To "tap [a permanent] for mana" is to activate a mana ability of that permanent that includes the {T} symbol in its activation cost. See rule 605, "Mana Abilities."

### CR 106.12a
An ability that triggers whenever a permanent "is tapped for mana" or is tapped for mana of a specified type triggers whenever such a mana ability resolves and produces mana or the specified type of mana.

### CR 106.12b
A replacement effect that applies if a permanent "is tapped for mana" or tapped for mana of a specific type and/or amount modifies the mana production event while such an ability is resolving and producing mana or the specified type and/or amount of mana.

### CR 106.6a
Some replacement effects increase the amount of mana produced by a spell or ability. In these cases, any restrictions or additional effects created by the spell or ability will apply to all mana produced. If the spell or ability creates a delayed triggered ability that triggers when the mana is spent, a separate delayed triggered ability is created for each mana produced. If the spell or ability creates a continuous effect or replacement effect if the mana is spent, a separate effect is created once for each mana produced.

## Engine Changes

### Change 1: Add `ManaAdded` source tracking to `GameEvent::ManaAdded`

**File**: `crates/engine/src/rules/events.rs`
**Action**: Add `source: Option<ObjectId>` field to `GameEvent::ManaAdded`. This tracks which permanent produced the mana, needed to determine what type of mana was produced (for Mirari's Wake "one mana of any type that land produced") and to scope triggers to the correct permanent.
**Line**: ~106

Before:
```rust
ManaAdded {
    player: PlayerId,
    color: ManaColor,
    amount: u32,
},
```

After:
```rust
ManaAdded {
    player: PlayerId,
    color: ManaColor,
    amount: u32,
    /// The permanent that produced this mana (if from a mana ability).
    /// None for mana from spell effects (Effect::AddMana, etc.).
    source: Option<ObjectId>,
},
```

### Change 2: Update all `ManaAdded` emission sites with `source` field

**File**: `crates/engine/src/rules/mana.rs`
**Action**: Pass `source` ObjectId in all `ManaAdded` events emitted from `handle_tap_for_mana`.
**Line**: ~197, ~206

**File**: `crates/engine/src/effects/mod.rs`
**Action**: Pass `source: None` in all `ManaAdded` events emitted from `Effect::AddMana*` variants (these are spell effects, not tap-mana abilities).
**Lines**: ~1465, ~1480, ~1499, ~1505, ~1523, ~1554, ~1573, ~1590

### Change 3: Add `ManaSourceFilter` enum for trigger conditions

**File**: `crates/engine/src/cards/card_definition.rs`
**Action**: Add a new enum near `TriggerCondition` to describe which permanent types trigger the mana ability.

```rust
/// Filter for "whenever you tap a [type] for mana" trigger conditions.
/// CR 106.12a: triggers whenever such a mana ability resolves and produces mana.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ManaSourceFilter {
    /// Any land you control (Mirari's Wake, Zendikar Resurgent)
    Land,
    /// A land with a specific subtype (Crypt Ghast: "a Swamp")
    LandSubtype(SubType),
    /// Any creature you control (Leyline of Abundance, Badgermole Cub)
    Creature,
    /// Any permanent you control (for future use)
    AnyPermanent,
    /// The enchanted land (Wild Growth: "enchanted land")
    EnchantedLand,
    /// This specific permanent (Forbidden Orchard: "this land")
    This,
}
```

### Change 4: Add `TriggerCondition::WhenTappedForMana` variant

**File**: `crates/engine/src/cards/card_definition.rs`
**Action**: Add new variant to `TriggerCondition` enum (discriminant 41).

```rust
/// CR 605.1b / CR 106.12a: "Whenever [you tap / a permanent is tapped] for mana"
///
/// Fires when a mana ability with {T} in its cost resolves and produces mana.
/// `source_filter` determines which permanents match (land, creature, Swamp, etc.).
/// If the trigger's effect only produces mana and doesn't target, it's a triggered
/// mana ability (CR 605.1b) that resolves immediately (CR 605.4a). If it targets
/// (e.g. Forbidden Orchard), it's a normal triggered ability (CR 605.5a).
WhenTappedForMana {
    source_filter: ManaSourceFilter,
},
```

### Change 5: Add `ManaMultiplierFilter` and `ReplacementTrigger::ManaWouldBeProduced` + `ReplacementModification::MultiplyMana`

**File**: `crates/engine/src/state/replacement_effect.rs`
**Action**: Add new trigger variant and modification variant for mana multiplication.

In `ReplacementTrigger` (after `WouldProliferate`, discriminant 11):
```rust
/// CR 106.12b: A replacement effect that applies when a permanent is tapped for mana.
/// "If you tap a permanent for mana, it produces [N] times as much of that mana instead."
ManaWouldBeProduced {
    /// Which permanents this applies to. For Nyxbloom/Mana Reflection, this is
    /// AnyPermanent (controlled by the specified player).
    controller: PlayerId,
},
```

In `ReplacementModification` (discriminant 19):
```rust
/// CR 106.12b / CR 106.6a: Multiply the mana produced by a tap-mana ability.
/// `multiplier` is the factor (2 for Mana Reflection, 3 for Nyxbloom Ancient).
/// Multiple instances stack multiplicatively (CR 106.6a + Nyxbloom ruling).
MultiplyMana(u32),
```

### Change 6: Apply mana replacement effects in `handle_tap_for_mana`

**File**: `crates/engine/src/rules/mana.rs`
**Action**: After computing the base mana production (step 8) but BEFORE adding to the pool, scan `state.replacement_effects` for `ManaWouldBeProduced` replacements where the controller matches the player and the source has `requires_tap: true`. Multiply each `(color, amount)` pair by the accumulated multiplier. This must happen BEFORE ManaAdded events are emitted.

**CR**: CR 106.12b -- replacement modifies the mana production event while the ability is resolving.

Insert between the current step 7 (sacrifice) and step 8 (add mana):

```rust
// 7b. Apply mana-production replacement effects (CR 106.12b).
// Only applies to mana abilities that include {T} in cost (CR 106.12).
let mana_multiplier = if ability.requires_tap {
    apply_mana_production_replacements(state, player, source)
} else {
    1u32
};
```

Then multiply all produced mana amounts by `mana_multiplier` before adding to pool.

Add helper function `apply_mana_production_replacements`:
```rust
/// CR 106.12b: Check for mana multiplication replacement effects.
/// Returns the total multiplier (product of all matching replacements).
/// Multiple Nyxbloom Ancients: 3 * 3 = 9x. Multiple Mana Reflections: 2 * 2 = 4x.
fn apply_mana_production_replacements(
    state: &GameState,
    player: PlayerId,
    source: ObjectId,
) -> u32 {
    let mut multiplier = 1u32;
    for effect in state.replacement_effects.iter() {
        if let ReplacementTrigger::ManaWouldBeProduced { controller } = &effect.trigger {
            if *controller == player {
                if let ReplacementModification::MultiplyMana(n) = &effect.modification {
                    multiplier *= n;
                }
            }
        }
    }
    multiplier
}
```

### Change 7: Fire mana-triggered abilities after `handle_tap_for_mana`

**File**: `crates/engine/src/rules/mana.rs`
**Action**: After adding mana to the pool (step 8), scan battlefield for permanents with `WhenTappedForMana` trigger conditions. For each matching trigger:
- If the trigger's effect is a pure mana-adding effect (no targets), it's a triggered mana ability (CR 605.1b) -- resolve it immediately inline (add mana directly).
- If the trigger has targets (e.g., Forbidden Orchard), it's NOT a mana ability (CR 605.5a) -- queue it as a `PendingTrigger` for normal stack resolution.

Add new function `fire_mana_triggered_abilities`:
```rust
/// CR 605.4a / CR 106.12a: Fire triggered abilities that trigger from tapping
/// a permanent for mana. Triggered mana abilities (no target, produces mana)
/// resolve immediately. Non-mana triggered abilities (target) go on the stack.
fn fire_mana_triggered_abilities(
    state: &mut GameState,
    player: PlayerId,
    source: ObjectId,
    mana_produced: &[(ManaColor, u32)],
    events: &mut Vec<GameEvent>,
)
```

This function should:
1. Collect all battlefield permanents (controlled by `player`) that have `AbilityDefinition::Triggered` with `trigger_condition: TriggerCondition::WhenTappedForMana { source_filter }`.
2. For each, check if `source_filter` matches `source` (e.g., is the source a land? a Swamp? the enchanted land?).
3. Also check Auras: for `ManaSourceFilter::EnchantedLand`, check if the trigger source (e.g., Wild Growth) is attached_to the tapped permanent.
4. For `ManaSourceFilter::This`, check if the trigger source IS the tapped permanent.
5. For matching triggers:
   - If `targets` is empty (triggered mana ability per CR 605.1b): resolve effect immediately. For effects like `AddManaAnyColor` ("one mana of any type that land produced"), use the `mana_produced` list to determine the type.
   - If `targets` is non-empty (NOT a mana ability per CR 605.5a): push a `PendingTrigger` to `state.pending_triggers`.

Call this at the end of `handle_tap_for_mana`, after mana is added and events emitted, but only when `ability.requires_tap` is true (CR 106.12: "tapping for mana" only applies to {T}-cost abilities).

### Change 8: Add `Effect::AddManaMatchingType` variant

**File**: `crates/engine/src/cards/card_definition.rs`
**Action**: Add a new Effect variant for "add one mana of any type that land produced". This is used by Mirari's Wake and Zendikar Resurgent where the added mana must match one of the types the source land produced.

```rust
/// CR 106.12a: "Add one mana of any type that land produced."
/// Deterministic fallback: adds one mana matching the first color in `mana_produced`.
/// Interactive choice deferred to M10 (when land produces multiple types).
AddManaMatchingType {
    player: PlayerTarget,
},
```

**File**: `crates/engine/src/effects/mod.rs`
**Action**: Add dispatch for `Effect::AddManaMatchingType`. In the mana-trigger context, the `EffectContext` will carry the produced mana colors. Pick the first color produced (deterministic fallback).

### Change 9: Hash updates

**File**: `crates/engine/src/state/hash.rs`
**Action**: Add hash arms for:
1. `TriggerCondition::WhenTappedForMana { source_filter }` -- discriminant 41
2. `ManaSourceFilter` variants (new impl block)
3. `ReplacementTrigger::ManaWouldBeProduced { controller }` -- discriminant 11
4. `ReplacementModification::MultiplyMana(n)` -- discriminant 19
5. `Effect::AddManaMatchingType` -- find current max Effect discriminant and add +1
6. `GameEvent::ManaAdded` -- add `source.hash_into(hasher)` to existing arm

### Change 10: Helpers.rs export

**File**: `crates/engine/src/cards/helpers.rs`
**Action**: Add `ManaSourceFilter` to the re-export list so card defs can use it.

### Change 11: Update `GameEvent::ManaAdded` in view_model and other display sites

**Files requiring update for `ManaAdded { source }` field addition**:
| File | Match expression | Action |
|------|-----------------|--------|
| `crates/engine/src/state/hash.rs` | `GameEvent::ManaAdded` | Add `source.hash_into(hasher)` |
| `tools/replay-viewer/src/view_model.rs` | `GameEvent::ManaAdded` (if matched) | Add `source` field |
| `crates/engine/src/testing/replay_harness.rs` | `GameEvent::ManaAdded` (if matched) | Add `source` field |

Check all sites that destructure `GameEvent::ManaAdded`:
```
Grep pattern="ManaAdded \{" -- all files
```

## Card Definition Fixes

### miraris_wake.rs
**Oracle text**: "Creatures you control get +1/+1. Whenever you tap a land for mana, add one mana of any type that land produced."
**Current state**: Has +1/+1 static, TODO for mana trigger
**Fix**: Add triggered ability:
```rust
AbilityDefinition::Triggered {
    trigger_condition: TriggerCondition::WhenTappedForMana {
        source_filter: ManaSourceFilter::Land,
    },
    effect: Effect::AddManaMatchingType {
        player: PlayerTarget::Controller,
    },
    targets: vec![],
    optional: false,
    once_per_turn: false,
},
```

### crypt_ghast.rs
**Oracle text**: "Extort. Whenever you tap a Swamp for mana, add an additional {B}."
**Current state**: Has Extort, TODO for mana trigger
**Fix**: Add triggered ability:
```rust
AbilityDefinition::Triggered {
    trigger_condition: TriggerCondition::WhenTappedForMana {
        source_filter: ManaSourceFilter::LandSubtype(SubType::Custom("Swamp".into())),
    },
    effect: Effect::AddMana {
        player: PlayerTarget::Controller,
        mana: ManaPool { black: 1, ..Default::default() },
    },
    targets: vec![],
    optional: false,
    once_per_turn: false,
},
```

### wild_growth.rs
**Oracle text**: "Enchant land. Whenever enchanted land is tapped for mana, its controller adds an additional {G}."
**Current state**: Has Enchant(Land), TODO for mana trigger
**Fix**: Add triggered ability:
```rust
AbilityDefinition::Triggered {
    trigger_condition: TriggerCondition::WhenTappedForMana {
        source_filter: ManaSourceFilter::EnchantedLand,
    },
    effect: Effect::AddMana {
        player: PlayerTarget::Controller,
        mana: ManaPool { green: 1, ..Default::default() },
    },
    targets: vec![],
    optional: false,
    once_per_turn: false,
},
```

### leyline_of_abundance.rs
**Oracle text**: "If this card is in your opening hand, you may begin the game with it on the battlefield. Whenever you tap a creature for mana, add an additional {G}. {6}{G}{G}: Put a +1/+1 counter on each creature you control."
**Current state**: Has activated ability, two TODOs (leyline opening hand + mana trigger)
**Fix**: Add mana trigger ability (leyline opening hand remains a separate TODO):
```rust
AbilityDefinition::Triggered {
    trigger_condition: TriggerCondition::WhenTappedForMana {
        source_filter: ManaSourceFilter::Creature,
    },
    effect: Effect::AddMana {
        player: PlayerTarget::Controller,
        mana: ManaPool { green: 1, ..Default::default() },
    },
    targets: vec![],
    optional: false,
    once_per_turn: false,
},
```

### badgermole_cub.rs
**Oracle text**: "When this creature enters, earthbend 1. Whenever you tap a creature for mana, add an additional {G}."
**Current state**: Two TODOs (earthbend + mana trigger)
**Fix**: Add mana trigger ability (earthbend remains a separate TODO):
```rust
AbilityDefinition::Triggered {
    trigger_condition: TriggerCondition::WhenTappedForMana {
        source_filter: ManaSourceFilter::Creature,
    },
    effect: Effect::AddMana {
        player: PlayerTarget::Controller,
        mana: ManaPool { green: 1, ..Default::default() },
    },
    targets: vec![],
    optional: false,
    once_per_turn: false,
},
```

### forbidden_orchard.rs
**Oracle text**: "{T}: Add one mana of any color. Whenever you tap this land for mana, target opponent creates a 1/1 colorless Spirit creature token."
**Current state**: Has mana ability, TODO for token trigger
**Fix**: Add triggered ability. NOTE: This targets, so it is NOT a mana ability (CR 605.5a). It goes on the stack normally. The trigger still fires from the mana ability activation though.
```rust
AbilityDefinition::Triggered {
    trigger_condition: TriggerCondition::WhenTappedForMana {
        source_filter: ManaSourceFilter::This,
    },
    effect: Effect::CreateToken {
        token: TokenSpec {
            name: "Spirit".to_string(),
            types: creature_types(&["Spirit"]),
            power: 1,
            toughness: 1,
            keywords: vec![],
            color: vec![],
            subtypes: vec![SubType::Custom("Spirit".into())],
        },
        count: EffectAmount::Fixed(1),
        controller: PlayerTarget::TargetOpponent,
    },
    targets: vec![EffectTarget::TargetOpponent],
    optional: false,
    once_per_turn: false,
},
```

## New Card Definitions

### nyxbloom_ancient.rs
**Oracle text**: "Trample. If you tap a permanent for mana, it produces three times as much of that mana instead."
**CardDefinition sketch**:
```rust
CardDefinition {
    card_id: cid("nyxbloom-ancient"),
    name: "Nyxbloom Ancient".to_string(),
    mana_cost: Some(ManaCost { generic: 4, green: 3, ..Default::default() }),
    types: creature_types(&["Elemental"]),
    supertypes: vec![],
    subtypes: vec![SubType::Custom("Elemental".into())],
    card_types_additional: vec![CardType::Enchantment],
    oracle_text: "Trample\nIf you tap a permanent for mana, it produces three times as much of that mana instead.".to_string(),
    power: Some(5),
    toughness: Some(5),
    abilities: vec![
        AbilityDefinition::Keyword(KeywordAbility::Trample),
        AbilityDefinition::Replacement {
            trigger: ReplacementTrigger::ManaWouldBeProduced {
                controller: PlayerId(0), // bound at registration time
            },
            modification: ReplacementModification::MultiplyMana(3),
            is_self: false,
            unless_condition: None,
        },
    ],
    ..Default::default()
}
```

### mana_reflection.rs
**Oracle text**: "If you tap a permanent for mana, it produces twice as much of that mana instead."
**CardDefinition sketch**:
```rust
CardDefinition {
    card_id: cid("mana-reflection"),
    name: "Mana Reflection".to_string(),
    mana_cost: Some(ManaCost { generic: 4, green: 2, ..Default::default() }),
    types: types(&[CardType::Enchantment]),
    oracle_text: "If you tap a permanent for mana, it produces twice as much of that mana instead.".to_string(),
    abilities: vec![
        AbilityDefinition::Replacement {
            trigger: ReplacementTrigger::ManaWouldBeProduced {
                controller: PlayerId(0),
            },
            modification: ReplacementModification::MultiplyMana(2),
            is_self: false,
            unless_condition: None,
        },
    ],
    ..Default::default()
}
```

### zendikar_resurgent.rs
**Oracle text**: "Whenever you tap a land for mana, add one mana of any type that land produced. Whenever you cast a creature spell, draw a card."
**CardDefinition sketch**:
```rust
CardDefinition {
    card_id: cid("zendikar-resurgent"),
    name: "Zendikar Resurgent".to_string(),
    mana_cost: Some(ManaCost { generic: 5, green: 2, ..Default::default() }),
    types: types(&[CardType::Enchantment]),
    oracle_text: "Whenever you tap a land for mana, add one mana of any type that land produced. (The types of mana are white, blue, black, red, green, and colorless.)\nWhenever you cast a creature spell, draw a card.".to_string(),
    abilities: vec![
        AbilityDefinition::Triggered {
            trigger_condition: TriggerCondition::WhenTappedForMana {
                source_filter: ManaSourceFilter::Land,
            },
            effect: Effect::AddManaMatchingType {
                player: PlayerTarget::Controller,
            },
            targets: vec![],
            optional: false,
            once_per_turn: false,
        },
        AbilityDefinition::Triggered {
            trigger_condition: TriggerCondition::WheneverYouCastSpell {
                during_opponent_turn: false,
                spell_type_filter: Some(vec![CardType::Creature]),
                noncreature_only: false,
                chosen_subtype_filter: false,
            },
            effect: Effect::DrawCards {
                player: PlayerTarget::Controller,
                count: EffectAmount::Fixed(1),
            },
            targets: vec![],
            optional: false,
            once_per_turn: false,
        },
    ],
    ..Default::default()
}
```

## Exhaustive Match Updates

### TriggerCondition::WhenTappedForMana (new variant, discriminant 41)

| File | Match expression | Line (approx) | Action |
|------|-----------------|------|--------|
| `crates/engine/src/state/hash.rs` | `TriggerCondition::` match | ~4399 | Add discriminant 41 + hash source_filter |
| `crates/engine/src/rules/abilities.rs` | trigger dispatch | various | Add matching arm for `WhenTappedForMana` (skip -- handled in mana.rs, not in check_triggers) |
| `crates/engine/src/testing/replay_harness.rs` | `TriggerCondition::` match (if exhaustive) | check | Add arm if exhaustive match exists |

NOTE: `TriggerCondition` is typically matched in `check_triggers` in `abilities.rs` via `fire_when_enters_triggered_effects` and event-based dispatch. The `WhenTappedForMana` variant will NOT be dispatched through `check_triggers` -- it's dispatched directly from `fire_mana_triggered_abilities` in `mana.rs`. However, if `abilities.rs` has an exhaustive match on `TriggerCondition` anywhere, a wildcard arm or explicit arm is needed. Check with grep.

### ManaSourceFilter (new enum)

| File | Action |
|------|--------|
| `crates/engine/src/state/hash.rs` | New `impl HashInto for ManaSourceFilter` block |
| `crates/engine/src/cards/helpers.rs` | Add to re-exports |

### ReplacementTrigger::ManaWouldBeProduced (new variant, discriminant 11)

| File | Match expression | Line (approx) | Action |
|------|-----------------|------|--------|
| `crates/engine/src/state/hash.rs` | `ReplacementTrigger::` match | ~1741 | Add discriminant 11 + hash controller |
| `crates/engine/src/rules/replacement.rs` | `ReplacementTrigger::` match (registration + application) | various | Add arm (skip in apply_replacement -- handled in mana.rs) |

### ReplacementModification::MultiplyMana (new variant, discriminant 19)

| File | Match expression | Line (approx) | Action |
|------|-----------------|------|--------|
| `crates/engine/src/state/hash.rs` | `ReplacementModification::` match | ~1799 | Add discriminant 19 + hash multiplier |
| `crates/engine/src/rules/replacement.rs` | `ReplacementModification::` match | various | Add arm (skip in apply_replacement_modification -- handled in mana.rs) |

### Effect::AddManaMatchingType (new variant)

| File | Match expression | Line (approx) | Action |
|------|-----------------|------|--------|
| `crates/engine/src/effects/mod.rs` | `Effect::` match (execute_effect) | near mana block | Add dispatch arm |
| `crates/engine/src/state/hash.rs` | `Effect::` match | find max disc | Add new discriminant |
| `tools/replay-viewer/src/view_model.rs` | `Effect::` match (if exhaustive) | check | Add arm |

### GameEvent::ManaAdded { source } field addition

All sites that destructure `GameEvent::ManaAdded { player, color, amount }` need `source` added:
| File | Line (approx) | Action |
|------|------|--------|
| `crates/engine/src/state/hash.rs` | ~3007 | Add `source.hash_into(hasher)` |
| `crates/engine/src/rules/mana.rs` | ~197, ~206 | Add `source: Some(source)` |
| `crates/engine/src/effects/mod.rs` | ~1465, ~1480, ~1499, ~1505, ~1523, ~1554, ~1573, ~1590 | Add `source: None` |

### Card registry (mod.rs in defs/)

**File**: `crates/engine/src/cards/defs/mod.rs`
**Action**: Add `mod nyxbloom_ancient;`, `mod mana_reflection;`, `mod zendikar_resurgent;` and register in `register_all`.

## Unit Tests

**File**: `crates/engine/tests/mana_triggers.rs` (new file)
**Tests to write**:
- `test_mana_trigger_land_adds_extra_mana` -- Mirari's Wake: tap Forest, get G + matching G. CR 605.4a, CR 106.12a.
- `test_mana_trigger_swamp_subtype_filter` -- Crypt Ghast: tap Swamp adds extra B, tap Forest does not. CR 106.12a.
- `test_mana_trigger_creature_filter` -- Leyline of Abundance: tap Llanowar Elves adds extra G, tap Forest does not. CR 605.1b.
- `test_mana_trigger_enchanted_land` -- Wild Growth on Forest: tap produces G + extra G. Tap a different Forest: no extra. CR 106.12a.
- `test_mana_trigger_this_permanent` -- Forbidden Orchard: tap produces mana + Spirit token trigger goes on stack (NOT immediate, because it targets). CR 605.5a.
- `test_mana_multiplier_double` -- Mana Reflection: tap Forest for G, get 2G. CR 106.12b.
- `test_mana_multiplier_triple` -- Nyxbloom Ancient: tap Forest for G, get 3G. CR 106.12b.
- `test_mana_multiplier_stacks_multiplicatively` -- Two Mana Reflections: tap Forest for G, get 4G. Per ruling: "effects are cumulative."
- `test_mana_multiplier_does_not_affect_triggered_mana` -- Nyxbloom Ancient + Mirari's Wake: Forest produces 3G (tripled), Wake trigger adds 1G (not tripled). Per Nyxbloom ruling: "If an ability triggers 'whenever you tap' something for mana and produces mana, that triggered mana ability won't be affected."
- `test_mana_trigger_only_fires_on_tap_abilities` -- Treasure token (sacrifice, not tap-only) does not trigger Mirari's Wake. CR 106.12: only {T}-cost mana abilities count.
- `test_zendikar_resurgent_creature_cast_draw` -- Cast creature spell, draw a card. Standard trigger test.

**Pattern**: Follow tests in `crates/engine/tests/card_def_fixes.rs` for card-integration patterns. Use `GameStateBuilder` + `CardDefinition` setup + `Command::TapForMana` + assert mana pool contents.

## Verification Checklist

- [ ] Engine primitive compiles (`cargo check`)
- [ ] All existing card def TODOs for this batch resolved (6 card fixes)
- [ ] New card defs authored (3: Nyxbloom Ancient, Mana Reflection, Zendikar Resurgent)
- [ ] New cards registered in `defs/mod.rs`
- [ ] Unit tests pass (`cargo test --all`)
- [ ] Clippy clean (`cargo clippy -- -D warnings`)
- [ ] Workspace builds (`cargo build --workspace`)
- [ ] No remaining mana-doubling TODOs in affected card defs
- [ ] Leyline of Abundance retains separate TODO for "opening hand" (different gap)
- [ ] Badgermole Cub retains separate TODO for "earthbend" (different gap)

## Risks & Edge Cases

- **ManaAdded source field is a breaking change to all destructuring sites.** Must grep ALL usages of `ManaAdded {` and add `source` field. Missing one causes compile error. The runner should `cargo check` after this change before proceeding.
- **Forbidden Orchard targets an opponent**: per CR 605.5a, its trigger is NOT a mana ability. The engine must distinguish between triggered mana abilities (resolve immediately per CR 605.4a) and normal triggered abilities that happen to trigger from mana production (go on the stack). The `targets` field on the `AbilityDefinition::Triggered` is the discriminator: if empty, it's a mana ability; if non-empty, it goes on the stack.
- **"Any type that land produced" deterministic fallback**: Mirari's Wake and Zendikar Resurgent say "one mana of any type that land produced." When the land produces multiple types (e.g., a dual land tapped for U via the first ability), the trigger adds one mana of that type. The engine currently lacks interactive color choice (deferred to M10), so deterministic fallback picks the first color in `mana_produced`. This is correct for basic lands (single color) but slightly wrong for dual lands that could produce multiple types -- acceptable for pre-alpha.
- **Wild Growth `EnchantedLand` filter**: Must check `source.attached_to` to find the enchanted permanent, then check if the tapped permanent matches. The trigger source is Wild Growth (the Aura), not the land itself.
- **Multiplier applied before triggers**: Per Nyxbloom ruling, mana multipliers affect the base mana production but NOT the mana added by triggered mana abilities. The order in `handle_tap_for_mana` must be: (1) apply multiplier replacement effects, (2) add multiplied mana to pool, (3) fire triggered mana abilities (which add their own unmultiplied mana).
- **Non-tap mana abilities (sacrifice)**: Treasure tokens have `sacrifice_self: true, requires_tap: true` but some mana abilities don't require tap (future). CR 106.12 only covers "tap for mana" so non-tap mana abilities should NOT trigger `WhenTappedForMana` or apply `ManaWouldBeProduced` replacements. The guard `if ability.requires_tap` handles this.
- **Effect::AddMana in effects/mod.rs does NOT go through mana.rs**: Spell effects that add mana (e.g., Dark Ritual, Seething Song) do NOT count as "tapping a permanent for mana" and should NOT trigger mana-triggered abilities or apply mana multipliers. The `source: None` on their `ManaAdded` events distinguishes them.
- **EffectContext for AddManaMatchingType**: The inline mana trigger resolution in `fire_mana_triggered_abilities` needs to pass the produced mana colors to the effect execution so `AddManaMatchingType` can pick a matching color. Either use a field on `EffectContext` (e.g., `mana_produced: Option<Vec<(ManaColor, u32)>>`) or resolve it directly in the trigger handler without going through `execute_effect`.
