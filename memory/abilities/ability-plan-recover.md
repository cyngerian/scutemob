# Ability Plan: Recover

**Generated**: 2026-03-06
**CR**: 702.59
**Priority**: P4
**Similar abilities studied**: Echo (CR 702.30) — pay-or-sacrifice pattern in `engine.rs:491-560`, `resolution.rs:1358-1400`, `command.rs:548-560`, `events.rs:875-894`, `state/mod.rs:134-143`; Gravestorm — `permanents_put_into_graveyard_this_turn` tracking in `state/mod.rs:122-133`; CreatureDied trigger dispatch in `abilities.rs:2765-2853`

## CR Rule Text

702.59. Recover

702.59a Recover is a triggered ability that functions only while the card with recover is in a player's graveyard. "Recover [cost]" means "When a creature is put into your graveyard from the battlefield, you may pay [cost]. If you do, return this card from your graveyard to your hand. Otherwise, exile this card."

## Key Edge Cases

- **Trigger fires from the graveyard zone**: The card with Recover must be IN the graveyard when a creature enters the same player's graveyard from the battlefield. Unlike most triggered abilities (which fire from the battlefield), this fires from the graveyard.
- **"Your graveyard" means the card's owner's graveyard**: Both the Recover card AND the dying creature must end up in the same player's graveyard. The dying creature's controller is irrelevant -- what matters is which graveyard the creature goes to (the owner's).
- **Recover card itself dying does NOT trigger its own Recover**: If a creature with Recover dies, it is the creature entering the graveyard. But CR 400.7 applies -- the card is a new object in the graveyard and did not "see" the event that caused it to move. HOWEVER: 702.59a says "When a creature is put into your graveyard from the battlefield" -- the Recover card IS now in the graveyard when this happens (it just arrived). The key question is whether the trigger condition was met at the time the event occurred. Since the creature (itself) entering the graveyard IS the triggering event, and the Recover card IS in the graveyard at that point, the trigger should fire. This is correct per the CR -- the trigger checks if the card is in the graveyard at the time the event occurs, and since the creature and the Recover card arrive simultaneously, the card IS in the graveyard when the event resolves.
- **Multiple Recover cards**: If multiple non-creature Recover cards are in your graveyard and a creature you control dies, ALL of them trigger independently. Each one produces a separate trigger on the stack (APNAP order).
- **Multiple creatures dying simultaneously (SBA batch)**: Multiple creatures dying in the same SBA batch each trigger Recover independently. Each dying creature causes each Recover card in the owner's graveyard to trigger once. A single Recover card could trigger multiple times from a board wipe.
- **Pay or exile is mandatory if trigger resolves**: When the trigger resolves, the controller MUST either pay the cost (returning the card to hand) or exile it. There is no "do nothing" option. This is the "otherwise, exile" clause.
- **Recover card leaving graveyard before trigger resolves**: If the Recover card is no longer in the graveyard when the trigger resolves (e.g., exiled by another effect, Bojuka Bog), the trigger does nothing (CR 400.7 -- it's a new object).
- **Tokens dying DO trigger Recover**: Tokens briefly exist in the graveyard (CR 704.5d) before ceasing to exist as SBA. This is sufficient to trigger Recover.
- **Multiplayer**: Each player's Recover cards only trigger when creatures enter THAT PLAYER'S graveyard. Opponent creatures dying trigger the opponent's Recover cards, not yours.

## Current State (from ability-wip.md)

- [ ] Step 1: Enum variant
- [ ] Step 2: Rule enforcement
- [ ] Step 3: Trigger wiring
- [ ] Step 4: Unit tests
- [ ] Step 5: Card definition
- [ ] Step 6: Game script
- [ ] Step 7: Coverage doc update

## Implementation Steps

### Step 1: Enum Variant

**File**: `crates/engine/src/state/types.rs`
**Action**: Add `KeywordAbility::Recover` variant after `Escalate` (line ~1031). Use discriminant 112.
**Doc comment**: `/// CR 702.59a: Recover [cost] -- triggered ability from graveyard. When a creature is put into your graveyard from the battlefield, you may pay [cost]. If you do, return this card to hand. Otherwise, exile it. Static marker for quick presence-checking.`

**File**: `crates/engine/src/cards/card_definition.rs`
**Action**: Add `AbilityDefinition::Recover { cost: ManaCost }` variant after `CumulativeUpkeep` (line ~498). Use discriminant 41.
**Doc comment**:
```
/// CR 702.59a: Recover [cost]. Triggered ability from the graveyard. When a
/// creature is put into your graveyard from the battlefield, you may pay [cost].
/// If you do, return this card from your graveyard to your hand. Otherwise,
/// exile this card.
///
/// Cards with this ability should also include
/// `AbilityDefinition::Keyword(KeywordAbility::Recover)` for quick
/// presence-checking without scanning all abilities.
Recover { cost: ManaCost },
```

**File**: `crates/engine/src/state/hash.rs`
**Action**: Add hash arms for:
1. `KeywordAbility::Recover` in the KeywordAbility hash impl (after Escalate, discriminant 112)
2. `AbilityDefinition::Recover { cost }` in the AbilityDefinition hash impl (after Escalate, discriminant 41)
3. `StackObjectKind::RecoverTrigger { .. }` in the StackObjectKind hash impl (discriminant 42)

**File**: `crates/engine/src/state/stack.rs`
**Action**: Add `StackObjectKind::RecoverTrigger` variant:
```rust
/// CR 702.59a: Recover trigger. Fires when a creature enters the card owner's
/// graveyard from the battlefield and the Recover card is also in that graveyard.
/// On resolution, emit RecoverPaymentRequired and add to pending_recover_payments.
RecoverTrigger {
    source_object: ObjectId,
    /// The ObjectId of the Recover card in the graveyard (the trigger source).
    recover_card: ObjectId,
    /// The mana cost to pay for Recover.
    recover_cost: ManaCost,
},
```

**File**: `tools/tui/src/play/panels/stack_view.rs`
**Action**: Add match arm for `StackObjectKind::RecoverTrigger { recover_card, .. }`:
```rust
StackObjectKind::RecoverTrigger { recover_card, .. } => {
    ("Recover: ".to_string(), Some(*recover_card))
}
```

**Match arms**: Grep for all exhaustive `match` on `KeywordAbility`, `AbilityDefinition`, and `StackObjectKind` and add the new variant to each.

### Step 2: Pending Payment Infrastructure

**File**: `crates/engine/src/state/mod.rs`
**Action**: Add `pending_recover_payments` field to `GameState`, following the `pending_echo_payments` pattern (line ~143):
```rust
/// CR 702.59a: Pending recover payment choices.
///
/// When a RecoverTrigger resolves, the controller must choose to pay the
/// recover cost or exile the card. The game pauses until a
/// `Command::PayRecover` is received for each entry.
/// Each entry is `(player, recover_card_id, recover_cost)`.
#[serde(default)]
pub pending_recover_payments: im::Vector<(PlayerId, ObjectId, ManaCost)>,
```

**Also in `state/mod.rs`**: No need to reset in `reset_turn_state` -- pending payments persist across turns (they are interactive choices, not per-turn state).

**File**: `crates/engine/src/state/builder.rs`
**Action**: Initialize `pending_recover_payments: Vector::new()` in `build()` method (after `pending_echo_payments`, line ~346).

**File**: `crates/engine/src/state/hash.rs`
**Action**: Add `self.pending_recover_payments.hash_into(&mut hasher)` to the GameState hash impl (after the echo payments hash).

### Step 3: Command and Events

**File**: `crates/engine/src/rules/command.rs`
**Action**: Add `Command::PayRecover` variant after `PayCumulativeUpkeep`:
```rust
// -- Recover (CR 702.59) -------------------------------------------
/// Choose whether to pay the recover cost for a card in the graveyard (CR 702.59a).
///
/// Sent in response to a `RecoverPaymentRequired` event. If `pay` is true,
/// the player pays the recover cost (mana is deducted) and the card is returned
/// from the graveyard to the player's hand. If `pay` is false, the card is
/// exiled (CR 702.59a: "Otherwise, exile this card.").
PayRecover {
    player: PlayerId,
    recover_card: ObjectId,
    /// True = pay the recover cost and return to hand. False = exile the card.
    pay: bool,
},
```

**File**: `crates/engine/src/rules/events.rs`
**Action**: Add two events after the Echo events section:
```rust
// -- Recover events (CR 702.59) ------------------------------------
/// CR 702.59a: A recover trigger has resolved. The controller must choose
/// whether to pay the recover cost or exile the card.
///
/// The engine pauses until a `Command::PayRecover` is received.
RecoverPaymentRequired {
    player: PlayerId,
    recover_card: ObjectId,
    cost: ManaCost,
},
/// CR 702.59a: A player paid the recover cost for a card and it was
/// returned from the graveyard to the player's hand.
RecoverPaid {
    player: PlayerId,
    recover_card: ObjectId,
},
/// CR 702.59a: A player declined to pay the recover cost. The card
/// was exiled from the graveyard.
RecoverDeclined {
    player: PlayerId,
    recover_card: ObjectId,
    exiled_card: ObjectId,
},
```

### Step 4: Trigger Wiring (CreatureDied -> RecoverTrigger)

**File**: `crates/engine/src/rules/abilities.rs`
**Action**: In `check_triggers`, add a new block inside the `GameEvent::CreatureDied` arm (after the existing SelfDies trigger loop, around line ~2853). This block scans ALL cards in the dying creature's OWNER's graveyard for Recover:

```rust
// CR 702.59a: Check for Recover triggers on other cards in the same graveyard.
// When a creature enters a player's graveyard from the battlefield, all cards
// with Recover in that graveyard trigger (including the creature itself if it
// has Recover, since it is now in the graveyard when the event is processed).
{
    let owner_gy = ZoneId::Graveyard(/* dying creature's owner */);
    // Iterate all objects in the owner's graveyard looking for Recover.
    for (obj_id, obj) in state.objects.iter() {
        if obj.zone != owner_gy {
            continue;
        }
        // Skip the creature that just died -- its SelfDies triggers are
        // already handled above. But DO check it for Recover (it IS in the
        // graveyard now and can trigger its own Recover).
        // Find Recover ability definition on this card.
        let recover_cost = find_recover_cost(state, *obj_id);
        if let Some(cost) = recover_cost {
            // Create a RecoverTrigger pending trigger.
            triggers.push(PendingTrigger {
                source: *obj_id,
                controller: obj.owner, // Owner of the card, not death_controller
                kind: PendingTriggerKind::Recover,
                // ... all other fields None/default
            });
        }
    }
}
```

**Helper function** `find_recover_cost(state, obj_id) -> Option<ManaCost>`: Check the card registry for `AbilityDefinition::Recover { cost }` on the card. Also check `obj.characteristics.keywords` for `KeywordAbility::Recover` as a quick filter before scanning ability definitions.

**IMPORTANT**: The creature that just died CAN trigger its own Recover. CR 702.59a says the trigger functions "while the card with recover is in a player's graveyard." When a creature with Recover dies, it IS in the graveyard at trigger-check time. The creature entering the graveyard IS the triggering event.

**PendingTriggerKind**: Add `PendingTriggerKind::Recover` variant. Add `recover_cost: Option<ManaCost>` and `recover_card: Option<ObjectId>` fields to `PendingTrigger`.

**File**: `crates/engine/src/rules/abilities.rs` (flush_pending_triggers)
**Action**: In `flush_pending_triggers`, handle `PendingTriggerKind::Recover` to create a `StackObjectKind::RecoverTrigger` stack entry. Pattern follows the Echo/CumulativeUpkeep trigger creation.

### Step 5: Resolution

**File**: `crates/engine/src/rules/resolution.rs`
**Action**: Add resolution handler for `StackObjectKind::RecoverTrigger` after the EchoTrigger handler. Pattern follows Echo exactly:

```rust
StackObjectKind::RecoverTrigger {
    source_object: _,
    recover_card,
    recover_cost,
} => {
    let controller = stack_obj.controller;

    // Check if the Recover card is still in the graveyard (CR 400.7).
    let still_in_graveyard = state
        .objects
        .get(&recover_card)
        .map(|obj| matches!(obj.zone, ZoneId::Graveyard(_)))
        .unwrap_or(false);

    if still_in_graveyard {
        // Emit the payment required event and pause for player choice.
        events.push(GameEvent::RecoverPaymentRequired {
            player: controller,
            recover_card,
            cost: recover_cost.clone(),
        });
        // Track the pending payment so Command::PayRecover can find it.
        state
            .pending_recover_payments
            .push_back((controller, recover_card, recover_cost));
    }
    // If not in graveyard, do nothing (CR 400.7 -- card is a new object).

    events.push(GameEvent::AbilityResolved {
        controller,
        stack_object_id: stack_obj.id,
    });
}
```

### Step 6: Command Handler

**File**: `crates/engine/src/rules/engine.rs`
**Action**: Add `Command::PayRecover` handler after `PayCumulativeUpkeep` (line ~472). Pattern follows `handle_pay_echo`:

```rust
Command::PayRecover {
    player,
    recover_card,
    pay,
} => {
    validate_player_exists(&state, player)?;
    loop_detection::reset_loop_detection(&mut state);
    let mut events = handle_pay_recover(&mut state, player, recover_card, pay)?;
    let new_triggers = abilities::check_triggers(&state, &events);
    for t in new_triggers {
        state.pending_triggers.push_back(t);
    }
    let trigger_events = abilities::flush_pending_triggers(&mut state);
    events.extend(trigger_events);
    all_events.extend(events);
}
```

**Function** `handle_pay_recover(state, player, recover_card, pay) -> Result<Vec<GameEvent>, GameStateError>`:
1. Find and remove the matching pending recover payment.
2. If `pay` is true: deduct mana cost from player's mana pool, move card from graveyard to hand, emit `RecoverPaid`.
3. If `pay` is false: move card from graveyard to exile, emit `RecoverDeclined`.
4. Both branches use `move_object_to_zone` (new ObjectId per CR 400.7).

### Step 7: Unit Tests

**File**: `crates/engine/tests/recover.rs`
**Tests to write**:
- `test_recover_basic_creature_dies_triggers_recover` -- CR 702.59a: Place a non-creature card with Recover in the graveyard, kill a creature. Verify RecoverTrigger goes on stack.
- `test_recover_pay_returns_to_hand` -- CR 702.59a: Resolve the trigger, pay the cost. Verify card moves from graveyard to hand.
- `test_recover_decline_exiles_card` -- CR 702.59a: Resolve the trigger, decline to pay. Verify card moves from graveyard to exile.
- `test_recover_card_left_graveyard_before_resolution` -- CR 400.7: If the Recover card is no longer in the graveyard when trigger resolves, nothing happens.
- `test_recover_creature_with_recover_dies_triggers_self` -- CR 702.59a: A creature with Recover dies. Its own Recover triggers (it is now in the graveyard).
- `test_recover_multiple_recover_cards_in_graveyard` -- Multiple Recover cards trigger independently from a single creature death.
- `test_recover_opponents_creature_death_does_not_trigger` -- A creature dying and going to an opponent's graveyard does not trigger YOUR Recover cards.
- `test_recover_noncreature_dying_does_not_trigger` -- A non-creature permanent going to the graveyard does not trigger Recover (only creatures).
**Pattern**: Follow tests for Echo in `crates/engine/tests/echo.rs` (or the file containing echo tests).

### Step 8: Card Definition (later phase)

**Suggested card**: Grim Harvest
- Name: Grim Harvest
- Mana Cost: {1}{B}
- Type: Instant
- Oracle Text: "Return target creature card from your graveyard to your hand. Recover {2}{B}"
- Simple instant effect (return creature card from GY to hand) + Recover {2}{B}
- Good test vehicle because the effect is simple and doesn't interact with Recover's trigger in confusing ways.
- **Card lookup**: use `card-definition-author` agent

Alternative: Icefall ({2}{R}{R} Sorcery, Destroy target artifact or land, Recover {R}{R}) -- also simple but targets artifacts/lands which may need target filtering.

### Step 9: Game Script (later phase)

**Suggested scenario**: Grim Harvest in graveyard, a creature on the battlefield dies (SBA or destruction). Recover triggers, player pays {2}{B}, Grim Harvest returns to hand.
**Subsystem directory**: `test-data/generated-scripts/stack/` (triggered ability resolution)

## Interactions to Watch

- **SBA batch deaths**: When multiple creatures die simultaneously (e.g., board wipe), each one independently triggers all Recover cards. A Recover card in the graveyard could get multiple triggers queued. Once the first resolves (returning to hand OR exiling), subsequent triggers find the card no longer in the graveyard and fizzle (CR 400.7).
- **Tokens dying**: Tokens briefly exist in the graveyard before ceasing to exist as SBA. This IS sufficient to trigger Recover. The Gravestorm counter already tracks this (`permanents_put_into_graveyard_this_turn` increments for tokens too).
- **Dredge interaction**: If a player dredges instead of drawing and mills creatures, those creatures were NOT put into the graveyard "from the battlefield" -- they were put there from the library. Recover does NOT trigger.
- **Replacement effects on death**: If a replacement effect exiles a creature instead of putting it in the graveyard (e.g., Kalitas, Rest in Peace), the creature never enters the graveyard, so Recover does NOT trigger.
- **Commander zone redirect**: If a commander dies and the owner chooses to move it to the command zone (SBA), the creature briefly existed in the graveyard first. Recover SHOULD trigger because the creature WAS put into the graveyard from the battlefield (the SBA then moves it out).
- **Owner vs. controller for graveyard**: Objects always go to their OWNER's graveyard (CR 400.3). If Player A controls Player B's creature and it dies, it goes to Player B's graveyard. Player B's Recover cards trigger, not Player A's.
- **Multiplayer**: Each player's Recover cards only watch their own graveyard. No cross-player triggering.
