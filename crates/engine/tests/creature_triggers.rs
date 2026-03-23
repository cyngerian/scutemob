//! Controller-filtered creature trigger tests (PB-23).
//!
//! Tests for:
//! - `WheneverCreatureDies` with controller filtering (None, You, Opponent)
//! - `WheneverCreatureYouControlAttacks`
//! - `WheneverCreatureYouControlDealsCombatDamageToPlayer`
//!
//! CR Rules covered:
//! - CR 603.10a: Death triggers look back in time; controller from pre-death state.
//! - CR 508.1m: Attack triggers fire after attackers are declared.
//! - CR 510.3a: Combat damage triggers fire after damage is dealt (NOT look-back).
//! - CR 603.2: Trigger fires once per event occurrence.
//! - CR 603.2c: One trigger fires per attacking creature.

use mtg_engine::{
    process_command, AttackTarget, CardId, CardRegistry, Command, Effect, EffectAmount, GameEvent,
    GameState, GameStateBuilder, ObjectId, ObjectSpec, PlayerId, PlayerTarget, Step, TriggerEvent,
    TriggeredAbilityDef, ZoneId,
};

// ── Helpers ───────────────────────────────────────────────────────────────────

fn find_obj(state: &GameState, name: &str) -> ObjectId {
    state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{}' not found", name))
}

fn hand_count(state: &GameState, player: PlayerId) -> usize {
    state
        .objects
        .values()
        .filter(|obj| obj.zone == ZoneId::Hand(player))
        .count()
}

fn pass_all(state: GameState, players: &[PlayerId]) -> (GameState, Vec<GameEvent>) {
    let mut all_events = Vec::new();
    let mut current = state;
    for &pl in players {
        let (s, ev) = process_command(current, Command::PassPriority { player: pl })
            .unwrap_or_else(|e| panic!("PassPriority by {:?} failed: {:?}", pl, e));
        current = s;
        all_events.extend(ev);
    }
    (current, all_events)
}

/// Build a triggered ability TriggeredAbilityDef with AnyCreatureDies filtered for "you control".
fn death_trigger_you_draw(
    trigger_event: TriggerEvent,
    controller: mtg_engine::TargetController,
) -> TriggeredAbilityDef {
    TriggeredAbilityDef {
        trigger_on: trigger_event,
        intervening_if: None,
        description: "Whenever a creature you control dies, draw a card. (CR 603.10a)".to_string(),
        effect: Some(Effect::DrawCards {
            player: PlayerTarget::Controller,
            count: EffectAmount::Fixed(1),
        }),
        etb_filter: None,
        death_filter: Some(mtg_engine::DeathTriggerFilter {
            controller_you: matches!(controller, mtg_engine::TargetController::You),
            controller_opponent: matches!(controller, mtg_engine::TargetController::Opponent),
            exclude_self: false,
            nontoken_only: false,
        }),
        targets: vec![],
    }
}

/// Build the "draw on any creature dies" ability (no filter).
fn death_trigger_any_draw() -> TriggeredAbilityDef {
    TriggeredAbilityDef {
        trigger_on: TriggerEvent::AnyCreatureDies,
        intervening_if: None,
        description: "Whenever any creature dies, draw a card. (CR 603.10a)".to_string(),
        effect: Some(Effect::DrawCards {
            player: PlayerTarget::Controller,
            count: EffectAmount::Fixed(1),
        }),
        etb_filter: None,
        death_filter: None,
        targets: vec![],
    }
}

/// Build the "draw when creature you control attacks" ability.
fn attack_trigger_draw() -> TriggeredAbilityDef {
    TriggeredAbilityDef {
        trigger_on: TriggerEvent::AnyCreatureYouControlAttacks,
        intervening_if: None,
        description: "Whenever a creature you control attacks, draw a card. (CR 508.1m)"
            .to_string(),
        effect: Some(Effect::DrawCards {
            player: PlayerTarget::Controller,
            count: EffectAmount::Fixed(1),
        }),
        etb_filter: None,
        death_filter: None,
        targets: vec![],
    }
}

/// Build the "draw when creature you control deals combat damage to a player" ability.
fn combat_damage_trigger_draw() -> TriggeredAbilityDef {
    TriggeredAbilityDef {
        trigger_on: TriggerEvent::AnyCreatureYouControlDealsCombatDamageToPlayer,
        intervening_if: None,
        description: "Whenever a creature you control deals combat damage to a player, draw a card. (CR 510.3a)".to_string(),
        effect: Some(Effect::DrawCards {
            player: PlayerTarget::Controller,
            count: EffectAmount::Fixed(1),
        }),
        etb_filter: None,
        death_filter: None,
        targets: vec![],
    }
}

/// Add library cards so draw effects don't silently fail.
fn library_card(player: PlayerId, id: &str, name: &str) -> ObjectSpec {
    ObjectSpec::creature(player, name, 1, 1)
        .in_zone(ZoneId::Library(player))
        .with_card_id(CardId(id.to_string()))
}

// ── Tests: WheneverCreatureDies with controller filter ────────────────────────

/// CR 603.10a: "Whenever a creature you control dies" fires when your creature dies.
#[test]
fn test_whenever_creature_you_control_dies_fires_on_your_creature() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    // P1 controls a watcher with "your creature dies = draw"
    let watcher = ObjectSpec::creature(p1, "Watcher", 2, 2)
        .with_card_id(CardId("watcher".to_string()))
        .with_triggered_ability(death_trigger_you_draw(
            TriggerEvent::AnyCreatureDies,
            mtg_engine::TargetController::You,
        ));
    // P1's creature that will die
    let fodder =
        ObjectSpec::creature(p1, "Fodder", 2, 2).with_card_id(CardId("fodder".to_string()));
    // P2's attacker that will kill Fodder
    let attacker =
        ObjectSpec::creature(p2, "Attacker", 3, 3).with_card_id(CardId("attacker".to_string()));
    let lib_card = library_card(p1, "lib1", "LibCard1");

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(watcher)
        .object(fodder)
        .object(attacker)
        .object(lib_card)
        .active_player(p2)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let attacker_id = find_obj(&state, "Attacker");
    let fodder_id = find_obj(&state, "Fodder");

    // P2 declares Attacker targeting P1
    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p2,
            attackers: vec![(attacker_id, AttackTarget::Player(p1))],
            enlist_choices: vec![],
        },
    )
    .expect("DeclareAttackers");

    // Advance to DeclareBlockers
    let (state, _) = pass_all(state, &[p2, p1]);

    // P1 blocks with Fodder (dies to Attacker)
    let (state, _) = process_command(
        state,
        Command::DeclareBlockers {
            player: p1,
            blockers: vec![(fodder_id, attacker_id)],
        },
    )
    .expect("DeclareBlockers");

    let initial_hand = hand_count(&state, p1);

    // Advance through combat damage — Fodder (2/2) dies to Attacker (3/3), trigger fires
    let (state, _) = pass_all(state, &[p2, p1]);
    // Resolve trigger
    let (state, _) = pass_all(state, &[p2, p1]);

    assert!(
        hand_count(&state, p1) > initial_hand,
        "Watcher should draw when P1's own creature (Fodder) dies; hand was {} now {}",
        initial_hand,
        hand_count(&state, p1)
    );
}

/// CR 603.10a: "Whenever a creature you control dies" does NOT fire for opponent's creatures.
#[test]
fn test_whenever_creature_you_control_dies_ignores_opponent_creature() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    // P1 has a watcher with controller_you filter
    let watcher = ObjectSpec::creature(p1, "Watcher", 2, 2)
        .with_card_id(CardId("watcher".to_string()))
        .with_triggered_ability(death_trigger_you_draw(
            TriggerEvent::AnyCreatureDies,
            mtg_engine::TargetController::You,
        ));
    // P1's attacker (will kill P2's creature)
    let attacker =
        ObjectSpec::creature(p1, "Attacker", 3, 3).with_card_id(CardId("attacker".to_string()));
    // P2's blocker (will die)
    let blocker =
        ObjectSpec::creature(p2, "Blocker", 2, 2).with_card_id(CardId("blocker".to_string()));
    let lib_card = library_card(p1, "lib1", "LibCard1");

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(watcher)
        .object(attacker)
        .object(blocker)
        .object(lib_card)
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let attacker_id = find_obj(&state, "Attacker");
    let blocker_id = find_obj(&state, "Blocker");

    // P1 declares Attacker targeting P2
    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(attacker_id, AttackTarget::Player(p2))],
            enlist_choices: vec![],
        },
    )
    .expect("DeclareAttackers");

    let (state, _) = pass_all(state, &[p1, p2]);

    // P2 blocks with Blocker (dies)
    let (state, _) = process_command(
        state,
        Command::DeclareBlockers {
            player: p2,
            blockers: vec![(blocker_id, attacker_id)],
        },
    )
    .expect("DeclareBlockers");

    let initial_hand = hand_count(&state, p1);

    // Combat damage — P2's Blocker dies (not P1's creature)
    let (state, _) = pass_all(state, &[p1, p2]);
    let (state, _) = pass_all(state, &[p1, p2]);

    // Watcher must NOT fire — opponent's creature died, not P1's
    assert_eq!(
        hand_count(&state, p1),
        initial_hand,
        "Watcher (controller_you filter) must NOT fire when opponent's creature dies"
    );
}

/// CR 603.10a: Global "whenever any creature dies" fires on any creature's death.
#[test]
fn test_whenever_any_creature_dies_fires_on_any() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    // P1 has a Blood Artist-style watcher (no controller filter)
    let artist = ObjectSpec::creature(p1, "Artist", 1, 1)
        .with_card_id(CardId("artist".to_string()))
        .with_triggered_ability(death_trigger_any_draw());
    let attacker =
        ObjectSpec::creature(p1, "Attacker", 3, 3).with_card_id(CardId("attacker".to_string()));
    let blocker =
        ObjectSpec::creature(p2, "Blocker", 2, 2).with_card_id(CardId("blocker".to_string()));
    let lib_card = library_card(p1, "lib1", "LibCard1");

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(artist)
        .object(attacker)
        .object(blocker)
        .object(lib_card)
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let attacker_id = find_obj(&state, "Attacker");
    let blocker_id = find_obj(&state, "Blocker");

    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(attacker_id, AttackTarget::Player(p2))],
            enlist_choices: vec![],
        },
    )
    .expect("DeclareAttackers");

    let (state, _) = pass_all(state, &[p1, p2]);

    let (state, _) = process_command(
        state,
        Command::DeclareBlockers {
            player: p2,
            blockers: vec![(blocker_id, attacker_id)],
        },
    )
    .expect("DeclareBlockers");

    let initial_hand = hand_count(&state, p1);

    // P2's Blocker dies to Attacker
    let (state, _) = pass_all(state, &[p1, p2]);
    // Resolve trigger
    let (state, _) = pass_all(state, &[p1, p2]);

    assert!(
        hand_count(&state, p1) > initial_hand,
        "Artist (any creature dies) should fire when opponent's creature (Blocker) dies"
    );
}

/// CR 603.10a: "Whenever a creature an opponent controls dies" fires for opponent's creatures.
#[test]
fn test_whenever_creature_opponent_controls_dies_fires() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    // P1 has a watcher with controller_opponent filter
    let watcher = ObjectSpec::creature(p1, "OppWatcher", 2, 2)
        .with_card_id(CardId("opp-watcher".to_string()))
        .with_triggered_ability(death_trigger_you_draw(
            TriggerEvent::AnyCreatureDies,
            mtg_engine::TargetController::Opponent,
        ));
    let attacker =
        ObjectSpec::creature(p1, "Attacker", 3, 3).with_card_id(CardId("attacker".to_string()));
    let blocker =
        ObjectSpec::creature(p2, "Blocker", 2, 2).with_card_id(CardId("blocker".to_string()));
    let lib_card = library_card(p1, "lib1", "LibCard1");

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(watcher)
        .object(attacker)
        .object(blocker)
        .object(lib_card)
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let attacker_id = find_obj(&state, "Attacker");
    let blocker_id = find_obj(&state, "Blocker");

    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(attacker_id, AttackTarget::Player(p2))],
            enlist_choices: vec![],
        },
    )
    .expect("DeclareAttackers");

    let (state, _) = pass_all(state, &[p1, p2]);

    let (state, _) = process_command(
        state,
        Command::DeclareBlockers {
            player: p2,
            blockers: vec![(blocker_id, attacker_id)],
        },
    )
    .expect("DeclareBlockers");

    let initial_hand = hand_count(&state, p1);

    // P2's Blocker dies
    let (state, _) = pass_all(state, &[p1, p2]);
    let (state, _) = pass_all(state, &[p1, p2]);

    assert!(
        hand_count(&state, p1) > initial_hand,
        "OppWatcher should draw when opponent's creature (Blocker) dies"
    );
}

/// CR 603.10a: "Whenever a creature an opponent controls dies" does NOT fire for your own creatures.
#[test]
fn test_whenever_creature_opponent_controls_dies_ignores_your_creature() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    // P1 has opponent-watcher (should not fire when P1's own creature dies)
    let watcher = ObjectSpec::creature(p1, "OppWatcher", 2, 2)
        .with_card_id(CardId("opp-watcher".to_string()))
        .with_triggered_ability(death_trigger_you_draw(
            TriggerEvent::AnyCreatureDies,
            mtg_engine::TargetController::Opponent,
        ));
    let fodder =
        ObjectSpec::creature(p1, "Fodder", 2, 2).with_card_id(CardId("fodder".to_string()));
    let attacker =
        ObjectSpec::creature(p2, "Attacker", 3, 3).with_card_id(CardId("attacker".to_string()));
    let lib_card = library_card(p1, "lib1", "LibCard1");

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(watcher)
        .object(fodder)
        .object(attacker)
        .object(lib_card)
        .active_player(p2)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let attacker_id = find_obj(&state, "Attacker");
    let fodder_id = find_obj(&state, "Fodder");

    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p2,
            attackers: vec![(attacker_id, AttackTarget::Player(p1))],
            enlist_choices: vec![],
        },
    )
    .expect("DeclareAttackers");

    let (state, _) = pass_all(state, &[p2, p1]);

    // P1 blocks with Fodder (dies)
    let (state, _) = process_command(
        state,
        Command::DeclareBlockers {
            player: p1,
            blockers: vec![(fodder_id, attacker_id)],
        },
    )
    .expect("DeclareBlockers");

    let initial_hand = hand_count(&state, p1);

    // P1's Fodder dies (not P2's creature)
    let (state, _) = pass_all(state, &[p2, p1]);
    let (state, _) = pass_all(state, &[p2, p1]);

    assert_eq!(
        hand_count(&state, p1),
        initial_hand,
        "OppWatcher must NOT fire when P1's own creature (Fodder) dies"
    );
}

// ── Tests: WheneverCreatureYouControlAttacks ──────────────────────────────────

/// CR 508.1m: "Whenever a creature you control attacks" fires when your creature attacks.
#[test]
fn test_whenever_creature_you_control_attacks_fires() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    // P1 has an "attack-draw" enchantment
    let enchantment = ObjectSpec::creature(p1, "AttackDraw", 0, 1)
        .in_zone(ZoneId::Battlefield)
        .with_card_id(CardId("attack-draw".to_string()))
        .with_triggered_ability(attack_trigger_draw());
    let attacker =
        ObjectSpec::creature(p1, "Attacker", 2, 2).with_card_id(CardId("attacker".to_string()));
    let lib_card = library_card(p1, "lib1", "LibCard1");

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(enchantment)
        .object(attacker)
        .object(lib_card)
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let attacker_id = find_obj(&state, "Attacker");
    let initial_hand = hand_count(&state, p1);

    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(attacker_id, AttackTarget::Player(p2))],
            enlist_choices: vec![],
        },
    )
    .expect("DeclareAttackers");

    // Trigger fires at attackers-declared; resolve it
    let (state, _) = pass_all(state, &[p1, p2]);

    assert!(
        hand_count(&state, p1) > initial_hand,
        "AttackDraw trigger should fire when P1's creature attacks; hand was {} now {}",
        initial_hand,
        hand_count(&state, p1)
    );
}

/// CR 508.1m: "Whenever a creature you control attacks" does NOT fire for opponent's attackers.
#[test]
fn test_whenever_creature_you_control_attacks_ignores_opponent() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    // P1 has the enchantment; P2 is the attacker
    let enchantment = ObjectSpec::creature(p1, "AttackDraw", 0, 1)
        .in_zone(ZoneId::Battlefield)
        .with_card_id(CardId("attack-draw".to_string()))
        .with_triggered_ability(attack_trigger_draw());
    let attacker =
        ObjectSpec::creature(p2, "Attacker", 2, 2).with_card_id(CardId("attacker".to_string()));
    let lib_card = library_card(p1, "lib1", "LibCard1");

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(enchantment)
        .object(attacker)
        .object(lib_card)
        .active_player(p2)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let attacker_id = find_obj(&state, "Attacker");
    let initial_hand = hand_count(&state, p1);

    // P2 attacks with THEIR creature
    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p2,
            attackers: vec![(attacker_id, AttackTarget::Player(p1))],
            enlist_choices: vec![],
        },
    )
    .expect("DeclareAttackers");

    let (state, _) = pass_all(state, &[p2, p1]);

    assert_eq!(
        hand_count(&state, p1),
        initial_hand,
        "AttackDraw must NOT fire when P2's creature (not P1's) attacks"
    );
}

/// CR 603.2c: "Whenever a creature you control attacks" fires once per attacking creature.
#[test]
fn test_whenever_creature_you_control_attacks_fires_per_creature() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let enchantment = ObjectSpec::creature(p1, "AttackDraw", 0, 1)
        .in_zone(ZoneId::Battlefield)
        .with_card_id(CardId("attack-draw".to_string()))
        .with_triggered_ability(attack_trigger_draw());
    let a1 = ObjectSpec::creature(p1, "A1", 2, 2).with_card_id(CardId("a1".to_string()));
    let a2 = ObjectSpec::creature(p1, "A2", 2, 2).with_card_id(CardId("a2".to_string()));
    let a3 = ObjectSpec::creature(p1, "A3", 2, 2).with_card_id(CardId("a3".to_string()));
    // 3 library cards for 3 draws
    let lib1 = library_card(p1, "lib1", "LibCard1");
    let lib2 = library_card(p1, "lib2", "LibCard2");
    let lib3 = library_card(p1, "lib3", "LibCard3");

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(enchantment)
        .object(a1)
        .object(a2)
        .object(a3)
        .object(lib1)
        .object(lib2)
        .object(lib3)
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let a1_id = find_obj(&state, "A1");
    let a2_id = find_obj(&state, "A2");
    let a3_id = find_obj(&state, "A3");
    let initial_hand = hand_count(&state, p1);

    // Attack with 3 creatures
    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![
                (a1_id, AttackTarget::Player(p2)),
                (a2_id, AttackTarget::Player(p2)),
                (a3_id, AttackTarget::Player(p2)),
            ],
            enlist_choices: vec![],
        },
    )
    .expect("DeclareAttackers");

    // Resolve 3 triggers (one per attacker)
    let (state, _) = pass_all(state, &[p1, p2]);
    let (state, _) = pass_all(state, &[p1, p2]);
    let (state, _) = pass_all(state, &[p1, p2]);

    let drawn = hand_count(&state, p1) - initial_hand;
    assert_eq!(
        drawn, 3,
        "AttackDraw should fire exactly 3 times for 3 attacking creatures, but drew {}",
        drawn
    );
}

// ── Tests: WheneverCreatureYouControlDealsCombatDamageToPlayer ────────────────

/// CR 510.3a: Combat damage trigger fires when your creature deals damage to a player.
#[test]
fn test_whenever_creature_you_control_deals_combat_damage_to_player_fires() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    // P1 has Coastal Piracy-style enchantment + unblocked attacker
    let piracy = ObjectSpec::creature(p1, "CoastalPiracy", 0, 1)
        .in_zone(ZoneId::Battlefield)
        .with_card_id(CardId("coastal-piracy".to_string()))
        .with_triggered_ability(combat_damage_trigger_draw());
    // Large attacker so it survives
    let attacker =
        ObjectSpec::creature(p1, "Attacker", 3, 3).with_card_id(CardId("attacker".to_string()));
    let lib_card = library_card(p1, "lib1", "LibCard1");

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(piracy)
        .object(attacker)
        .object(lib_card)
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let attacker_id = find_obj(&state, "Attacker");
    let initial_hand = hand_count(&state, p1);

    // P1 attacks unblocked
    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(attacker_id, AttackTarget::Player(p2))],
            enlist_choices: vec![],
        },
    )
    .expect("DeclareAttackers");

    // Advance to DeclareBlockers (no blockers) → CombatDamage → trigger fires
    let (state, _) = pass_all(state, &[p1, p2]);
    // No blockers declared — P2 passes
    let (state, _) = pass_all(state, &[p1, p2]);
    // Trigger resolves
    let (state, _) = pass_all(state, &[p1, p2]);

    assert!(
        hand_count(&state, p1) > initial_hand,
        "CoastalPiracy trigger should fire when P1's Attacker deals combat damage to P2; hand was {} now {}",
        initial_hand,
        hand_count(&state, p1)
    );
}

/// CR 510.3a: Combat damage trigger does NOT fire when opponent's creature attacks.
#[test]
fn test_whenever_creature_you_control_deals_combat_damage_ignores_opponent() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    // P1 has the enchantment; P2's creature attacks P1 unblocked
    let piracy = ObjectSpec::creature(p1, "CoastalPiracy", 0, 1)
        .in_zone(ZoneId::Battlefield)
        .with_card_id(CardId("coastal-piracy".to_string()))
        .with_triggered_ability(combat_damage_trigger_draw());
    let attacker =
        ObjectSpec::creature(p2, "Attacker", 2, 2).with_card_id(CardId("attacker".to_string()));
    let lib_card = library_card(p1, "lib1", "LibCard1");

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(piracy)
        .object(attacker)
        .object(lib_card)
        .active_player(p2)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let attacker_id = find_obj(&state, "Attacker");
    let initial_hand = hand_count(&state, p1);

    // P2's Attacker targets P1 (unblocked)
    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p2,
            attackers: vec![(attacker_id, AttackTarget::Player(p1))],
            enlist_choices: vec![],
        },
    )
    .expect("DeclareAttackers");

    let (state, _) = pass_all(state, &[p2, p1]);
    let (state, _) = pass_all(state, &[p2, p1]);
    let (state, _) = pass_all(state, &[p2, p1]);

    assert_eq!(
        hand_count(&state, p1),
        initial_hand,
        "CoastalPiracy must NOT fire when P2's creature deals combat damage to P1 (P2 controls attacker, not P1)"
    );
}
