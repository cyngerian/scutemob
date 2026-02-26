# Ability Plan: Dredge

**Generated**: 2026-02-26
**CR**: 702.52
**Priority**: P2
**Similar abilities studied**: Flashback (graveyard-zone ability, `AbilityDefinition::Flashback`), Cycling (`AbilityDefinition::Cycling`), WouldDraw replacement effects (SkipDraw path in `replacement.rs`)

## CR Rule Text

> **702.52.** Dredge
>
> **702.52a** Dredge is a static ability that functions only while the card with dredge is in a player's graveyard. "Dredge N" means "As long as you have at least N cards in your library, if you would draw a card, you may instead mill N cards and return this card from your graveyard to your hand."
>
> **702.52b** A player with fewer cards in their library than the number required by a dredge ability can't mill any of them this way.

## Key Edge Cases

From CR rules and card rulings (Life from the Loam, Stinkweed Imp, et al.):

1. **Dredge replaces draws from ANY source** -- not just the draw step. "Dredge can replace any card draw, not only the one during your draw step." (Ruling, 2024-01-12)
2. **One draw can't be replaced by multiple dredge abilities** -- each dredge card replaces at most one draw. (Ruling, 2024-01-12)
3. **Must have >= N cards in library** -- "You can't attempt to use a dredge ability if you don't have enough cards in your library." (Ruling, 2024-01-12; CR 702.52b)
4. **Multiple draws in a sequence** -- "If you're instructed to draw two cards and you replace the first draw with a dredge ability, another card with a dredge ability (including one that was milled by the first dredge ability) may be used to replace the second draw." (Ruling, 2024-01-12). This means dredge checks happen per-draw, not per-batch.
5. **"Draw" keyword required** -- "If an effect puts a card into your hand without specifically using the word 'draw,' you're not drawing a card. Dredge can't replace this event." (Ruling, 2024-01-12)
6. **Atomic action** -- "Once you've announced that you're applying a card's dredge ability to replace a draw, players can't take any actions until you've put that card into your hand and milled cards." (Ruling, 2024-01-12)
7. **Player CHOICE** -- Dredge says "you may instead". The player actively chooses to dredge instead of drawing. This is a replacement effect with a may-ability.
8. **CR 614.11** -- Draw replacement effects apply even if no cards could be drawn (empty library). But dredge additionally requires >= N cards, so an empty library means dredge can't apply (702.52b).
9. **Multiplayer** -- Each player's dredge options are checked when THEY would draw. No special multiplayer interactions beyond the usual per-player draw replacement.
10. **Interaction with other WouldDraw replacements** -- If Dredge + another WouldDraw replacement (e.g., Zur's Weirding, Notion Thief's draw redirect) both apply, the affected player chooses order (CR 616.1).
11. **Dredge does NOT count as drawing** -- The card returned to hand by dredge was not "drawn" (it was returned by a replacement effect). This matters for cards_drawn_this_turn and Sylvan Library (CC#33).

## Current State (from ability-wip.md)

- [ ] Step 1: Enum variant
- [ ] Step 2: Rule enforcement
- [ ] Step 3: Trigger wiring
- [ ] Step 4: Unit tests
- [ ] Step 5: Card definition
- [ ] Step 6: Game script
- [ ] Step 7: Coverage doc update

## Architecture Decision: Dredge as a Replacement Effect

### Why NOT use `ReplacementModification` / `state.replacement_effects`

The existing replacement effect system (`ReplacementTrigger::WouldDraw` + `ReplacementModification::SkipDraw`) handles permanent-based global replacements (e.g., "If a player would draw, skip that draw"). These are registered when a permanent enters the battlefield and deactivated when it leaves.

Dredge is fundamentally different:
- It functions **from the graveyard**, not from the battlefield.
- It requires the **player's choice** for each individual draw.
- Multiple dredge cards in the graveyard each represent a separate option.
- The set of available dredge options changes dynamically (cards enter/leave graveyard mid-draw-sequence).
- The card offering dredge **moves zones** as part of the replacement (graveyard to hand) -- it's not a static permanent generating the effect.

### Chosen Approach: Extend `DrawAction` with a `Dredge` variant

The cleanest approach is to integrate dredge into the existing `check_would_draw_replacement` function in `replacement.rs`. Before checking `state.replacement_effects`, scan the player's graveyard for objects with `KeywordAbility::Dredge(n)` where `n <= library_size`. Each eligible dredge card is a candidate replacement.

**Player choice mechanism**: When the player has at least one eligible dredge card in their graveyard, the engine must pause and ask the player whether they want to dredge (and which card) or draw normally. This is modeled as:

1. A new `DrawAction::DredgeAvailable` variant containing the list of eligible `(ObjectId, u32)` pairs (card + dredge amount).
2. A new `Command::ChooseDredge` variant: `{ player: PlayerId, card: Option<ObjectId> }` where `None` means "draw normally" and `Some(id)` means "dredge this card".
3. A new `GameEvent::DredgeChoiceRequired` variant emitted when dredge options exist.
4. A new `GameEvent::Dredged` variant emitted when dredge completes (mills N, returns card to hand).

The `draw_card` / `draw_one_card` call sites already check `check_would_draw_replacement` and return early on `DrawAction::Skip` or `DrawAction::NeedsChoice`. Adding `DrawAction::DredgeAvailable` follows the same pattern: emit the choice event and return, waiting for the player's command.

When `Command::ChooseDredge { card: Some(id) }` arrives:
1. Validate the card is in the player's graveyard with `KeywordAbility::Dredge(n)`.
2. Validate the player has >= n cards in library.
3. Mill n cards (reuse the existing `mill_cards` helper, but must make it `pub(crate)` or inline the logic).
4. Move the dredge card from graveyard to hand.
5. Emit `GameEvent::Dredged` + mill events.
6. Do NOT increment `cards_drawn_this_turn` (dredge is NOT drawing).

When `Command::ChooseDredge { card: None }` arrives:
1. Proceed with the normal draw (call `draw_card_inner` or equivalent).

### Interaction with existing `WouldDraw` replacements

If both dredge cards AND `state.replacement_effects` WouldDraw replacements are applicable, the player must choose among all of them (CR 616.1). This is handled by combining dredge options with registered replacement effects in `check_would_draw_replacement`. The choice event lists all options.

For simplicity in Phase 1: treat dredge options and registered WouldDraw replacements as an either/or choice. The `DrawAction::DredgeAvailable` variant includes both the dredge card list and any applicable replacement IDs, so the player can choose among all of them.

## Implementation Steps

### Step 1: Enum Variant — `KeywordAbility::Dredge(u32)`

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/types.rs`
**Action**: Add `Dredge(u32)` variant to `KeywordAbility` enum after `Cycling` (line ~201).
**Pattern**: Follow `KeywordAbility::Ward(u32)` at line 157 -- parameterized keyword with u32 payload.

```rust
/// CR 702.52: Dredge N -- if you would draw a card, you may instead mill N cards
/// and return this card from your graveyard to your hand. Functions only while
/// this card is in the graveyard. Requires >= N cards in library.
Dredge(u32),
```

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/hash.rs`
**Action**: Add `KeywordAbility::Dredge(n)` arm to `HashInto for KeywordAbility` (after `Cycling` at line 317). Use discriminant 29.

```rust
// Dredge (discriminant 29) -- CR 702.52
KeywordAbility::Dredge(n) => {
    29u8.hash_into(hasher);
    n.hash_into(hasher);
}
```

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/hash.rs`
**Action**: Also check if `AbilityDefinition` hash needs a new variant. Since we are using `AbilityDefinition::Keyword(KeywordAbility::Dredge(n))`, the existing `AbilityDefinition::Keyword` arm (discriminant 3) + `KeywordAbility::Dredge` hash arm covers it. No additional change needed.

**Match arm sweep**: Grep for all `match.*KeywordAbility` or `KeywordAbility::` match blocks in the codebase and add `Dredge(_)` arms where needed. Key locations:
- `state/hash.rs` (done above)
- Any display/debug formatting
- `state/builder.rs` `enrich_spec_from_def` -- keywords are pushed generically, no per-variant work needed
- `rules/combat.rs` -- no combat relevance for Dredge
- `rules/protection.rs` -- no protection relevance for Dredge

### Step 2: Command, Event, and DrawAction Variants

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/command.rs`
**Action**: Add `ChooseDredge` command variant after `CycleCard` (line ~186).

```rust
/// CR 702.52: Choose whether to dredge a card from the graveyard instead of drawing.
///
/// Sent in response to a `DredgeChoiceRequired` event. If `card` is `Some(id)`,
/// the player dredges that card (mills N, returns card to hand). If `card` is `None`,
/// the player draws normally.
ChooseDredge {
    player: PlayerId,
    /// The dredge card to return from graveyard, or None to draw normally.
    card: Option<ObjectId>,
},
```

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/events.rs`
**Action**: Add two event variants.

```rust
/// CR 702.52: One or more dredge cards are available in the player's graveyard
/// and the player must choose whether to dredge one or draw normally.
///
/// The engine pauses until a `Command::ChooseDredge` is received.
/// `options` lists `(ObjectId, u32)` pairs of (dredge card, dredge amount).
DredgeChoiceRequired {
    player: PlayerId,
    options: Vec<(ObjectId, u32)>,
},

/// CR 702.52: A player dredged a card -- milled N cards and returned the
/// dredge card from graveyard to hand instead of drawing.
Dredged {
    player: PlayerId,
    /// The dredge card that was returned to hand (new ObjectId after zone change).
    card_new_id: ObjectId,
    /// Number of cards milled.
    milled: u32,
},
```

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/hash.rs`
**Action**: Add hash arms for `GameEvent::DredgeChoiceRequired` and `GameEvent::Dredged` in the `HashInto for GameEvent` impl. Use next available discriminants.

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/replacement.rs`
**Action**: Add `DrawAction::DredgeAvailable` variant.

```rust
/// One or more dredge cards in the player's graveyard can replace this draw (CR 702.52).
/// Contains the `DredgeChoiceRequired` event to emit.
DredgeAvailable(GameEvent),
```

### Step 3: Dredge Detection in `check_would_draw_replacement`

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/replacement.rs`
**Action**: Extend `check_would_draw_replacement` (line 422) to scan the player's graveyard for dredge-eligible cards BEFORE checking `state.replacement_effects`.

Logic:
1. Count cards in the player's library: `let library_count = state.zones.get(&ZoneId::Library(player)).map(|z| z.len()).unwrap_or(0);`
2. Scan the player's graveyard objects for `KeywordAbility::Dredge(n)` where `n as usize <= library_count`.
3. Collect eligible `(ObjectId, u32)` pairs.
4. Also find applicable WouldDraw replacements from `state.replacement_effects` (existing logic).
5. Decision matrix:
   - No dredge options AND no WouldDraw replacements: `DrawAction::Proceed`
   - No dredge options, WouldDraw replacements exist: existing logic (Skip or NeedsChoice)
   - Dredge options exist, no WouldDraw replacements: `DrawAction::DredgeAvailable(DredgeChoiceRequired { player, options })`
   - Both dredge options AND WouldDraw replacements exist: `DrawAction::NeedsChoice(...)` with all options (CR 616.1). For Phase 1, emit `DredgeChoiceRequired` -- the player can choose dredge or normal draw (which then gets intercepted by WouldDraw replacements). This simplification works because the WouldDraw replacements will be re-checked on the actual draw if the player declines dredge.

**CR justification**: CR 702.52a says dredge generates a replacement effect ("you may instead"). CR 616.1 says when multiple replacements apply, the affected player chooses. However, since dredge gives the player a MAY choice (they can decline), and the engine already re-checks WouldDraw replacements on each draw attempt, the simplest correct implementation is:
- If dredge options exist, emit `DredgeChoiceRequired` and pause.
- If the player declines dredge (`ChooseDredge { card: None }`), proceed to the normal draw path which will re-check WouldDraw replacements.
- If the player accepts dredge (`ChooseDredge { card: Some(id) }`), perform the dredge (mill + return) and skip the draw entirely.

### Step 4: Command Handler for `ChooseDredge`

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/engine.rs`
**Action**: Add a match arm for `Command::ChooseDredge` in `process_command` (after the `CycleCard` arm, around line 139).

```rust
Command::ChooseDredge { player, card } => {
    let events = replacement::handle_choose_dredge(&mut state, player, card)?;
    all_events.extend(events);
}
```

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/replacement.rs`
**Action**: Add `handle_choose_dredge` function.

```rust
/// CR 702.52: Handle the player's choice to dredge or draw normally.
///
/// If `card` is `Some(id)`:
///   1. Validate the card is in the player's graveyard with Dredge(n).
///   2. Validate library has >= n cards.
///   3. Mill n cards from the top of the library.
///   4. Move the dredge card from graveyard to hand.
///   5. Emit Dredged event + CardMilled events.
///   6. Do NOT increment cards_drawn_this_turn.
///
/// If `card` is `None`:
///   Proceed with normal draw (call draw_card).
pub fn handle_choose_dredge(
    state: &mut GameState,
    player: PlayerId,
    card: Option<ObjectId>,
) -> Result<Vec<GameEvent>, GameStateError> { ... }
```

Key implementation details:
- When `card` is `Some(id)`: validate, mill, move card, emit events. The mill helper needs to be accessible -- either make `effects::mill_cards` `pub(crate)` or inline the mill logic.
- When `card` is `None`: call `turn_actions::draw_card(state, player)` to perform the normal draw (which re-checks WouldDraw replacements).
- After dredge completes, call `check_triggers()` + `flush_pending_triggers()` to catch any triggers from milling or the card entering hand (though dredge returning to hand is not an ETB trigger, mill-related triggers like Sidisi, Brood Tyrant might trigger in the future).

### Step 5: Wire `DredgeAvailable` into draw paths

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/turn_actions.rs` (line ~108)
**Action**: Add `DrawAction::DredgeAvailable(event) => return Ok(vec![event]),` to the match arm in `draw_card`.

**File**: `/home/airbaggie/scutemob/crates/engine/src/effects/mod.rs` (line ~1534)
**Action**: Add `DrawAction::DredgeAvailable(event) => return vec![event],` to the match arm in `draw_one_card`.

Both call sites already handle `DrawAction::Skip` and `DrawAction::NeedsChoice` by returning early with the event. `DredgeAvailable` follows the same pattern.

### Step 6: Unit Tests

**File**: `/home/airbaggie/scutemob/crates/engine/tests/dredge.rs` (new file)
**Pattern**: Follow `crates/engine/tests/cycling.rs` and `crates/engine/tests/flashback.rs` for structure.

**Tests to write**:

1. `test_dredge_basic_draw_step_replaces_draw` -- CR 702.52a
   - Setup: Player A has a Dredge 3 creature in graveyard, >= 3 cards in library.
   - During draw step, engine emits `DredgeChoiceRequired`.
   - Player sends `ChooseDredge { card: Some(id) }`.
   - Assert: 3 cards milled, dredge card in hand, no `CardDrawn` event, `cards_drawn_this_turn` unchanged.

2. `test_dredge_decline_draws_normally` -- CR 702.52a (may clause)
   - Setup: Same as above.
   - Player sends `ChooseDredge { card: None }`.
   - Assert: Normal draw occurs, `CardDrawn` event emitted, `cards_drawn_this_turn` incremented.

3. `test_dredge_insufficient_library_cards` -- CR 702.52b
   - Setup: Player A has a Dredge 5 creature in graveyard, only 3 cards in library.
   - During draw step, NO `DredgeChoiceRequired` emitted (dredge not eligible).
   - Normal draw proceeds.

4. `test_dredge_card_not_in_graveyard_no_choice` -- CR 702.52a "functions only while in graveyard"
   - Setup: Player A has a Dredge card on the battlefield (not in graveyard).
   - During draw step, no dredge options offered.

5. `test_dredge_multiple_cards_in_graveyard` -- Multiple dredge options
   - Setup: Player A has two dredge cards in graveyard (Dredge 3 and Dredge 5), >= 5 cards in library.
   - `DredgeChoiceRequired` emitted with both options.
   - Player chooses one; assert correct mill count.

6. `test_dredge_during_effect_draw_not_just_draw_step` -- Ruling: "Dredge can replace any card draw"
   - Setup: Player A has dredge card in graveyard, casts a "draw 2 cards" spell.
   - First draw: `DredgeChoiceRequired` emitted, player dredges.
   - Second draw: another `DredgeChoiceRequired` emitted (potentially with a newly-milled dredge card).
   - Validates per-draw dredge checks (CR 614.11a).

7. `test_dredge_milled_card_available_for_second_draw` -- Ruling: "a card with a dredge ability (including one that was milled by the first dredge ability) may be used to replace the second draw"
   - Setup: Player A has Dredge 3 card in graveyard, another Dredge 2 card 2nd from top of library.
   - First draw: dredge the Dredge 3 card, milling 3 cards (which include the Dredge 2 card).
   - Second draw: `DredgeChoiceRequired` shows the Dredge 2 card (newly in graveyard) as an option.

8. `test_dredge_not_a_draw_sylvan_library_counter` -- Ruling/CC#33: Dredge does not count as "drawing"
   - Setup: Player A dredges instead of drawing.
   - Assert: `cards_drawn_this_turn` is NOT incremented.

9. `test_dredge_empty_library_eligible_when_enough_cards` -- Edge: library has exactly N cards
   - Setup: Player A has Dredge 3 card in graveyard, exactly 3 cards in library.
   - `DredgeChoiceRequired` emitted (>= 3 satisfied).
   - After dredging, library is empty but no PlayerLost (didn't try to draw from empty).

10. `test_dredge_invalid_command_wrong_player` -- Error handling
    - Player B sends `ChooseDredge` when Player A has the choice.
    - Assert: error returned.

11. `test_dredge_invalid_command_card_not_in_graveyard` -- Error handling
    - Player sends `ChooseDredge { card: Some(id) }` for a card not in their graveyard.
    - Assert: error returned.

### Step 7: Card Definition (later phase)

**Suggested card**: Life from the Loam -- simple sorcery with Dredge 3, well-known Commander staple.
- Secondary candidates: Stinkweed Imp (creature, Dredge 5, also has Flying + combat trigger), Dakmor Salvage (land, Dredge 2).
**Card lookup**: Use `card-definition-author` agent.

### Step 8: Game Script (later phase)

**Suggested scenario**: "Player dredges Life from the Loam during draw step instead of drawing; mills 3 cards; Life from the Loam moves from graveyard to hand."
**Subsystem directory**: `test-data/generated-scripts/replacement/`

Second scenario: "Player has two dredge cards in graveyard, draws two cards from a spell, dredges different cards for each draw."

## Interactions to Watch

1. **Dredge + other WouldDraw replacements (CR 616.1)**: If a player has both dredge cards in graveyard and a WouldDraw replacement from a permanent (e.g., Notion Thief redirecting draws), the player chooses which to apply. The Phase 1 simplification (dredge choice first, then re-check WouldDraw on decline) is correct because dredge is a "may" -- declining dredge and proceeding to draw will encounter the WouldDraw replacement on the re-check.

2. **Dredge + empty library (CR 614.11 + 702.52b)**: Draw replacement effects apply even with empty library, but dredge specifically requires >= N cards. If library has 0 cards and a Dredge 3 card is in graveyard, dredge is NOT offered, and the draw attempt causes `PlayerLost` (CR 104.3b). If library has 2 cards and Dredge 3 is in graveyard, dredge is not offered (702.52b), and the draw succeeds normally (still has cards).

3. **Dredge + Sylvan Library (CC#33)**: Dredge does NOT count as drawing. If Sylvan Library cares about "cards drawn this draw step," dredged cards are not counted.

4. **Dredge does not trigger "draw" triggers**: Since dredge replaces the draw, abilities that trigger on drawing a card (e.g., Consecrated Sphinx) do NOT trigger for the dredge. The "return to hand" is not a draw.

5. **Object identity (CR 400.7)**: When the dredge card moves from graveyard to hand, it gets a new ObjectId. The `Dredged` event must report the new ObjectId.

6. **Mill triggers**: The N cards milled by dredge may trigger effects that care about cards entering the graveyard. The mill events (`CardMilled`) should fire normally, enabling future trigger support.

7. **Multiplayer**: Each player's dredge options are independent. During a multi-draw effect targeting all players (e.g., Windfall), each player's draws are handled sequentially in APNAP order, and each can choose to dredge per-draw.

## Files Modified (Summary)

| File | Change |
|------|--------|
| `crates/engine/src/state/types.rs` | Add `KeywordAbility::Dredge(u32)` |
| `crates/engine/src/state/hash.rs` | Add hash arms for `Dredge`, `DredgeChoiceRequired`, `Dredged`, `ChooseDredge` |
| `crates/engine/src/rules/command.rs` | Add `Command::ChooseDredge` |
| `crates/engine/src/rules/events.rs` | Add `GameEvent::DredgeChoiceRequired`, `GameEvent::Dredged` |
| `crates/engine/src/rules/replacement.rs` | Add `DrawAction::DredgeAvailable`, extend `check_would_draw_replacement`, add `handle_choose_dredge` |
| `crates/engine/src/rules/engine.rs` | Add `Command::ChooseDredge` match arm in `process_command` |
| `crates/engine/src/rules/turn_actions.rs` | Add `DrawAction::DredgeAvailable` match arm in `draw_card` |
| `crates/engine/src/effects/mod.rs` | Add `DrawAction::DredgeAvailable` match arm in `draw_one_card` |
| `crates/engine/tests/dredge.rs` | New test file with 11 tests |
