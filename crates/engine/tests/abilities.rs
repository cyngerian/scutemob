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
use mtg_engine::{AttackTarget, Effect, EffectAmount, PlayerTarget};

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

/// Tap-only activated ability (no mana cost).
fn tap_ability(description: &str) -> ActivatedAbility {
    ActivatedAbility {
        cost: ActivationCost {
            requires_tap: true,
            mana_cost: None,
            sacrifice_self: false,
        },
        description: description.to_string(),
        effect: None,
        sorcery_speed: false,
    }
}

/// Tap-and-pay activated ability (e.g., "{T}: draw a card").
fn tap_and_pay_ability(description: &str, mana: ManaCost) -> ActivatedAbility {
    ActivatedAbility {
        cost: ActivationCost {
            requires_tap: true,
            mana_cost: Some(mana),
            sacrifice_self: false,
        },
        description: description.to_string(),
        effect: None,
        sorcery_speed: false,
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
        .build()
        .unwrap();

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
        .build()
        .unwrap();

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
        .build()
        .unwrap();

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
        .build()
        .unwrap();

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
        .build()
        .unwrap();
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
        .build()
        .unwrap();

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
        .build()
        .unwrap();

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
        .build()
        .unwrap();

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
        .build()
        .unwrap();

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
        .build()
        .unwrap();

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
            cast_with_dash: false,
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
        .build()
        .unwrap();

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
            cast_with_dash: false,
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
        .build()
        .unwrap();

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
            cast_with_dash: false,
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
        .build()
        .unwrap();

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
            cast_with_dash: false,
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
        .build()
        .unwrap();

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
            cast_with_dash: false,
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
        .build()
        .unwrap();

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
            cast_with_dash: false,
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

// ---------------------------------------------------------------------------
// Sacrifice-as-cost
// ---------------------------------------------------------------------------

#[test]
/// CR 602.2c — sacrifice is paid at activation time; source leaves the battlefield
/// before the ability is placed on the stack. At resolution, the embedded_effect
/// is used because the source no longer exists.
///
/// Full flow: sacrifice-cost artifact → stack → all pass → resolve → draw card.
fn test_sacrifice_as_cost_full_flow_draw_card() {
    let p1 = p(1);
    let p2 = p(2);
    let p3 = p(3);
    let p4 = p(4);

    // Artifact with "Sacrifice this: draw a card." — no tap or mana cost, sacrifice only.
    let draw_effect = Effect::DrawCards {
        player: PlayerTarget::Controller,
        count: EffectAmount::Fixed(1),
    };
    let artifact = ObjectSpec::artifact(p1, "Jar of Eyeballs (stub)")
        .with_activated_ability(ActivatedAbility {
            cost: ActivationCost {
                requires_tap: false,
                mana_cost: None,
                sacrifice_self: true,
            },
            description: "Sacrifice: Draw a card.".into(),
            effect: Some(draw_effect),
            sorcery_speed: false,
        })
        .in_zone(ZoneId::Battlefield);

    // Add a card to p1's library so there's something to draw.
    let library_card = ObjectSpec::creature(p1, "Grizzly Bears", 2, 2).in_zone(ZoneId::Library(p1));

    let state = GameStateBuilder::four_player()
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(artifact)
        .object(library_card)
        .build()
        .unwrap();

    // Find the artifact on the battlefield.
    let source_id = *state
        .zones
        .get(&ZoneId::Battlefield)
        .unwrap()
        .object_ids()
        .first()
        .unwrap();

    // Activate the ability — sacrifice cost is paid immediately.
    let (state, events) = process_command(
        state,
        Command::ActivateAbility {
            player: p1,
            source: source_id,
            ability_index: 0,
            targets: vec![],
        },
    )
    .unwrap();

    // Source is now in p1's graveyard (sacrifice paid at activation time, CR 602.2).
    // Per CR 400.7, the object gets a new ObjectId when it changes zones — look up by name.
    let in_graveyard = state.objects.values().any(|o| {
        o.characteristics.name == "Jar of Eyeballs (stub)" && matches!(o.zone, ZoneId::Graveyard(_))
    });
    assert!(
        in_graveyard,
        "sacrificed source should be in graveyard immediately after activation"
    );

    // PermanentDestroyed (non-creature sacrifice) emitted.
    assert!(
        events.iter().any(|e| matches!(
            e,
            GameEvent::PermanentDestroyed { object_id, .. } if *object_id == source_id
        )),
        "PermanentDestroyed event expected for non-creature sacrifice"
    );

    // Ability is on the stack with embedded_effect.
    assert_eq!(state.stack_objects.len(), 1);
    assert!(matches!(
        state.stack_objects[0].kind,
        StackObjectKind::ActivatedAbility { .. }
    ));

    // Record hand size before resolution.
    let hand_before = state
        .zones
        .get(&ZoneId::Hand(p1))
        .map(|z| z.object_ids().len())
        .unwrap_or(0);

    // All four players pass priority → ability resolves.
    let (state, _) = process_command(state, Command::PassPriority { player: p1 }).unwrap();
    let (state, _) = process_command(state, Command::PassPriority { player: p2 }).unwrap();
    let (state, _) = process_command(state, Command::PassPriority { player: p3 }).unwrap();
    let (state, _) = process_command(state, Command::PassPriority { player: p4 }).unwrap();

    // Stack is empty after resolution.
    assert!(
        state.stack_objects.is_empty(),
        "stack should be empty after ability resolves"
    );

    // Player 1 drew a card: hand size increased by 1.
    let hand_after = state
        .zones
        .get(&ZoneId::Hand(p1))
        .map(|z| z.object_ids().len())
        .unwrap_or(0);
    assert_eq!(
        hand_after,
        hand_before + 1,
        "player 1 should have drawn one card from the sacrifice ability"
    );
}

// ---------------------------------------------------------------------------
// Dies trigger (CR 603.6c / CR 700.4 / CR 603.10a)
// ---------------------------------------------------------------------------

/// Triggered ability: fires when the source permanent dies (CR 700.4).
fn dies_trigger(description: &str) -> TriggeredAbilityDef {
    TriggeredAbilityDef {
        trigger_on: TriggerEvent::SelfDies,
        intervening_if: None,
        description: description.to_string(),
        effect: None,
    }
}

/// Dies trigger with a DrawCards effect for functional (resolution) testing.
fn dies_draw_trigger() -> TriggeredAbilityDef {
    TriggeredAbilityDef {
        trigger_on: TriggerEvent::SelfDies,
        intervening_if: None,
        description: "When ~ dies, draw a card. (CR 700.4)".to_string(),
        effect: Some(Effect::DrawCards {
            player: PlayerTarget::Controller,
            count: EffectAmount::Fixed(1),
        }),
    }
}

#[test]
/// CR 700.4 + CR 704.5g — creature with lethal damage dies via SBA, dies trigger fires
fn test_dies_trigger_fires_on_lethal_damage_sba() {
    let p1 = p(1);
    // A 2/2 creature with 2 damage marked (lethal) and a SelfDies trigger.
    let dying_creature = ObjectSpec::creature(p1, "Lethal Bear", 2, 2)
        .with_damage(2)
        .with_triggered_ability(dies_trigger("When ~ dies (CR 700.4)"))
        .in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::four_player()
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(dying_creature)
        .build()
        .unwrap();

    // All four players pass priority — when all have passed, SBAs fire.
    // Lethal damage SBA (CR 704.5g) moves creature to graveyard → CreatureDied emitted
    // → check_triggers finds SelfDies trigger → PendingTrigger queued
    // → flush_pending_triggers puts it on stack → AbilityTriggered emitted.
    let (state, _) = process_command(state, Command::PassPriority { player: p1 }).unwrap();
    let (state, _) = process_command(state, Command::PassPriority { player: p(2) }).unwrap();
    let (state, _) = process_command(state, Command::PassPriority { player: p(3) }).unwrap();
    let (state, events) = process_command(state, Command::PassPriority { player: p(4) }).unwrap();

    // The dies trigger should be on the stack.
    assert_eq!(
        state.stack_objects.len(),
        1,
        "dies trigger should be placed on the stack after creature dies via SBA"
    );
    assert!(
        matches!(
            state.stack_objects[0].kind,
            StackObjectKind::TriggeredAbility { .. }
        ),
        "stack object should be a triggered ability"
    );
    assert_eq!(
        state.stack_objects[0].controller, p1,
        "trigger controller should be the creature's owner/controller"
    );

    // AbilityTriggered event emitted.
    assert!(
        events.iter().any(
            |e| matches!(e, GameEvent::AbilityTriggered { controller, .. } if *controller == p1)
        ),
        "AbilityTriggered event should be emitted when dies trigger fires"
    );

    // CreatureDied event emitted.
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::CreatureDied { .. })),
        "CreatureDied event should be emitted"
    );
}

#[test]
/// CR 700.4 + CR 704.5f — creature with 0 toughness dies via SBA, dies trigger fires
fn test_dies_trigger_fires_on_zero_toughness_sba() {
    let p1 = p(1);
    // A 1/0 creature: toughness 0 → dies via CR 704.5f SBA.
    let zero_tough = ObjectSpec::creature(p1, "Frail Wisp", 1, 0)
        .with_triggered_ability(dies_trigger("When ~ dies (CR 700.4)"))
        .in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::four_player()
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(zero_tough)
        .build()
        .unwrap();

    let (state, _) = process_command(state, Command::PassPriority { player: p1 }).unwrap();
    let (state, _) = process_command(state, Command::PassPriority { player: p(2) }).unwrap();
    let (state, _) = process_command(state, Command::PassPriority { player: p(3) }).unwrap();
    let (state, events) = process_command(state, Command::PassPriority { player: p(4) }).unwrap();

    // Creature should be in the graveyard now.
    let in_grave = state
        .objects
        .values()
        .any(|o| o.characteristics.name == "Frail Wisp" && matches!(o.zone, ZoneId::Graveyard(_)));
    assert!(in_grave, "0-toughness creature should be in the graveyard");

    // Dies trigger should be on the stack.
    assert_eq!(
        state.stack_objects.len(),
        1,
        "dies trigger should be on the stack after 0-toughness SBA"
    );
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::AbilityTriggered { .. })),
        "AbilityTriggered event should be emitted for 0-toughness SBA death"
    );
}

#[test]
/// CR 700.4 — "dies" means specifically battlefield-to-graveyard; exile is not "dying"
/// so dies trigger must NOT fire when a creature is exiled instead.
fn test_dies_trigger_does_not_fire_when_exiled() {
    use mtg_engine::{
        EffectDuration, ObjectFilter, ReplacementEffect, ReplacementId, ReplacementModification,
        ReplacementTrigger, ZoneType,
    };

    let p1 = p(1);
    // A 2/2 creature with lethal damage and a SelfDies trigger.
    let creature = ObjectSpec::creature(p1, "Exiled Bear", 2, 2)
        .with_damage(2)
        .with_triggered_ability(dies_trigger("When ~ dies (CR 700.4)"))
        .in_zone(ZoneId::Battlefield);

    // Rest-in-Peace-like replacement: any permanent going to graveyard → exile instead.
    let rip_effect = ReplacementEffect {
        id: ReplacementId(1),
        source: None,
        controller: p1,
        duration: EffectDuration::Indefinite,
        is_self_replacement: false,
        trigger: ReplacementTrigger::WouldChangeZone {
            from: Some(ZoneType::Battlefield),
            to: ZoneType::Graveyard,
            filter: ObjectFilter::Any,
        },
        modification: ReplacementModification::RedirectToZone(ZoneType::Exile),
    };

    let state = GameStateBuilder::four_player()
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(creature)
        .with_replacement_effect(rip_effect)
        .build()
        .unwrap();

    let (state, _) = process_command(state, Command::PassPriority { player: p1 }).unwrap();
    let (state, _) = process_command(state, Command::PassPriority { player: p(2) }).unwrap();
    let (state, _) = process_command(state, Command::PassPriority { player: p(3) }).unwrap();
    let (state, events) = process_command(state, Command::PassPriority { player: p(4) }).unwrap();

    // Creature should be in exile, not graveyard.
    let in_exile = state
        .objects
        .values()
        .any(|o| o.characteristics.name == "Exiled Bear" && o.zone == ZoneId::Exile);
    assert!(
        in_exile,
        "creature should be in exile (replacement redirected graveyard→exile)"
    );

    // No CreatureDied event — creature was exiled, not put in graveyard (CR 700.4).
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, GameEvent::CreatureDied { .. })),
        "CreatureDied must NOT be emitted when creature is exiled instead of dying"
    );

    // No AbilityTriggered event for the dies trigger.
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, GameEvent::AbilityTriggered { .. })),
        "dies trigger must NOT fire when creature is exiled instead of going to graveyard"
    );

    // Stack is empty — no trigger queued.
    assert!(
        state.stack_objects.is_empty(),
        "stack should be empty: no dies trigger fires on exile"
    );
}

#[test]
/// CR 603.6c — dies trigger resolves and its DrawCards effect executes
fn test_dies_trigger_resolves_draws_card() {
    let p1 = p(1);
    // Creature with a dies trigger that draws a card on resolution.
    let dying_creature = ObjectSpec::creature(p1, "Selfless Spirit (stub)", 2, 1)
        .with_damage(1)
        .with_triggered_ability(dies_draw_trigger())
        .in_zone(ZoneId::Battlefield);

    // Add a card to p1's library so there's something to draw.
    let library_card = ObjectSpec::creature(p1, "Grizzly Bears", 2, 2).in_zone(ZoneId::Library(p1));

    let state = GameStateBuilder::four_player()
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(dying_creature)
        .object(library_card)
        .build()
        .unwrap();

    // Round 1: all pass → SBA kills creature → dies trigger goes on stack.
    let (state, _) = process_command(state, Command::PassPriority { player: p1 }).unwrap();
    let (state, _) = process_command(state, Command::PassPriority { player: p(2) }).unwrap();
    let (state, _) = process_command(state, Command::PassPriority { player: p(3) }).unwrap();
    let (state, _) = process_command(state, Command::PassPriority { player: p(4) }).unwrap();

    // Trigger should now be on the stack.
    assert_eq!(
        state.stack_objects.len(),
        1,
        "dies trigger should be on the stack"
    );

    let hand_before = state
        .zones
        .get(&ZoneId::Hand(p1))
        .map(|z| z.object_ids().len())
        .unwrap_or(0);

    // Round 2: all pass → dies trigger resolves → controller draws a card.
    let (state, _) = process_command(state, Command::PassPriority { player: p1 }).unwrap();
    let (state, _) = process_command(state, Command::PassPriority { player: p(2) }).unwrap();
    let (state, _) = process_command(state, Command::PassPriority { player: p(3) }).unwrap();
    let (state, events) = process_command(state, Command::PassPriority { player: p(4) }).unwrap();

    // Stack empty after resolution.
    assert!(
        state.stack_objects.is_empty(),
        "stack should be empty after dies trigger resolves"
    );

    // AbilityResolved event emitted.
    assert!(
        events.iter().any(
            |e| matches!(e, GameEvent::AbilityResolved { controller, .. } if *controller == p1)
        ),
        "AbilityResolved event should be emitted when dies trigger resolves"
    );

    // p1 drew a card: hand grew by 1.
    let hand_after = state
        .zones
        .get(&ZoneId::Hand(p1))
        .map(|z| z.object_ids().len())
        .unwrap_or(0);
    assert_eq!(
        hand_after,
        hand_before + 1,
        "controller should draw one card when dies trigger resolves"
    );
}

#[test]
/// CR 700.4 — sacrifice puts creature into graveyard from battlefield = "dies"
fn test_dies_trigger_fires_on_sacrifice() {
    let p1 = p(1);
    // Creature with a sacrifice-self activated ability and a SelfDies trigger.
    // Sacrifice is paid at activation time (CR 602.2c), emitting CreatureDied.
    let creature = ObjectSpec::creature(p1, "Doomed Traveler (stub)", 1, 1)
        .with_activated_ability(ActivatedAbility {
            cost: ActivationCost {
                requires_tap: false,
                mana_cost: None,
                sacrifice_self: true,
            },
            description: "Sacrifice: trigger dies".to_string(),
            effect: None,
            sorcery_speed: false,
        })
        .with_triggered_ability(dies_trigger("When ~ dies (CR 700.4)"))
        .in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::four_player()
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(creature)
        .build()
        .unwrap();

    let source_id = *state
        .zones
        .get(&ZoneId::Battlefield)
        .unwrap()
        .object_ids()
        .first()
        .unwrap();

    // Activate the sacrifice ability — creature moves to graveyard immediately.
    let (state, events) = process_command(
        state,
        Command::ActivateAbility {
            player: p1,
            source: source_id,
            ability_index: 0,
            targets: vec![],
        },
    )
    .unwrap();

    // Creature died: CreatureDied emitted at activation time (sacrifice-as-cost).
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::CreatureDied { .. })),
        "CreatureDied should be emitted when creature is sacrificed as a cost"
    );

    // The sacrificed activated ability and the dies trigger should both be on the stack.
    // Dies trigger was flushed after the CreatureDied event from sacrifice.
    let has_triggered = state
        .stack_objects
        .iter()
        .any(|so| matches!(so.kind, StackObjectKind::TriggeredAbility { .. }));
    assert!(
        has_triggered,
        "dies trigger should be on the stack after sacrifice-as-cost"
    );
}

#[test]
/// CR 700.4 — destruction via spell effect causes creature to die; dies trigger fires
fn test_dies_trigger_fires_on_destruction_effect() {
    let p1 = p(1);
    // A 2/2 creature with a dies trigger. Lethal damage simulates destruction.
    let creature = ObjectSpec::creature(p1, "Vanilla Creature", 2, 2)
        .with_damage(2)
        .with_triggered_ability(dies_trigger("When ~ dies (CR 700.4)"))
        .in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::four_player()
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(creature)
        .build()
        .unwrap();

    // All pass → SBA fires → creature destroyed → CreatureDied → dies trigger fires.
    let (state, _) = process_command(state, Command::PassPriority { player: p1 }).unwrap();
    let (state, _) = process_command(state, Command::PassPriority { player: p(2) }).unwrap();
    let (state, _) = process_command(state, Command::PassPriority { player: p(3) }).unwrap();
    let (state, events) = process_command(state, Command::PassPriority { player: p(4) }).unwrap();

    // Creature is in graveyard.
    let in_grave = state.objects.values().any(|o| {
        o.characteristics.name == "Vanilla Creature" && matches!(o.zone, ZoneId::Graveyard(_))
    });
    assert!(
        in_grave,
        "creature should be in graveyard after lethal damage SBA"
    );

    // Dies trigger on stack.
    assert_eq!(
        state.stack_objects.len(),
        1,
        "dies trigger should be on the stack"
    );
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::AbilityTriggered { .. })),
        "AbilityTriggered event should be emitted"
    );
}

#[test]
/// CR 603.3b — multiple creatures with dies triggers die simultaneously in SBAs;
/// all triggers queue and are placed on the stack in APNAP order.
fn test_dies_trigger_multiple_creatures_simultaneous_sba() {
    let p1 = p(1); // Active player
    let p2 = p(2); // Non-active player

    // Both players have a creature with lethal damage and a SelfDies trigger.
    let creature_p1 = ObjectSpec::creature(p1, "P1 Creature", 2, 2)
        .with_damage(2)
        .with_triggered_ability(dies_trigger("P1 dies trigger"))
        .in_zone(ZoneId::Battlefield);

    let creature_p2 = ObjectSpec::creature(p2, "P2 Creature", 3, 3)
        .with_damage(3)
        .with_triggered_ability(dies_trigger("P2 dies trigger"))
        .in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::four_player()
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(creature_p1)
        .object(creature_p2)
        .build()
        .unwrap();

    // All pass → both creatures die simultaneously → both triggers queue.
    let (state, _) = process_command(state, Command::PassPriority { player: p1 }).unwrap();
    let (state, _) = process_command(state, Command::PassPriority { player: p(2) }).unwrap();
    let (state, _) = process_command(state, Command::PassPriority { player: p(3) }).unwrap();
    let (state, events) = process_command(state, Command::PassPriority { player: p(4) }).unwrap();

    // Both triggers should be on the stack.
    assert!(
        state.stack_objects.len() >= 2,
        "both dies triggers should be on the stack (got {})",
        state.stack_objects.len()
    );

    // Both AbilityTriggered events emitted.
    let trigger_events: Vec<_> = events
        .iter()
        .filter(|e| matches!(e, GameEvent::AbilityTriggered { .. }))
        .collect();
    assert_eq!(
        trigger_events.len(),
        2,
        "exactly two AbilityTriggered events expected (one per dying creature)"
    );

    // APNAP order (CR 603.3b): p1 is active player → p1's trigger pushed first (bottom),
    // p2 is non-active → p2's trigger pushed second (top → resolves first).
    let top = &state.stack_objects[state.stack_objects.len() - 1];
    let bottom = &state.stack_objects[state.stack_objects.len() - 2];
    assert_eq!(
        top.controller, p2,
        "non-active player's dies trigger should be on top of stack (resolves first)"
    );
    assert_eq!(
        bottom.controller, p1,
        "active player's dies trigger should be below (resolves last)"
    );
}

#[test]
/// Corner case #24 — token creature with a dies trigger fires correctly (CR 603.6c + CR 700.4).
///
/// CR 700.4: "Dies" means "is put into a graveyard from the battlefield."
/// CR 603.6c: Leaves-the-battlefield triggers fire when a permanent moves from the battlefield.
/// CR 704.5d: A token in a non-battlefield zone ceases to exist (SBA).
///
/// The fix (Finding 1): trigger checking now runs inside each SBA pass. After pass 1 moves
/// the token to the graveyard (emitting CreatureDied), check_triggers queues the SelfDies
/// trigger while the token still exists in state.objects. Pass 2 then runs CR 704.5d to
/// remove the token from the graveyard, but the trigger is already queued and will fire.
fn test_dies_trigger_token_creature_fires() {
    let p1 = p(1);
    // Token creature with lethal damage and a SelfDies triggered ability.
    let token = ObjectSpec::creature(p1, "Goblin Token", 1, 1)
        .with_damage(1)
        .token()
        .with_triggered_ability(dies_trigger("When ~ dies (token, CR 700.4)"))
        .in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::four_player()
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(token)
        .build()
        .unwrap();

    // All pass → SBA pass 1 moves token to graveyard (CreatureDied emitted)
    //          → check_triggers queues SelfDies trigger (token still in state.objects)
    //          → SBA pass 2 removes token from graveyard (CR 704.5d)
    //          → flush_pending_triggers puts trigger on stack → AbilityTriggered emitted.
    let (state, _) = process_command(state, Command::PassPriority { player: p1 }).unwrap();
    let (state, _) = process_command(state, Command::PassPriority { player: p(2) }).unwrap();
    let (state, _) = process_command(state, Command::PassPriority { player: p(3) }).unwrap();
    let (state, events) = process_command(state, Command::PassPriority { player: p(4) }).unwrap();

    // Token no longer exists anywhere (CR 704.5d removes it from graveyard).
    let token_exists = state
        .objects
        .values()
        .any(|o| o.characteristics.name == "Goblin Token");
    assert!(!token_exists, "token should be cleaned up by SBA 704.5d");

    // CreatureDied was emitted (in the first SBA pass).
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::CreatureDied { .. })),
        "CreatureDied should be emitted even for token creature deaths"
    );

    // CR 603.6c: "when this dies" trigger on token DOES fire — trigger was queued during
    // pass 1 (before pass 2 ran CR 704.5d). The trigger is then flushed to the stack.
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::AbilityTriggered { .. })),
        "self-dies trigger on token should fire (CR 603.6c): trigger queued inside SBA pass 1"
    );
    assert!(
        !state.stack_objects.is_empty(),
        "dies trigger should be on the stack after firing for token creature"
    );
}

#[test]
/// CR 603.6c / CR 700.4 / CR 704.5g — WhenDies trigger must fire when the creature was built
/// via `enrich_spec_from_def` (the card-definition path), not just via manual
/// `.with_triggered_ability()`. This is a regression test for the bug where `enrich_spec_from_def`
/// failed to populate `spec.triggered_abilities` with the SelfDies trigger, causing dies
/// triggers on card-definition-backed objects to silently not fire in the script harness.
fn test_dies_trigger_via_card_definition_enrich_path() {
    use mtg_engine::{all_cards, enrich_spec_from_def, CardDefinition};
    use std::collections::HashMap;

    let p1 = p(1);

    // Build the card-definition map exactly as the script harness does.
    let cards = all_cards();
    let defs: HashMap<String, CardDefinition> =
        cards.iter().map(|d| (d.name.clone(), d.clone())).collect();

    // Solemn Simulacrum is a 2/2 artifact creature with:
    //   "When Solemn Simulacrum dies, you may draw a card."
    // Enrich via the card-definition path (the same code path used by the script harness).
    let base = ObjectSpec::card(p1, "Solemn Simulacrum")
        .in_zone(ZoneId::Battlefield)
        .with_damage(2); // lethal damage (2 damage on 2/2 → CR 704.5g)
    let solemn = enrich_spec_from_def(base, &defs);

    // Sanity-check: the enriched spec must have exactly one SelfDies trigger (the
    // WhenDies → DrawCards ability). If this assertion fails, the bug is in
    // enrich_spec_from_def — it did not convert the WhenDies ability into a SelfDies trigger.
    assert_eq!(
        solemn
            .triggered_abilities
            .iter()
            .filter(|t| t.trigger_on == TriggerEvent::SelfDies)
            .count(),
        1,
        "enrich_spec_from_def must populate exactly one SelfDies trigger from the \
         WhenDies ability in Solemn Simulacrum's card definition"
    );

    let state = GameStateBuilder::four_player()
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(solemn)
        .build()
        .unwrap();

    // All four players pass priority → SBAs fire (CR 704.5g: lethal damage → creature dies)
    // → CreatureDied emitted → check_triggers finds SelfDies trigger on the graveyard object
    // → PendingTrigger queued → flush_pending_triggers places it on the stack.
    let (state, _) = process_command(state, Command::PassPriority { player: p1 }).unwrap();
    let (state, _) = process_command(state, Command::PassPriority { player: p(2) }).unwrap();
    let (state, _) = process_command(state, Command::PassPriority { player: p(3) }).unwrap();
    let (state, events) = process_command(state, Command::PassPriority { player: p(4) }).unwrap();

    // The creature should be in the graveyard (lethal damage SBA fired).
    let in_grave = state.objects.values().any(|o| {
        o.characteristics.name == "Solemn Simulacrum" && matches!(o.zone, ZoneId::Graveyard(_))
    });
    assert!(
        in_grave,
        "Solemn Simulacrum should have moved to the graveyard via SBA 704.5g"
    );

    // CreatureDied event should have been emitted.
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::CreatureDied { .. })),
        "CreatureDied event should be emitted when Solemn Simulacrum dies via SBA"
    );

    // AbilityTriggered event should have been emitted — the WhenDies trigger fired.
    assert!(
        events.iter().any(
            |e| matches!(e, GameEvent::AbilityTriggered { controller, .. } if *controller == p1)
        ),
        "AbilityTriggered event should be emitted for Solemn Simulacrum's WhenDies trigger \
         when built via enrich_spec_from_def (card-definition path)"
    );

    // The WhenDies trigger should be on the stack.
    assert!(
        !state.stack_objects.is_empty(),
        "Solemn Simulacrum's WhenDies trigger should be on the stack after dying via SBA \
         (card-definition enrich path)"
    );
    assert!(
        matches!(
            state.stack_objects[0].kind,
            StackObjectKind::TriggeredAbility { .. }
        ),
        "stack object should be a triggered ability (WhenDies from card definition)"
    );
}

#[test]
/// CR 603.6c / CR 700.4 / CR 704.5g — Full end-to-end: Lightning Bolt deals damage to
/// Solemn Simulacrum (enrich_spec_from_def path), lethal-damage SBA fires, dies trigger
/// resolves, and p1 draws a card. Verifies hand count goes from 3 to 4.
///
/// This test replicates the full scenario from script 058 inline:
/// - p1 controls Solemn Simulacrum (card-def path)
/// - p2 casts Lightning Bolt targeting it (3 damage on a 2/2 = lethal)
/// - Both players pass priority → Lightning Bolt resolves → 3 damage marked
/// - SBA fires (CR 704.5g) → Simulacrum dies → dies trigger fires
/// - Both pass again → trigger resolves → p1 draws 1 card (hand 3 → 4)
fn test_dies_trigger_full_via_lightning_bolt_and_sba() {
    use mtg_engine::state::Target;
    use mtg_engine::{
        all_cards, card_name_to_id, enrich_spec_from_def, CardDefinition, CardRegistry,
    };
    use std::collections::HashMap;

    let p1 = p(1);
    let p2 = p(2);

    // Build the card-definition map.
    let cards = all_cards();
    let defs: HashMap<String, CardDefinition> =
        cards.iter().map(|d| (d.name.clone(), d.clone())).collect();
    let registry = CardRegistry::new(cards);

    // Helper: build a spec with card_id set (required for spell resolution to look up the
    // CardDefinition from the registry — without card_id, resolution.rs skips the effect).
    let make_spec =
        |owner: mtg_engine::state::PlayerId, name: &str, zone: mtg_engine::state::zone::ZoneId| {
            enrich_spec_from_def(
                ObjectSpec::card(owner, name)
                    .in_zone(zone)
                    .with_card_id(card_name_to_id(name)),
                &defs,
            )
        };

    // Solemn Simulacrum on the battlefield (0 damage pre-set — damage will come from Lightning Bolt).
    let solemn = make_spec(p1, "Solemn Simulacrum", ZoneId::Battlefield);

    // Lightning Bolt in p2's hand (3 damage to any target).
    let bolt = make_spec(p2, "Lightning Bolt", ZoneId::Hand(p2));

    // Three cards in p1's hand (Forest, Island, Plains) + one card in library (Mountain).
    // These don't need to be spell-resolvable, so no card_id is strictly required,
    // but we use make_spec for consistency.
    let forest = make_spec(p1, "Forest", ZoneId::Hand(p1));
    let island = make_spec(p1, "Island", ZoneId::Hand(p1));
    let plains = make_spec(p1, "Plains", ZoneId::Hand(p1));
    let mountain = make_spec(p1, "Mountain", ZoneId::Library(p1));

    let mut state = GameStateBuilder::new()
        .active_player(p2)
        .at_step(Step::PreCombatMain)
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(solemn)
        .object(bolt)
        .object(forest)
        .object(island)
        .object(plains)
        .object(mountain)
        .build()
        .unwrap();

    // Patch p2's mana pool (Lightning Bolt costs {R}).
    if let Some(ps) = state.players.get_mut(&p2) {
        ps.mana_pool.red = 1;
    }

    // Verify initial state: 3 cards in p1's hand, 1 in library.
    let p1_hand_count_initial = state
        .objects
        .values()
        .filter(|o| o.zone == ZoneId::Hand(p1))
        .count();
    assert_eq!(
        p1_hand_count_initial, 3,
        "p1 should start with 3 cards in hand"
    );

    // Find the Solemn Simulacrum on the battlefield.
    let solemn_id = state
        .objects
        .iter()
        .find(|(_, o)| {
            o.characteristics.name == "Solemn Simulacrum" && o.zone == ZoneId::Battlefield
        })
        .map(|(id, _)| *id)
        .expect("Solemn Simulacrum must be on battlefield");

    // Verify Solemn has the SelfDies trigger after enrichment.
    let solemn_obj = state.objects.get(&solemn_id).unwrap();
    assert_eq!(
        solemn_obj
            .characteristics
            .triggered_abilities
            .iter()
            .filter(|t| t.trigger_on == TriggerEvent::SelfDies)
            .count(),
        1,
        "Solemn Simulacrum should have exactly one SelfDies trigger after enrich_spec_from_def"
    );

    // Find Lightning Bolt in p2's hand.
    let bolt_id = state
        .objects
        .iter()
        .find(|(_, o)| o.characteristics.name == "Lightning Bolt" && o.zone == ZoneId::Hand(p2))
        .map(|(id, _)| *id)
        .expect("Lightning Bolt must be in p2's hand");

    // p2 casts Lightning Bolt targeting Solemn Simulacrum.
    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p2,
            card: bolt_id,
            targets: vec![Target::Object(solemn_id)],
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
            cast_with_dash: false,
        },
    )
    .expect("p2 should be able to cast Lightning Bolt targeting Solemn Simulacrum");

    // Stack should have Lightning Bolt on it.
    assert_eq!(
        state.stack_objects.len(),
        1,
        "Lightning Bolt should be on the stack"
    );

    // p2 (active player) passes priority, then p1 passes → Lightning Bolt resolves.
    // After resolution: 3 damage marked on Solemn → SBA fires → Simulacrum dies.
    // Dies trigger fires → placed on stack.
    let (state, _) = process_command(state, Command::PassPriority { player: p2 }).unwrap();
    let (state, _) = process_command(state, Command::PassPriority { player: p1 }).unwrap();

    // After resolution: Solemn Simulacrum should be in the graveyard.
    let solemn_in_grave = state.objects.values().any(|o| {
        o.characteristics.name == "Solemn Simulacrum" && matches!(o.zone, ZoneId::Graveyard(_))
    });
    assert!(
        solemn_in_grave,
        "Solemn Simulacrum should be in the graveyard after Lightning Bolt + SBA"
    );

    // The dies trigger should be on the stack.
    assert_eq!(
        state.stack_objects.len(),
        1,
        "Solemn Simulacrum's WhenDies trigger should be on the stack after lethal damage + SBA \
         (card-definition enrich path, Lightning Bolt scenario)"
    );

    // p2 then p1 pass priority again → dies trigger resolves → p1 draws a card.
    let (state, _) = process_command(state, Command::PassPriority { player: p2 }).unwrap();
    let (state, _) = process_command(state, Command::PassPriority { player: p1 }).unwrap();

    // Stack should now be empty.
    assert!(
        state.stack_objects.is_empty(),
        "Stack should be empty after dies trigger resolves"
    );

    // p1 should have drawn 1 card from the dies trigger (3 initial + 1 drawn = 4).
    let p1_hand_count_final = state
        .objects
        .values()
        .filter(|o| o.zone == ZoneId::Hand(p1))
        .count();
    assert_eq!(
        p1_hand_count_final, 4,
        "p1 should have 4 cards in hand after Solemn Simulacrum's WhenDies trigger draws 1 \
         (card-definition enrich path, lethal-damage SBA path, Lightning Bolt scenario)"
    );
}

// ---------------------------------------------------------------------------
// Attack trigger (CR 508.1m / CR 508.3a / CR 603)
// ---------------------------------------------------------------------------

/// Attack trigger: fires when THIS creature is declared as an attacker (CR 508.3a).
fn attack_trigger(description: &str) -> TriggeredAbilityDef {
    TriggeredAbilityDef {
        trigger_on: TriggerEvent::SelfAttacks,
        intervening_if: None,
        description: description.to_string(),
        effect: None,
    }
}

/// Attack trigger with a DrawCards effect for functional (resolution) testing.
fn attack_draw_trigger() -> TriggeredAbilityDef {
    TriggeredAbilityDef {
        trigger_on: TriggerEvent::SelfAttacks,
        intervening_if: None,
        description: "Whenever ~ attacks, draw a card. (CR 508.3a)".to_string(),
        effect: Some(Effect::DrawCards {
            player: PlayerTarget::Controller,
            count: EffectAmount::Fixed(1),
        }),
    }
}

#[test]
/// CR 508.1m, CR 508.3a, CR 603.2 — attack trigger fires when creature is declared as
/// attacker; trigger is placed on the stack; AbilityTriggered event emitted.
/// 4-player multiplayer for multiplayer-first coverage (CR 508.2b: APNAP ordering).
fn test_attack_trigger_fires_on_declare_attackers() {
    let p1 = p(1);
    let p2 = p(2);

    let attacker = ObjectSpec::creature(p1, "Attack Bear", 2, 2)
        .with_triggered_ability(attack_trigger("Whenever ~ attacks (CR 508.3a)"))
        .in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::four_player()
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .object(attacker)
        .build()
        .unwrap();

    let attacker_id = state
        .objects
        .values()
        .find(|o| o.controller == p1 && o.zone == ZoneId::Battlefield)
        .unwrap()
        .id;

    // Declare the creature as an attacker against p2.
    // CR 508.1m: abilities that trigger on attackers being declared trigger now.
    // CR 508.3a: "Whenever ~ attacks" triggers if declared as attacker.
    let (state, events) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(attacker_id, AttackTarget::Player(p2))],
            enlist_choices: vec![],
        },
    )
    .expect("DeclareAttackers should succeed");

    // AbilityTriggered event should have been emitted for the attacking creature.
    assert!(
        events.iter().any(|e| matches!(
            e,
            GameEvent::AbilityTriggered { source_object_id, controller, .. }
            if *source_object_id == attacker_id && *controller == p1
        )),
        "AbilityTriggered event should be emitted when creature with SelfAttacks trigger \
         is declared as attacker (CR 508.3a)"
    );

    // The trigger should be on the stack (CR 508.2b: goes on stack before priority).
    assert_eq!(
        state.stack_objects.len(),
        1,
        "attack trigger should be placed on the stack (CR 508.2b)"
    );
    assert!(
        matches!(
            state.stack_objects[0].kind,
            StackObjectKind::TriggeredAbility { .. }
        ),
        "stack object should be a triggered ability"
    );
    assert_eq!(
        state.stack_objects[0].controller, p1,
        "trigger controller should be the attacker's controller"
    );
}

#[test]
/// CR 508.3a / enrich_spec_from_def — a creature built from a CardDefinition with
/// WhenAttacks trigger correctly receives a SelfAttacks runtime trigger.
/// This validates the enrichment gap fixed in testing/replay_harness.rs.
fn test_attack_trigger_via_card_definition_enrich_path() {
    use mtg_engine::{
        enrich_spec_from_def, AbilityDefinition, CardDefinition, CardType, TriggerCondition,
        TypeLine,
    };
    use std::collections::HashMap;

    let p1 = p(1);
    let p2 = p(2);

    // Build a CardDefinition inline with a WhenAttacks trigger that draws a card.
    // (No attack-trigger card exists in definitions.rs yet — Step 5 adds Audacious Thief.)
    // CardType::Creature is required so DeclareAttackers validation passes.
    let attack_def = CardDefinition {
        card_id: mtg_engine::CardId("test-attack-creature".to_string()),
        name: "Test Attacker".to_string(),
        types: TypeLine {
            card_types: vec![CardType::Creature].into(),
            ..Default::default()
        },
        abilities: vec![AbilityDefinition::Triggered {
            trigger_condition: TriggerCondition::WhenAttacks,
            effect: mtg_engine::Effect::DrawCards {
                player: mtg_engine::PlayerTarget::Controller,
                count: mtg_engine::EffectAmount::Fixed(1),
            },
            intervening_if: None,
        }],
        power: Some(2),
        toughness: Some(2),
        ..Default::default()
    };

    let defs: HashMap<String, CardDefinition> =
        std::iter::once((attack_def.name.clone(), attack_def)).collect();

    // Enrich via the card-definition path (same as script harness).
    let base = ObjectSpec::card(p1, "Test Attacker").in_zone(ZoneId::Battlefield);
    let enriched = enrich_spec_from_def(base, &defs);

    // Sanity-check: exactly one SelfAttacks trigger from the WhenAttacks ability.
    assert_eq!(
        enriched
            .triggered_abilities
            .iter()
            .filter(|t| t.trigger_on == TriggerEvent::SelfAttacks)
            .count(),
        1,
        "enrich_spec_from_def must populate exactly one SelfAttacks trigger from the \
         WhenAttacks ability in the card definition (CR 508.3a)"
    );

    let state = GameStateBuilder::four_player()
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .object(enriched)
        .build()
        .unwrap();

    let attacker_id = state
        .objects
        .values()
        .find(|o| o.controller == p1 && o.zone == ZoneId::Battlefield)
        .unwrap()
        .id;

    // Declare attacker — trigger should fire via the enriched SelfAttacks ability.
    let (state, events) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(attacker_id, AttackTarget::Player(p2))],
            enlist_choices: vec![],
        },
    )
    .expect("DeclareAttackers should succeed for enriched card-def creature");

    // AbilityTriggered event should be emitted (card-definition enrich path).
    assert!(
        events.iter().any(
            |e| matches!(e, GameEvent::AbilityTriggered { controller, .. } if *controller == p1)
        ),
        "AbilityTriggered event should be emitted for WhenAttacks trigger built via \
         enrich_spec_from_def (card-definition path)"
    );

    // Trigger on stack.
    assert!(
        !state.stack_objects.is_empty(),
        "attack trigger should be on the stack after enrich_spec_from_def path"
    );
    assert!(
        matches!(
            state.stack_objects[0].kind,
            StackObjectKind::TriggeredAbility { .. }
        ),
        "stack object should be a triggered ability (WhenAttacks from card definition)"
    );
}

#[test]
/// CR 508.3a, CR 603 — attack trigger with DrawCards effect resolves; controller draws a card.
fn test_attack_trigger_resolves_draws_card() {
    let p1 = p(1);
    let p2 = p(2);

    let attacker = ObjectSpec::creature(p1, "Drawing Attacker", 2, 2)
        .with_triggered_ability(attack_draw_trigger())
        .in_zone(ZoneId::Battlefield);

    // Add a card to p1's library so there's something to draw.
    let library_card = ObjectSpec::creature(p1, "Library Card", 1, 1).in_zone(ZoneId::Library(p1));

    let state = GameStateBuilder::four_player()
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .object(attacker)
        .object(library_card)
        .build()
        .unwrap();

    let attacker_id = state
        .objects
        .values()
        .find(|o| o.controller == p1 && o.zone == ZoneId::Battlefield)
        .unwrap()
        .id;

    // Declare attacker → attack trigger goes on stack.
    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(attacker_id, AttackTarget::Player(p2))],
            enlist_choices: vec![],
        },
    )
    .expect("DeclareAttackers should succeed");

    // Trigger should be on the stack.
    assert_eq!(
        state.stack_objects.len(),
        1,
        "attack trigger should be on the stack before resolution"
    );

    let hand_before = state
        .zones
        .get(&ZoneId::Hand(p1))
        .map(|z| z.object_ids().len())
        .unwrap_or(0);

    // All four players pass priority → trigger resolves → p1 draws a card.
    let (state, _) = process_command(state, Command::PassPriority { player: p1 }).unwrap();
    let (state, _) = process_command(state, Command::PassPriority { player: p(2) }).unwrap();
    let (state, _) = process_command(state, Command::PassPriority { player: p(3) }).unwrap();
    let (state, events) = process_command(state, Command::PassPriority { player: p(4) }).unwrap();

    // Stack should be empty after resolution.
    assert!(
        state.stack_objects.is_empty(),
        "stack should be empty after attack trigger resolves"
    );

    // AbilityResolved event emitted.
    assert!(
        events.iter().any(
            |e| matches!(e, GameEvent::AbilityResolved { controller, .. } if *controller == p1)
        ),
        "AbilityResolved event should be emitted when attack trigger resolves"
    );

    // p1 drew a card: hand grew by 1.
    let hand_after = state
        .zones
        .get(&ZoneId::Hand(p1))
        .map(|z| z.object_ids().len())
        .unwrap_or(0);
    assert_eq!(
        hand_after,
        hand_before + 1,
        "controller should draw one card when attack trigger resolves"
    );
}

#[test]
/// CR 508.3a (negative) — attack trigger does NOT fire when a different creature attacks;
/// only the declared attacker's own SelfAttacks trigger fires.
fn test_attack_trigger_does_not_fire_for_non_attacker() {
    let p1 = p(1);
    let p2 = p(2);

    // Only the bystander has a SelfAttacks trigger; the actual attacker does not.
    let bystander = ObjectSpec::creature(p1, "Trigger Bystander", 1, 1)
        .with_triggered_ability(attack_trigger("SelfAttacks trigger on non-attacker"))
        .in_zone(ZoneId::Battlefield);

    let attacker = ObjectSpec::creature(p1, "Plain Attacker", 2, 2).in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::four_player()
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .object(bystander)
        .object(attacker)
        .build()
        .unwrap();

    // Find the plain attacker (no trigger) by name.
    let attacker_id = state
        .objects
        .values()
        .find(|o| o.characteristics.name == "Plain Attacker" && o.zone == ZoneId::Battlefield)
        .unwrap()
        .id;

    let bystander_id = state
        .objects
        .values()
        .find(|o| o.characteristics.name == "Trigger Bystander" && o.zone == ZoneId::Battlefield)
        .unwrap()
        .id;

    // Declare only the plain attacker — the bystander is NOT attacking.
    let (state, events) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(attacker_id, AttackTarget::Player(p2))],
            enlist_choices: vec![],
        },
    )
    .expect("DeclareAttackers should succeed");

    // No AbilityTriggered event for the bystander.
    let bystander_triggered = events.iter().any(|e| {
        matches!(e, GameEvent::AbilityTriggered { source_object_id, .. }
            if *source_object_id == bystander_id)
    });
    assert!(
        !bystander_triggered,
        "SelfAttacks trigger on non-attacking creature must NOT fire (CR 508.3a)"
    );

    // Stack should be empty (the plain attacker has no trigger either).
    assert!(
        state.stack_objects.is_empty(),
        "stack should be empty — only the non-trigger attacker declared"
    );
}

#[test]
/// CR 508.1m, CR 603.3b (APNAP) — two creatures with SelfAttacks triggers declared
/// simultaneously both have their triggers fire; 2 AbilityTriggered events, 2 stack objects.
fn test_attack_trigger_multiple_attackers() {
    let p1 = p(1);
    let p2 = p(2);

    let attacker_a = ObjectSpec::creature(p1, "Attacker Alpha", 2, 2)
        .with_triggered_ability(attack_trigger("Alpha attacks trigger"))
        .in_zone(ZoneId::Battlefield);

    let attacker_b = ObjectSpec::creature(p1, "Attacker Beta", 2, 2)
        .with_triggered_ability(attack_trigger("Beta attacks trigger"))
        .in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::four_player()
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .object(attacker_a)
        .object(attacker_b)
        .build()
        .unwrap();

    let attacker_a_id = state
        .objects
        .values()
        .find(|o| o.characteristics.name == "Attacker Alpha" && o.zone == ZoneId::Battlefield)
        .unwrap()
        .id;
    let attacker_b_id = state
        .objects
        .values()
        .find(|o| o.characteristics.name == "Attacker Beta" && o.zone == ZoneId::Battlefield)
        .unwrap()
        .id;

    // Declare both creatures as attackers simultaneously.
    // CR 508.1m: all attack triggers from this declaration fire together.
    let (state, events) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![
                (attacker_a_id, AttackTarget::Player(p2)),
                (attacker_b_id, AttackTarget::Player(p2)),
            ],
            enlist_choices: vec![],
        },
    )
    .expect("DeclareAttackers with two attackers should succeed");

    // Both AbilityTriggered events emitted.
    let triggered_events: Vec<_> = events
        .iter()
        .filter(|e| matches!(e, GameEvent::AbilityTriggered { .. }))
        .collect();
    assert_eq!(
        triggered_events.len(),
        2,
        "exactly 2 AbilityTriggered events expected — one per attacking creature (CR 508.1m)"
    );

    // Both triggers should be on the stack.
    assert!(
        state.stack_objects.len() >= 2,
        "both attack triggers should be on the stack (got {})",
        state.stack_objects.len()
    );

    // Verify both triggering creatures are represented.
    let triggered_ids: Vec<_> = events
        .iter()
        .filter_map(|e| {
            if let GameEvent::AbilityTriggered {
                source_object_id, ..
            } = e
            {
                Some(*source_object_id)
            } else {
                None
            }
        })
        .collect();
    assert!(
        triggered_ids.contains(&attacker_a_id),
        "Attacker Alpha's trigger should have fired"
    );
    assert!(
        triggered_ids.contains(&attacker_b_id),
        "Attacker Beta's trigger should have fired"
    );
}
