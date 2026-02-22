//! Tests for activated and triggered abilities (CR 602-603).
//!
//! M3-E: ActivateAbility command, triggered ability infrastructure, APNAP ordering,
//! intervening-if clause, and ability resolution from the stack.

use mtg_engine::rules::{process_command, Command, GameEvent};
use mtg_engine::state::player::ManaPool;
use mtg_engine::state::turn::Step;
use mtg_engine::state::zone::ZoneId;
use mtg_engine::state::{
    error::GameStateError, ActivatedAbility, ActivationCost, GameStateBuilder, InterveningIf,
    ManaColor, ManaCost, ObjectSpec, PlayerId, StackObjectKind, TriggerEvent, TriggeredAbilityDef,
};

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

/// Tap-only activated ability (no mana cost).
fn tap_ability(description: &str) -> ActivatedAbility {
    ActivatedAbility {
        cost: ActivationCost {
            requires_tap: true,
            mana_cost: None,
        },
        description: description.to_string(),
        effect: None,
    }
}

/// Tap-and-pay activated ability (e.g., "{T}: draw a card").
fn tap_and_pay_ability(description: &str, mana: ManaCost) -> ActivatedAbility {
    ActivatedAbility {
        cost: ActivationCost {
            requires_tap: true,
            mana_cost: Some(mana),
        },
        description: description.to_string(),
        effect: None,
    }
}

/// Triggered ability: fires when the source permanent enters the battlefield.
fn etb_trigger(description: &str) -> TriggeredAbilityDef {
    TriggeredAbilityDef {
        trigger_on: TriggerEvent::SelfEntersBattlefield,
        intervening_if: None,
        description: description.to_string(),
        effect: None,
    }
}

/// Triggered ability: fires whenever any permanent enters the battlefield.
fn any_etb_trigger(description: &str) -> TriggeredAbilityDef {
    TriggeredAbilityDef {
        trigger_on: TriggerEvent::AnyPermanentEntersBattlefield,
        intervening_if: None,
        description: description.to_string(),
        effect: None,
    }
}

// ---------------------------------------------------------------------------
// Activated ability happy path
// ---------------------------------------------------------------------------

#[test]
/// CR 602.2 — controller activates a tap ability; it goes on the stack
fn test_activate_ability_tap_places_on_stack() {
    let p1 = p(1);
    let creature = ObjectSpec::creature(p1, "Sparkmage", 2, 2)
        .with_activated_ability(tap_ability("{T}: Deal 1 damage (stub)"))
        .in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::four_player()
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(creature)
        .build();

    let source_id = *state
        .zones
        .get(&ZoneId::Battlefield)
        .unwrap()
        .object_ids()
        .first()
        .unwrap();

    let (new_state, events) = process_command(
        state,
        Command::ActivateAbility {
            player: p1,
            source: source_id,
            ability_index: 0,
            targets: vec![],
        },
    )
    .unwrap();

    // Ability is on the stack.
    assert_eq!(new_state.stack_objects.len(), 1);
    let stack_obj = &new_state.stack_objects[0];
    assert!(matches!(
        stack_obj.kind,
        StackObjectKind::ActivatedAbility { .. }
    ));
    assert_eq!(stack_obj.controller, p1);

    // Source was tapped.
    let source = new_state.objects.get(&source_id).unwrap();
    assert!(source.status.tapped, "source permanent should be tapped");

    // Events: PermanentTapped, AbilityActivated, PriorityGiven.
    assert!(events.iter().any(
        |e| matches!(e, GameEvent::PermanentTapped { object_id, .. } if *object_id == source_id)
    ));
    assert!(events.iter().any(|e| matches!(
        e,
        GameEvent::AbilityActivated {
            player,
            source_object_id,
            ..
        } if *player == p1 && *source_object_id == source_id
    )));
    assert!(events
        .iter()
        .any(|e| matches!(e, GameEvent::PriorityGiven { player } if *player == p1)));

    // Active player gets priority; players_passed reset.
    assert_eq!(new_state.turn.priority_holder, Some(p1));
    assert!(new_state.turn.players_passed.is_empty());
}

#[test]
/// CR 602.2b — activating a tap ability taps the source permanently
fn test_activate_ability_tap_cost_taps_source() {
    let p1 = p(1);
    let creature = ObjectSpec::creature(p1, "Prodigal Pyromancer", 2, 2)
        .with_activated_ability(tap_ability("{T}: Deal 1 damage"))
        .in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::four_player()
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(creature)
        .build();

    let source_id = *state
        .zones
        .get(&ZoneId::Battlefield)
        .unwrap()
        .object_ids()
        .first()
        .unwrap();

    let (new_state, _) = process_command(
        state,
        Command::ActivateAbility {
            player: p1,
            source: source_id,
            ability_index: 0,
            targets: vec![],
        },
    )
    .unwrap();

    assert!(new_state.objects.get(&source_id).unwrap().status.tapped);
}

#[test]
/// CR 602.2a — mana cost is paid when activating ability with a mana cost component
fn test_activate_ability_pays_mana_cost() {
    let p1 = p(1);
    let mut mana_cost = ManaCost::default();
    mana_cost.blue = 1; // {U} tap ability
    let creature = ObjectSpec::creature(p1, "Venser", 2, 2)
        .with_activated_ability(tap_and_pay_ability("{U}{T}: Blink target", mana_cost))
        .in_zone(ZoneId::Battlefield);

    // Give player enough blue mana.
    let mut pool = ManaPool::default();
    pool.add(ManaColor::Blue, 2);

    let state = GameStateBuilder::four_player()
        .active_player(p1)
        .add_player_with(p1, |pb| pb.mana(pool))
        .object(creature)
        .at_step(Step::PreCombatMain)
        .build();

    let source_id = *state
        .zones
        .get(&ZoneId::Battlefield)
        .unwrap()
        .object_ids()
        .first()
        .unwrap();

    let (new_state, events) = process_command(
        state,
        Command::ActivateAbility {
            player: p1,
            source: source_id,
            ability_index: 0,
            targets: vec![],
        },
    )
    .unwrap();

    // Mana was spent (1 blue consumed from the 2 we added).
    assert_eq!(
        new_state.players.get(&p1).unwrap().mana_pool.total(),
        1,
        "one blue should remain"
    );
    assert!(events
        .iter()
        .any(|e| matches!(e, GameEvent::ManaCostPaid { .. })));
}

// ---------------------------------------------------------------------------
// Activated ability error cases
// ---------------------------------------------------------------------------

#[test]
/// CR 116 — activating without priority is illegal
fn test_activate_ability_not_priority_holder_fails() {
    let p1 = p(1);
    let p2 = p(2);
    let creature = ObjectSpec::creature(p2, "Sparkmage", 2, 2)
        .with_activated_ability(tap_ability("{T}: Something"))
        .in_zone(ZoneId::Battlefield);

    // p1 has priority; p2 does not.
    let state = GameStateBuilder::four_player()
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(creature)
        .build();

    let source_id = *state
        .zones
        .get(&ZoneId::Battlefield)
        .unwrap()
        .object_ids()
        .first()
        .unwrap();

    let result = process_command(
        state,
        Command::ActivateAbility {
            player: p2,
            source: source_id,
            ability_index: 0,
            targets: vec![],
        },
    );
    assert!(matches!(
        result,
        Err(GameStateError::NotPriorityHolder { .. })
    ));
}

#[test]
/// CR 602.2 — can only activate abilities on permanents you control
fn test_activate_ability_wrong_controller_fails() {
    let p1 = p(1);
    let p2 = p(2);
    // p1 controls the creature, p2 has priority.
    let creature = ObjectSpec::creature(p1, "Sparkmage", 2, 2)
        .with_activated_ability(tap_ability("{T}: Something"))
        .in_zone(ZoneId::Battlefield);

    let mut state = GameStateBuilder::four_player()
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(creature)
        .build();
    // Give priority to p2.
    state.turn.priority_holder = Some(p2);

    let source_id = *state
        .zones
        .get(&ZoneId::Battlefield)
        .unwrap()
        .object_ids()
        .first()
        .unwrap();

    let result = process_command(
        state,
        Command::ActivateAbility {
            player: p2,
            source: source_id,
            ability_index: 0,
            targets: vec![],
        },
    );
    assert!(matches!(result, Err(GameStateError::NotController { .. })));
}

#[test]
/// CR 602.2 — ability index out of range returns an error
fn test_activate_ability_invalid_index_fails() {
    let p1 = p(1);
    let creature = ObjectSpec::creature(p1, "Sparkmage", 2, 2)
        .with_activated_ability(tap_ability("{T}: Something"))
        .in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::four_player()
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(creature)
        .build();

    let source_id = *state
        .zones
        .get(&ZoneId::Battlefield)
        .unwrap()
        .object_ids()
        .first()
        .unwrap();

    let result = process_command(
        state,
        Command::ActivateAbility {
            player: p1,
            source: source_id,
            ability_index: 5, // Only index 0 exists
            targets: vec![],
        },
    );
    assert!(matches!(
        result,
        Err(GameStateError::InvalidAbilityIndex { .. })
    ));
}

#[test]
/// CR 602.2b — activating a tap ability when the source is already tapped fails
fn test_activate_ability_already_tapped_fails() {
    let p1 = p(1);
    let creature = ObjectSpec::creature(p1, "Sparkmage", 2, 2)
        .with_activated_ability(tap_ability("{T}: Something"))
        .tapped()
        .in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::four_player()
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(creature)
        .build();

    let source_id = *state
        .zones
        .get(&ZoneId::Battlefield)
        .unwrap()
        .object_ids()
        .first()
        .unwrap();

    let result = process_command(
        state,
        Command::ActivateAbility {
            player: p1,
            source: source_id,
            ability_index: 0,
            targets: vec![],
        },
    );
    assert!(matches!(
        result,
        Err(GameStateError::PermanentAlreadyTapped(_))
    ));
}

#[test]
/// CR 602.2a — activating with insufficient mana fails
fn test_activate_ability_insufficient_mana_fails() {
    let p1 = p(1);
    let mut mana_cost = ManaCost::default();
    mana_cost.blue = 2; // {U}{U}
    let creature = ObjectSpec::creature(p1, "Venser", 2, 2)
        .with_activated_ability(tap_and_pay_ability("{U}{U}{T}: Something", mana_cost))
        .in_zone(ZoneId::Battlefield);

    // Player only has 1 blue mana.
    let mut pool = ManaPool::default();
    pool.add(ManaColor::Blue, 1);

    let state = GameStateBuilder::four_player()
        .active_player(p1)
        .add_player_with(p1, |pb| pb.mana(pool))
        .object(creature)
        .at_step(Step::PreCombatMain)
        .build();

    let source_id = *state
        .zones
        .get(&ZoneId::Battlefield)
        .unwrap()
        .object_ids()
        .first()
        .unwrap();

    let result = process_command(
        state,
        Command::ActivateAbility {
            player: p1,
            source: source_id,
            ability_index: 0,
            targets: vec![],
        },
    );
    assert!(matches!(result, Err(GameStateError::InsufficientMana)));
}

// ---------------------------------------------------------------------------
// Ability resolution
// ---------------------------------------------------------------------------

#[test]
/// CR 608.3b — activated ability resolves when all players pass priority
fn test_activated_ability_resolves_after_all_pass() {
    let p1 = p(1);
    let p2 = p(2);
    let p3 = p(3);
    let p4 = p(4);

    let creature = ObjectSpec::creature(p1, "Sparkmage", 2, 2)
        .with_activated_ability(tap_ability("{T}: Something"))
        .in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::four_player()
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(creature)
        .build();

    let source_id = *state
        .zones
        .get(&ZoneId::Battlefield)
        .unwrap()
        .object_ids()
        .first()
        .unwrap();

    // Activate the ability.
    let (state, _) = process_command(
        state,
        Command::ActivateAbility {
            player: p1,
            source: source_id,
            ability_index: 0,
            targets: vec![],
        },
    )
    .unwrap();

    assert_eq!(state.stack_objects.len(), 1);

    // All four players pass priority → stack resolves.
    let (state, _) = process_command(state, Command::PassPriority { player: p1 }).unwrap();
    let (state, _) = process_command(state, Command::PassPriority { player: p2 }).unwrap();
    let (state, _) = process_command(state, Command::PassPriority { player: p3 }).unwrap();
    let (state, events) = process_command(state, Command::PassPriority { player: p4 }).unwrap();

    // Stack is empty after resolution.
    assert!(
        state.stack_objects.is_empty(),
        "stack should be empty after ability resolves"
    );

    // AbilityResolved event emitted.
    assert!(events
        .iter()
        .any(|e| matches!(e, GameEvent::AbilityResolved { controller, .. } if *controller == p1)));
}

// ---------------------------------------------------------------------------
// Triggered abilities
// ---------------------------------------------------------------------------

#[test]
/// CR 603.2 — SelfEntersBattlefield triggers when the permanent enters
fn test_triggered_ability_self_etb_fires_on_enter() {
    let p1 = p(1);
    // A creature-spell that will enter the battlefield when resolved.
    let creature_card = ObjectSpec::creature(p1, "Siege-Gang Commander", 2, 2)
        .with_triggered_ability(etb_trigger("When ~ enters, do something"))
        .in_zone(ZoneId::Hand(p1));

    let state = GameStateBuilder::four_player()
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(creature_card)
        .build();

    let card_id = *state
        .zones
        .get(&ZoneId::Hand(p1))
        .unwrap()
        .object_ids()
        .first()
        .unwrap();

    // Cast the creature (no cost for M3-E tests).
    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: card_id,
            targets: vec![],
        },
    )
    .unwrap();

    // All four players pass → creature resolves and enters battlefield.
    let (state, _) = process_command(state, Command::PassPriority { player: p1 }).unwrap();
    let (state, _) = process_command(state, Command::PassPriority { player: p(2) }).unwrap();
    let (state, _) = process_command(state, Command::PassPriority { player: p(3) }).unwrap();
    let (state, _) = process_command(state, Command::PassPriority { player: p(4) }).unwrap();

    // The triggered ability should be on the stack now (flushed during resolution).
    assert_eq!(
        state.stack_objects.len(),
        1,
        "triggered ability should be on the stack"
    );
    assert!(matches!(
        state.stack_objects[0].kind,
        StackObjectKind::TriggeredAbility { .. }
    ));
}

#[test]
/// CR 603.2 — AnyPermanentEntersBattlefield triggers for any permanent entering
fn test_triggered_ability_any_etb_watches_all_permanents() {
    let p1 = p(1);
    let p2 = p(2);

    // Watcher has "whenever any permanent enters the battlefield, do X".
    let watcher = ObjectSpec::enchantment(p1, "Aura Shards (stub)")
        .with_triggered_ability(any_etb_trigger("Whenever a permanent enters..."))
        .in_zone(ZoneId::Battlefield);

    // A creature that p2 will cast.
    let creature_card = ObjectSpec::creature(p2, "Llanowar Elves", 2, 2).in_zone(ZoneId::Hand(p2));

    // p2's turn, empty stack, main phase.
    let state = GameStateBuilder::four_player()
        .active_player(p2)
        .at_step(Step::PreCombatMain)
        .object(watcher)
        .object(creature_card)
        .build();

    // Give p2 priority.
    let mut state = state;
    state.turn.priority_holder = Some(p2);

    let card_id = *state
        .zones
        .get(&ZoneId::Hand(p2))
        .unwrap()
        .object_ids()
        .first()
        .unwrap();

    // p2 casts the creature.
    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p2,
            card: card_id,
            targets: vec![],
        },
    )
    .unwrap();

    // All players pass → creature enters battlefield → watcher triggers.
    let (state, _) = process_command(state, Command::PassPriority { player: p2 }).unwrap();
    let (state, _) = process_command(state, Command::PassPriority { player: p(3) }).unwrap();
    let (state, _) = process_command(state, Command::PassPriority { player: p(4) }).unwrap();
    let (state, _) = process_command(state, Command::PassPriority { player: p1 }).unwrap();

    // Watcher's trigger should be on the stack.
    assert_eq!(
        state.stack_objects.len(),
        1,
        "watcher's triggered ability should be on the stack"
    );
    let trigger = &state.stack_objects[0];
    assert!(matches!(
        trigger.kind,
        StackObjectKind::TriggeredAbility { .. }
    ));
    // The trigger controller is p1 (watcher's controller).
    assert_eq!(trigger.controller, p1);
}

#[test]
/// CR 603.3 — APNAP ordering: active player's triggers go on the stack first
/// (and thus resolve last); non-active players' triggers resolve first.
fn test_triggered_ability_apnap_ordering() {
    let p1 = p(1); // Active player
    let p2 = p(2); // Non-active player

    // Both players have a permanent that triggers on any permanent entering.
    let watcher_p1 = ObjectSpec::enchantment(p1, "P1 Watcher")
        .with_triggered_ability(any_etb_trigger("P1 trigger"))
        .in_zone(ZoneId::Battlefield);

    let watcher_p2 = ObjectSpec::enchantment(p2, "P2 Watcher")
        .with_triggered_ability(any_etb_trigger("P2 trigger"))
        .in_zone(ZoneId::Battlefield);

    // A creature that will enter the battlefield.
    let creature_card = ObjectSpec::creature(p1, "Goblin Guide", 2, 2).in_zone(ZoneId::Hand(p1));

    let state = GameStateBuilder::four_player()
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(watcher_p1)
        .object(watcher_p2)
        .object(creature_card)
        .build();

    let card_id = *state
        .zones
        .get(&ZoneId::Hand(p1))
        .unwrap()
        .object_ids()
        .first()
        .unwrap();

    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: card_id,
            targets: vec![],
        },
    )
    .unwrap();

    // All pass → creature enters → both watchers trigger.
    let (state, _) = process_command(state, Command::PassPriority { player: p1 }).unwrap();
    let (state, _) = process_command(state, Command::PassPriority { player: p2 }).unwrap();
    let (state, _) = process_command(state, Command::PassPriority { player: p(3) }).unwrap();
    let (state, _) = process_command(state, Command::PassPriority { player: p(4) }).unwrap();

    // Two triggered abilities on the stack (the creature also may trigger but it has no ability).
    // At least two triggers should be on the stack.
    assert!(
        state.stack_objects.len() >= 2,
        "both watchers should have triggered"
    );

    // APNAP: p1's trigger was pushed first (bottom of stack), p2's trigger second (top of stack).
    // The LAST entry is on top and resolves first.
    // p1 is active player → p1's trigger goes first (ends up at the bottom).
    // p2 is non-active → p2's trigger goes second (ends up on top → resolves first).
    let top_trigger = &state.stack_objects[state.stack_objects.len() - 1];
    let bottom_trigger = &state.stack_objects[state.stack_objects.len() - 2];

    assert_eq!(
        top_trigger.controller, p2,
        "non-active player's trigger should be on top (resolves first)"
    );
    assert_eq!(
        bottom_trigger.controller, p1,
        "active player's trigger should be below (resolves last)"
    );
}

#[test]
/// CR 603.4 — intervening-if: ability does NOT trigger when condition is false at trigger time
fn test_triggered_ability_intervening_if_false_does_not_trigger() {
    let p1 = p(1);

    // "Whenever ~ enters, if your life total is 50 or more, do something."
    let conditional_creature = ObjectSpec::creature(p1, "High Life Watcher", 2, 2)
        .with_triggered_ability(TriggeredAbilityDef {
            trigger_on: TriggerEvent::SelfEntersBattlefield,
            intervening_if: Some(InterveningIf::ControllerLifeAtLeast(50)),
            description: "When ~ enters, if you have 50+ life, do something.".into(),
            effect: None,
        })
        .in_zone(ZoneId::Hand(p1));

    // Player starts at 40 life — condition (50+) is false at trigger time.
    let state = GameStateBuilder::four_player()
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(conditional_creature)
        .build();

    let card_id = *state
        .zones
        .get(&ZoneId::Hand(p1))
        .unwrap()
        .object_ids()
        .first()
        .unwrap();

    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: card_id,
            targets: vec![],
        },
    )
    .unwrap();

    // All pass → creature enters.
    let (state, _) = process_command(state, Command::PassPriority { player: p1 }).unwrap();
    let (state, _) = process_command(state, Command::PassPriority { player: p(2) }).unwrap();
    let (state, _) = process_command(state, Command::PassPriority { player: p(3) }).unwrap();
    let (state, _) = process_command(state, Command::PassPriority { player: p(4) }).unwrap();

    // Condition was false at trigger time → no trigger on the stack.
    assert!(
        state.stack_objects.is_empty(),
        "ability should not have triggered (condition false at trigger time)"
    );
}

#[test]
/// CR 603.4 — intervening-if: ability DOES trigger when condition is true at trigger time
fn test_triggered_ability_intervening_if_true_triggers() {
    let p1 = p(1);

    // "When ~ enters, if your life total is 30 or more, do something."
    let conditional_creature = ObjectSpec::creature(p1, "Sanctuary Warden", 2, 2)
        .with_triggered_ability(TriggeredAbilityDef {
            trigger_on: TriggerEvent::SelfEntersBattlefield,
            intervening_if: Some(InterveningIf::ControllerLifeAtLeast(30)),
            description: "When ~ enters, if you have 30+ life, do something.".into(),
            effect: None,
        })
        .in_zone(ZoneId::Hand(p1));

    // Player starts at 40 life — condition (30+) is true at trigger time.
    let state = GameStateBuilder::four_player()
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(conditional_creature)
        .build();

    let card_id = *state
        .zones
        .get(&ZoneId::Hand(p1))
        .unwrap()
        .object_ids()
        .first()
        .unwrap();

    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: card_id,
            targets: vec![],
        },
    )
    .unwrap();

    // All pass → creature enters.
    let (state, _) = process_command(state, Command::PassPriority { player: p1 }).unwrap();
    let (state, _) = process_command(state, Command::PassPriority { player: p(2) }).unwrap();
    let (state, _) = process_command(state, Command::PassPriority { player: p(3) }).unwrap();
    let (state, _) = process_command(state, Command::PassPriority { player: p(4) }).unwrap();

    // Condition was true → trigger is on the stack.
    assert_eq!(
        state.stack_objects.len(),
        1,
        "ability should have triggered (condition true at trigger time)"
    );
    assert!(matches!(
        state.stack_objects[0].kind,
        StackObjectKind::TriggeredAbility { .. }
    ));
}

#[test]
/// CR 603.3 — triggered ability resolves when all players pass priority
fn test_triggered_ability_resolves_after_all_pass() {
    let p1 = p(1);
    let p2 = p(2);
    let p3 = p(3);
    let p4 = p(4);

    let creature_card = ObjectSpec::creature(p1, "Siege-Gang Commander", 2, 2)
        .with_triggered_ability(etb_trigger("When ~ enters, do something"))
        .in_zone(ZoneId::Hand(p1));

    let state = GameStateBuilder::four_player()
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(creature_card)
        .build();

    let card_id = *state
        .zones
        .get(&ZoneId::Hand(p1))
        .unwrap()
        .object_ids()
        .first()
        .unwrap();

    // Cast creature, pass priority to put it on the stack, then resolve it.
    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: card_id,
            targets: vec![],
        },
    )
    .unwrap();

    // Spell resolves → creature enters → trigger goes on stack.
    let (state, _) = process_command(state, Command::PassPriority { player: p1 }).unwrap();
    let (state, _) = process_command(state, Command::PassPriority { player: p2 }).unwrap();
    let (state, _) = process_command(state, Command::PassPriority { player: p3 }).unwrap();
    let (state, _) = process_command(state, Command::PassPriority { player: p4 }).unwrap();

    // Triggered ability is now on the stack.
    assert_eq!(state.stack_objects.len(), 1);
    assert!(matches!(
        state.stack_objects[0].kind,
        StackObjectKind::TriggeredAbility { .. }
    ));

    // All players pass again → triggered ability resolves.
    let (state, _) = process_command(state, Command::PassPriority { player: p1 }).unwrap();
    let (state, _) = process_command(state, Command::PassPriority { player: p2 }).unwrap();
    let (state, _) = process_command(state, Command::PassPriority { player: p3 }).unwrap();
    let (state, events) = process_command(state, Command::PassPriority { player: p4 }).unwrap();

    // Stack is empty, AbilityResolved emitted.
    assert!(
        state.stack_objects.is_empty(),
        "stack should be empty after trigger resolves"
    );
    assert!(events
        .iter()
        .any(|e| matches!(e, GameEvent::AbilityResolved { controller, .. } if *controller == p1)));
}
