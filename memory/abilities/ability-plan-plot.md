# Ability Plan: Plot

**Generated**: 2026-03-02
**CR**: 702.170
**Priority**: P4
**Batch**: 5.3 (batch plan said CR 702.164, correct is 702.170)
**Similar abilities studied**: Foretell (CR 702.143 -- special action exile from hand, cast from exile later), Suspend (CR 702.62 -- cast without paying mana cost from exile)

## CR Rule Text

```
702.170. Plot

702.170a Plot is a keyword ability that functions while the card with plot is in a
player's hand. "Plot [cost]" means "Any time you have priority during your main phase
while the stack is empty, you may exile this card from your hand and pay [cost]. It
becomes a plotted card."

702.170b Exiling a card using its plot ability is a special action, which doesn't use
the stack. See rule 116, "Special Actions."

702.170c In addition to the plot special action, some spells and abilities cause a card
in exile to become plotted.

702.170d A plotted card's owner may cast it from exile without paying its mana cost
during their main phase while the stack is empty during any turn after the turn in which
it became plotted. Casting a spell this way follows the rules for paying alternative
costs in rules 601.2b and 601.2f-h. A plotted card may be cast this way even if it
doesn't have the plot ability while in exile.

702.170e If an effect refers to plotting a card, it means performing the special action
associated with a plot ability.

702.170f An effect may allow the plot ability of a card to function in a zone other than
a player's hand. In that case, the card is exiled from the zone it is in as the action
is taken rather than from its owner's hand.
```

Related special action rule (CR 116.2k):
```
116.2k A player who has a card with plot in their hand may exile that card. This is a
special action. A player can take this action any time they have priority during their
own turn while the stack is empty. See rule 702.170, "Plot."
```

And CR 116.3:
```
116.3 If a player takes a special action, that player receives priority afterward.
```

## Key Edge Cases

From CR and card rulings (Slickshot Show-Off):

1. **Timing for plot special action (CR 116.2k / 702.170a)**: Main phase + empty stack
   (sorcery speed). This is stricter than Foretell (any time during your turn). The CR
   text says "main phase while the stack is empty" which is sorcery-speed timing.

2. **Timing for free cast (CR 702.170d)**: Main phase + empty stack, on any turn AFTER
   the turn it was plotted. Same sorcery-speed restriction. Must be a different turn
   than when plotted.

3. **Free cast is "without paying its mana cost" (CR 702.170d)**: ManaCost is zero.
   This follows alternative cost rules (CR 601.2b, 601.2f-h). Cannot combine with other
   alternative costs (CR 118.9a). Additional costs (kicker) are still allowed. Mandatory
   additional costs must still be paid.

4. **X = 0 for plotted cards (ruling 2024-04-12 on Slickshot Show-Off)**: If a plotted
   card has {X} in its mana cost, X must be 0 when casting without paying mana cost.

5. **Face-up in exile (CR 702.170a)**: "It becomes a plotted card." Unlike Foretell
   (face-down), Plot cards are face-up in exile. This is PUBLIC information. No hidden
   info event handling needed (contrast with Foretell's `face_down = true`).

6. **Cast even without plot keyword in exile (CR 702.170d)**: "A plotted card may be
   cast this way even if it doesn't have the plot ability while in exile." The `is_plotted`
   flag is the controlling state, not the keyword's presence in exile.

7. **No other alternative costs with Plot free-cast (ruling 2024-04-12)**: "If you're
   casting a plotted card from exile without paying its mana cost, you can't choose to
   cast it for any other alternative costs."

8. **Special action is unrespondable (CR 702.170b / ruling 2024-04-12)**: "Once you
   announce you're taking that action, no other player can respond by trying to remove
   that card from your hand."

9. **702.170f: Zone flexibility for effects**: Some effects allow plotting from zones
   other than hand. V1 scope: hand only. Future: support 702.170f when needed.

10. **Multiplayer**: No special multiplayer considerations beyond the standard "your
    main phase" timing restriction. Works the same in 4-player Commander as in 1v1.

## Current State (from ability-wip.md)

- [ ] 1. Enum variant
- [ ] 2. Rule enforcement
- [ ] 3. Trigger wiring (N/A -- Plot has no triggers)
- [ ] 4. Unit tests
- [ ] 5. Card definition
- [ ] 6. Game script
- [ ] 7. Coverage doc update

## Implementation Steps

### Step 1: Enum Variant and Type Changes

#### 1a. `KeywordAbility::Plot` variant

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/types.rs`
**Action**: Add `Plot` variant after `Blitz` (line ~860).
**Pattern**: Follow `KeywordAbility::Foretell` at line 472-480.

```rust
/// CR 702.170: Plot [cost] -- special action from hand; cast from exile for free.
///
/// Marker for quick presence-checking (`keywords.contains`).
/// The plot cost itself is stored in `AbilityDefinition::Plot { cost }`.
///
/// During the owner's main phase with empty stack, they may pay the plot cost
/// and exile this card face up (special action, CR 116.2k). On a later turn,
/// during their main phase with empty stack, they may cast it without paying
/// its mana cost (alternative cost, CR 702.170d).
Plot,
```

**Discriminant**: 97 (next after Blitz=96 in hash.rs).

#### 1b. `AltCostKind::Plot` variant

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/types.rs`
**Action**: Add `Plot` variant in `AltCostKind` enum (line ~111), replacing the
`// Future: Plot, Prototype, Impending` comment.
**Pattern**: Follow `AltCostKind::Foretell` at line 104.

```rust
Plot,
// Future: Prototype, Impending (add as implemented)
```

#### 1c. `AbilityDefinition::Plot { cost }` variant

**File**: `/home/airbaggie/scutemob/crates/engine/src/cards/card_definition.rs`
**Action**: Add `Plot` variant after `Blitz { cost }` (line ~350).
**Pattern**: Follow `AbilityDefinition::Foretell { cost }` at line 220-227.

```rust
/// CR 702.170: Plot [cost]. During your main phase with empty stack, pay [cost]
/// and exile this card from your hand face up. On a later turn, during your main
/// phase with empty stack, cast it without paying its mana cost (CR 702.170d).
///
/// Cards with this ability should also include
/// `AbilityDefinition::Keyword(KeywordAbility::Plot)` for quick
/// presence-checking without scanning all abilities.
Plot { cost: ManaCost },
```

**AbilityDefinition hash discriminant**: 30 (next after Blitz=29 in hash.rs).

#### 1d. `GameObject` fields for plotted status

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/game_object.rs`
**Action**: Add `is_plotted: bool` and `plotted_turn: u32` fields, following
the pattern of `is_foretold`/`foretold_turn` at line ~384-391.

```rust
/// CR 702.170a: If true, this card was plotted -- exiled face-up via the plot
/// special action. The card can be cast from exile without paying its mana cost
/// on any later turn (CR 702.170d).
///
/// Unlike `is_foretold`, the card is face-up in exile (public information).
/// Zone changes (CR 400.7) clear this -- but since plotted cards are in exile,
/// any zone change from exile clears this.
#[serde(default)]
pub is_plotted: bool,
/// CR 702.170d: The turn number when this card was plotted.
///
/// The card can only be cast "during any turn after the turn in which it became
/// plotted" -- i.e., on any turn where `state.turn.turn_number > plotted_turn`.
/// Zero means not plotted. Set alongside `is_plotted`.
#[serde(default)]
pub plotted_turn: u32,
```

#### 1e. `StackObject` field for plotted tracking

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/stack.rs`
**Action**: Add `was_plotted: bool` field after `was_blitzed` (line ~166).
**Pattern**: Follow `cast_with_foretell: bool` at line 111.

```rust
/// CR 702.170d: If true, this spell was cast from exile as a plotted card
/// (without paying its mana cost). Used by resolution.rs to know the spell
/// was plot-cast.
///
/// Must always be false for copies (`is_copy: true`) -- copies are not cast.
#[serde(default)]
pub was_plotted: bool,
```

#### 1f. Hash discriminants

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/hash.rs`
**Actions**:
- Add `KeywordAbility::Plot => 97u8.hash_into(hasher)` after Blitz (line ~528)
- Add `AbilityDefinition::Plot { cost } => { 30u8.hash_into(hasher); cost.hash_into(hasher); }` after Blitz (line ~3164)
- Add `self.is_plotted.hash_into(hasher)` and `self.plotted_turn.hash_into(hasher)` in `GameObject` hash (after `foreteld_turn`, line ~390 area)
- Add `self.was_plotted.hash_into(hasher)` in `StackObject` hash (after `was_blitzed`, line ~1631)

#### 1g. `GameEvent::CardPlotted` variant

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/events.rs`
**Action**: Add `CardPlotted` event after `CardForetold` (line ~776).
**Pattern**: Follow `CardForetold` at line 770-776.

```rust
// -- Plot events (CR 702.170) ----------------------------------------
/// A card was plotted -- exiled face-up from hand via the plot special
/// action (CR 702.170a / CR 116.2k). The plot cost was paid.
///
/// `new_exile_id` is the ObjectId of the card in the exile zone (new per CR 400.7).
/// Unlike foretell (face-down), plotted cards are face-up (public information).
CardPlotted {
    player: PlayerId,
    /// The card's ObjectId before exile (now retired).
    object_id: ObjectId,
    /// New ObjectId in the exile zone.
    new_exile_id: ObjectId,
},
```

**Note**: `CardPlotted` does NOT return `true` from `reveals_hidden_info()` because
the card is exiled face-up (public). Foretell returns `true` because it is face-down.

#### 1h. `Command::PlotCard` variant

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/command.rs`
**Action**: Add `PlotCard` command after `ForetellCard` (line ~352).
**Pattern**: Follow `ForetellCard` at line 347-352.

```rust
// -- Plot (CR 702.170) -----------------------------------------------
/// Plot a card from hand (CR 702.170a / CR 116.2k).
///
/// Special action: pay [plot cost], exile a card with plot from your hand face up.
/// This does not use the stack. Legal during your main phase while stack is empty.
/// The card can be cast without paying its mana cost on a future turn.
PlotCard { player: PlayerId, card: ObjectId },
```

#### 1i. Zone-change cleanup for `is_plotted` / `plotted_turn`

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/mod.rs`
**Action**: Add `is_plotted: false, plotted_turn: 0,` in BOTH `move_object_to_zone` sites
(line ~284 and ~379), following the pattern of `is_foretold: false, foretold_turn: 0`.

#### 1j. Builder initialization

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/builder.rs`
**Action**: Initialize `is_plotted: false, plotted_turn: 0` in `GameObjectBuilder` default.
**Grep**: Search for `is_foretold` in builder.rs to find the exact location.

#### 1k. Match arm additions

Files that exhaustively match `KeywordAbility`, `AltCostKind`, `StackObjectKind`,
`AbilityDefinition`, `Command`, or `GameEvent` need new arms:

- `crates/engine/src/rules/engine.rs`: Add `Command::PlotCard` handler arm (delegates to `plot::handle_plot_card`)
- `crates/engine/src/rules/abilities.rs`: Any exhaustive match on `StackObjectKind` (no new StackObjectKind needed for Plot)
- `crates/engine/src/rules/resolution.rs`: Exhaustive match on `StackObjectKind` (no new variant; but the Spell resolution path may need was_plotted awareness)
- `crates/engine/src/rules/casting.rs`: `AltCostKind::Plot` in alt_cost booleans, zone-bypass, mutual exclusion, cost determination
- `tools/tui/src/play/panels/stack_view.rs`: No change (no new StackObjectKind variant)
- `tools/replay-viewer/src/view_model.rs`: No change (no new StackObjectKind variant)
- `crates/engine/src/testing/replay_harness.rs`: Add `"plot_card"` action type
- Token creation in `effects/mod.rs`: Initialize `is_plotted: false, plotted_turn: 0`
- `crates/engine/src/cards/helpers.rs`: No change expected (Plot is not a type alias)

### Step 2: Rule Enforcement -- Plot Special Action Handler

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/plot.rs` (NEW FILE)
**Action**: Create `plot.rs` module implementing `handle_plot_card`.
**Pattern**: Follow `foretell.rs` at `/home/airbaggie/scutemob/crates/engine/src/rules/foretell.rs`

The handler validates and executes the plot special action:

```
pub fn handle_plot_card(
    state: &mut GameState,
    player: PlayerId,
    card: ObjectId,
) -> Result<Vec<GameEvent>, GameStateError>
```

Validation steps:
1. Player has priority (CR 116.2k).
2. It is the player's turn (CR 116.2k: "during their own turn").
3. It is a main phase (CR 702.170a: "during your main phase").
4. The stack is empty (CR 702.170a / CR 116.2k: "while the stack is empty").
5. The card is in the player's hand (CR 702.170a: "exile this card from your hand").
6. The card has `KeywordAbility::Plot` (CR 702.170a).
7. Player can pay the plot cost (from `AbilityDefinition::Plot { cost }`).

Execution:
1. Deduct the plot cost from the player's mana pool.
2. Emit `ManaCostPaid` event.
3. Record `current_turn = state.turn.turn_number`.
4. Move the card from hand to exile (`move_object_to_zone(card, ZoneId::Exile)`).
5. Set `is_plotted = true` and `plotted_turn = current_turn` on the new exile object.
6. Set `face_down = false` on the new exile object (face-up, public info -- CR 702.170a).
7. Emit `CardPlotted` event.

**CR differences from Foretell**:
- Plot requires main phase + empty stack (sorcery speed). Foretell only requires your turn.
- Plot exiles face-UP. Foretell exiles face-DOWN.
- Plot cost is variable (per card). Foretell cost is always {2}.
- Plot cost is looked up from `AbilityDefinition::Plot { cost }`. Foretell is hardcoded {2}.

**Register module**: Add `pub mod plot;` to `/home/airbaggie/scutemob/crates/engine/src/rules/mod.rs`

### Step 3: Rule Enforcement -- Plot Free Cast via CastSpell

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/casting.rs`
**Action**: Add `AltCostKind::Plot` handling throughout the casting pipeline.
**Pattern**: Follow the `AltCostKind::Foretell` pattern at lines 74, 181-199, 273, 559-596, 900-909, 1487.

The changes map exactly to how Foretell is wired, with key differences:

#### 3a. Alt-cost boolean derivation (line ~80)

```rust
let cast_with_plot = alt_cost == Some(AltCostKind::Plot);
```

#### 3b. Cast-from-exile validation (line ~181 area)

After the foretell validation block:

```rust
// CR 702.170d: Plot -- allowed if cast_with_plot is true.
// Card must be in ZoneId::Exile with is_plotted == true and plotted on a prior turn.
if cast_with_plot {
    if card_obj.zone != ZoneId::Exile {
        return Err(GameStateError::InvalidCommand(
            "plot: card must be in exile (CR 702.170d)".into(),
        ));
    }
    if !card_obj.is_plotted {
        return Err(GameStateError::InvalidCommand(
            "plot: card was not plotted (CR 702.170d)".into(),
        ));
    }
    if card_obj.plotted_turn >= state.turn.turn_number {
        return Err(GameStateError::InvalidCommand(
            "plot: cannot cast plotted card on the same turn it was plotted (CR 702.170d: 'any turn after the turn in which it became plotted')".into(),
        ));
    }
}
```

#### 3c. Zone-bypass (line ~267-282)

Add `&& !cast_with_plot` to the `card_obj.zone != ZoneId::Hand(player)` check,
so that casting from exile via Plot doesn't fail the hand-zone requirement.

#### 3d. Sorcery-speed enforcement

Plot's free-cast is already sorcery-speed (main phase, empty stack) per CR 702.170d.
The existing sorcery-speed check in casting.rs handles this automatically for
non-instant, non-flash spells. But a plotted sorcery-speed spell being cast during
a main phase with empty stack should pass. Verify: no special handling needed because
the card's original timing rules apply (sorcery-speed cards need sorcery timing,
instant-speed cards can still only be plot-cast at sorcery speed per CR 702.170d).

Actually, CR 702.170d says "during their main phase while the stack is empty" --
this is sorcery-speed timing. Even instants can only be plot-cast at sorcery speed.
This might need special enforcement: if a plotted card is an instant, the player
cannot cast it during an opponent's turn or with things on the stack. The existing
casting.rs checks should handle this IF we add a sorcery-speed restriction for
plot-cast spells. Check: the existing code allows instants at any time with priority.
For Plot, we need to restrict to main phase + empty stack.

**Enforcement**: After the alt-cost validation, if `cast_with_plot` is true,
verify the timing is sorcery-speed:
```rust
if cast_with_plot {
    // CR 702.170d: Plot free-cast timing = main phase + empty stack
    if !is_main_phase(&state) || !state.stack_objects.is_empty() {
        return Err(GameStateError::InvalidCommand(
            "plot: plotted cards can only be cast during your main phase while the stack is empty (CR 702.170d)".into(),
        ));
    }
    if state.turn.active_player != player {
        return Err(GameStateError::InvalidCommand(
            "plot: plotted cards can only be cast during your turn (CR 702.170d)".into(),
        ));
    }
}
```

#### 3e. Mutual exclusion (CR 118.9a)

Add a new Step 1k validation block for Plot (after Blitz at line ~845):

```rust
let casting_with_plot = if cast_with_plot {
    // Validate mutual exclusion with all other alternative costs
    // (same pattern as foretell/dash/blitz blocks)
    ...
    true
} else {
    false
};
```

Must check mutual exclusion against: flashback, evoke, bestow, madness, miracle,
escape, foretell, overload, retrace, jump-start, aftermath, dash, blitz.
And add `casting_with_plot` checks to all existing alt-cost validation blocks.

#### 3f. Cost determination (line ~935 area)

After the blitz cost block:

```rust
} else if casting_with_plot {
    // CR 702.170d: Cast without paying mana cost (alternative cost).
    // Cost is zero. Mandatory additional costs (kicker) still apply.
    Some(ManaCost::default())
}
```

This is the key difference from Foretell: Foretell pays the foretell cost; Plot pays zero.

#### 3g. StackObject creation (line ~1487 area)

Add `was_plotted: casting_with_plot,` to the StackObject struct literal.

#### 3h. Storm/Cascade StackObject creation (line ~1600 and ~1647 areas)

Add `was_plotted: false,` to the StackObject literals for triggered abilities.

#### 3i. `get_plot_cost` helper function

Add a function at the bottom of casting.rs (following `get_blitz_cost` pattern):

```rust
/// CR 702.170a: Look up the plot cost from the card's `AbilityDefinition`.
fn get_plot_cost(
    card_id: &Option<CardId>,
    registry: &CardRegistry,
) -> Option<ManaCost> {
    card_id.as_ref().and_then(|cid| {
        registry.get(cid.clone()).and_then(|def| {
            def.abilities.iter().find_map(|a| {
                if let AbilityDefinition::Plot { cost } = a {
                    Some(cost.clone())
                } else {
                    None
                }
            })
        })
    })
}
```

This function is used by `plot.rs` (Step 2) to look up the cost for the plot action.
Import it as `pub(crate)` or have `plot.rs` call it directly.

### Step 3b (Trigger Wiring): N/A

Plot has NO triggered abilities. The special action (plot) and the free-cast
(CastSpell with AltCostKind::Plot) are both player-initiated. No triggers to wire.

### Step 4: Command Handler Wiring

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/engine.rs`
**Action**: Add `Command::PlotCard` handler arm.
**Pattern**: Follow `Command::ForetellCard` at line ~302-307.

```rust
// -- Plot (CR 702.170) ----------------------------------------
Command::PlotCard { player, card } => {
    validate_player_active(&state, player)?;
    // CR 104.4b: plotting is a meaningful player choice; reset loop detection.
    loop_detection::reset_loop_detection(&mut state);
    let events = plot::handle_plot_card(&mut state, player, card)?;
    all_events.extend(events);
    // CR 116.3: Special action => player receives priority afterward.
    // Priority is already set to the player since they took the action.
}
```

**Note**: Like Foretell, the plot special action does NOT need `check_triggers` +
`flush_pending_triggers` because plotting itself doesn't create any triggers.
However, if future cards add "whenever you plot a card" triggers, this would need
the trigger flush pair. For now, no triggers.

### Step 5: Replay Harness Action Type

**File**: `/home/airbaggie/scutemob/crates/engine/src/testing/replay_harness.rs`
**Action**: Add `"plot_card"` action type.
**Pattern**: Follow `"foretell_card"` at line ~685-690.

```rust
"plot_card" => {
    let card_id = find_in_hand(state, player, card_name?)?;
    Some(Command::PlotCard {
        player,
        card: card_id,
    })
}
```

Also add `"cast_spell_plot"` for the free-cast from exile:

```rust
"cast_spell_plot" => {
    // Find the plotted card in exile
    let card_id = find_in_exile(state, player, card_name?)?;
    Some(Command::CastSpell {
        player,
        card: card_id,
        alt_cost: Some(AltCostKind::Plot),
        ..Default::default()
    })
}
```

**Note**: Need to verify whether `find_in_exile` helper exists. If not, add it
following the `find_in_hand` pattern but searching `ZoneId::Exile` objects.
Grep for `find_in_exile` or `find_in_graveyard` to find the pattern.

### Step 6: Unit Tests

**File**: `/home/airbaggie/scutemob/crates/engine/tests/plot.rs` (NEW FILE)
**Pattern**: Follow `/home/airbaggie/scutemob/crates/engine/tests/foretell.rs`

**Tests to write**:

1. **`test_plot_basic_exile_face_up`** -- CR 702.170a
   - Setup: Player has a card with Plot in hand, enough mana for plot cost
   - Action: PlotCard command
   - Assert: Card moves to exile, is_plotted = true, plotted_turn = current_turn, face_down = false
   - Assert: Plot cost deducted from mana pool
   - Assert: CardPlotted event emitted

2. **`test_plot_does_not_use_stack`** -- CR 702.170b
   - Setup: Player has a card with Plot in hand during main phase
   - Action: PlotCard command
   - Assert: Stack remains empty (special action, no stack involvement)

3. **`test_plot_cannot_cast_same_turn`** -- CR 702.170d
   - Setup: Player plots a card (card is in exile with is_plotted = true, plotted_turn = current turn)
   - Action: CastSpell with AltCostKind::Plot on the SAME turn
   - Assert: Error -- cannot cast plotted card on the same turn it was plotted

4. **`test_plot_cast_from_exile_on_later_turn`** -- CR 702.170d
   - Setup: Player has a plotted card in exile (plotted_turn < current turn number)
   - Action: CastSpell with AltCostKind::Plot
   - Assert: Spell is cast without paying mana cost (zero mana consumed)
   - Assert: Card moves from exile to stack then resolves to battlefield (if permanent)

5. **`test_plot_free_cast_costs_zero`** -- CR 702.170d
   - Setup: Player has a plotted creature with mana cost {3}{R}{R} in exile
   - Action: CastSpell with AltCostKind::Plot, player has 0 mana
   - Assert: Cast succeeds (mana cost = 0, no mana needed)

6. **`test_plot_requires_main_phase_empty_stack`** -- CR 702.170a / CR 116.2k
   - Setup: Player has a card with Plot, but it is not main phase or stack not empty
   - Action: PlotCard command
   - Assert: Error -- must be main phase with empty stack

7. **`test_plot_free_cast_requires_sorcery_timing`** -- CR 702.170d
   - Setup: Player has a plotted instant in exile, but it is not their main phase
     (or stack is not empty)
   - Action: CastSpell with AltCostKind::Plot
   - Assert: Error -- plotted cards can only be cast during main phase with empty stack

8. **`test_plot_requires_player_turn`** -- CR 116.2k
   - Setup: It is another player's turn
   - Action: PlotCard command
   - Assert: Error -- can only plot during your own turn

9. **`test_plot_requires_plot_keyword`** -- CR 702.170a
   - Setup: Card in hand does NOT have KeywordAbility::Plot
   - Action: PlotCard command
   - Assert: Error -- card does not have Plot

10. **`test_plot_requires_card_in_hand`** -- CR 702.170a
    - Setup: Card is on battlefield (not in hand)
    - Action: PlotCard command
    - Assert: Error -- card must be in hand

11. **`test_plot_insufficient_mana`** -- CR 702.170a
    - Setup: Card has Plot cost {1}{R}, player has only {R}
    - Action: PlotCard command
    - Assert: Error -- insufficient mana

12. **`test_plot_mutual_exclusion_with_foretell`** -- CR 118.9a
    - Setup: Plotted card in exile
    - Action: CastSpell with AltCostKind::Foretell (should fail since it is plotted, not foretold)
    - Assert: Error -- wrong alt cost flag (not foretold)

13. **`test_plot_mutual_exclusion_with_flashback`** -- CR 118.9a
    - Setup: Plotted card in exile
    - Action: CastSpell with AltCostKind::Plot + AltCostKind::Flashback (should fail)
    - Assert: Error -- cannot combine alternative costs
    - Note: Since alt_cost is `Option<AltCostKind>` (single value), mutual exclusion is
      implicit -- you pick one. But verify that a card with both Plot and Flashback
      can't bypass cost via an unexpected interaction.

14. **`test_plot_card_identity_tracking`** -- CR 400.7
    - Setup: Plot a card, verify the new ObjectId in exile is different from the hand ObjectId
    - Assert: Old ObjectId gone, new ObjectId in exile zone has is_plotted = true

15. **`test_plot_mana_value_unchanged`** -- CR 118.9c
    - Setup: Cast a plotted card (mana cost {3}{R}{R}, mana value 5)
    - Assert: While on the stack, the spell's mana value is still 5 (not 0)

### Step 7: Card Definition

**Suggested card**: Slickshot Show-Off
**Oracle text**: Flying, haste / Whenever you cast a noncreature spell, this creature
gets +2/+0 until end of turn. / Plot {1}{R}
**Type**: Creature -- Bird Wizard, 1/2
**Mana cost**: {1}{R}
**Plot cost**: {1}{R}

This card is ideal because:
- Simple creature with straightforward abilities (Flying, Haste)
- The triggered ability (noncreature spell -> +2/+0) tests interaction with Plot
- Plot cost equals mana cost, making comparison tests easy
- Good for verifying that the free-cast still triggers "whenever you cast" abilities

**Card lookup**: use `card-definition-author` agent

### Step 8: Game Script

**Suggested scenario**: "Plot a creature, cast it for free on the next turn"
**Steps**:
1. Player 1 has Slickshot Show-Off in hand with {1}{R} available
2. Player 1 plots Slickshot Show-Off (pays {1}{R}, card exiled face-up)
3. Advance to Player 1's next turn (main phase)
4. Player 1 casts the plotted card from exile for free (0 mana)
5. Verify: Slickshot Show-Off enters the battlefield with flying and haste

**Subsystem directory**: `test-data/generated-scripts/stack/`

## Interactions to Watch

### Plot vs Other Alt Costs
- Plot free-cast is an alternative cost (CR 702.170d / CR 601.2b). Cannot combine with
  other alternative costs (Flashback, Evoke, Bestow, etc.) per CR 118.9a.
- Additional costs (Kicker, Convoke) ARE allowed with Plot free-cast.

### Plot vs Commander Tax
- If a commander has Plot, plotting it from hand moves it to exile (not the command zone).
  The commander zone-change SBA (CR 903.9a) applies: owner may choose to move it to the
  command zone. If they do, the card is no longer plotted.
- Commander tax does NOT apply to the plot special action (it's not casting).
- If somehow a commander becomes plotted in exile, casting it via Plot from exile bypasses
  the command zone. Commander tax should NOT apply (CR 903.8 applies to spells cast from
  command zone only). But the plot cost was already paid, and the free-cast has zero cost.

### Plot vs Timing Restrictions
- Both the plot action (CR 116.2k) and the free-cast (CR 702.170d) are sorcery-speed.
- Even instants can only be free-cast via Plot at sorcery speed.
- Flash does NOT override Plot's timing restriction.

### Plot vs Countering
- If a plotted spell is countered, it goes to the graveyard (not back to exile).
  No special "exile instead" rule like Flashback. The card loses is_plotted on zone change.

### Plot vs Copy Effects
- A copy of a plotted card on the stack is not itself "plotted" (was_plotted = false
  for copies). Copies don't inherit cast-method flags.

### Plot vs Suspend
- Suspend also casts from exile for free, but through a different mechanism (trigger-based
  auto-cast vs player-initiated CastSpell). Both use the "without paying mana cost" pattern.
  A card cannot be both suspended and plotted simultaneously (different flags).

## File Change Summary

| File | Action | Scope |
|------|--------|-------|
| `crates/engine/src/state/types.rs` | Add `KeywordAbility::Plot`, `AltCostKind::Plot` | Enum variants |
| `crates/engine/src/cards/card_definition.rs` | Add `AbilityDefinition::Plot { cost }` | Enum variant |
| `crates/engine/src/state/game_object.rs` | Add `is_plotted`, `plotted_turn` fields | Struct fields |
| `crates/engine/src/state/stack.rs` | Add `was_plotted` field to `StackObject` | Struct field |
| `crates/engine/src/state/hash.rs` | Hash discriminants for all new types/fields | Hash impl |
| `crates/engine/src/state/mod.rs` | Reset `is_plotted`/`plotted_turn` on zone change (2 sites) | Zone-change cleanup |
| `crates/engine/src/state/builder.rs` | Initialize `is_plotted: false, plotted_turn: 0` | Builder default |
| `crates/engine/src/rules/events.rs` | Add `GameEvent::CardPlotted` | Event variant |
| `crates/engine/src/rules/command.rs` | Add `Command::PlotCard` | Command variant |
| `crates/engine/src/rules/plot.rs` | **NEW**: `handle_plot_card` handler | Special action handler |
| `crates/engine/src/rules/mod.rs` | Add `pub mod plot;` | Module registration |
| `crates/engine/src/rules/engine.rs` | Add `Command::PlotCard` handler arm | Command dispatch |
| `crates/engine/src/rules/casting.rs` | Add Plot alt-cost pipeline (8 sites) | Cast validation |
| `crates/engine/src/testing/replay_harness.rs` | Add `"plot_card"`, `"cast_spell_plot"` actions | Script support |
| `crates/engine/src/effects/mod.rs` | Initialize `is_plotted: false, plotted_turn: 0` in token creation | Token defaults |
| `crates/engine/tests/plot.rs` | **NEW**: 15 unit tests | Test file |

**Total new files**: 2 (plot.rs, tests/plot.rs)
**Total modified files**: ~14
