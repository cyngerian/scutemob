//! Ninjutsu keyword ability tests (CR 702.49).
//!
//! Ninjutsu is an activated ability that functions while the card is in a
//! player's hand. "Ninjutsu [cost]" means "[cost], Reveal this card from your
//! hand, Return an unblocked attacking creature you control to its owner's hand:
//! Put this card onto the battlefield from your hand tapped and attacking."
//! (CR 702.49a)
//!
//! Commander ninjutsu (CR 702.49d) is a variant that also functions from the
//! command zone and bypasses commander tax.
//!
//! Key rules verified:
//! - Basic swap: attacker returned to hand, ninja enters tapped and attacking (CR 702.49a).
//! - Ninja attacks the same target as the returned attacker (CR 702.49c).
//! - Ninja is NOT declared as an attacker; AttackersDeclared does not fire (CR 508.3a, 508.4).
//! - Timing restriction: only during DeclareBlockers, FirstStrikeDamage, CombatDamage, EndOfCombat (CR 702.49c).
//! - Blocked attacker cannot be targeted (CR 702.49c).
//! - Ninja not in hand is rejected (CR 702.49a).
//! - Ninja leaving hand before resolution: ability does nothing (CR 400.7).
//! - Returns to OWNER's hand, not controller (CR 702.49a).
//! - Ninja deals combat damage normally after entering.
//! - Split second blocks ninjutsu (CR 702.61a).
//! - 4-player multiplayer scenario.
//! - Commander ninjutsu from command zone; no commander tax increment (CR 702.49d).

use mtg_engine::{
    process_command, AbilityDefinition, AttackTarget, CardDefinition, CardId, CardRegistry,
    CardType, CombatState, Command, GameEvent, GameState, GameStateBuilder, KeywordAbility,
    ManaColor, ManaCost, ObjectId, ObjectSpec, PlayerId, Step, TypeLine, ZoneId,
};

// ── Helpers ───────────────────────────────────────────────────────────────────

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

fn find_object(state: &GameState, name: &str) -> ObjectId {
    state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{}' not found", name))
}

fn find_in_zone(state: &GameState, name: &str, zone: ZoneId) -> Option<ObjectId> {
    state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name && obj.zone == zone)
        .map(|(id, _)| *id)
}

fn on_battlefield(state: &GameState, name: &str) -> bool {
    find_in_zone(state, name, ZoneId::Battlefield).is_some()
}

fn in_hand(state: &GameState, name: &str, owner: PlayerId) -> bool {
    find_in_zone(state, name, ZoneId::Hand(owner)).is_some()
}

/// Pass priority for all listed players once.
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

/// Test ninja: Creature {2}{U} 2/2, Ninjutsu {U}.
fn ninja_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("test-ninja".to_string()),
        name: "Test Ninja".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            blue: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        power: Some(2),
        toughness: Some(2),
        oracle_text: "Ninjutsu {U}".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Ninjutsu),
            AbilityDefinition::Ninjutsu {
                cost: ManaCost {
                    blue: 1,
                    ..Default::default()
                },
            },
        ],
        ..Default::default()
    }
}

/// Test ninja with commander ninjutsu.
fn commander_ninja_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("commander-ninja".to_string()),
        name: "Commander Ninja".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            blue: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        power: Some(2),
        toughness: Some(2),
        oracle_text: "Commander ninjutsu {U}".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::CommanderNinjutsu),
            AbilityDefinition::CommanderNinjutsu {
                cost: ManaCost {
                    blue: 1,
                    ..Default::default()
                },
            },
        ],
        ..Default::default()
    }
}

/// ObjectSpec for the test ninja in a player's hand.
/// Uses ObjectSpec::creature to ensure power/toughness (2/2) are set properly
/// so combat damage works when the ninja enters the battlefield via ninjutsu.
fn ninja_in_hand(owner: PlayerId) -> ObjectSpec {
    ObjectSpec::creature(owner, "Test Ninja", 2, 2)
        .in_zone(ZoneId::Hand(owner))
        .with_card_id(CardId("test-ninja".to_string()))
        .with_keyword(KeywordAbility::Ninjutsu)
}

/// Set up a CombatState with `attacker_name` attacking `defender`.
/// The attacker must already be on the battlefield.
fn setup_combat(state: &mut GameState, attacker_name: &str, defender: PlayerId) {
    let attacker_id = find_object(state, attacker_name);
    let active = state.turn.active_player;
    state.combat = Some({
        let mut cs = CombatState::new(active);
        cs.attackers
            .insert(attacker_id, AttackTarget::Player(defender));
        cs
    });
}

// ── Test 1: Basic swap ────────────────────────────────────────────────────────

#[test]
/// CR 702.49a — Activate ninjutsu: pay {U}, attacker returns to hand, ninja
/// enters the battlefield tapped and attacking the same target.
fn test_ninjutsu_basic_swap() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![ninja_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(ObjectSpec::creature(p1, "Foot Soldier", 2, 2))
        .object(ninja_in_hand(p1))
        .active_player(p1)
        .at_step(Step::DeclareBlockers)
        .build()
        .unwrap();

    // Set up combat: Foot Soldier attacking p2, no blockers.
    setup_combat(&mut state, "Foot Soldier", p2);

    let attacker_id = find_object(&state, "Foot Soldier");
    let ninja_id = find_object(&state, "Test Ninja");

    // Give p1 {U} mana.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Blue, 1);
    state.turn.priority_holder = Some(p1);

    // Activate ninjutsu: return Foot Soldier, put Test Ninja on stack.
    let (state, activate_events) = process_command(
        state,
        Command::ActivateNinjutsu {
            player: p1,
            ninja_card: ninja_id,
            attacker_to_return: attacker_id,
        },
    )
    .expect("ActivateNinjutsu should succeed");

    // AbilityActivated should be emitted.
    assert!(
        activate_events
            .iter()
            .any(|e| matches!(e, GameEvent::AbilityActivated { player, .. } if *player == p1)),
        "CR 702.49a: AbilityActivated event expected"
    );

    // NinjutsuAbility is on the stack.
    assert_eq!(
        state.stack_objects.len(),
        1,
        "CR 702.49a: NinjutsuAbility should be on stack"
    );

    // Foot Soldier returned to p1's hand as cost.
    assert!(
        in_hand(&state, "Foot Soldier", p1),
        "CR 702.49a: returned attacker should be in owner's hand"
    );

    // Resolve by passing priority for all players.
    let (state, _) = pass_all(state, &[p1, p2]);

    // Stack should be empty.
    assert!(
        state.stack_objects.is_empty(),
        "Stack should be empty after resolution"
    );

    // Test Ninja should be on the battlefield.
    assert!(
        on_battlefield(&state, "Test Ninja"),
        "CR 702.49a: ninja should be on the battlefield after resolution"
    );

    // Test Ninja should be tapped (CR 702.49a: "tapped and attacking").
    let ninja_bf_id = find_in_zone(&state, "Test Ninja", ZoneId::Battlefield)
        .expect("Test Ninja should be on battlefield");
    assert!(
        state.objects.get(&ninja_bf_id).unwrap().status.tapped,
        "CR 702.49a: ninja enters tapped"
    );

    // Test Ninja is registered in combat.attackers targeting p2.
    let combat = state.combat.as_ref().expect("combat state should exist");
    assert!(
        combat
            .attackers
            .get(&ninja_bf_id)
            .map(|t| matches!(t, AttackTarget::Player(pid) if *pid == p2))
            .unwrap_or(false),
        "CR 702.49c: ninja should be attacking p2 in combat state"
    );

    // Foot Soldier is still in hand.
    assert!(
        in_hand(&state, "Foot Soldier", p1),
        "CR 702.49a: attacker stays in hand after ability resolves"
    );
}

// ── Test 2: Ninja attacks the same target ─────────────────────────────────────

#[test]
/// CR 702.49c — In a 4-player game, if the returned attacker was targeting P3,
/// the ninja inherits that target and attacks P3 specifically.
fn test_ninjutsu_ninja_attacks_same_target() {
    let p1 = p(1);
    let p2 = p(2);
    let p3 = p(3);
    let p4 = p(4);

    let registry = CardRegistry::new(vec![ninja_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .add_player(p4)
        .with_registry(registry)
        .object(ObjectSpec::creature(p1, "Scout", 1, 1))
        .object(ninja_in_hand(p1))
        .active_player(p1)
        .at_step(Step::DeclareBlockers)
        .build()
        .unwrap();

    // Scout attacking P3 (not P2).
    let scout_id = find_object(&state, "Scout");
    state.combat = Some({
        let mut cs = CombatState::new(p1);
        cs.attackers.insert(scout_id, AttackTarget::Player(p3));
        cs
    });

    let ninja_id = find_object(&state, "Test Ninja");
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Blue, 1);
    state.turn.priority_holder = Some(p1);

    let (state, _) = process_command(
        state,
        Command::ActivateNinjutsu {
            player: p1,
            ninja_card: ninja_id,
            attacker_to_return: scout_id,
        },
    )
    .expect("ActivateNinjutsu should succeed");

    // Resolve.
    let (state, _) = pass_all(state, &[p1, p2, p3, p4]);

    // Ninja should be attacking P3 (the inherited target).
    let ninja_bf_id = find_in_zone(&state, "Test Ninja", ZoneId::Battlefield)
        .expect("Test Ninja should be on battlefield");
    let combat = state.combat.as_ref().expect("combat state should exist");
    assert!(
        combat
            .attackers
            .get(&ninja_bf_id)
            .map(|t| matches!(t, AttackTarget::Player(pid) if *pid == p3))
            .unwrap_or(false),
        "CR 702.49c: ninja must attack P3, the target inherited from the returned attacker"
    );
}

// ── Test 3: Ninja is NOT declared as attacker ─────────────────────────────────

#[test]
/// CR 508.3a, 508.4 — The ninja is put onto the battlefield attacking but was
/// never declared as an attacker. AttackersDeclared event should NOT be emitted
/// during ninjutsu resolution. This means SelfAttacks triggers do not fire.
fn test_ninjutsu_not_declared_as_attacker() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![ninja_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(ObjectSpec::creature(p1, "Scout", 1, 1))
        .object(ninja_in_hand(p1))
        .active_player(p1)
        .at_step(Step::DeclareBlockers)
        .build()
        .unwrap();

    setup_combat(&mut state, "Scout", p2);

    let scout_id = find_object(&state, "Scout");
    let ninja_id = find_object(&state, "Test Ninja");
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Blue, 1);
    state.turn.priority_holder = Some(p1);

    let (state, _activate_events) = process_command(
        state,
        Command::ActivateNinjutsu {
            player: p1,
            ninja_card: ninja_id,
            attacker_to_return: scout_id,
        },
    )
    .expect("ActivateNinjutsu should succeed");

    // Collect all events during resolution.
    let (_, resolve_events) = pass_all(state, &[p1, p2]);

    // AttackersDeclared MUST NOT be emitted during ninjutsu resolution.
    // (CR 508.4: "Such creatures are 'attacking' but, for the purposes of trigger
    // events and effects, they never 'attacked.'")
    assert!(
        !resolve_events
            .iter()
            .any(|e| matches!(e, GameEvent::AttackersDeclared { .. })),
        "CR 508.3a/508.4: AttackersDeclared must NOT fire when ninja enters via ninjutsu"
    );
}

// ── Test 4: Wrong step rejected ───────────────────────────────────────────────

#[test]
/// CR 702.49c — Ninjutsu can only be activated during DeclareBlockers,
/// FirstStrikeDamage, CombatDamage, or EndOfCombat steps. Activating during
/// DeclareAttackers, BeginningOfCombat, or main phase is rejected.
fn test_ninjutsu_wrong_step_rejected() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![ninja_def()]);

    // Test 1: DeclareAttackers step (too early — blockers not declared yet).
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry.clone())
        .object(ObjectSpec::creature(p1, "Scout", 1, 1))
        .object(ninja_in_hand(p1))
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    setup_combat(&mut state, "Scout", p2);
    let scout_id = find_object(&state, "Scout");
    let ninja_id = find_object(&state, "Test Ninja");
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Blue, 1);
    state.turn.priority_holder = Some(p1);

    let result = process_command(
        state.clone(),
        Command::ActivateNinjutsu {
            player: p1,
            ninja_card: ninja_id,
            attacker_to_return: scout_id,
        },
    );
    assert!(
        result.is_err(),
        "CR 702.49c: ninjutsu cannot be activated during DeclareAttackers"
    );

    // Test 2: PreCombatMain step.
    let mut state2 = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry.clone())
        .object(ObjectSpec::creature(p1, "Scout", 1, 1))
        .object(ninja_in_hand(p1))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let scout_id2 = find_object(&state2, "Scout");
    let ninja_id2 = find_object(&state2, "Test Ninja");
    state2
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Blue, 1);
    state2.turn.priority_holder = Some(p1);

    let result2 = process_command(
        state2,
        Command::ActivateNinjutsu {
            player: p1,
            ninja_card: ninja_id2,
            attacker_to_return: scout_id2,
        },
    );
    assert!(
        result2.is_err(),
        "CR 702.49c: ninjutsu cannot be activated during main phase"
    );

    // Test 3: BeginningOfCombat step.
    let registry3 = CardRegistry::new(vec![ninja_def()]);
    let mut state3 = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry3)
        .object(ObjectSpec::creature(p1, "Scout", 1, 1))
        .object(ninja_in_hand(p1))
        .active_player(p1)
        .at_step(Step::BeginningOfCombat)
        .build()
        .unwrap();

    setup_combat(&mut state3, "Scout", p2);
    let scout_id3 = find_object(&state3, "Scout");
    let ninja_id3 = find_object(&state3, "Test Ninja");
    state3
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Blue, 1);
    state3.turn.priority_holder = Some(p1);

    let result3 = process_command(
        state3,
        Command::ActivateNinjutsu {
            player: p1,
            ninja_card: ninja_id3,
            attacker_to_return: scout_id3,
        },
    );
    assert!(
        result3.is_err(),
        "CR 702.49c: ninjutsu cannot be activated during BeginningOfCombat"
    );
}

// ── Test 5: Blocked attacker rejected ─────────────────────────────────────────

#[test]
/// CR 702.49c — Ninjutsu requires an UNBLOCKED attacker. A blocked attacker
/// cannot be targeted.
fn test_ninjutsu_blocked_attacker_rejected() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![ninja_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(ObjectSpec::creature(p1, "Attacker", 2, 2))
        .object(ObjectSpec::creature(p2, "Blocker", 2, 2))
        .object(ninja_in_hand(p1))
        .active_player(p1)
        .at_step(Step::DeclareBlockers)
        .build()
        .unwrap();

    let attacker_id = find_object(&state, "Attacker");
    let blocker_id = find_object(&state, "Blocker");
    let ninja_id = find_object(&state, "Test Ninja");

    // Set up combat: Attacker is attacking and HAS a blocker declared.
    state.combat = Some({
        let mut cs = CombatState::new(p1);
        cs.attackers.insert(attacker_id, AttackTarget::Player(p2));
        // Add a blocker for Attacker. Also record in blocked_attackers so
        // is_blocked() returns true (CR 509.1h — checked by ninjutsu enforcement).
        cs.blockers.insert(blocker_id, attacker_id);
        cs.blocked_attackers.insert(attacker_id);
        cs
    });

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Blue, 1);
    state.turn.priority_holder = Some(p1);

    let result = process_command(
        state,
        Command::ActivateNinjutsu {
            player: p1,
            ninja_card: ninja_id,
            attacker_to_return: attacker_id,
        },
    );
    assert!(
        result.is_err(),
        "CR 702.49c: blocked attacker cannot be targeted for ninjutsu"
    );
}

// ── Test 6: Ninja not in hand rejected ────────────────────────────────────────

#[test]
/// CR 702.49a — The ninja must be in the player's hand. If it's on the
/// battlefield already, activation is rejected.
fn test_ninjutsu_ninja_not_in_hand_rejected() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![ninja_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(ObjectSpec::creature(p1, "Scout", 1, 1))
        // Ninja is on the graveyard -- NOT in hand.
        .object(
            ObjectSpec::card(p1, "Test Ninja")
                .in_zone(ZoneId::Graveyard(p1))
                .with_card_id(CardId("test-ninja".to_string()))
                .with_keyword(KeywordAbility::Ninjutsu),
        )
        .active_player(p1)
        .at_step(Step::DeclareBlockers)
        .build()
        .unwrap();

    setup_combat(&mut state, "Scout", p2);
    let scout_id = find_object(&state, "Scout");
    let ninja_id = find_object(&state, "Test Ninja");
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Blue, 1);
    state.turn.priority_holder = Some(p1);

    let result = process_command(
        state,
        Command::ActivateNinjutsu {
            player: p1,
            ninja_card: ninja_id,
            attacker_to_return: scout_id,
        },
    );
    assert!(
        result.is_err(),
        "CR 702.49a: ninja in graveyard (not in hand) should be rejected"
    );
}

// ── Test 7: Ninja leaves hand before resolution ───────────────────────────────

#[test]
/// CR 400.7, ruling 2021-03-19 — The attacker is returned to hand as a cost
/// (immediately). If the ninja leaves hand before the ability resolves, the
/// ability does nothing; the ninja does NOT enter the battlefield.
fn test_ninjutsu_ninja_leaves_hand_before_resolution() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![ninja_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(ObjectSpec::creature(p1, "Scout", 1, 1))
        .object(ninja_in_hand(p1))
        .active_player(p1)
        .at_step(Step::DeclareBlockers)
        .build()
        .unwrap();

    setup_combat(&mut state, "Scout", p2);
    let scout_id = find_object(&state, "Scout");
    let ninja_id = find_object(&state, "Test Ninja");
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Blue, 1);
    state.turn.priority_holder = Some(p1);

    // Activate ninjutsu: attacker returned to hand as cost.
    let (mut state, _) = process_command(
        state,
        Command::ActivateNinjutsu {
            player: p1,
            ninja_card: ninja_id,
            attacker_to_return: scout_id,
        },
    )
    .expect("ActivateNinjutsu should succeed");

    // Manually move the ninja from hand to graveyard (simulate discard).
    let ninja_hand_id = find_in_zone(&state, "Test Ninja", ZoneId::Hand(p1))
        .expect("Test Ninja should be in hand (it was not moved as a cost)");
    let (_, _) = state
        .move_object_to_zone(ninja_hand_id, ZoneId::Graveyard(p1))
        .expect("move to graveyard should succeed");

    // The ability is still on the stack.
    assert_eq!(
        state.stack_objects.len(),
        1,
        "ability should still be on stack"
    );

    // Ninja not in hand anymore.
    assert!(
        !in_hand(&state, "Test Ninja", p1),
        "Test Ninja should not be in hand after manual move"
    );

    // Resolve: ability does nothing because ninja is no longer in expected zone.
    let (state, _) = pass_all(state, &[p1, p2]);

    // Stack should be empty.
    assert!(state.stack_objects.is_empty(), "stack should be empty");

    // Ninja should NOT be on the battlefield.
    assert!(
        !on_battlefield(&state, "Test Ninja"),
        "CR 400.7: ninja should not enter battlefield if it left hand before resolution"
    );
}

// ── Test 8: Returns to owner's hand, not controller ───────────────────────────

#[test]
/// CR 702.49a — "Return an unblocked attacking creature you control to its
/// OWNER's hand." In multiplayer, if P1 controls P2's creature, ninjutsu
/// returns it to P2's hand (the owner).
fn test_ninjutsu_returns_to_owner_not_controller() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![ninja_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        // Creature owned by P2 but controlled by P1 (theft scenario).
        .object(
            ObjectSpec::creature(p2, "Stolen Creature", 2, 2), // will set controller to p1 below
        )
        .object(ninja_in_hand(p1))
        .active_player(p1)
        .at_step(Step::DeclareBlockers)
        .build()
        .unwrap();

    // Set controller of Stolen Creature to p1.
    let stolen_id = find_object(&state, "Stolen Creature");
    if let Some(obj) = state.objects.get_mut(&stolen_id) {
        obj.controller = p1;
    }

    // Set up combat: Stolen Creature attacking p2 under p1's control.
    state.combat = Some({
        let mut cs = CombatState::new(p1);
        cs.attackers.insert(stolen_id, AttackTarget::Player(p2));
        cs
    });

    let ninja_id = find_object(&state, "Test Ninja");
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Blue, 1);
    state.turn.priority_holder = Some(p1);

    let (state, _) = process_command(
        state,
        Command::ActivateNinjutsu {
            player: p1,
            ninja_card: ninja_id,
            attacker_to_return: stolen_id,
        },
    )
    .expect("ActivateNinjutsu should succeed");

    // Stolen Creature must go to P2's hand (owner), NOT P1's hand.
    assert!(
        in_hand(&state, "Stolen Creature", p2),
        "CR 702.49a: stolen creature must return to its OWNER's hand (p2), not controller's (p1)"
    );
    assert!(
        !in_hand(&state, "Stolen Creature", p1),
        "CR 702.49a: stolen creature must NOT go to controller's hand (p1)"
    );
}

// ── Test 9: Ninja deals combat damage ─────────────────────────────────────────

#[test]
/// CR 702.49c — The ninja enters the battlefield attacking. After resolution,
/// it deals combat damage normally in the CombatDamage step.
fn test_ninjutsu_combat_damage() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![ninja_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(ObjectSpec::creature(p1, "Scout", 1, 1))
        .object(ninja_in_hand(p1))
        .active_player(p1)
        .at_step(Step::DeclareBlockers)
        .build()
        .unwrap();

    setup_combat(&mut state, "Scout", p2);
    let scout_id = find_object(&state, "Scout");
    let ninja_id = find_object(&state, "Test Ninja");
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Blue, 1);
    state.turn.priority_holder = Some(p1);

    // Activate ninjutsu.
    let (state, _) = process_command(
        state,
        Command::ActivateNinjutsu {
            player: p1,
            ninja_card: ninja_id,
            attacker_to_return: scout_id,
        },
    )
    .expect("ActivateNinjutsu should succeed");

    // Resolve the ability (pass priority for both players).
    let (state, _) = pass_all(state, &[p1, p2]);

    // Ninja is on battlefield attacking p2.
    assert!(
        on_battlefield(&state, "Test Ninja"),
        "ninja should be on battlefield"
    );

    // Pass priority again to advance to CombatDamage step.
    // This resolves the DeclareBlockers step (no blockers were declared for the ninja).
    let (state, _) = pass_all(state, &[p1, p2]);

    // Now we should be in CombatDamage. Pass again to trigger damage assignment.
    let (state, _) = pass_all(state, &[p1, p2]);

    // P2 should have lost 2 life (ninja is 2/2).
    let p2_life = state.players.get(&p2).map(|ps| ps.life_total).unwrap_or(40);
    assert_eq!(
        p2_life, 38,
        "CR 702.49c: 2/2 ninja attacks unblocked, p2 should be at 38 life"
    );
}

// ── Test 10: Split second blocks ninjutsu ─────────────────────────────────────

#[test]
/// CR 702.61a — Split second prevents all activated abilities while a spell
/// with split second is on the stack.
fn test_ninjutsu_split_second_blocks() {
    use mtg_engine::StackObjectKind;

    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![ninja_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(ObjectSpec::creature(p1, "Scout", 1, 1))
        .object(ninja_in_hand(p1))
        .active_player(p1)
        .at_step(Step::DeclareBlockers)
        .build()
        .unwrap();

    setup_combat(&mut state, "Scout", p2);
    let scout_id = find_object(&state, "Scout");
    let ninja_id = find_object(&state, "Test Ninja");

    // Place a fake split-second spell on the stack.
    // We set it up using a StackObject directly.
    let fake_spell_id = state.next_object_id();
    use mtg_engine::StackObject;
    let stack_obj = StackObject {
        id: fake_spell_id,
        controller: p1,
        kind: StackObjectKind::Spell {
            source_object: scout_id,
        },
        targets: vec![],
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
        cast_with_jump_start: false,
        cast_with_aftermath: false,
        was_dashed: false,
        was_blitzed: false,
        was_plotted: false,
        was_prototyped: false,
        was_impended: false,
        was_bargained: false,
        was_surged: false,
        was_casualty_paid: false,
        // CR 702.148a: test objects are not cleave casts.
        was_cleaved: false,
        // CR 715.3d: test objects are not adventure casts.
        was_cast_as_adventure: false,
        // CR 702.47a: test objects have no spliced effects.
        spliced_effects: vec![],
        spliced_card_ids: vec![],
        modes_chosen: vec![],
        x_value: 0,
        evidence_collected: false,
        is_cast_transformed: false,
        additional_costs: vec![],
    };
    state.stack_objects.push_back(stack_obj);

    // Grant the scout (used as the "source object") split second keyword so
    // has_split_second_on_stack() returns true.
    if let Some(obj) = state.objects.get_mut(&scout_id) {
        obj.characteristics
            .keywords
            .insert(KeywordAbility::SplitSecond);
    }

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Blue, 1);
    state.turn.priority_holder = Some(p1);

    let result = process_command(
        state,
        Command::ActivateNinjutsu {
            player: p1,
            ninja_card: ninja_id,
            attacker_to_return: scout_id,
        },
    );
    assert!(
        result.is_err(),
        "CR 702.61a: split second prevents ninjutsu activation"
    );
}

// ── Test 11: 4-player multiplayer ─────────────────────────────────────────────

#[test]
/// CR 702.49c — In a 4-player game, P1 attacks P3 with an unblocked creature.
/// After ninjutsu, the ninja attacks P3. P2 and P4 life totals are unchanged.
fn test_ninjutsu_multiplayer_four_player() {
    let p1 = p(1);
    let p2 = p(2);
    let p3 = p(3);
    let p4 = p(4);

    let registry = CardRegistry::new(vec![ninja_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .add_player(p4)
        .with_registry(registry)
        .object(ObjectSpec::creature(p1, "Scout", 1, 1))
        .object(ninja_in_hand(p1))
        .active_player(p1)
        .at_step(Step::DeclareBlockers)
        .build()
        .unwrap();

    // Scout attacking P3.
    let scout_id = find_object(&state, "Scout");
    state.combat = Some({
        let mut cs = CombatState::new(p1);
        cs.attackers.insert(scout_id, AttackTarget::Player(p3));
        cs
    });

    let ninja_id = find_object(&state, "Test Ninja");
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Blue, 1);
    state.turn.priority_holder = Some(p1);

    let (state, _) = process_command(
        state,
        Command::ActivateNinjutsu {
            player: p1,
            ninja_card: ninja_id,
            attacker_to_return: scout_id,
        },
    )
    .expect("ActivateNinjutsu should succeed");

    // Resolve: all four players pass.
    let (state, _) = pass_all(state, &[p1, p2, p3, p4]);

    // Ninja attacking P3.
    let ninja_bf_id = find_in_zone(&state, "Test Ninja", ZoneId::Battlefield)
        .expect("ninja should be on battlefield");
    let combat = state.combat.as_ref().expect("combat state should exist");
    assert!(
        combat
            .attackers
            .get(&ninja_bf_id)
            .map(|t| matches!(t, AttackTarget::Player(pid) if *pid == p3))
            .unwrap_or(false),
        "CR 702.49c: ninja should be attacking P3"
    );

    // P2 and P4 life totals should be unchanged (40 each).
    let p2_life = state.players.get(&p2).map(|ps| ps.life_total).unwrap_or(0);
    let p4_life = state.players.get(&p4).map(|ps| ps.life_total).unwrap_or(0);
    assert_eq!(p2_life, 40, "P2 should not have lost life");
    assert_eq!(p4_life, 40, "P4 should not have lost life");
}

// ── Test 12: Commander ninjutsu from command zone ─────────────────────────────

#[test]
/// CR 702.49d — Commander ninjutsu can be activated from the command zone.
/// It is an activated ability, NOT a spell cast. Commander tax is NOT
/// incremented. See ruling 2020-11-10 (Yuriko).
fn test_commander_ninjutsu_from_command_zone() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![commander_ninja_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(ObjectSpec::creature(p1, "Scout", 1, 1))
        // Commander Ninja is in the command zone (not hand).
        // Use creature() to ensure power/toughness are set.
        .object(
            ObjectSpec::creature(p1, "Commander Ninja", 2, 2)
                .in_zone(ZoneId::Command(p1))
                .with_card_id(CardId("commander-ninja".to_string()))
                .with_keyword(KeywordAbility::CommanderNinjutsu),
        )
        .active_player(p1)
        .at_step(Step::DeclareBlockers)
        .build()
        .unwrap();

    setup_combat(&mut state, "Scout", p2);

    let scout_id = find_object(&state, "Scout");
    let ninja_id = find_object(&state, "Commander Ninja");

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Blue, 1);
    state.turn.priority_holder = Some(p1);

    // Verify the ninja is in the command zone.
    assert_eq!(
        state.objects.get(&ninja_id).map(|o| o.zone),
        Some(ZoneId::Command(p1)),
        "Commander Ninja should be in command zone before activation"
    );

    // Record commander tax before activation (should be empty).
    let tax_before = state
        .players
        .get(&p1)
        .map(|ps| {
            ps.commander_tax
                .get(&CardId("commander-ninja".to_string()))
                .copied()
                .unwrap_or(0)
        })
        .unwrap_or(0);
    assert_eq!(tax_before, 0, "commander tax should start at 0");

    // Activate commander ninjutsu from command zone.
    let (state, activate_events) = process_command(
        state,
        Command::ActivateNinjutsu {
            player: p1,
            ninja_card: ninja_id,
            attacker_to_return: scout_id,
        },
    )
    .expect("ActivateNinjutsu (commander ninjutsu) should succeed from command zone");

    assert!(
        activate_events
            .iter()
            .any(|e| matches!(e, GameEvent::AbilityActivated { player, .. } if *player == p1)),
        "CR 702.49d: AbilityActivated expected for commander ninjutsu"
    );

    // Resolve.
    let (state, _) = pass_all(state, &[p1, p2]);

    // Commander Ninja should be on the battlefield.
    assert!(
        on_battlefield(&state, "Commander Ninja"),
        "CR 702.49d: commander ninja should be on the battlefield"
    );

    // Commander tax must NOT be incremented (this is an activated ability, not a cast).
    let tax_after = state
        .players
        .get(&p1)
        .map(|ps| {
            ps.commander_tax
                .get(&CardId("commander-ninja".to_string()))
                .copied()
                .unwrap_or(0)
        })
        .unwrap_or(0);
    assert_eq!(
        tax_after, 0,
        "CR 702.49d: commander ninjutsu bypasses commander tax (ruling 2020-11-10)"
    );

    // Scout returned to hand as cost.
    assert!(
        in_hand(&state, "Scout", p1),
        "Scout should be in p1's hand after being returned as ninjutsu cost"
    );
}
