# Ability Plan: Scavenge

**Generated**: 2026-03-06
**CR**: 702.97
**Priority**: P4
**Similar abilities studied**: Embalm (CR 702.128) in `abilities.rs:1348-1520`, `resolution.rs:3394-3475`, `command.rs:503-513`, `stack.rs:726-738`

## CR Rule Text

702.97. Scavenge

702.97a Scavenge is an activated ability that functions only while the card with scavenge is in a graveyard. "Scavenge [cost]" means "[Cost], Exile this card from your graveyard: Put a number of +1/+1 counters equal to the power of the card you exiled on target creature. Activate only as a sorcery."

## Key Edge Cases

- **Exile is part of the cost** (ruling 2013-04-15): "Once the ability is activated and the cost is paid, it's too late to stop the ability by trying to remove the card from the graveyard." The card is exiled immediately when the ability is activated, before it goes on the stack.
- **Power is captured at activation time** (Varolz ruling 2013-04-15): "The number of counters that a card's scavenge ability puts on a creature is based on the card's power as it last existed in the graveyard." Must snapshot power before exiling.
- **Targets a creature**: Unlike Embalm (no targets), Scavenge targets a creature on the battlefield. The ability fizzles if the target creature is no longer legal at resolution.
- **Sorcery speed** (CR 702.97a): "Activate only as a sorcery" -- main phase, empty stack, active player only.
- **Multiple instances** (Varolz ruling): A card with multiple scavenge abilities can activate either, but not both (exiled by one activation removes it from graveyard). Varolz grants scavenge to ALL creature cards in your graveyard.
- **X in mana cost** (Varolz ruling): "If the creature card you scavenge has {X} in its mana cost, X is 0." (Applies to Varolz's granted scavenge cost, not to scavenge's own cost field.)
- **Not a spell cast**: Scavenge is an activated ability, not a spell. No "cast" triggers fire.

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
**Action**: Add `KeywordAbility::Scavenge` variant after `Graft(u32)` at line ~1105.
**Discriminant**: KW 120.
**Pattern**: Follow `KeywordAbility::Embalm` (discriminant 92).

```
/// CR 702.97: Scavenge [cost] -- activated ability from graveyard.
/// "[Cost], Exile this card from your graveyard: Put a number of +1/+1
/// counters equal to the power of the card you exiled on target creature.
/// Activate only as a sorcery."
///
/// Discriminant 120.
Scavenge,
```

**File**: `crates/engine/src/state/hash.rs`
**Action**: Add hash arm for `KeywordAbility::Scavenge` in the `KeywordAbility` `HashInto` impl, after `Graft(n)` (around line ~604). Use `120u8`.

### Step 2: AbilityDefinition Variant

**File**: `crates/engine/src/cards/card_definition.rs`
**Action**: Add `AbilityDefinition::Scavenge { cost: ManaCost }` variant after the last ability definition.
**Discriminant**: AbilDef 47.
**Pattern**: Follow `AbilityDefinition::Embalm { cost: ManaCost }` at line ~287.

```
/// CR 702.97: Scavenge [cost]. The card's scavenge ability can be activated
/// from its owner's graveyard by paying this cost plus exiling the card. When
/// the ability resolves, put +1/+1 counters equal to the card's power on
/// target creature.
///
/// Cards with this ability should also include
/// `AbilityDefinition::Keyword(KeywordAbility::Scavenge)` for quick
/// presence-checking without scanning all abilities. Discriminant 47.
Scavenge { cost: ManaCost },
```

**File**: `crates/engine/src/state/hash.rs`
**Action**: Add hash arm for `AbilityDefinition::Scavenge { cost }` in the `AbilityDefinition` `HashInto` impl. Use `47u8`.

### Step 3: StackObjectKind Variant

**File**: `crates/engine/src/state/stack.rs`
**Action**: Add `StackObjectKind::ScavengeAbility` variant after `GraftTrigger`. Fields:
- `source_card_id: Option<CardId>` -- registry key for the exiled card (for display/logging)
- `power_snapshot: u32` -- power of the card as it last existed in the graveyard (captured at activation time, CR 702.97a + Varolz ruling)

**Discriminant**: SOK 45.

```
/// CR 702.97a: Scavenge activated ability on the stack.
///
/// When this ability resolves: put `power_snapshot` +1/+1 counters on the
/// target creature. The card was already exiled as cost; `power_snapshot`
/// is the card's power as it last existed in the graveyard (Varolz ruling
/// 2013-04-15). The target creature is stored in the StackObject's `targets`
/// field.
ScavengeAbility {
    source_card_id: Option<crate::state::player::CardId>,
    power_snapshot: u32,
},
```

**File**: `crates/engine/src/state/hash.rs`
**Action**: Add hash arm for `StackObjectKind::ScavengeAbility` in the `StackObjectKind` `HashInto` impl. Use `45u8`. Hash `source_card_id` and `power_snapshot`.

### Step 4: Command Variant

**File**: `crates/engine/src/rules/command.rs`
**Action**: Add `Command::ScavengeCard` variant near the `EmbalmCard` variant (around line ~513).

```
// -- Scavenge (CR 702.97) -----------------------------------------------
/// Activate a card's scavenge ability from the graveyard (CR 702.97a).
///
/// The card must be in the player's graveyard with `KeywordAbility::Scavenge`.
/// The card is exiled as part of the activation cost. The ability is placed
/// on the stack targeting `target_creature`. When it resolves, +1/+1 counters
/// equal to the card's power (as it last existed in the graveyard) are placed
/// on the target creature.
///
/// "Activate only as a sorcery" -- main phase, stack empty, active player.
ScavengeCard {
    player: PlayerId,
    card: ObjectId,
    target_creature: ObjectId,
},
```

### Step 5: Command Handler (Rule Enforcement)

**File**: `crates/engine/src/rules/engine.rs`
**Action**: Add `Command::ScavengeCard` arm in the main match, after the `EmbalmCard` arm (around line ~399).
**Pattern**: Follow the `EmbalmCard` handler pattern exactly.

```
// -- Scavenge (CR 702.97) -------------------------------------------------
Command::ScavengeCard { player, card, target_creature } => {
    validate_player_active(&state, player)?;
    loop_detection::reset_loop_detection(&mut state);
    let mut events = abilities::handle_scavenge_card(&mut state, player, card, target_creature)?;
    let new_triggers = abilities::check_triggers(&state, &events);
    for t in new_triggers {
        state.pending_triggers.push_back(t);
    }
    let trigger_events = abilities::flush_pending_triggers(&mut state);
    events.extend(trigger_events);
    all_events.extend(events);
}
```

### Step 6: Handler Function

**File**: `crates/engine/src/rules/abilities.rs`
**Action**: Add `pub fn handle_scavenge_card()` and helper `fn get_scavenge_cost()` after the Embalm section (after line ~1540).
**Pattern**: Follow `handle_embalm_card()` at line 1348 with these differences:
1. Accept `target_creature: ObjectId` parameter
2. Validate target is a creature on the battlefield (before paying costs)
3. **Capture power BEFORE exile** -- read the card's power from layer-resolved characteristics while it's still in the graveyard. Use `calculate_characteristics()` (not card registry) per the parameterized-keyword gotcha.
4. Pass `power_snapshot` and targets to the `StackObject`
5. Push `ScavengeAbility` as the `StackObjectKind`
6. Set `targets: vec![SpellTarget { target: Target::Object(target_creature), zone_at_cast: ZoneId::Battlefield }]`

Validation steps (in order):
1. Priority check (CR 602.2)
2. Split second check (CR 702.61a)
3. Zone check (CR 702.97a): card must be in player's own graveyard
4. Keyword check (CR 702.97a): card must have `KeywordAbility::Scavenge`
5. Sorcery speed check: active player, main phase, empty stack
6. Target validation: `target_creature` must be a creature on the battlefield
7. Look up scavenge cost from `AbilityDefinition::Scavenge { cost }`
8. Pay mana cost
9. **Snapshot power** from the card (use `calculate_characteristics()` for the graveyard object, read `.power.unwrap_or(0)`)
10. Exile card from graveyard as cost (CR 702.97a)
11. Push `ScavengeAbility { source_card_id, power_snapshot }` onto the stack with target

Helper function `get_scavenge_cost()`:
```
fn get_scavenge_cost(
    card_id: &Option<CardId>,
    registry: &CardRegistry,
) -> Option<ManaCost> {
    card_id.as_ref().and_then(|cid| {
        registry.get(cid.clone()).and_then(|def| {
            def.abilities.iter().find_map(|a| match a {
                AbilityDefinition::Scavenge { cost } => Some(cost.clone()),
                _ => None,
            })
        })
    })
}
```

### Step 7: Resolution

**File**: `crates/engine/src/rules/resolution.rs`
**Action**: Add `StackObjectKind::ScavengeAbility` arm in the main resolution match.
**CR**: 702.97a -- "Put a number of +1/+1 counters equal to the power of the card you exiled on target creature."

Resolution logic:
1. Extract `power_snapshot` from the `ScavengeAbility` variant
2. Read the target creature from `stack_obj.targets[0]`
3. **Fizzle check**: verify target creature is still on the battlefield and is still a creature (if not, the ability fizzles -- remove from stack, done)
4. Add `power_snapshot` +1/+1 counters to the target creature using the same counter-addition pattern as Riot (resolution.rs line ~576-592)
5. Emit `GameEvent::CounterAdded { object_id, counter: CounterType::PlusOnePlusOne, count: power_snapshot }`
6. Emit `GameEvent::AbilityResolved { controller, stack_object_id }`

Also add `StackObjectKind::ScavengeAbility { .. }` to:
- The counter-spell catch-all arm (resolution.rs line ~4203-4220) -- abilities are non-standard to counter; just remove from stack
- The `is_ability` check if one exists

### Step 8: TUI Stack View

**File**: `tools/tui/src/play/panels/stack_view.rs`
**Action**: Add arm for `StackObjectKind::ScavengeAbility { .. }` in the match at line ~107.
**Pattern**: Follow `EmbalmAbility` (no source_object since card was exiled as cost).

```
StackObjectKind::ScavengeAbility { .. } => {
    ("Scavenge: ".to_string(), None)
}
```

### Step 9: Replay Viewer View Model

**File**: `tools/replay-viewer/src/view_model.rs`
**Action**: Add arm for `StackObjectKind::ScavengeAbility { .. }` in the `stack_object_kind_to_label` function (around line ~500).

```
StackObjectKind::ScavengeAbility { .. } => {
    ("scavenge_ability", None)
}
```

### Step 10: Replay Harness Action

**File**: `crates/engine/src/testing/replay_harness.rs`
**Action**: Add `"scavenge_card"` action type in `translate_player_action()`.
**Pattern**: Follow `"embalm_card"` at line ~608. Difference: also extract `target_creature` from the action JSON.

```
"scavenge_card" => {
    let card_id = find_in_graveyard(state, player, card_name?)?;
    let target_name = action.get("target_creature")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("scavenge_card: missing target_creature"))?;
    let target_id = find_on_battlefield(state, target_name)?;
    Some(Command::ScavengeCard {
        player,
        card: card_id,
        target_creature: target_id,
    })
}
```

Note: `find_on_battlefield` may need to be implemented or reused from existing helpers. Check if a general "find creature on battlefield by name" helper exists; if not, use the same pattern as modular trigger target resolution.

### Step 11: Unit Tests

**File**: `crates/engine/tests/scavenge.rs` (new file)
**Tests to write**:

1. `test_scavenge_basic_adds_counters` -- CR 702.97a basic case. Put Deadbridge Goliath (5/5) in graveyard, scavenge it targeting a 2/2 creature. Assert target has 5 +1/+1 counters, scavenge source is in exile.

2. `test_scavenge_card_exiled_as_cost` -- Ruling 2013-04-15. After activating scavenge, the card is immediately in exile (not graveyard). The ability is on the stack. Opponents cannot remove the card to prevent scavenge.

3. `test_scavenge_sorcery_speed_restriction` -- CR 702.97a "activate only as a sorcery". Verify: (a) non-active player gets error, (b) non-main-phase gets error, (c) non-empty stack gets error.

4. `test_scavenge_requires_keyword` -- Card in graveyard without Scavenge keyword returns error.

5. `test_scavenge_requires_graveyard` -- Card on battlefield or in hand returns error.

6. `test_scavenge_fizzles_if_target_leaves` -- Ability on stack, target creature leaves battlefield before resolution. Ability resolves but does nothing (fizzle).

7. `test_scavenge_zero_power` -- Scavenge a 0/X creature (e.g., Slitherhead with 1 power, but test with a hypothetical 0-power creature). Assert 0 counters added (or 1 for Slitherhead). Use a custom ObjectSpec with power 0 to verify 0 counters.

8. `test_scavenge_not_a_cast` -- Scavenge is an activated ability, not a spell. `spells_cast_this_turn` unchanged; no `SpellCast` event.

9. `test_scavenge_requires_mana_payment` -- Attempting scavenge without sufficient mana returns `InsufficientMana` error.

10. `test_scavenge_multiplayer_only_active_player` -- In 4-player game, non-active player cannot scavenge.

**Pattern**: Follow tests in `crates/engine/tests/embalm.rs`. Use `GameStateBuilder::four_player()`, add card to graveyard via ObjectSpec, add target creature to battlefield, set mana pool, execute `Command::ScavengeCard`, then `pass_all_four` to resolve.

### Step 12: Card Definition (later phase)

**Suggested card**: Deadbridge Goliath -- 5/5 creature with Scavenge {4}{G}{G}. Simple stats, prominent scavenge cost, good for testing.
**Alternative**: Dreg Mangler -- 3/3 Haste with Scavenge {3}{B}{G}. Also good; tests haste interaction.
**Card lookup**: use `card-definition-author` agent.

### Step 13: Game Script (later phase)

**Suggested scenario**: Deadbridge Goliath dies in combat, then scavenged onto another creature. Verify +1/+1 counter count, exile status.
**Subsystem directory**: `test-data/generated-scripts/baseline/` or a new `graveyard/` directory.

## Interactions to Watch

- **Fizzle check**: If the target creature leaves the battlefield after scavenge is activated but before it resolves, the ability does nothing (standard fizzle rules, CR 608.2b).
- **Power modification in graveyard**: If an effect modifies the card's power while in the graveyard (rare but possible with continuous effects), `calculate_characteristics()` will capture the modified value. This is correct per CR -- power is checked as it "last existed" in the graveyard.
- **Stifle/counter interaction**: Scavenge is an activated ability on the stack. It can be countered by Stifle. The card is already exiled (cost was paid), so countering the ability means no counters are placed but the card stays in exile.
- **Doubling Season / Branching Evolution**: These modify "counters would be placed" events. If implemented as replacement effects on counter placement, they should interact correctly with scavenge's counter placement at resolution time. The engine's counter-addition code in resolution.rs may need to route through replacement effects.
- **Multiplayer**: No special multiplayer considerations beyond standard APNAP ordering and sorcery-speed restriction (active player only).
