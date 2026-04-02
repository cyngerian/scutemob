# Primitive Batch Plan: PB-G -- BounceAll / Mass Return-to-Hand

**Generated**: 2026-04-01
**Primitive**: `Effect::BounceAll { filter: TargetFilter }` + TargetFilter extensions (`max_toughness`, `exclude_subtypes`, `is_attacking`)
**CR Rules**: CR 508.1k (attacking creature definition), CR 400.7 (zone change = new object)
**Cards affected**: 4 new + 2 existing fixes (Crux of Fate, Recruiter of the Guard)
**Dependencies**: None (DestroyAll/ExileAll pattern already exists)
**Deferred items from prior PBs**: None

## Primitive Specification

Add `Effect::BounceAll { filter: TargetFilter }` to the `Effect` enum. This returns all
permanents on the battlefield matching the filter to their owners' hands. It follows the
exact pattern of `Effect::DestroyAll` and `Effect::ExileAll`:
1. Snapshot matching objects before any zone changes (pre-resolution state)
2. Use layer-resolved characteristics via `calculate_characteristics()`
3. Check `filter.controller` against `obj.controller`
4. Move each matching object to its owner's hand via `move_object_to_zone()`
5. Emit `GameEvent::ObjectReturnedToHand` for each bounced object
6. Check replacement effects via `check_zone_change_replacement()` (Battlefield -> Hand)
7. Store count in `ctx.last_effect_count`

Additionally, extend `TargetFilter` with three new fields needed by the 4 cards:
- `max_toughness: Option<i32>` -- max toughness filter (matches existing `max_power` pattern)
- `exclude_subtypes: Vec<SubType>` -- exclude objects with any of these subtypes
- `is_attacking: bool` -- must be an attacking creature (runtime check, not in `matches_filter`)

## CR Rule Text

**CR 508.1k**: "Each chosen creature still controlled by the active player becomes an
attacking creature. It remains an attacking creature until it's removed from combat or
the combat phase ends, whichever comes first."

**CR 400.7**: "An object that moves from one zone to another becomes a new object with no
memory of, or relation to, its previous existence."

No specific CR rule governs "return to hand" as a keyword action. It is simply a zone change
from any zone to the owner's hand. The engine already handles this via `move_object_to_zone()`
and emits `GameEvent::ObjectReturnedToHand`.

## Engine Changes

### Change 1: Add `Effect::BounceAll` variant

**File**: `crates/engine/src/cards/card_definition.rs`
**Action**: Add new variant after `ExileAll` (~line 1122):
```rust
/// Return all permanents on the battlefield matching the filter to their owners' hands.
/// Stores the count of actually-bounced permanents in ctx.last_effect_count.
BounceAll { filter: TargetFilter },
```
**Pattern**: Follow `ExileAll { filter: TargetFilter }` at line 1122

### Change 2: Add `max_toughness`, `exclude_subtypes`, `is_attacking` to TargetFilter

**File**: `crates/engine/src/cards/card_definition.rs`
**Action**: Add three new fields to the `TargetFilter` struct (~line 1983-2042):
```rust
/// Max toughness (inclusive). None = no restriction.
/// Used for "creature with toughness N or less" (Scourge of Fleets, Recruiter of the Guard).
pub max_toughness: Option<i32>,

/// Subtype exclusion (AND semantics -- must NOT have any of these subtypes).
/// Used for "except for Krakens, Leviathans, Octopuses, and Serpents" (Whelming Wave).
/// Also used for "destroy all non-Dragon creatures" (Crux of Fate).
#[serde(default)]
pub exclude_subtypes: Vec<SubType>,

/// Must be currently attacking (CR 508.1k).
/// NOTE: Like `is_token`, this is a runtime property of the `GameObject` (checked via
/// `CombatState.attackers`), NOT a `Characteristics` field. It is NOT checked inside
/// `matches_filter()`. It MUST be checked explicitly at each call site that uses it
/// (currently: BounceAll execution, and any future DestroyAll/ExileAll call sites).
#[serde(default)]
pub is_attacking: bool,
```

### Change 3: Update `matches_filter()` for `max_toughness` and `exclude_subtypes`

**File**: `crates/engine/src/effects/mod.rs`
**Action**: Add two new filter checks to `matches_filter()` (~after line 5486, before the return):
```rust
// Max toughness filter (matches max_power pattern).
if let Some(max_t) = filter.max_toughness {
    if chars.toughness.map(|t| t > max_t).unwrap_or(true) {
        return false;
    }
}

// Subtype exclusion: reject if object has ANY of the excluded subtypes.
if !filter.exclude_subtypes.is_empty()
    && filter.exclude_subtypes.iter().any(|st| chars.subtypes.contains(st))
{
    return false;
}
```

**Note**: `is_attacking` is NOT added to `matches_filter()` -- it requires `CombatState` access
and must be checked at the call site (same pattern as `is_token`).

### Change 4: BounceAll execution in effects/mod.rs

**File**: `crates/engine/src/effects/mod.rs`
**Action**: Add execution arm after the `ExileAll` arm (~after line 1097). Pattern follows
`ExileAll` exactly but moves to owner's hand instead of exile.

The key differences from ExileAll:
- Zone destination: `ZoneId::Hand(owner)` instead of `ZoneId::Exile`
- Replacement check: `check_zone_change_replacement(state, id, ZoneType::Battlefield, ZoneType::Hand, owner, ...)`
- Event: `GameEvent::ObjectReturnedToHand` instead of `ObjectExiled`
- `is_attacking` check: if `filter.is_attacking`, additionally check `state.combat.as_ref().map_or(false, |c| c.attackers.contains_key(&id))`

Full execution logic:
```rust
Effect::BounceAll { filter } => {
    // Snapshot matching objects before any zone changes.
    // CR 613.1d: Use layer-resolved characteristics for filter matching.
    let ids_to_bounce: Vec<ObjectId> = state
        .objects
        .iter()
        .filter(|(id, obj)| {
            obj.zone == ZoneId::Battlefield
                && obj.is_phased_in()
                && {
                    let chars =
                        crate::rules::layers::calculate_characteristics(state, **id)
                            .unwrap_or_else(|| obj.characteristics.clone());
                    matches_filter(&chars, filter)
                }
                && match filter.controller {
                    TargetController::Any => true,
                    TargetController::You => obj.controller == ctx.controller,
                    TargetController::Opponent => obj.controller != ctx.controller,
                }
                // is_attacking: runtime check against CombatState (not in matches_filter)
                && (!filter.is_attacking
                    || state
                        .combat
                        .as_ref()
                        .map_or(false, |c| c.attackers.contains_key(id)))
                // is_token: runtime check (not in matches_filter)
                && (!filter.is_token || obj.is_token)
        })
        .map(|(&id, _)| id)
        .collect();
    let mut bounced_count: u32 = 0;
    for id in ids_to_bounce {
        let owner = state
            .objects
            .get(&id)
            .map(|o| o.owner)
            .unwrap_or(ctx.controller);
        // CR 614: Check replacement effects before moving.
        let action = crate::rules::replacement::check_zone_change_replacement(
            state,
            id,
            ZoneType::Battlefield,
            ZoneType::Hand,
            owner,
            &std::collections::HashSet::new(),
        );
        // Handle replacement action (same pattern as ExileAll).
        // ... (Redirect / ChoiceRequired / Proceed match arms)
        // On successful move to hand:
        //   events.push(GameEvent::ObjectReturnedToHand { ... });
        //   bounced_count += 1;
    }
    ctx.last_effect_count = bounced_count;
}
```

### Change 5: HashInto for new TargetFilter fields

**File**: `crates/engine/src/state/hash.rs`
**Action**: Update the `HashInto for TargetFilter` impl (~line 3884) to hash the three new fields:
```rust
// After existing fields (line ~3903, after is_token):
self.max_toughness.hash_into(hasher);
self.exclude_subtypes.hash_into(hasher);
self.is_attacking.hash_into(hasher);
```

### Change 6: HashInto for Effect::BounceAll

**File**: `crates/engine/src/state/hash.rs`
**Action**: Add hash arm for `Effect::BounceAll` after `ExileAll` (~line 4856). Use discriminant 74:
```rust
// BounceAll (discriminant 74) — mass return to hand
Effect::BounceAll { filter } => {
    74u8.hash_into(hasher);
    filter.hash_into(hasher);
}
```

### Change 7: Exhaustive match updates

No other files match on `Effect::` variants outside of `effects/mod.rs` and `hash.rs`.
The replay-viewer and TUI do not match on Effect variants (confirmed via grep).

Summary of all files needing changes:

| File | Match/Struct | Line | Action |
|------|-------------|------|--------|
| `cards/card_definition.rs` | `Effect` enum | ~L1122 | Add `BounceAll { filter: TargetFilter }` variant |
| `cards/card_definition.rs` | `TargetFilter` struct | ~L2042 | Add `max_toughness`, `exclude_subtypes`, `is_attacking` fields |
| `effects/mod.rs` | `execute_effect` match | ~L1097 | Add `BounceAll` execution arm |
| `effects/mod.rs` | `matches_filter` fn | ~L5486 | Add `max_toughness` and `exclude_subtypes` checks |
| `state/hash.rs` | `HashInto for TargetFilter` | ~L3884 | Hash 3 new fields |
| `state/hash.rs` | `HashInto for Effect` | ~L4856 | Add discriminant 74 arm |

## Card Definition Fixes

### Existing card defs with TODOs fixed by TargetFilter extensions:

### crux_of_fate.rs
**Oracle text**: "Choose one -- Destroy all Dragon creatures / Destroy all non-Dragon creatures"
**Current state**: TODO at line 31 -- destroys ALL creatures because `exclude_subtypes` missing
**Fix**: Mode 2 should use `DestroyAll { filter: TargetFilter { has_card_type: Some(CardType::Creature), exclude_subtypes: vec![SubType::Dragon], ..Default::default() }, cant_be_regenerated: false }`

### recruiter_of_the_guard.rs
**Oracle text**: "When this creature enters, you may search your library for a creature card with toughness 2 or less..."
**Current state**: TODO at line 18 -- search filter lacks `max_toughness`
**Fix**: Set `max_toughness: Some(2)` on the search TargetFilter

## New Card Definitions

### aetherize.rs
**Oracle text**: "Return all attacking creatures to their owner's hand."
**CardDefinition sketch**:
```rust
CardDefinition {
    name: "Aetherize",
    mana_cost: Some(ManaCost { generic: 3, blue: 1, ..Default::default() }),
    type_line: TypeLine { card_types: vec![CardType::Instant], ..Default::default() },
    effect: Effect::BounceAll {
        filter: TargetFilter {
            has_card_type: Some(CardType::Creature),
            is_attacking: true,
            ..Default::default()
        },
    },
    ..
}
```

### whelming_wave.rs
**Oracle text**: "Return all creatures to their owners' hands except for Krakens, Leviathans, Octopuses, and Serpents."
**CardDefinition sketch**:
```rust
effect: Effect::BounceAll {
    filter: TargetFilter {
        has_card_type: Some(CardType::Creature),
        exclude_subtypes: vec![SubType::Kraken, SubType::Leviathan, SubType::Octopus, SubType::Serpent],
        ..Default::default()
    },
}
```

### scourge_of_fleets.rs
**Oracle text**: "When this creature enters, return each creature your opponents control with toughness X or less to its owner's hand, where X is the number of Islands you control."
**CardDefinition sketch**: This card uses an ETB trigger. Since the toughness threshold is
dynamic (= number of Islands you control), the card definition needs a way to pass a dynamic
`max_toughness`. Two approaches:

**Approach A (recommended)**: Use `Effect::ForEach` over opponents' creatures, with a
`Conditional` that checks toughness against `PermanentCount`. This avoids adding dynamic
filter support to BounceAll.

**Approach B**: Add an optional `max_toughness_override: Option<EffectAmount>` field to
`BounceAll` that, when present, overrides `filter.max_toughness` with the resolved amount.
This is cleaner DSL but adds complexity to the Effect variant.

**Recommended: Approach A** -- Express Scourge of Fleets as:
```rust
// ETB triggered ability
triggered_abilities: vec![TriggeredAbilityDef {
    trigger: TriggerCondition::SelfEntersBattlefield,
    effect: Effect::BounceAll {
        filter: TargetFilter {
            has_card_type: Some(CardType::Creature),
            controller: TargetController::Opponent,
            ..Default::default()
        },
    },
    ..
}]
```

**Problem**: Approach A still can't express the dynamic toughness threshold with a static
TargetFilter. We need Approach B or a third option.

**Approach C (simplest)**: Add `max_toughness_amount: Option<EffectAmount>` directly to
`Effect::BounceAll` as an optional dynamic override. When present, its resolved value is
used as the max toughness for filtering (applied at execution time, before the filter loop).
When absent, `filter.max_toughness` (static) is used.

```rust
BounceAll {
    filter: TargetFilter,
    /// Optional dynamic toughness override. When set, resolved at execution time
    /// and applied as max_toughness for the filter.
    max_toughness_amount: Option<EffectAmount>,
},
```

Card def:
```rust
effect: Effect::BounceAll {
    filter: TargetFilter {
        has_card_type: Some(CardType::Creature),
        controller: TargetController::Opponent,
        ..Default::default()
    },
    max_toughness_amount: Some(EffectAmount::PermanentCount {
        filter: TargetFilter {
            has_subtype: Some(SubType::Island),
            ..Default::default()
        },
        controller: PlayerTarget::Controller,
    }),
},
```

**Decision: Use Approach C.** The `max_toughness_amount` field defaults to `None` for simple
cases (Aetherize, Whelming Wave, Filter Out) and is only used by Scourge of Fleets. The
execution code resolves the amount and temporarily sets `max_toughness` on the filter before
the snapshot loop.

### filter_out.rs
**Oracle text**: "Return all noncreature, nonland permanents to their owners' hands."
**CardDefinition sketch**:
```rust
effect: Effect::BounceAll {
    filter: TargetFilter {
        non_creature: true,
        non_land: true,
        ..Default::default()
    },
    max_toughness_amount: None,
}
```

## Unit Tests

**File**: `crates/engine/tests/mass_bounce.rs` (new file)
**Tests to write**:

- `test_bounce_all_creatures_basic` -- BounceAll with creature filter returns all creatures to owners' hands; non-creatures stay. CR 400.7: bounced objects become new objects in hand.
- `test_bounce_all_attacking_creatures` -- BounceAll with `is_attacking: true` only bounces attackers, not non-attacking creatures. CR 508.1k.
- `test_bounce_all_exclude_subtypes` -- BounceAll with `exclude_subtypes: [Kraken, Serpent]` bounces creatures that are NOT those types, keeps those types on battlefield. (Whelming Wave pattern.)
- `test_bounce_all_noncreature_nonland` -- BounceAll with `non_creature: true, non_land: true` bounces artifacts/enchantments/planeswalkers, keeps creatures and lands. (Filter Out pattern.)
- `test_bounce_all_opponent_creatures_with_toughness` -- BounceAll with `controller: Opponent, max_toughness_amount: Some(Fixed(3))` bounces opponent creatures with toughness <= 3. (Scourge of Fleets pattern.)
- `test_bounce_all_count_tracking` -- Verify `ctx.last_effect_count` tracks bounced count correctly.
- `test_bounce_all_multiplayer` -- 4-player game, BounceAll with `controller: Opponent` bounces all 3 opponents' matching permanents.
- `test_bounce_all_respects_replacement_effects` -- Commander zone-change replacement: when a commander is bounced, the SBA should allow it to go to command zone instead.
- `test_bounce_all_tokens_cease_to_exist` -- Token bounced to hand briefly exists there, then ceases to exist as SBA (CR 704.5d).

**Pattern**: Follow tests in `crates/engine/tests/mass_destroy.rs` (same builder setup, `run_effect` helper, assertions on zone contents).

### TargetFilter extension tests (in mass_bounce.rs or mass_destroy.rs):

- `test_max_toughness_filter` -- Verify `max_toughness: Some(2)` on a DestroyAll filter correctly destroys only creatures with toughness <= 2.
- `test_exclude_subtypes_filter` -- Verify `exclude_subtypes: vec![Dragon]` on a DestroyAll filter correctly skips Dragons (fixes Crux of Fate).

## Verification Checklist

- [ ] Engine primitive compiles (`cargo check`)
- [ ] All existing card def TODOs for this batch resolved (crux_of_fate.rs, recruiter_of_the_guard.rs)
- [ ] New card defs authored (aetherize.rs, whelming_wave.rs, scourge_of_fleets.rs, filter_out.rs)
- [ ] Unit tests pass (`cargo test --all`)
- [ ] Clippy clean (`cargo clippy -- -D warnings`)
- [ ] Workspace builds (`cargo build --workspace`)
- [ ] No remaining TODOs in affected card defs

## Risks & Edge Cases

- **`is_attacking` outside combat phase**: If BounceAll with `is_attacking: true` is somehow
  executed outside the combat phase (e.g., via a delayed trigger), `state.combat` will be
  `None` and no creatures will match. This is correct behavior (no attacking creatures exist
  outside combat). Aetherize is an instant typically cast during combat, so this is fine.

- **`is_attacking` is NOT checked by `matches_filter()`**: This follows the same pattern as
  `is_token`. The BounceAll execution code must explicitly check it. If `is_attacking` is
  later used in `DestroyAll` or `ExileAll` filters, those execution sites must also add the
  explicit check. Add a NOTE comment in the TargetFilter field documentation (like `is_token`).

- **Replacement effects on bounce**: When a commander is bounced, it goes to the hand (not
  graveyard/exile), so the CR 903.9b hand-redirect replacement may not apply (it applies when
  going to hand from stack, not from battlefield). Check: CR 903.9a applies to graveyard/exile
  only. A commander bounced to hand simply goes to the owner's hand. No special handling needed
  beyond the standard `check_zone_change_replacement()` call.

- **Object identity (CR 400.7)**: Each bounced permanent becomes a new object in hand. The
  execution code should use the new_id from `move_object_to_zone()` in the event, and the
  old id as `object_id`. This matches the ExileAll pattern.

- **Scourge of Fleets dynamic toughness**: The `max_toughness_amount` is resolved once before
  the snapshot loop, not per-object. This is correct because "X is the number of Islands you
  control" is determined as the ability resolves (Scourge ruling 2014-04-26), before objects
  are bounced. Islands don't leave the battlefield during bounce, so the count is stable.

- **SubType enum completeness**: Verify that `SubType::Kraken`, `SubType::Leviathan`,
  `SubType::Octopus`, and `SubType::Serpent` exist in the `SubType` enum. These are standard
  creature types and should already exist.

- **Default field values**: All three new TargetFilter fields must have `#[serde(default)]`
  and sensible defaults (`None`/`vec![]`/`false`). The `Default` derive on TargetFilter will
  handle this automatically since `Option<i32>` defaults to `None`, `Vec<SubType>` to `vec![]`,
  and `bool` to `false`.
