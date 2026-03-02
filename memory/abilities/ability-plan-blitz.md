# Ability Plan: Blitz

**Generated**: 2026-03-02
**CR**: 702.152
**Priority**: P4
**Similar abilities studied**: Dash (CR 702.109) -- `crates/engine/src/rules/casting.rs`, `resolution.rs`, `turn_actions.rs`, `abilities.rs`, `tests/dash.rs`

## CR Rule Text

702.152. Blitz

702.152a: Blitz represents three abilities: two static abilities that function while the card with blitz is on the stack, one of which may create a delayed triggered ability, and a static ability that functions while the object with blitz is on the battlefield. "Blitz [cost]" means "You may cast this card by paying [cost] rather than its mana cost," "If this spell's blitz cost was paid, sacrifice the permanent this spell becomes at the beginning of the next end step," and "As long as this permanent's blitz cost was paid, it has haste and 'When this permanent is put into a graveyard from the battlefield, draw a card.'" Casting a spell for its blitz cost follows the rules for paying alternative costs in rules 601.2b and 601.2f-h.

702.152b: If a spell has multiple instances of blitz, only one may be used to cast that spell. If a permanent has multiple instances of blitz, each one refers only to payments made for that blitz ability as the spell was cast, not to any payments made for other instances of blitz.

## Key Edge Cases

- **Draw triggers on any death, not just end-step sacrifice** (Mezzio Mugger ruling 2022-04-29): "The triggered ability that lets its controller draw a card triggers when it dies for any reason, not just when you sacrifice it during the end step."
- **Sacrifice only if still on battlefield** (Ruling 2022-04-29): "If you pay the blitz cost to cast a creature spell, that permanent will be sacrificed only if it's still on the battlefield when that triggered ability resolves. If it dies or goes to another zone before then, it will stay where it is."
- **Copies do not inherit blitz benefits** (Ruling 2022-04-29): "If a creature enters the battlefield as a copy of or becomes a copy of a creature whose blitz cost was paid, the copy won't have haste, won't be sacrificed, and its controller won't draw a card when it dies." -- Handled automatically by `cast_alt_cost` check (copies never go through CastSpell).
- **No forced attack** (Ruling 2022-04-29): "You don't have to attack with the creature with blitz unless another ability says you do."
- **Blitz is an alternative cost** (CR 118.9a): cannot combine with flashback, evoke, or other alternative costs.
- **Commander tax applies on top of blitz cost** (CR 118.9d): same as dash.
- **Multiplayer**: Blitz sacrifice trigger fires for the permanent's controller. In multiplayer, if control changes (e.g., via Homeward Path), the new controller's end step sacrifices the creature. The draw trigger fires for the controller at death time (CR 603.3a).

## Differences from Dash (the pattern model)

| Aspect | Dash | Blitz |
|--------|------|-------|
| End step action | Return to hand | Sacrifice |
| Haste | Yes | Yes |
| Death benefit | None | Draw a card |
| End step trigger type | `DashReturnTrigger` (move to hand) | `BlitzSacrificeTrigger` (sacrifice) |
| Additional trigger | None | SelfDies draw trigger (granted at ETB) |

## Current State (from ability-wip.md)

- [ ] 1. Enum variant
- [ ] 2. Rule enforcement
- [ ] 3. Trigger wiring
- [ ] 4. Unit tests
- [ ] 5. Card definition
- [ ] 6. Game script
- [ ] 7. Coverage doc update

## Verified Assumptions

These were confirmed during planning by reading the actual source:

1. **CONFIRMED**: `TriggeredAbilityDef` has `pub effect: Option<crate::cards::card_definition::Effect>` at `game_object.rs:249`. The inline draw effect via `Effect::DrawCards` works with the standard `TriggeredAbility` resolution path (resolution.rs:658-672).
2. **CONFIRMED**: All sacrifice/destroy paths use `move_object_to_zone` directly (no separate `sacrifice_permanent` helper). The evoke sacrifice trigger (resolution.rs:718+) uses the same pattern.
3. **CONFIRMED**: `check_triggers` for `CreatureDied` iterates `SelfDies` triggers from the graveyard object's `triggered_abilities` (abilities.rs:2637-2724). Adding a SelfDies trigger at resolution time means it fires automatically on any death.
4. **CONFIRMED**: `triggered_abilities` is `Vec<TriggeredAbilityDef>` (not `im::Vector`), so use `.push()` not `.push_back()`.
5. **CONFIRMED**: `PlayerTarget::Controller` in a SelfDies trigger resolves to the death-time controller (captured in `CreatureDied.controller`, set on `PendingTrigger.controller`, propagated to `StackObject.controller`, used in `EffectContext`).

## Implementation Steps

### Step 1: Enum Variants and Type Changes

#### 1a. AltCostKind::Blitz

**File**: `crates/engine/src/state/types.rs`
**Action**: Add `Blitz` variant to `AltCostKind` enum after `Dash` (line 109).
**Pattern**: Follow `Dash` at line 109.
**Note**: `AltCostKind` derives `Hash` -- no manual hash impl needed.

```rust
Dash,
Blitz,
// Future: Plot, Prototype, Impending (add as implemented)
```

Also update the comment on line 110 to remove Blitz from the "Future" list.

#### 1b. KeywordAbility::Blitz

**File**: `crates/engine/src/state/types.rs`
**Action**: Add `Blitz` variant to `KeywordAbility` enum after `Dash` (line 847).
**Pattern**: Follow `Dash` at lines 838-847.
**CR**: 702.152a

```rust
/// CR 702.152: Blitz [cost] -- alternative cost granting haste,
/// sacrifice at end step, and draw-a-card on death.
///
/// Marker for quick presence-checking (`keywords.contains`).
/// The blitz cost itself is stored in `AbilityDefinition::Blitz { cost }`.
Blitz,
```

**Hash**: Add to `state/hash.rs` KeywordAbility HashInto impl after `Dash => 95u8` (line 526).
Next discriminant: **96**.

```rust
// Blitz (discriminant 96) -- CR 702.152
KeywordAbility::Blitz => 96u8.hash_into(hasher),
```

#### 1c. AbilityDefinition::Blitz

**File**: `crates/engine/src/cards/card_definition.rs`
**Action**: Add `Blitz { cost: ManaCost }` variant to `AbilityDefinition` enum after `Dash { cost: ManaCost }` (line 341).
**Pattern**: Follow `Dash` at lines 334-341.
**CR**: 702.152a

```rust
/// CR 702.152: Blitz [cost]. You may cast this card by paying [cost] rather
/// than its mana cost. If you do, the permanent gains haste, is sacrificed
/// at the beginning of the next end step, and gains "When this permanent is
/// put into a graveyard from the battlefield, draw a card."
///
/// Cards with this ability should also include
/// `AbilityDefinition::Keyword(KeywordAbility::Blitz)` for quick
/// presence-checking without scanning all abilities.
Blitz { cost: ManaCost },
```

**Hash**: Add to `state/hash.rs` AbilityDefinition HashInto impl after `Dash { cost } => 28u8` (line 3152).
Next discriminant: **29**.

```rust
// Blitz (discriminant 29) -- CR 702.152
AbilityDefinition::Blitz { cost } => {
    29u8.hash_into(hasher);
    cost.hash_into(hasher);
}
```

#### 1d. StackObjectKind::BlitzSacrificeTrigger

**File**: `crates/engine/src/state/stack.rs`
**Action**: Add `BlitzSacrificeTrigger { source_object: ObjectId }` variant to `StackObjectKind` after `DashReturnTrigger` (line 703).
**Pattern**: Follow `DashReturnTrigger` at lines 688-703, but sacrifice instead of return.
**CR**: 702.152a

```rust
/// CR 702.152a: Blitz delayed triggered ability on the stack.
///
/// "Sacrifice the permanent this spell becomes at the beginning of the
/// next end step."
/// This is a delayed triggered ability created when the blitz spell resolves
/// and the permanent enters the battlefield.
///
/// When this trigger resolves:
/// 1. Check if the source is still on the battlefield (CR 400.7).
/// 2. If yes, sacrifice it (move to graveyard, which fires CreatureDied).
/// 3. If no (already died, blinked, bounced), do nothing.
///
/// If countered (e.g., by Stifle), the permanent stays on the battlefield
/// but retains haste and the draw-on-death trigger (those are static
/// abilities linked to cast_alt_cost, not to this trigger -- CR 702.152a).
BlitzSacrificeTrigger { source_object: ObjectId },
```

**Hash**: Add to `state/hash.rs` StackObjectKind HashInto impl after `DashReturnTrigger => 31u8` (line 1559).
Next discriminant: **32**.

```rust
// BlitzSacrificeTrigger (discriminant 32) -- CR 702.152a
StackObjectKind::BlitzSacrificeTrigger { source_object } => {
    32u8.hash_into(hasher);
    source_object.hash_into(hasher);
}
```

#### 1e. PendingTriggerKind::BlitzSacrifice

**File**: `crates/engine/src/state/stubs.rs`
**Action**: Add `BlitzSacrifice` variant to `PendingTriggerKind` after `DashReturn` (line 69).
**Pattern**: Follow `DashReturn` at lines 68-69.
**CR**: 702.152a

```rust
/// CR 702.152a: Blitz delayed sacrifice trigger.
BlitzSacrifice,
```

#### 1f. StackObject.was_blitzed field

**File**: `crates/engine/src/state/stack.rs`
**Action**: Add `was_blitzed: bool` field to `StackObject` struct after `was_dashed` (line 158).
**Pattern**: Follow `was_dashed` at lines 152-158.
**CR**: 702.152a

```rust
/// CR 702.152a: If true, this spell was cast by paying its blitz cost
/// (an alternative cost). When the permanent enters the battlefield,
/// it gains haste, gains "When this dies, draw a card," and a delayed
/// trigger sacrifices it at the beginning of the next end step.
///
/// Must always be false for copies (`is_copy: true`) -- copies are not cast.
#[serde(default)]
pub was_blitzed: bool,
```

**Hash**: Add to `state/hash.rs` StackObject HashInto impl after `self.was_dashed` (line 1622).

```rust
// Blitz (CR 702.152a) -- alternative cost paid; haste + draw-on-death + sacrifice trigger
self.was_blitzed.hash_into(hasher);
```

#### 1g. All StackObject construction sites: add `was_blitzed: false`

Every site that constructs a `StackObject` struct literal must include `was_blitzed: false`.
All 16 existing sites (same files as `was_dashed: false`):

**Files and lines** (search for `was_dashed:` to find all):
- `casting.rs:1418` -- main CastSpell: set `was_blitzed: casting_with_blitz`
- `casting.rs:1526` -- cascade free-cast: `was_blitzed: false`
- `casting.rs:1572` -- cascade (2nd path): `was_blitzed: false`
- `copy.rs:199` -- storm copy: `was_blitzed: false`
- `copy.rs:381` -- cascade copy: `was_blitzed: false`
- `abilities.rs:347` -- (trigger flush): `was_blitzed: false`
- `abilities.rs:569` -- (trigger flush): `was_blitzed: false`
- `abilities.rs:745` -- (trigger flush): `was_blitzed: false`
- `abilities.rs:1000` -- (trigger flush): `was_blitzed: false`
- `abilities.rs:1192` -- (trigger flush): `was_blitzed: false`
- `abilities.rs:1393` -- (trigger flush): `was_blitzed: false`
- `abilities.rs:1593` -- (trigger flush): `was_blitzed: false`
- `abilities.rs:3450` -- (encore trigger flush): `was_blitzed: false`
- `abilities.rs:3663` -- (dash trigger flush): `was_blitzed: false`
- `abilities.rs:4018` -- (trigger flush): `was_blitzed: false`
- `resolution.rs:1513` -- (suspend free-cast): `was_blitzed: false`

#### 1h. TUI stack_view.rs

**File**: `tools/tui/src/play/panels/stack_view.rs`
**Action**: Add match arm for `StackObjectKind::BlitzSacrificeTrigger` after `DashReturnTrigger` (line 124).
**Pattern**: Follow `DashReturnTrigger` at lines 122-123.

```rust
StackObjectKind::BlitzSacrificeTrigger { source_object } => {
    ("Blitz sacrifice: ".to_string(), Some(*source_object))
}
```

#### 1i. Replay viewer view_model.rs

**File**: `tools/replay-viewer/src/view_model.rs`
**Action**: Add match arm for `StackObjectKind::BlitzSacrificeTrigger` after `DashReturnTrigger` (line 517).
**Pattern**: Follow `DashReturnTrigger` at lines 515-516.

```rust
StackObjectKind::BlitzSacrificeTrigger { source_object } => {
    ("blitz_sacrifice_trigger", Some(*source_object))
}
```

### Step 2: Rule Enforcement (Casting)

**File**: `crates/engine/src/rules/casting.rs`
**Action**: Add blitz casting logic, following the exact dash pattern.
**Pattern**: Follow dash casting logic at lines 77, 711-773, 854-858, 1418, 1837-1855.
**CR**: 702.152a, 601.2b, 601.2f-h, 118.9a

#### 2a. Extract `cast_with_blitz` boolean (near line 77)

```rust
let cast_with_blitz = alt_cost == Some(AltCostKind::Blitz);
```

#### 2b. Validate blitz mutual exclusion (after dash block, near line 773)

Add a new block after the dash validation block:

```rust
// Step 1j: Validate blitz mutual exclusion (CR 702.152a / CR 118.9a).
// Blitz is an alternative cost -- cannot combine with other alternative costs.
let casting_with_blitz = if cast_with_blitz {
    if casting_with_flashback {
        return Err(GameStateError::InvalidCommand(
            "cannot combine blitz with flashback (CR 118.9a: only one alternative cost)".into(),
        ));
    }
    if casting_with_evoke {
        return Err(GameStateError::InvalidCommand(
            "cannot combine blitz with evoke (CR 118.9a: only one alternative cost)".into(),
        ));
    }
    if casting_with_bestow {
        return Err(GameStateError::InvalidCommand(
            "cannot combine blitz with bestow (CR 118.9a: only one alternative cost)".into(),
        ));
    }
    if casting_with_madness {
        return Err(GameStateError::InvalidCommand(
            "cannot combine blitz with madness (CR 118.9a: only one alternative cost)".into(),
        ));
    }
    if cast_with_miracle {
        return Err(GameStateError::InvalidCommand(
            "cannot combine blitz with miracle (CR 118.9a: only one alternative cost)".into(),
        ));
    }
    if casting_with_escape {
        return Err(GameStateError::InvalidCommand(
            "cannot combine blitz with escape (CR 118.9a: only one alternative cost)".into(),
        ));
    }
    if casting_with_foretell {
        return Err(GameStateError::InvalidCommand(
            "cannot combine blitz with foretell (CR 118.9a: only one alternative cost)".into(),
        ));
    }
    if casting_with_overload {
        return Err(GameStateError::InvalidCommand(
            "cannot combine blitz with overload (CR 118.9a: only one alternative cost)".into(),
        ));
    }
    if casting_with_retrace {
        return Err(GameStateError::InvalidCommand(
            "cannot combine blitz with retrace (CR 118.9a: only one alternative cost)".into(),
        ));
    }
    if casting_with_jump_start {
        return Err(GameStateError::InvalidCommand(
            "cannot combine blitz with jump-start (CR 118.9a: only one alternative cost)".into(),
        ));
    }
    if casting_with_aftermath {
        return Err(GameStateError::InvalidCommand(
            "cannot combine blitz with aftermath (CR 118.9a: only one alternative cost)".into(),
        ));
    }
    if casting_with_dash {
        return Err(GameStateError::InvalidCommand(
            "cannot combine blitz with dash (CR 118.9a: only one alternative cost)".into(),
        ));
    }
    if get_blitz_cost(&card_id, &state.card_registry).is_none() {
        return Err(GameStateError::InvalidCommand(
            "spell does not have blitz".into(),
        ));
    }
    true
} else {
    false
};
```

Also: add `casting_with_blitz` to dash's exclusion checks (add `if casting_with_blitz { return Err(...) }` inside the dash block, after the aftermath check near line 768).

#### 2c. Base mana cost override for blitz (after dash's cost override, near line 858)

```rust
} else if casting_with_blitz {
    // CR 702.152a: Pay blitz cost instead of mana cost.
    // CR 118.9c: The spell's printed mana cost is unchanged; only the payment differs.
    get_blitz_cost(&card_id, &state.card_registry)
}
```

#### 2d. Set `was_blitzed` on StackObject (at main CastSpell construction, line 1418 area)

```rust
was_blitzed: casting_with_blitz,
```

#### 2e. Add `get_blitz_cost` helper function (after `get_dash_cost` at line 1855)

```rust
/// CR 702.152a: Look up the blitz cost from the card's `AbilityDefinition`.
///
/// Returns the `ManaCost` stored in `AbilityDefinition::Blitz { cost }`, or `None`
/// if the card has no definition or no blitz ability defined.
fn get_blitz_cost(
    card_id: &Option<crate::state::CardId>,
    registry: &crate::cards::CardRegistry,
) -> Option<ManaCost> {
    card_id.as_ref().and_then(|cid| {
        registry.get(cid.clone()).and_then(|def| {
            def.abilities.iter().find_map(|a| {
                if let AbilityDefinition::Blitz { cost } = a {
                    Some(cost.clone())
                } else {
                    None
                }
            })
        })
    })
}
```

### Step 3: Resolution (ETB Effects)

**File**: `crates/engine/src/rules/resolution.rs`
**Action**: Transfer blitz status from stack to permanent at resolution. Grant haste keyword and add a SelfDies draw trigger.
**Pattern**: Follow dash at lines 281-298.
**CR**: 702.152a

#### 3a. Transfer `was_blitzed` to `cast_alt_cost` (line 282-290 area)

Add `was_blitzed` check in the `cast_alt_cost` assignment chain:

```rust
obj.cast_alt_cost = if stack_obj.was_evoked {
    Some(AltCostKind::Evoke)
} else if stack_obj.was_escaped {
    Some(AltCostKind::Escape)
} else if stack_obj.was_dashed {
    Some(AltCostKind::Dash)
} else if stack_obj.was_blitzed {
    Some(AltCostKind::Blitz)
} else {
    None
};
```

#### 3b. Grant haste and add draw-on-death trigger (after dash haste grant, line 298 area)

```rust
if stack_obj.was_blitzed {
    // CR 702.152a: "it has haste" -- grant haste keyword.
    obj.characteristics.keywords.insert(KeywordAbility::Haste);
    // CR 702.152a: "'When this permanent is put into a graveyard
    // from the battlefield, draw a card.'" -- add SelfDies trigger.
    // Uses standard TriggeredAbilityDef with inline Effect::DrawCards.
    // Resolves through the standard TriggeredAbility resolution path
    // (resolution.rs:620-679) when the creature dies via any path.
    obj.characteristics.triggered_abilities.push(
        TriggeredAbilityDef {
            trigger_on: TriggerEvent::SelfDies,
            intervening_if: None,
            description: "Blitz (CR 702.152a): When this permanent is put into \
                          a graveyard from the battlefield, draw a card."
                .to_string(),
            effect: Some(Effect::DrawCards {
                player: PlayerTarget::Controller,
                count: EffectAmount::Fixed(1),
            }),
        },
    );
}
```

**Required imports** in resolution.rs (verify these are already imported; add if not):
- `TriggerEvent` -- from `crate::state::game_object::TriggerEvent`
- `TriggeredAbilityDef` -- from `crate::state::game_object::TriggeredAbilityDef`
- `Effect`, `PlayerTarget`, `EffectAmount` -- from `crate::cards::card_definition`

#### 3c. BlitzSacrificeTrigger resolution (after DashReturnTrigger resolution, line 1049 area)

Follow the evoke sacrifice pattern (resolution.rs:718+) for the sacrifice mechanic, combined with
the dash return pattern (resolution.rs:1016-1049) for the delayed-trigger structure.

```rust
// CR 702.152a: Blitz delayed triggered ability resolves.
//
// "Sacrifice the permanent this spell becomes at the beginning
// of the next end step."
// If the source has left the battlefield by resolution time (CR 400.7),
// the trigger does nothing -- the creature is a new object elsewhere.
StackObjectKind::BlitzSacrificeTrigger { source_object } => {
    let controller = stack_obj.controller;

    // Check if the source is still on the battlefield (CR 400.7).
    let source_info = state
        .objects
        .get(&source_object)
        .filter(|obj| obj.zone == ZoneId::Battlefield)
        .map(|obj| {
            (
                obj.owner,
                obj.controller,
                obj.characteristics
                    .card_types
                    .contains(&CardType::Creature),
                obj.counters.clone(),
            )
        });

    if let Some((owner, pre_death_controller, is_creature, pre_death_counters)) =
        source_info
    {
        // Sacrifice: move to owner's graveyard.
        // Sacrifice bypasses indestructible (CR 701.17a).
        // Replacement effects (e.g., Rest in Peace) still apply.
        let (new_id, _old) =
            state.move_object_to_zone(source_object, ZoneId::Graveyard(owner))?;

        if is_creature {
            events.push(GameEvent::CreatureDied {
                object_id: source_object,
                new_grave_id: new_id,
                controller: pre_death_controller,
                pre_death_counters,
            });
        } else {
            events.push(GameEvent::PermanentDestroyed {
                object_id: source_object,
                new_grave_id: new_id,
            });
        }
    }
    // If not on battlefield, do nothing (CR 400.7 -- creature is a new object).
    // Ruling 2022-04-29: "it will stay where it is"

    events.push(GameEvent::AbilityResolved {
        controller,
        stack_object_id: stack_obj.id,
    });
}
```

#### 3d. Counter handling for BlitzSacrificeTrigger

Add `BlitzSacrificeTrigger` to the counter-ability arm in resolution.rs (near line 3056 where DashReturnTrigger is listed):

```rust
| StackObjectKind::BlitzSacrificeTrigger { .. }
```

### Step 4: End Step Trigger Queuing

**File**: `crates/engine/src/rules/turn_actions.rs`
**Action**: Queue `BlitzSacrifice` triggers for all blitzed permanents at end step, following the dash pattern.
**Pattern**: Follow dash end-step trigger queuing at lines 208-254.
**CR**: 702.152a

Add after the dash trigger queuing block (line 254):

```rust
// CR 702.152a: Queue sacrifice triggers for all blitzed permanents.
// "Sacrifice the permanent this spell becomes at the beginning of the
// next end step."
// Each blitzed permanent has `cast_alt_cost == Some(AltCostKind::Blitz)` set when
// the blitz spell resolves (resolution.rs). At end step, we queue a BlitzSacrificeTrigger.
let blitzed_permanents: Vec<(ObjectId, crate::state::player::PlayerId)> = state
    .objects
    .values()
    .filter(|obj| {
        obj.zone == ZoneId::Battlefield
            && obj.cast_alt_cost == Some(AltCostKind::Blitz)
    })
    .map(|obj| (obj.id, obj.controller))
    .collect();

for (obj_id, controller) in blitzed_permanents {
    state.pending_triggers.push_back(PendingTrigger {
        source: obj_id,
        ability_index: 0, // unused for blitz sacrifice triggers
        controller,
        kind: PendingTriggerKind::BlitzSacrifice,
        triggering_event: None,
        entering_object_id: None,
        targeting_stack_id: None,
        triggering_player: None,
        exalted_attacker_id: None,
        defending_player_id: None,
        madness_exiled_card: None,
        madness_cost: None,
        miracle_revealed_card: None,
        miracle_cost: None,
        modular_counter_count: None,
        evolve_entering_creature: None,
        suspend_card_id: None,
        hideaway_count: None,
        partner_with_name: None,
        ingest_target_player: None,
        flanking_blocker_id: None,
        rampage_n: None,
        provoke_target_creature: None,
        renown_n: None,
        poisonous_n: None,
        poisonous_target_player: None,
        enlist_enlisted_creature: None,
        encore_activator: None,
    });
}
```

### Step 5: Trigger Flush Wiring

**File**: `crates/engine/src/rules/abilities.rs`
**Action**: Add `PendingTriggerKind::BlitzSacrifice` arm to `flush_pending_triggers` match, converting it to `StackObjectKind::BlitzSacrificeTrigger`.
**Pattern**: Follow `PendingTriggerKind::DashReturn` at lines 3631-3636.
**CR**: 702.152a

```rust
PendingTriggerKind::BlitzSacrifice => {
    // CR 702.152a: Blitz delayed sacrifice trigger -- "sacrifice the
    // permanent at the beginning of the next end step."
    StackObjectKind::BlitzSacrificeTrigger {
        source_object: trigger.source,
    }
}
```

### Step 6: Replay Harness

**File**: `crates/engine/src/testing/replay_harness.rs`
**Action**: Add `"cast_spell_blitz"` action type.
**Pattern**: Follow `"cast_spell_dash"` at line 806.
**CR**: 702.152a

```rust
// CR 702.152a: Cast a spell with blitz from the player's hand.
// The blitz cost (an alternative cost) is paid instead of the mana cost.
"cast_spell_blitz" => {
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
        alt_cost: Some(AltCostKind::Blitz),
        escape_exile_cards: vec![],
        retrace_discard_land: None,
        jump_start_discard: None,
    })
}
```

### Step 7: Unit Tests

**File**: `crates/engine/tests/blitz.rs`
**Tests to write**:
**Pattern**: Follow tests in `crates/engine/tests/dash.rs` (lines 1-808).

#### Test 1: `test_blitz_basic_cast_with_blitz_cost`
- CR 702.152a: Cast a creature for its blitz cost.
- After ETB: creature on battlefield with haste, `cast_alt_cost == Some(AltCostKind::Blitz)`.
- Mana consumed: blitz cost, not normal mana cost.
- Mock card: "Blitz Goblin" -- Creature {1}{R} 2/2, Blitz {R}.

#### Test 2: `test_blitz_normal_cast_no_sacrifice_no_draw`
- CR 702.152a (negative): Cast the same creature for normal mana cost.
- No haste from blitz, no sacrifice at end step, no draw on death.
- Advance to end step, verify creature stays on battlefield.

#### Test 3: `test_blitz_sacrifice_at_end_step`
- CR 702.152a: Blitzed creature is sacrificed at beginning of next end step.
- Cast with blitz at PostCombatMain, advance to end step, pass priority.
- BlitzSacrificeTrigger resolves -> creature moved to graveyard.
- Verify creature NOT on battlefield, IS in graveyard.

#### Test 4: `test_blitz_draw_card_on_death`
- CR 702.152a: When blitzed creature dies (for any reason), controller draws a card.
- Cast with blitz, creature enters battlefield, manually move to graveyard.
- Emit CreatureDied event manually (or use a helper that triggers check_triggers).
- Verify controller's hand size increases by 1 after the draw trigger resolves.
- Ensure library has at least 1 card for the draw.

#### Test 5: `test_blitz_draw_on_sacrifice_at_end_step`
- CR 702.152a: Combined test -- blitz sacrifice at end step triggers the draw.
- Cast with blitz at PostCombatMain, advance to end step.
- Sacrifice trigger resolves (creature goes to graveyard -> CreatureDied event).
- Draw trigger fires and resolves -> controller draws a card.
- After full resolution: creature in graveyard, controller drew a card.
- Need to drain stack fully (sacrifice trigger + draw trigger = 2 resolutions).

#### Test 6: `test_blitz_creature_left_battlefield_before_end_step`
- Ruling 2022-04-29: sacrifice only if still on battlefield at resolution time.
- Cast with blitz, manually move creature to graveyard before end step.
- At end step, sacrifice trigger fires but finds creature not on battlefield -> does nothing.
- Creature remains in graveyard (not double-moved).

#### Test 7: `test_blitz_card_without_blitz_rejected`
- CR 702.152a: Attempting to cast a non-blitz card with `alt_cost: Some(AltCostKind::Blitz)` is rejected.
- Error message should contain "blitz".

#### Test 8: `test_blitz_alternative_cost_exclusivity`
- CR 118.9a: Blitz cannot be combined with other alternative costs.
- Attempt to cast a blitz card with `alt_cost: Some(AltCostKind::Evoke)` -> rejected (card has no evoke).
- Or build a card with both and try -> rejected.

#### Test 9: `test_blitz_commander_tax_applies`
- CR 118.9d: Commander tax is added on top of blitz cost.
- Commander with blitz cost {R}, cast once before (tax = {2}).
- Attempt with only {R} -> fails. Attempt with {2}{R} -> succeeds.
- Pattern: follow `test_dash_commander_tax_applies` exactly.

### Step 8: Card Definition (later phase)

**Suggested card**: Riveteers Requisitioner -- {1}{R}, Creature -- Lizard Rogue 3/1, Blitz {2}{R}; also has "When this creature dies, create a Treasure token" which tests interaction between blitz draw trigger and a normal SelfDies trigger.
**Card lookup**: use `card-definition-author` agent.

### Step 9: Game Script (later phase)

**Suggested scenario**: Cast Riveteers Requisitioner with blitz, attack, advance to end step, sacrifice trigger resolves (creature dies -> both the treasure trigger and blitz draw trigger fire), then both resolve.
**Subsystem directory**: `test-data/generated-scripts/stack/` (involves stack/trigger ordering)

## Interactions to Watch

- **SelfDies trigger ordering**: When the blitzed creature dies, BOTH the blitz draw trigger AND any other SelfDies triggers (like Riveteers Requisitioner's treasure creation) fire simultaneously. They go on the stack in the order the controller chooses (APNAP). The engine handles this via the standard triggered ability ordering.
- **CreatureDied event path**: The draw trigger is a standard `SelfDies` triggered ability added to the creature's `triggered_abilities` at resolution time. It fires from ALL `CreatureDied` event paths (SBA, sacrifice, destroy effect) without any special wiring -- the existing `check_triggers` handler for `CreatureDied` iterates all `SelfDies` triggers on the graveyard object.
- **Replacement effects on sacrifice**: If Rest in Peace is on the battlefield, the sacrificed creature is exiled instead of going to the graveyard. The "when this dies" draw trigger does NOT fire because "dies" means "put into a graveyard from the battlefield" (CR 700.4). The replacement to exile means it never "died."
- **Indestructible**: Sacrifice bypasses indestructible. An indestructible blitzed creature is still sacrificed at end step.
- **Control change**: If another player gains control of a blitzed creature, the sacrifice trigger at end step still fires (it was queued for the original controller). However, `pre_death_controller` capture in resolution matters for whose draw trigger fires -- the draw trigger is on the creature's triggered_abilities, so it fires for the controller at death time (CR 603.3a).
- **Commander**: If a commander is blitzed from the command zone, the sacrifice at end step moves it to graveyard. The commander SBA then allows the owner to redirect it to the command zone. The draw trigger fires because the creature briefly existed in the graveyard (CR 400.7 / tokens gotcha principle applies to commanders too -- they briefly enter the graveyard).
- **Stifle**: If the sacrifice trigger is countered (Stifle), the creature stays on battlefield with haste and the draw-on-death trigger intact. The haste and draw trigger are static abilities (from `cast_alt_cost == Some(AltCostKind::Blitz)`), not tied to the sacrifice trigger.

## Files Modified (Summary)

| File | Changes |
|------|---------|
| `crates/engine/src/state/types.rs` | `AltCostKind::Blitz`, `KeywordAbility::Blitz` |
| `crates/engine/src/state/stack.rs` | `StackObject.was_blitzed`, `StackObjectKind::BlitzSacrificeTrigger` |
| `crates/engine/src/state/stubs.rs` | `PendingTriggerKind::BlitzSacrifice` |
| `crates/engine/src/state/hash.rs` | Hash discriminants for all new types (KW 96, AbDef 29, SOK 32) |
| `crates/engine/src/cards/card_definition.rs` | `AbilityDefinition::Blitz { cost }` |
| `crates/engine/src/rules/casting.rs` | Blitz casting validation + cost lookup + mutual exclusion |
| `crates/engine/src/rules/resolution.rs` | ETB haste + draw trigger + sacrifice trigger resolution |
| `crates/engine/src/rules/turn_actions.rs` | End-step sacrifice trigger queuing |
| `crates/engine/src/rules/abilities.rs` | `BlitzSacrifice` flush wiring + `was_blitzed: false` at all sites |
| `crates/engine/src/rules/copy.rs` | `was_blitzed: false` at all copy sites |
| `crates/engine/src/testing/replay_harness.rs` | `"cast_spell_blitz"` action type |
| `tools/tui/src/play/panels/stack_view.rs` | `BlitzSacrificeTrigger` match arm |
| `tools/replay-viewer/src/view_model.rs` | `BlitzSacrificeTrigger` match arm |
| `crates/engine/tests/blitz.rs` | 9 unit tests |

## Discriminant Summary

| Type | Variant | Discriminant |
|------|---------|-------------|
| `KeywordAbility` | `Blitz` | 96 |
| `AbilityDefinition` | `Blitz { cost }` | 29 |
| `StackObjectKind` | `BlitzSacrificeTrigger` | 32 |
