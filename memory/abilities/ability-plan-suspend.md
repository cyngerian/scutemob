# Ability Plan: Suspend

**Generated**: 2026-02-28
**CR**: 702.62
**Priority**: P3
**Similar abilities studied**: Foretell (special action exile from hand: `rules/foretell.rs`), Madness/Miracle (trigger-based cast-from-exile/hand: `rules/abilities.rs`, `state/stack.rs`), Cascade (cast without paying mana cost: `rules/copy.rs`)

## CR Rule Text

**702.62.** Suspend

**702.62a** Suspend is a keyword that represents three abilities. The first is a static ability that functions while the card with suspend is in a player's hand. The second and third are triggered abilities that function in the exile zone. "Suspend N--[cost]" means "If you could begin to cast this card by putting it onto the stack from your hand, you may pay [cost] and exile it with N time counters on it. This action doesn't use the stack," and "At the beginning of your upkeep, if this card is suspended, remove a time counter from it," and "When the last time counter is removed from this card, if it's exiled, you may play it without paying its mana cost if able. If you don't, it remains exiled. If you cast a creature spell this way, it gains haste until you lose control of the spell or the permanent it becomes."

**702.62b** A card is "suspended" if it's in the exile zone, has suspend, and has a time counter on it.

**702.62c** While determining if you could begin to cast a card with suspend, take into consideration any effects that would prohibit that card from being cast.

**702.62d** Casting a spell as an effect of its suspend ability follows the rules for paying alternative costs in rules 601.2b and 601.2f-h.

**116.2f** A player who has a card with suspend in their hand may exile that card. This is a special action. A player can take this action any time they have priority, but only if they could begin to cast that card by putting it onto the stack. See rule 702.62, "Suspend."

**107.3d** If a cost associated with a special action, such as a suspend cost or a morph cost, has an {X} or an X in it, the value of X is chosen by the player taking the special action immediately before they pay that cost.

## Key Edge Cases

1. **Suspend is a special action (CR 116.2f)**: Exiling a card with suspend does NOT use the stack and cannot be responded to. It can be done any time the player has priority (not just sorcery speed), provided the player could begin to cast the card normally.

2. **Cast without paying mana cost**: When the last time counter is removed, the spell is cast without paying its mana cost. This means:
   - X spells must choose X=0 (ruling 2024-02-02).
   - Cannot combine with alternative costs (flashback, evoke, etc.) (ruling 2024-02-02).
   - Mandatory additional costs (like kicker's mandatory component) must still be paid.
   - The spell's mana value is still determined by its printed mana cost (ruling 2024-02-02).

3. **Creature haste (CR 702.62a)**: If the suspended card is a creature spell, the creature gains haste until "you lose control of the spell or the permanent it becomes." This is a continuous effect, not a keyword grant.

4. **Cards with no mana cost (e.g., Ancestral Vision, Lotus Bloom)**: These cards literally cannot be cast normally (no mana cost = can't be cast). They can ONLY be cast via suspend or other alternative means. The suspend special action check (CR 702.62c) considers whether you could "begin to cast" -- for no-mana-cost cards, the answer is yes from hand during your main phase with empty stack (sorcery timing), even though you can't complete all casting steps.

5. **Suspended card definition (CR 702.62b)**: A card is "suspended" only if ALL THREE conditions are met: (1) in exile zone, (2) has suspend keyword, (3) has one or more time counters on it. A card in exile with 0 time counters is NOT suspended (relevant after all counters removed but spell not cast, or after the trigger is countered).

6. **Countering the upkeep trigger**: If the "remove a time counter" trigger is countered (e.g., by Stifle), no time counter is removed. The trigger fires again next upkeep (ruling 2024-02-02).

7. **Countering the cast trigger**: If the "cast without paying mana cost" trigger is countered, the card remains exiled with no time counters and is NO LONGER suspended (ruling 2024-02-02). It is stranded in exile.

8. **Last counter removed by any means**: The cast trigger fires whenever the last time counter is removed, regardless of why or what effect removed it (ruling 2024-02-02). Proliferate, time travel, or other counter-manipulation can trigger the cast.

9. **Cards exiled with suspend are exiled face up** (ruling 2024-02-02). Unlike foretell, suspend cards are public information.

10. **Optional cast (2024 rules change)**: "You MAY cast it" -- the player can decline. If declined, the card remains exiled with no time counters (no longer suspended). This is a recent rules change from "must cast if able" to "may cast."

11. **Timing permissions ignored**: When casting via suspend trigger resolution, timing permissions based on card type are ignored. A sorcery can be cast this way even during an opponent's turn or at instant speed (ruling 2024-02-02).

12. **Multiplayer**: Suspend's upkeep trigger fires at the beginning of the OWNER'S upkeep only. In a 4-player game, each player's suspended cards only tick down on their own upkeep.

## Current State (from ability-wip.md)

- [ ] Step 1: Enum variant
- [ ] Step 2: Rule enforcement
- [ ] Step 3: Trigger wiring
- [ ] Step 4: Unit tests
- [ ] Step 5: Card definition
- [ ] Step 6: Game script
- [ ] Step 7: Coverage doc update

## Implementation Steps

### Step 1: Enum Variant & AbilityDefinition

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/types.rs`
**Action**: Add `KeywordAbility::Suspend` variant after `Myriad` (line ~531).

```rust
/// CR 702.62: Suspend N -- [cost]. Three abilities: (1) special action to exile
/// from hand with N time counters by paying [cost], (2) upkeep trigger to remove
/// a time counter, (3) trigger to cast without paying mana cost when last counter
/// removed. If the suspended spell is a creature, it gains haste.
///
/// Marker for quick presence-checking (`keywords.contains`).
/// The suspend cost and counter count are stored in
/// `AbilityDefinition::Suspend { cost, time_counters }`.
Suspend,
```

NOTE: Unlike `Dredge(u32)` or `Annihilator(u32)`, Suspend's parameters (cost and N) are stored in `AbilityDefinition::Suspend` rather than in the keyword variant itself. This follows the pattern of Flashback, Cycling, Evoke, Foretell, etc. -- the keyword variant is a plain marker, and the parameterized data lives in the `AbilityDefinition` enum. Rationale: the keyword is stored in `OrdSet<KeywordAbility>` (which deduplicates), but the cost/counter data is needed only at the casting/special-action site, not for quick presence checks.

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/hash.rs`
**Action**: Add hash discriminant `65u8` for `KeywordAbility::Suspend` after `Myriad` (line ~437).

```rust
KeywordAbility::Suspend => 65u8.hash_into(hasher),
```

**File**: `/home/airbaggie/scutemob/crates/engine/src/cards/card_definition.rs`
**Action**: Add `AbilityDefinition::Suspend` variant after `Buyback` (line ~245).

```rust
/// CR 702.62: Suspend N -- [cost]. Exile this card from your hand with N time
/// counters on it by paying [cost]. At the beginning of your upkeep, remove a
/// time counter. When the last is removed, you may cast it without paying its
/// mana cost. If it's a creature, it gains haste.
///
/// Cards with this ability should also include
/// `AbilityDefinition::Keyword(KeywordAbility::Suspend)` for quick
/// presence-checking without scanning all abilities.
Suspend { cost: ManaCost, time_counters: u32 },
```

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/hash.rs`
**Action**: Add hash arm for `AbilityDefinition::Suspend` in the `AbilityDefinition` HashInto impl.

**File**: `/home/airbaggie/scutemob/tools/replay-viewer/src/view_model.rs`
**Action**: Add display arm for `KeywordAbility::Suspend` in `format_keyword_ability` (after `Myriad` arm, line ~658).

```rust
KeywordAbility::Suspend => "Suspend".to_string(),
```

**Match arms to grep**: Search for all `KeywordAbility` match expressions and add the `Suspend` arm:
```
Grep pattern="KeywordAbility::" path="/home/airbaggie/scutemob/crates/engine/src/" output_mode="files_with_matches"
Grep pattern="KeywordAbility::" path="/home/airbaggie/scutemob/tools/" output_mode="files_with_matches"
```

### Step 2: Suspend Special Action (Exile from Hand)

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/suspend.rs` (NEW FILE)
**Action**: Create a new module for Suspend, following the pattern of `foretell.rs`.
**Pattern**: Follow `foretell.rs` (lines 1-133)
**CR**: 702.62a + 116.2f

The `handle_suspend_card` function validates and executes the suspend special action:

1. Validate player has priority (CR 116.2f).
2. Validate the card is in the player's hand.
3. Validate the card has `KeywordAbility::Suspend`.
4. Look up `AbilityDefinition::Suspend { cost, time_counters }` from the card definition.
5. Validate the player could begin to cast the card (CR 702.62c): check card types and timing. For sorcery/creature/enchantment/artifact/planeswalker, require sorcery timing (active player, main phase, empty stack). For instants or cards with Flash, any time with priority. NOTE: For cards with no mana cost (e.g., Ancestral Vision), this is still legal -- the check is about timing restrictions, not whether the mana can be paid.
6. Validate and deduct the suspend cost from the mana pool.
7. Move the card from hand to exile (new ObjectId via CR 400.7).
8. Add `time_counters` time counters to the exiled card.
9. Mark the card as `is_suspended = true` (new field on GameObject -- see below).
10. Emit `ManaCostPaid` and `CardSuspended` events.

NOTE: Unlike foretell, suspended cards are exiled FACE UP (ruling). Do NOT set `face_down = true`.

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/game_object.rs`
**Action**: Add `is_suspended: bool` field to `GameObject` (after `is_foretold`, around line 371).

```rust
/// CR 702.62b: If true, this object in exile was suspended (exiled via the
/// suspend special action from hand). Used to identify suspended cards for
/// the upkeep counter-removal trigger. A card is "suspended" if it's in
/// exile, has suspend, and has a time counter on it (CR 702.62b).
///
/// This flag is set when the suspend special action exiles the card. Unlike
/// foretell, suspended cards are exiled face up.
#[serde(default)]
pub is_suspended: bool,
```

Also add this field to `state/hash.rs` in the `GameObject` HashInto impl.

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/events.rs`
**Action**: Add `CardSuspended` event variant.

```rust
/// CR 702.62a / CR 116.2f: A card was exiled from hand via the suspend special
/// action. The suspend cost was paid and the card was exiled with N time counters.
/// This is a special action -- it does not use the stack.
CardSuspended {
    player: PlayerId,
    /// The card's ObjectId before exile (now retired).
    object_id: ObjectId,
    /// New ObjectId in the exile zone.
    new_exile_id: ObjectId,
    /// Number of time counters placed on the card.
    time_counters: u32,
},
```

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/command.rs`
**Action**: Add `SuspendCard` command variant.

```rust
// -- Suspend (CR 702.62) -----------------------------------------------
/// Suspend a card from hand (CR 702.62a / CR 116.2f).
///
/// Special action: pay the suspend cost, exile the card from hand with N
/// time counters. This does not use the stack. Legal any time the player
/// has priority and could begin to cast the card.
SuspendCard { player: PlayerId, card: ObjectId },
```

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/engine.rs`
**Action**: Add handler for `Command::SuspendCard` in `process_command`, following the `ForetellCard` pattern (around line 294).

```rust
Command::SuspendCard { player, card } => {
    validate_player_active(&state, player)?;
    loop_detection::reset_loop_detection(&mut state);
    let events = suspend::handle_suspend_card(&mut state, player, card)?;
    all_events.extend(events);
}
```

Also add `mod suspend;` to `rules/mod.rs`.

### Step 3: Upkeep Trigger Infrastructure + Suspend Triggers

This is the most complex step. Suspend requires two triggered abilities that function in the EXILE zone, not on the battlefield. The current trigger infrastructure (`collect_triggers_for_event`) only scans battlefield permanents. We need to extend it.

#### Step 3a: Add TriggerEvent Variants

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/game_object.rs`
**Action**: Add `TriggerEvent::OwnerUpkeepStart` variant (around line 198).

```rust
/// CR 503.1: Triggers at the beginning of the active player's upkeep step.
/// Used by upkeep-based triggers (Suspend, Vanishing, Cumulative Upkeep).
/// Fires on ALL permanents controlled by the active player AND on suspended
/// cards owned by the active player in exile.
OwnerUpkeepStart,
```

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/hash.rs`
**Action**: Add hash discriminant for `TriggerEvent::OwnerUpkeepStart` (next available: 17u8 after `ControllerProliferates` at 16u8).

#### Step 3b: Fire Upkeep Triggers in check_triggers

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/abilities.rs`
**Action**: Add a `GameEvent::StepChanged { step: Step::Upkeep, .. }` arm to `check_triggers` (around line 1590, before the `_ => {}` catch-all).

This arm must:
1. Identify the active player at upkeep start.
2. Scan ALL battlefield permanents controlled by the active player for `OwnerUpkeepStart` triggers (via `collect_triggers_for_event`).
3. Scan ALL exiled objects owned by the active player that are suspended (have Suspend keyword + time counters > 0) for the suspend upkeep trigger.

For suspended cards in exile, since they are not on the battlefield, `collect_triggers_for_event` won't find them. Instead, add a dedicated scan in the `StepChanged` handler:

```rust
GameEvent::StepChanged { step: Step::Upkeep, .. } => {
    let active = state.turn.active_player;

    // 1. Collect standard upkeep triggers from battlefield permanents
    //    (for cards with AtBeginningOfYourUpkeep triggered abilities).
    collect_triggers_for_event(
        state,
        &mut triggers,
        TriggerEvent::OwnerUpkeepStart,
        None, // all battlefield permanents
        None,
    );

    // 2. CR 702.62a: Collect suspend upkeep triggers from exile.
    // Scan all exiled objects owned by the active player with suspend + time counters.
    let suspended_cards: Vec<ObjectId> = state
        .objects
        .values()
        .filter(|obj| {
            obj.zone == ZoneId::Exile
                && obj.owner == active
                && obj.is_suspended
                && obj.counters.get(&CounterType::Time).copied().unwrap_or(0) > 0
        })
        .map(|obj| obj.id)
        .collect();

    for card_id in suspended_cards {
        triggers.push(PendingTrigger {
            source: card_id,
            ability_index: 0,
            controller: active,
            triggering_event: None,
            entering_object_id: None,
            targeting_stack_id: None,
            triggering_player: None,
            exalted_attacker_id: None,
            defending_player_id: None,
            is_evoke_sacrifice: false,
            is_madness_trigger: false,
            madness_exiled_card: None,
            madness_cost: None,
            is_miracle_trigger: false,
            miracle_revealed_card: None,
            miracle_cost: None,
            is_unearth_trigger: false,
            is_exploit_trigger: false,
            is_modular_trigger: false,
            modular_counter_count: None,
            is_evolve_trigger: false,
            evolve_entering_creature: None,
            is_myriad_trigger: false,
            is_suspend_counter_trigger: true,  // NEW FIELD
            suspend_card_id: Some(card_id),     // NEW FIELD
        });
    }
}
```

NOTE: The `StepChanged` event is emitted by `advance_to_next_step` in `turn_structure.rs` (line 54) and `advance_to_next_turn` (line 107). These events flow through `enter_step` -> `check_triggers` via the `action_events` from `execute_turn_based_actions`. BUT -- `execute_turn_based_actions` for `Step::Upkeep` currently returns `Ok(Vec::new())` (the catch-all `_ => Ok(Vec::new())` at line 28 of `turn_actions.rs`). The `StepChanged` event is emitted BEFORE `enter_step` is called, by `advance_to_next_step` in `turn_structure.rs`. We need to verify the flow:

Actually, looking more carefully at the code: `StepChanged` is emitted by `advance_to_next_step` and returned as events from that function. Those events are then passed to `enter_step` via the flow in `handle_all_passed`. The `enter_step` function calls `execute_turn_based_actions` which generates ITS OWN events for the step (untap actions, draw actions, etc.), then calls `check_triggers` on THOSE events. But the `StepChanged` event is NOT part of the turn-based-action events -- it was emitted earlier.

So we need to either:
(a) Emit a `StepChanged` event from `execute_turn_based_actions` for `Step::Upkeep`, OR
(b) Add a separate trigger-check call in `enter_step` that fires the upkeep-specific triggers directly.

The cleanest approach is **(b)**: add suspend-specific trigger generation directly in `execute_turn_based_actions` for `Step::Upkeep`. This matches the existing pattern of `end_step_actions` generating delayed exile triggers for Unearth.

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/turn_actions.rs`
**Action**: Add `Step::Upkeep` handler to `execute_turn_based_actions` (line 19-29).

```rust
Step::Upkeep => Ok(upkeep_actions(state)),
```

Add function `upkeep_actions`:

```rust
/// CR 503.1: Upkeep step turn-based actions.
///
/// CR 702.62a: For each suspended card owned by the active player in exile
/// (has suspend, is_suspended, time counters > 0), queue a pending trigger
/// to remove a time counter. These triggers use the stack and can be
/// responded to (e.g., Stifle can counter them).
fn upkeep_actions(state: &mut GameState) -> Vec<GameEvent> {
    let active = state.turn.active_player;

    // Collect suspended cards in exile owned by the active player.
    let suspended_cards: Vec<ObjectId> = state
        .objects
        .values()
        .filter(|obj| {
            obj.zone == ZoneId::Exile
                && obj.owner == active
                && obj.is_suspended
                && obj.counters.get(&CounterType::Time).copied().unwrap_or(0) > 0
        })
        .map(|obj| obj.id)
        .collect();

    for card_id in suspended_cards {
        state.pending_triggers.push_back(PendingTrigger {
            source: card_id,
            ability_index: 0,
            controller: active,
            triggering_event: None,
            entering_object_id: None,
            targeting_stack_id: None,
            triggering_player: None,
            exalted_attacker_id: None,
            defending_player_id: None,
            is_evoke_sacrifice: false,
            is_madness_trigger: false,
            madness_exiled_card: None,
            madness_cost: None,
            is_miracle_trigger: false,
            miracle_revealed_card: None,
            miracle_cost: None,
            is_unearth_trigger: false,
            is_exploit_trigger: false,
            is_modular_trigger: false,
            modular_counter_count: None,
            is_evolve_trigger: false,
            evolve_entering_creature: None,
            is_myriad_trigger: false,
            is_suspend_counter_trigger: true,  // NEW
            suspend_card_id: Some(card_id),     // NEW
        });
    }

    Vec::new() // Events come from trigger resolution, not directly
}
```

#### Step 3c: Add PendingTrigger Fields for Suspend

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/stubs.rs`
**Action**: Add suspend-specific fields to `PendingTrigger` (after `is_myriad_trigger`, around line 180).

```rust
/// CR 702.62a: If true, this pending trigger is a suspend upkeep trigger.
///
/// When flushed to the stack, creates a `StackObjectKind::SuspendCounterTrigger`
/// instead of the normal `StackObjectKind::TriggeredAbility`. When this trigger
/// resolves, one time counter is removed from the suspended card. If that was
/// the last counter, a second trigger is immediately queued to cast the card.
#[serde(default)]
pub is_suspend_counter_trigger: bool,
/// CR 702.62a: ObjectId of the suspended card in exile.
///
/// Only meaningful when `is_suspend_counter_trigger` or `is_suspend_cast_trigger`
/// is true.
#[serde(default)]
pub suspend_card_id: Option<ObjectId>,
/// CR 702.62a: If true, this pending trigger is the suspend cast trigger
/// ("when the last time counter is removed, you may cast it").
///
/// When flushed to the stack, creates a `StackObjectKind::SuspendCastTrigger`.
/// When this trigger resolves, the owner may cast the card without paying its
/// mana cost.
#[serde(default)]
pub is_suspend_cast_trigger: bool,
```

#### Step 3d: Add StackObjectKind Variants for Suspend

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/stack.rs`
**Action**: Add two new `StackObjectKind` variants (after `MyriadTrigger`, around line 308).

```rust
/// CR 702.62a: Suspend upkeep counter-removal trigger.
///
/// "At the beginning of your upkeep, if this card is suspended, remove a
/// time counter from it." When this trigger resolves:
/// 1. Check if the card is still in exile and still suspended (CR 603.4).
/// 2. Remove one time counter.
/// 3. If that was the last counter, queue a SuspendCastTrigger.
///
/// If countered (e.g., Stifle), no counter is removed.
SuspendCounterTrigger {
    source_object: ObjectId,
    suspended_card: ObjectId,
},

/// CR 702.62a: Suspend cast trigger (last counter removed).
///
/// "When the last time counter is removed from this card, if it's exiled,
/// you may play it without paying its mana cost if able." When this trigger
/// resolves:
/// 1. Check if the card is still in exile (CR 603.4 intervening-if).
/// 2. Cast the card without paying its mana cost (deterministic: always cast).
///    - For creature spells, grant haste until control loss.
///    - X spells use X=0.
///    - No alternative costs allowed.
/// 3. If the player declines (future interactive choice), the card stays in
///    exile with no time counters (no longer suspended).
///
/// If countered (e.g., Stifle), the card stays in exile with 0 time counters
/// and is no longer suspended (CR 702.62b -- not suspended without counters).
SuspendCastTrigger {
    source_object: ObjectId,
    suspended_card: ObjectId,
    owner: PlayerId,
},
```

#### Step 3e: Wire flush_pending_triggers

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/abilities.rs`
**Action**: Add suspend trigger handling in `flush_pending_triggers` (around line 1811, in the `kind` selection chain).

```rust
} else if trigger.is_suspend_counter_trigger {
    StackObjectKind::SuspendCounterTrigger {
        source_object: trigger.source,
        suspended_card: trigger.suspend_card_id.unwrap_or(trigger.source),
    }
} else if trigger.is_suspend_cast_trigger {
    StackObjectKind::SuspendCastTrigger {
        source_object: trigger.source,
        suspended_card: trigger.suspend_card_id.unwrap_or(trigger.source),
        owner: trigger.controller,
    }
}
```

#### Step 3f: Resolve Suspend Triggers

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/resolution.rs`
**Action**: Add resolution handlers for `SuspendCounterTrigger` and `SuspendCastTrigger` in the stack resolution match. Follow the `UnearthTrigger` or `MadnessTrigger` pattern.

**SuspendCounterTrigger resolution**:
1. Find the suspended card by `suspended_card` ObjectId.
2. Check it is still in exile and still has time counters (intervening-if).
3. Remove one time counter.
4. Emit `GameEvent::CounterRemoved`.
5. If the remaining time counter count is now 0:
   - Queue a `SuspendCastTrigger` as a pending trigger.
6. Return events.

**SuspendCastTrigger resolution**:
1. Find the card by `suspended_card` ObjectId.
2. Check it is still in exile (intervening-if; CR 603.4).
3. Deterministic behavior for V1: always cast (no interactive choice).
4. Cast the card without paying its mana cost:
   - Move the card from exile to stack (new ObjectId via CR 400.7).
   - Create a `StackObject` for the spell, similar to cascade's free-cast pattern (`copy.rs` line 332-365).
   - Set `was_suspended = true` on the stack object (new field; see below).
   - Increment `spells_cast_this_turn`.
   - Emit `GameEvent::SpellCast`.
   - If the card is a creature, register a temporary continuous effect granting haste (until control loss = effectively permanent for the controller; use `UntilControllerLosesControl` or simplify to end-of-game duration for V1).
5. If the card is no longer in exile (e.g., someone else moved it), do nothing.

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/stack.rs`
**Action**: Add `was_suspended: bool` field to `StackObject` (after `was_buyback_paid`, around line 118).

```rust
/// CR 702.62a: If true, this spell was cast via suspend's cast trigger.
/// The spell was cast without paying its mana cost. If the spell is a
/// creature, the permanent gains haste (registered at resolution time).
/// No alternative costs were applied.
#[serde(default)]
pub was_suspended: bool,
```

Also update `resolution.rs` to check `was_suspended` at permanent-ETB time: if the permanent is a creature and `was_suspended`, register a continuous haste effect. The simplest approach for V1 is to set `has_summoning_sickness = false` on the creature when it enters, since haste effectively negates summoning sickness. The "until you lose control" duration is functionally permanent for the casting player. If control changes, the new controller does NOT benefit from haste.

### Step 4: Unit Tests

**File**: `/home/airbaggie/scutemob/crates/engine/tests/suspend.rs` (NEW FILE)
**Tests to write**:

1. **`test_suspend_basic_exile_from_hand`** -- CR 702.62a / 116.2f
   - Setup: Player has a card with Suspend 2 -- {R} in hand, sufficient mana.
   - Action: `SuspendCard` command.
   - Assert: Card is in exile with 2 time counters, `is_suspended = true`, not face down.
   - Assert: Mana was deducted.
   - Assert: `CardSuspended` event emitted.

2. **`test_suspend_counter_removal_on_upkeep`** -- CR 702.62a (second ability)
   - Setup: Card in exile with `is_suspended = true`, 2 time counters, owner is active player.
   - Action: Advance to upkeep step (pass through untap).
   - Assert: A suspend counter trigger goes on the stack.
   - Action: All pass priority (trigger resolves).
   - Assert: Card now has 1 time counter.
   - Assert: Card is still suspended (still has counters).

3. **`test_suspend_last_counter_triggers_cast`** -- CR 702.62a (third ability)
   - Setup: Card in exile with `is_suspended = true`, 1 time counter.
   - Action: Advance to upkeep; counter removal trigger resolves.
   - Assert: Card now has 0 time counters.
   - Assert: A suspend cast trigger is queued/on the stack.
   - Action: All pass priority (cast trigger resolves).
   - Assert: Card was cast (on stack or resolved to battlefield/graveyard).

4. **`test_suspend_creature_gains_haste`** -- CR 702.62a (haste clause)
   - Setup: Creature card with Suspend in exile, 1 time counter.
   - Action: Advance to upkeep; all triggers resolve; spell resolves.
   - Assert: Creature is on battlefield.
   - Assert: Creature can attack (has haste / no summoning sickness).

5. **`test_suspend_cast_without_paying_mana_cost`** -- CR 702.62d
   - Setup: Sorcery with Suspend and mana cost {2}{R}, in exile with 1 time counter.
   - Action: Advance to upkeep; triggers resolve; spell cast.
   - Assert: No mana was deducted from the owner's pool.
   - Assert: Spell effect executed.

6. **`test_suspend_invalid_not_in_hand`** -- Error case
   - Setup: Card with Suspend on the battlefield (not in hand).
   - Action: Attempt `SuspendCard`.
   - Assert: Error (InvalidCommand).

7. **`test_suspend_invalid_no_keyword`** -- Error case
   - Setup: Card without Suspend in hand.
   - Action: Attempt `SuspendCard`.
   - Assert: Error (InvalidCommand).

8. **`test_suspend_card_no_longer_suspended_after_cast`** -- CR 702.62b
   - Setup: Card suspended with 1 time counter; cast trigger resolves.
   - Assert: Card is on the stack (or resolved). `is_suspended` should be reset per CR 400.7 (zone change creates new object).

9. **`test_suspend_not_active_player_upkeep_no_trigger`** -- Multiplayer edge case
   - Setup: 4-player game. Player B has a suspended card. It is Player A's turn.
   - Action: Advance through Player A's upkeep.
   - Assert: Player B's suspended card still has the same number of counters.

**Pattern**: Follow tests in `/home/airbaggie/scutemob/crates/engine/tests/foretell.rs` (if it exists) or `/home/airbaggie/scutemob/crates/engine/tests/` for general structure.

### Step 5: Card Definition (later phase)

**Suggested card**: **Rift Bolt** ({2}{R} Sorcery, Suspend 1 -- {R}, deals 3 damage to any target)

Rift Bolt is ideal because:
- Has a real mana cost (unlike Ancestral Vision/Lotus Bloom which have no mana cost)
- Suspend 1 means only 1 upkeep cycle needed for testing
- The spell effect (deal 3 damage to any target) already exists as `Effect::DealDamage`
- Tests the targeting flow (targets chosen at cast time, not at exile time)
- Has a simple, non-zero suspend cost ({R})

**Card lookup**: use `card-definition-author` agent

**Secondary card**: **Ancestral Vision** (no mana cost, Suspend 4 -- {U}, draw 3 cards) -- tests the "no mana cost" edge case where the card can ONLY be cast via suspend.

### Step 6: Game Script (later phase)

**Suggested scenario**: "Rift Bolt Suspend 1 -- exile, upkeep remove counter, cast free, deal 3 damage"

1. Initial state: Player 1 has Rift Bolt in hand, 1 red mana in pool.
2. Step 1: Player 1 suspends Rift Bolt (SuspendCard command).
3. Assert: Rift Bolt in exile with 1 time counter.
4. Step 2: Advance to Player 1's next upkeep.
5. Assert: Suspend counter trigger on stack.
6. Step 3: All pass -- trigger resolves, removes last counter.
7. Assert: Suspend cast trigger on stack.
8. Step 4: All pass -- cast trigger resolves, Rift Bolt cast for free targeting opponent.
9. Assert: Rift Bolt on stack as a spell.
10. Step 5: All pass -- Rift Bolt resolves, deals 3 damage to target.
11. Assert: Target player lost 3 life.

**Subsystem directory**: `test-data/generated-scripts/stack/` (Suspend interacts heavily with the stack)

## Interactions to Watch

### Suspend + Split Second (CR 702.61)
Split second prevents casting spells and activating non-mana abilities. BUT:
- The suspend special action (exile from hand) is a special action, not a spell or ability. CR 702.61b says special actions are still allowed under split second. However, 116.2f says the suspend special action requires that the player "could begin to cast" the card. Under split second, the player cannot begin to cast -- so the special action should be blocked.
- The suspend upkeep trigger still triggers and resolves normally (CR 702.61b: triggered abilities trigger and resolve normally under split second).
- The suspend cast trigger fires and resolves normally. The cast itself bypasses the "no casting" restriction because the cast happens as part of trigger resolution, not as a new action on the stack.

### Suspend + Cascade/Storm
If the suspended spell has cascade or storm, those triggers fire when the spell is cast (even though it was cast via suspend). The spell IS cast -- it triggers "whenever you cast a spell" effects.

### Suspend + Commander Tax
The suspend cast trigger casts without paying mana cost. Commander tax applies only to casting from the command zone (CR 903.8). Suspend casts from exile, so commander tax does not apply.

### Suspend + Timing Restrictions
The suspend cast trigger ignores timing restrictions. A sorcery suspended from another player's turn can be cast when the last counter is removed during the owner's upkeep, even though it's not the main phase.

### Suspend + Cards with No Mana Cost
Cards like Ancestral Vision and Lotus Bloom have no mana cost. They can be suspended (the special action is legal because the timing check passes -- you could begin to cast from hand during main phase), but they literally cannot be cast normally. When the suspend cast trigger resolves, "cast without paying its mana cost" works on a card with no mana cost -- the mana cost is effectively {0}.

### Multiplayer Implications
- Each player's suspended cards only tick down on THEIR upkeep.
- If a player leaves the game, their suspended cards in exile cease to exist (CR 800.4a).
- APNAP ordering: If multiple players have suspend triggers at the same time (impossible -- each player has their own upkeep), this is a non-issue. But if one player has multiple suspended cards, all their upkeep triggers go on the stack in APNAP order (the owner chooses the order since they are all controlled by the same player).

## V1 Simplifications

The following are acceptable simplifications for the initial implementation:

1. **No interactive suspend activation**: The V1 `SuspendCard` command is a player-initiated command. The test harness must emit it explicitly. Auto-detection of "player should/could suspend" is deferred.

2. **Deterministic cast on trigger resolution**: When the suspend cast trigger resolves, the card is always cast (no "may" choice). Interactive choice (decline to cast) is deferred.

3. **Haste as summoning sickness removal**: Instead of a full "gains haste until you lose control" continuous effect, V1 simply clears `has_summoning_sickness = false` when the creature enters the battlefield via suspend. This is correct for the common case (player never loses control). The edge case of control-change removing the haste effect is deferred.

4. **No X-cost suspend cards in V1**: The check that X=0 when casting without paying mana cost should be trivially handled since the engine already forces X=0 when no mana is paid, but specific X-cost suspend card definitions are deferred.

## Files Modified Summary

| File | Action |
|------|--------|
| `crates/engine/src/state/types.rs` | Add `KeywordAbility::Suspend` |
| `crates/engine/src/state/hash.rs` | Add hash discriminant 65 for Suspend + AbilityDefinition hash + GameObject field hash |
| `crates/engine/src/state/game_object.rs` | Add `is_suspended: bool` field, add `TriggerEvent::OwnerUpkeepStart` |
| `crates/engine/src/state/stubs.rs` | Add `is_suspend_counter_trigger`, `suspend_card_id`, `is_suspend_cast_trigger` to PendingTrigger |
| `crates/engine/src/state/stack.rs` | Add `SuspendCounterTrigger`, `SuspendCastTrigger` to StackObjectKind; add `was_suspended` to StackObject |
| `crates/engine/src/cards/card_definition.rs` | Add `AbilityDefinition::Suspend { cost, time_counters }` |
| `crates/engine/src/rules/command.rs` | Add `Command::SuspendCard` |
| `crates/engine/src/rules/events.rs` | Add `GameEvent::CardSuspended` |
| `crates/engine/src/rules/engine.rs` | Add handler for `SuspendCard` command |
| `crates/engine/src/rules/suspend.rs` | **NEW** -- `handle_suspend_card` special action |
| `crates/engine/src/rules/mod.rs` | Add `pub mod suspend;` |
| `crates/engine/src/rules/turn_actions.rs` | Add `Step::Upkeep` handler with suspend trigger queuing |
| `crates/engine/src/rules/abilities.rs` | Wire suspend triggers in `flush_pending_triggers` |
| `crates/engine/src/rules/resolution.rs` | Add resolution for `SuspendCounterTrigger` and `SuspendCastTrigger` |
| `crates/engine/tests/suspend.rs` | **NEW** -- 9 unit tests |
| `tools/replay-viewer/src/view_model.rs` | Add `KeywordAbility::Suspend` display |
| `crates/engine/src/testing/replay_harness.rs` | Add `suspend_card` action type for script harness |
| `crates/engine/src/lib.rs` | Re-export new types if needed |
