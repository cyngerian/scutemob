# Ability Plan: Surveil

**Generated**: 2026-02-26
**CR**: 701.25
**Priority**: P2
**Similar abilities studied**: Scry (Effect::Scry in `crates/engine/src/cards/card_definition.rs:289`, execution in `crates/engine/src/effects/mod.rs:886-915`, hash in `crates/engine/src/state/hash.rs:2226-2231`, event in `crates/engine/src/rules/events.rs:670-676`, test in `crates/engine/tests/card_def_fixes.rs:48-141`)

## CR Rule Text

701.25. Surveil

> [701.25a] To "surveil N" means to look at the top N cards of your library, then put any number of them into your graveyard and the rest on top of your library in any order.

> [701.25b] If an effect allows you to look at additional cards while you surveil, those cards are included among the cards you may put into your graveyard and on top of your library in any order.

> [701.25c] If a player is instructed to surveil 0, no surveil event occurs. Abilities that trigger whenever a player surveils won't trigger.

> [701.25d] An ability that triggers "whenever you surveil" triggers after the process described in rule 701.25a is complete, even if some or all of those actions were impossible.

## Key Edge Cases

- **Surveil 0 does nothing (CR 701.25c)**: No surveil event occurs. "Whenever you surveil" triggers do NOT fire. Mirrors Scry 0 behavior (CR 701.22b).
- **Empty library / fewer than N cards (CR 701.25d)**: If the library has fewer than N cards, the player looks at whatever cards are available. The surveil event still occurs (triggers fire) even if some actions were impossible. Only surveil 0 suppresses the event.
- **Surveil is a keyword action, not a keyword ability**: It does NOT grant a permanent a static ability. Cards say "Surveil N" as part of a spell effect or triggered/activated ability effect. It is analogous to Scry, not to Flying.
- **"Whenever you surveil" triggers (CR 701.25d)**: These are triggered abilities on other permanents (e.g., Dimir Spybug). The trigger fires once per surveil action regardless of how many cards are put into the graveyard. It fires after the surveil is complete.
- **Ordering of remaining cards**: Cards not put into the graveyard go on top of the library in any order. The deterministic fallback (like Scry) will use ObjectId ascending order for the cards that remain on top.
- **Fizzled spells do not surveil**: If a spell with surveil fizzles (all targets illegal), the spell does not resolve and surveil does not happen. (Ruling on Sinister Sabotage.)
- **Surveil still happens even if other parts of the spell had no effect**: If a spell counters a target and surveils, the surveil still happens even if the target was uncounterable. (Ruling on Sinister Sabotage.)
- **Multiplayer**: Surveil is always performed by a single player. No APNAP ordering issues for the surveil action itself (unlike simultaneous scry, CR 701.22c). However, "whenever you surveil" triggers on multiple permanents follow APNAP ordering when placed on the stack.

## Current State (from ability-wip.md)

- [ ] Step 1: Enum variant (Effect::Surveil + GameEvent::Surveilled)
- [ ] Step 2: Rule enforcement (effect execution in effects/mod.rs)
- [ ] Step 3: Trigger wiring (TriggerCondition::WheneverYouSurveil + TriggerEvent + check_triggers + enrich_spec_from_def)
- [ ] Step 4: Unit tests
- [ ] Step 5: Card definition
- [ ] Step 6: Game script

## Implementation Steps

### Step 1: Effect Variant + Event

Surveil is a keyword action (like Scry), not a keyword ability (like Flying). It does NOT need a `KeywordAbility::Surveil` variant. It needs:

1. An `Effect::Surveil` variant
2. A `GameEvent::Surveilled` event

#### Step 1a: Effect::Surveil variant

**File**: `/home/airbaggie/scutemob/crates/engine/src/cards/card_definition.rs`
**Action**: Add `Surveil` variant to the `Effect` enum, right after `Scry` (line ~292).
**Pattern**: Follow `Effect::Scry` at line 289.

```rust
/// CR 701.25: Surveil N -- look at the top N cards of your library, then put
/// any number of them into your graveyard and the rest on top in any order.
///
/// Deterministic fallback: puts ALL top N cards into the graveyard
/// (interactive ordering deferred to M10+). This mirrors the Scry fallback
/// but sends cards to the graveyard instead of the bottom of the library.
Surveil {
    player: PlayerTarget,
    count: EffectAmount,
},
```

**Fields**: Identical to `Effect::Scry` -- `player: PlayerTarget` and `count: EffectAmount`.

#### Step 1b: GameEvent::Surveilled variant

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/events.rs`
**Action**: Add `Surveilled` variant to the `GameEvent` enum, after `Scried` (line ~676).
**Pattern**: Follow `GameEvent::Scried` at line 676.

```rust
// ── Surveil event ───────────────────────────────────────────────────
/// A player performed a surveil action (CR 701.25).
///
/// Emitted by `Effect::Surveil` when the player looks at the top N cards
/// of their library and puts some into the graveyard, rest on top.
/// CR 701.25c: NOT emitted when surveilling 0.
Surveilled { player: PlayerId, count: u32 },
```

#### Step 1c: Hash for Effect::Surveil

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/hash.rs`
**Action**: Add hash arm for `Effect::Surveil` using discriminant **32** (next available after SacrificePermanents=31).
**Location**: After the `Effect::AttachEquipment` arm (line ~2248), before the closing `}`.

```rust
// CR 701.25: Surveil (discriminant 32)
Effect::Surveil { player, count } => {
    32u8.hash_into(hasher);
    player.hash_into(hasher);
    count.hash_into(hasher);
}
```

#### Step 1d: Hash for GameEvent::Surveilled

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/hash.rs`
**Action**: Add hash arm for `GameEvent::Surveilled` using discriminant **75** (next available after AuraAttached=74).
**Location**: After the `GameEvent::AuraAttached` arm (line ~1733), before the closing `}`.

```rust
// CR 701.25: Surveilled (discriminant 75)
GameEvent::Surveilled { player, count } => {
    75u8.hash_into(hasher);
    player.hash_into(hasher);
    count.hash_into(hasher);
}
```

### Step 2: Rule Enforcement (Effect Execution)

**File**: `/home/airbaggie/scutemob/crates/engine/src/effects/mod.rs`
**Action**: Add execution arm for `Effect::Surveil` in the main `match effect` block.
**Location**: After the `Effect::Scry` arm (line ~915), before `Effect::Shuffle`.
**Pattern**: Follow `Effect::Scry` implementation at lines 886-915.
**CR**: 701.25a -- put any number into graveyard, rest on top in any order.
**CR**: 701.25c -- surveil 0 produces no event.

The implementation differs from Scry in ONE critical way: cards go to the **graveyard** instead of the **bottom of the library**.

Deterministic fallback strategy: put ALL looked-at cards into the graveyard (the maximally aggressive strategy for a deterministic fallback -- mirrors how Scry's fallback puts all cards on the bottom). Interactive card selection is deferred to M10+.

```rust
// CR 701.25: Surveil N -- deterministic fallback: put top N cards into graveyard
// (interactive selection deferred to M10+).
Effect::Surveil { player, count } => {
    let n = resolve_amount(state, count, ctx).max(0) as usize;
    let players = resolve_player_target_list(state, player, ctx);
    for p in players {
        // CR 701.25c: surveil 0 produces no event
        if n == 0 {
            continue;
        }
        let lib_zone = ZoneId::Library(p);
        let graveyard_zone = ZoneId::Graveyard(p);
        // Collect the top N cards of the library (ordered from top).
        let top_ids: Vec<ObjectId> = state
            .zones
            .get(&lib_zone)
            .map(|z| z.object_ids())
            .unwrap_or_default()
            .into_iter()
            .take(n)
            .collect();
        // Deterministic fallback: move all looked-at cards to graveyard.
        // Sort by ObjectId ascending for determinism.
        let mut to_graveyard = top_ids.clone();
        to_graveyard.sort_by_key(|id| id.0);
        let actual_count = to_graveyard.len();
        for id in to_graveyard {
            let _ = state.move_object_to_zone(id, graveyard_zone);
        }
        // CR 701.25d: event fires even if some actions were impossible
        // (e.g., library had fewer than N cards).
        events.push(GameEvent::Surveilled {
            player: p,
            count: actual_count as u32,
        });
    }
}
```

**Key differences from Scry**:
- Scry moves cards to `lib_zone` (bottom of library via re-insert)
- Surveil moves cards to `ZoneId::Graveyard(p)` via `move_object_to_zone`
- Surveil 0 check is explicit per CR 701.25c (no event emitted)
- Uses `actual_count` (cards actually moved) rather than `n` (requested count) in the event, for the case where library has fewer than N cards

**Important**: `move_object_to_zone` creates new ObjectIds for the moved cards (CR 400.7). This is correct -- the cards are new objects in the graveyard.

### Step 3: Trigger Wiring

Surveil needs "whenever you surveil" trigger support for cards like Dimir Spybug. This requires:

#### Step 3a: TriggerCondition variant

**File**: `/home/airbaggie/scutemob/crates/engine/src/cards/card_definition.rs`
**Action**: Add `WheneverYouSurveil` to the `TriggerCondition` enum (line ~566, before the closing `}`).

```rust
/// "Whenever you surveil" (CR 701.25d).
WheneverYouSurveil,
```

#### Step 3b: TriggerEvent variant

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/game_object.rs`
**Action**: Add `ControllerSurveils` to the `TriggerEvent` enum (line ~146, before the closing `}`).
**Pattern**: Follow `ControllerCastsNoncreatureSpell` at line 126.

```rust
/// CR 701.25d: Triggers when the controller of this permanent surveils.
/// Used by Dimir Spybug and similar "whenever you surveil" cards.
/// The controller match is done at trigger-collection time in `rules/abilities.rs`.
ControllerSurveils,
```

#### Step 3c: check_triggers dispatch

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/abilities.rs`
**Action**: Add a `GameEvent::Surveilled` arm to the `check_triggers` match block.
**Location**: After the existing event arms (find a good spot -- after `CardCycled` or similar).
**Pattern**: Follow the `SpellCast` -> `ControllerCastsNoncreatureSpell` pattern (lines 567-613) but simpler -- no type filtering needed.

```rust
GameEvent::Surveilled { player, .. } => {
    // CR 701.25d: "Whenever you surveil" triggers on all permanents
    // controlled by the surveilling player.
    let controller_sources: Vec<ObjectId> = state
        .objects
        .values()
        .filter(|obj| obj.zone == ZoneId::Battlefield && obj.controller == *player)
        .map(|obj| obj.id)
        .collect();

    for obj_id in controller_sources {
        collect_triggers_for_event(
            state,
            &mut triggers,
            TriggerEvent::ControllerSurveils,
            Some(obj_id),
            None,
        );
    }
}
```

#### Step 3d: enrich_spec_from_def wiring

**File**: `/home/airbaggie/scutemob/crates/engine/src/testing/replay_harness.rs`
**Action**: Add a `TriggerCondition::WheneverYouSurveil` enrichment block in `enrich_spec_from_def`.
**Location**: After the `WheneverOpponentCastsSpell` block (line ~633), before the `spec` return.
**Pattern**: Follow the `WheneverOpponentCastsSpell` block at lines 619-633.

```rust
// CR 701.25d: Convert "Whenever you surveil" card-definition triggers into
// runtime TriggeredAbilityDef entries so check_triggers can dispatch them
// via Surveilled events.
for ability in &def.abilities {
    if let AbilityDefinition::Triggered {
        trigger_condition: TriggerCondition::WheneverYouSurveil,
        effect,
        ..
    } = ability
    {
        spec = spec.with_triggered_ability(TriggeredAbilityDef {
            trigger_on: TriggerEvent::ControllerSurveils,
            intervening_if: None,
            description: "Whenever you surveil (CR 701.25d)".to_string(),
            effect: Some(effect.clone()),
        });
    }
}
```

#### Step 3e: Hash for TriggerEvent::ControllerSurveils

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/hash.rs`
**Action**: Add arm to the `impl HashInto for TriggerEvent` block (line 938-960).
**Discriminant**: **12** (next available after `ControllerCreatureAttacksAlone` = 11 at line 958).
**Location**: After the `ControllerCreatureAttacksAlone` arm (line 958), before the closing `}`.

```rust
// CR 701.25d: Surveil trigger — discriminant 12
TriggerEvent::ControllerSurveils => 12u8.hash_into(hasher),
```

#### Step 3f: Hash for TriggerCondition::WheneverYouSurveil

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/hash.rs`
**Action**: Add arm to the `impl HashInto for TriggerCondition` block (line 1954-1988).
**Discriminant**: **18** (next available after `WhenBecomesTargetByOpponent` = 17 at line 1986).
**Location**: After the `WhenBecomesTargetByOpponent` arm (line 1986), before the closing `}`.

```rust
// CR 701.25d: "Whenever you surveil" — discriminant 18
TriggerCondition::WheneverYouSurveil => 18u8.hash_into(hasher),
```

### Step 4: Unit Tests

**File**: `/home/airbaggie/scutemob/crates/engine/tests/surveil.rs`
**Pattern**: Follow tests in `/home/airbaggie/scutemob/crates/engine/tests/card_def_fixes.rs:48-141` (Scry tests).

**Tests to write**:

1. **`test_surveil_basic_cards_go_to_graveyard`** (CR 701.25a)
   - Setup: player with 5 cards in library
   - Cast a spell with Surveil 2
   - Assert: 2 cards moved from library to graveyard (deterministic fallback: all go to graveyard)
   - Assert: `GameEvent::Surveilled { player, count: 2 }` emitted
   - Assert: library size decreased by 2, graveyard size increased by 2

2. **`test_surveil_zero_no_event`** (CR 701.25c)
   - Setup: player with cards in library
   - Execute `Effect::Surveil { count: Fixed(0) }`
   - Assert: no `GameEvent::Surveilled` event emitted
   - Assert: library and graveyard sizes unchanged

3. **`test_surveil_empty_library_still_emits_event`** (CR 701.25d)
   - Setup: player with 0 cards in library
   - Execute `Effect::Surveil { count: Fixed(2) }`
   - Assert: `GameEvent::Surveilled { player, count: 0 }` emitted
   - Assert: no crash/panic

4. **`test_surveil_library_fewer_than_n`** (CR 701.25d)
   - Setup: player with 1 card in library
   - Execute `Effect::Surveil { count: Fixed(3) }`
   - Assert: 1 card moved to graveyard
   - Assert: `GameEvent::Surveilled { player, count: 1 }` emitted (actual count, not requested)
   - Assert: library is now empty

5. **`test_surveil_then_draw_sequence`** (common card pattern: "Surveil 2, then draw a card")
   - Setup: player with 5 cards in library
   - Execute `Effect::Sequence([Surveil { count: 2 }, DrawCards { count: 1 }])`
   - Assert: Surveilled event precedes CardDrawn event
   - Assert: 2 cards in graveyard, 1 card drawn (from the 3rd card in the original library)

6. **`test_whenever_you_surveil_trigger`** (CR 701.25d)
   - Setup: player controls a creature with `TriggerCondition::WheneverYouSurveil` that puts a +1/+1 counter on itself
   - Execute a surveil effect
   - Assert: trigger fires and counter is placed
   - This validates the full trigger pipeline: event emission -> check_triggers -> TriggerEvent::ControllerSurveils -> trigger resolution

7. **`test_surveil_zero_does_not_fire_trigger`** (CR 701.25c)
   - Setup: same as test 6 but surveil 0
   - Assert: no trigger fires (no event emitted)

### Step 5: Card Definition (later phase)

**Suggested card**: Consider ({U}, Instant; "Surveil 1, draw a card.")
- Simple, single-color, low cost
- Uses Surveil as part of a sequence effect (surveil then draw)
- Good for testing the Effect::Surveil + sequence pattern

**Alternative**: Notion Rain ({1}{U}{B}, Sorcery; "Surveil 2, then draw two cards. Notion Rain deals 2 damage to you.")
- Tests surveil with higher N value
- Tests three-part sequence (surveil, draw, damage)

**Card lookup**: use `card-definition-author` agent

### Step 6: Game Script (later phase)

**Suggested scenario**: Cast Consider (Surveil 1 + draw 1) -- verify surveil puts a card in graveyard before the draw, and the draw takes the next card from the library.
**Subsystem directory**: `test-data/generated-scripts/stack/`

## Interactions to Watch

- **Surveil vs Scry**: Mechanically nearly identical except destination (graveyard vs bottom of library). Cards that say "if a card was put into your graveyard this turn" will count surveilled cards. Cards that say "if you've surveilled this turn" need the `Surveilled` event.
- **Object identity (CR 400.7)**: Cards moved from library to graveyard become new objects. The old ObjectIds from the library are dead after `move_object_to_zone`.
- **Hidden information**: Surveil looks at the top of the library (hidden zone). The `Surveilled` event is public (all players know you surveilled), but which specific cards were looked at is private. The engine currently doesn't enforce hidden information filtering (deferred to M10+ network layer), so this is noted but not blocking.
- **"Whenever you surveil" fires once per surveil action**: Not once per card. Dimir Spybug ruling: "You put only one +1/+1 counter on Dimir Spybug each time you surveil, no matter how many cards you looked at when you surveilled."
- **Multiplayer**: Surveil is always a single-player action. No APNAP ordering for the surveil itself. Multiple "whenever you surveil" triggers from different permanents are ordered by APNAP as usual when placed on the stack.
- **Dredge interaction**: If a card with Dredge is among the cards surveilled into the graveyard, that dredge card is now available for the next draw replacement. Not a direct interaction issue but worth testing eventually.

## Files Modified (Summary)

| File | Changes |
|------|---------|
| `/home/airbaggie/scutemob/crates/engine/src/cards/card_definition.rs` | `Effect::Surveil` variant + `TriggerCondition::WheneverYouSurveil` variant |
| `/home/airbaggie/scutemob/crates/engine/src/rules/events.rs` | `GameEvent::Surveilled` variant |
| `/home/airbaggie/scutemob/crates/engine/src/state/game_object.rs` | `TriggerEvent::ControllerSurveils` variant |
| `/home/airbaggie/scutemob/crates/engine/src/state/hash.rs` | 4 new hash arms: Effect(32), GameEvent(75), TriggerEvent(12), TriggerCondition(18) |
| `/home/airbaggie/scutemob/crates/engine/src/effects/mod.rs` | Effect execution logic (~30 lines) |
| `/home/airbaggie/scutemob/crates/engine/src/rules/abilities.rs` | `check_triggers` arm for `Surveilled` event (~15 lines) |
| `/home/airbaggie/scutemob/crates/engine/src/testing/replay_harness.rs` | `enrich_spec_from_def` block for `WheneverYouSurveil` (~15 lines) |
| `/home/airbaggie/scutemob/crates/engine/tests/surveil.rs` | 7 new tests |
