# Ability Plan: Cycling

**Generated**: 2026-02-26
**CR**: 702.29
**Priority**: P2 (upgraded to P1-track for this implementation pass)
**Similar abilities studied**: Flashback (CR 702.34) -- non-standard-zone ability with `AbilityDefinition` variant + `KeywordAbility` marker; Equip/Mind Stone activated abilities in `rules/abilities.rs`

## CR Rule Text

702.29. Cycling

702.29a Cycling is an activated ability that functions only while the card with cycling is in a player's hand. "Cycling [cost]" means "[Cost], Discard this card: Draw a card."

702.29b Although the cycling ability can be activated only if the card is in a player's hand, it continues to exist while the object is on the battlefield and in all other zones. Therefore objects with cycling will be affected by effects that depend on objects having one or more activated abilities.

702.29c Some cards with cycling have abilities that trigger when they're cycled. "When you cycle this card" means "When you discard this card to pay an activation cost of a cycling ability." These abilities trigger from whatever zone the card winds up in after it's cycled.

702.29d Some cards have abilities that trigger whenever a player "cycles or discards" a card. These abilities trigger only once when a card is cycled.

702.29e Typecycling is a variant of the cycling ability. "[Type]cycling [cost]" means "[Cost], Discard this card: Search your library for a [type] card, reveal it, and put it into your hand. Then shuffle your library." This type is usually a subtype (as in "mountaincycling") but can be any card type, subtype, supertype, or combination thereof (as in "basic landcycling").

702.29f Typecycling abilities are cycling abilities, and typecycling costs are cycling costs. Any cards that trigger when a player cycles a card will trigger when a card is discarded to pay an activation cost of a typecycling ability. Any effect that stops players from cycling cards will stop players from activating cards' typecycling abilities. Any effect that increases or reduces a cycling cost will increase or reduce a typecycling cost. Any effect that looks for a card with cycling will find a card with typecycling.

## Key Edge Cases

- **Hand-only activation (CR 702.29a)**: Cycling can ONLY be activated from a player's hand. The current `handle_activate_ability` requires `obj.zone == ZoneId::Battlefield` (line 64 of `abilities.rs`). This is the primary infrastructure change needed.
- **Discard-as-cost (CR 702.29a)**: The card itself is discarded as part of the activation cost, similar to sacrifice-as-cost but from hand to graveyard. The source object moves to the graveyard BEFORE the cycling ability goes on the stack.
- **Instant-speed timing**: Cycling has NO timing restriction -- it can be activated any time you have priority. It is NOT sorcery-speed (unlike Equip). CR 702.29a says nothing about sorcery-speed, and cycling can be activated in response to spells.
- **The ability uses the stack (CR 602.2)**: The "draw a card" effect is on the stack and can be responded to (e.g., Stifle can counter the cycling trigger). The discard happens immediately as cost.
- **"When you cycle" triggers (CR 702.29c)**: These trigger from whatever zone the card is in after cycling (i.e., the graveyard). The trigger goes on the stack ON TOP of the cycling ability, so it resolves BEFORE the draw. Deferred to a future step (card-specific triggers).
- **"Cycles or discards" (CR 702.29d)**: A single cycle event triggers both "cycle" and "discard" triggers exactly once, not twice. The engine needs to emit a `CardCycled` event in addition to `CardDiscarded`, or mark the discard as cycling-related so that `CardDiscarded` handlers know whether cycling also applies.
- **Keyword exists in all zones (CR 702.29b)**: Even though cycling can only be activated from hand, the keyword is visible on the battlefield/graveyard/exile for effects that check "cards with cycling." The `KeywordAbility::Cycling` variant must be present in characteristics regardless of zone.
- **Typecycling (CR 702.29e-f)**: Deferred -- typecycling is a variant that searches the library instead of drawing. Can be added as a separate `AbilityDefinition::Typecycling` variant later.
- **Cycling cost reduction (Fluctuator)**: Fluctuator says "Cycling abilities you activate cost {2} less to activate." This is a cost-reduction continuous effect. Deferred to future work.
- **Multiplayer**: No special multiplayer considerations beyond standard cycling rules. Any player with priority can cycle (including non-active players during opponents' turns).

## Current State (from ability-wip.md)

- [ ] Step 1: Enum variant
- [ ] Step 2: Rule enforcement
- [ ] Step 3: Trigger wiring
- [ ] Step 4: Unit tests
- [ ] Step 5: Card definition
- [ ] Step 6: Game script
- [ ] Step 7: Coverage doc update

Nothing exists yet -- no `KeywordAbility::Cycling`, no `AbilityDefinition::Cycling`, no cycling handling in `abilities.rs`.

## Implementation Steps

### Step 1: Enum Variant + AbilityDefinition Variant

**1a. KeywordAbility variant**

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/types.rs`
**Action**: Add `Cycling` variant to `KeywordAbility` enum, after `Flashback` (line 194).
**Pattern**: Follow `KeywordAbility::Flashback` at line 188-194.

```rust
/// CR 702.29: Cycling [cost] -- activated ability from hand.
/// "Cycling [cost]" means "[cost], Discard this card: Draw a card."
/// Activate only from hand. The keyword exists in all zones (CR 702.29b).
///
/// This variant is a marker for quick presence-checking (`keywords.contains`).
/// The cycling cost itself is stored in `AbilityDefinition::Cycling { cost }`.
Cycling,
```

**1b. Hash discriminant for KeywordAbility::Cycling**

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/hash.rs`
**Action**: Add hash arm after `KeywordAbility::Flashback => 27u8` (line 315).
**Next discriminant**: `28u8`

```rust
// Cycling (discriminant 28) -- CR 702.29
KeywordAbility::Cycling => 28u8.hash_into(hasher),
```

**1c. AbilityDefinition::Cycling variant**

**File**: `/home/airbaggie/scutemob/crates/engine/src/cards/card_definition.rs`
**Action**: Add `Cycling { cost: ManaCost }` variant to `AbilityDefinition` enum, after `Flashback` (line 144).
**Pattern**: Follow `AbilityDefinition::Flashback { cost: ManaCost }` at line 137-144.

```rust
/// CR 702.29: Cycling [cost]. The card may be activated from hand by paying
/// [cost] and discarding itself. The effect is "draw a card."
///
/// Cards with this ability should also include
/// `AbilityDefinition::Keyword(KeywordAbility::Cycling)` for quick
/// presence-checking without scanning all abilities.
Cycling { cost: ManaCost },
```

**1d. Hash discriminant for AbilityDefinition::Cycling**

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/hash.rs`
**Action**: Add hash arm for `AbilityDefinition::Cycling` after `AbilityDefinition::Flashback` (line 2203-2206).
**Next discriminant**: `9u8`

```rust
// Cycling (discriminant 9) -- CR 702.29
AbilityDefinition::Cycling { cost } => {
    9u8.hash_into(hasher);
    cost.hash_into(hasher);
}
```

**1e. Match arm audit**

Grep for exhaustive `match` on `KeywordAbility` and `AbilityDefinition` to find any
other match arms that need updating. Key locations:
- `state/hash.rs` (covered above)
- Any display/formatting code
- `enrich_spec_from_def` in `testing/replay_harness.rs` -- the keyword arm already
  handles all keywords generically via `AbilityDefinition::Keyword(kw)` (line 381),
  so `Cycling` will be picked up automatically.
- The `cost_to_activation_cost` function (line 674) skips `DiscardCard` -- this is
  the current behavior and needs to change (see Step 2).

### Step 2: Rule Enforcement -- Cycling Activation from Hand

This is the core infrastructure change. Currently `handle_activate_ability` in `abilities.rs` requires the source to be on the battlefield (line 64). Cycling activates from the hand.

There are two design approaches:

**Option A (Recommended): New Command `CycleCard`**

A dedicated command is cleaner because cycling has fundamentally different zone requirements
(hand, not battlefield), different cost structure (discard self + mana, not tap/sacrifice),
and a fixed effect (draw a card). A dedicated command avoids bloating `handle_activate_ability`
with zone-conditional logic.

**Option B: Extend `ActivateAbility` with zone awareness**

This would add a `from_zone` field or make the zone check conditional. Rejected because
cycling's cost model (discard self, not sacrifice self) and zone model (hand, not battlefield)
are sufficiently different to warrant separation.

#### Step 2a: New Command variant

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/command.rs`
**Action**: Add `CycleCard` command after `ActivateAbility`.

```rust
/// Cycle a card from hand (CR 702.29a).
///
/// Cycling is an activated ability that functions only while the card is in
/// the player's hand. The activation cost is the cycling cost (mana) plus
/// discarding the card itself. The effect is "draw a card" and uses the stack.
///
/// Unlike `ActivateAbility` (which requires the source on the battlefield),
/// `CycleCard` works from the hand zone. The card is discarded as cost
/// (immediately), and a cycling ability is placed on the stack. When it
/// resolves, the controller draws a card.
CycleCard {
    player: PlayerId,
    card: ObjectId,
},
```

#### Step 2b: Command handler dispatch

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/engine.rs`
**Action**: Add match arm for `Command::CycleCard` in `process_command`. Follow the
pattern of `Command::ActivateAbility` (line 89).

```rust
Command::CycleCard { player, card } => {
    let events = abilities::handle_cycle_card(state, player, card)?;
    // CR 603.2: Check for triggers after cycling (including "when you cycle" triggers).
    let trigger_list = abilities::check_triggers(state, &events);
    state.pending_triggers.extend(trigger_list);
    let trigger_events = abilities::flush_pending_triggers(state);
    events.extend(trigger_events);
    all_events.extend(events);
}
```

#### Step 2c: Cycling handler function

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/abilities.rs`
**Action**: Add `handle_cycle_card` function. Place it after `handle_activate_ability` (after line 350).

The function must:

1. **Validate priority** (CR 602.2): `state.turn.priority_holder == Some(player)`.
2. **Validate zone** (CR 702.29a): Card must be in `ZoneId::Hand(player)`.
3. **Validate cycling keyword**: Card must have `KeywordAbility::Cycling` in its keywords.
4. **Look up cycling cost**: Find `AbilityDefinition::Cycling { cost }` in the card's
   definition via the `CardRegistry`. Follow the Flashback pattern (`get_flashback_cost`
   in `casting.rs` lines 390-402).
5. **Pay mana cost** (CR 602.2b): Deduct mana from player's pool.
6. **Discard self as cost** (CR 702.29a): Move the card from hand to graveyard. This
   happens BEFORE the ability goes on the stack. Emit `CardDiscarded` event. Also emit
   a new `CardCycled` event for "when you cycle" trigger matching.
7. **Push cycling ability on stack**: Create a `StackObject` with a new
   `StackObjectKind::CyclingAbility { source_object, new_grave_id }`. The
   `new_grave_id` tracks where the card ended up (for "when you cycle" triggers that
   reference the card in its new zone). Alternatively, reuse
   `StackObjectKind::ActivatedAbility` with `embedded_effect` set to
   `Effect::DrawCards { player: PlayerTarget::Controller, count: EffectAmount::Fixed(1) }`.

   **Recommended**: Reuse `StackObjectKind::ActivatedAbility` with `embedded_effect` to
   avoid adding a new stack variant. The `source_object` is the old (now-dead) ObjectId
   from the hand. The `embedded_effect` is the draw effect. `ability_index: 0` is a
   placeholder since the source no longer exists.

8. **Reset priority** (CR 602.2e): Reset `players_passed`, give priority to active player.
9. **Emit events**: `CardDiscarded`, `CardCycled` (new), `AbilityActivated`, `PriorityGiven`.

```rust
/// Handle a CycleCard command: validate, pay cost, discard self, push draw onto the stack.
///
/// CR 702.29a: Cycling is an activated ability from hand. "[Cost], Discard this card: Draw a card."
/// The discard is part of the cost (happens immediately). The draw uses the stack.
pub fn handle_cycle_card(
    state: &mut GameState,
    player: PlayerId,
    card: ObjectId,
) -> Result<Vec<GameEvent>, GameStateError> {
    // 1. Priority check (CR 602.2)
    // 2. Zone check -- must be in Hand(player) (CR 702.29a)
    // 3. Keyword check -- must have KeywordAbility::Cycling
    // 4. Look up cycling cost from CardRegistry
    // 5. Pay mana cost
    // 6. Discard self (move to graveyard, emit CardDiscarded + CardCycled)
    // 7. Push StackObject with embedded_effect = DrawCards
    // 8. Reset priority
    // 9. Return events
    ...
}
```

#### Step 2d: New GameEvent variant -- `CardCycled`

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/events.rs`
**Action**: Add `CardCycled` event after `CardDiscarded`.

```rust
/// A card was cycled from a player's hand (CR 702.29a).
///
/// This event fires IN ADDITION TO `CardDiscarded` (CR 702.29d: "cycles or discards"
/// triggers fire once). The `CardCycled` event enables "when you cycle" triggers
/// (CR 702.29c) that are distinct from generic discard triggers.
CardCycled {
    player: PlayerId,
    /// ObjectId of the card in hand (now retired -- CR 400.7).
    object_id: ObjectId,
    /// New ObjectId in the graveyard.
    new_id: ObjectId,
},
```

**Also update**: `GameEvent::reveals_hidden_info()` -- `CardCycled` reveals hidden info (like `CardDiscarded`).

**Also update**: `state/hash.rs` -- add a discriminant for `GameEvent::CardCycled` in the `GameEvent` hash impl (if events are hashed; check whether `GameEvent` has a `HashInto` impl).

#### Step 2e: Cycling cost lookup helper

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/abilities.rs` (or a new helper in `casting.rs`)
**Action**: Add `get_cycling_cost` function. Follow the pattern of `get_flashback_cost` in `casting.rs` lines 390-402.

```rust
/// CR 702.29a: Look up the cycling cost from the card's AbilityDefinition.
fn get_cycling_cost(
    card_id: &Option<CardId>,
    registry: &CardRegistry,
) -> Option<ManaCost> {
    card_id.as_ref().and_then(|cid| {
        registry.get(cid.clone()).and_then(|def| {
            def.abilities.iter().find_map(|a| {
                if let AbilityDefinition::Cycling { cost } = a {
                    Some(cost.clone())
                } else {
                    None
                }
            })
        })
    })
}
```

#### Step 2f: Resolution -- no special handling needed

The `StackObjectKind::ActivatedAbility` with `embedded_effect` already handles resolution
correctly in `resolution.rs` (line 257-288). The `embedded_effect` is
`Effect::DrawCards { player: Controller, count: Fixed(1) }`, which resolves via
`execute_effect` in `effects/mod.rs`. No changes needed to resolution.

#### Step 2g: Replay harness -- new action type `cycle_card`

**File**: `/home/airbaggie/scutemob/crates/engine/src/testing/replay_harness.rs`
**Action**: Add `"cycle_card"` match arm in `translate_player_action` (after `"activate_ability"`, around line 254).

```rust
"cycle_card" => {
    let card_id = find_in_hand(state, player, card_name?)?;
    Some(Command::CycleCard {
        player,
        card: card_id,
    })
}
```

### Step 3: Trigger Wiring

For the basic cycling implementation (P1 scope), we emit `CardCycled` and `CardDiscarded` events. The trigger infrastructure for "when you cycle this card" (CR 702.29c) and "whenever a player cycles or discards" (CR 702.29d) requires:

**Deferred for this pass**: Full "when you cycle" trigger wiring requires a new `TriggerEvent::SelfCycled` or `TriggerEvent::ControllerCyclesOrDiscards` and corresponding `check_triggers` arms. This is card-specific (Decree of Pain, Astral Drift, etc.) and can be wired when those card definitions are authored.

**What is done in this pass**:
- The `CardCycled` event is emitted alongside `CardDiscarded`. Both events are available for future trigger-matching.
- The `CardDiscarded` event already exists (line 375 of `events.rs`) and is emitted by the cycling handler. Any existing "whenever a player discards" triggers will fire correctly from cycling.
- The `check_triggers` function does NOT currently have a `CardDiscarded` arm (it only fires on battlefield events). Adding a `CardCycled` arm to `check_triggers` is deferred until a card definition needs it.

**Note for future work**: When adding "when you cycle" triggers:
1. Add `TriggerEvent::SelfCycled` to `game_object.rs`.
2. Add a `GameEvent::CardCycled` arm to `check_triggers` in `abilities.rs`.
3. The trigger fires from whatever zone the card is in after cycling (graveyard), per CR 702.29c.

### Step 4: Unit Tests

**File**: `/home/airbaggie/scutemob/crates/engine/tests/cycling.rs` (new file)
**Pattern**: Follow tests in `/home/airbaggie/scutemob/crates/engine/tests/abilities.rs` for activated ability test structure.

**Tests to write**:

1. **`test_cycling_basic_draws_card`** -- CR 702.29a
   - Setup: Player has a card with Cycling {1} in hand, library has 1+ cards, player has 1 mana.
   - Action: CycleCard command.
   - Assert: Card moved from hand to graveyard (discard), cycling ability on stack, after resolution player drew 1 card.
   - Note: Must pass priority around table to resolve the cycling ability from the stack.

2. **`test_cycling_requires_hand`** -- CR 702.29a
   - Setup: Card with Cycling keyword on battlefield.
   - Action: CycleCard command with that card's ObjectId.
   - Assert: Error returned (card not in hand).

3. **`test_cycling_requires_mana`** -- CR 702.29a
   - Setup: Card with Cycling {2} in hand, player has 0 mana.
   - Action: CycleCard command.
   - Assert: InsufficientMana error.

4. **`test_cycling_requires_priority`** -- CR 602.2
   - Setup: Card with Cycling in hand, but it's not this player's priority.
   - Action: CycleCard command.
   - Assert: NotPriorityHolder error.

5. **`test_cycling_discard_is_immediate_cost`** -- CR 702.29a
   - Setup: Card with Cycling in hand.
   - Action: CycleCard command, then inspect state BEFORE resolution.
   - Assert: Card is in graveyard (discard happened as cost), cycling ability is on stack (draw has not happened yet).

6. **`test_cycling_instant_speed`** -- CR 702.29a (no timing restriction)
   - Setup: Non-active player has a card with Cycling in hand, during opponent's main phase with spells on the stack.
   - Action: CycleCard command from non-active player (who has priority).
   - Assert: Succeeds -- cycling is not sorcery-speed.

7. **`test_cycling_keyword_on_battlefield`** -- CR 702.29b
   - Setup: Card with Cycling keyword on battlefield.
   - Assert: The permanent's `keywords` set contains `KeywordAbility::Cycling`. (This validates that the keyword is visible in all zones, not just hand.)

8. **`test_cycling_requires_cycling_keyword`** -- CR 702.29a
   - Setup: Card WITHOUT Cycling keyword in hand.
   - Action: CycleCard command.
   - Assert: Error (card does not have cycling).

**Build pattern**: Use `GameStateBuilder::four_player()` with objects placed via `ObjectSpec::card()` enriched with cycling keyword and abilities. For the card definition, create an inline `CardDefinition` with `AbilityDefinition::Keyword(KeywordAbility::Cycling)` and `AbilityDefinition::Cycling { cost: ManaCost { generic: 1, ..Default::default() } }`.

The `enrich_spec_from_def` function will pick up the `KeywordAbility::Cycling` marker automatically via the generic keyword loop (line 381 of `replay_harness.rs`). However, the cycling cost itself is stored in `AbilityDefinition::Cycling { cost }`, which is looked up at CycleCard command time from the `CardRegistry`. Tests must register the card definition in the registry.

### Step 5: Card Definition (later phase)

**Suggested card**: Lonely Sandbar
- Land with "This land enters tapped. {T}: Add {U}. Cycling {U}."
- Simple cycling land -- no triggered abilities on cycle, just the basic cycling mechanic.
- Commander staple (common in blue decks for land-slot flexibility).

**Alternative/additional card**: Forgotten Cave (same pattern, red).

**Card definition structure**:
```rust
CardDefinition {
    card_id: cid("lonely-sandbar"),
    name: "Lonely Sandbar".to_string(),
    mana_cost: None, // land
    types: TypeLine {
        card_types: ordset![CardType::Land],
        ..Default::default()
    },
    oracle_text: "Lonely Sandbar enters the battlefield tapped.\n{T}: Add {U}.\nCycling {U}".to_string(),
    abilities: vec![
        // ETB tapped replacement
        AbilityDefinition::Replacement { /* ... */ },
        // {T}: Add {U}
        AbilityDefinition::Activated {
            cost: Cost::Tap,
            effect: Effect::AddMana { player: PlayerTarget::Controller, mana: /* {U} */ },
            timing_restriction: None,
        },
        // CR 702.29a: Cycling marker
        AbilityDefinition::Keyword(KeywordAbility::Cycling),
        // CR 702.29a: Cycling cost {U}
        AbilityDefinition::Cycling {
            cost: ManaCost { blue: 1, ..Default::default() },
        },
    ],
    ..Default::default()
}
```

### Step 6: Game Script (later phase)

**Suggested scenario**: "Player cycles Lonely Sandbar from hand, paying {U}. Card goes to graveyard. Cycling ability resolves, player draws a card."

**Subsystem directory**: `test-data/generated-scripts/stack/` (cycling uses the stack)

**Script outline**:
1. Initial state: P1 has Lonely Sandbar in hand, Island on battlefield (for {U}), 3 cards in library.
2. P1 taps Island for {U}.
3. P1 cycles Lonely Sandbar.
4. Assert: Lonely Sandbar in P1's graveyard, cycling ability on stack.
5. All players pass priority.
6. Cycling ability resolves, P1 draws a card.
7. Assert: P1's hand count increased by 1 (net: same hand size since cycling discarded 1 and drew 1).

### Step 7: Coverage Doc Update

**File**: `/home/airbaggie/scutemob/docs/mtg-engine-ability-coverage.md`
**Action**: Update Cycling row from `none` to `validated` (after tests pass).

## Interactions to Watch

- **Cycling + Stifle/counterspell**: The draw effect is on the stack and can be countered. The discard already happened (it's a cost, not an effect). If the cycling ability is countered, the player still loses the card but doesn't draw. This should work automatically because the discard happens at activation time and the draw is on the stack.
- **Cycling + "when you cycle" triggers (CR 702.29c)**: These trigger AFTER the discard (cost) but BEFORE the draw (resolution). They go on the stack on top of the cycling ability. Not implemented in this pass but the event infrastructure (`CardCycled`) enables it.
- **Cycling + "whenever you discard" triggers (CR 702.29d)**: Cycling fires "discard" triggers too. Since the engine already emits `CardDiscarded`, any future "whenever you discard" triggers will fire from cycling automatically. A single cycle fires "cycle" and "discard" triggers exactly once each, not twice.
- **Cycling + Flashback interaction**: A card could theoretically have both cycling and flashback. No conflict -- cycling activates from hand, flashback casts from graveyard.
- **Cycling + Rest in Peace**: If Rest in Peace is on the battlefield, the cycling discard would be exiled instead of going to graveyard (replacement effect). The draw still happens. The engine's existing replacement effect infrastructure handles this.
- **Cycling + Waste Not / "whenever a player discards"**: The `CardDiscarded` event fires when cycling, so Waste Not style triggers work. No extra wiring needed beyond what exists.
- **Cycling from hand with empty library**: The discard cost succeeds, the ability goes on the stack. On resolution, `DrawCards` from an empty library triggers the SBA for losing the game (CR 704.5b). This works automatically via existing draw logic.

## Design Decision: Command vs. Extended ActivateAbility

**Decision: New `Command::CycleCard` variant.**

Rationale:
1. Cycling activates from hand, not battlefield. The entire validation path in `handle_activate_ability` assumes battlefield zone (line 64), controller check (line 67-72), summoning sickness (line 180-195), tap cost (line 173-203), sacrifice cost (line 221-246). None of these apply to cycling.
2. The cost structure is different: cycling costs mana + discard self. The `ActivationCost` struct has `requires_tap`, `mana_cost`, `sacrifice_self` but no `discard_self`. Adding `discard_self` and zone-conditional logic would bloat the struct and handler for one ability.
3. The effect is always "draw a card" -- no variable effects, no targets. A dedicated handler is simpler.
4. Flashback set the precedent: it got special handling in `casting.rs` rather than being shoehorned into a generic activated ability path. Cycling follows the same pattern.
5. The replay harness already has action-specific commands (`cast_spell_flashback`). Adding `cycle_card` is natural.

## Files Modified Summary

| File | Change |
|------|--------|
| `crates/engine/src/state/types.rs` | Add `KeywordAbility::Cycling` |
| `crates/engine/src/state/hash.rs` | Add hash arms for `KeywordAbility::Cycling` (28u8), `AbilityDefinition::Cycling` (9u8), `GameEvent::CardCycled` (if hashed) |
| `crates/engine/src/cards/card_definition.rs` | Add `AbilityDefinition::Cycling { cost: ManaCost }` |
| `crates/engine/src/rules/command.rs` | Add `Command::CycleCard { player, card }` |
| `crates/engine/src/rules/engine.rs` | Add match arm for `Command::CycleCard` |
| `crates/engine/src/rules/abilities.rs` | Add `handle_cycle_card` function + `get_cycling_cost` helper |
| `crates/engine/src/rules/events.rs` | Add `GameEvent::CardCycled` variant |
| `crates/engine/src/testing/replay_harness.rs` | Add `"cycle_card"` action handler |
| `crates/engine/tests/cycling.rs` | New test file with 8 tests |
| `docs/mtg-engine-ability-coverage.md` | Update Cycling row status |
