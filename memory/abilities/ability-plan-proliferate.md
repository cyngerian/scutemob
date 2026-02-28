# Ability Plan: Proliferate

**Generated**: 2026-02-28
**CR**: 701.34 (NOTE: ability-wip.md says 701.27, but 701.27 is Transform; the correct CR for Proliferate is 701.34)
**Priority**: P3
**Similar abilities studied**: Surveil (Effect::Surveil in `crates/engine/src/effects/mod.rs:1140`), Scry (Effect::Scry in `crates/engine/src/effects/mod.rs:1109`), AddCounter (Effect::AddCounter in `crates/engine/src/effects/mod.rs:920`)

## CR Rule Text

**701.34. Proliferate**

> 701.34a To proliferate means to choose any number of permanents and/or players that have a counter, then give each one additional counter of each kind that permanent or player already has.

> 701.34b In a Two-Headed Giant game, poison counters are shared by the team. If more than one player on a team is chosen this way, only one of those players can be given an additional poison counter. The player who proliferates chooses which player that is. See rule 810, "Two-Headed Giant Variant."

## Key Edge Cases

From CR and card rulings (Contagion Clasp, Spread the Sickness, Vat Emergence):

1. **"Any number" includes zero** -- you can choose 0 permanents and/or 0 players (ruling 2023-02-04). The simplified "auto-select all" implementation should still emit a `Proliferated` event even when there are no eligible targets (so "whenever you proliferate" triggers fire).

2. **Only battlefield permanents, not cards in other zones** -- "You can't choose cards in any zone other than the battlefield, even if they have counters on them" (ruling 2023-02-04). This means the implementation must only iterate objects in `ZoneId::Battlefield`.

3. **All counter types on a chosen permanent/player must be incremented** -- "If a permanent has more than one kind of counter on it, and you choose for it to get additional counters, it must get one of each kind of counter it already has" (ruling 2023-02-04). The simplified "auto-select all" implementation naturally satisfies this since we add one of each kind.

4. **+1/+1 and -1/-1 cancellation via SBAs** -- "If a permanent ever has both +1/+1 counters and -1/-1 counters on it at the same time, they're removed in pairs as a state-based action" (ruling 2023-02-04). This is already handled by the engine's SBA system; proliferating both types just triggers the existing SBA.

5. **"Whenever you proliferate" triggers fire even if no permanents/players chosen** -- "An ability that triggers 'Whenever you proliferate' triggers even if you chose no permanents or players while doing so" (rulings on Vat Emergence, Core Prowler, etc., 2023-02-04). The `Proliferated` event must ALWAYS be emitted when proliferate resolves.

6. **Proliferate requires the spell/ability to resolve** -- "If each target chosen is an illegal target as that spell or ability tries to resolve, it won't resolve. You won't proliferate" (rulings on Vexing Radgull, etc., 2024-03-08). This is handled by normal spell resolution; proliferate only fires as part of effect execution.

7. **Player counters: poison counters** -- Players can have poison counters (the only player counter type in the engine). When a player has poison counters, proliferate adds 1 more poison counter. Note: `PlayerState.poison_counters` is a dedicated `u32` field, not in the `counters` OrdMap (which is on `GameObject` only).

8. **Multiplayer (Commander)** -- All players and all battlefield permanents are candidates, regardless of controller. Opponents' permanents with counters are eligible. Multiple opponents can be proliferated in a single action.

9. **Two-Headed Giant exception (CR 701.34b)** -- Not applicable to Commander format. Deferred.

## Current State (from ability-wip.md)

- [ ] Step 1: Enum variant / keyword action
- [ ] Step 2: Rule enforcement
- [ ] Step 3: Trigger wiring
- [ ] Step 4: Unit tests
- [ ] Step 5: Card definition
- [ ] Step 6: Game script
- [ ] Step 7: Coverage doc update

## Implementation Steps

### Step 1: Add Effect::Proliferate Variant

**File**: `/home/airbaggie/scutemob/crates/engine/src/cards/card_definition.rs`
**Action**: Add `Effect::Proliferate` variant to the `Effect` enum (around line 357, in the Counters section after `RemoveCounter`).

```rust
/// CR 701.34a: Proliferate -- choose any number of permanents and/or players
/// that have a counter, then give each one additional counter of each kind
/// that permanent or player already has.
///
/// Simplified implementation: auto-selects all eligible permanents on the
/// battlefield and all players with counters (controller "chooses all").
/// Interactive selection deferred to M10+.
///
/// Always emits a Proliferated event (even with 0 eligible targets) to
/// support "whenever you proliferate" triggers (ruling 2023-02-04).
Proliferate,
```

**Pattern**: Similar to `Effect::Nothing` -- no parameters needed. The controller who executes the effect is from `EffectContext.controller`.

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/hash.rs`
**Action**: Add hash arm for `Effect::Proliferate` at the end of the `HashInto for Effect` impl (after discriminant 37 for Regenerate, line ~2583). Use discriminant **38**.

```rust
// CR 701.34a: Proliferate (discriminant 38)
Effect::Proliferate => {
    38u8.hash_into(hasher);
}
```

**Match arms to update**: The `Effect` enum is exhaustively matched in exactly 2 files:
- `crates/engine/src/effects/mod.rs` (execute_effect) -- covered in Step 2
- `crates/engine/src/state/hash.rs` (HashInto) -- covered above

### Step 2: Effect Execution (Rule Enforcement)

**File**: `/home/airbaggie/scutemob/crates/engine/src/effects/mod.rs`
**Action**: Add the `Effect::Proliferate` handler arm in `execute_effect()`, in the Counters section (after the `Effect::RemoveCounter` handler, around line 969).

**CR**: 701.34a -- "choose any number of permanents and/or players that have a counter, then give each one additional counter of each kind"

**Implementation**:

```rust
// CR 701.34a: Proliferate -- add one counter of each kind to each
// permanent on the battlefield and each player that already has counters.
// Simplified: auto-select all eligible (interactive choice deferred to M10+).
Effect::Proliferate => {
    let controller = ctx.controller;

    // 1. Iterate all permanents on the battlefield with at least one counter.
    //    CR ruling 2023-02-04: "You can't choose cards in any zone other
    //    than the battlefield, even if they have counters on them."
    let battlefield_objects: Vec<(ObjectId, Vec<(CounterType, u32)>)> = state
        .objects
        .iter()
        .filter(|(_, obj)| obj.zone == ZoneId::Battlefield && !obj.counters.is_empty())
        .map(|(id, obj)| {
            let counter_types: Vec<(CounterType, u32)> = obj
                .counters
                .iter()
                .map(|(ct, &count)| (ct.clone(), count))
                .collect();
            (*id, counter_types)
        })
        .collect();

    // Add one counter of each kind to each eligible permanent.
    for (obj_id, counter_types) in &battlefield_objects {
        if let Some(obj) = state.objects.get_mut(obj_id) {
            for (counter_type, _) in counter_types {
                let cur = obj.counters.get(counter_type).copied().unwrap_or(0);
                obj.counters.insert(counter_type.clone(), cur + 1);
                events.push(GameEvent::CounterAdded {
                    object_id: *obj_id,
                    counter: counter_type.clone(),
                    count: 1,
                });
            }
        }
    }

    // 2. Iterate all players with poison counters (the only player counter type).
    //    CR 701.34a: "players that have a counter" -- poison_counters > 0.
    let eligible_players: Vec<PlayerId> = state
        .players
        .iter()
        .filter(|(_, ps)| !ps.has_lost && ps.poison_counters > 0)
        .map(|(id, _)| *id)
        .collect();

    for pid in &eligible_players {
        if let Some(player) = state.players.get_mut(pid) {
            player.poison_counters += 1;
            events.push(GameEvent::PoisonCountersGiven {
                player: *pid,
                amount: 1,
                source: ctx.source,
            });
        }
    }

    // 3. Always emit Proliferated event (ruling 2023-02-04:
    //    "triggers even if you chose no permanents or players").
    events.push(GameEvent::Proliferated {
        controller,
        permanents_affected: battlefield_objects.len() as u32,
        players_affected: eligible_players.len() as u32,
    });
}
```

**Important notes**:
- The `CounterType` imports should already be in scope (used by AddCounter handler).
- The `ZoneId::Battlefield` check ensures we only affect permanents, not cards in graveyard/exile/library (CR ruling).
- Reusing `GameEvent::CounterAdded` for each individual counter addition keeps existing counter triggers working.
- Reusing `GameEvent::PoisonCountersGiven` for poison counter increments keeps existing SBA checks working.

### Step 3: Add GameEvent::Proliferated

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/events.rs`
**Action**: Add `Proliferated` variant after the `Regenerated` event (around line 825, before the closing `}` of the enum).

```rust
// -- Proliferate event (CR 701.34) ------------------------------------------
/// CR 701.34a: A player proliferated.
///
/// Emitted by `Effect::Proliferate` after all counters have been added.
/// Always emitted, even if no permanents or players were chosen (ruling
/// 2023-02-04: "triggers even if you chose no permanents or players").
/// Enables "whenever you proliferate" triggers.
Proliferated {
    /// The player who performed the proliferate action.
    controller: PlayerId,
    /// Number of permanents that received additional counters.
    permanents_affected: u32,
    /// Number of players that received additional counters.
    players_affected: u32,
},
```

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/hash.rs`
**Action**: Add hash arm for `GameEvent::Proliferated` in the `HashInto for GameEvent` impl (after discriminant 84 for Regenerated, line ~2026). Use discriminant **85**.

```rust
// CR 701.34a: Proliferated (discriminant 85)
GameEvent::Proliferated {
    controller,
    permanents_affected,
    players_affected,
} => {
    85u8.hash_into(hasher);
    controller.hash_into(hasher);
    permanents_affected.hash_into(hasher);
    players_affected.hash_into(hasher);
}
```

### Step 4: Add TriggerEvent::ControllerProliferates (Future-Proofing)

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/game_object.rs`
**Action**: Add `ControllerProliferates` variant to `TriggerEvent` enum (after `ControllerInvestigates`, around line 195).

```rust
/// CR 701.34: Triggers when the controller of this permanent proliferates.
/// Used by "whenever you proliferate" cards (e.g., Core Prowler, Vat Emergence).
/// The controller match is done at trigger-collection time in `rules/abilities.rs`.
ControllerProliferates,
```

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/hash.rs`
**Action**: Add hash arm for `TriggerEvent::ControllerProliferates` with discriminant **17** (after ControllerInvestigates at 16, line ~1087).

```rust
// CR 701.34: ControllerProliferates trigger -- discriminant 17
TriggerEvent::ControllerProliferates => 17u8.hash_into(hasher),
```

### Step 5: Wire Proliferate Trigger in abilities.rs

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/abilities.rs`
**Action**: Add a `GameEvent::Proliferated` arm in the `check_triggers` function, before the wildcard `_ => {}` arm (around line 1542).

**Pattern**: Follow the `GameEvent::Surveilled` handler at line ~1437 which collects `TriggerEvent::ControllerSurveils` triggers.

```rust
GameEvent::Proliferated { controller, .. } => {
    // CR 701.34: "Whenever you proliferate" triggers for the proliferating player.
    // Iterate all permanents controlled by `controller` that have
    // ControllerProliferates triggers.
    for (obj_id, obj) in state.objects.iter() {
        if obj.zone == ZoneId::Battlefield
            && obj.controller == Some(*controller)
        {
            collect_triggers_for_event(
                state,
                &mut triggers,
                TriggerEvent::ControllerProliferates,
                Some(*obj_id),
                None,
            );
        }
    }
}
```

### Step 6: Unit Tests

**File**: `/home/airbaggie/scutemob/crates/engine/tests/proliferate.rs`
**Action**: Create new test file with comprehensive tests.

**Tests to write**:

1. **`test_proliferate_basic_plus_one_counter`** -- CR 701.34a. A creature with 2 +1/+1 counters gains 1 more after proliferate. Assert counters = 3 and CounterAdded event emitted.

2. **`test_proliferate_minus_one_counter`** -- CR 701.34a. A creature with 1 -1/-1 counter gains 1 more after proliferate. Assert counters = 2.

3. **`test_proliferate_loyalty_counter_on_planeswalker`** -- CR 701.34a. A planeswalker with loyalty counters (set via `with_counter(CounterType::Loyalty, 3)`) gains 1 more after proliferate. Assert counters = 4. (Note: need to build a planeswalker-typed object with CardType::Planeswalker.)

4. **`test_proliferate_charge_counter_on_artifact`** -- CR 701.34a. An artifact with 1 charge counter gains 1 more. Assert counters = 2.

5. **`test_proliferate_poison_counter_on_player`** -- CR 701.34a. A player with 5 poison counters gets 1 more after proliferate. Assert `player.poison_counters == 6` and PoisonCountersGiven event emitted.

6. **`test_proliferate_multiple_counter_types`** -- CR ruling 2023-02-04. A permanent with both +1/+1 (2) and charge (1) counters gets 1 of each. Assert +1/+1 = 3, charge = 2.

7. **`test_proliferate_multiple_targets`** -- CR 701.34a. Two creatures with counters and one player with poison counters all get proliferated in one action. Assert all three gain counters.

8. **`test_proliferate_no_counters_noop`** -- CR 701.34a. A creature with no counters is NOT affected. A player with 0 poison counters is NOT affected. Only the `Proliferated` event is emitted.

9. **`test_proliferate_event_always_emitted`** -- Ruling 2023-02-04. When there are no eligible targets at all (no permanents with counters, no players with poison), the `Proliferated` event with permanents_affected=0, players_affected=0 is still emitted.

10. **`test_proliferate_ignores_non_battlefield`** -- Ruling 2023-02-04. A card in the graveyard with counters (setup via builder placing object in graveyard with counters) is NOT proliferated. Only battlefield permanents count.

11. **`test_proliferate_multiplayer_affects_all_players`** -- Multiplayer (4 players). Multiple players with poison counters all receive +1. Multiple opponents' creatures with counters all receive +1.

**Pattern**: Follow the Surveil test structure in `/home/airbaggie/scutemob/crates/engine/tests/surveil.rs`:
- Use `GameStateBuilder` with 2+ players
- Create a simple "Proliferate Spell" sorcery CardDefinition with `Effect::Proliferate`
- Register it in `CardRegistry::new(vec![def])`
- Cast it with `CastSpell`, pass priority to resolve, check state
- For counter setup, use `ObjectSpec::creature(owner, name, power, toughness).with_counter(CounterType::X, N).in_zone(ZoneId::Battlefield)`
- For poison setup, use `GameStateBuilder::player_poison(player, N)`
- For graveyard-counter test, use `.with_counter(CounterType::X, N).in_zone(ZoneId::Graveyard(owner))`

**Key helper definitions for the test file**:

```rust
/// Build a "Proliferate" sorcery card definition.
fn proliferate_spell_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("proliferate-spell".to_string()),
        name: "Proliferate Spell".to_string(),
        mana_cost: Some(ManaCost { generic: 1, ..Default::default() }),
        types: TypeLine {
            card_types: [CardType::Sorcery].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Proliferate.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::Proliferate,
            targets: vec![],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}
```

### Step 7: Card Definition (later phase)

**Suggested card**: Spread the Sickness
- **Oracle**: "Destroy target creature, then proliferate."
- **Cost**: {4}{B}
- **Type**: Sorcery
- **Effect**: `Effect::Sequence(vec![Effect::DestroyPermanent { target: EffectTarget::DeclaredTarget { index: 0 } }, Effect::Proliferate])`
- **Targets**: `vec![EffectTargetSpec { filter: TargetFilter { types: Some(vec![CardType::Creature]), ..Default::default() }, count: 1 }]`
- **Why this card**: Simpler than Contagion Clasp (no activated ability, no ETB trigger -- just a targeted sorcery). Tests the common pattern of "do something, then proliferate" as a Sequence.
- **Ruling**: "If the creature regenerates or has indestructible when Spread the Sickness resolves, you'll still proliferate." (Spread the Sickness 2013-07-01). This works naturally with Sequence -- DestroyPermanent fires (may be prevented by indestructible), then Proliferate fires unconditionally.
- **Alternative**: Contagion Clasp ({2} Artifact; ETB: -1/-1 counter on target creature; {4},{T}: Proliferate) for a more comprehensive test of activated abilities + proliferate.
- **Card lookup**: Use `card-definition-author` agent.

### Step 8: Game Script (later phase)

**Suggested scenario**: "Proliferate adds counters to multiple permanents and players in a 4-player game"
- Player A casts Spread the Sickness targeting Player B's creature (destroying it).
- Player A controls a creature with 2 +1/+1 counters.
- Player C controls an artifact with 1 charge counter.
- Player B has 3 poison counters.
- After Spread the Sickness resolves: creature destroyed, then proliferate fires.
- Assert: Player A's creature now has 3 +1/+1 counters, Player C's artifact has 2 charge counters, Player B now has 4 poison counters.
**Subsystem directory**: `test-data/generated-scripts/stack/` (proliferate as a stack-resolving effect)

## Interactions to Watch

1. **+1/+1 and -1/-1 counter cancellation (SBA CR 704.5q)**: After proliferate adds both types to a single permanent, SBAs will cancel them in pairs. The test should verify this happens naturally. The existing SBA implementation already handles this.

2. **Poison counter SBA (CR 704.5c)**: If proliferate pushes a player to 10+ poison counters, the SBA should cause them to lose. This is already handled by the existing SBA system.

3. **Infect damage vs. proliferate**: Proliferate does NOT deal damage. It simply adds counters. The `PoisonCountersGiven` event is reused for the event, but it does not interact with damage prevention or protection.

4. **Shield counters (CR 701.46)**: If a permanent has shield counters, proliferate adds 1 more. Shield counters are removed instead of the permanent being destroyed (handled by existing shield counter logic in SBAs).

5. **Planeswalker loyalty**: Proliferating a planeswalker adds a loyalty counter. This is equivalent to a +1 loyalty activation but does NOT count as activating a loyalty ability (no "once per turn" restriction).

6. **Energy counters on players**: The engine's `PlayerState` does not have an `energy_counters` field -- energy is NOT tracked as a player counter in the current implementation. If energy counters are added later, the proliferate implementation will need to be updated to also iterate player energy. For now, only `poison_counters` is relevant.

7. **"Whenever you proliferate" trigger timing**: The `Proliferated` event is emitted AFTER all counter additions are complete. Triggers from the event go on the stack after the spell/ability finishes resolving (standard trigger timing). The individual `CounterAdded` events from proliferate may also trigger "whenever a counter is placed" abilities -- these are separate triggers.

## Files Modified Summary

| File | Change |
|------|--------|
| `crates/engine/src/cards/card_definition.rs` ~L370 | Add `Effect::Proliferate` variant (no params) |
| `crates/engine/src/effects/mod.rs` ~L969 | Add `Effect::Proliferate` handler arm |
| `crates/engine/src/rules/events.rs` ~L825 | Add `GameEvent::Proliferated` variant |
| `crates/engine/src/state/game_object.rs` ~L195 | Add `TriggerEvent::ControllerProliferates` variant |
| `crates/engine/src/state/hash.rs` ~L2583 | Effect discriminant 38 |
| `crates/engine/src/state/hash.rs` ~L2026 | GameEvent discriminant 85 |
| `crates/engine/src/state/hash.rs` ~L1087 | TriggerEvent discriminant 17 |
| `crates/engine/src/rules/abilities.rs` ~L1542 | Add `Proliferated` arm in `check_triggers` |
| `crates/engine/tests/proliferate.rs` (NEW) | 11 unit tests |
