# Ability Plan: Connive

**Generated**: 2026-02-28
**CR**: 701.50 (keyword action, NOT a keyword ability)
**Priority**: P3
**Similar abilities studied**: Effect::Surveil (effects/mod.rs:1062-1096), discard_cards helper (effects/mod.rs:1966-2030)

## CR Rule Text

701.50. Connive

701.50a Certain spells and abilities instruct a permanent to connive. To do so, that
permanent's controller draws a card, then discards a card. If a nonland card is discarded
this way, that player puts a +1/+1 counter on the conniving permanent.

701.50b A permanent "connives" after the process described in rule 701.50a is complete,
even if some or all of those actions were impossible.

701.50c If a permanent changes zones before an effect causes it to connive, its last
known information is used to determine which object connived and who controlled it.

701.50d If multiple permanents are instructed to connive at the same time, the first
player in APNAP order who controls (or, in the case of a permanent no longer on the
battlefield, last controlled; see rule 701.50c) one or more of those permanents chooses
one of them and it connives. Then this process is repeated for each remaining instruction
to connive.

701.50e Connive N is a variant of connive. The permanent's controller draws N cards,
discards N cards, then puts a number of +1/+1 counters on the permanent equal to the
number of nonland cards discarded this way.

## Key Edge Cases

From CR children:
- **701.50b**: The permanent "connives" even if draw/discard were impossible (e.g., empty
  hand, empty library). A `Connived` event should always fire so "whenever [this] connives"
  triggers work.
- **701.50c**: If the permanent left the battlefield before connive resolves, the controller
  still draws and discards, but no +1/+1 counter is placed (there is no permanent to put
  it on). "Whenever [this] connives" triggers still fire.
- **701.50e**: Connive N draws N, discards N, then counts nonland among the N discarded
  cards. The count parameter is already modeled as `EffectAmount` in the enum variant.

From Raffine's Informant rulings:
- **No split resolution**: Once connive starts resolving, no player can take actions between
  the draw, discard, and counter placement. This is naturally handled by the effect handler
  executing atomically.
- **Empty hand, can't draw**: If no card is discarded (because hand was empty and draw was
  impossible), no +1/+1 counter is placed. The permanent still connives (701.50b).
- **Creature left the battlefield**: If the conniving creature left the battlefield, the
  controller still draws/discards. No counter is placed. "Whenever [this] connives" triggers
  still fire per 701.50b.

Multiplayer considerations:
- **701.50d**: Multiple simultaneous connive instructions resolve in APNAP order. For the
  initial implementation, this can be handled sequentially since each connive is typically
  triggered by a single source. Full APNAP ordering for simultaneous connive is a LOW
  priority edge case.

## Current State (from ability-wip.md)

- [x] Step 1: Enum variant -- `Effect::Connive { target: EffectTarget, count: EffectAmount }` exists at `card_definition.rs:400`
- [x] Step 1b: Hash support -- exists at `hash.rs:2497` (discriminant 35)
- [ ] Step 2: Effect handler in `effects/mod.rs`
- [ ] Step 3: GameEvent::Connived + hash support
- [ ] Step 4: Unit tests
- [ ] Step 5: Card definition
- [ ] Step 6: Game script

## Implementation Steps

### Step 1: Enum Variant (DONE)

**File**: `/home/airbaggie/scutemob/crates/engine/src/cards/card_definition.rs`
**Status**: Already added at line 400.
```rust
Connive {
    target: EffectTarget,
    count: EffectAmount,
},
```

### Step 1b: Hash Support (DONE)

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/hash.rs`
**Status**: Already added at line 2497 (discriminant 35).

### Step 2: GameEvent::Connived

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/events.rs`
**Action**: Add a `Connived` event variant near the `Surveilled` event (around line 689).

```rust
/// CR 701.50b: A permanent connived. Emitted after the draw/discard/counter
/// sequence completes, even if some actions were impossible.
/// Used by "whenever [this creature] connives" triggers.
Connived {
    /// The ObjectId of the conniving permanent (may no longer be on battlefield).
    object_id: ObjectId,
    /// The controller who performed the draw/discard.
    player: PlayerId,
    /// Number of +1/+1 counters placed (0 if only lands discarded or creature
    /// left the battlefield).
    counters_placed: u32,
},
```

**Pattern**: Follow `Surveilled` at line 689.

**Hash support**: Add to `state/hash.rs` GameEvent hash impl. Use next available discriminant (79).
```rust
GameEvent::Connived { object_id, player, counters_placed } => {
    79u8.hash_into(hasher);
    object_id.hash_into(hasher);
    player.hash_into(hasher);
    counters_placed.hash_into(hasher);
}
```
Insert before the closing `}` of the GameEvent match (after `GameEvent::CardForetold` at line 1960).

### Step 3: Effect Handler

**File**: `/home/airbaggie/scutemob/crates/engine/src/effects/mod.rs`
**Action**: Replace the stub at lines 1487-1492 with the full implementation.
**CR**: 701.50a, 701.50b, 701.50c, 701.50e

The implementation logic:

```rust
Effect::Connive { target, count } => {
    // CR 701.50a/e: Resolve the conniving permanent and draw/discard count.
    let n = resolve_amount(state, count, ctx).max(0) as usize;
    let targets = resolve_effect_target_list(state, target, ctx);

    for resolved in targets {
        if let ResolvedTarget::Object(creature_id) = resolved {
            // CR 701.50a: The permanent's CONTROLLER draws and discards.
            // CR 701.50c: If the permanent left the battlefield, use last
            // known controller (ctx.controller as fallback).
            let controller = state
                .objects
                .get(&creature_id)
                .map(|obj| obj.controller)
                .unwrap_or(ctx.controller);

            // Step 1: Draw N cards (CR 701.50e).
            for _ in 0..n {
                let draw_evts = draw_one_card(state, controller);
                events.extend(draw_evts);
            }

            // Step 2: Discard N cards deterministically.
            // We need to track which cards are discarded and whether they
            // are nonland, so we cannot use the generic `discard_cards` helper.
            // Instead, inline the discard logic and capture card types.
            let hand_zone = ZoneId::Hand(controller);
            let mut nonland_count: u32 = 0;

            for _ in 0..n {
                // Find the first card in hand (deterministic: min ObjectId).
                let card_id = state
                    .objects
                    .iter()
                    .filter(|(_, obj)| obj.zone == hand_zone)
                    .map(|(&id, _)| id)
                    .min_by_key(|id| id.0);

                if let Some(card_id) = card_id {
                    // CR 701.50a: Check if the discarded card is nonland BEFORE
                    // moving it (card types may not survive zone change).
                    let is_nonland = state
                        .objects
                        .get(&card_id)
                        .map(|obj| !obj.characteristics.card_types.contains(&CardType::Land))
                        .unwrap_or(false);

                    if is_nonland {
                        nonland_count += 1;
                    }

                    // Check for Madness (CR 702.35a) before zone change.
                    let has_madness = state
                        .objects
                        .get(&card_id)
                        .map(|obj| {
                            obj.characteristics
                                .keywords
                                .contains(&KeywordAbility::Madness)
                        })
                        .unwrap_or(false);

                    let destination = if has_madness {
                        ZoneId::Exile
                    } else {
                        ZoneId::Graveyard(controller)
                    };

                    if let Ok((new_id, _)) = state.move_object_to_zone(card_id, destination) {
                        events.push(GameEvent::CardDiscarded {
                            player: controller,
                            object_id: card_id,
                            new_id,
                        });

                        // If Madness, queue the madness trigger (same as discard_cards helper).
                        if has_madness {
                            // Look up madness cost and queue trigger.
                            let obj_card_id = state
                                .objects
                                .get(&new_id)
                                .and_then(|o| o.card_id.clone());
                            // (Copy the madness trigger logic from discard_cards)
                        }
                    }
                }
            }

            // Step 3: Place +1/+1 counters (CR 701.50a/e).
            // Only if the permanent is still on the battlefield (CR 701.50c).
            if nonland_count > 0 {
                if let Some(obj) = state.objects.get_mut(&creature_id) {
                    if obj.zone == ZoneId::Battlefield {
                        let cur = obj
                            .counters
                            .get(&CounterType::PlusOnePlusOne)
                            .copied()
                            .unwrap_or(0);
                        obj.counters
                            .insert(CounterType::PlusOnePlusOne, cur + nonland_count);
                        events.push(GameEvent::CounterAdded {
                            object_id: creature_id,
                            counter: CounterType::PlusOnePlusOne,
                            new_total: cur + nonland_count,
                        });
                    }
                }
            }

            // CR 701.50b: The permanent "connives" regardless of whether
            // actions were possible. Emit Connived event for trigger support.
            events.push(GameEvent::Connived {
                object_id: creature_id,
                player: controller,
                counters_placed: if state
                    .objects
                    .get(&creature_id)
                    .map(|o| o.zone == ZoneId::Battlefield)
                    .unwrap_or(false)
                {
                    nonland_count
                } else {
                    0
                },
            });
        }
    }
}
```

**Key design decisions**:
1. Cannot reuse `discard_cards` helper because we need to inspect each discarded card's
   types before/during the discard to count nonland cards. The helper does not return this
   information.
2. The Madness interaction in the discard loop must be replicated from `discard_cards`. The
   implementer should extract the madness-trigger logic or copy the relevant portion.
3. The `counters_placed` field in the Connived event reflects actual counters placed (0 if
   creature left the battlefield), for informational purposes. The event itself always fires
   per 701.50b.

**Imports needed**: `CounterType` is already imported via `crate::state::types`. Verify that
`CardType::Land` is accessible from the existing `use crate::state::types::CardType;` at
line 36.

### Step 4: TriggerEvent::SourceConnives (OPTIONAL, deferred)

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/game_object.rs`
**Action**: Add `SourceConnives` variant to `TriggerEvent` enum (around line 186).
**Note**: This enables "whenever [this creature] connives" triggers (e.g., Ledger Shredder
rulings). For the initial implementation, this is optional. The `Connived` event is emitted
regardless, and trigger wiring can be added in a follow-up.

If added:
```rust
/// CR 701.50b: Triggers when the source permanent connives.
/// Used by cards like "Whenever this creature connives, [effect]."
SourceConnives,
```

And wire it in `abilities.rs` `check_triggers` function, in the GameEvent match, similar to
`Surveilled` at line 1426:
```rust
GameEvent::Connived { object_id, player, .. } => {
    // CR 701.50b: "Whenever [this creature] connives" triggers on the
    // conniving permanent itself.
    collect_triggers_for_event(
        state,
        &mut triggers,
        TriggerEvent::SourceConnives,
        Some(*object_id),
        None,
    );
}
```

Also add to hash.rs TriggerEvent match and stubs.rs stub arms.

**Decision**: Implement this trigger wiring as part of Step 3 since it is straightforward
and follows the exact pattern of `ControllerSurveils`. Without it, "whenever this creature
connives" cards like Ledger Shredder cannot function.

### Step 5: Unit Tests

**File**: `/home/airbaggie/scutemob/crates/engine/tests/connive.rs`
**Tests to write**:

1. **`test_connive_basic_nonland_discard_adds_counter`** -- CR 701.50a
   - Setup: creature on battlefield, 5 cards in library (nonland), 0 in hand
   - Cast a sorcery with `Effect::Connive { target: Source, count: Fixed(1) }`
   - After resolution: creature has 1 +1/+1 counter, hand size net 0 (drew 1, discarded 1)
   - Verify `Connived` event emitted

2. **`test_connive_land_discard_no_counter`** -- CR 701.50a
   - Setup: creature on battlefield, 1 land card in hand, empty library except 1 nonland
   - Cast connive spell; draw the nonland, then discard the land (deterministic: min ObjectId)
   - After resolution: creature has 0 +1/+1 counters
   - Verify `Connived` event emitted with `counters_placed: 0`

3. **`test_connive_n_multiple_draws_discards`** -- CR 701.50e
   - Setup: creature on battlefield, 5 cards in library (mix of lands and nonlands)
   - Cast `Effect::Connive { target: Source, count: Fixed(3) }`
   - After resolution: creature has counters equal to number of nonland cards discarded

4. **`test_connive_empty_hand_no_library_still_connives`** -- CR 701.50b
   - Setup: creature on battlefield, empty library, empty hand
   - Cast connive spell (creature already on battlefield from setup, spell from a different zone)
   - After resolution: creature has 0 counters, `Connived` event still fires

5. **`test_connive_creature_left_battlefield_no_counter`** -- CR 701.50c
   - Setup: Use a sequence effect that destroys the creature, then connives it
   - After resolution: controller drew/discarded but no counter placed (creature is in graveyard)
   - `Connived` event fires with `counters_placed: 0`

6. **`test_connive_etb_trigger_on_creature`** -- Raffine's Informant pattern
   - Setup: creature with ETB trigger that causes it to connive (use `SelfEntersBattlefield` trigger with `Effect::Connive { target: Source, count: Fixed(1) }`)
   - After entering: creature connives via ETB trigger resolution
   - Verify: creature has 1 +1/+1 counter if nonland was discarded

**Pattern**: Follow tests in `/home/airbaggie/scutemob/crates/engine/tests/surveil.rs`.
Use the same helper functions (`find_object`, `pass_all`, `count_in_zone`, `find_by_name_in_zone`).
Use `ObjectSpec::creature()` for the conniving creature (not `ObjectSpec::card().with_types`).
Use synthetic sorcery card definitions with `Effect::Connive`.

### Step 6: Card Definition (later phase)

**Suggested card**: Raffine's Informant
- Oracle text: "When this creature enters, it connives."
- Type: Creature -- Human Wizard
- Cost: {1}{W}
- P/T: 2/1
- Simple ETB connive, perfect for validation

**Card lookup data** (from MCP):
```
Name: Raffine's Informant
Mana Cost: {1}{W}
Type: Creature -- Human Wizard
Oracle Text: When this creature enters, it connives. (Draw a card, then discard a card.
If you discarded a nonland card, put a +1/+1 counter on this creature.)
P/T: 2/1
Color Identity: ["W"]
```

**CardDefinition sketch**:
```rust
CardDefinition {
    card_id: CardId("raffines-informant".to_string()),
    name: "Raffine's Informant".to_string(),
    mana_cost: Some(ManaCost { white: 1, generic: 1, ..Default::default() }),
    types: TypeLine {
        card_types: [CardType::Creature].into_iter().collect(),
        subtypes: [SubType::Human, SubType::Wizard].into_iter().collect(),
        ..Default::default()
    },
    oracle_text: "When this creature enters, it connives.".to_string(),
    power: Some(2),
    toughness: Some(1),
    color_identity: vec![Color::White],
    abilities: vec![
        AbilityDefinition::Triggered(TriggeredAbilityDef {
            trigger_on: TriggerEvent::SelfEntersBattlefield,
            intervening_if: None,
            description: "When this creature enters, it connives. (CR 701.50a)".to_string(),
            effect: Some(Effect::Connive {
                target: EffectTarget::Source,
                count: EffectAmount::Fixed(1),
            }),
        }),
    ],
    ..Default::default()
}
```

### Step 7: Game Script (later phase)

**Suggested scenario**: Raffine's Informant ETB connive
**Subsystem directory**: `test-data/generated-scripts/stack/`
**Scenario**:
1. P1 has Raffine's Informant in hand, 2 nonland cards in library
2. P1 casts Raffine's Informant
3. All players pass priority; spell resolves
4. ETB trigger goes on stack ("it connives")
5. All players pass priority; trigger resolves
6. P1 draws 1, discards 1 (deterministic: lowest ObjectId in hand)
7. Assert: if discarded card was nonland, creature has 1 +1/+1 counter

## Interactions to Watch

- **Madness (CR 702.35a)**: If the card discarded to connive has Madness, it goes to exile
  instead of graveyard, and the madness trigger fires. The connive handler must replicate
  the madness check from `discard_cards`. The discarded card is still checked for nonland
  status before the zone move (card types are available in hand).

- **Dredge replacement**: If the draw from connive is replaced by dredge, the connive
  still happens (the dredge replacement fires on the draw, and the discard follows normally).
  No special handling needed -- `draw_one_card` already checks dredge.

- **"Whenever this creature connives" triggers**: Cards like Ledger Shredder have triggers
  that fire when the creature connives. The `Connived` event (always emitted per 701.50b)
  enables these. The `SourceConnives` trigger event wiring is needed for this.

- **Creature leaving the battlefield mid-connive**: Per 701.50c, if the creature leaves
  before connive resolves, the controller still draws/discards but no counter is placed.
  The implementation must check `creature_id` is still on the battlefield before adding
  counters.

- **Empty hand after draw**: If the player draws a card but their hand was otherwise empty,
  they must discard. Deterministic fallback discards the card they just drew (min ObjectId).
  This is correct per the rules -- they draw, then discard.

- **Library empty (can't draw)**: Per 701.50b, the permanent still connives even if the
  draw was impossible. The `draw_one_card` function handles empty library by setting
  `has_lost = true` -- the connive handler should still proceed with the discard step
  and emit the `Connived` event.
