# Ability Plan: Forecast

**Generated**: 2026-03-06
**CR**: 702.57
**Priority**: P4
**Similar abilities studied**: Cycling (`rules/abilities.rs:408-560`, `command.rs:374-385`, `engine.rs:243-256`) — activated ability from hand zone with its own Command variant and handler

## CR Rule Text

702.57. Forecast

702.57a A forecast ability is a special kind of activated ability that can be activated only from a player's hand. It's written "Forecast -- [Activated ability]."

702.57b A forecast ability may be activated only during the upkeep step of the card's owner and only once each turn. The controller of the forecast ability reveals the card with that ability from their hand as the ability is activated. That player plays with that card revealed in their hand until it leaves the player's hand or until a step or phase that isn't an upkeep step begins, whichever comes first.

## Key Edge Cases

- **Only during owner's upkeep** (CR 702.57b): The card's owner must be the active player AND the current step must be `Step::Upkeep`. In multiplayer, this means only during the turn of the card's owner.
- **Once per turn** (CR 702.57b): Each forecast ability can be activated at most once per turn. This is per-card, not per-player (a player could theoretically forecast different cards in the same upkeep). Need per-card tracking.
- **Card stays in hand**: Unlike Cycling, the card is NOT discarded. It is revealed as part of activation but remains in hand. The "revealed" status lasts until the card leaves hand or the upkeep step ends.
- **The effect uses the stack**: Forecast is an activated ability — it goes on the stack and can be responded to (e.g., Stifle).
- **Effect is card-specific**: Each Forecast card has its own effect (draw a card, create a token, untap creatures, etc.). The effect is defined per-card in the `AbilityDefinition::Forecast` variant.
- **Split second blocks it** (CR 702.61a): Like all non-mana activated abilities, Forecast cannot be activated while a spell with split second is on the stack.
- **Priority required**: Must have priority to activate (standard activated ability rule, CR 602.2).
- **Multiplayer**: "Owner's upkeep" means the active player must be the card's owner. Other players cannot forecast during someone else's upkeep.
- **Reveal is informational**: The engine does not currently model "revealed cards in hand" as a distinct state. The reveal is a cosmetic/information event, not mechanically significant. We can emit a `GameEvent::CardRevealed` but do not need a persistent revealed state for engine correctness.
- **Sky Hussar ruling (2024-01-12)**: Tapping creatures for the forecast cost does not require summoning-sickness exemption (tapping creatures is an activation cost, not a `{T}` ability — same logic as Convoke).

## Current State (from ability-wip.md)

- [ ] Step 1: Enum variant
- [ ] Step 2: Rule enforcement
- [ ] Step 3: Trigger wiring (N/A — Forecast is an activated ability, not triggered)
- [ ] Step 4: Unit tests
- [ ] Step 5: Card definition
- [ ] Step 6: Game script

## Implementation Steps

### Step 1: Enum Variant & AbilityDefinition

**File**: `crates/engine/src/state/types.rs`
**Action**: Add `KeywordAbility::Forecast` variant after `CumulativeUpkeep` (line ~1075). This is a static marker for quick presence-checking.
**Pattern**: Follow `KeywordAbility::Cycling` at line 314 — bare keyword marker.
**Discriminant**: KW 117 (next available after Recover=116, Vanishing, Fading, Echo, CumulativeUpkeep).

```
/// CR 702.57a: Forecast -- activated ability from hand during owner's upkeep.
/// Static marker for quick presence-checking (`keywords.contains`).
/// The forecast cost and effect are stored in `AbilityDefinition::Forecast`.
Forecast,
```

**File**: `crates/engine/src/cards/card_definition.rs`
**Action**: Add `AbilityDefinition::Forecast` variant after `Recover` (line ~507). This stores the mana cost and effect.
**Discriminant**: AbilDef 46 (next after Recover=45).

```
/// CR 702.57: Forecast [cost], Reveal this card from your hand: [Effect].
/// Activated ability from hand, only during owner's upkeep, once per turn.
///
/// Cards with this ability should also include
/// `AbilityDefinition::Keyword(KeywordAbility::Forecast)` for quick
/// presence-checking without scanning all abilities.
Forecast { cost: ManaCost, effect: Effect },
```

**File**: `crates/engine/src/state/hash.rs`
**Action**: Add `KeywordAbility::Forecast` to the `HashInto` impl match arm. Follow the pattern of other bare keyword variants (just hash the discriminant).

**File**: `crates/engine/src/cards/helpers.rs`
**Action**: No changes needed — `ManaCost` and `Effect` are already exported.

### Step 2: Per-Turn Tracking

**File**: `crates/engine/src/state/mod.rs`
**Action**: Add a field to `GameState` to track which cards have used their forecast this turn:
```
/// CR 702.57b: Cards that have activated their forecast ability this turn.
/// Keyed by CardId (not ObjectId) since the card stays in hand and retains identity.
/// Reset at the start of each turn in `reset_turn_state`.
pub forecast_used_this_turn: im::OrdSet<CardId>,
```
**Pattern**: Follow `permanents_put_into_graveyard_this_turn` (line 133) — a per-turn counter/set on GameState.

**File**: `crates/engine/src/rules/turn_actions.rs`
**Action**: In `reset_turn_state` (line ~1014), add:
```
state.forecast_used_this_turn = im::OrdSet::new();
```
After line 1048 (after `permanents_put_into_graveyard_this_turn = 0`).

**File**: `crates/engine/src/state/builder.rs`
**Action**: Initialize `forecast_used_this_turn: im::OrdSet::new()` in `GameStateBuilder::build()` (near line 345 where `permanents_put_into_graveyard_this_turn: 0` is set).

**File**: `crates/engine/src/state/hash.rs`
**Action**: Add `self.forecast_used_this_turn.hash_into(hasher)` to the `GameState` HashInto impl, after the `permanents_put_into_graveyard_this_turn` line.

### Step 3: Command Variant

**File**: `crates/engine/src/rules/command.rs`
**Action**: Add `Command::ActivateForecast` variant. Place it near `CycleCard` (line ~385).

```
/// CR 702.57: Forecast -- activated ability from hand during owner's upkeep.
///
/// Unlike `CycleCard` (which discards as cost), the card stays in hand.
/// Unlike `ActivateAbility` (which requires the source on the battlefield),
/// this command works from the hand zone.
///
/// The effect is looked up from the card's `AbilityDefinition::Forecast`.
/// `targets` contains any targets for the forecast effect.
ActivateForecast {
    player: PlayerId,
    card: ObjectId,
    targets: Vec<Target>,
},
```

**File**: `crates/engine/src/rules/engine.rs`
**Action**: Add `Command::ActivateForecast` handler in `process_command`. Place it near the `CycleCard` handler (line ~256). Pattern:
```
Command::ActivateForecast { player, card, targets } => {
    validate_player_active(&state, player)?;
    loop_detection::reset_loop_detection(&mut state);
    let mut events = abilities::handle_activate_forecast(
        &mut state, player, card, targets,
    )?;
    let new_triggers = abilities::check_triggers(&state, &events);
    for t in new_triggers {
        state.pending_triggers.push_back(t);
    }
    let trigger_events = abilities::flush_pending_triggers(&mut state);
    events.extend(trigger_events);
    all_events.extend(events);
}
```

### Step 4: Rule Enforcement (Handler)

**File**: `crates/engine/src/rules/abilities.rs`
**Action**: Add `handle_activate_forecast` function. Place it after `handle_cycle_card` (line ~560).

The handler must:
1. **Priority check** (CR 602.2): `state.turn.priority_holder != Some(player)` -> error.
2. **Split second check** (CR 702.61a): `has_split_second_on_stack(state)` -> error.
3. **Zone check** (CR 702.57a): card must be in `ZoneId::Hand(player)`.
4. **Keyword check** (CR 702.57a): card must have `KeywordAbility::Forecast`.
5. **Upkeep check** (CR 702.57b): `state.turn.step != Step::Upkeep` -> error.
6. **Owner's upkeep check** (CR 702.57b): `state.turn.active_player != player` -> error. (The active player is the one whose upkeep it is. The card's owner must be the active player.)
7. **Once-per-turn check** (CR 702.57b): card's `CardId` must not be in `state.forecast_used_this_turn`.
8. **Look up cost and effect** from `AbilityDefinition::Forecast` in card registry.
9. **Pay mana cost** (CR 602.2b).
10. **Mark as used**: Add `card_id` to `state.forecast_used_this_turn`.
11. **Push ability on stack**: Create `StackObject` with `StackObjectKind::ForecastAbility { source_object: card, embedded_effect: Box::new(effect) }`.
12. **Reset priority** to active player (CR 602.2e).
13. **Emit events**: `GameEvent::AbilityActivated`, `GameEvent::PriorityGiven`.

**Pattern**: Follow `handle_cycle_card` (lines 408-560) for the overall structure — priority check, zone check, keyword check, cost lookup, mana payment, stack push, priority reset.

**Key difference from Cycling**: The card is NOT moved from hand. No discard event. The effect varies per card (not always "draw a card").

### Step 5: StackObjectKind Variant

**File**: `crates/engine/src/state/stack.rs`
**Action**: Add `StackObjectKind::ForecastAbility` variant after `RecoverTrigger` (line ~943).
**Discriminant**: SOK 43.

```
/// CR 702.57a: Forecast activated ability on the stack.
///
/// The source card remains in the player's hand. The effect is captured
/// at activation time from the card definition's `AbilityDefinition::Forecast`.
///
/// When this ability resolves, execute the embedded effect.
/// If the effect has targets, validate them at resolution time (CR 608.2b).
///
/// Discriminant 43.
ForecastAbility {
    source_object: ObjectId,
    embedded_effect: Box<crate::cards::card_definition::Effect>,
},
```

**File**: `crates/engine/src/rules/resolution.rs`
**Action**: Add resolution handler for `StackObjectKind::ForecastAbility`. When this resolves, execute the `embedded_effect` using the standard effect execution pipeline. The source card stays in hand — no zone move.
**Pattern**: Follow `StackObjectKind::ActivatedAbility` resolution path — execute the effect, check targets.

### Step 6: Match Arm Updates

The following files have exhaustive matches on `StackObjectKind` and need a new arm for `ForecastAbility`:

1. **`tools/tui/src/play/panels/stack_view.rs`** (line ~161): Add arm:
   ```
   StackObjectKind::ForecastAbility { source_object, .. } => {
       ("Forecast: ".to_string(), Some(*source_object))
   }
   ```

2. **`tools/replay-viewer/src/view_model.rs`** (line ~421 area): Add arm:
   ```
   StackObjectKind::ForecastAbility { source_object, .. } => {
       ("forecast", Some(*source_object))
   }
   ```

3. **`crates/engine/src/state/hash.rs`** (line ~1375 area): Add arm for `ForecastAbility` in the `StackObjectKind` HashInto impl.

### Step 7: Harness Action

**File**: `crates/engine/src/testing/replay_harness.rs`
**Action**: Add `activate_forecast` action type in `translate_player_action()`.

```
"activate_forecast" => {
    let card_name = action["card"].as_str().unwrap();
    let card_id = card_name_to_id(&state, card_name, player)?;
    let targets = parse_targets(&action, &state)?; // reuse existing target parsing
    Some(Command::ActivateForecast { player, card: card_id, targets })
}
```

### Step 8: Unit Tests

**File**: `crates/engine/tests/forecast.rs` (new file)
**Tests to write**:
- `test_forecast_basic_activates_during_upkeep` -- CR 702.57a: Set up a card with Forecast in hand during owner's upkeep. Activate it. Verify ability goes on stack, mana paid, card still in hand.
- `test_forecast_fails_outside_upkeep` -- CR 702.57b: Try to activate during main phase. Verify error.
- `test_forecast_fails_during_opponent_upkeep` -- CR 702.57b: Try to activate during another player's upkeep (multiplayer). Verify error.
- `test_forecast_once_per_turn` -- CR 702.57b: Activate once (succeeds), try again in same upkeep (fails with error).
- `test_forecast_resets_each_turn` -- CR 702.57b: Activate turn 1, advance to turn 2's upkeep, activate again (succeeds).
- `test_forecast_card_stays_in_hand` -- CR 702.57a: After activation and resolution, verify the card is still in the player's hand (not discarded, not exiled).
- `test_forecast_effect_resolves` -- Verify the forecast effect actually executes (e.g., draw a card, create a token).
- `test_forecast_blocked_by_split_second` -- CR 702.61a: Forecast cannot be activated while split second is on the stack.
**Pattern**: Follow `crates/engine/tests/cycling.rs` for the overall test structure (GameStateBuilder, card registration, command dispatch).

### Step 9: Card Definition (later phase)

**Suggested card**: Pride of the Clouds ({W}{U}, 1/1 Elemental Cat with Flying)
- Forecast -- {2}{W}{U}, Reveal: Create a 1/1 white and blue Bird creature token with flying.
- Simpler effect than Sky Hussar (which requires tapping creatures as cost, not just mana).
- Alternative: A custom test card with a simple mana-only cost and a basic effect (e.g., draw a card).

**Card lookup**: use `card-definition-author` agent.

### Step 10: Game Script (later phase)

**Suggested scenario**: Player has a Forecast card in hand. During their upkeep, activate forecast, pay cost, ability resolves. Verify card still in hand, effect applied. Try a second activation (fails). Next turn, activate again (succeeds).
**Subsystem directory**: `test-data/generated-scripts/stack/`

## Interactions to Watch

- **Cycling vs Forecast**: Both are hand-zone activated abilities. Cycling discards; Forecast does not. Ensure the handler patterns diverge at the right point.
- **Split second**: Standard block — same pattern as Cycling and ActivateAbility handlers.
- **Stifle/counterspell**: The forecast ability is on the stack and can be countered. If countered, the "once per turn" is already consumed (activation cost is paid before stack push).
- **Priority reset**: After activation, priority goes to the active player (standard CR 602.2e pattern, same as all other activated abilities).
- **Multiplayer timing**: "Owner's upkeep" = the active player must be the card's owner. This is checked by `state.turn.active_player != player` since the player activating must be the card's owner (card is in Hand(player) and must be activated during that player's upkeep).
- **Trigger doubling (Panharmonicon)**: Does NOT apply — Forecast is an activated ability, not a triggered ability. Panharmonicon only doubles ETB triggers.
- **Cards with non-mana costs**: Sky Hussar's forecast cost is "Tap two untapped white and/or blue creatures you control" — this is a complex cost that cannot be represented as a simple `ManaCost`. For the initial implementation, use `ManaCost`-only costs. Non-mana costs (tap creatures, sacrifice, etc.) would require extending `AbilityDefinition::Forecast` to use `Cost` instead of `ManaCost`. This is acceptable as a Phase 2 enhancement — most Forecast cards use mana costs.
