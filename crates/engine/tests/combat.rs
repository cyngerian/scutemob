//! Combat phase tests (CR 506-511).
//!
//! Tests cover: attacker/blocker declaration, combat damage, first strike,
//! double strike, trample, deathtouch+trample, multiple blockers,
//! combat triggers, commander damage, and multiplayer combat.

use mtg_engine::{
    process_command, AttackTarget, CombatDamageTarget, Command, GameEvent, GameState,
    GameStateBuilder, GameStateError, KeywordAbility, ObjectSpec, PlayerId, Step,
};

// ---------------------------------------------------------------------------
// Helper: advance through all priority passes to reach a target step.
// Passes for each player until the step changes.
// ---------------------------------------------------------------------------

fn pass_all(state: GameState, players: &[PlayerId]) -> (GameState, Vec<GameEvent>) {
    let mut all_events = Vec::new();
    let mut current = state;
    for &p in players {
        let (s, ev) = process_command(current, Command::PassPriority { player: p })
            .unwrap_or_else(|e| panic!("PassPriority by {:?} failed: {:?}", p, e));
        current = s;
        all_events.extend(ev);
    }
    (current, all_events)
}

// ---------------------------------------------------------------------------
// Test 1: Unblocked attacker deals damage to player (CR 510.1a)
// ---------------------------------------------------------------------------

#[test]
/// CR 510.1a — unblocked creature deals combat damage equal to its power to
/// the player it's attacking.
fn test_510_unblocked_attacker_deals_damage_to_player() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(ObjectSpec::creature(p1, "Bear", 2, 2))
        .at_step(Step::DeclareAttackers)
        .active_player(p1)
        .build().unwrap();

    // Find the attacker ID.
    let attacker_id = state
        .objects
        .values()
        .find(|o| o.controller == p1)
        .unwrap()
        .id;

    // Declare attacker.
    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(attacker_id, AttackTarget::Player(p2))],
        },
    )
    .expect("declare attackers failed");

    // Both players pass through DeclareAttackers step → DeclareBlockers.
    let (state, _) = pass_all(state, &[p1, p2]);

    // Verify we're in DeclareBlockers step.
    assert_eq!(state.turn.step, Step::DeclareBlockers);

    // p2 declares no blockers.
    let (state, _) = process_command(
        state,
        Command::DeclareBlockers {
            player: p2,
            blockers: vec![],
        },
    )
    .expect("declare blockers failed");

    // Both pass → CombatDamage (no first strike).
    let (state, events) = pass_all(state, &[p1, p2]);

    // Damage should be dealt now (step advanced to CombatDamage and action executed).
    let damage_dealt = events.iter().any(|e| {
        matches!(
            e,
            GameEvent::CombatDamageDealt { assignments }
            if assignments.iter().any(|a| {
                matches!(a.target, CombatDamageTarget::Player(pid) if pid == p2)
                    && a.amount == 2
            })
        )
    });
    assert!(damage_dealt, "Expected 2 damage to be dealt to p2");

    // p2's life total should be reduced.
    let p2_life = state.players.get(&p2).unwrap().life_total;
    assert_eq!(p2_life, 38, "p2 should have 38 life after taking 2 damage");
}

// ---------------------------------------------------------------------------
// Test 2: Blocked creature does not deal player damage (CR 509.1h)
// ---------------------------------------------------------------------------

#[test]
/// CR 509.1h — once blocked, a creature remains blocked even if its blocker
/// is removed. Without trample, a blocked creature deals no player damage.
fn test_509_blocked_attacker_no_player_damage() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(ObjectSpec::creature(p1, "Attacker", 5, 5))
        .object(ObjectSpec::creature(p2, "Blocker", 1, 1))
        .at_step(Step::DeclareAttackers)
        .active_player(p1)
        .build().unwrap();

    let attacker_id = state
        .objects
        .values()
        .find(|o| o.controller == p1 && o.characteristics.name == "Attacker")
        .unwrap()
        .id;
    let blocker_id = state
        .objects
        .values()
        .find(|o| o.controller == p2)
        .unwrap()
        .id;

    // Declare attacker.
    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(attacker_id, AttackTarget::Player(p2))],
        },
    )
    .unwrap();

    let (state, _) = pass_all(state, &[p1, p2]);

    // p2 blocks with their creature.
    let (state, _) = process_command(
        state,
        Command::DeclareBlockers {
            player: p2,
            blockers: vec![(blocker_id, attacker_id)],
        },
    )
    .unwrap();

    let (state, events) = pass_all(state, &[p1, p2]);

    // p2's life should be unchanged (attacker is blocked, no trample).
    let p2_life = state.players.get(&p2).unwrap().life_total;
    assert_eq!(
        p2_life, 40,
        "Blocked attacker (no trample) should deal no player damage"
    );

    // Damage should still be dealt to the blocker.
    let blocker_took_damage = events.iter().any(|e| {
        matches!(
            e,
            GameEvent::CombatDamageDealt { assignments }
            if assignments.iter().any(|a| {
                matches!(a.target, CombatDamageTarget::Creature(id) if id == blocker_id)
                    && a.amount == 5
            })
        )
    });
    assert!(blocker_took_damage, "Blocker should have taken 5 damage");
}

// ---------------------------------------------------------------------------
// Test 3: Mutual combat damage — both creatures die (CR 510.2)
// ---------------------------------------------------------------------------

#[test]
/// CR 510.2 — combat damage is dealt simultaneously; a 3/3 blocking a 3/3
/// causes both to receive lethal damage and die as SBAs.
fn test_510_mutual_combat_damage_both_die() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(ObjectSpec::creature(p1, "Knight", 3, 3))
        .object(ObjectSpec::creature(p2, "Troll", 3, 3))
        .at_step(Step::DeclareAttackers)
        .active_player(p1)
        .build().unwrap();

    let attacker_id = state
        .objects
        .values()
        .find(|o| o.controller == p1)
        .unwrap()
        .id;
    let blocker_id = state
        .objects
        .values()
        .find(|o| o.controller == p2)
        .unwrap()
        .id;

    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(attacker_id, AttackTarget::Player(p2))],
        },
    )
    .unwrap();
    let (state, _) = pass_all(state, &[p1, p2]);

    let (state, _) = process_command(
        state,
        Command::DeclareBlockers {
            player: p2,
            blockers: vec![(blocker_id, attacker_id)],
        },
    )
    .unwrap();

    let (state, events) = pass_all(state, &[p1, p2]);

    // Both creatures should have died (SBAs after combat damage).
    let attacker_alive = state.objects.values().any(|o| o.id == attacker_id);
    let blocker_alive = state.objects.values().any(|o| o.id == blocker_id);
    assert!(!attacker_alive, "Attacker (3/3 hit by 3) should have died");
    assert!(!blocker_alive, "Blocker (3/3 hit by 3) should have died");

    // Both players' life totals should be unchanged (damage went to creatures).
    assert_eq!(state.players.get(&p1).unwrap().life_total, 40);
    assert_eq!(state.players.get(&p2).unwrap().life_total, 40);

    // CreatureDied events should have been emitted for both.
    let deaths = events
        .iter()
        .filter(|e| matches!(e, GameEvent::CreatureDied { .. }))
        .count();
    assert_eq!(deaths, 2, "Expected 2 CreatureDied events");
}

// ---------------------------------------------------------------------------
// Test 4: First strike kills blocker before regular damage (CR 702.7)
// ---------------------------------------------------------------------------

#[test]
/// CR 702.7 — a creature with first strike deals damage before creatures
/// without it. A 2/1 first striker kills a 2/2 blocker before taking damage.
fn test_702_7_first_strike_kills_blocker_before_regular_damage() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(
            ObjectSpec::creature(p1, "FirstStriker", 2, 1)
                .with_keyword(KeywordAbility::FirstStrike),
        )
        .object(ObjectSpec::creature(p2, "Blocker", 2, 2))
        .at_step(Step::DeclareAttackers)
        .active_player(p1)
        .build().unwrap();

    let attacker_id = state
        .objects
        .values()
        .find(|o| o.controller == p1)
        .unwrap()
        .id;
    let blocker_id = state
        .objects
        .values()
        .find(|o| o.controller == p2)
        .unwrap()
        .id;

    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(attacker_id, AttackTarget::Player(p2))],
        },
    )
    .unwrap();
    let (state, _) = pass_all(state, &[p1, p2]);

    let (state, _) = process_command(
        state,
        Command::DeclareBlockers {
            player: p2,
            blockers: vec![(blocker_id, attacker_id)],
        },
    )
    .unwrap();

    // Both pass → FirstStrikeDamage step entered (first-strike damage applied here).
    let (state, fs_events) = pass_all(state, &[p1, p2]);
    assert_eq!(
        state.turn.step,
        Step::FirstStrikeDamage,
        "Should enter FirstStrikeDamage step"
    );

    // The blocker should have died from first-strike damage (CreatureDied fired on step entry).
    let blocker_dead = fs_events
        .iter()
        .any(|e| matches!(e, GameEvent::CreatureDied { .. }));
    assert!(
        blocker_dead,
        "Blocker should have died from first-strike damage"
    );

    // Pass through FirstStrikeDamage → CombatDamage (blocker already dead; attacker survives).
    let (state, _) = pass_all(state, &[p1, p2]);

    // The attacker should survive (blocker had no first strike, can't deal damage in first step).
    let attacker_alive = state.objects.values().any(|o| o.id == attacker_id);
    assert!(
        attacker_alive,
        "First striker should survive (blocker can't deal back in first-strike step)"
    );

    // p1's life total unchanged (attacker survived).
    assert_eq!(state.players.get(&p1).unwrap().life_total, 40);
}

// ---------------------------------------------------------------------------
// Test 5: Double strike deals damage in both steps (CR 702.4)
// ---------------------------------------------------------------------------

#[test]
/// CR 702.4 — a creature with double strike deals first-strike damage and
/// regular combat damage. A 2/2 double striker deals 4 total damage.
fn test_702_4_double_strike_deals_in_both_steps() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(
            ObjectSpec::creature(p1, "DoubleStriker", 2, 2)
                .with_keyword(KeywordAbility::DoubleStrike),
        )
        .at_step(Step::DeclareAttackers)
        .active_player(p1)
        .build().unwrap();

    let attacker_id = state
        .objects
        .values()
        .find(|o| o.controller == p1)
        .unwrap()
        .id;

    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(attacker_id, AttackTarget::Player(p2))],
        },
    )
    .unwrap();
    let (state, _) = pass_all(state, &[p1, p2]);

    // No blockers.
    let (state, _) = process_command(
        state,
        Command::DeclareBlockers {
            player: p2,
            blockers: vec![],
        },
    )
    .unwrap();

    // Pass through DeclareBlockers → FirstStrikeDamage step entered (first-strike damage applied here).
    let (state, fs_step_events) = pass_all(state, &[p1, p2]);
    assert_eq!(state.turn.step, Step::FirstStrikeDamage);

    let first_damage: u32 = fs_step_events
        .iter()
        .filter_map(|e| {
            if let GameEvent::CombatDamageDealt { assignments } = e {
                Some(
                    assignments
                        .iter()
                        .filter(
                            |a| matches!(a.target, CombatDamageTarget::Player(pid) if pid == p2),
                        )
                        .map(|a| a.amount)
                        .sum::<u32>(),
                )
            } else {
                None
            }
        })
        .sum();
    assert_eq!(first_damage, 2, "Should deal 2 in first-strike step");

    // Pass through FirstStrikeDamage → CombatDamage step entered (regular damage applied here).
    let (state, regular_step_events) = pass_all(state, &[p1, p2]);
    assert_eq!(state.turn.step, Step::CombatDamage);

    let second_damage: u32 = regular_step_events
        .iter()
        .filter_map(|e| {
            if let GameEvent::CombatDamageDealt { assignments } = e {
                Some(
                    assignments
                        .iter()
                        .filter(
                            |a| matches!(a.target, CombatDamageTarget::Player(pid) if pid == p2),
                        )
                        .map(|a| a.amount)
                        .sum::<u32>(),
                )
            } else {
                None
            }
        })
        .sum();
    assert_eq!(
        second_damage, 2,
        "Should deal 2 again in regular combat damage step"
    );

    // p2 should have taken 4 total damage (2 first-strike + 2 regular).
    assert_eq!(state.players.get(&p2).unwrap().life_total, 36);
}

// ---------------------------------------------------------------------------
// Test 6: Trample excess goes to defending player (CR 702.19b)
// ---------------------------------------------------------------------------

#[test]
/// CR 702.19b — trample allows excess combat damage beyond what is needed for
/// lethal to be dealt to the defending player. A 5/5 trampler vs a 2/2 blocker
/// deals 3 to the player.
fn test_702_19_trample_excess_to_player() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(ObjectSpec::creature(p1, "Trampler", 5, 5).with_keyword(KeywordAbility::Trample))
        .object(ObjectSpec::creature(p2, "Blocker", 2, 2))
        .at_step(Step::DeclareAttackers)
        .active_player(p1)
        .build().unwrap();

    let attacker_id = state
        .objects
        .values()
        .find(|o| o.controller == p1)
        .unwrap()
        .id;
    let blocker_id = state
        .objects
        .values()
        .find(|o| o.controller == p2)
        .unwrap()
        .id;

    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(attacker_id, AttackTarget::Player(p2))],
        },
    )
    .unwrap();
    let (state, _) = pass_all(state, &[p1, p2]);

    let (state, _) = process_command(
        state,
        Command::DeclareBlockers {
            player: p2,
            blockers: vec![(blocker_id, attacker_id)],
        },
    )
    .unwrap();

    let (state, events) = pass_all(state, &[p1, p2]);

    // 2 damage to blocker, 3 trample to p2.
    let blocker_damage: u32 = events
        .iter()
        .filter_map(|e| {
            if let GameEvent::CombatDamageDealt { assignments } = e {
                Some(
                    assignments
                        .iter()
                        .filter(|a| matches!(a.target, CombatDamageTarget::Creature(id) if id == blocker_id))
                        .map(|a| a.amount)
                        .sum::<u32>(),
                )
            } else {
                None
            }
        })
        .sum();
    assert_eq!(
        blocker_damage, 2,
        "Trampler should assign 2 (lethal) to blocker"
    );

    assert_eq!(
        state.players.get(&p2).unwrap().life_total,
        37,
        "p2 should take 3 trample damage"
    );
}

// ---------------------------------------------------------------------------
// Test 7: Deathtouch + Trample — 1 lethal to blocker, rest tramples (CR 702.2 + 702.19)
// ---------------------------------------------------------------------------

#[test]
/// CR 702.2c + 702.19b — deathtouch makes 1 damage lethal for assignment
/// purposes. A 4/4 with deathtouch and trample blocked by a 3/3 assigns
/// 1 to the blocker (lethal) and 3 to the player.
fn test_702_deathtouch_with_trample() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(
            ObjectSpec::creature(p1, "DeathtouchTrampler", 4, 4)
                .with_keyword(KeywordAbility::Deathtouch)
                .with_keyword(KeywordAbility::Trample),
        )
        .object(ObjectSpec::creature(p2, "BigBlocker", 3, 3))
        .at_step(Step::DeclareAttackers)
        .active_player(p1)
        .build().unwrap();

    let attacker_id = state
        .objects
        .values()
        .find(|o| o.controller == p1)
        .unwrap()
        .id;
    let blocker_id = state
        .objects
        .values()
        .find(|o| o.controller == p2)
        .unwrap()
        .id;

    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(attacker_id, AttackTarget::Player(p2))],
        },
    )
    .unwrap();
    let (state, _) = pass_all(state, &[p1, p2]);

    let (state, _) = process_command(
        state,
        Command::DeclareBlockers {
            player: p2,
            blockers: vec![(blocker_id, attacker_id)],
        },
    )
    .unwrap();

    let (state, events) = pass_all(state, &[p1, p2]);

    // With deathtouch, only 1 damage is "lethal" for trample purposes.
    let blocker_damage: u32 = events
        .iter()
        .filter_map(|e| {
            if let GameEvent::CombatDamageDealt { assignments } = e {
                Some(
                    assignments
                        .iter()
                        .filter(|a| matches!(a.target, CombatDamageTarget::Creature(id) if id == blocker_id))
                        .map(|a| a.amount)
                        .sum::<u32>(),
                )
            } else {
                None
            }
        })
        .sum();
    assert_eq!(
        blocker_damage, 1,
        "Only 1 (deathtouch lethal) assigned to blocker"
    );
    assert_eq!(
        state.players.get(&p2).unwrap().life_total,
        37,
        "3 trample damage to p2 (4 power - 1 lethal)"
    );
}

// ---------------------------------------------------------------------------
// Test 8: Multiple blockers — damage assignment order (CR 509.2, CR 510.1c)
// ---------------------------------------------------------------------------

#[test]
/// CR 510.1c — when multiple creatures block, the attacker assigns damage in
/// the declared order; each blocker must receive lethal before the next gets any.
/// A 5/5 attacker vs [2/2, 2/2] blockers with OrderBlockers declared.
fn test_509_2_multiple_blockers_damage_order() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(ObjectSpec::creature(p1, "BigAttacker", 5, 5))
        .object(ObjectSpec::creature(p2, "Blocker1", 2, 2))
        .object(ObjectSpec::creature(p2, "Blocker2", 2, 2))
        .at_step(Step::DeclareAttackers)
        .active_player(p1)
        .build().unwrap();

    let attacker_id = state
        .objects
        .values()
        .find(|o| o.controller == p1)
        .unwrap()
        .id;
    let blocker1_id = state
        .objects
        .values()
        .find(|o| o.controller == p2 && o.characteristics.name == "Blocker1")
        .unwrap()
        .id;
    let blocker2_id = state
        .objects
        .values()
        .find(|o| o.controller == p2 && o.characteristics.name == "Blocker2")
        .unwrap()
        .id;

    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(attacker_id, AttackTarget::Player(p2))],
        },
    )
    .unwrap();
    let (state, _) = pass_all(state, &[p1, p2]);

    // p2 blocks with both creatures.
    let (state, _) = process_command(
        state,
        Command::DeclareBlockers {
            player: p2,
            blockers: vec![(blocker1_id, attacker_id), (blocker2_id, attacker_id)],
        },
    )
    .unwrap();

    // p1 orders blockers: blocker1 first, then blocker2.
    let (state, _) = process_command(
        state,
        Command::OrderBlockers {
            player: p1,
            attacker: attacker_id,
            order: vec![blocker1_id, blocker2_id],
        },
    )
    .unwrap();

    let (state, events) = pass_all(state, &[p1, p2]);

    // Blocker1 (2/2) should get 2 damage (lethal), blocker2 should get 3 (remaining 5-2).
    let b1_damage: u32 = events
        .iter()
        .filter_map(|e| {
            if let GameEvent::CombatDamageDealt { assignments } = e {
                Some(
                    assignments
                        .iter()
                        .filter(|a| matches!(a.target, CombatDamageTarget::Creature(id) if id == blocker1_id))
                        .map(|a| a.amount)
                        .sum::<u32>(),
                )
            } else {
                None
            }
        })
        .sum();
    let b2_damage: u32 = events
        .iter()
        .filter_map(|e| {
            if let GameEvent::CombatDamageDealt { assignments } = e {
                Some(
                    assignments
                        .iter()
                        .filter(|a| matches!(a.target, CombatDamageTarget::Creature(id) if id == blocker2_id))
                        .map(|a| a.amount)
                        .sum::<u32>(),
                )
            } else {
                None
            }
        })
        .sum();

    assert_eq!(b1_damage, 2, "Blocker1 should get exactly lethal (2)");
    assert_eq!(b2_damage, 3, "Blocker2 gets remaining 3 damage");

    // Both should be dead (2/2 hit by ≥2).
    let b1_alive = state.objects.values().any(|o| o.id == blocker1_id);
    let b2_alive = state.objects.values().any(|o| o.id == blocker2_id);
    assert!(!b1_alive, "Blocker1 should have died");
    assert!(!b2_alive, "Blocker2 should have died");

    // No damage to p2 (no trample on the attacker).
    assert_eq!(state.players.get(&p2).unwrap().life_total, 40);
}

// ---------------------------------------------------------------------------
// Test 9: SelfAttacks trigger fires when creature attacks (CR 603.5)
// ---------------------------------------------------------------------------

#[test]
/// CR 603.5 — "whenever this creature attacks" triggers when it is declared
/// as an attacker and the trigger goes on the stack.
fn test_603_self_attacks_trigger_fires() {
    use mtg_engine::{TriggerEvent, TriggeredAbilityDef};

    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let triggered_ability = TriggeredAbilityDef {
        trigger_on: TriggerEvent::SelfAttacks,
        intervening_if: None,
        description: "When this attacks, do something".to_string(),
        effect: None,
    };

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(
            ObjectSpec::creature(p1, "Attacker", 2, 2).with_triggered_ability(triggered_ability),
        )
        .at_step(Step::DeclareAttackers)
        .active_player(p1)
        .build().unwrap();

    let attacker_id = state
        .objects
        .values()
        .find(|o| o.controller == p1)
        .unwrap()
        .id;

    let (state, events) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(attacker_id, AttackTarget::Player(p2))],
        },
    )
    .expect("declare attackers failed");

    // The triggered ability should have been flushed to the stack (AbilityTriggered event).
    let triggered = events
        .iter()
        .any(|e| matches!(e, GameEvent::AbilityTriggered { source_object_id, .. } if *source_object_id == attacker_id));
    assert!(
        triggered,
        "SelfAttacks trigger should have fired and been placed on the stack"
    );

    // Stack should have one object (the triggered ability).
    assert_eq!(
        state.stack_objects.len(),
        1,
        "Triggered ability should be on the stack"
    );
}

// ---------------------------------------------------------------------------
// Test 10: Commander damage tracked in commander_damage_received (CR 903.10a)
// ---------------------------------------------------------------------------

#[test]
/// CR 903.10a — 21 or more combat damage dealt by a single commander causes
/// the damaged player to lose. Commander damage is tracked per player per card.
fn test_903_10a_commander_damage_tracked() {
    use mtg_engine::CardId;

    let p1 = PlayerId(1);
    let p2 = PlayerId(2);
    let commander_card = CardId("commander-1".to_string());

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .player_commander(p1, commander_card.clone())
        .object(ObjectSpec::creature(p1, "Commander", 5, 5).with_card_id(commander_card.clone()))
        .at_step(Step::DeclareAttackers)
        .active_player(p1)
        .build().unwrap();

    let attacker_id = state
        .objects
        .values()
        .find(|o| o.controller == p1)
        .unwrap()
        .id;

    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(attacker_id, AttackTarget::Player(p2))],
        },
    )
    .unwrap();
    let (state, _) = pass_all(state, &[p1, p2]);

    let (state, _) = process_command(
        state,
        Command::DeclareBlockers {
            player: p2,
            blockers: vec![],
        },
    )
    .unwrap();

    let (state, _) = pass_all(state, &[p1, p2]);

    // Commander damage should be tracked in p2's record.
    let p2_state = state.players.get(&p2).unwrap();
    let damage_from_p1 = p2_state
        .commander_damage_received
        .get(&p1)
        .and_then(|m| m.get(&commander_card))
        .copied()
        .unwrap_or(0);
    assert_eq!(
        damage_from_p1, 5,
        "p2 should have 5 commander damage from p1's commander"
    );
}

// ---------------------------------------------------------------------------
// Test 11: Multiplayer — attack two different players simultaneously (CR 506.4)
// ---------------------------------------------------------------------------

#[test]
/// CR 506.4 — in multiplayer, the attacking player may attack multiple opponents
/// or their controlled planeswalkers in a single combat phase.
fn test_506_multiplayer_simultaneous_attacks() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);
    let p3 = PlayerId(3);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .object(ObjectSpec::creature(p1, "Attacker1", 2, 2))
        .object(ObjectSpec::creature(p1, "Attacker2", 3, 3))
        .at_step(Step::DeclareAttackers)
        .active_player(p1)
        .build().unwrap();

    let att1_id = state
        .objects
        .values()
        .find(|o| o.controller == p1 && o.characteristics.name == "Attacker1")
        .unwrap()
        .id;
    let att2_id = state
        .objects
        .values()
        .find(|o| o.controller == p1 && o.characteristics.name == "Attacker2")
        .unwrap()
        .id;

    // Attack p2 and p3 simultaneously.
    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![
                (att1_id, AttackTarget::Player(p2)),
                (att2_id, AttackTarget::Player(p3)),
            ],
        },
    )
    .unwrap();
    let (state, _) = pass_all(state, &[p1, p2, p3]);

    // Both p2 and p3 declare no blockers.
    let (state, _) = process_command(
        state,
        Command::DeclareBlockers {
            player: p2,
            blockers: vec![],
        },
    )
    .unwrap();
    let (state, _) = process_command(
        state,
        Command::DeclareBlockers {
            player: p3,
            blockers: vec![],
        },
    )
    .unwrap();

    let (state, _) = pass_all(state, &[p1, p2, p3]);

    // p2 should have 38 life (took 2), p3 should have 37 (took 3).
    assert_eq!(state.players.get(&p2).unwrap().life_total, 38);
    assert_eq!(state.players.get(&p3).unwrap().life_total, 37);
    // p1 untouched.
    assert_eq!(state.players.get(&p1).unwrap().life_total, 40);
}

// ---------------------------------------------------------------------------
// Attack target validation tests (MR-M6-01, CR 508.1 / CR 903.6)
// ---------------------------------------------------------------------------

#[test]
/// CR 508.1 / CR 903.6 — a player cannot declare an attack targeting themselves.
fn test_508_attack_self_rejected() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(ObjectSpec::creature(p1, "Bear", 2, 2))
        .at_step(Step::DeclareAttackers)
        .active_player(p1)
        .build().unwrap();

    let bear_id = state.objects.values().find(|o| o.controller == p1).unwrap().id;

    let result = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(bear_id, AttackTarget::Player(p1))],
        },
    );
    assert!(
        matches!(result.unwrap_err(), GameStateError::InvalidAttackTarget(_)),
        "attacking self should be rejected"
    );
}

#[test]
/// CR 508.1 — attacking a player that doesn't exist in the game is rejected.
fn test_508_attack_nonexistent_player_rejected() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(ObjectSpec::creature(p1, "Bear", 2, 2))
        .at_step(Step::DeclareAttackers)
        .active_player(p1)
        .build().unwrap();

    let bear_id = state.objects.values().find(|o| o.controller == p1).unwrap().id;

    let result = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(bear_id, AttackTarget::Player(PlayerId(99)))],
        },
    );
    assert!(
        matches!(result.unwrap_err(), GameStateError::PlayerNotFound(_)),
        "attacking a nonexistent player should be rejected"
    );
}

#[test]
/// CR 903.6 — a player cannot attack their own planeswalker.
fn test_508_attack_own_planeswalker_rejected() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(ObjectSpec::creature(p1, "Bear", 2, 2))
        .object(ObjectSpec::planeswalker(p1, "Jace", 5))
        .at_step(Step::DeclareAttackers)
        .active_player(p1)
        .build().unwrap();

    let bear_id = state
        .objects
        .values()
        .find(|o| o.controller == p1 && o.characteristics.name == "Bear")
        .unwrap()
        .id;
    let pw_id = state
        .objects
        .values()
        .find(|o| o.characteristics.name == "Jace")
        .unwrap()
        .id;

    let result = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(bear_id, AttackTarget::Planeswalker(pw_id))],
        },
    );
    assert!(
        matches!(result.unwrap_err(), GameStateError::InvalidAttackTarget(_)),
        "attacking own planeswalker should be rejected"
    );
}

#[test]
/// CR 903.6 — a player may attack an opponent's planeswalker (positive test).
fn test_508_attack_opponent_planeswalker_accepted() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(ObjectSpec::creature(p1, "Bear", 2, 2))
        .object(ObjectSpec::planeswalker(p2, "Jace", 5))
        .at_step(Step::DeclareAttackers)
        .active_player(p1)
        .build().unwrap();

    let bear_id = state
        .objects
        .values()
        .find(|o| o.controller == p1)
        .unwrap()
        .id;
    let pw_id = state
        .objects
        .values()
        .find(|o| o.controller == p2)
        .unwrap()
        .id;

    let result = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(bear_id, AttackTarget::Planeswalker(pw_id))],
        },
    );
    assert!(result.is_ok(), "attacking opponent's planeswalker should succeed");
}

// ---------------------------------------------------------------------------
// Duplicate blocker test (MR-M6-02, CR 509)
// ---------------------------------------------------------------------------

#[test]
/// CR 509 — a creature cannot block two different attackers (OrdMap would silently overwrite).
fn test_509_duplicate_blocker_rejected() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(ObjectSpec::creature(p1, "Attacker1", 2, 2))
        .object(ObjectSpec::creature(p1, "Attacker2", 2, 2))
        .object(ObjectSpec::creature(p2, "Blocker", 2, 2))
        .at_step(Step::DeclareAttackers)
        .active_player(p1)
        .build().unwrap();

    let attacker1_id = state
        .objects
        .values()
        .find(|o| o.controller == p1 && o.characteristics.name == "Attacker1")
        .unwrap()
        .id;
    let attacker2_id = state
        .objects
        .values()
        .find(|o| o.controller == p1 && o.characteristics.name == "Attacker2")
        .unwrap()
        .id;
    let blocker_id = state
        .objects
        .values()
        .find(|o| o.controller == p2)
        .unwrap()
        .id;

    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![
                (attacker1_id, AttackTarget::Player(p2)),
                (attacker2_id, AttackTarget::Player(p2)),
            ],
        },
    )
    .expect("declare attackers should succeed");

    let (state, _) = pass_all(state, &[p1, p2]);
    assert_eq!(state.turn.step, Step::DeclareBlockers);

    // Try to block both attackers with the same creature in one declaration.
    let result = process_command(
        state,
        Command::DeclareBlockers {
            player: p2,
            blockers: vec![(blocker_id, attacker1_id), (blocker_id, attacker2_id)],
        },
    );
    assert!(
        matches!(result.unwrap_err(), GameStateError::DuplicateBlocker(id) if id == blocker_id),
        "same creature blocking two attackers should be rejected"
    );
}

// ---------------------------------------------------------------------------
// Incomplete blocker order test (MR-M6-03, CR 509.2)
// ---------------------------------------------------------------------------

#[test]
/// CR 509.2 — the attacker's controller must order ALL blockers, not a subset.
fn test_509_incomplete_blocker_order_rejected() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(ObjectSpec::creature(p1, "Attacker", 5, 5))
        .object(ObjectSpec::creature(p2, "Blocker1", 2, 2))
        .object(ObjectSpec::creature(p2, "Blocker2", 2, 2))
        .at_step(Step::DeclareAttackers)
        .active_player(p1)
        .build().unwrap();

    let attacker_id = state
        .objects
        .values()
        .find(|o| o.controller == p1)
        .unwrap()
        .id;
    let blocker1_id = state
        .objects
        .values()
        .find(|o| o.controller == p2 && o.characteristics.name == "Blocker1")
        .unwrap()
        .id;
    let blocker2_id = state
        .objects
        .values()
        .find(|o| o.controller == p2 && o.characteristics.name == "Blocker2")
        .unwrap()
        .id;

    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(attacker_id, AttackTarget::Player(p2))],
        },
    )
    .expect("declare attackers should succeed");

    let (state, _) = pass_all(state, &[p1, p2]);
    assert_eq!(state.turn.step, Step::DeclareBlockers);

    // p2 blocks with both creatures.
    let (state, _) = process_command(
        state,
        Command::DeclareBlockers {
            player: p2,
            blockers: vec![(blocker1_id, attacker_id), (blocker2_id, attacker_id)],
        },
    )
    .expect("declare blockers should succeed");

    // p1 provides an incomplete order (only 1 of 2 blockers).
    let result = process_command(
        state,
        Command::OrderBlockers {
            player: p1,
            attacker: attacker_id,
            order: vec![blocker1_id],
        },
    );
    assert!(
        matches!(
            result.unwrap_err(),
            GameStateError::IncompleteBlockerOrder { provided: 1, required: 2 }
        ),
        "partial blocker ordering should be rejected"
    );
}

// ---------------------------------------------------------------------------
// Cross-player blocking test (MR-M6-09, CR 509.1c)
// ---------------------------------------------------------------------------

#[test]
/// CR 509.1c — a defending player may only block an attacker that is targeting them
/// (or their controlled planeswalker), not an attacker targeting a different player.
fn test_509_cross_player_block_rejected() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);
    let p3 = PlayerId(3);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .object(ObjectSpec::creature(p1, "Attacker", 3, 3))
        .object(ObjectSpec::creature(p3, "Blocker", 2, 2))
        .at_step(Step::DeclareAttackers)
        .active_player(p1)
        .build().unwrap();

    let attacker_id = state
        .objects
        .values()
        .find(|o| o.controller == p1)
        .unwrap()
        .id;
    let blocker_id = state
        .objects
        .values()
        .find(|o| o.controller == p3)
        .unwrap()
        .id;

    // p1 attacks p2 (not p3).
    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(attacker_id, AttackTarget::Player(p2))],
        },
    )
    .expect("declare attackers should succeed");

    let (state, _) = pass_all(state, &[p1, p2, p3]);
    assert_eq!(state.turn.step, Step::DeclareBlockers);

    // p3 tries to block p1's attacker (which is attacking p2, not p3).
    let result = process_command(
        state,
        Command::DeclareBlockers {
            player: p3,
            blockers: vec![(blocker_id, attacker_id)],
        },
    );
    assert!(
        matches!(
            result.unwrap_err(),
            GameStateError::CrossPlayerBlock { blocker, attacker }
            if blocker == blocker_id && attacker == attacker_id
        ),
        "blocking an attacker that targets another player should be rejected"
    );
}

// ---------------------------------------------------------------------------
// Re-declare blockers test (MR-M6-10, CR 509.1)
// ---------------------------------------------------------------------------

#[test]
/// CR 509.1 — each defending player declares blockers exactly once.
/// Attempting to declare a second time in the same combat should be rejected.
fn test_509_redeclare_blockers_rejected() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(ObjectSpec::creature(p1, "Attacker", 2, 2))
        .at_step(Step::DeclareAttackers)
        .active_player(p1)
        .build().unwrap();

    let attacker_id = state
        .objects
        .values()
        .find(|o| o.controller == p1)
        .unwrap()
        .id;

    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(attacker_id, AttackTarget::Player(p2))],
        },
    )
    .expect("declare attackers should succeed");

    let (state, _) = pass_all(state, &[p1, p2]);
    assert_eq!(state.turn.step, Step::DeclareBlockers);

    // p2 declares no blockers (valid first declaration).
    let (state, _) = process_command(
        state,
        Command::DeclareBlockers {
            player: p2,
            blockers: vec![],
        },
    )
    .expect("first blocker declaration should succeed");

    // p2 tries to declare again — should be rejected.
    let result = process_command(
        state,
        Command::DeclareBlockers {
            player: p2,
            blockers: vec![],
        },
    );
    assert!(
        matches!(result.unwrap_err(), GameStateError::AlreadyDeclaredBlockers(pid) if pid == p2),
        "re-declaring blockers should be rejected"
    );
}
