# Ability Plan: Miracle

**Generated**: 2026-02-27
**CR**: 702.94
**Priority**: P3
**Similar abilities studied**: Madness (CR 702.35) -- static ability linked to triggered ability, alternative cost casting from non-standard zone, custom StackObjectKind, auto-decline on resolution. Files: `crates/engine/src/rules/casting.rs`, `crates/engine/src/rules/abilities.rs`, `crates/engine/src/rules/resolution.rs`, `crates/engine/src/state/stack.rs`, `crates/engine/src/state/stubs.rs`, `crates/engine/tests/madness.rs`.

## CR Rule Text

### 702.94 Miracle

**702.94a** Miracle is a static ability linked to a triggered ability. (See rule 603.11.) "Miracle [cost]" means "You may reveal this card from your hand as you draw it if it's the first card you've drawn this turn. When you reveal this card this way, you may cast it by paying [cost] rather than its mana cost."

**702.94b** If a player chooses to reveal a card using its miracle ability, they play with that card revealed until that card leaves their hand, that ability resolves, or that ability otherwise leaves the stack. (See rule 701.20a.)

### Related Rules

**603.11** Some objects have a static ability that's linked to one or more triggered abilities. (See rule 607, "Linked Abilities.") These objects combine the abilities into one paragraph, with the static ability first, followed by each triggered ability that's linked to it.

**701.20a** (Reveal) To reveal a card, show that card to all players for a brief time. [...] If revealing a card causes a triggered ability to trigger, the card remains revealed until that triggered ability leaves the stack. If that ability isn't put onto the stack the next time a player would receive priority, the card ceases to be revealed.

## Key Edge Cases

From card rulings (Terminus, Entreat the Angels, Bonfire of the Damned):

1. **Miracle is an alternative cost (CR 118.9)** -- it cannot be combined with other alternative costs (flashback, evoke, bestow, madness). The spell's mana value remains unchanged (based on printed mana cost, not miracle cost).

2. **If the card leaves the hand before the trigger resolves**, the player cannot cast it using miracle. The trigger does nothing.

3. **Cast timing**: "You cast the card with miracle during the resolution of the triggered ability. **Ignore any timing rules based on the card's type.**" This means sorceries can be cast at instant speed when miracled. This parallels madness.

4. **First card drawn this turn**: "Multiple card draws are always treated as a sequence of individual card draws. [...] Only the first card drawn this way may be revealed and cast using its miracle ability." The engine already tracks `cards_drawn_this_turn` per player.

5. **Works on any turn**: "You can reveal and cast a card with miracle on any turn, not just your own, if it's the first card you've drawn that turn."

6. **Revealing is optional**: "You don't have to reveal a drawn card with miracle if you don't wish to cast it at that time."

7. **Still draws the card**: "You still draw the card, whether you use the miracle ability or not." The draw happens before the reveal decision.

8. **"If an effect puts a card into your hand without using the word 'draw,' the card wasn't drawn."** Only actual draws (library top -> hand via the draw mechanism) trigger miracle, not effects like bounce.

9. **Multiplayer**: All players could miracle on any player's turn, as long as it's the first card they drew that turn. Commander multiplayer has no special interaction beyond this.

## Current State (from ability-wip.md)

- [ ] Step 1: Enum variant
- [ ] Step 2: Rule enforcement
- [ ] Step 3: Trigger wiring
- [ ] Step 4: Unit tests
- [ ] Step 5: Card definition
- [ ] Step 6: Game script
- [ ] Step 7: Coverage doc update

## Implementation Steps

### Step 1: Enum Variant and AbilityDefinition

**File**: `crates/engine/src/state/types.rs`
**Action**: Add `KeywordAbility::Miracle` variant after `KeywordAbility::Madness` (line ~372).
**Pattern**: Follow `KeywordAbility::Madness` at line 372.

```rust
/// CR 702.94: Miracle [cost] -- static ability linked to triggered ability.
/// "You may reveal this card from your hand as you draw it if it's the first
/// card you've drawn this turn. When you reveal this card this way, you may
/// cast it by paying [cost] rather than its mana cost."
///
/// Marker for quick presence-checking (`keywords.contains`).
/// The miracle cost itself is stored in `AbilityDefinition::Miracle { cost }`.
Miracle,
```

**File**: `crates/engine/src/cards/card_definition.rs`
**Action**: Add `AbilityDefinition::Miracle { cost: ManaCost }` variant after `Madness { cost }` (line ~192).
**Pattern**: Follow `AbilityDefinition::Madness { cost }` at line 192.

```rust
/// CR 702.94: Miracle [cost]. When this card is drawn as the first card of
/// the turn, the player may reveal it and cast it by paying [cost] instead
/// of its mana cost (alternative cost, CR 118.9).
///
/// Cards with this ability should also include
/// `AbilityDefinition::Keyword(KeywordAbility::Miracle)` for quick
/// presence-checking without scanning all abilities.
Miracle { cost: ManaCost },
```

**File**: `crates/engine/src/state/hash.rs`
**Action**: Add hash discriminant 49 for `KeywordAbility::Miracle` after `Madness => 48u8` (line ~387).

```rust
// Miracle (discriminant 49) -- CR 702.94
KeywordAbility::Miracle => 49u8.hash_into(hasher),
```

**File**: `tools/replay-viewer/src/view_model.rs`
**Action**: Add display arm for `KeywordAbility::Miracle` after `Madness` (line ~618).

```rust
KeywordAbility::Miracle => "Miracle".to_string(),
```

### Step 2: Command and Event Infrastructure

Miracle needs:
- A new `GameEvent::MiracleRevealChoiceRequired` event emitted when a miracle-eligible draw occurs.
- A new `Command::ChooseMiracle` command for the player's decision.
- A new `StackObjectKind::MiracleTrigger` to carry the miracle data on the stack.

#### Step 2a: MiracleRevealChoiceRequired Event

**File**: `crates/engine/src/rules/events.rs`
**Action**: Add a new event variant. Place it near `DredgeChoiceRequired` (line ~710).
**CR**: 702.94a -- the player "may reveal this card" -- this requires a player choice.

```rust
/// CR 702.94a: A card with miracle was drawn as the first card this turn.
/// The player may choose to reveal it and trigger the miracle ability.
///
/// The engine pauses until a `Command::ChooseMiracle` is received.
/// `card_object_id` is the drawn card's new ObjectId in hand.
/// `miracle_cost` is the miracle alternative cost from the card definition.
MiracleRevealChoiceRequired {
    player: PlayerId,
    card_object_id: ObjectId,
    miracle_cost: ManaCost,
},
```

**File**: `crates/engine/src/state/hash.rs`
**Action**: Add hash arm for this new event variant. Next discriminant after existing events.

#### Step 2b: ChooseMiracle Command

**File**: `crates/engine/src/rules/command.rs`
**Action**: Add a new command variant after `ChooseDredge` (line ~249).
**CR**: 702.94a -- "You may reveal this card."

```rust
/// Choose whether to reveal a miracle card drawn as the first card this turn (CR 702.94a).
///
/// Sent in response to a `MiracleRevealChoiceRequired` event. If `reveal` is `true`,
/// the card is revealed and a miracle trigger is placed on the stack. When that
/// trigger resolves, the player may cast the spell by paying the miracle cost.
/// If `reveal` is `false`, the card stays in hand as a normal draw.
///
/// Validation: the card must be in the player's hand with `KeywordAbility::Miracle`,
/// and `cards_drawn_this_turn` must be 1 (first draw).
ChooseMiracle {
    player: PlayerId,
    /// The drawn card in hand. If `reveal` is false, this field is ignored but should
    /// match the `card_object_id` from the `MiracleRevealChoiceRequired` event.
    card: ObjectId,
    /// True = reveal and put miracle trigger on stack. False = decline (normal draw).
    reveal: bool,
},
```

#### Step 2c: MiracleTrigger StackObjectKind

**File**: `crates/engine/src/state/stack.rs`
**Action**: Add a new `StackObjectKind::MiracleTrigger` variant after `MadnessTrigger` (line ~176).
**CR**: 702.94a -- "When you reveal this card this way, you may cast it by paying [cost]."

```rust
/// CR 702.94a: Miracle triggered ability on the stack.
///
/// When a player reveals a card using its miracle ability, this trigger fires:
/// "When you reveal this card this way, you may cast it by paying [cost] rather
/// than its mana cost."
///
/// `revealed_card` is the ObjectId of the card in hand (new ID after draw zone move).
/// `miracle_cost` is captured at trigger time from the card definition.
/// When this trigger resolves, the player may cast the card for `miracle_cost`.
/// If they decline, the card stays in hand normally.
MiracleTrigger {
    source_object: ObjectId,
    revealed_card: ObjectId,
    miracle_cost: ManaCost,
    owner: PlayerId,
},
```

**File**: `crates/engine/src/state/hash.rs`
**Action**: Add hash arm for `StackObjectKind::MiracleTrigger` in the existing StackObjectKind match.

#### Step 2d: PendingTrigger Fields

**File**: `crates/engine/src/state/stubs.rs`
**Action**: Add `is_miracle_trigger: bool`, `miracle_revealed_card: Option<ObjectId>`, `miracle_cost: Option<ManaCost>` fields to `PendingTrigger` after the madness fields (line ~108). These follow the exact same pattern as the madness fields.
**Pattern**: Follow `is_madness_trigger`, `madness_exiled_card`, `madness_cost` at lines 91-108.

```rust
/// CR 702.94a: If true, this pending trigger is a Miracle trigger.
///
/// When flushed to the stack, creates a `StackObjectKind::MiracleTrigger`
/// instead of the normal `StackObjectKind::TriggeredAbility`. The fields
/// `miracle_revealed_card` and `miracle_cost` carry the miracle-specific data.
/// The `ability_index` field is unused when this is true.
#[serde(default)]
pub is_miracle_trigger: bool,
/// CR 702.94a: ObjectId of the revealed card in hand.
///
/// Only meaningful when `is_miracle_trigger` is true.
#[serde(default)]
pub miracle_revealed_card: Option<ObjectId>,
/// CR 702.94a: The miracle alternative cost captured at trigger time.
///
/// Only meaningful when `is_miracle_trigger` is true.
#[serde(default)]
pub miracle_cost: Option<ManaCost>,
```

**IMPORTANT**: Every existing `PendingTrigger` literal construction must add `is_miracle_trigger: false, miracle_revealed_card: None, miracle_cost: None`. Grep for all `PendingTrigger {` constructions and add these fields. There are approximately 10-15 sites in `abilities.rs`.

### Step 3: Draw-Site Miracle Detection

The miracle check must fire at ALL draw sites where `cards_drawn_this_turn` is incremented. These are:

1. **`rules/turn_actions.rs:draw_card()`** (line ~145-158) -- draw-step draw
2. **`effects/mod.rs:draw_one_card()`** (line ~1856-1904) -- effect-based draw (e.g., "draw 3 cards")
3. **`rules/replacement.rs:draw_card_skipping_dredge()`** (line ~1555-1564) -- post-dredge-decline normal draw

**NOT** at `commander.rs:draw_7_cards()` (mulligan draws -- pregame, CR 103.5 -- miracle does not apply).

#### Step 3a: Miracle Check Helper

**File**: `crates/engine/src/rules/abilities.rs` (or a new `crates/engine/src/rules/miracle.rs` module)
**Action**: Add a helper function that checks whether a just-drawn card triggers a miracle choice.
**CR**: 702.94a -- "first card you've drawn this turn."

```rust
/// CR 702.94a: Check if a just-drawn card has miracle and is eligible for reveal.
///
/// Returns `Some(MiracleRevealChoiceRequired)` if:
///   1. The card has `KeywordAbility::Miracle`.
///   2. `player.cards_drawn_this_turn == 1` (it was the first draw this turn).
///   3. The card has an `AbilityDefinition::Miracle { cost }` in the card registry.
///
/// Returns `None` if the card is not miracle-eligible.
pub fn check_miracle_eligible(
    state: &GameState,
    player: PlayerId,
    drawn_card_id: ObjectId,
) -> Option<GameEvent> {
    // Step 1: Was this the first draw of the turn?
    let cards_drawn = state.players.get(&player)?.cards_drawn_this_turn;
    if cards_drawn != 1 {
        return None;
    }

    // Step 2: Does the drawn card have the Miracle keyword?
    let obj = state.objects.get(&drawn_card_id)?;
    if !obj.characteristics.keywords.contains(&KeywordAbility::Miracle) {
        return None;
    }

    // Step 3: Look up the miracle cost from the card definition.
    let miracle_cost = obj.card_id.as_ref().and_then(|cid| {
        state.card_registry.get(cid.clone()).and_then(|def| {
            def.abilities.iter().find_map(|a| {
                if let AbilityDefinition::Miracle { cost } = a {
                    Some(cost.clone())
                } else {
                    None
                }
            })
        })
    })?;

    Some(GameEvent::MiracleRevealChoiceRequired {
        player,
        card_object_id: drawn_card_id,
        miracle_cost,
    })
}
```

#### Step 3b: Wire Into Draw Sites

**File**: `crates/engine/src/rules/turn_actions.rs`
**Action**: After the `CardDrawn` event is emitted (line ~155-158), call `check_miracle_eligible`. If it returns `Some(event)`, append that event to the return vector.

```rust
// After: Ok(vec![GameEvent::CardDrawn { player, new_object_id: new_id }])
// Becomes:
let mut events = vec![GameEvent::CardDrawn { player, new_object_id: new_id }];
if let Some(miracle_event) = abilities::check_miracle_eligible(state, player, new_id) {
    events.push(miracle_event);
}
Ok(events)
```

**File**: `crates/engine/src/effects/mod.rs`
**Action**: Same pattern in `draw_one_card()` (line ~1891-1899). After `CardDrawn`, check for miracle.

**File**: `crates/engine/src/rules/replacement.rs`
**Action**: Same pattern in `draw_card_skipping_dredge()` (line ~1555-1564). After `CardDrawn`, check for miracle.

### Step 4: ChooseMiracle Command Handler

**File**: `crates/engine/src/rules/mod.rs` (where `process_command` dispatches)
**Action**: Add a `Command::ChooseMiracle` arm to the command dispatch.
**CR**: 702.94a -- player reveals the card and a trigger goes on the stack.

The handler should:

1. Validate the card is in the player's hand.
2. Validate the card has `KeywordAbility::Miracle`.
3. Validate `cards_drawn_this_turn == 1` (still the first draw context).
4. If `reveal == false`: do nothing (card stays in hand, normal draw).
5. If `reveal == true`:
   a. Look up the miracle cost from the card registry.
   b. Push a `PendingTrigger` with `is_miracle_trigger: true`, `miracle_revealed_card: Some(card)`, `miracle_cost: Some(cost)` into `state.pending_triggers`.
   c. Call `flush_pending_triggers` to place the `MiracleTrigger` on the stack.
   d. Emit a `GameEvent::CardRevealed` or similar event (optional -- for display).
   e. Reset `players_passed` and grant priority to the active player.

**Pattern**: Follow `Command::ChooseDredge` handler in `replacement.rs` / `mod.rs`.

### Step 5: MiracleTrigger Resolution

**File**: `crates/engine/src/rules/resolution.rs`
**Action**: Add a `StackObjectKind::MiracleTrigger` arm to the resolution match (near `MadnessTrigger`, line ~603-640).
**CR**: 702.94a -- "you may cast it by paying [cost] rather than its mana cost."

The resolution handler should:

1. Check if the revealed card is still in the player's hand (CR 400.7 -- if it left hand, trigger does nothing).
2. **MVP**: Auto-cast the card for the miracle cost. OR: Auto-decline (card stays in hand). The player can also manually cast it from hand before the trigger resolves, using `CastSpell` with `cast_with_miracle: true`.

**Recommended approach (mirrors Madness pattern)**: Auto-decline on resolution. The player casts the card from hand using the normal `CastSpell` command before the trigger resolves. This is simpler and aligns with the Madness pattern.

However, the CR says "you may cast it **during resolution** of the triggered ability." This means casting must happen AS the trigger resolves, not before. Two options:

**Option A (Simpler, matches Madness MVP)**: During MiracleTrigger resolution, auto-decline. The player must have already cast the card (from hand) before the trigger resolved. This is technically incorrect per CR but matches the Madness MVP approach and can be corrected later.

**Option B (More correct)**: During MiracleTrigger resolution, check if the card is in hand. If so, automatically cast it for the miracle cost (move to stack, pay cost from pool). This requires the player to have sufficient mana in their pool at resolution time.

**Recommended: Option A** for initial implementation. Document the gap. The card is in hand and can be cast normally with `CastSpell` at any time. The miracle trigger's resolution is a fallback that does nothing if the card is still in hand (player chose not to miracle).

**WAIT -- actually, this needs more thought.** The miracle trigger fires on reveal. The cast happens during trigger resolution. The player has to reveal the card AS they draw it (before it mixes with other hand cards), and then when the trigger resolves, they choose to cast or not. If they don't cast, the card stays in hand.

The correct flow for MVP:
1. Player draws a card (first draw this turn) with Miracle.
2. Engine emits `MiracleRevealChoiceRequired` -- player chooses to reveal or not.
3. If revealed: `MiracleTrigger` goes on stack.
4. When `MiracleTrigger` resolves: player may cast the spell for miracle cost OR decline.
5. MVP auto-behavior at step 4: **attempt to cast automatically** if player has sufficient mana, OR let them use a separate `CastSpell` command with `cast_with_miracle: true` before the trigger resolves. The simplest approach: at resolution, emit a "you may cast this" event and let the player use `CastSpell` with `cast_with_miracle: true` while the trigger is still on stack. If they pass priority and the trigger resolves, the card stays in hand.

**Revised recommended approach**: On `MiracleTrigger` resolution, do nothing -- the card stays in hand. The player's window to cast is while the trigger is on the stack (they have priority). They use `CastSpell` with `cast_with_miracle: true` (a new flag on the `CastSpell` command, similar to `cast_with_evoke`). The casting.rs code recognizes this flag and uses the miracle cost instead of the mana cost. This is the cleanest integration.

### Step 5b: Miracle Alternative Cost in Casting

**File**: `crates/engine/src/rules/casting.rs`
**Action**: Add miracle cost support parallel to flashback/evoke/bestow/madness.
**CR**: 702.94a -- miracle cost is an alternative cost (CR 118.9).

1. Add `cast_with_miracle: bool` field to `Command::CastSpell` (with `#[serde(default)]`).
2. In `handle_cast_spell`:
   a. Validate that if `cast_with_miracle` is true, the card is in hand and has `KeywordAbility::Miracle`.
   b. Validate mutual exclusion with other alternative costs (CR 118.9a).
   c. Validate that a `MiracleTrigger` is on the stack for this card (the reveal happened).
   d. Use the miracle cost instead of the mana cost.
   e. **Ignore timing restrictions** (sorceries can be cast at instant speed, CR ruling).
3. Add `cast_with_miracle: bool` to `StackObject` (with `#[serde(default)]`).

**File**: `crates/engine/src/rules/command.rs`
**Action**: Add `#[serde(default)] cast_with_miracle: bool` to `Command::CastSpell` (line ~111, near `cast_with_bestow`).

**File**: `crates/engine/src/state/stack.rs`
**Action**: Add `#[serde(default)] pub cast_with_miracle: bool` to `StackObject` (line ~84, after `cast_with_madness`).

**File**: `crates/engine/src/state/hash.rs`
**Action**: Add `self.cast_with_miracle.hash_into(hasher)` to `StackObject`'s hash impl.

#### Casting Flow Detail

In `handle_cast_spell` (casting.rs):

```rust
// After existing madness zone check (~line 95):
// Miracle: card is in hand with miracle keyword + miracle trigger on stack.
let casting_with_miracle = if cast_with_miracle {
    if !card_obj.zone == ZoneId::Hand(player) {
        return Err(...);
    }
    if !card_obj.characteristics.keywords.contains(&KeywordAbility::Miracle) {
        return Err(...);
    }
    // Verify a MiracleTrigger for this card is on the stack.
    let has_miracle_trigger = state.stack_objects.iter().any(|so| {
        matches!(so.kind, StackObjectKind::MiracleTrigger { revealed_card, .. }
            if revealed_card == card /* compare by card_id or name */)
    });
    if !has_miracle_trigger {
        return Err(...);
    }
    true
} else {
    false
};
```

Then in the alternative cost selection (line ~260):
```rust
} else if casting_with_miracle {
    // CR 702.94a: Pay miracle cost instead of mana cost.
    get_miracle_cost(&card_id, &state.card_registry)
}
```

And add a `get_miracle_cost` helper function (mirrors `get_flashback_cost`).

Timing override (near line ~160):
```rust
// CR 702.94a ruling: Miracle ignores timing restrictions.
if casting_with_miracle {
    // Sorceries can be cast at instant speed when miracled.
    is_instant_speed = true; // or skip the sorcery-speed check
}
```

Mutual exclusion with other alt costs (near line ~188):
```rust
if casting_with_miracle {
    if casting_with_flashback || casting_with_evoke || casting_with_bestow || casting_with_madness {
        return Err(GameStateError::InvalidCommand(
            "cannot combine miracle with other alternative costs (CR 118.9a)".into(),
        ));
    }
}
```

### Step 5c: MiracleTrigger Stack Cleanup

When the `MiracleTrigger` resolves (player passed priority without casting):

**File**: `crates/engine/src/rules/resolution.rs`
**Action**: Add arm for `StackObjectKind::MiracleTrigger`.

```rust
StackObjectKind::MiracleTrigger {
    source_object: _,
    revealed_card,
    miracle_cost: _,
    owner: _,
} => {
    let controller = stack_obj.controller;
    // CR 702.94a: If the card is still in hand, the player chose not to
    // cast it via miracle. The card stays in hand. No action needed.
    // If the card has already been cast (moved to stack), nothing to do.
    events.push(GameEvent::AbilityResolved {
        controller,
        stack_object_id: stack_obj.id,
    });
}
```

Also add `MiracleTrigger { .. }` to the countering match arm (line ~742).

### Step 6: Flush Pending Triggers -- Miracle Branch

**File**: `crates/engine/src/rules/abilities.rs`
**Action**: In `flush_pending_triggers` (line ~1163), add a branch for `is_miracle_trigger` parallel to `is_madness_trigger` (line ~1174).

```rust
} else if trigger.is_miracle_trigger {
    StackObjectKind::MiracleTrigger {
        source_object: trigger.source,
        revealed_card: trigger.miracle_revealed_card.unwrap_or(trigger.source),
        miracle_cost: trigger.miracle_cost.clone().unwrap_or_default(),
        owner: trigger.controller,
    }
}
```

Also add `is_miracle_trigger: false, miracle_revealed_card: None, miracle_cost: None` to ALL existing `PendingTrigger` construction sites. Grep for `PendingTrigger {` in the codebase to find all sites (~15 locations across `abilities.rs`, `cycling` handler, etc.).

### Step 7: Unit Tests

**File**: `crates/engine/tests/miracle.rs`
**Tests to write**:

- `test_miracle_first_draw_emits_choice_event` -- CR 702.94a: Drawing a miracle card as the first card this turn emits `MiracleRevealChoiceRequired`.
- `test_miracle_second_draw_no_choice_event` -- CR 702.94a (negative): Drawing a miracle card as the second draw does NOT emit the choice event.
- `test_miracle_non_miracle_card_no_choice` -- CR 702.94a (negative): Drawing a non-miracle card emits no miracle event.
- `test_miracle_reveal_puts_trigger_on_stack` -- CR 702.94a: Choosing to reveal puts a `MiracleTrigger` on the stack.
- `test_miracle_decline_reveal_no_trigger` -- CR 702.94a: Choosing not to reveal results in normal draw (no trigger).
- `test_miracle_cast_for_miracle_cost` -- CR 702.94a: Casting the spell with `cast_with_miracle: true` uses the miracle cost (reduced mana payment).
- `test_miracle_sorcery_ignores_timing` -- CR ruling: A sorcery with miracle can be cast at instant speed when miracled (not active player's turn, stack not empty).
- `test_miracle_trigger_resolves_without_cast` -- CR 702.94a: If the player passes priority without casting, the trigger resolves and the card stays in hand.
- `test_miracle_card_leaves_hand_before_resolution` -- CR ruling: If the card leaves hand before the trigger resolves, casting is impossible.
- `test_miracle_cannot_combine_with_flashback` -- CR 118.9a: Cannot combine miracle with flashback.
- `test_miracle_mana_value_unchanged` -- CR 118.9c: Mana value is based on printed mana cost, not miracle cost.
- `test_miracle_opponent_turn_first_draw` -- CR ruling: Miracle works on any turn, including opponent's turn (e.g., opponent makes you draw via an effect).

**Pattern**: Follow tests in `crates/engine/tests/madness.rs` for structure: card definitions, helper functions, `process_command` calls, event/state assertions.

### Step 8: Card Definition (later phase)

**Suggested card**: **Terminus** -- `{4}{W}{W}` Sorcery, "Put all creatures on the bottom of their owners' libraries. Miracle {W}". This is an iconic miracle card with a powerful board-wipe effect and a dramatic miracle cost reduction (6 mana -> 1 mana). The effect (put all creatures on bottom of library) is a clean Effect to implement.

Alternative: **Devastation Tide** -- `{3}{U}{U}` Sorcery, "Return all nonland permanents to their owners' hands. Miracle {1}{U}". Simpler targeting (all nonland permanents).

**Card lookup**: Use `card-definition-author` agent with Terminus.

### Step 9: Game Script (later phase)

**Suggested scenario**: Player draws Terminus during draw step (first draw of turn), reveals it, and casts it for miracle cost {W}. All creatures on the battlefield are put on the bottom of their owners' libraries.

**Subsystem directory**: `test-data/generated-scripts/stack/`

## Interactions to Watch

1. **Miracle + Dredge**: If the player dredges instead of drawing, that is NOT a draw (CR 702.52a). Dredge does not increment `cards_drawn_this_turn`. If the player later draws a card (from another effect), THAT is the first draw and miracle would apply. The existing `cards_drawn_this_turn` counter already handles this correctly -- dredge does not increment it.

2. **Miracle + Replacement effects on draw**: If a draw is replaced by another effect (e.g., Rest in Peace + something), the replacement may not result in a "draw." If the replacement still results in drawing, miracle applies. If the replacement eliminates the draw entirely (SkipDraw), no card is drawn and miracle does not apply.

3. **Miracle + draw-step TBA**: The draw-step draw goes through `turn_actions::draw_card`. This is a TBA that fires when entering the draw step. The `MiracleRevealChoiceRequired` event must be emitted during `enter_step` processing, before priority is granted. The `process_command` loop must handle this event and pause for player input.

4. **Miracle timing vs stack state**: When the miracle trigger is on the stack, other players get priority. They could respond (e.g., Stifle the trigger). If the trigger is countered, the player cannot cast for miracle cost. The card stays in hand normally.

5. **Miracle cost + commander tax**: Commander tax applies on top of any alternative cost (CR 118.9d). If a commander with miracle is cast from hand for miracle cost, commander tax does NOT apply (the card is being cast from hand, not the command zone). But if somehow a commander were in hand and cast with miracle, tax = 0 (not from command zone).

6. **`cast_with_miracle` field propagation**: Add `cast_with_miracle: false` to ALL existing `StackObject` construction sites and `Command::CastSpell` construction sites. Grep for `cast_with_madness` to find all sites (there are ~15-20).

7. **Event ordering**: `CardDrawn` must be emitted BEFORE `MiracleRevealChoiceRequired`. The draw happens first; then the reveal decision. This matches CR 702.94a: "as you draw it."

## File Modification Summary

| File | Changes |
|------|---------|
| `crates/engine/src/state/types.rs` | Add `KeywordAbility::Miracle` variant |
| `crates/engine/src/cards/card_definition.rs` | Add `AbilityDefinition::Miracle { cost }` variant |
| `crates/engine/src/state/hash.rs` | Add hash arms for new types (3-4 sites) |
| `crates/engine/src/rules/events.rs` | Add `MiracleRevealChoiceRequired` event |
| `crates/engine/src/rules/command.rs` | Add `ChooseMiracle` command; add `cast_with_miracle` to `CastSpell` |
| `crates/engine/src/state/stack.rs` | Add `MiracleTrigger` kind; add `cast_with_miracle` field to `StackObject` |
| `crates/engine/src/state/stubs.rs` | Add miracle fields to `PendingTrigger` |
| `crates/engine/src/rules/abilities.rs` | Add miracle check helper; wire into `flush_pending_triggers`; update ALL `PendingTrigger` literals |
| `crates/engine/src/rules/casting.rs` | Add miracle alt-cost handling parallel to madness/flashback/evoke/bestow |
| `crates/engine/src/rules/resolution.rs` | Add `MiracleTrigger` resolution arm |
| `crates/engine/src/rules/turn_actions.rs` | Wire miracle check after draw |
| `crates/engine/src/effects/mod.rs` | Wire miracle check after effect-based draw |
| `crates/engine/src/rules/replacement.rs` | Wire miracle check after dredge-decline draw |
| `crates/engine/src/rules/mod.rs` | Add `ChooseMiracle` command dispatch |
| `tools/replay-viewer/src/view_model.rs` | Add Miracle display arm |
| `crates/engine/tests/miracle.rs` | New test file with 12 tests |

## Complexity Assessment

Miracle is moderately complex. It is structurally very similar to Madness:
- Both are static abilities linked to triggered abilities.
- Both involve alternative costs (CR 118.9).
- Both require a player choice event and a custom command.
- Both have custom `StackObjectKind` variants.
- Both ignore sorcery-speed timing restrictions.

The main additional complexity vs. Madness:
- Miracle triggers on DRAW, not DISCARD -- must wire into 3 draw sites instead of 2 discard sites.
- Miracle has a two-phase choice: first "do you reveal?" then "do you cast?" (Madness has just one: "do you cast?").
- The `cards_drawn_this_turn == 1` check adds a state dependency.

Estimated effort: ~3-4 hours for implementation + tests.
