# Primitive Batch Plan: PB-28 -- CDA / Count-Based P/T

**Generated**: 2026-03-25
**Primitive**: Dynamic CDA P/T evaluation in Layer 7a via `LayerModification::SetPtDynamic` and `AbilityDefinition::CdaPowerToughness`
**CR Rules**: 604.3, 604.3a, 613.4, 613.4a
**Cards affected**: ~12 existing fixes + 0 new (scope limited to true CDA cards with existing defs)
**Dependencies**: PB-7 (EffectAmount variants), PB-24 (Condition on statics)
**Deferred items from prior PBs**: None directly applicable

## Primitive Specification

Currently, `power: None` / `toughness: None` on a CardDefinition signals a `*/*` CDA creature,
but the engine has no mechanism to dynamically evaluate the CDA at layer-calculation time.
The existing `LayerModification::SetPtViaCda { power: i32, toughness: i32 }` takes **fixed**
values, meaning a caller must pre-compute the P/T. For CDA cards, the P/T changes every time
the game state changes (e.g., a land enters, a creature dies), so fixed values go stale.

This batch adds:

1. **`LayerModification::SetPtDynamic`** -- a new Layer 7a variant that stores `EffectAmount`
   values for power and toughness, evaluated at layer-calculation time against the current
   game state. This replaces the stale-value problem.

2. **`AbilityDefinition::CdaPowerToughness`** -- a new ability variant that registers a
   Layer 7a continuous effect with `is_cda: true` when the permanent enters the battlefield.
   Card defs use this to declare their CDA P/T formula.

3. **`resolve_cda_amount()`** -- a layer-system-accessible function that evaluates a subset
   of `EffectAmount` variants (those that don't require `EffectContext`) against the game
   state, using the source object's controller as the reference player.

CR 604.3 says CDAs function in all zones (hand, graveyard, exile, stack, outside the game).
For P/T CDAs this is relevant: a `*/*` creature in the graveyard should have its CDA-computed
P/T visible (e.g., for "return target creature card with power 2 or less"). The layer system
already processes all objects regardless of zone, so this works naturally.

CR 604.3a criterion (5) says CDAs must NOT "set the values of such characteristics only if
certain conditions are met." This means CDA P/T formulas are unconditional -- they always
apply. We enforce this by not having a `condition` field on the CDA ability.

## CR Rule Text

**CR 604.3**: Some static abilities are characteristic-defining abilities. A
characteristic-defining ability conveys information about an object's characteristics that
would normally be found elsewhere on that object (such as in its mana cost, type line, or
power/toughness box). Characteristic-defining abilities can add to or override information
found elsewhere on that object. Characteristic-defining abilities function in all zones.
They also function outside the game and before the game begins.

**CR 604.3a**: A static ability is a characteristic-defining ability if it meets the following
criteria: (1) It defines an object's colors, subtypes, power, or toughness; (2) it is printed
on the card it affects, it was granted to the token it affects by the effect that created the
token, or it was acquired by the object it affects as the result of a copy effect or
text-changing effect; (3) it does not directly affect the characteristics of any other objects;
(4) it is not an ability that an object grants to itself; and (5) it does not set the values
of such characteristics only if certain conditions are met.

**CR 613.4**: Within layer 7, apply effects in a series of sublayers in the order described
below. Within each sublayer, apply effects in timestamp order.

**CR 613.4a**: Layer 7a: Effects from characteristic-defining abilities that define power
and/or toughness are applied. See rule 604.3.

## Engine Changes

### Change 1: Add `LayerModification::SetPtDynamic` variant

**File**: `crates/engine/src/state/continuous_effect.rs`
**Action**: Add new variant after `SetPtViaCda` (line ~258):

```rust
/// Sets P/T via a CDA with dynamic evaluation (CR 613.4a).
///
/// Unlike `SetPtViaCda` (which takes pre-computed fixed values), this variant
/// stores `EffectAmount` values that are evaluated at layer-calculation time
/// against the current game state. Used by */* creatures whose P/T depends on
/// a count that changes as the game progresses.
///
/// The `EffectAmount` variants used here MUST NOT require `EffectContext`
/// (no XValue, no LastEffectCount, no LastDiceRoll). Valid variants:
/// Fixed, PermanentCount, CardCount, DevotionTo, CounterCount, PowerOf, ToughnessOf.
SetPtDynamic {
    power: EffectAmount,
    toughness: EffectAmount,
},
```

**Imports needed**: Add `use crate::cards::card_definition::EffectAmount;` to continuous_effect.rs (or use full path).

**Pattern**: Follow `SetPtViaCda` at line 258.

### Change 2: Hash the new variant

**File**: `crates/engine/src/state/hash.rs`
**Action**: Add hash arm for `SetPtDynamic` after `RemoveCardTypes` (discriminant 22):

```rust
LayerModification::SetPtDynamic { power, toughness } => {
    22u8.hash_into(hasher);
    power.hash_into(hasher);
    toughness.hash_into(hasher);
}
```

`EffectAmount` already has `HashInto` (line ~3844 in hash.rs). No new hash impl needed.

**Pattern**: Follow `SetPtViaCda` hash at line 1270.

### Change 3: Evaluate `SetPtDynamic` in `apply_layer_modification`

**File**: `crates/engine/src/rules/layers.rs`
**Action**:

a) Change `apply_layer_modification` signature to include `object_id: ObjectId`:

```rust
fn apply_layer_modification(
    state: &GameState,
    chars: &mut Characteristics,
    modification: &LayerModification,
    mana_value: u32,
    object_id: ObjectId,  // NEW: needed for CDA evaluation
) {
```

b) Update the call site at line ~355 to pass `object_id`:

```rust
apply_layer_modification(state, &mut chars, &effect.modification, mana_value, object_id);
```

c) Add match arm for `SetPtDynamic` after `SetPtViaCda` (line ~981):

```rust
// Layer 7a: Dynamic CDAs (CR 613.4a)
LayerModification::SetPtDynamic { power, toughness } => {
    let controller = state
        .objects
        .get(&object_id)
        .map(|o| o.controller)
        .unwrap_or(PlayerId(0));
    let p = resolve_cda_amount(state, power, object_id, controller);
    let t = resolve_cda_amount(state, toughness, object_id, controller);
    chars.power = Some(p);
    chars.toughness = Some(t);
}
```

**CR**: 613.4a -- CDA P/T effects applied in Layer 7a.

### Change 4: Add `resolve_cda_amount` function

**File**: `crates/engine/src/rules/layers.rs`
**Action**: Add a new function (after `apply_layer_modification` or at end of file):

```rust
/// Evaluate an EffectAmount in CDA context (no EffectContext available).
///
/// CR 604.3: CDAs function in all zones. The evaluation uses the source object's
/// controller as the reference player for "you control" semantics.
///
/// Only a subset of EffectAmount variants are valid for CDA evaluation:
/// Fixed, PermanentCount, CardCount, DevotionTo, CounterCount.
/// Variants requiring EffectContext (XValue, LastEffectCount, LastDiceRoll) will
/// return 0 with a debug_assert.
fn resolve_cda_amount(
    state: &GameState,
    amount: &EffectAmount,
    object_id: ObjectId,
    controller: PlayerId,
) -> i32 {
    match amount {
        EffectAmount::Fixed(n) => *n,
        EffectAmount::PermanentCount { filter, controller: player_target } => {
            // Resolve PlayerTarget::Controller to the source object's controller.
            let players = resolve_cda_player_target(state, player_target, controller);
            state
                .objects
                .values()
                .filter(|obj| {
                    obj.zone == ZoneId::Battlefield
                        && obj.is_phased_in()
                        && players.contains(&obj.controller)
                        && {
                            let chars =
                                calculate_characteristics(state, obj.id)
                                    .unwrap_or_else(|| obj.characteristics.clone());
                            crate::effects::matches_filter(&chars, filter)
                        }
                })
                .count() as i32
        }
        EffectAmount::CardCount { zone, player: _, filter } => {
            let zone_id = resolve_cda_zone_target(zone, state, controller);
            state
                .objects
                .values()
                .filter(|obj| {
                    obj.zone == zone_id
                        && filter.as_ref().map(|f| {
                            crate::effects::matches_filter(&obj.characteristics, f)
                        }).unwrap_or(true)
                })
                .count() as i32
        }
        EffectAmount::DevotionTo(color) => {
            state
                .objects
                .values()
                .filter(|obj| {
                    obj.zone == ZoneId::Battlefield
                        && obj.is_phased_in()
                        && obj.controller == controller
                })
                .map(|obj| count_pips_of_color(&obj.characteristics.mana_cost, color))
                .sum()
        }
        EffectAmount::CounterCount { target, counter } => {
            // For CDA context, target should be EffectTarget::Source (the object itself).
            // Other targets are not valid in CDA context.
            if matches!(target, EffectTarget::Source) {
                state
                    .objects
                    .get(&object_id)
                    .and_then(|obj| obj.counters.get(counter).copied())
                    .unwrap_or(0) as i32
            } else {
                debug_assert!(false, "CDA CounterCount with non-Source target");
                0
            }
        }
        _ => {
            debug_assert!(false, "EffectAmount variant {:?} not valid in CDA context", amount);
            0
        }
    }
}
```

Helper functions needed:

```rust
/// Resolve a PlayerTarget in CDA context (no EffectContext).
fn resolve_cda_player_target(
    state: &GameState,
    target: &PlayerTarget,
    controller: PlayerId,
) -> Vec<PlayerId> {
    match target {
        PlayerTarget::Controller => vec![controller],
        PlayerTarget::EachPlayer => state.turn_order.iter().copied().collect(),
        PlayerTarget::EachOpponent => state
            .turn_order
            .iter()
            .copied()
            .filter(|&p| p != controller)
            .collect(),
        _ => vec![controller], // fallback
    }
}

/// Resolve a ZoneTarget in CDA context.
fn resolve_cda_zone_target(
    zone: &ZoneTarget,
    state: &GameState,
    controller: PlayerId,
) -> ZoneId {
    // Mirrors resolve_zone_target from effects/mod.rs but uses controller directly.
    match zone {
        ZoneTarget::Hand { owner } => {
            let pid = resolve_cda_player_for_zone(owner, controller);
            ZoneId::Hand(pid)
        }
        ZoneTarget::Graveyard { owner } => {
            let pid = resolve_cda_player_for_zone(owner, controller);
            ZoneId::Graveyard(pid)
        }
        ZoneTarget::Library { owner, .. } => {
            let pid = resolve_cda_player_for_zone(owner, controller);
            ZoneId::Library(pid)
        }
        ZoneTarget::Battlefield { .. } => ZoneId::Battlefield,
        ZoneTarget::Exile => ZoneId::Exile,
        ZoneTarget::CommandZone => ZoneId::CommandZone,
    }
}
```

**Note**: `matches_filter` in `effects/mod.rs` is currently `pub(crate)` or private. May need
to make it `pub(crate)` if not already. Check visibility.

**Note**: `count_pips_of_color` helper may need to be extracted from the existing DevotionTo
implementation in `effects/mod.rs` and made reusable, OR duplicated in layers.rs.

### Change 5: Add `AbilityDefinition::CdaPowerToughness`

**File**: `crates/engine/src/cards/card_definition.rs`
**Action**: Add new variant to `AbilityDefinition` enum (after `StaticRestriction` or similar):

```rust
/// Characteristic-defining ability for power and/or toughness (CR 604.3, 613.4a).
///
/// Registers a Layer 7a continuous effect with `is_cda: true` when the permanent
/// enters the battlefield. The EffectAmount values are evaluated dynamically
/// at layer-calculation time.
///
/// CR 604.3: CDAs function in all zones. The layer system processes all objects
/// regardless of zone, so this is handled automatically.
///
/// CR 604.3a(5): CDAs must not be conditional. No condition field here.
CdaPowerToughness {
    power: EffectAmount,
    toughness: EffectAmount,
},
```

### Change 6: Register CDA continuous effect on ETB

**File**: `crates/engine/src/rules/replacement.rs`
**Action**: In `register_static_continuous_effects` (line ~1696), add a match arm for the new
`AbilityDefinition::CdaPowerToughness` variant:

```rust
AbilityDefinition::CdaPowerToughness { power, toughness } => {
    let eff_id = state.next_object_id().0;
    let ts = state.timestamp_counter;
    state.timestamp_counter += 1;
    state.continuous_effects.push_back(ContinuousEffect {
        id: EffectId(eff_id),
        source: Some(new_id),
        timestamp: ts,
        layer: EffectLayer::PtCda,
        duration: EffectDuration::WhileSourceOnBattlefield,
        filter: EffectFilter::SingleObject(new_id),
        modification: LayerModification::SetPtDynamic {
            power: power.clone(),
            toughness: toughness.clone(),
        },
        is_cda: true,  // CR 613.3: CDAs apply before non-CDAs in same layer
        condition: None, // CR 604.3a(5): CDAs are unconditional
    });
}
```

**CR**: 604.3 -- CDAs function in all zones. `WhileSourceOnBattlefield` is technically wrong
for "all zones" but the layer system evaluates the CDA for all objects regardless. The
duration controls when the ContinuousEffect is cleaned up. For non-battlefield zones, the
CDA is evaluated directly from the `power: None` / `toughness: None` on the card definition
at object creation time (handled by `enrich_spec_from_def`). WAIT -- this is a concern.

**IMPORTANT**: CR 604.3 says CDAs function in ALL zones. The current approach of registering
a continuous effect only on ETB means the CDA won't work in hand/graveyard/exile. For the
initial implementation, this is acceptable because:
- P/T is primarily relevant on the battlefield (for combat, SBAs)
- Targeting cards in graveyards by P/T is a future concern
- The layer system already evaluates continuous effects for all zones

For full CR compliance, a future enhancement could evaluate CDA directly from CardDefinition
when `calculate_characteristics` finds `power: None` on a non-battlefield object. For now,
the ETB registration is sufficient.

### Change 7: Exhaustive match updates

Files requiring new match arms for `AbilityDefinition::CdaPowerToughness`:

| File | Match expression | Action |
|------|-----------------|--------|
| `crates/engine/src/rules/replacement.rs` | `register_static_continuous_effects` match on AbilityDefinition | New arm (Change 6 above) |
| `crates/engine/src/rules/abilities.rs` | `check_triggers` / ability scanning | Add `_ => {}` or explicit no-op arm (verify existing wildcard covers it) |
| `crates/engine/src/testing/replay_harness.rs` | `enrich_spec_from_def` AbilityDefinition match | Add arm (see Change 8) |
| `crates/engine/src/state/hash.rs` | `HashInto for AbilityDefinition` | Add hash arm |
| `tools/replay-viewer/src/view_model.rs` | AbilityDefinition display | Add arm if matched exhaustively |
| `tools/tui/src/play/panels/stack_view.rs` | If AbilityDefinition is matched | Add arm if matched exhaustively |

Files requiring new match arms for `LayerModification::SetPtDynamic`:

| File | Match expression | Action |
|------|-----------------|--------|
| `crates/engine/src/state/hash.rs` | `HashInto for LayerModification` | Discriminant 22 (Change 2) |
| `crates/engine/src/rules/layers.rs` | `apply_layer_modification` match | New arm (Change 3) |
| `crates/engine/src/rules/layers.rs` | `depends_on` match | Falls through to `_ => false` wildcard -- OK |

### Change 8: Update `enrich_spec_from_def` for CDA awareness

**File**: `crates/engine/src/testing/replay_harness.rs` (or wherever `enrich_spec_from_def` lives)
**Action**: When a CardDefinition has `power: None` / `toughness: None`, the object's
`Characteristics.power` should be set to `None` (not `Some(0)`). Verify current behavior.
If `enrich_spec_from_def` currently sets `chars.power = def.power`, this is already correct
since `def.power` is `None`.

Also add `AbilityDefinition::CdaPowerToughness { .. } => {}` to any match on abilities in
`enrich_spec_from_def`.

### Change 9: Make `matches_filter` accessible from layers.rs

**File**: `crates/engine/src/effects/mod.rs`
**Action**: Ensure `matches_filter` has `pub(crate)` visibility so it can be called from
`rules/layers.rs`. Currently check its visibility -- if it's already `pub(crate)`, no change
needed.

## Card Definition Fixes

Existing card defs that this primitive unblocks. All are true CDA cards with `*/*` or `*/N`
printed P/T.

### battle_squadron.rs
**Oracle text**: "Flying\nBattle Squadron's power and toughness are each equal to the number of creatures you control."
**Current state**: `power: None, toughness: None`, TODO at line 16
**Fix**: Add `AbilityDefinition::CdaPowerToughness`:
```rust
AbilityDefinition::CdaPowerToughness {
    power: EffectAmount::PermanentCount {
        filter: TargetFilter { has_card_type: Some(CardType::Creature), ..Default::default() },
        controller: PlayerTarget::Controller,
    },
    toughness: EffectAmount::PermanentCount {
        filter: TargetFilter { has_card_type: Some(CardType::Creature), ..Default::default() },
        controller: PlayerTarget::Controller,
    },
},
```
Remove TODO.

### molimo_maro_sorcerer.rs
**Oracle text**: "Trample\nMolimo's power and toughness are each equal to the number of lands you control."
**Current state**: `power: None, toughness: None`, TODO at lines 6, 22
**Fix**: Add `AbilityDefinition::CdaPowerToughness`:
```rust
AbilityDefinition::CdaPowerToughness {
    power: EffectAmount::PermanentCount {
        filter: TargetFilter { has_card_type: Some(CardType::Land), ..Default::default() },
        controller: PlayerTarget::Controller,
    },
    toughness: EffectAmount::PermanentCount {
        filter: TargetFilter { has_card_type: Some(CardType::Land), ..Default::default() },
        controller: PlayerTarget::Controller,
    },
},
```
Remove TODO.

### greensleeves_maro_sorcerer.rs
**Oracle text**: "Protection from planeswalkers and from Wizards\nGreensleeves's power and toughness are each equal to the number of lands you control.\nLandfall -- Whenever a land you control enters, create a 3/3 green Badger creature token."
**Current state**: `power: None, toughness: None`, TODO expected
**Fix**: Same CDA as Molimo (lands you control). Protection from planeswalkers/Wizards is separate.

### cultivator_colossus.rs
**Oracle text**: "Trample\nCultivator Colossus's power and toughness are each equal to the number of lands you control.\nWhen this creature enters, you may put a land card from your hand onto the battlefield tapped. If you do, draw a card and repeat this process."
**Current state**: `power: None, toughness: None`, TODOs at lines 8, 22
**Fix**: Add CDA (lands you control). The ETB loop remains a separate TODO (too complex for this batch).

### reckless_one.rs
**Oracle text**: "Haste\nReckless One's power and toughness are each equal to the number of Goblins on the battlefield."
**Current state**: `power: None, toughness: None`, TODOs at lines 4, 19
**Fix**: Add `AbilityDefinition::CdaPowerToughness`:
```rust
AbilityDefinition::CdaPowerToughness {
    power: EffectAmount::PermanentCount {
        filter: TargetFilter {
            has_card_type: Some(CardType::Creature),
            has_subtype: Some(SubType("Goblin".to_string())),
            ..Default::default()
        },
        controller: PlayerTarget::EachPlayer,  // ALL Goblins, not just yours
    },
    toughness: EffectAmount::PermanentCount {
        filter: TargetFilter {
            has_card_type: Some(CardType::Creature),
            has_subtype: Some(SubType("Goblin".to_string())),
            ..Default::default()
        },
        controller: PlayerTarget::EachPlayer,
    },
},
```
Remove TODO.

### psychosis_crawler.rs
**Oracle text**: "Psychosis Crawler's power and toughness are each equal to the number of cards in your hand.\nWhenever you draw a card, each opponent loses 1 life."
**Current state**: `power: None, toughness: None`, TODO at line 22
**Fix**: Add `AbilityDefinition::CdaPowerToughness`:
```rust
AbilityDefinition::CdaPowerToughness {
    power: EffectAmount::CardCount {
        zone: ZoneTarget::Hand { owner: PlayerTarget::Controller },
        player: PlayerTarget::Controller,
        filter: None,
    },
    toughness: EffectAmount::CardCount {
        zone: ZoneTarget::Hand { owner: PlayerTarget::Controller },
        player: PlayerTarget::Controller,
        filter: None,
    },
},
```
Remove TODO. The draw trigger is already implemented.

### abomination_of_llanowar.rs
**Oracle text**: "Vigilance; menace\nAbomination of Llanowar's power and toughness are each equal to the number of Elves you control plus the number of Elf cards in your graveyard."
**Current state**: `power: None, toughness: None`, TODOs at lines 3, 23
**Fix**: This card needs **two** EffectAmount values summed (Elves on battlefield + Elf cards in graveyard). The current `EffectAmount` enum doesn't have a `Sum` variant.

**Option A**: Add `EffectAmount::Sum(Box<EffectAmount>, Box<EffectAmount>)` variant.
**Option B**: Add `EffectAmount::AddAmounts(Vec<EffectAmount>)` variant.
**Option C**: Handle this specific card with a combined count in `resolve_cda_amount`.

Recommend **Option A** (`EffectAmount::Sum`) as it's general-purpose and useful beyond this card.

```rust
AbilityDefinition::CdaPowerToughness {
    power: EffectAmount::Sum(
        Box::new(EffectAmount::PermanentCount {
            filter: TargetFilter {
                has_card_type: Some(CardType::Creature),
                has_subtype: Some(SubType("Elf".to_string())),
                ..Default::default()
            },
            controller: PlayerTarget::Controller,
        }),
        Box::new(EffectAmount::CardCount {
            zone: ZoneTarget::Graveyard { owner: PlayerTarget::Controller },
            player: PlayerTarget::Controller,
            filter: Some(TargetFilter {
                has_card_type: Some(CardType::Creature),
                has_subtype: Some(SubType("Elf".to_string())),
                ..Default::default()
            }),
        }),
    ),
    toughness: /* same as power */,
},
```
Remove TODO.

### jagged_scar_archers.rs
**Oracle text**: "Reach\nJagged-Scar Archers's power and toughness are each equal to the number of Elves you control."
**Current state**: `power: None, toughness: None`, TODOs at lines 4, 21
**Fix**: Same pattern as Battle Squadron but with Elf subtype filter and `Controller` only.

### adeline_resplendent_cathar.rs
**Oracle text**: "Vigilance\nAdeline's power is equal to the number of creatures you control.\nWhenever you attack, for each opponent, create a 1/1 white Human creature token that's tapped and attacking that player or a planeswalker they control."
**Current state**: `power: None, toughness: Some(4)`, TODO at line 24
**Fix**: Add `AbilityDefinition::CdaPowerToughness` with different power and toughness:
```rust
AbilityDefinition::CdaPowerToughness {
    power: EffectAmount::PermanentCount {
        filter: TargetFilter { has_card_type: Some(CardType::Creature), ..Default::default() },
        controller: PlayerTarget::Controller,
    },
    toughness: EffectAmount::Fixed(4),
},
```
The attack trigger remains a separate TODO (per-opponent token creation). Remove only the CDA TODO.

### nighthawk_scavenger.rs
**Oracle text**: "Flying, deathtouch, lifelink\nNighthawk Scavenger's power is equal to 1 plus the number of card types among cards in your opponents' graveyards."
**Current state**: `power: Some(1), toughness: Some(3)` (wrong -- should be `power: None`)
**Fix**: This card needs `EffectAmount::DistinctCardTypesInZone` -- a new variant counting
distinct card types. **DEFER to a future batch** unless the runner adds this variant.

Actually, re-examining: the printed P/T is `1+*/3` where `*` = card type count. With the
`Sum` variant:
```rust
power: EffectAmount::Sum(
    Box::new(EffectAmount::Fixed(1)),
    Box::new(EffectAmount::DistinctCardTypesInGraveyards {
        player: PlayerTarget::EachOpponent,
    }),
),
toughness: EffectAmount::Fixed(3),
```

Since `DistinctCardTypesInGraveyards` is a new EffectAmount variant, this is additional scope.
**Include if simple; defer if complex.**

### wrenn_and_seven.rs (token CDA)
**Oracle text**: "-3: Create a green Treefolk creature token with reach and 'This creature's power and toughness are each equal to the number of lands you control.'"
**Current state**: Token created with `power: 0, toughness: 0`, TODO at line 47
**Fix**: The token needs a CDA. This requires `TokenSpec` to support CDA on created tokens.
The current `TokenSpec` has `power: i32, toughness: i32` (fixed). Adding CDA to tokens is
additional scope. **DEFER** -- note that the token works but has wrong P/T.

## New EffectAmount Variant: Sum

### EffectAmount::Sum

**File**: `crates/engine/src/cards/card_definition.rs`
**Action**: Add variant to `EffectAmount` enum:

```rust
/// Sum of two amounts. Used for CDAs like "equal to X plus Y" (e.g., Abomination of
/// Llanowar: "number of Elves you control plus Elf cards in graveyard").
Sum(Box<EffectAmount>, Box<EffectAmount>),
```

**Hash**: Add to `HashInto for EffectAmount` in hash.rs (discriminant 11):
```rust
EffectAmount::Sum(a, b) => {
    11u8.hash_into(hasher);
    a.hash_into(hasher);
    b.hash_into(hasher);
}
```

**Evaluation in effects/mod.rs**: Add arm in `resolve_amount`:
```rust
EffectAmount::Sum(a, b) => resolve_amount(state, a, ctx) + resolve_amount(state, b, ctx),
```

**Evaluation in layers.rs**: Add arm in `resolve_cda_amount`:
```rust
EffectAmount::Sum(a, b) => {
    resolve_cda_amount(state, a, object_id, controller)
        + resolve_cda_amount(state, b, object_id, controller)
}
```

## New EffectAmount Variant: DistinctCardTypesInGraveyards (optional)

If time permits, add for Nighthawk Scavenger / Tarmogoyf:

**File**: `crates/engine/src/cards/card_definition.rs`
**Action**: Add variant:

```rust
/// Count of distinct card types among cards in the specified zone(s).
/// Used by Tarmogoyf, Nighthawk Scavenger, Delirium.
DistinctCardTypesInZone {
    zone: ZoneTarget,
    player: PlayerTarget,
},
```

This counts unique card types (Creature, Instant, Sorcery, Land, Enchantment, Artifact,
Planeswalker, Kindred, Battle) among cards in the specified graveyards. Implementation:
collect all card_types from objects in the zone, deduplicate, count.

**Defer to PB-28.5 or a future batch if complex.** The core CDA mechanism is the priority.

## Unit Tests

**File**: `crates/engine/tests/cda_tests.rs` (new file)
**Tests to write**:

- `test_cda_power_toughness_basic` -- CR 604.3, 613.4a: Create a */* creature with
  CdaPowerToughness(PermanentCount creatures), verify P/T equals creature count on battlefield.

- `test_cda_updates_dynamically` -- CR 604.3: Create a */* creature, add another creature,
  verify P/T increased. Remove a creature, verify P/T decreased.

- `test_cda_layer_7a_before_7b` -- CR 613.4a/b: A */* CDA creature affected by Humility
  (Layer 7b sets to 1/1). CDA sets P/T in 7a, then Humility overrides in 7b. Result: 1/1.

- `test_cda_layer_7a_before_7c` -- CR 613.4a/c: A */* CDA creature with a +1/+1 counter.
  CDA sets base P/T in 7a, counter adds in 7c. Result: CDA_value + 1 / CDA_value + 1.

- `test_cda_zero_toughness_dies` -- CR 704.5f: A */* creature whose CDA evaluates to 0
  (e.g., no creatures you control besides itself, but it counts itself) should have correct
  P/T. If P/T = 0 after CDA, it dies to SBA.

- `test_cda_counts_self` -- Verify that a "creatures you control" CDA counts the creature
  itself (it's on the battlefield when the CDA is evaluated).

- `test_cda_partial_power_only` -- Adeline pattern: `*/4` where only power is CDA.
  Verify power equals creature count, toughness is fixed 4.

- `test_cda_with_effect_amount_sum` -- Abomination of Llanowar pattern: P/T = battlefield
  Elves + graveyard Elf cards. Verify sum works correctly.

- `test_cda_multiplayer` -- Reckless One pattern: "Goblins on the battlefield" counts ALL
  players' Goblins, not just controller's. Verify in 4-player game.

**Pattern**: Follow tests for Layer 7 interactions in `crates/engine/tests/layers_tests.rs`
(if exists) or continuous effect tests.

## Verification Checklist

- [ ] Engine primitive compiles (`cargo check`)
- [ ] All existing card def TODOs for CDA P/T resolved (9-10 cards)
- [ ] New card defs authored (if any -- likely none this batch)
- [ ] Unit tests pass (`cargo test --all`)
- [ ] Clippy clean (`cargo clippy -- -D warnings`)
- [ ] Workspace builds (`cargo build --workspace`)
- [ ] No remaining CDA TODOs in affected card defs

## Risks & Edge Cases

- **Recursive evaluation**: `resolve_cda_amount` calls `calculate_characteristics` for
  `PermanentCount` (to use layer-resolved types for filter matching). This is safe because
  the inner `calculate_characteristics` call processes a DIFFERENT object, not the CDA object
  itself. However, if two CDA objects reference each other (unlikely but theoretically
  possible), this could infinite-loop. Mitigate with a recursion depth guard or accept as
  a theoretical risk.

- **CR 604.3 all-zones compliance**: The initial implementation only registers CDA effects
  on ETB (battlefield). For non-battlefield zones, `power: None` persists in characteristics.
  This means library/graveyard targeting by P/T won't correctly evaluate CDAs. This is a
  known limitation acceptable for alpha. The fix is to evaluate CDA amounts directly from
  CardDefinition in `calculate_characteristics` when `chars.power` is `None` and the def
  has a `CdaPowerToughness` ability.

- **`matches_filter` visibility**: The `matches_filter` function in `effects/mod.rs` may
  be private. Needs `pub(crate)` visibility for `layers.rs` to call it. If making it public
  is undesirable, duplicate the filter matching logic in layers.rs (but this violates DRY).

- **`EffectAmount::Sum` import in card defs**: Card defs use `use crate::cards::helpers::*;`
  so `EffectAmount::Sum` must be usable through that import. `EffectAmount` is already
  exported from helpers.rs, so `Sum` will be available automatically.

- **Token CDA**: Wrenn and Seven's token and Promise of Power's token need CDA support on
  `TokenSpec`. This is OUT OF SCOPE for PB-28 -- tokens will continue to have fixed P/T.
  A future batch could add `cda_power: Option<EffectAmount>` to `TokenSpec`.

- **Nighthawk Scavenger `power: Some(1)` is wrong**: The card def currently has
  `power: Some(1)` but the oracle P/T is `1+*/3`. The `power` field should be `None`
  (it's a CDA). Fix in the card def.

## Implementation Order

1. Add `EffectAmount::Sum` variant + hash + resolve_amount arm + resolve_cda_amount arm
2. Add `LayerModification::SetPtDynamic` variant + hash
3. Add `AbilityDefinition::CdaPowerToughness` variant + hash + exhaustive matches
4. Add `resolve_cda_amount` + helpers in layers.rs
5. Update `apply_layer_modification` signature and call site
6. Add match arm in `apply_layer_modification` for `SetPtDynamic`
7. Add registration in `register_static_continuous_effects`
8. Make `matches_filter` pub(crate) if needed
9. Fix card defs (9-10 cards)
10. Write unit tests
11. `cargo build --workspace` + `cargo test --all` + `cargo clippy`
