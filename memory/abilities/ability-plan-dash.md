# Ability Plan: Dash

**Generated**: 2026-03-01
**CR**: 702.109
**Priority**: P4
**Batch**: 5 (5.1)
**Similar abilities studied**: Evoke (alternative cost pattern in `casting.rs`), Unearth (delayed end-step trigger pattern in `turn_actions.rs` / `resolution.rs`)

## CR Rule Text

**702.109. Dash**

> 702.109a Dash represents three abilities: two static abilities that function while the card with dash is on the stack, one of which may create a delayed triggered ability, and a static ability that functions while the object with dash is on the battlefield. "Dash [cost]" means "You may cast this card by paying [cost] rather than its mana cost," "If this spell's dash cost was paid, return the permanent this spell becomes to its owner's hand at the beginning of the next end step," and "As long as this permanent's dash cost was paid, it has haste." Casting a spell for its dash cost follows the rules for paying alternative costs in rules 601.2b and 601.2f-h.

**608.3g** (related):
> If the object that's resolving has a static ability that functions on the stack and creates a delayed triggered ability, that delayed triggered ability is created as that permanent is put onto the battlefield in any of the steps above. (See rules 702.109, "Dash," 702.152, "Blitz," and 702.185, "Warp.")

## Key Edge Cases

1. **Alternative cost exclusivity (CR 118.9a)**: Dash is an alternative cost. Only one alternative cost can be applied to a spell. Cannot combine with flashback, evoke, bestow, madness, miracle, escape, foretell, overload, retrace, jump-start, or aftermath.
2. **Mana value unchanged (CR 118.9c)**: Dash does not change the spell's mana cost, only what the controller pays. Spells that ask for the spell's mana cost still see the original value.
3. **Dash does NOT force attacks (ruling 2014-11-24)**: "You don't have to attack with the creature with dash unless another ability says you do." This is purely a common misconception clarification -- no engine enforcement needed beyond not adding any attack requirement.
4. **Return to hand only if still on battlefield (ruling 2014-11-24)**: "If you pay the dash cost to cast a creature spell, that card will be returned to its owner's hand only if it's still on the battlefield when its triggered ability resolves. If it dies or goes to another zone before then, it will stay where it is." The delayed trigger must check battlefield presence at resolution time.
5. **Copies do NOT inherit dash properties (ruling 2014-11-24)**: "If a creature enters the battlefield as a copy of or becomes a copy of a creature whose dash cost was paid, the copy won't have haste and won't be returned to its owner's hand." The `was_dashed` flag on `GameObject` is per-object and is NOT copied by `copy.rs` or token creation.
6. **Dash is a cast (ruling 2014-11-24)**: "If you choose to pay the dash cost rather than the mana cost, you're still casting the spell. It goes on the stack and can be responded to and countered." Uses the standard `CastSpell` command path.
7. **Timing restriction unchanged**: "You can cast a creature spell for its dash cost only when you otherwise could cast that creature spell." Dash does not grant instant-speed casting.
8. **Haste is a static ability, not granted temporarily (CR 702.109a)**: "As long as this permanent's dash cost was paid, it has haste." This is tied to the `was_dashed` flag, NOT a continuous effect with expiry. If an effect removes all abilities (e.g., Humility), the creature loses haste, but the `was_dashed` flag persists and the return-to-hand trigger still fires.
9. **Zone change resets `was_dashed` (CR 400.7)**: If the creature is blinked (exiled and returned), it is a new object. It loses `was_dashed`, loses haste (from dash), and is NOT returned at end step.
10. **Commander tax applies (CR 118.9d)**: If casting a commander with dash, commander tax is added on top of the dash cost.
11. **Multiplayer**: Dash works identically in multiplayer. The delayed trigger is controlled by the original caster. No special multiplayer considerations.
12. **Cost reduction applies to dash cost (ruling from Conduit of Ruin)**: Cost reductions can apply to the dash cost as an alternative cost.

## Current State (from ability-wip.md)

- [ ] 1. Enum variant
- [ ] 2. Rule enforcement
- [ ] 3. Trigger wiring
- [ ] 4. Unit tests
- [ ] 5. Card definition
- [ ] 6. Game script
- [ ] 7. Coverage doc update

## Implementation Steps

### Step 1: Enum Variant & Type Infrastructure

#### 1a. `KeywordAbility::Dash` variant

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/types.rs`
**Action**: Add `Dash` variant after `Encore` (line ~818).
**Pattern**: Follow `KeywordAbility::Encore` at line 810-818.

```rust
/// CR 702.109: Dash [cost] -- alternative cost granting haste and
/// return-to-hand at end step.
///
/// "You may cast this card by paying [cost] rather than its mana cost.
/// If you do, it gains haste, and it's returned from the battlefield to
/// its owner's hand at the beginning of the next end step."
///
/// Marker for quick presence-checking (`keywords.contains`).
/// The dash cost itself is stored in `AbilityDefinition::Dash { cost }`.
Dash,
```

#### 1b. `AbilityDefinition::Dash { cost }` variant

**File**: `/home/airbaggie/scutemob/crates/engine/src/cards/card_definition.rs`
**Action**: Add `Dash { cost: ManaCost }` variant after `Encore { cost: ManaCost }` (line ~306).
**Pattern**: Follow `AbilityDefinition::Evoke { cost }` at line 173-175.

```rust
/// CR 702.109: Dash [cost]. You may cast this card by paying [cost] rather
/// than its mana cost. If you do, the permanent gains haste and is returned
/// to its owner's hand at the beginning of the next end step.
///
/// Cards with this ability should also include
/// `AbilityDefinition::Keyword(KeywordAbility::Dash)` for quick
/// presence-checking without scanning all abilities.
Dash { cost: ManaCost },
```

#### 1c. `StackObjectKind::DashReturnTrigger` variant

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/stack.rs`
**Action**: Add `DashReturnTrigger { source_object: ObjectId }` variant after `EncoreSacrificeTrigger` (line ~680).
**Pattern**: Follow `StackObjectKind::UnearthTrigger` at lines 270-279.

```rust
/// CR 702.109a: Dash delayed triggered ability on the stack.
///
/// "Return the permanent this spell becomes to its owner's hand at the
/// beginning of the next end step."
/// This is a delayed triggered ability created when the dash spell resolves
/// and the permanent enters the battlefield.
///
/// When this trigger resolves:
/// 1. Check if the source is still on the battlefield (CR 400.7).
/// 2. If yes, return it to its owner's hand.
/// 3. If no (died, blinked, bounced), do nothing.
///
/// If countered (e.g., by Stifle), the permanent stays on the battlefield
/// but retains haste (the haste is a static ability linked to was_dashed,
/// not to this trigger).
DashReturnTrigger { source_object: ObjectId },
```

#### 1d. `StackObject` field: `was_dashed`

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/stack.rs`
**Action**: Add `was_dashed: bool` field to `StackObject` struct after `cast_with_aftermath` (line ~151).
**Pattern**: Follow `was_evoked: bool` at lines 59-65.

```rust
/// CR 702.109a: If true, this spell was cast by paying its dash cost
/// (an alternative cost). When the permanent enters the battlefield,
/// it gains haste and a delayed trigger returns it at end step.
///
/// Must always be false for copies (`is_copy: true`) -- copies are not cast.
#[serde(default)]
pub was_dashed: bool,
```

#### 1e. `GameObject` field: `was_dashed`

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/game_object.rs`
**Action**: Add `was_dashed: bool` field after `encore_activated_by` (line ~440 area).
**Pattern**: Follow `was_evoked: bool` at lines 350-354 and `was_unearthed: bool` at lines 401-404.

```rust
/// CR 702.109a: If true, this permanent was cast by paying its dash cost.
/// Grants haste ("as long as this permanent's dash cost was paid, it has haste")
/// and triggers return-to-hand at end step.
///
/// Set during spell resolution when the permanent enters the battlefield.
/// Reset to false on zone changes (CR 400.7).
#[serde(default)]
pub was_dashed: bool,
```

#### 1f. `PendingTrigger` field: `is_dash_return_trigger`

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/stubs.rs`
**Action**: Add `is_dash_return_trigger: bool` field.
**Pattern**: Follow `is_unearth_trigger: bool` at line 133.

```rust
/// CR 702.109a: If true, this pending trigger is a dash return-to-hand trigger.
///
/// When flushed to the stack, creates a `StackObjectKind::DashReturnTrigger`
/// instead of the normal `StackObjectKind::TriggeredAbility`. The `ability_index`
/// field is unused when this is true.
#[serde(default)]
pub is_dash_return_trigger: bool,
```

#### 1g. Hash discriminants

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/hash.rs`
**Action**: Add hash entries for all new types:

1. `KeywordAbility::Dash` -- discriminant **95** (after Encore=94)
2. `StackObjectKind::DashReturnTrigger` -- discriminant **31** (after EncoreSacrificeTrigger=30)
3. `AbilityDefinition::Dash { cost }` -- discriminant **28** (after Encore=27)
4. `StackObject.was_dashed` -- add to StackObject hasher (after `cast_with_aftermath`)
5. `GameObject.was_dashed` -- add to GameObject hasher (after `encore_activated_by`)
6. `PendingTrigger.is_dash_return_trigger` -- add to PendingTrigger hasher (after `is_encore_sacrifice_trigger` / `encore_activator`)

**Pattern**: Follow existing discriminant assignments at:
- KeywordAbility: line ~524 (`KeywordAbility::Encore => 94u8`)
- StackObjectKind: lines ~1572-1580 (`EncoreSacrificeTrigger => 30u8`)
- AbilityDefinition: lines ~3160-3164 (`Encore => 27u8`)
- StackObject fields: lines ~1637-1641
- GameObject fields: lines ~693-697
- PendingTrigger fields: lines ~1135-1136

#### 1h. Initialize `was_dashed: false` at all object creation sites

**Files** (6 sites total):
1. `/home/airbaggie/scutemob/crates/engine/src/state/builder.rs` -- `add_object` call (~line 922)
2. `/home/airbaggie/scutemob/crates/engine/src/effects/mod.rs` -- `create_base_token` function (~line 2488)
3. `/home/airbaggie/scutemob/crates/engine/src/rules/resolution.rs` -- 4 token creation sites (~lines 1246, 2337, 2519, 2718)
4. `/home/airbaggie/scutemob/crates/engine/src/state/mod.rs` -- 2 `move_object_to_zone` sites (~lines 305, 402): reset `was_dashed: false`

**Action**: Add `was_dashed: false,` after `encore_activated_by: None,` at each site.

#### 1i. Initialize `was_dashed: false` on `StackObject` creation

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/casting.rs`
**Action**: Add `was_dashed: false` in the non-dash StackObject creation path (~line 1340), and conditionally set to `true` in the casting handler. Also in replay_harness.rs and all StackObject creation sites in `resolution.rs`.

**File**: `/home/airbaggie/scutemob/crates/engine/src/testing/replay_harness.rs`
**Action**: Add `was_dashed: false` to all `Command::CastSpell` construction sites.

#### 1j. Match arm stubs

Run `grep` for exhaustive matches on `KeywordAbility`, `StackObjectKind`, and `AbilityDefinition` to find all match arms that need a new case:

- `StackObjectKind` match in `resolution.rs` -- add `DashReturnTrigger` resolution arm (Step 2b)
- `StackObjectKind` match in `resolution.rs` fizzle arm (~line 3000) -- add `DashReturnTrigger`
- `StackObjectKind` match in `view_model.rs` (~line 514) -- add `DashReturnTrigger`
- `StackObjectKind` match in `stack_view.rs` (~line 119) -- add `DashReturnTrigger`
- `PendingTrigger` dispatch in `abilities.rs` `flush_pending_triggers` (~line 3652) -- add `is_dash_return_trigger`

### Step 2: Rule Enforcement

#### 2a. Alternative cost handling in `casting.rs`

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/casting.rs`
**Action**: Add dash as an alternative cost option. This involves:

1. **Add `cast_with_dash: bool` parameter** to `handle_cast_spell` function signature (~line 72, after `cast_with_aftermath`).

2. **Add `cast_with_dash` field to `Command::CastSpell`** in `command.rs` (~line 100, after `cast_with_aftermath`).

3. **Add `cast_with_dash` destructuring** in `engine.rs` `Command::CastSpell` match arm (~line 100).

4. **Validate dash (Step 1h in casting.rs)**: Add a validation step after the aftermath validation block (~line 705). Dash is an alternative cost -- mutual exclusion with all other alternative costs:

```rust
// Step 1i: Validate dash (CR 702.109a / CR 118.9a).
// Dash is an alternative cost -- cannot combine with other alternative costs.
let casting_with_dash = if cast_with_dash {
    if casting_with_flashback {
        return Err(GameStateError::InvalidCommand(
            "cannot combine dash with flashback (CR 118.9a: only one alternative cost)".into(),
        ));
    }
    if casting_with_evoke {
        return Err(GameStateError::InvalidCommand(
            "cannot combine dash with evoke (CR 118.9a: only one alternative cost)".into(),
        ));
    }
    if casting_with_bestow {
        return Err(GameStateError::InvalidCommand(
            "cannot combine dash with bestow (CR 118.9a: only one alternative cost)".into(),
        ));
    }
    if casting_with_madness {
        return Err(GameStateError::InvalidCommand(
            "cannot combine dash with madness (CR 118.9a: only one alternative cost)".into(),
        ));
    }
    if cast_with_miracle {
        return Err(GameStateError::InvalidCommand(
            "cannot combine dash with miracle (CR 118.9a: only one alternative cost)".into(),
        ));
    }
    if casting_with_escape {
        return Err(GameStateError::InvalidCommand(
            "cannot combine dash with escape (CR 118.9a: only one alternative cost)".into(),
        ));
    }
    if casting_with_foretell {
        return Err(GameStateError::InvalidCommand(
            "cannot combine dash with foretell (CR 118.9a: only one alternative cost)".into(),
        ));
    }
    if casting_with_overload {
        return Err(GameStateError::InvalidCommand(
            "cannot combine dash with overload (CR 118.9a: only one alternative cost)".into(),
        ));
    }
    if casting_with_retrace {
        return Err(GameStateError::InvalidCommand(
            "cannot combine dash with retrace (CR 118.9a: only one alternative cost)".into(),
        ));
    }
    if casting_with_jump_start {
        return Err(GameStateError::InvalidCommand(
            "cannot combine dash with jump-start (CR 118.9a: only one alternative cost)".into(),
        ));
    }
    if casting_with_aftermath {
        return Err(GameStateError::InvalidCommand(
            "cannot combine dash with aftermath (CR 118.9a: only one alternative cost)".into(),
        ));
    }
    if get_dash_cost(&card_id, &state.card_registry).is_none() {
        return Err(GameStateError::InvalidCommand(
            "spell does not have dash".into(),
        ));
    }
    true
} else {
    false
};
```

Also add dash to ALL existing mutual exclusion checks (evoke, bestow, madness, miracle, escape, foretell, overload, retrace, jump-start, aftermath blocks must reject `casting_with_dash`).

5. **Add dash to cost selection** (~line 708, the `base_cost_before_tax` if-else chain): Add a branch for dash cost:

```rust
} else if casting_with_dash {
    // CR 702.109a: Pay dash cost instead of mana cost.
    // CR 118.9c: The spell's printed mana cost is unchanged; only the payment differs.
    get_dash_cost(&card_id, &state.card_registry)
```

6. **Set `was_dashed` on StackObject** (~line 1340): Set `was_dashed: casting_with_dash,` in the StackObject creation.

7. **Add `get_dash_cost` helper function** (~line 1627, after `get_evoke_cost`):

```rust
/// CR 702.109a: Look up the dash cost from the card's `AbilityDefinition`.
///
/// Returns the `ManaCost` stored in `AbilityDefinition::Dash { cost }`, or `None`
/// if the card has no definition or no dash ability defined.
fn get_dash_cost(
    card_id: &Option<crate::state::CardId>,
    registry: &crate::cards::CardRegistry,
) -> Option<ManaCost> {
    card_id.as_ref().and_then(|cid| {
        registry.get(cid.clone()).and_then(|def| {
            def.abilities.iter().find_map(|a| {
                if let AbilityDefinition::Dash { cost } = a {
                    Some(cost.clone())
                } else {
                    None
                }
            })
        })
    })
}
```

#### 2b. Spell resolution: transfer `was_dashed` and grant haste

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/resolution.rs`
**Action**: In the Spell resolution permanent-ETB block (~line 273-294), after `was_escaped` transfer, add:

```rust
// CR 702.109a: Transfer dashed status from stack to permanent.
// "As long as this permanent's dash cost was paid, it has haste."
// Also set up the delayed triggered ability for return-to-hand.
obj.was_dashed = stack_obj.was_dashed;
if stack_obj.was_dashed {
    // CR 702.109a: "it has haste" -- grant haste keyword.
    obj.characteristics.keywords.insert(KeywordAbility::Haste);
}
```

**CR reference**: 702.109a -- "As long as this permanent's dash cost was paid, it has haste."

**Note**: The haste is granted as a keyword on the `characteristics.keywords` set. If Humility or similar removes all abilities, the keyword goes away (correct per rules). The `was_dashed` flag persists even when abilities are removed, ensuring the return-to-hand trigger still fires (it's a delayed triggered ability from the spell, not an ability of the permanent).

#### 2c. Dash delayed trigger creation at ETB

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/resolution.rs`
**Action**: After the permanent ETB block (~line 294), before the `apply_self_etb_from_definition` call, add:

```rust
// CR 702.109a / CR 608.3g: "If this spell's dash cost was paid, return
// the permanent this spell becomes to its owner's hand at the beginning
// of the next end step." This is a delayed triggered ability created when
// the permanent enters the battlefield.
if stack_obj.was_dashed {
    state.pending_triggers.push_back(PendingTrigger {
        source: new_id,
        ability_index: 0, // unused for dash return triggers
        controller,
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
        is_suspend_counter_trigger: false,
        is_suspend_cast_trigger: false,
        suspend_card_id: None,
        is_hideaway_trigger: false,
        hideaway_count: None,
        is_partner_with_trigger: false,
        partner_name: None,
        partner_target_player: None,
        is_ingest_trigger: false,
        ingest_target_player: None,
        is_flanking_trigger: false,
        flanking_blocker_id: None,
        is_rampage_trigger: false,
        rampage_n: None,
        is_provoke_trigger: false,
        provoke_creature_id: None,
        is_renown_trigger: false,
        renown_n: None,
        is_melee_trigger: false,
        is_poisonous_trigger: false,
        poisonous_n: None,
        poisonous_target_player: None,
        is_enlist_trigger: false,
        enlist_enlisted_creature: None,
        is_encore_sacrifice_trigger: false,
        encore_activator: None,
        is_dash_return_trigger: true,
    });
}
```

**WAIT -- Important design choice**: The trigger should NOT fire immediately at ETB. Per CR 702.109a, the return happens "at the beginning of the next end step." The PendingTrigger is created at resolution time, but it is a DELAYED trigger -- it should fire at end step, not immediately. So the trigger should NOT be pushed into `pending_triggers` at ETB time. Instead, the `was_dashed` flag should be read by `end_step_actions()` in `turn_actions.rs` (the same pattern Unearth uses).

**Revised approach**: Do NOT create a PendingTrigger at ETB time. Instead, mark the permanent with `was_dashed = true` (already done in 2b). Then in `end_step_actions()` in `turn_actions.rs`, scan for `was_dashed == true` permanents and queue `DashReturnTrigger` pending triggers. This is exactly the Unearth pattern.

### Step 3: Trigger Wiring

#### 3a. End step trigger queueing

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/turn_actions.rs`
**Action**: In `end_step_actions()` (~line 130), after the Unearth trigger collection (~lines 131-190) and Encore trigger collection (~lines 192-240), add a third block for Dash:

```rust
// CR 702.109a: Queue return-to-hand triggers for all dashed permanents.
// "Return the permanent this spell becomes to its owner's hand at the
// beginning of the next end step."
// Each dashed permanent is tagged with `was_dashed = true` when the dash
// spell resolves (resolution.rs). At end step, we queue a DashReturnTrigger.
let dashed_permanents: Vec<(ObjectId, crate::state::player::PlayerId)> = state
    .objects
    .values()
    .filter(|obj| obj.zone == ZoneId::Battlefield && obj.was_dashed)
    .map(|obj| (obj.id, obj.controller))
    .collect();

for (obj_id, controller) in dashed_permanents {
    state.pending_triggers.push_back(PendingTrigger {
        source: obj_id,
        ability_index: 0, // unused for dash return triggers
        controller,
        triggering_event: None,
        entering_object_id: None,
        targeting_stack_id: None,
        triggering_player: None,
        exalted_attacker_id: None,
        defending_player_id: None,
        is_evoke_sacrifice: false,
        // ... all other trigger flags false ...
        is_dash_return_trigger: true,
    });
}
```

#### 3b. Flush `is_dash_return_trigger` to `StackObjectKind::DashReturnTrigger`

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/abilities.rs`
**Action**: In `flush_pending_triggers()`, in the if-else chain that maps `PendingTrigger` flags to `StackObjectKind` (~line 3652, after the `is_unearth_trigger` arm), add:

```rust
} else if trigger.is_dash_return_trigger {
    // CR 702.109a: Dash delayed return trigger -- "return the permanent to
    // its owner's hand at the beginning of the next end step."
    StackObjectKind::DashReturnTrigger {
        source_object: trigger.source,
    }
```

Also add `is_dash_return_trigger: false,` to every other `PendingTrigger` construction site in `abilities.rs` (there are many -- ~15+ sites based on grep results).

#### 3c. `DashReturnTrigger` resolution

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/resolution.rs`
**Action**: Add a new resolution arm for `StackObjectKind::DashReturnTrigger` after the `UnearthTrigger` arm (~line 973). Follow the Unearth pattern but return to hand instead of exile:

```rust
// CR 702.109a: Dash delayed triggered ability resolves.
//
// "Return the permanent this spell becomes to its owner's hand at the
// beginning of the next end step."
// If the source has left the battlefield by resolution time (CR 400.7),
// the trigger does nothing -- the creature is a new object elsewhere.
StackObjectKind::DashReturnTrigger { source_object } => {
    let controller = stack_obj.controller;

    // Check if the source is still on the battlefield (CR 400.7).
    let owner_opt = state
        .objects
        .get(&source_object)
        .filter(|obj| obj.zone == ZoneId::Battlefield)
        .map(|obj| obj.owner);

    if let Some(owner) = owner_opt {
        // Return to owner's hand (not controller's -- CR 702.109a says "owner's hand").
        let (new_id, _) = state.move_object_to_zone(source_object, ZoneId::Hand(owner))?;

        events.push(GameEvent::ObjectMovedZone {
            object_id: source_object,
            new_object_id: new_id,
            from: ZoneId::Battlefield,
            to: ZoneId::Hand(owner),
        });
    }

    events.push(GameEvent::AbilityResolved {
        controller,
        stack_object_id: stack_obj.id,
    });
}
```

#### 3d. Add `DashReturnTrigger` to the fizzle/counter match arm

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/resolution.rs`
**Action**: Add `| StackObjectKind::DashReturnTrigger { .. }` to the fizzle match arm (~line 3004) alongside `UnearthTrigger`, `EvokeSacrificeTrigger`, etc.

### Step 4: Supporting File Updates

#### 4a. `Command::CastSpell` field

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/command.rs`
**Action**: Add `cast_with_dash: bool` field to `CastSpell` variant (~line 100).

```rust
/// CR 702.109a: If true, this spell is being cast by paying its dash cost
/// (an alternative cost). The permanent gains haste and returns to hand at end step.
#[serde(default)]
cast_with_dash: bool,
```

#### 4b. `engine.rs` dispatch

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/engine.rs`
**Action**: Add `cast_with_dash` to the `Command::CastSpell` destructuring (~line 100) and pass it to `handle_cast_spell` (~line 125).

#### 4c. Replay harness: `cast_spell_dash` action type

**File**: `/home/airbaggie/scutemob/crates/engine/src/testing/replay_harness.rs`
**Action**:
1. Add `cast_with_dash: false` to ALL existing `Command::CastSpell` construction sites (~15 sites).
2. Add a new `"cast_spell_dash"` arm that creates a `CastSpell` with `cast_with_dash: true`:

```rust
// CR 702.109a: Cast a spell with dash from the player's hand.
// The dash cost (an alternative cost) is paid instead of the mana cost.
"cast_spell_dash" => {
    let card_id = find_in_hand(state, player, card_name?)?;
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
        cast_with_foretell: false,
        cast_with_buyback: false,
        cast_with_overload: false,
        retrace_discard_land: None,
        cast_with_jump_start: false,
        jump_start_discard: None,
        cast_with_aftermath: false,
        cast_with_dash: true,
    })
}
```

#### 4d. TUI `stack_view.rs` match arm

**File**: `/home/airbaggie/scutemob/tools/tui/src/play/panels/stack_view.rs`
**Action**: Add match arm for `DashReturnTrigger` (~line 121, after `EncoreSacrificeTrigger`):

```rust
StackObjectKind::DashReturnTrigger { source_object } => {
    ("Dash return: ".to_string(), Some(*source_object))
}
```

#### 4e. Replay viewer `view_model.rs` match arm

**File**: `/home/airbaggie/scutemob/tools/replay-viewer/src/view_model.rs`
**Action**: Add match arm for `DashReturnTrigger` (~line 514, after `EncoreSacrificeTrigger`):

```rust
StackObjectKind::DashReturnTrigger { source_object } => {
    ("dash_return_trigger", Some(*source_object))
}
```

#### 4f. Haste from `was_dashed` in `calculate_characteristics`

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/layers.rs` (or wherever `calculate_characteristics` lives)
**Action**: After Layer 6 processing (where abilities are added/removed), check if the object has `was_dashed` and, if Haste was removed by an ability-removing effect, re-add it.

**WAIT**: Actually, re-reading the CR: "As long as this permanent's dash cost was paid, it has haste." This is a STATIC ability of the permanent. If an effect removes all abilities, the permanent loses this static ability too, and therefore loses haste. The `was_dashed` flag ensures the return-to-hand trigger still fires (it's a delayed trigger from the spell, independent of the permanent's abilities).

So the haste grant at resolution time (Step 2b, adding `KeywordAbility::Haste` to `characteristics.keywords`) is the correct approach. We do NOT need a Layer system check. If Humility removes all abilities including Haste, that is correct behavior. The creature will NOT have haste under Humility (correct -- Humility removes the static ability that grants haste). The return-to-hand delayed trigger fires regardless (it's on the stack, independent of the permanent).

**Updated conclusion**: No `calculate_characteristics` change needed. The haste is inserted at resolution time in `resolution.rs` and is handled by the normal layer system (Layer 6 ability removal like Humility will remove it, which is correct per CR).

### Step 5: Unit Tests

**File**: `/home/airbaggie/scutemob/crates/engine/tests/dash.rs`

**Tests to write**:

1. **`test_dash_basic_cast_with_dash_cost`** -- CR 702.109a
   - Set up: Player has a creature card with Dash in hand and enough mana for dash cost.
   - Action: Cast with `cast_with_dash: true`.
   - Assert: Spell goes on the stack. After resolution, creature is on the battlefield with haste (no summoning sickness effect). The `was_dashed` flag is true.
   - Pattern: Follow `test_evoke_basic_cast_with_evoke_cost` in `evoke.rs`.

2. **`test_dash_return_to_hand_at_end_step`** -- CR 702.109a
   - Set up: Player has a dashed creature on the battlefield (`was_dashed: true`).
   - Action: Enter end step (advance turn to end step).
   - Assert: DashReturnTrigger is queued. After resolution, creature is in owner's hand.
   - Pattern: Follow `test_unearth_exile_at_end_step` in `unearth.rs`.

3. **`test_dash_creature_died_before_end_step`** -- Ruling 2014-11-24
   - Set up: Player has a dashed creature on the battlefield. Creature dies (SBA from damage).
   - Action: Advance to end step.
   - Assert: No DashReturnTrigger fires (creature already left the battlefield). Creature stays in graveyard.
   - Pattern: Follow `test_unearth_card_removed_before_resolution` in `unearth.rs`.

4. **`test_dash_normal_cast_no_return`** -- Negative test
   - Set up: Player has a creature with Dash in hand, casts it normally (not dashed).
   - Action: Cast without `cast_with_dash`. Advance to end step.
   - Assert: Creature stays on battlefield. No DashReturnTrigger fires. No haste from dash.

5. **`test_dash_alternative_cost_exclusivity`** -- CR 118.9a
   - Set up: Player has a creature with both Dash and Flashback (or mock card with both).
   - Action: Try to cast with `cast_with_dash: true, cast_with_flashback: true` (and other combinations).
   - Assert: Error returned for each combination.
   - Pattern: Follow `test_evoke_cannot_combine_with_flashback` in `evoke.rs`.

6. **`test_dash_mana_value_unchanged`** -- CR 118.9c
   - Set up: Cast a creature with Dash cost different from mana cost.
   - Action: Cast with dash. Check mana value on the stack.
   - Assert: Mana value reflects the original mana cost, not the dash cost.
   - Pattern: Follow `test_evoke_does_not_change_mana_value` in `evoke.rs`.

7. **`test_dash_commander_tax_applies`** -- CR 118.9d
   - Set up: Commander with Dash keyword. Cast from command zone once, it dies, cast again.
   - Action: Second cast with dash from command zone.
   - Assert: Total cost = dash cost + {2} commander tax.

8. **`test_dash_multiplayer_return_owner_hand`** -- Multiplayer
   - Set up: 4-player game. Player A casts a creature with dash. Control changes to Player B.
   - Action: End step arrives.
   - Assert: Creature returns to Player A's hand (owner's hand, not controller's hand).

9. **`test_dash_copy_does_not_inherit_dash`** -- Ruling 2014-11-24
   - Set up: Player dashes a creature. Another effect creates a copy of it.
   - Action: Check copy's properties.
   - Assert: Copy does NOT have `was_dashed = true`, does NOT have haste from dash, is NOT returned at end step.

### Step 6: Card Definition (later phase)

**Suggested card**: Goblin Heelcutter
- Simple Dash creature with a triggered ability (combat trigger is additional but not required for basic Dash testing).
- Mana cost: {3}{R}, Dash cost: {2}{R}, P/T: 3/2
- Use `card-definition-author` agent.

**Alternative simpler card**: Zurgo Bellstriker
- Mana cost: {R}, Dash cost: {1}{R}, P/T: 2/2
- Legendary Creature -- also tests legend rule interaction.
- Has a blocking restriction (can't block power >= 2) which is separate but doesn't interfere.

**Best choice for initial card**: Zurgo Bellstriker (simplest -- Dash only, no combat trigger interaction).

### Step 7: Game Script (later phase)

**Suggested scenario**: Basic Dash lifecycle
- Player casts Zurgo Bellstriker for Dash cost {1}{R}
- Creature enters battlefield with haste
- Player attacks with it
- Combat damage resolves
- End step: delayed trigger returns Zurgo to hand
- Assertions: life totals reflect combat damage, Zurgo is in hand at end

**Subsystem directory**: `test-data/generated-scripts/stack/` (alternative cost is a stack/casting concern)

## Interactions to Watch

1. **Evoke vs Dash mutual exclusion**: A card could theoretically have both Evoke and Dash. Only one alternative cost can be used. The engine's existing mutual exclusion validation in `casting.rs` handles this.
2. **Blink interaction (CR 400.7)**: If a dashed creature is blinked (exiled then returned), it is a new object. `was_dashed` resets to `false` in `move_object_to_zone`. The creature stays on the battlefield permanently (no return trigger, no haste from dash). Correct per rules.
3. **Stifle/Disallow**: The DashReturnTrigger can be countered (it uses the stack). If countered, the creature stays on the battlefield with haste (haste is a static ability, not tied to the trigger). The creature is effectively "saved" from returning.
4. **Humility**: Removes all abilities including the static ability that grants haste. The creature loses haste. The return-to-hand delayed trigger STILL fires (it's a triggered ability from the spell, not an ability of the permanent). Creature returns to hand at end step even under Humility.
5. **Copy effects**: `was_dashed` is NOT part of copiable values. Copies do not inherit it. This is already handled by `was_dashed: false` initialization in copy/token creation sites.
6. **Multiple Dash creatures**: Each gets its own independent delayed trigger. They all fire at end step and resolve in APNAP order (or as chosen by the controller if all have the same controller).

## File Change Summary

| File | Change |
|------|--------|
| `crates/engine/src/state/types.rs` | Add `KeywordAbility::Dash` |
| `crates/engine/src/cards/card_definition.rs` | Add `AbilityDefinition::Dash { cost }` |
| `crates/engine/src/state/stack.rs` | Add `StackObjectKind::DashReturnTrigger`, add `was_dashed: bool` to `StackObject` |
| `crates/engine/src/state/game_object.rs` | Add `was_dashed: bool` to `GameObject` |
| `crates/engine/src/state/stubs.rs` | Add `is_dash_return_trigger: bool` to `PendingTrigger` |
| `crates/engine/src/state/hash.rs` | Add hash discriminants 95/31/28, hash new fields |
| `crates/engine/src/state/builder.rs` | Init `was_dashed: false` |
| `crates/engine/src/state/mod.rs` | Reset `was_dashed: false` in both `move_object_to_zone` |
| `crates/engine/src/effects/mod.rs` | Init `was_dashed: false` in `create_base_token` |
| `crates/engine/src/rules/command.rs` | Add `cast_with_dash: bool` to `CastSpell` |
| `crates/engine/src/rules/engine.rs` | Destructure + pass `cast_with_dash` |
| `crates/engine/src/rules/casting.rs` | Add `cast_with_dash` param, validation, cost selection, `get_dash_cost()` |
| `crates/engine/src/rules/resolution.rs` | Transfer `was_dashed`, grant haste, add `DashReturnTrigger` resolution, add to fizzle arm |
| `crates/engine/src/rules/turn_actions.rs` | Queue `DashReturnTrigger` in `end_step_actions()` |
| `crates/engine/src/rules/abilities.rs` | Map `is_dash_return_trigger` to `DashReturnTrigger`, init `false` at all PendingTrigger sites |
| `crates/engine/src/testing/replay_harness.rs` | Add `cast_with_dash: false` everywhere, add `cast_spell_dash` action |
| `tools/tui/src/play/panels/stack_view.rs` | Add `DashReturnTrigger` match arm |
| `tools/replay-viewer/src/view_model.rs` | Add `DashReturnTrigger` match arm |
| `crates/engine/tests/dash.rs` | 9 unit tests |
