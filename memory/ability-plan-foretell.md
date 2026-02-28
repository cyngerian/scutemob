# Ability Plan: Foretell

**Generated**: 2026-02-27
**CR**: 702.143
**Priority**: P3
**Similar abilities studied**: Escape (cast from exile with alternative cost, `rules/casting.rs`), BringCompanion (special action with {N} mana payment, `rules/commander.rs:648`), Madness (exile face-down then cast from exile, `rules/casting.rs`)

## CR Rule Text

702.143. Foretell

[702.143a] Foretell is a keyword that functions while the card with foretell is in a player's hand. Any time a player has priority during their turn, that player may pay {2} and exile a card with foretell from their hand face down. That player may look at that card as long as it remains in exile. They may cast that card after the current turn has ended by paying any foretell cost it has rather than paying that spell's mana cost. Casting a spell this way follows the rules for paying alternative costs in rules 601.2b and 601.2f-h.

[702.143b] Exiling a card using its foretell ability is a special action, which doesn't use the stack. See rule 116, "Special Actions."

[702.143c] If an effect refers to foretelling a card, it means performing the special action associated with a foretell ability. If an effect refers to a card or spell that was foretold, it means a card put in the exile zone as a result of the special action associated with a foretell ability, or a spell that was a foretold card before it was cast, even if it was cast for a cost other than a foretell cost.

[702.143d] If an effect states that a card in exile becomes foretold, that card becomes a foretold card. That effect may give the card a foretell cost. That card's owner may look at that card as long as it remains in exile and it may be cast for any foretell cost it has after the turn it became a foretold card has ended, even if the resulting spell doesn't have foretell.

[702.143e] If a player owns multiple foretold cards in exile, they must ensure that those cards can be easily differentiated from each other and from any other face-down cards in exile which that player owns. This includes knowing both the order in which those cards were put into exile and any foretell costs other than their printed foretell costs those cards may have.

[702.143f] If a player leaves the game, all face-down foretold cards that player owns must be revealed to all players. At the end of each game, all face-down foretold cards must be revealed to all players.

### Related Rules

[116.2h] A player who has a card with foretell in their hand may pay {2} and exile that card face down. This is a special action. A player may take this action any time they have priority during their turn. See rule 702.143, "Foretell."

[118.9a] Only one alternative cost can be applied to any one spell as it's being cast.

[601.2b] ...Previously made choices (such as choosing to cast a spell with flashback from a graveyard...) may restrict the player's options when making these choices.

## Key Edge Cases

- **Foretell is a special action (CR 116.2h / 702.143b)**: It does NOT use the stack. The card is immediately exiled face-down. No player can respond to the foretell action itself. Similar to how `PlayLand` and `BringCompanion` work in the engine.
- **Timing: any time the player has priority during their turn (CR 702.143a / 116.2h)**: NOT restricted to sorcery speed. Unlike `BringCompanion` (main phase + empty stack), foretell can be done in response to spells, during combat, etc. -- any time during the player's own turn when they have priority.
- **Cannot cast on the same turn (CR 702.143a: "after the current turn has ended")**: The foretold card can be cast on any FUTURE turn, not the turn it was foretelled. Must track the turn number when the card was foretold.
- **Foretell cost is an alternative cost (CR 118.9)**: Mutual exclusion with flashback, escape, evoke, bestow, madness, miracle, and all other alternative costs. Only one alternative cost per spell.
- **Foretell cost does NOT ignore timing restrictions (ruling 2021-02-05)**: Casting a foretold card follows normal timing rules for that card type. Instants can be cast on any player's turn; sorceries require sorcery speed.
- **The {2} foretell action cost is generic mana**: Paid from the player's mana pool. Deducted immediately as part of the special action (like `BringCompanion`'s {3}).
- **Face-down in exile**: The card is exiled face-down. Opponents cannot see it. The owner can look at it. This maps to `ObjectStatus::face_down = true` + hidden information considerations.
- **Hidden information**: The `CardForetold` event should be marked as revealing hidden info to the owner only. Opponents see that a card was exiled face-down but not its identity. The `GameEvent` should use the `reveals_hidden_info()` pattern or a future `private_to` field.
- **Kicker interacts with foretell (ruling 2021-02-05)**: When casting for foretell cost, the player CAN still pay additional costs (kicker, etc.) but CANNOT apply a different alternative cost.
- **Multiplayer**: All players see the special action happen (a card goes to exile face-down) but only the owner sees the card identity. On player elimination (CR 702.143f), all face-down foretold cards must be revealed.
- **702.143d (effects making cards foretold)**: Out of scope for initial implementation. This is for cards like Dream Devourer that give foretell to other cards. The initial implementation handles the core foretell keyword only.

## Current State (from ability-wip.md)

- [ ] Step 1: Enum variant
- [ ] Step 2: Rule enforcement
- [ ] Step 3: Trigger wiring
- [ ] Step 4: Unit tests
- [ ] Step 5: Card definition
- [ ] Step 6: Game script
- [ ] Step 7: Coverage doc update

## Implementation Steps

### Step 1: Enum Variant and Type Infrastructure

#### 1a: `KeywordAbility::Foretell` variant

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/types.rs`
**Action**: Add `Foretell` variant to `KeywordAbility` enum after `Escape` (line ~397).
**Pattern**: Follow `KeywordAbility::Escape` at line 397 -- marker keyword for quick presence-checking.

```rust
/// CR 702.143: Foretell [cost] -- special action from hand; cast from exile for foretell cost.
///
/// Marker for quick presence-checking (`keywords.contains`).
/// The foretell cost itself is stored in `AbilityDefinition::Foretell { cost }`.
///
/// During the owner's turn, they may pay {2} and exile this card face down
/// (special action, CR 116.2h). On a later turn, they may cast it for its
/// foretell cost (alternative cost, CR 118.9).
Foretell,
```

#### 1b: `AbilityDefinition::Foretell` variant

**File**: `/home/airbaggie/scutemob/crates/engine/src/cards/card_definition.rs`
**Action**: Add `Foretell { cost: ManaCost }` variant to `AbilityDefinition` enum after `Escape`/`EscapeWithCounter` (line ~219).
**Pattern**: Follow `AbilityDefinition::Escape { cost, exile_count }` at line 212.

```rust
/// CR 702.143: Foretell [cost]. During your turn, pay {2} and exile this card
/// from your hand face down. Cast it on a later turn for [cost] rather than
/// its mana cost (alternative cost, CR 118.9).
///
/// Cards with this ability should also include
/// `AbilityDefinition::Keyword(KeywordAbility::Foretell)` for quick
/// presence-checking without scanning all abilities.
Foretell { cost: ManaCost },
```

#### 1c: `Command::ForetellCard` variant

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/command.rs`
**Action**: Add a new `ForetellCard` command variant after `CrewVehicle` (line ~319).
**Pattern**: Follow `Command::BringCompanion` -- similar special action with mana payment.

```rust
// -- Foretell (CR 702.143) -----------------------------------------------
/// Foretell a card from hand (CR 702.143a / CR 116.2h).
///
/// Special action: pay {2}, exile a card with foretell from your hand face down.
/// This does not use the stack. Legal any time you have priority during your turn.
/// The card can be cast for its foretell cost on a future turn.
ForetellCard { player: PlayerId, card: ObjectId },
```

#### 1d: `CastSpell::cast_with_foretell` flag

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/command.rs`
**Action**: Add `cast_with_foretell: bool` field to `Command::CastSpell` after `cast_with_escape` (line ~132).
**Pattern**: Follow `cast_with_escape: bool`.

```rust
/// CR 702.143a: If true, cast this spell by paying its foretell cost instead
/// of its mana cost. This is an alternative cost (CR 118.9) -- cannot
/// combine with flashback, evoke, bestow, madness, miracle, escape, or other
/// alternative costs.
///
/// The card must be in exile with `is_foretold == true` and
/// `foretold_turn < current turn number`.
#[serde(default)]
cast_with_foretell: bool,
```

#### 1e: `GameObject` foretell tracking fields

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/game_object.rs`
**Action**: Add two fields to `GameObject` after `was_escaped` (line ~343):

```rust
/// CR 702.143a: If true, this object in exile was foretold (exiled face-down
/// via the foretell special action). Used to determine whether the card can be
/// cast from exile for its foretell cost.
///
/// Set when the ForetellCard command is processed. Reset to false on zone
/// changes (CR 400.7) -- but since foretold cards are already in exile,
/// any zone change from exile clears this.
#[serde(default)]
pub is_foretold: bool,

/// CR 702.143a: The turn number when this card was foretold.
///
/// The card can only be cast for its foretell cost "after the current turn
/// has ended" -- i.e., on any turn where `state.turn.turn_number > foretold_turn`.
/// Zero means not foretold. Set alongside `is_foretold`.
#[serde(default)]
pub foretold_turn: u32,
```

#### 1f: `StackObject::cast_with_foretell` flag

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/stack.rs`
**Action**: Add `cast_with_foretell: bool` field to `StackObject` after `was_escaped` (line ~103).
**Pattern**: Follow `was_escaped: bool`.

```rust
/// CR 702.143a: If true, this spell was cast from exile by paying its foretell
/// cost. The foretell cost is an alternative cost (CR 118.9). Unlike flashback,
/// foretell does NOT change where the card goes on resolution -- it resolves
/// normally (permanent to battlefield, instant/sorcery to graveyard).
///
/// Must always be false for copies (`is_copy: true`) -- copies are not cast.
#[serde(default)]
pub cast_with_foretell: bool,
```

#### 1g: `GameEvent::CardForetold` event

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/events.rs`
**Action**: Add `CardForetold` event variant (after the last existing event variant, or in a logical grouping).

```rust
/// A card was foretold -- exiled face-down from hand via the foretell special
/// action (CR 702.143a / CR 116.2h). The {2} cost was paid.
///
/// `new_exile_id` is the ObjectId of the card in the exile zone (new per CR 400.7).
/// This event reveals hidden info (the card identity is private to the owner;
/// opponents only see that a card was exiled face-down).
CardForetold {
    player: PlayerId,
    /// The card's ObjectId before exile (now retired).
    object_id: ObjectId,
    /// New ObjectId in the exile zone.
    new_exile_id: ObjectId,
},
```

#### 1h: Hash updates

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/hash.rs`
**Actions** (4 locations):

1. **`KeywordAbility` hasher** (after discriminant 50 for Escape, line ~391):
   Add `KeywordAbility::Foretell => 51u8.hash_into(hasher),`

2. **`GameObject` hasher** (after `was_escaped` at line ~543):
   Add `self.is_foretold.hash_into(hasher);` and `self.foretold_turn.hash_into(hasher);`

3. **`StackObject` hasher** (after `was_escaped` at line ~1207):
   Add `self.cast_with_foretell.hash_into(hasher);`

4. **`AbilityDefinition` hasher** (after discriminant 16 for EscapeWithCounter, line ~2500):
   Add:
   ```rust
   // Foretell (discriminant 17) -- CR 702.143
   AbilityDefinition::Foretell { cost } => {
       17u8.hash_into(hasher);
       cost.hash_into(hasher);
   }
   ```

5. **`GameEvent` hasher**: Add discriminant for `CardForetold` event.

#### 1i: Zone-change reset

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/mod.rs`
**Action**: In `move_object_to_zone` (lines ~275 and ~355), add `is_foretold: false` and `foretold_turn: 0` to the new `GameObject` construction alongside `was_evoked: false`, `was_escaped: false`.
**Pattern**: Follow `was_escaped: false` at lines 284 and 360.

#### 1j: Builder defaults

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/builder.rs`
**Action**: In the `GameObject` construction in `build()` (line ~670), add `is_foretold: false` and `foretold_turn: 0` alongside `was_escaped: false`.
**Pattern**: Follow `was_escaped: false` at line 672.

#### 1k: All StackObject construction sites

**Action**: Grep for all locations that construct `StackObject` and add `cast_with_foretell: false` to each. There are multiple sites:
- `casting.rs:handle_cast_spell` (main spell construction, line ~764) -- this one will be `cast_with_foretell: casting_with_foretell`
- `casting.rs` (cascade trigger, storm trigger constructions)
- `copy.rs` (spell copy construction)
- `abilities.rs` (activated/triggered ability stack objects)
- Any other StackObject construction site

**Pattern**: Grep `cast_with_madness:` to find all StackObject construction sites (every site that sets `cast_with_madness` must also set `cast_with_foretell`).

#### 1l: View model update

**File**: `/home/airbaggie/scutemob/tools/replay-viewer/src/view_model.rs`
**Action**: Add `KeywordAbility::Foretell => "Foretell".to_string()` to the `format_keyword` match (line ~623, after `Escape`).

#### 1m: Engine `process_command` match arm

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/engine.rs`
**Action**: Add a match arm for `Command::ForetellCard` in `process_command` (after the `CrewVehicle` or `ChooseMiracle` arm).
**Pattern**: Follow `Command::BringCompanion` (line ~202).

```rust
Command::ForetellCard { player, card } => {
    validate_player_active(&state, player)?;
    // CR 104.4b: foretelling is a meaningful player choice; reset loop detection.
    loop_detection::reset_loop_detection(&mut state);
    let events = foretell::handle_foretell_card(&mut state, player, card)?;
    all_events.extend(events);
}
```

#### 1n: CastSpell destructuring update

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/engine.rs`
**Action**: Add `cast_with_foretell` to the `Command::CastSpell` destructuring pattern (line ~71) and pass it through to `casting::handle_cast_spell`.

#### 1o: Foretell module

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/foretell.rs` (NEW)
**Action**: Create a new module for the foretell special action handler.
**Also**: Add `pub mod foretell;` to `/home/airbaggie/scutemob/crates/engine/src/rules/mod.rs` (line ~24).

#### 1p: lib.rs exports

**File**: `/home/airbaggie/scutemob/crates/engine/src/lib.rs`
**Action**: Ensure `Command::ForetellCard` is accessible. Check that `Command` is already re-exported (it is, via `pub use rules::Command`). No new exports needed unless `ForetellCard` uses new types not already public.

#### 1q: reveals_hidden_info update

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/events.rs`
**Action**: Add `GameEvent::CardForetold { .. } => true` to `reveals_hidden_info()` (line ~762, before the catch-all `_ => false`).
**CR**: 702.143a -- the card identity is hidden from opponents.

### Step 2: Rule Enforcement -- Foretell Special Action

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/foretell.rs` (NEW)
**Action**: Implement `handle_foretell_card(state, player, card) -> Result<Vec<GameEvent>, GameStateError>`.
**Pattern**: Follow `handle_bring_companion` in `rules/commander.rs:648` -- special action with mana payment and zone move.

Handler logic:

```
/// CR 702.143a / CR 116.2h: Handle the ForetellCard special action.
///
/// Validates:
/// 1. Player has priority (already checked by engine.rs validate_player_active)
/// 2. It is the player's turn (CR 116.2h: "during their turn")
/// 3. The card is in the player's hand
/// 4. The card has KeywordAbility::Foretell (CR 702.143a)
/// 5. Player has {2} generic mana available
///
/// On success:
/// - Deducts {2} generic mana
/// - Moves the card from hand to exile (CR 400.7: new ObjectId)
/// - Sets face_down = true on the new exile object
/// - Sets is_foretold = true and foretold_turn = current turn number
/// - Emits ManaCostPaid and CardForetold events
///
/// NOTE: Unlike BringCompanion, foretell does NOT require main phase or empty stack.
/// It can be done any time the player has priority during their own turn (CR 116.2h).
pub fn handle_foretell_card(
    state: &mut GameState,
    player: PlayerId,
    card: ObjectId,
) -> Result<Vec<GameEvent>, GameStateError>
```

Key validation differences from BringCompanion:
- **No sorcery-speed restriction**: foretell only requires "priority during their turn" (CR 116.2h)
- **Card must be in hand**: verify `card_obj.zone == ZoneId::Hand(player)`
- **Card must have Foretell keyword**: verify `card_obj.characteristics.keywords.contains(&KeywordAbility::Foretell)`
- **{2} generic mana cost**: same pattern as BringCompanion's {3}

After zone move to exile:
- Set `new_obj.status.face_down = true`
- Set `new_obj.is_foretold = true`
- Set `new_obj.foretold_turn = state.turn.turn_number`

### Step 3: Rule Enforcement -- Casting from Exile for Foretell Cost

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/casting.rs`
**Action**: Extend `handle_cast_spell` to support `cast_with_foretell`.
**Pattern**: Follow the `cast_with_escape` pattern (lines ~339-417).

Changes needed in `handle_cast_spell`:

#### 3a: Add `cast_with_foretell: bool` parameter

Add `cast_with_foretell` to the function signature (after `cast_with_escape`). Update the call site in `engine.rs`.

#### 3b: Detect foretell casting

In the zone-validation block (lines ~89-181), add detection for foretell casting:

```rust
// CR 702.143a: Foretell -- allowed if card is in exile, is_foretold == true,
// and the current turn is later than foretold_turn.
let casting_with_foretell = if cast_with_foretell {
    // Explicit flag -- validate all conditions.
    let card_obj = state.object(card)?;
    if card_obj.zone != ZoneId::Exile {
        return Err(GameStateError::InvalidCommand(
            "foretell: card must be in exile (CR 702.143a)".into(),
        ));
    }
    if !card_obj.is_foretold {
        return Err(GameStateError::InvalidCommand(
            "foretell: card was not foretold (CR 702.143a)".into(),
        ));
    }
    if card_obj.foretold_turn >= state.turn.turn_number {
        return Err(GameStateError::InvalidCommand(
            "foretell: cannot cast foretold card on the same turn it was foretold (CR 702.143a: 'after the current turn has ended')".into(),
        ));
    }
    true
} else {
    false
};
```

#### 3c: Zone validation bypass

In the "card is not in your hand" check (line ~161-171), add `&& !casting_with_foretell` to the condition:

```rust
if card_obj.zone != ZoneId::Hand(player)
    && !casting_from_command_zone
    && !casting_with_flashback
    && !casting_with_madness
    && !casting_with_escape_auto
    && !cast_with_escape
    && !casting_with_foretell  // NEW
{
    return Err(...);
}
```

#### 3d: Mutual exclusion with other alternative costs

Add mutual exclusion checks (CR 118.9a) in the foretell validation block:

```rust
if casting_with_foretell {
    if casting_with_flashback { return Err("cannot combine foretell with flashback"); }
    if casting_with_evoke { return Err("cannot combine foretell with evoke"); }
    if casting_with_bestow { return Err("cannot combine foretell with bestow"); }
    if casting_with_madness { return Err("cannot combine foretell with madness"); }
    if cast_with_miracle { return Err("cannot combine foretell with miracle"); }
    if casting_with_escape { return Err("cannot combine foretell with escape"); }
}
```

Also add to the existing escape/evoke/bestow/etc. mutual exclusion blocks:
- Escape block: add `if cast_with_foretell { return Err(...); }`
- Evoke block: similarly
- Etc.

#### 3e: Foretell cost lookup

Add a `get_foretell_cost` helper function:

```rust
/// CR 702.143a / CR 118.9: Look up the foretell cost from the card's AbilityDefinition.
fn get_foretell_cost(
    card_id: &Option<CardId>,
    registry: &CardRegistry,
) -> Option<ManaCost> {
    card_id.as_ref().and_then(|cid| {
        registry.get(cid.clone()).and_then(|def| {
            def.abilities.iter().find_map(|a| {
                if let AbilityDefinition::Foretell { cost } = a {
                    Some(cost.clone())
                } else {
                    None
                }
            })
        })
    })
}
```

**Pattern**: Follow `get_escape_cost` at line 1073.

#### 3f: Use foretell cost as mana cost

In the cost determination block (lines ~419-464), add a foretell branch:

```rust
} else if casting_with_foretell {
    // CR 702.143a: Pay foretell cost instead of mana cost (alternative cost, CR 118.9).
    let cost = get_foretell_cost(&card_id, &state.card_registry);
    if cost.is_none() {
        return Err(GameStateError::InvalidCommand(
            "card has Foretell keyword but no foretell cost defined".into(),
        ));
    }
    cost
} else {
    base_mana_cost
}
```

#### 3g: Timing restrictions

**CR ruling 2021-02-05**: Casting a foretold card from exile follows normal timing rules.
No special override needed -- the existing sorcery-speed check already applies. Do NOT add foretell to the timing bypass list (which currently only has madness and miracle).

#### 3h: StackObject construction

In the `StackObject` construction (line ~764), add:
```rust
cast_with_foretell: casting_with_foretell,
```

#### 3i: Update all existing StackObject construction sites

All other StackObject construction sites (cascade trigger, storm trigger, etc.) should set `cast_with_foretell: false`.

### Step 3 (Trigger Wiring): Not Applicable

Foretell does not use triggers. The foretell action is a special action (no stack, no triggers). Casting for foretell cost is handled via the existing `CastSpell` flow.

However, note: some cards trigger "whenever you foretell a card" (e.g., Ranar the Ever-Watchful). This is future work for when those cards are defined. The `CardForetold` event emitted in Step 2 provides the hook for those triggers.

### Step 4: Unit Tests

**File**: `/home/airbaggie/scutemob/crates/engine/tests/foretell.rs` (NEW)
**Pattern**: Follow `crates/engine/tests/escape.rs` for structure and helpers.

#### Tests to write:

1. **`test_foretell_basic_exile_face_down`** -- CR 702.143a / CR 116.2h
   - Player has a foretell card in hand, pays {2}, card is exiled face-down.
   - Assert: card is in exile zone, `face_down == true`, `is_foretold == true`, `foretold_turn == current turn`.
   - Assert: `ManaCostPaid` and `CardForetold` events emitted.

2. **`test_foretell_cast_from_exile_on_later_turn`** -- CR 702.143a
   - Player foretells a card on turn 1, then on turn 2 casts it for the foretell cost.
   - Use `GameStateBuilder` with `turn_number(2)` for the cast state, or advance the turn.
   - Assert: spell resolves normally, card moves from exile to stack to graveyard (for instant/sorcery).

3. **`test_foretell_cannot_cast_same_turn`** -- CR 702.143a: "after the current turn has ended"
   - Player foretells a card on turn 1, then attempts to cast it on the same turn.
   - Assert: `InvalidCommand` error.

4. **`test_foretell_requires_player_turn`** -- CR 116.2h: "during their turn"
   - Player attempts to foretell during an opponent's turn.
   - Assert: error (either "not priority holder" or "not your turn").

5. **`test_foretell_requires_foretell_keyword`** -- CR 702.143a
   - Player attempts to foretell a card without the Foretell keyword.
   - Assert: `InvalidCommand` error.

6. **`test_foretell_requires_card_in_hand`** -- CR 702.143a
   - Player attempts to foretell a card that is not in their hand (e.g., on battlefield).
   - Assert: `InvalidCommand` error.

7. **`test_foretell_insufficient_mana`** -- Cost validation
   - Player has less than {2} mana and tries to foretell.
   - Assert: `InsufficientMana` error.

8. **`test_foretell_does_not_use_stack`** -- CR 702.143b
   - After foretelling, the stack should be empty.
   - Assert: `state.stack_objects.is_empty()`.

9. **`test_foretell_during_combat`** -- CR 116.2h: any time with priority during turn
   - Set up combat step, foretell a card during combat.
   - Assert: succeeds (not restricted to main phase).

10. **`test_foretell_mutual_exclusion_with_flashback`** -- CR 118.9a
    - Card in exile is foretold, player also has flashback. Attempt to combine.
    - Assert: `InvalidCommand` error about alternative cost mutual exclusion.

11. **`test_foretell_mutual_exclusion_with_escape`** -- CR 118.9a
    - Attempt to use foretell and escape simultaneously.
    - Assert: `InvalidCommand` error.

12. **`test_foretell_with_kicker`** -- Ruling 2021-02-05
    - Foretell a card with kicker, cast it from exile paying foretell cost + kicker.
    - Assert: kicker is paid, spell is kicked.

13. **`test_foretell_instant_timing`** -- Ruling 2021-02-05
    - Foretell an instant card, cast it from exile on an opponent's turn (at instant speed).
    - Assert: succeeds (instants follow their normal timing from exile).

14. **`test_foretell_sorcery_timing`** -- Ruling 2021-02-05
    - Foretell a sorcery card, attempt to cast it from exile at instant speed.
    - Assert: `InvalidCommand` error (sorcery speed required).

15. **`test_foretell_card_identity_tracking`** -- CR 400.7
    - After foretelling, the old ObjectId is dead. Find the card in exile by name.
    - Assert: correct card in exile with correct attributes.

16. **`test_foretell_mana_value_unchanged`** -- CR 118.9c
    - A foretold card retains its original mana cost (for mana value calculations).
    - The foretell cost is what is PAID, not the mana cost of the spell.
    - Assert: mana value of the card in exile matches the printed mana cost.

### Step 5: Card Definition (later phase)

**Suggested card**: Saw It Coming ({1}{U}{U}, Instant -- Counter target spell. Foretell {1}{U}.)
**Card lookup**: Oracle text confirmed via MCP: "Counter target spell. Foretell {1}{U}"
**CardDefinition structure**:
```rust
CardDefinition {
    id: "saw-it-coming",
    name: "Saw It Coming",
    mana_cost: ManaCost { generic: 1, blue: 2, ..default() },
    card_types: [Instant],
    oracle_text: "Counter target spell.\nForetell {1}{U}",
    abilities: [
        AbilityDefinition::Spell {
            effect: Effect::CounterSpell { target: EffectTarget::DeclaredTarget { index: 0 } },
            targets: [TargetRequirement { filter: TargetFilter::Spell, ..default() }],
            modes: None,
            cant_be_countered: false,
        },
        AbilityDefinition::Keyword(KeywordAbility::Foretell),
        AbilityDefinition::Foretell { cost: ManaCost { generic: 1, blue: 1, ..default() } },
    ],
    ..default()
}
```

### Step 6: Game Script (later phase)

**Suggested scenario**: Player 1 foretells Saw It Coming on turn 1, then on turn 2, Player 2 casts a spell, and Player 1 casts Saw It Coming from exile for {1}{U} to counter it.

**Subsystem directory**: `test-data/generated-scripts/stack/`

**Script outline**:
1. Initial state: P1 has Saw It Coming in hand, both players have lands for mana.
2. Turn 1 (P1): Tap 2 lands for mana, ForetellCard (Saw It Coming).
3. Assertions: Saw It Coming in exile face-down, P1 hand reduced.
4. Turn 2 (P2 active): P2 casts some spell (e.g., Lightning Bolt targeting P1).
5. P1 responds: Tap Island + another land, CastSpell (Saw It Coming from exile, cast_with_foretell: true) targeting P2's spell.
6. Assertions: P2's spell is countered, Saw It Coming in P1's graveyard.

### Step 7: Replay Harness Action Type

**File**: `/home/airbaggie/scutemob/crates/engine/src/testing/replay_harness.rs`
**Action**: Add a `"foretell_card"` action type to `translate_player_action`.
**Pattern**: Follow `"cast_spell_flashback"` or `"cast_spell_escape"` patterns.

```rust
"foretell_card" => {
    let card_id = find_in_hand(state, player, card_name?)?;
    Some(Command::ForetellCard { player, card: card_id })
}
```

Also add a `"cast_spell_foretell"` action type for casting from exile:

```rust
"cast_spell_foretell" => {
    let card_id = find_in_exile(state, player, card_name?)?;
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
        cast_with_bestow: false,
        cast_with_miracle: false,
        cast_with_escape: false,
        escape_exile_cards: vec![],
        cast_with_foretell: true,
    })
}
```

**Note**: A `find_in_exile` helper may need to be added to `replay_harness.rs`, since foretold cards are in the shared `ZoneId::Exile` zone. Match by name AND owner. Be aware that `ZoneId::Exile` is a shared zone -- filter by `obj.owner == player`.

## Interactions to Watch

- **Face-down objects in exile**: The engine already has `ObjectStatus::face_down` and the hash module already hashes face-down cards in the private state hash (lines 2648-2660 of `hash.rs`). This infrastructure is ready.
- **ZoneId::Exile is shared**: Unlike `ZoneId::Hand(player)` or `ZoneId::Graveyard(player)`, `ZoneId::Exile` has no player ID. When finding foretold cards, filter by `obj.owner == player` in addition to `obj.zone == ZoneId::Exile` and `obj.is_foretold == true`.
- **CR 400.7 on zone move**: When the card moves from hand to exile via ForetellCard, it gets a new ObjectId. The `is_foretold` and `foretold_turn` must be set on the NEW object after the zone move. The `move_object_to_zone` function in `state/mod.rs` creates the new object with defaults -- the handler must modify the new object AFTER the zone move.
- **Resolution of foretold spells**: Foretell does NOT change where the card goes on resolution (unlike flashback which exiles). Instants/sorceries go to graveyard normally. Permanents go to battlefield normally. No special handling in `resolution.rs` needed.
- **Kicker compatibility**: Foretell cost is an alternative cost; kicker is an additional cost. They combine per CR 118.9d: "If an alternative cost is being paid, any additional costs, cost increases, and cost reductions that affect that spell are applied to that alternative cost." The existing kicker handling already works with alternative costs.
- **Commander tax**: If a commander has foretell (unlikely but theoretically possible with partner + foretell), commander tax applies on top of the foretell cost (it's an additional cost). The existing commander tax logic in `casting.rs` should apply automatically since it runs after cost determination.
- **Multiplayer**: All opponents see `CardForetold` but cannot see the card identity. The existing `reveals_hidden_info()` mechanism flags this. The actual hidden-info filtering happens in the M10 network layer (not yet built).
