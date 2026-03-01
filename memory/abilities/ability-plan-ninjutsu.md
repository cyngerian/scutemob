# Ability Plan: Ninjutsu

**Generated**: 2026-03-01
**CR**: 702.49
**Priority**: P4
**Batch**: 3 (Combat Modifiers & Ninjutsu)
**Effort**: Medium/Complex
**Similar abilities studied**: Unearth (CR 702.84, `abilities.rs:626-786`, `resolution.rs:864-943`), Myriad token entry-as-attacking (`resolution.rs:1256-1264`)

## CR Rule Text

Full text from MCP lookup:

```
702.49. Ninjutsu

702.49a Ninjutsu is an activated ability that functions only while the card
with ninjutsu is in a player's hand. "Ninjutsu [cost]" means "[Cost], Reveal
this card from your hand, Return an unblocked attacking creature you control
to its owner's hand: Put this card onto the battlefield from your hand tapped
and attacking."

702.49b The card with ninjutsu remains revealed from the time the ability is
announced until the ability leaves the stack.

702.49c A ninjutsu ability may be activated only while a creature on the
battlefield is unblocked (see rule 509.1h). The creature with ninjutsu is put
onto the battlefield unblocked. It will be attacking the same player,
planeswalker, or battle as the creature that was returned to its owner's hand.

702.49d Commander ninjutsu is a variant of the ninjutsu ability that also
functions while the card with commander ninjutsu is in the command zone.
"Commander ninjutsu [cost]" means "[Cost], Reveal this card from your hand or
from the command zone, Return an unblocked attacking creature you control to
its owner's hand: Put this card onto the battlefield tapped and attacking."
```

Supporting rules:

```
509.1h An attacking creature with one or more creatures declared as blockers
for it becomes a blocked creature; one with no creatures declared as blockers
for it becomes an unblocked creature. This remains unchanged until the creature
is removed from combat, an effect says that it becomes blocked or unblocked,
or the combat phase ends, whichever comes first. A creature remains blocked
even if all the creatures blocking it are removed from combat.

508.3a An ability that reads "Whenever [a creature] attacks, . . ." triggers
if that creature is declared as an attacker. Similarly, "Whenever [a creature]
attacks [a player, planeswalker, or battle], . . ." triggers if that creature
is declared as an attacker attacking that player or permanent. Such abilities
won't trigger if a creature is put onto the battlefield attacking.

508.4 If a creature is put onto the battlefield attacking, its controller
chooses which defending player, planeswalker a defending player controls, or
battle a defending player protects it's attacking as it enters the battlefield
(unless the effect that put it onto the battlefield specifies what it's
attacking). [...] Such creatures are "attacking" but, for the purposes of
trigger events and effects, they never "attacked."

508.4a If the effect that puts a creature onto the battlefield attacking
specifies it's attacking a certain player, and that player is no longer in the
game when the effect resolves, the creature is put onto the battlefield but is
never considered an attacking creature. [Same for planeswalkers no longer on
the battlefield.]

508.4c A creature that's put onto the battlefield attacking or that is stated
to be attacking isn't affected by requirements or restrictions that apply to
the declaration of attackers.

602.2 To activate an ability is to put it onto the stack and pay its costs, so
that it will eventually resolve and have its effect. [...]

602.2a The player announces that they are activating the ability. If an
activated ability is being activated from a hidden zone, the card that has
that ability is revealed (see rule 701.20a). That ability is created on the
stack as an object that's not a card. [...]
```

## Key Edge Cases

1. **Timing window** (CR 702.49c, ruling 2021-03-19): Ninjutsu can be activated
   during the declare blockers step, combat damage step, first-strike combat
   damage step, or end of combat step -- any time after blockers are declared
   and the attacking creature is unblocked. Before declare blockers, creatures
   are neither blocked nor unblocked, so ninjutsu cannot be activated.

2. **Ninja does NOT trigger "when attacks"** (CR 508.3a, 508.4, ruling
   2021-03-19): The ninja is put onto the battlefield attacking, but was never
   declared as an attacker. "Whenever [creature] attacks" triggers do NOT fire.
   This is automatically correct in the engine because `SelfAttacks` triggers
   only fire on `GameEvent::AttackersDeclared` (in `check_triggers` at
   `abilities.rs:1373-1399`), not on `PermanentEnteredBattlefield`.

3. **Ninja attacks the same target** (CR 702.49c): The ninja inherits the
   attack target of the returned creature. This is a ninjutsu-specific rule --
   NOT the general "controller chooses" rule from 508.4.

4. **First strike timing** (ruling 2021-03-19): If an unblocked attacker has
   first strike, you can ninjutsu after first-strike damage is dealt. The ninja
   enters during the regular combat damage step and will deal damage there
   (even if the ninja itself has first strike -- the first-strike damage step
   already happened).

5. **Ability resolves from stack** (ruling 2021-03-19): The ninja card is NOT
   moved to the battlefield as a cost. It stays in hand until the ability
   resolves. If it leaves hand before resolution (discarded, etc.), the ability
   resolves and does nothing. The attacker IS returned as part of the cost
   (immediate).

6. **Commander Ninjutsu bypasses commander tax** (CR 702.49d, ruling
   2020-11-10): "Activating Yuriko's commander ninjutsu ability isn't the same
   as casting Yuriko as a spell. You won't have to pay the commander tax to
   activate that ability, and activating that ability won't increase the
   commander tax to pay later." This is an activated ability, not a spell cast.

7. **Attacker must be unblocked at activation time**: The check is at
   activation, not resolution. If the attacker becomes blocked between
   activation and resolution (via an effect), the ninjutsu still resolves --
   the attacker was already returned to hand as a cost.

8. **The returned creature goes to its OWNER's hand** (CR 702.49a): Not the
   controller's hand. In multiplayer, if you control an opponent's creature via
   theft, ninjutsu returns it to the opponent's hand.

9. **Attack target invalid at resolution** (CR 508.4a): If the target player
   has left the game when the ability resolves, the creature enters the
   battlefield but is never considered attacking. Same for planeswalkers that
   left the battlefield. The ninja still enters (tapped) but is not registered
   in combat.attackers.

10. **Split second blocks ninjutsu**: Like all activated abilities, ninjutsu
    cannot be activated while a spell with split second is on the stack
    (CR 702.61a).

11. **Multiplayer**: No special multiplayer implications beyond the
    owner-vs-controller distinction. Commander Ninjutsu is significant for
    Commander format -- Yuriko can be put onto the battlefield from the command
    zone, bypassing commander tax entirely.

## Key Card Rulings (from MCP lookup)

**Ninja of the Deep Hours** (6 rulings, 2021-03-19):
- The ninjutsu ability can be activated during the declare blockers step,
  combat damage step, or end of combat step.
- Although the Ninja is attacking, it was never declared as an attacking
  creature (for purposes of abilities that trigger whenever a creature attacks).
- The creature enters attacking the same player or planeswalker that the
  returned creature was attacking. This is a rule specific to ninjutsu.
- As you activate a ninjutsu ability, you reveal the Ninja card in your hand
  and return the attacking creature. The Ninja isn't put onto the battlefield
  until the ability resolves. If it leaves your hand before then, it won't
  enter the battlefield at all.
- If a creature in combat has first strike or double strike, you can activate
  the ninjutsu ability during the first-strike combat damage step.
- The ninjutsu ability can be activated only after blockers have been declared.

**Yuriko, the Tiger's Shadow** (4 rulings, 2020-11-10):
- Activating Yuriko's commander ninjutsu ability isn't the same as casting
  Yuriko as a spell. No commander tax paid or incremented.
- Commander ninjutsu is a variant that can be activated from the command zone
  as well as from your hand.

## Current State (from ability-wip.md)

No steps completed. The ability-wip.md currently tracks Training (closed).
Ninjutsu is entirely unimplemented -- no enum variant, no command, no handler,
no tests.

- [ ] Step 1: Enum variant + AbilityDefinition
- [ ] Step 2: Command variant
- [ ] Step 3: Command handler (activation)
- [ ] Step 4: Stack object kind + resolution
- [ ] Step 5: Match arm updates (counter, view_model, TUI, hash)
- [ ] Step 6: Replay harness action type
- [ ] Step 7: Unit tests
- [ ] Step 8: Card definition
- [ ] Step 9: Game script

## Implementation Steps

### Step 1: Enum Variant + AbilityDefinition

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/types.rs`
**Action**: Add `KeywordAbility::Ninjutsu` and `KeywordAbility::CommanderNinjutsu`
variants after `Training` (line ~700, before the closing `}` of the enum).
```rust
/// CR 702.49: Ninjutsu -- activated ability from hand. Pay cost, return an
/// unblocked attacking creature you control to its owner's hand, put this
/// card onto the battlefield tapped and attacking the same target.
///
/// Marker for quick presence-checking (`keywords.contains`).
/// The ninjutsu cost is stored in `AbilityDefinition::Ninjutsu { cost }`.
Ninjutsu,
/// CR 702.49d: Commander Ninjutsu -- variant that also works from the
/// command zone. Bypasses commander tax entirely (it is an activated
/// ability, not a spell cast).
CommanderNinjutsu,
```

**File**: `/home/airbaggie/scutemob/crates/engine/src/cards/card_definition.rs`
**Action**: Add `AbilityDefinition::Ninjutsu` and `AbilityDefinition::CommanderNinjutsu`
variants after `Overload` (line ~263, before closing `}` of the enum).
```rust
/// CR 702.49: Ninjutsu [cost]. Activated from hand: pay cost, return an
/// unblocked attacker to its owner's hand, put this card onto battlefield
/// tapped and attacking the same target.
///
/// Cards with this ability should also include
/// `AbilityDefinition::Keyword(KeywordAbility::Ninjutsu)` for quick
/// presence-checking without scanning all abilities.
Ninjutsu { cost: ManaCost },
/// CR 702.49d: Commander Ninjutsu [cost]. Same as ninjutsu but can also
/// be activated from the command zone. Bypasses commander tax entirely.
///
/// Cards with this ability should also include
/// `AbilityDefinition::Keyword(KeywordAbility::CommanderNinjutsu)` for
/// quick presence-checking.
CommanderNinjutsu { cost: ManaCost },
```
**Pattern**: Follow `AbilityDefinition::Overload { cost: ManaCost }` at line ~263.

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/hash.rs`
**Action 1**: Add discriminants for `KeywordAbility::Ninjutsu` (83) and
`KeywordAbility::CommanderNinjutsu` (84) in the `HashInto` impl for
`KeywordAbility` (after Training discriminant 82, line ~494).
```rust
// Ninjutsu (discriminant 83) -- CR 702.49
KeywordAbility::Ninjutsu => 83u8.hash_into(hasher),
// Commander Ninjutsu (discriminant 84) -- CR 702.49d
KeywordAbility::CommanderNinjutsu => 84u8.hash_into(hasher),
```

**Action 2**: Add discriminants for `AbilityDefinition::Ninjutsu` (22) and
`AbilityDefinition::CommanderNinjutsu` (23) in the `HashInto` impl for
`AbilityDefinition` (after Overload discriminant 21, line ~2994).
```rust
// Ninjutsu (discriminant 22) -- CR 702.49
AbilityDefinition::Ninjutsu { cost } => {
    22u8.hash_into(hasher);
    cost.hash_into(hasher);
}
// Commander Ninjutsu (discriminant 23) -- CR 702.49d
AbilityDefinition::CommanderNinjutsu { cost } => {
    23u8.hash_into(hasher);
    cost.hash_into(hasher);
}
```

**File**: `/home/airbaggie/scutemob/tools/replay-viewer/src/view_model.rs`
**Action**: Add match arms in `keyword_to_string()` function (after Training,
line ~706).
```rust
KeywordAbility::Ninjutsu => "Ninjutsu".to_string(),
KeywordAbility::CommanderNinjutsu => "Commander Ninjutsu".to_string(),
```

### Step 2: Command Variant

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/command.rs`
**Action**: Add `Command::ActivateNinjutsu` variant after `UnearthCard`
(line ~381, before the closing `}` of the enum).
```rust
// -- Ninjutsu (CR 702.49) -----------------------------------------------
/// Activate a card's ninjutsu ability from hand (or command zone for
/// commander ninjutsu).
///
/// CR 702.49a: The player pays the ninjutsu cost, returns an unblocked
/// attacking creature they control to its owner's hand, and the ninjutsu
/// card is put onto the battlefield tapped and attacking the same target.
///
/// This is an activated ability, NOT a spell cast. No "cast" triggers fire.
/// Commander ninjutsu (CR 702.49d) bypasses commander tax entirely.
///
/// The ability goes on the stack. The attacker is returned to hand as a
/// cost (immediately). The ninja enters the battlefield when the ability
/// resolves.
ActivateNinjutsu {
    player: PlayerId,
    /// The card with ninjutsu in the player's hand (or command zone).
    ninja_card: ObjectId,
    /// The unblocked attacking creature to return to its owner's hand.
    attacker_to_return: ObjectId,
},
```

### Step 3: Command Handler (Activation)

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/abilities.rs`
**Action**: Add `handle_ninjutsu` function and `get_ninjutsu_cost` helper.
Pattern follows `handle_unearth_card` (line 626-765) but with different zone
checks, no sorcery restriction, and additional attacker validation.

Function signature:
```rust
/// Handle an ActivateNinjutsu command: validate, pay cost, return attacker
/// to hand, push ninjutsu ability onto stack.
///
/// CR 702.49a: Ninjutsu is an activated ability from hand.
/// CR 702.49c: May only be activated when an unblocked attacker exists.
/// CR 702.49d: Commander ninjutsu also functions from the command zone.
pub fn handle_ninjutsu(
    state: &mut GameState,
    player: PlayerId,
    ninja_card: ObjectId,
    attacker_to_return: ObjectId,
) -> Result<Vec<GameEvent>, GameStateError>
```

Validation checks (in order):

1. **Priority check** (CR 602.2): `state.turn.priority_holder == Some(player)`
2. **Split second check** (CR 702.61a): `casting::has_split_second_on_stack(state)`
3. **Combat phase + step check** (CR 702.49c): Must be in combat phase at
   `DeclareBlockers`, `FirstStrikeDamage`, `CombatDamage`, or `EndOfCombat`.
   NOT `DeclareAttackers` or `BeginningOfCombat` (before blockers are declared,
   creatures are neither blocked nor unblocked).
   ```rust
   use crate::state::turn::Step;
   let step = state.turn.step;
   let valid_step = matches!(
       step,
       Step::DeclareBlockers
           | Step::FirstStrikeDamage
           | Step::CombatDamage
           | Step::EndOfCombat
   );
   ```
4. **Combat state exists**: `state.combat.is_some()` -- safety check.
5. **Zone check** (CR 702.49a/d): Ninja card must be in player's hand
   (`ZoneId::Hand(player)`) OR, if it has `KeywordAbility::CommanderNinjutsu`,
   in the command zone (`ZoneId::Command(player)`).
   **CRITICAL**: The zone is `ZoneId::Command(player)`, NOT `CommandZone`.
6. **Keyword check**: Card must have `KeywordAbility::Ninjutsu` or
   `KeywordAbility::CommanderNinjutsu`.
7. **Attacker validation**: `attacker_to_return` must be:
   - In `state.objects` and on the battlefield (`ZoneId::Battlefield`)
   - Controlled by `player`
   - In `state.combat.as_ref().unwrap().attackers`
   - NOT blocked: `!combat.is_blocked(attacker_to_return)` (uses
     `CombatState::is_blocked()` at `combat.rs:102`)
8. **Capture attack target BEFORE returning the attacker** (CR 702.49c):
   ```rust
   let attack_target = state.combat.as_ref().unwrap()
       .attackers.get(&attacker_to_return).cloned()
       .expect("attacker should be in combat.attackers");
   ```
9. **Cost lookup**: Find `AbilityDefinition::Ninjutsu { cost }` or
   `AbilityDefinition::CommanderNinjutsu { cost }` from card registry.
   Add `get_ninjutsu_cost()` helper following `get_unearth_cost()` at line
   771-785.
10. **Pay mana cost** (CR 602.2b): Same pattern as unearth.
11. **Return attacker to owner's hand** (cost, CR 702.49a): This is part of the
    cost, done immediately.
    - Look up `obj.owner` BEFORE moving (owner, not controller).
    - `state.move_object_to_zone(attacker_to_return, ZoneId::Hand(owner))`
    - Remove attacker from `combat.attackers`:
      `state.combat.as_mut().unwrap().attackers.remove(&attacker_to_return);`
      Note: `move_object_to_zone` creates a new ObjectId; the old one in
      combat.attackers is now stale and MUST be removed.
    - Emit `GameEvent::ObjectReturnedToHand { player: owner, object_id:
      attacker_to_return, new_hand_id }`.
12. **Determine `from_command_zone` flag**: Check if ninja was in
    `ZoneId::Command(player)` (for commander ninjutsu stack tracking).
13. **Push ninjutsu ability onto stack** as `StackObjectKind::NinjutsuAbility`.
    ```rust
    let stack_id = state.next_object_id();
    let stack_obj = StackObject {
        id: stack_id,
        controller: player,
        kind: StackObjectKind::NinjutsuAbility {
            source_object: ninja_card,
            ninja_card,
            attack_target: attack_target.clone(),
            from_command_zone,
        },
        targets: Vec::new(),
        cant_be_countered: false,
        is_copy: false,
        cast_with_flashback: false,
        kicker_times_paid: 0,
        was_evoked: false,
        was_bestowed: false,
        cast_with_madness: false,
        cast_with_miracle: false,
        was_escaped: false,
        cast_with_foretell: false,
        was_buyback_paid: false,
        was_suspended: false,
        was_overloaded: false,
    };
    state.stack_objects.push_back(stack_obj);
    ```
14. **Reset priority** (CR 602.2e): active player gets priority.
    ```rust
    state.turn.players_passed = OrdSet::new();
    let active = state.turn.active_player;
    state.turn.priority_holder = Some(active);
    ```
15. Emit `GameEvent::AbilityActivated` and `GameEvent::PriorityGiven`.

Helper function:
```rust
/// CR 702.49a: Look up the ninjutsu cost from the card's `AbilityDefinition`.
///
/// Returns the `ManaCost` stored in `AbilityDefinition::Ninjutsu { cost }` or
/// `AbilityDefinition::CommanderNinjutsu { cost }`, or `None` if the card has
/// no definition or no ninjutsu ability defined.
fn get_ninjutsu_cost(
    card_id: &Option<CardId>,
    registry: &crate::cards::CardRegistry,
) -> Option<ManaCost> {
    card_id.as_ref().and_then(|cid| {
        registry.get(cid.clone()).and_then(|def| {
            def.abilities.iter().find_map(|a| match a {
                AbilityDefinition::Ninjutsu { cost }
                | AbilityDefinition::CommanderNinjutsu { cost } => Some(cost.clone()),
                _ => None,
            })
        })
    })
}
```

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/engine.rs`
**Action**: Add command dispatch arm after `UnearthCard` (line ~334).
```rust
Command::ActivateNinjutsu {
    player,
    ninja_card,
    attacker_to_return,
} => {
    validate_player_active(&state, player)?;
    // CR 104.4b: ninjutsu is a meaningful player choice; reset loop detection.
    loop_detection::reset_loop_detection(&mut state);
    let mut events = abilities::handle_ninjutsu(
        &mut state, player, ninja_card, attacker_to_return,
    )?;
    let new_triggers = abilities::check_triggers(&state, &events);
    for t in new_triggers {
        state.pending_triggers.push_back(t);
    }
    let trigger_events = abilities::flush_pending_triggers(&mut state);
    events.extend(trigger_events);
    all_events.extend(events);
}
```
**Note**: This goes BEFORE the `}` closing arm of the match, after the last
current command (`UnearthCard`). The trigger flush pattern is mandatory
(see gotchas-infra.md: "Every Command handler that can produce triggers...").

### Step 4: Stack Object Kind + Resolution

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/stack.rs`
**Action**: Add `StackObjectKind::NinjutsuAbility` variant after
`RenownTrigger` (line ~513, before the closing `}` of the enum).
```rust
/// CR 702.49a: Ninjutsu activated ability on the stack.
///
/// When this ability resolves: put the ninja card from hand (or command
/// zone) onto the battlefield tapped and attacking the captured
/// `attack_target`.
///
/// `source_object` / `ninja_card` are the ObjectId of the card in
/// hand/command zone (same value; source_object follows the convention
/// used by other StackObjectKind variants).
/// `attack_target` is the attack target inherited from the returned
/// attacker.
/// `from_command_zone` indicates commander ninjutsu (for zone checks at
/// resolution time).
///
/// If the ninja card is no longer in hand/command zone at resolution
/// time, the ability does nothing (CR 400.7 -- object left the expected
/// zone).
NinjutsuAbility {
    source_object: ObjectId,
    ninja_card: ObjectId,
    attack_target: crate::state::combat::AttackTarget,
    from_command_zone: bool,
},
```

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/resolution.rs`
**Action**: Add resolution arm for `StackObjectKind::NinjutsuAbility` in the
main `resolve_top_of_stack` match (after `UnearthAbility` resolution,
line ~943).

Resolution logic:
```rust
StackObjectKind::NinjutsuAbility {
    source_object: _,
    ninja_card,
    attack_target,
    from_command_zone,
} => {
    let controller = stack_obj.controller;

    // 1. Check if ninja card is still in the expected zone (CR 400.7).
    //    Hand for regular ninjutsu; command zone for commander ninjutsu.
    //    CRITICAL: ZoneId::Command(player), NOT ZoneId::CommandZone.
    let expected_zone = if from_command_zone {
        ZoneId::Command(controller)
    } else {
        ZoneId::Hand(controller)
    };
    let still_in_zone = state
        .objects
        .get(&ninja_card)
        .map(|obj| obj.zone == expected_zone)
        .unwrap_or(false);

    if still_in_zone {
        // 2. Check attack target is still valid (CR 508.4a).
        //    If invalid, creature enters battlefield but is not attacking.
        let target_valid = match &attack_target {
            AttackTarget::Player(pid) => state.player(*pid)
                .map(|p| !p.has_lost && !p.has_conceded)
                .unwrap_or(false),
            AttackTarget::Planeswalker(oid) => state.objects
                .get(oid)
                .map(|o| o.zone == ZoneId::Battlefield)
                .unwrap_or(false),
        };

        let combat_active = state.combat.is_some();

        // 3. Move ninja from hand/command zone to battlefield (CR 702.49a).
        let (new_id, _old) =
            state.move_object_to_zone(ninja_card, ZoneId::Battlefield)?;

        // 4. Set controller, tapped status.
        let card_id = state.objects.get(&new_id).and_then(|o| o.card_id.clone());
        if let Some(obj) = state.objects.get_mut(&new_id) {
            obj.controller = controller;
            obj.status.tapped = true; // CR 702.49a: "tapped and attacking"
        }

        // 5. Register in combat state as attacking the same target.
        //    CR 702.49c: "attacking the same player, planeswalker, or battle"
        //    CR 702.49c: "put onto the battlefield unblocked"
        //    Only if target is valid AND combat is active.
        if target_valid && combat_active {
            if let Some(combat) = state.combat.as_mut() {
                combat.attackers.insert(new_id, attack_target.clone());
            }
        }
        // If target_valid is false (CR 508.4a), ninja enters but is NOT
        // an attacking creature. It is still tapped.

        // 6. Apply self ETB replacements + global ETB replacements.
        //    Follow the full ETB site pattern (gotchas-infra.md).
        let registry = state.card_registry.clone();
        let self_evts = super::replacement::apply_self_etb_from_definition(
            state, new_id, controller, card_id.as_ref(), &registry,
        );
        events.extend(self_evts);
        let etb_evts =
            super::replacement::apply_etb_replacements(state, new_id, controller);
        events.extend(etb_evts);

        // 7. Register replacement abilities and static continuous effects.
        super::replacement::register_permanent_replacement_abilities(
            state, new_id, controller, card_id.as_ref(), &registry,
        );
        super::replacement::register_static_continuous_effects(
            state, new_id, card_id.as_ref(), &registry,
        );

        // 8. Emit PermanentEnteredBattlefield.
        events.push(GameEvent::PermanentEnteredBattlefield {
            player: controller,
            object_id: new_id,
        });

        // 9. Fire WhenEntersBattlefield triggered effects from card def.
        let etb_trigger_evts = super::replacement::fire_when_enters_triggered_effects(
            state, new_id, controller, card_id.as_ref(), &registry,
        );
        events.extend(etb_trigger_evts);
    }
    // If ninja left the expected zone, ability does nothing (CR 400.7).

    events.push(GameEvent::AbilityResolved {
        controller,
        stack_object_id: stack_obj.id,
    });
}
```

**Pattern**: Follows Unearth resolution (line 864-943) for the full ETB site
pattern, and Myriad resolution (line 1256-1264) for the
`combat.attackers.insert()` pattern.

**Critical ETB site pattern** (from gotchas-infra.md):
1. `apply_self_etb_from_definition` (e.g., "enters tapped" -- redundant for
   ninjutsu since already tapped, but must still be called for correctness)
2. `apply_etb_replacements` (e.g., Rest in Peace)
3. `register_permanent_replacement_abilities`
4. `register_static_continuous_effects`
5. Emit `PermanentEnteredBattlefield`
6. `fire_when_enters_triggered_effects` (inline ETBs from card definition)

**Important difference from the Unearth pattern**: The Unearth resolution at
line 928-936 calls `fire_when_enters_triggered_effects`. The original stub plan
omitted this call. It MUST be included so that self-referential ETB triggers
(e.g., if a ninja has an ETB effect in its card definition) fire correctly.

### Step 5: Match Arm Updates

Multiple files have exhaustive matches on `StackObjectKind` and
`KeywordAbility` that must be updated.

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/resolution.rs`
**Action**: Add `NinjutsuAbility` to the `counter_stack_object` match
(line ~1997, within the `| StackObjectKind::RenownTrigger { .. }` chain).
```rust
| StackObjectKind::NinjutsuAbility { .. }
```

**File**: `/home/airbaggie/scutemob/tools/replay-viewer/src/view_model.rs`
**Action 1**: Add match arm in the stack-to-view match for NinjutsuAbility
(after RenownTrigger, line ~487).
```rust
StackObjectKind::NinjutsuAbility { source_object, .. } => {
    ("ninjutsu_ability", Some(*source_object))
}
```
**Action 2**: keyword_to_string arms (covered in Step 1).

**File**: `/home/airbaggie/scutemob/tools/tui/src/play/panels/stack_view.rs`
**Action**: Add match arm (after RenownTrigger, line ~93).
```rust
StackObjectKind::NinjutsuAbility { source_object, .. } => {
    ("Ninjutsu: ".to_string(), Some(*source_object))
}
```

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/hash.rs`
**Action**: Add StackObjectKind hash arm (after RenownTrigger discriminant 22,
line ~1461).
```rust
// NinjutsuAbility (discriminant 23) -- CR 702.49a
StackObjectKind::NinjutsuAbility {
    source_object,
    ninja_card,
    attack_target,
    from_command_zone,
} => {
    23u8.hash_into(hasher);
    source_object.hash_into(hasher);
    ninja_card.hash_into(hasher);
    attack_target.hash_into(hasher);
    from_command_zone.hash_into(hasher);
}
```

**Grep check**: Run to find ALL exhaustive matches that need updating:
```
Grep pattern="StackObjectKind::" path="crates/engine/src/" output_mode="files_with_matches"
Grep pattern="KeywordAbility::" path="crates/engine/src/" output_mode="files_with_matches"
```
Known files with exhaustive matches:
- `rules/resolution.rs` (resolve + counter) -- covered above
- `state/hash.rs` (KeywordAbility + StackObjectKind + AbilityDefinition) -- covered
- `tools/replay-viewer/src/view_model.rs` (keyword_to_string + stack match) -- covered
- `tools/tui/src/play/panels/stack_view.rs` -- covered

### Step 6: Replay Harness Action Type

**File**: `/home/airbaggie/scutemob/crates/engine/src/testing/replay_harness.rs`

**Action 1**: Add `find_in_command_zone` helper function (after
`find_in_graveyard` at line ~1120, following same pattern).
```rust
/// CR 702.49d: Find a named card in a player's command zone (for commander
/// ninjutsu).
fn find_in_command_zone(
    state: &GameState,
    player: PlayerId,
    name: &str,
) -> Option<crate::state::ObjectId> {
    state.objects.iter().find_map(|(&id, obj)| {
        if obj.characteristics.name == name && obj.zone == ZoneId::Command(player) {
            Some(id)
        } else {
            None
        }
    })
}
```

**Action 2**: Add `"activate_ninjutsu"` action type in
`translate_player_action()` (after `"unearth_card"`, line ~505).
```rust
// CR 702.49a: Activate ninjutsu from hand (or command zone for commander
// ninjutsu). `card_name` is the ninja card; `attacker_name` is the
// unblocked attacker to return to hand.
"activate_ninjutsu" => {
    let ninja_name = card_name?;
    let ninja_id = find_in_hand(state, player, ninja_name)
        .or_else(|| {
            // Commander ninjutsu: try command zone
            find_in_command_zone(state, player, ninja_name)
        })?;
    let attacker_name = action.get("attacker_name")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let attacker_id = find_on_battlefield(state, player, attacker_name)?;
    Some(Command::ActivateNinjutsu {
        player,
        ninja_card: ninja_id,
        attacker_to_return: attacker_id,
    })
}
```

**Note**: The `card_name` field names the ninja card. The `attacker_name` field
is a new JSON property on the action object, naming the unblocked attacker.
JSON script actions will look like:
```json
{
    "action_type": "activate_ninjutsu",
    "player": "p1",
    "card_name": "Ninja of the Deep Hours",
    "attacker_name": "Eager Construct",
    "mana_payment": { "blue": 1, "generic": 1 }
}
```

### Step 7: Unit Tests

**File**: `/home/airbaggie/scutemob/crates/engine/tests/ninjutsu.rs`

**Imports needed**:
```rust
use mtg_engine::{
    process_command, AbilityDefinition, AttackTarget, CardDefinition, CardId,
    CardRegistry, CardType, CombatState, Command, GameEvent, GameState,
    GameStateBuilder, KeywordAbility, ManaColor, ManaCost, ObjectId, ObjectSpec,
    PlayerId, Step, TypeLine, ZoneId,
};
```

**Helper functions** (follow `tests/unearth.rs` pattern):
- `p(n)` -- PlayerId constructor
- `find_object(state, name)` -- find by name across all zones
- `find_in_zone(state, name, zone)` -- find in specific zone
- `on_battlefield(state, name)` -- check battlefield
- `in_hand(state, name, owner)` -- check hand
- `pass_all(state, players)` -- pass priority for all players
- `ninja_def()` -- CardDefinition for test ninja (Creature, {2}{U}, 2/2,
  Ninjutsu {U})
- `ninja_in_hand(owner)` -- ObjectSpec in hand with card_id + keyword
- `setup_combat(state, attacker_name, defender)` -- inject CombatState with
  named attacker attacking defender, no blockers

**Tests to write** (12 tests):

1. **`test_ninjutsu_basic_swap`** -- CR 702.49a
   - Set up: P1 has ninja in hand, a 2/2 creature on battlefield (unblocked
     attacker). DeclareBlockers step, CombatState with attacker -> P2.
   - P1 activates ninjutsu: pay {U}, attacker returns to P1's hand.
   - Stack has NinjutsuAbility.
   - Resolve: ninja enters battlefield tapped and attacking P2.
   - Assert: ninja on battlefield, tapped, in combat.attackers targeting P2;
     original attacker in P1's hand; P1 paid mana.

2. **`test_ninjutsu_ninja_attacks_same_target`** -- CR 702.49c
   - 4-player game. Attacker was attacking Player 3 (not P2).
   - After ninjutsu resolves: verify ninja is attacking P3 specifically.

3. **`test_ninjutsu_not_declared_as_attacker`** -- CR 508.3a, 508.4
   - Verify that `AttackersDeclared` event is NOT emitted when ninjutsu
     resolves. This means `SelfAttacks` triggers do not fire for the ninja.
   - Assert: no `AttackersDeclared` event in the resolution events.

4. **`test_ninjutsu_wrong_step_rejected`** -- CR 702.49c
   - Attempt to activate ninjutsu during `DeclareAttackers` step: expect
     `InvalidCommand` error.
   - Also test: `PreCombatMain` step (not combat phase) returns error.
   - Also test: `BeginningOfCombat` step returns error.

5. **`test_ninjutsu_blocked_attacker_rejected`** -- CR 702.49c
   - Set up CombatState with a blocker assigned to the attacker.
   - Attempt to activate ninjutsu: expect `InvalidCommand` error.

6. **`test_ninjutsu_ninja_not_in_hand_rejected`** -- CR 702.49a
   - Ninja is on the battlefield (not in hand).
   - Attempt to activate: expect `InvalidCommand` error.

7. **`test_ninjutsu_ninja_leaves_hand_before_resolution`** -- CR 400.7,
   ruling 2021-03-19
   - Activate ninjutsu (attacker returned as cost, on stack).
   - Manually move ninja out of hand (simulate discard via
     `state.move_object_to_zone(ninja_id, ZoneId::Graveyard(p1))`).
   - Resolve (pass_all): ability does nothing (ninja not in hand).
   - Attacker already gone (returned as cost).

8. **`test_ninjutsu_returns_to_owner_not_controller`** -- CR 702.49a
   - P1 controls a creature owned by P2 (set `obj.controller = p1` and
     `obj.owner = p2` on an object on battlefield).
   - P1 activates ninjutsu returning the stolen creature.
   - Assert: creature goes to P2's hand (owner), not P1's hand.

9. **`test_ninjutsu_combat_damage`** -- verify the ninja deals combat damage
   - Activate ninjutsu during DeclareBlockers step. Resolve.
   - Pass to CombatDamage step.
   - Assert: defending player lost life equal to ninja's power.

10. **`test_ninjutsu_split_second_blocks`** -- CR 702.61a
    - Place a Spell StackObject with `KeywordAbility::SplitSecond` on the
      stack.
    - Attempt to activate ninjutsu: expect `InvalidCommand` error.

11. **`test_ninjutsu_multiplayer_four_player`** -- multiplayer scenario
    - 4-player game. P1 attacks P3 with an unblocked creature.
    - P1 activates ninjutsu: ninja enters attacking P3.
    - Assert: P2, P4 life totals unchanged; P3 is the target.

12. **`test_commander_ninjutsu_from_command_zone`** -- CR 702.49d
    - Card with CommanderNinjutsu in the command zone
      (`ZoneId::Command(p1)`).
    - Activate: ninja enters battlefield tapped and attacking.
    - Assert: `state.player(p1).commander_tax` is NOT incremented (this is
      an activated ability, not a cast).

**Test patterns**:
- Combat state setup: `state.combat = Some({ let mut cs = CombatState::new(p1); cs.attackers.insert(attacker_id, AttackTarget::Player(p2)); cs });`
- Mana: `state.players.get_mut(&p1).unwrap().mana_pool.add(ManaColor::Blue, 1);`
- Priority: `state.turn.priority_holder = Some(p1);`
- Follow `tests/unearth.rs` for structure and `tests/skulk.rs` for combat
  state setup at DeclareBlockers step.

### Step 8: Card Definition (later phase)

**Suggested card**: Ninja of the Deep Hours
- Mana cost: {3}{U}
- Type: Creature -- Human Ninja
- P/T: 2/2
- Ninjutsu {1}{U}
- "Whenever Ninja of the Deep Hours deals combat damage to a player, you may
  draw a card."
- Card lookup: use `card-definition-author` agent

**File**: `/home/airbaggie/scutemob/crates/engine/src/cards/defs/ninja_of_the_deep_hours.rs`

The card uses:
- `AbilityDefinition::Keyword(KeywordAbility::Ninjutsu)` -- presence marker
- `AbilityDefinition::Ninjutsu { cost: ManaCost { blue: 1, generic: 1, .. } }`
  -- actual cost
- `AbilityDefinition::Triggered { trigger_condition: TriggerCondition::SelfDealsCombatDamageToPlayer, effect: Effect::DrawCards { player: EffectTarget::Controller, count: 1 }, ... }`
  -- combat damage trigger (TriggerEvent::SelfDealsCombatDamageToPlayer exists,
  confirmed at `hash.rs:1170`)

### Step 9: Game Script (later phase)

**Suggested scenario**: "Ninja of the Deep Hours swaps in for unblocked
attacker, draws card on combat damage"

**Subsystem directory**: `test-data/generated-scripts/combat/`

Script outline:
1. Initial state: P1 has Ninja of the Deep Hours in hand, a generic 2/2
   creature on battlefield. P1 has {1}{U} mana available. DeclareAttackers step.
2. P1 declares 2/2 attacking P2. All pass to DeclareBlockers.
3. P2 declares no blockers. Priority returns.
4. P1 activates ninjutsu ({1}{U}): return 2/2 to hand, ninjutsu ability on
   stack.
5. All pass: ninjutsu resolves. Ninja enters tapped, attacking P2.
6. All pass to CombatDamage. Ninja deals 2 damage to P2.
7. Combat damage trigger fires: draw a card.
8. Assert: P2 at 38 life, P1 drew 1 card, 2/2 in P1's hand, Ninja on
   battlefield.

**Sequence number**: Use next available in `combat/` directory (check existing
scripts -- currently through 120).

## Interactions to Watch

1. **ETB triggers** (Panharmonicon, etc.): The ninja enters the battlefield,
   so ETB triggers fire normally. "Put onto the battlefield attacking" does NOT
   count as "declared as an attacker" -- the engine correctly fires `SelfAttacks`
   only on `AttackersDeclared` events, not `PermanentEnteredBattlefield`.

2. **Combat damage triggers on the ninja**: The ninja is attacking and will deal
   combat damage in the current combat damage step (if ninjutsu resolved before
   damage). Combat damage triggers (like Ninja of the Deep Hours' "draw a card")
   fire normally.

3. **Removal in response**: After ninjutsu activation (attacker already returned
   to hand), an opponent can respond before the ability resolves. If they
   counter the ability (Stifle), the ninja stays in hand and the attacker was
   already returned. This is a real cost-vs-effect asymmetry.

4. **Zone-change identity (CR 400.7)**: The attacker returned to hand is a new
   object. The ninja entering the battlefield is a new object. All
   auras/equipment on the returned creature fall off. The ninja enters clean.

5. **First strike / double strike combat damage**: If ninjutsu resolves during
   the first-strike damage step (after first-strike damage is dealt), the ninja
   participates in the regular combat damage step. If the ninja itself has first
   strike, it missed the first-strike step and deals no first-strike damage. It
   still deals regular damage.

6. **Commander ninjutsu + commander zone return**: If Yuriko enters via
   commander ninjutsu and later dies, the commander zone-return SBA fires
   normally. The commander_tax counter is NOT incremented because ninjutsu is an
   activated ability, not a cast.

7. **"Enters tapped" ETB replacement**: The ninja already enters tapped per
   ninjutsu. If the ninja also has an "enters tapped" replacement (unlikely but
   possible), it's already tapped -- no conflict.

8. **`move_object_to_zone` for attacker return**: The attacker's old ObjectId
   in `combat.attackers` must be manually removed because `move_object_to_zone`
   does not touch `CombatState`. Without this removal, the stale ObjectId
   remains in the attackers map pointing to a nonexistent object.

## Implementation Order Summary

1. Types: `KeywordAbility::Ninjutsu`, `KeywordAbility::CommanderNinjutsu`,
   `AbilityDefinition::Ninjutsu`, `AbilityDefinition::CommanderNinjutsu`
2. Hash: KeywordAbility discriminants 83+84; AbilityDefinition discriminants
   22+23; StackObjectKind discriminant 23
3. Command: `Command::ActivateNinjutsu`
4. Handler: `abilities::handle_ninjutsu()` + `get_ninjutsu_cost()`
5. Stack: `StackObjectKind::NinjutsuAbility`
6. Resolution: resolve arm in `resolution.rs`
7. Match arms: counter (resolution.rs), view_model, TUI stack_view, hash
8. Harness: `find_in_command_zone()` helper + `"activate_ninjutsu"` action type
9. Tests: 12 tests in `tests/ninjutsu.rs`
10. Card def: Ninja of the Deep Hours
11. Script: combat scenario

## Files Modified (Complete List)

| File | Changes |
|------|---------|
| `crates/engine/src/state/types.rs` | Add `Ninjutsu`, `CommanderNinjutsu` to `KeywordAbility` |
| `crates/engine/src/cards/card_definition.rs` | Add `Ninjutsu { cost }`, `CommanderNinjutsu { cost }` to `AbilityDefinition` |
| `crates/engine/src/state/hash.rs` | Add KeywordAbility discriminants 83+84, AbilityDefinition discriminants 22+23, StackObjectKind discriminant 23 |
| `crates/engine/src/rules/command.rs` | Add `Command::ActivateNinjutsu` |
| `crates/engine/src/rules/engine.rs` | Add dispatch arm for `ActivateNinjutsu` |
| `crates/engine/src/rules/abilities.rs` | Add `handle_ninjutsu()` + `get_ninjutsu_cost()` |
| `crates/engine/src/state/stack.rs` | Add `StackObjectKind::NinjutsuAbility` |
| `crates/engine/src/rules/resolution.rs` | Add resolution arm + counter arm |
| `crates/engine/src/testing/replay_harness.rs` | Add `find_in_command_zone()` + `"activate_ninjutsu"` action type |
| `tools/replay-viewer/src/view_model.rs` | Add keyword_to_string + stack match arms |
| `tools/tui/src/play/panels/stack_view.rs` | Add stack kind match arm |
| `crates/engine/tests/ninjutsu.rs` | New file: 12 tests |
| `crates/engine/src/cards/defs/ninja_of_the_deep_hours.rs` | New card definition (later phase) |

## Corrections from Prior Stub

The prior stub plan had several issues that are corrected in this plan:

1. **Wrong zone enum**: Used `ZoneId::CommandZone(player)` throughout. The
   correct variant is `ZoneId::Command(player)` (verified at
   `crates/engine/src/state/zone.rs:38`).

2. **Missing `fire_when_enters_triggered_effects`**: The resolution arm omitted
   this call, which is part of the standard ETB site pattern (see Unearth at
   line 928-936). Without it, self-referential ETB triggers on the ninja would
   silently never fire.

3. **Missing `AbilityDefinition` hash discriminants**: Only covered
   `KeywordAbility` hashing but not `AbilityDefinition::Ninjutsu` and
   `AbilityDefinition::CommanderNinjutsu` which need discriminants 22 and 23.

4. **Missing `StackObjectKind` hash discriminant**: The hash.rs file has a
   separate exhaustive match for `StackObjectKind` that needs discriminant 23
   for `NinjutsuAbility`.

5. **Attacker removal from combat.attackers**: Not explicitly mentioned in the
   activation handler. When the attacker is returned to hand via
   `move_object_to_zone`, its old ObjectId becomes stale (CR 400.7) but is
   still present in `combat.attackers`. Must be explicitly removed.
