# Ability Plan: Unearth

**Generated**: 2026-02-27
**CR**: 702.84
**Priority**: P3
**Similar abilities studied**: Evoke (sacrifice on ETB trigger, `was_evoked` flag pattern), Cycling (dedicated Command for non-battlefield zone activation), Flashback (graveyard-cast with exile replacement), Persist/Undying (MoveZone + counter in builder.rs triggered abilities)

## CR Rule Text

702.84. Unearth

702.84a Unearth is an activated ability that functions while the card with unearth is in a graveyard. "Unearth [cost]" means "[Cost]: Return this card from your graveyard to the battlefield. It gains haste. Exile it at the beginning of the next end step. If it would leave the battlefield, exile it instead of putting it anywhere else. Activate only as a sorcery."

## Key Edge Cases

From Dregscape Zombie rulings (2008-10-01):

1. **If removed from graveyard before resolution**: "If you activate a card's unearth ability but that card is removed from your graveyard before the ability resolves, that unearth ability will resolve and do nothing." -- The ability goes on the stack; the card stays in the graveyard until resolution. If the card moved zones before resolution, the ability fizzles (CR 400.7).

2. **Exile replacement vs. actual exile**: "If a creature returned to the battlefield with unearth would leave the battlefield for any reason, it's exiled instead -- unless the spell or ability that's causing the creature to leave the battlefield is actually trying to exile it! In that case, it succeeds at exiling it." -- The replacement only fires when the destination is NOT exile. If the destination IS exile, no replacement needed.

3. **Exile/delayed trigger are NOT granted to the creature**: "Unearth grants haste to the creature that's returned to the battlefield. However, neither of the 'exile' abilities is granted to that creature. If that creature loses all its abilities, it will still be exiled at the beginning of the end step, and if it would leave the battlefield, it is still exiled instead." -- The exile replacement effect and delayed trigger are properties of the UNEARTH ABILITY on the stack, not of the creature. Losing all abilities (e.g., Humility) does NOT prevent the exile.

4. **Delayed trigger can be countered**: "At the beginning of the end step, a creature returned to the battlefield with unearth is exiled. This is a delayed triggered ability, and it can be countered by effects such as Stifle or Voidslime that counter triggered abilities. If the ability is countered, the creature will stay on the battlefield and the delayed trigger won't trigger again. However, the replacement effect will still exile the creature when it eventually leaves the battlefield." -- Countering the delayed trigger is legal. The replacement effect persists independently.

5. **Not casting**: "Activating a creature card's unearth ability isn't the same as casting the creature card. The unearth ability is put on the stack, but the creature card is not. Spells and abilities that interact with activated abilities (such as Stifle) will interact with unearth, but spells and abilities that interact with spells (such as Remove Soul) will not." -- This is an activated ability, NOT a cast. No cast triggers fire. The ability uses the stack.

6. **Flickered/exiled permanently**: If the unearthed creature is exiled and returned by an effect like Flickerwisp or Oblivion Ring, "the creature card will return to the battlefield as a new object with no relation to its previous existence. The unearth effect will no longer apply to it." -- CR 400.7: zone change creates a new object. The unearth tracking (`was_unearthed` flag) is lost.

7. **Multiple unearth abilities**: Sedris, the Traitor King can give a creature card in the graveyard multiple unearth abilities. Either may be activated.

8. **Multiplayer**: Unearth says "activate only as a sorcery" -- active player, main phase, stack empty. In multiplayer, only the active player can activate their creatures' unearth. The exile replacement and delayed trigger work identically across any number of players.

## Current State (from ability-wip.md)

- [ ] 1. Enum variant
- [ ] 2. Rule enforcement
- [ ] 3. Trigger wiring
- [ ] 4. Unit tests
- [ ] 5. Card definition
- [ ] 6. Game script
- [ ] 7. Coverage doc update

## Implementation Steps

### Step 1: Enum Variant and Type Infrastructure

#### 1a. KeywordAbility::Unearth variant

**File**: `crates/engine/src/state/types.rs`
**Action**: Add `Unearth` variant to `KeywordAbility` enum after `Foretell` (line ~407).
**Pattern**: Follow `KeywordAbility::Flashback` at line 219 -- marker keyword with cost stored separately in `AbilityDefinition::Unearth { cost }`.

```rust
/// CR 702.84: Unearth [cost] -- activated ability from graveyard.
/// "[Cost]: Return this card from your graveyard to the battlefield.
/// It gains haste. Exile it at the beginning of the next end step.
/// If it would leave the battlefield, exile it instead of putting it
/// anywhere else. Activate only as a sorcery."
///
/// Marker for quick presence-checking (`keywords.contains`).
/// The unearth cost itself is stored in `AbilityDefinition::Unearth { cost }`.
Unearth,
```

#### 1b. AbilityDefinition::Unearth variant

**File**: `crates/engine/src/cards/card_definition.rs`
**Action**: Add `Unearth { cost: ManaCost }` variant to `AbilityDefinition` enum, after `Foretell` (line ~406).
**Pattern**: Follow `AbilityDefinition::Flashback { cost }` at line 145.

```rust
/// CR 702.84: Unearth [cost]. The card's unearth ability can be activated
/// from its owner's graveyard by paying this cost. When the ability resolves,
/// the card returns to the battlefield with haste, a delayed exile trigger
/// at the next end step, and a replacement effect that exiles it if it
/// would leave the battlefield for any non-exile zone.
///
/// Cards with this ability should also include
/// `AbilityDefinition::Keyword(KeywordAbility::Unearth)` for quick
/// presence-checking without scanning all abilities.
Unearth { cost: ManaCost },
```

#### 1c. was_unearthed flag on GameObject

**File**: `crates/engine/src/state/game_object.rs`
**Action**: Add `was_unearthed: bool` field to `GameObject` struct, after `was_escaped` (line ~343).
**Pattern**: Follow `was_evoked: bool` at line 322.

```rust
/// CR 702.84a: If true, this permanent was returned to the battlefield via
/// an unearth ability. Two effects track this:
/// 1. Replacement effect: if this permanent would leave the battlefield for
///    any zone other than exile, it is exiled instead.
/// 2. Delayed triggered ability: at the beginning of the next end step,
///    exile this permanent.
///
/// These effects are NOT abilities on the creature -- they persist even if
/// the creature loses all abilities (e.g., Humility).
///
/// Set when the UnearthCard ability resolves. Reset on zone changes (CR 400.7).
#[serde(default)]
pub was_unearthed: bool,
```

**IMPORTANT**: Per edge case 3, the exile replacement and delayed trigger are NOT granted to the creature. They are effects created by the unearth ability's resolution. The `was_unearthed` flag is the tracking mechanism -- it persists on the object even if the object loses all abilities.

#### 1d. Hash update

**File**: `crates/engine/src/state/hash.rs`
**Action**: Add `self.was_unearthed.hash_into(hasher)` to the `GameObject` `HashInto` impl, after `was_escaped` (line ~545).
**Pattern**: Follow `self.was_evoked.hash_into(hasher)` at line 541.

Also add a new discriminant for `StackObjectKind::UnearthTrigger` in the StackObjectKind hash impl.

#### 1e. Command::UnearthCard variant

**File**: `crates/engine/src/rules/command.rs`
**Action**: Add `UnearthCard { player: PlayerId, card: ObjectId }` variant, after `ForetellCard` (line ~337).
**Pattern**: Follow `Command::CycleCard` at line 272. Both are dedicated commands for zone-specific activated abilities.

```rust
// -- Unearth (CR 702.84) -----------------------------------------------
/// Activate a card's unearth ability from the graveyard (CR 702.84a).
///
/// The card must be in the player's graveyard with `KeywordAbility::Unearth`.
/// The unearth cost is paid, and the unearth ability is placed on the stack.
/// When it resolves, the card returns to the battlefield with haste,
/// a delayed exile trigger (beginning of next end step), and a replacement
/// effect (if it would leave battlefield for non-exile, exile instead).
///
/// "Activate only as a sorcery" -- main phase, stack empty, active player.
///
/// Unlike `CastSpell`, this is an activated ability, not a spell cast.
/// No "cast" triggers fire.
UnearthCard { player: PlayerId, card: ObjectId },
```

#### 1f. StackObjectKind::UnearthTrigger variant

**File**: `crates/engine/src/state/stack.rs`
**Action**: Add `UnearthTrigger { source_object: ObjectId }` variant after `MiracleTrigger` (line ~220).
**Pattern**: Follow `StackObjectKind::EvokeSacrificeTrigger` at line 188 -- same simple pattern (carries source object for resolution).

```rust
/// CR 702.84a: Unearth delayed triggered ability on the stack.
///
/// "Exile [this permanent] at the beginning of the next end step."
/// This is a delayed triggered ability created when the unearth ability
/// resolves. It fires at the beginning of the next end step.
///
/// If the source has left the battlefield by resolution time (CR 400.7),
/// the trigger does nothing. If countered (e.g., by Stifle), the permanent
/// stays on the battlefield but the replacement effect still applies.
UnearthTrigger { source_object: ObjectId },
```

Also add it to the `StackObjectKind` match exhaustiveness in:
- `resolution.rs` counter handler (line ~795)
- `view_model.rs` (line ~437)

### Step 2: Command Handler (UnearthCard)

**File**: `crates/engine/src/rules/abilities.rs`
**Action**: Add `handle_unearth_card` function.
**CR**: 702.84a -- validates graveyard zone, keyword, sorcery timing, pays mana cost, pushes ability onto stack.

```rust
/// Handle an UnearthCard command: validate, pay cost, push unearth ability onto stack.
///
/// CR 702.84a: Unearth is an activated ability from the graveyard.
/// "[Cost]: Return this card from your graveyard to the battlefield. It gains haste.
/// Exile it at the beginning of the next end step. If it would leave the battlefield,
/// exile it instead. Activate only as a sorcery."
pub fn handle_unearth_card(
    state: &mut GameState,
    player: PlayerId,
    card: ObjectId,
) -> Result<Vec<GameEvent>, GameStateError>
```

Implementation pattern (follow `handle_cycle_card` at line 384):

1. **Priority check** (CR 602.2)
2. **Split second check** (CR 702.61a)
3. **Zone check**: card must be in `ZoneId::Graveyard(player)` (CR 702.84a: "from your graveyard")
4. **Keyword check**: card must have `KeywordAbility::Unearth`
5. **Sorcery speed check** (CR 702.84a: "activate only as a sorcery"):
   - `state.turn.active_player == player`
   - Step is `PreCombatMain` or `PostCombatMain`
   - `state.stack_objects.is_empty()`
6. **Look up unearth cost** from CardRegistry via `get_unearth_cost()` helper (pattern: `get_flashback_cost` at line 1022)
7. **Pay mana cost** (CR 602.2b)
8. **Push activated ability onto stack**: Use `StackObjectKind::ActivatedAbility` with `source_object: card`, `ability_index: 0` (unused), and `embedded_effect: None` (resolution handled specially). NOTE: Unlike cycling, the card does NOT leave the graveyard as a cost -- it stays in the graveyard until the ability resolves.
9. **Emit events**: `AbilityActivated`, `PriorityGiven`
10. **Trigger check + flush** (per command handler pattern gotcha)

**CRITICAL DIFFERENCE from Cycling**: In cycling, the card is discarded as cost (moves zone immediately). In unearth, the card stays in the graveyard. The zone move happens at RESOLUTION time, not activation time.

**However**, the pushed stack ability needs to know this is an unearth ability, not a generic activated ability. Two approaches:

**Option A (preferred)**: Add a new `StackObjectKind::UnearthAbility { source_object: ObjectId }` that the resolution handler can match on. This is cleaner than overloading `ActivatedAbility` with a flag.

**Option B**: Use `ActivatedAbility` with a special embedded effect. Less clean.

**Decision**: Use Option A. Add `StackObjectKind::UnearthAbility { source_object: ObjectId }` to the stack.rs enum, alongside `UnearthTrigger`. The resolution handler will have a dedicated match arm.

```rust
/// CR 702.84a: Unearth activated ability on the stack.
///
/// When this ability resolves: (1) move the source card from graveyard to
/// battlefield, (2) grant haste, (3) set was_unearthed flag, (4) create
/// a delayed trigger for end-step exile.
///
/// If the source card is no longer in the graveyard at resolution time,
/// the ability does nothing (card was exiled, shuffled, etc.).
UnearthAbility { source_object: ObjectId },
```

#### 2b. Command dispatch

**File**: `crates/engine/src/rules/engine.rs`
**Action**: Add `Command::UnearthCard` match arm in `process_command`, after the `ForetellCard` handler.
**Pattern**: Follow `Command::CycleCard` dispatch at the relevant position.

```rust
Command::UnearthCard { player, card } => {
    validate_player_active(&state, player)?;
    loop_detection::reset_loop_detection(&mut state);
    let mut events = abilities::handle_unearth_card(&mut state, player, card)?;
    let new_triggers = abilities::check_triggers(&state, &events);
    for t in new_triggers {
        state.pending_triggers.push_back(t);
    }
    let trigger_events = abilities::flush_pending_triggers(&mut state);
    events.extend(trigger_events);
    all_events.extend(events);
}
```

#### 2c. Helper: get_unearth_cost

**File**: `crates/engine/src/rules/abilities.rs` (or `casting.rs` -- but abilities.rs is better since this is an activated ability, not casting)
**Action**: Add `get_unearth_cost()` function.
**Pattern**: Follow `get_flashback_cost` in `casting.rs` at line 1022.

```rust
/// CR 702.84a: Look up the unearth cost from the card's `AbilityDefinition`.
fn get_unearth_cost(
    card_id: &Option<CardId>,
    registry: &CardRegistry,
) -> Option<ManaCost> {
    card_id.as_ref().and_then(|cid| {
        registry.get(cid.clone()).and_then(|def| {
            def.abilities.iter().find_map(|a| {
                if let AbilityDefinition::Unearth { cost } = a {
                    Some(cost.clone())
                } else {
                    None
                }
            })
        })
    })
}
```

### Step 3: Resolution Handler

**File**: `crates/engine/src/rules/resolution.rs`
**Action**: Add resolution handler for `StackObjectKind::UnearthAbility`.
**CR**: 702.84a -- move card from graveyard to battlefield, grant haste, set `was_unearthed`, register delayed trigger and replacement effect.

Add a new match arm in the `handle_resolve_top` function, after the existing `EvokeSacrificeTrigger` handler (line ~538).

```rust
StackObjectKind::UnearthAbility { source_object } => {
    let controller = stack_obj.controller;

    // Check if the source card is still in the graveyard (CR 400.7).
    // Per ruling: "If that card is removed from your graveyard before
    // the ability resolves, that unearth ability will resolve and do nothing."
    let still_in_graveyard = state
        .objects
        .get(&source_object)
        .map(|obj| matches!(obj.zone, ZoneId::Graveyard(_)))
        .unwrap_or(false);

    if still_in_graveyard {
        // 1. Move card from graveyard to battlefield (CR 702.84a).
        let (new_id, _old) =
            state.move_object_to_zone(source_object, ZoneId::Battlefield)?;

        // 2. Set controller and was_unearthed flag.
        if let Some(obj) = state.objects.get_mut(&new_id) {
            obj.controller = controller;
            obj.was_unearthed = true;
            // 3. Grant haste (CR 702.84a: "It gains haste").
            obj.characteristics.keywords.insert(KeywordAbility::Haste);
        }

        // 4. Emit PermanentEnteredBattlefield event.
        events.push(GameEvent::PermanentEnteredBattlefield {
            owner: controller,
            object_id: new_id,
        });

        // 5. Register the unearth replacement effect: "If it would leave
        //    the battlefield, exile it instead of putting it anywhere else."
        //    This is NOT an ability on the creature -- it persists even if
        //    the creature loses all abilities (ruling).
        //    Implemented by checking `was_unearthed` in the zone-change
        //    replacement pipeline (see Step 4).

        // 6. Register ETB triggers, static effects, etc. for the permanent
        //    (same as normal spell resolution ETB path).
        // ... (apply_self_etb_from_definition, register_static_continuous_effects,
        //       fire_when_enters_triggered_effects)

        // 7. Create delayed trigger: "Exile at the beginning of the next end step."
        //    We do NOT push it on the stack now -- it fires later at the end step.
        //    Instead, we push an UnearthTrigger PendingTrigger when the end step
        //    StepChanged event fires. Track via was_unearthed on the object.
    }

    events.push(GameEvent::AbilityResolved {
        controller,
        stack_object_id: stack_obj.id,
    });
}
```

**IMPORTANT**: The ETB path must mirror the spell resolution ETB path from lines 203-300 in resolution.rs:
- `apply_self_etb_from_definition` (for any ETB abilities the creature has)
- `apply_global_etb_replacements` (for global replacement effects like Rest in Peace)
- `register_static_continuous_effects` (for static abilities)
- `fire_when_enters_triggered_effects` (for ETB triggers)

Extract a shared helper or call the same functions. This is critical -- an unearthed creature's ETB abilities must fire normally.

#### 3b. Unearth delayed trigger resolution

**File**: `crates/engine/src/rules/resolution.rs`
**Action**: Add resolution handler for `StackObjectKind::UnearthTrigger`.
**CR**: 702.84a -- exile the unearthed permanent.

```rust
StackObjectKind::UnearthTrigger { source_object } => {
    let controller = stack_obj.controller;

    // Check if the source is still on the battlefield (CR 400.7).
    let still_on_battlefield = state
        .objects
        .get(&source_object)
        .map(|obj| obj.zone == ZoneId::Battlefield)
        .unwrap_or(false);

    if still_on_battlefield {
        let (_new_id, _old) =
            state.move_object_to_zone(source_object, ZoneId::Exile)?;
        // Emit appropriate event
    }
    // If not on battlefield, do nothing (already exiled by replacement or removed).

    events.push(GameEvent::AbilityResolved {
        controller,
        stack_object_id: stack_obj.id,
    });
}
```

### Step 4: Replacement Effect (Zone-Change Interception)

**File**: `crates/engine/src/rules/replacement.rs` (or wherever `check_zone_change_replacement` is called from)
**Action**: Before checking registered replacement effects, check `was_unearthed` on the object and redirect non-exile destinations to exile.
**CR**: 702.84a -- "If it would leave the battlefield, exile it instead of putting it anywhere else."

This is best implemented as an early check in the zone-change pipeline, NOT as a registered ReplacementEffect. Reason: the ruling says this effect persists even if the creature loses all abilities. It is NOT an ability on the creature. A registered ReplacementEffect could potentially be interacted with (removed, overridden), but the unearth effect is unconditional.

**Implementation approach**: In all zone-change sites that call `check_zone_change_replacement`, add an early check:

```rust
// CR 702.84a: Unearth replacement -- if an unearthed permanent would leave
// the battlefield for any zone other than exile, exile it instead.
// This is NOT an ability on the creature -- it persists even if the creature
// loses all abilities.
if let Some(obj) = state.objects.get(&object_id) {
    if obj.was_unearthed && obj.zone == ZoneId::Battlefield {
        if destination != ZoneId::Exile {
            // Redirect to exile
            destination = ZoneId::Exile;
        }
    }
}
```

**Sites to modify** (per ETB site gotcha -- two ETB sites exist):
1. `rules/sba.rs` -- SBA-driven zone changes (lethal damage, 0 toughness, legend rule, etc.)
2. `rules/resolution.rs` -- spell/ability resolution zone changes
3. `rules/replacement.rs` -- the `check_zone_change_replacement` function itself (add as first check before querying registered replacements)
4. Any `move_object_to_zone` call sites for combat damage, destroy effects, etc.

The cleanest approach is to add this check INSIDE `check_zone_change_replacement` as the very first thing, so it automatically intercepts ALL zone-change paths that already use this function. If the object has `was_unearthed` and the destination is not exile, return `ZoneChangeAction::Redirect { destination: ZoneId::Exile }`.

### Step 5: Delayed Trigger Wiring (End Step)

**File**: `crates/engine/src/rules/abilities.rs` (in `check_triggers`)
**Action**: Add handling for `GameEvent::StepChanged` to fire unearth delayed triggers at the beginning of the end step.
**CR**: 702.84a -- "Exile it at the beginning of the next end step."

When `StepChanged { step: Step::End, .. }` is emitted, scan all battlefield objects for `was_unearthed == true` and create `PendingTrigger` entries that flush to `StackObjectKind::UnearthTrigger`.

```rust
GameEvent::StepChanged { step, .. } => {
    if *step == Step::End {
        // CR 702.84a: Unearth delayed trigger -- "Exile [this permanent]
        // at the beginning of the next end step."
        for obj in state.objects.values() {
            if obj.was_unearthed && obj.zone == ZoneId::Battlefield {
                let unearth_trigger = PendingTrigger {
                    source: obj.id,
                    ability_index: 0, // unused for unearth trigger
                    controller: obj.controller,
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
                    is_unearth_trigger: true,  // NEW FIELD
                };
                triggers.push(unearth_trigger);
            }
        }
    }
}
```

#### 5b. PendingTrigger: is_unearth_trigger field

**File**: `crates/engine/src/state/stubs.rs`
**Action**: Add `is_unearth_trigger: bool` field to `PendingTrigger` struct, after `is_miracle_trigger` (line ~116).
**Pattern**: Follow `is_evoke_sacrifice` at line 81.

```rust
/// CR 702.84a: If true, this pending trigger is the unearth delayed exile trigger.
///
/// When flushed to the stack, creates a `StackObjectKind::UnearthTrigger`
/// instead of the normal `StackObjectKind::TriggeredAbility`. The `ability_index`
/// field is unused when this is true.
#[serde(default)]
pub is_unearth_trigger: bool,
```

#### 5c. flush_pending_triggers: UnearthTrigger dispatch

**File**: `crates/engine/src/rules/abilities.rs` (in `flush_pending_triggers`)
**Action**: Add `is_unearth_trigger` check in the flush logic, after the `is_miracle_trigger` check (line ~1202).
**Pattern**: Follow the `is_evoke_sacrifice` dispatch at line 1191.

```rust
} else if trigger.is_unearth_trigger {
    StackObjectKind::UnearthTrigger {
        source_object: trigger.source,
    }
}
```

### Step 6: Replay Harness Action Type

**File**: `crates/engine/src/testing/replay_harness.rs`
**Action**: Add `"unearth_card"` action type to `translate_player_action`.
**Pattern**: Follow `"cycle_card"` at line 444.

```rust
"unearth_card" => {
    let card_id = find_in_graveyard(state, player, card_name?)?;
    Some(Command::UnearthCard {
        player,
        card: card_id,
    })
}
```

### Step 7: View Model Update

**File**: `tools/replay-viewer/src/view_model.rs`
**Action**: Add `UnearthAbility` and `UnearthTrigger` arms to the stack object kind match (line ~437).
**Pattern**: Follow `EvokeSacrificeTrigger` at line 431.

```rust
StackObjectKind::UnearthAbility { source_object } => {
    ("unearth_ability", Some(*source_object))
}
StackObjectKind::UnearthTrigger { source_object } => {
    ("unearth_trigger", Some(*source_object))
}
```

### Step 8: Unit Tests

**File**: `crates/engine/tests/unearth.rs` (new file)
**Tests to write**:
**Pattern**: Follow tests for Persist/Undying in `crates/engine/tests/persist_undying.rs`

1. `test_unearth_basic_return_to_battlefield` -- CR 702.84a: Activate unearth on a creature in the graveyard; ability resolves; creature enters battlefield with haste.

2. `test_unearth_sorcery_speed_restriction` -- CR 702.84a: Cannot activate unearth during opponent's turn, during combat, or with spells on the stack.

3. `test_unearth_exile_at_end_step` -- CR 702.84a: After unearth resolves, at the beginning of the next end step, the creature is exiled. Advance to end step and verify exile.

4. `test_unearth_replacement_exile_on_bounce` -- CR 702.84a: If an unearthed creature would be returned to hand (bounced), it is exiled instead.

5. `test_unearth_replacement_exile_on_destroy` -- CR 702.84a: If an unearthed creature would die (go to graveyard), it is exiled instead.

6. `test_unearth_exile_does_not_replace` -- CR 702.84a ruling: If an effect tries to exile the unearthed creature, the exile succeeds normally (no replacement needed -- destination is already exile).

7. `test_unearth_card_removed_before_resolution` -- Ruling: If the card is removed from the graveyard before the ability resolves, the ability does nothing.

8. `test_unearth_not_a_cast` -- The creature entering via unearth is NOT cast. `spells_cast_this_turn` should not increment. Cast triggers should not fire.

9. `test_unearth_creature_has_haste` -- CR 702.84a: The unearthed creature gains haste. It can attack the turn it enters.

10. `test_unearth_loses_abilities_still_exiled` -- Ruling: Even if the creature loses all abilities (e.g., via a continuous effect), the exile replacement and delayed trigger still apply.

11. `test_unearth_delayed_trigger_countered` -- Ruling: If the delayed trigger is countered (Stifle), the creature stays. The replacement effect still works.

12. `test_unearth_multiplayer_only_active_player` -- Multiplayer: only the active player can activate unearth (sorcery speed).

### Step 9: Card Definition (later phase)

**Suggested card**: Dregscape Zombie
- Name: Dregscape Zombie
- Mana Cost: {1}{B}
- Type: Creature -- Zombie
- P/T: 2/1
- Oracle Text: Unearth {B}
- Keywords: Unearth
- Color Identity: B

**Card lookup**: use `card-definition-author` agent

### Step 10: Game Script (later phase)

**Suggested scenario**: p1 has Dregscape Zombie in graveyard with {B} available. p1 activates unearth during main phase. Zombie returns to battlefield with haste. p1 declares attackers (Zombie attacks p2). At end step, Zombie is exiled.

**Subsystem directory**: `test-data/generated-scripts/stack/` (or new `graveyard/` directory)

## Interactions to Watch

1. **Commander zone return SBA (CR 903.9a)**: If an unearthed commander would be exiled by the unearth replacement, the commander zone return SBA still applies afterward. The owner may choose to move it to the command zone from exile.

2. **Rest in Peace replacement**: If Rest in Peace is on the battlefield, the unearth replacement (exile instead of graveyard) and RiP replacement (exile instead of graveyard) converge -- both want exile. No conflict. But check ordering with `check_zone_change_replacement`.

3. **Flicker/Blink effects**: If an unearthed creature is exiled and returned (e.g., Flickerwisp), it returns as a NEW object (CR 400.7). The `was_unearthed` flag is lost. The creature is no longer subject to the unearth exile effects. This is correct per the ruling.

4. **Humility / Dress Down**: Removing all abilities from the unearthed creature does NOT prevent the exile effects. The `was_unearthed` flag check is independent of the creature's abilities.

5. **Panharmonicon**: If the unearthed creature has an ETB trigger, Panharmonicon should double it normally. The creature entering via unearth IS entering the battlefield.

6. **ETB replacement effects**: Global ETB replacements (e.g., "enters tapped") should apply to unearthed creatures entering the battlefield.

7. **Stifle on the unearth ability itself**: If the unearth activated ability is countered on the stack, the card stays in the graveyard. No exile, no haste, no nothing.

## Files Modified Summary

| File | Action |
|------|--------|
| `crates/engine/src/state/types.rs` | Add `KeywordAbility::Unearth` |
| `crates/engine/src/cards/card_definition.rs` | Add `AbilityDefinition::Unearth { cost }` |
| `crates/engine/src/state/game_object.rs` | Add `was_unearthed: bool` field |
| `crates/engine/src/state/hash.rs` | Hash `was_unearthed` + new StackObjectKind discriminants |
| `crates/engine/src/state/stack.rs` | Add `UnearthAbility` and `UnearthTrigger` variants |
| `crates/engine/src/state/stubs.rs` | Add `is_unearth_trigger: bool` to PendingTrigger |
| `crates/engine/src/rules/command.rs` | Add `Command::UnearthCard` |
| `crates/engine/src/rules/engine.rs` | Add command dispatch for `UnearthCard` |
| `crates/engine/src/rules/abilities.rs` | Add `handle_unearth_card`, `get_unearth_cost`, StepChanged trigger, flush dispatch |
| `crates/engine/src/rules/resolution.rs` | Add resolution handlers for `UnearthAbility` and `UnearthTrigger`; add to counter arms |
| `crates/engine/src/rules/replacement.rs` | Add `was_unearthed` zone-change interception |
| `crates/engine/src/testing/replay_harness.rs` | Add `"unearth_card"` action type |
| `tools/replay-viewer/src/view_model.rs` | Add view model arms for new stack kinds |
| `crates/engine/tests/unearth.rs` | New test file with 12 tests |
