//! Combat phase tests (CR 506-511).
//!
//! Tests cover: attacker/blocker declaration, combat damage, first strike,
//! double strike, trample, deathtouch+trample, multiple blockers,
//! combat triggers, commander damage, and multiplayer combat.

use mtg_engine::{
    process_command, AttackTarget, CombatDamageTarget, Command, ContinuousEffect, CounterType,
    EffectDuration, EffectFilter, EffectId, EffectLayer, GameEvent, GameState, GameStateBuilder,
    GameStateError, KeywordAbility, LayerModification, ObjectSpec, PlayerId, Step,
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
        .build()
        .unwrap();

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
            enlist_choices: vec![],
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
        .build()
        .unwrap();

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
            enlist_choices: vec![],
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
        .build()
        .unwrap();

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
            enlist_choices: vec![],
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
        .build()
        .unwrap();

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
            enlist_choices: vec![],
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
        .build()
        .unwrap();

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
            enlist_choices: vec![],
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
        .build()
        .unwrap();

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
            enlist_choices: vec![],
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
        .build()
        .unwrap();

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
            enlist_choices: vec![],
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
        .build()
        .unwrap();

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
            enlist_choices: vec![],
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
// Test 9a: Trample + multiple blockers — excess goes to player (CR 702.19b)
// ---------------------------------------------------------------------------

#[test]
/// CR 702.19b — the controller of a creature with trample assigns lethal damage
/// to each blocker in order, then any excess goes to the defending player.
/// A 6/6 trampler blocked by two 2/2s assigns 2 lethal to each blocker and
/// 2 excess to the defending player.
fn test_702_19b_trample_multiple_blockers_excess_to_player() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(ObjectSpec::creature(p1, "Trampler", 6, 6).with_keyword(KeywordAbility::Trample))
        .object(ObjectSpec::creature(p2, "Blocker1", 2, 2))
        .object(ObjectSpec::creature(p2, "Blocker2", 2, 2))
        .at_step(Step::DeclareAttackers)
        .active_player(p1)
        .build()
        .unwrap();

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
            enlist_choices: vec![],
        },
    )
    .unwrap();
    let (state, _) = pass_all(state, &[p1, p2]);

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

    // 2 lethal to each blocker, 2 excess trample to p2.
    let b1_damage: u32 = events
        .iter()
        .filter_map(|e| {
            if let GameEvent::CombatDamageDealt { assignments } = e {
                Some(
                    assignments
                        .iter()
                        .filter(|a| {
                            matches!(a.target, CombatDamageTarget::Creature(id) if id == blocker1_id)
                        })
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
                        .filter(|a| {
                            matches!(a.target, CombatDamageTarget::Creature(id) if id == blocker2_id)
                        })
                        .map(|a| a.amount)
                        .sum::<u32>(),
                )
            } else {
                None
            }
        })
        .sum();

    assert_eq!(b1_damage, 2, "Blocker1 should receive exactly lethal (2)");
    assert_eq!(b2_damage, 2, "Blocker2 should receive exactly lethal (2)");
    assert_eq!(
        state.players.get(&p2).unwrap().life_total,
        38,
        "p2 should take 2 excess trample damage (6 power - 2 - 2 = 2)"
    );
}

// ---------------------------------------------------------------------------
// Test 9b: Trample when all blockers die before damage step (CR 702.19d)
// ---------------------------------------------------------------------------

#[test]
/// CR 702.19d — if a trample creature is blocked but all blockers are removed
/// before the combat damage step, the trample creature assigns its full power
/// to the defending player as though all blockers had been assigned lethal damage.
/// Scenario: 4/4 double strike + trample vs a 1/1 blocker.
/// In the first-strike step the blocker dies (1 lethal + 3 trample to player).
/// In the regular damage step there are no blockers remaining — the 702.19d code
/// path fires and the double-striker deals its full 4 power to the player again.
/// p2 ends up at 40 - 3 - 4 = 33.
fn test_702_19d_trample_blockers_removed_before_damage() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(
            ObjectSpec::creature(p1, "TrampleDoubleStriker", 4, 4)
                .with_keyword(KeywordAbility::Trample)
                .with_keyword(KeywordAbility::DoubleStrike),
        )
        .object(ObjectSpec::creature(p2, "TinyBlocker", 1, 1))
        .at_step(Step::DeclareAttackers)
        .active_player(p1)
        .build()
        .unwrap();

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
            enlist_choices: vec![],
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

    // Both pass → FirstStrikeDamage step entered. The 4/4 first striker kills the 1/1 blocker.
    let (state, fs_events) = pass_all(state, &[p1, p2]);
    assert_eq!(
        state.turn.step,
        Step::FirstStrikeDamage,
        "Should be in FirstStrikeDamage step"
    );
    let blocker_died = fs_events
        .iter()
        .any(|e| matches!(e, GameEvent::CreatureDied { .. }));
    assert!(blocker_died, "TinyBlocker should die in first-strike step");

    // The blocker is gone from the battlefield.
    let blocker_on_field = state.objects.values().any(|o| o.id == blocker_id);
    assert!(
        !blocker_on_field,
        "TinyBlocker should no longer be on the battlefield"
    );

    // Trample dealt 3 excess in the first-strike step (4 power - 1 lethal = 3).
    assert_eq!(
        state.players.get(&p2).unwrap().life_total,
        37,
        "p2 should take 3 trample damage in first-strike step"
    );

    // Pass through FirstStrikeDamage → CombatDamage. The trample creature was blocked
    // but no blockers remain — the CR 702.19d code path fires. The double-striker deals
    // its full 4 power to the defending player a second time.
    let (state, _) = pass_all(state, &[p1, p2]);

    assert_eq!(
        state.players.get(&p2).unwrap().life_total,
        33,
        "p2 should take 4 more damage in regular step (702.19d: blocked but all blockers gone)"
    );
}

// ---------------------------------------------------------------------------
// Test 9: SelfAttacks trigger fires when creature attacks (CR 508.3a)
// ---------------------------------------------------------------------------

#[test]
/// CR 508.3a — "whenever this creature attacks" triggers when it is declared
/// as an attacker and the trigger goes on the stack.
fn test_603_self_attacks_trigger_fires() {
    use mtg_engine::{TriggerEvent, TriggeredAbilityDef};

    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let triggered_ability = TriggeredAbilityDef {
        counter_filter: None,
        counter_on_self: false,
        once_per_turn: false,
        etb_filter: None,
        death_filter: None,
        combat_damage_filter: None,
        triggering_creature_filter: None,
        targets: vec![],
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
        .build()
        .unwrap();

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
            enlist_choices: vec![],
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
        .build()
        .unwrap();

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
            enlist_choices: vec![],
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
        .build()
        .unwrap();

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
            enlist_choices: vec![],
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
        .build()
        .unwrap();

    let bear_id = state
        .objects
        .values()
        .find(|o| o.controller == p1)
        .unwrap()
        .id;

    let result = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(bear_id, AttackTarget::Player(p1))],
            enlist_choices: vec![],
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
        .build()
        .unwrap();

    let bear_id = state
        .objects
        .values()
        .find(|o| o.controller == p1)
        .unwrap()
        .id;

    let result = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(bear_id, AttackTarget::Player(PlayerId(99)))],
            enlist_choices: vec![],
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
        .build()
        .unwrap();

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
            enlist_choices: vec![],
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
        .build()
        .unwrap();

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
            enlist_choices: vec![],
        },
    );
    assert!(
        result.is_ok(),
        "attacking opponent's planeswalker should succeed"
    );
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
        .build()
        .unwrap();

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
            enlist_choices: vec![],
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
        .build()
        .unwrap();

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
            enlist_choices: vec![],
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
            GameStateError::IncompleteBlockerOrder {
                provided: 1,
                required: 2
            }
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
        .build()
        .unwrap();

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
            enlist_choices: vec![],
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
        .build()
        .unwrap();

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
            enlist_choices: vec![],
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

// ---------------------------------------------------------------------------
// CC#20: First strike + double strike combined blocking
// ---------------------------------------------------------------------------

#[test]
/// CC#20 / CR 702.7 + CR 702.4 — A first-strike creature blocks a double-strike creature.
///
/// Scenario: p1 attacks with a 3/1 DoubleStrike creature. p2 blocks with a 2/2 FirstStrike
/// creature. In the first-strike combat damage step, both creatures deal their damage
/// simultaneously (both have first-strike or double-strike):
///   - Attacker (3/1 DoubleStrike) deals 3 to the blocker (lethal on 2 toughness).
///   - Blocker (2/2 FirstStrike) deals 2 to the attacker (lethal on 1 toughness).
///
/// Both creatures die in the first-strike step. In the regular combat damage step,
/// neither creature deals damage (both are dead — they never reach the regular step alive).
///
/// CR 702.7b: "A creature with first strike deals combat damage before creatures without
/// first strike." CR 702.4b: "A creature with double strike deals both first-strike and
/// regular combat damage."
fn test_cc20_first_strike_blocks_double_strike() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    // Attacker: 3/1 DoubleStrike (deals first-strike AND regular damage).
    // Blocker: 2/2 FirstStrike (deals first-strike damage only).
    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(
            ObjectSpec::creature(p1, "DoubleStriker", 3, 1)
                .with_keyword(KeywordAbility::DoubleStrike),
        )
        .object(
            ObjectSpec::creature(p2, "FirstStriker", 2, 2)
                .with_keyword(KeywordAbility::FirstStrike),
        )
        .at_step(Step::DeclareAttackers)
        .active_player(p1)
        .build()
        .unwrap();

    let attacker_id = state
        .objects
        .values()
        .find(|o| o.characteristics.name == "DoubleStriker")
        .unwrap()
        .id;
    let blocker_id = state
        .objects
        .values()
        .find(|o| o.characteristics.name == "FirstStriker")
        .unwrap()
        .id;

    // Declare attacker.
    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(attacker_id, AttackTarget::Player(p2))],
            enlist_choices: vec![],
        },
    )
    .unwrap();
    let (state, _) = pass_all(state, &[p1, p2]);

    // p2 blocks with the FirstStriker.
    let (state, _) = process_command(
        state,
        Command::DeclareBlockers {
            player: p2,
            blockers: vec![(blocker_id, attacker_id)],
        },
    )
    .unwrap();

    // Pass through DeclareBlockers → FirstStrikeDamage step.
    // Both creatures deal first-strike damage simultaneously in this step.
    let (state, fs_events) = pass_all(state, &[p1, p2]);
    assert_eq!(
        state.turn.step,
        Step::FirstStrikeDamage,
        "should enter FirstStrikeDamage step"
    );

    // Both creatures should have died from first-strike damage:
    // - DoubleStriker (3 power) kills FirstStriker (2 toughness).
    // - FirstStriker (2 power) kills DoubleStriker (1 toughness).
    let deaths_in_fs_step = fs_events
        .iter()
        .filter(|e| matches!(e, GameEvent::CreatureDied { .. }))
        .count();
    assert_eq!(
        deaths_in_fs_step, 2,
        "both creatures should die in the first-strike step \
         (DoubleStriker kills FirstStriker, FirstStriker kills DoubleStriker); \
         deaths: {}",
        deaths_in_fs_step
    );

    // Both should be off the battlefield.
    let attacker_alive = state.objects.values().any(|o| o.id == attacker_id);
    let blocker_alive = state.objects.values().any(|o| o.id == blocker_id);
    assert!(
        !attacker_alive,
        "DoubleStriker (3/1) should be dead (took 2 from first-striker, lethal on 1 toughness)"
    );
    assert!(
        !blocker_alive,
        "FirstStriker (2/2) should be dead (took 3 from double-striker, lethal on 2 toughness)"
    );

    // Pass through FirstStrikeDamage → CombatDamage step.
    // Both creatures are dead; no regular combat damage is dealt.
    let (state, regular_events) = pass_all(state, &[p1, p2]);
    assert_eq!(state.turn.step, Step::CombatDamage);

    let damage_in_regular_step: u32 = regular_events
        .iter()
        .filter_map(|e| {
            if let GameEvent::CombatDamageDealt { assignments } = e {
                Some(assignments.iter().map(|a| a.amount).sum::<u32>())
            } else {
                None
            }
        })
        .sum();

    // CR 702.4: DoubleStriker deals regular damage too — but it's dead, so no damage in step 2.
    assert_eq!(
        damage_in_regular_step, 0,
        "no creature deals regular damage (both are dead after first-strike step); \
         damage in regular step: {}",
        damage_in_regular_step
    );

    // p2's life total: should be unchanged (DoubleStriker never reached regular damage step alive).
    assert_eq!(
        state.players.get(&p2).unwrap().life_total,
        40,
        "p2 should not have taken damage (DoubleStriker was blocked and killed before regular step)"
    );
    assert_eq!(
        state.players.get(&p1).unwrap().life_total,
        40,
        "p1 should not have taken damage (all damage was between creatures)"
    );
}

// ---------------------------------------------------------------------------
// Tests: SelfDealsCombatDamageToPlayer trigger (CR 510.3a, CR 603.2)
// ---------------------------------------------------------------------------

#[test]
/// CR 510.3a — "whenever ~ deals combat damage to a player" triggers after
/// combat damage is dealt during the CombatDamage step. An unblocked creature
/// with the trigger fires when it damages the defending player.
/// CR 603.2g — damage amount > 0 is required for the trigger to fire.
fn test_510_3a_combat_damage_trigger_fires_on_unblocked_attacker() {
    use mtg_engine::{TriggerEvent, TriggeredAbilityDef};

    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let triggered_ability = TriggeredAbilityDef {
        counter_filter: None,
        counter_on_self: false,
        once_per_turn: false,
        etb_filter: None,
        death_filter: None,
        combat_damage_filter: None,
        triggering_creature_filter: None,
        targets: vec![],
        trigger_on: TriggerEvent::SelfDealsCombatDamageToPlayer,
        intervening_if: None,
        description: "Whenever ~ deals combat damage to a player, do something".to_string(),
        effect: None,
    };

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(
            ObjectSpec::creature(p1, "Trigger Creature", 2, 2)
                .with_triggered_ability(triggered_ability),
        )
        .at_step(Step::DeclareAttackers)
        .active_player(p1)
        .build()
        .unwrap();

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
            enlist_choices: vec![],
        },
    )
    .expect("declare attackers failed");

    // Pass through DeclareAttackers → DeclareBlockers.
    let (state, _) = pass_all(state, &[p1, p2]);

    // p2 declares no blockers.
    let (state, _) = process_command(
        state,
        Command::DeclareBlockers {
            player: p2,
            blockers: vec![],
        },
    )
    .expect("declare blockers failed");

    // Pass through DeclareBlockers → CombatDamage (turn-based action fires).
    let (state, events) = pass_all(state, &[p1, p2]);

    // AbilityTriggered event should have been emitted for the creature.
    let triggered = events.iter().any(|e| {
        matches!(
            e,
            GameEvent::AbilityTriggered { source_object_id, .. }
            if *source_object_id == attacker_id
        )
    });
    assert!(
        triggered,
        "SelfDealsCombatDamageToPlayer trigger should fire for unblocked attacker"
    );

    // The triggered ability should be on the stack.
    assert_eq!(
        state.stack_objects.len(),
        1,
        "Triggered ability should be placed on the stack"
    );

    // p2's life should be reduced by 2.
    assert_eq!(
        state.players.get(&p2).unwrap().life_total,
        38,
        "p2 should have taken 2 combat damage"
    );
}

#[test]
/// CR 510.1c — a blocked creature (without trample) deals its damage to
/// blockers only, not to the defending player. The SelfDealsCombatDamageToPlayer
/// trigger must NOT fire because no combat damage was dealt to a player.
fn test_510_3a_combat_damage_trigger_does_not_fire_on_blocked_creature() {
    use mtg_engine::{TriggerEvent, TriggeredAbilityDef};

    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let triggered_ability = TriggeredAbilityDef {
        counter_filter: None,
        counter_on_self: false,
        once_per_turn: false,
        etb_filter: None,
        death_filter: None,
        combat_damage_filter: None,
        triggering_creature_filter: None,
        targets: vec![],
        trigger_on: TriggerEvent::SelfDealsCombatDamageToPlayer,
        intervening_if: None,
        description: "Whenever ~ deals combat damage to a player, do something".to_string(),
        effect: None,
    };

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(
            ObjectSpec::creature(p1, "Trigger Attacker", 2, 2)
                .with_triggered_ability(triggered_ability),
        )
        .object(ObjectSpec::creature(p2, "Blocker", 3, 3))
        .at_step(Step::DeclareAttackers)
        .active_player(p1)
        .build()
        .unwrap();

    let attacker_id = state
        .objects
        .values()
        .find(|o| o.controller == p1 && o.characteristics.name == "Trigger Attacker")
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
            enlist_choices: vec![],
        },
    )
    .expect("declare attackers failed");

    let (state, _) = pass_all(state, &[p1, p2]);

    let (state, _) = process_command(
        state,
        Command::DeclareBlockers {
            player: p2,
            blockers: vec![(blocker_id, attacker_id)],
        },
    )
    .expect("declare blockers failed");

    // Pass through DeclareBlockers → CombatDamage.
    let (state, events) = pass_all(state, &[p1, p2]);

    // No AbilityTriggered event for the attacker (damage went to the blocker).
    let triggered = events.iter().any(|e| {
        matches!(
            e,
            GameEvent::AbilityTriggered { source_object_id, .. }
            if *source_object_id == attacker_id
        )
    });
    assert!(
        !triggered,
        "SelfDealsCombatDamageToPlayer trigger must NOT fire when creature is blocked (no trample)"
    );

    // The stack should have no triggered abilities from the attacker.
    assert_eq!(
        state.stack_objects.len(),
        0,
        "No triggered ability should be on the stack"
    );

    // p2's life total should be unchanged (all damage went to the blocker).
    assert_eq!(
        state.players.get(&p2).unwrap().life_total,
        40,
        "p2 should not have taken any combat damage"
    );
}

#[test]
/// CR 510.1a — a creature with 0 power assigns no combat damage.
/// CR 603.2g — because no damage is dealt, the SelfDealsCombatDamageToPlayer
/// trigger does NOT fire (prevented/zero damage does not trigger).
fn test_510_3a_combat_damage_trigger_does_not_fire_when_damage_is_zero() {
    use mtg_engine::{TriggerEvent, TriggeredAbilityDef};

    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let triggered_ability = TriggeredAbilityDef {
        counter_filter: None,
        counter_on_self: false,
        once_per_turn: false,
        etb_filter: None,
        death_filter: None,
        combat_damage_filter: None,
        triggering_creature_filter: None,
        targets: vec![],
        trigger_on: TriggerEvent::SelfDealsCombatDamageToPlayer,
        intervening_if: None,
        description: "Whenever ~ deals combat damage to a player, do something".to_string(),
        effect: None,
    };

    // 0-power creature: assigns 0 damage (CR 510.1a).
    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(
            ObjectSpec::creature(p1, "Zero Power", 0, 2).with_triggered_ability(triggered_ability),
        )
        .at_step(Step::DeclareAttackers)
        .active_player(p1)
        .build()
        .unwrap();

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
            enlist_choices: vec![],
        },
    )
    .expect("declare attackers failed");

    let (state, _) = pass_all(state, &[p1, p2]);

    let (state, _) = process_command(
        state,
        Command::DeclareBlockers {
            player: p2,
            blockers: vec![],
        },
    )
    .expect("declare blockers failed");

    // Pass through DeclareBlockers → CombatDamage.
    let (state, events) = pass_all(state, &[p1, p2]);

    // No AbilityTriggered event — 0 damage was assigned, trigger does not fire.
    let triggered = events.iter().any(|e| {
        matches!(
            e,
            GameEvent::AbilityTriggered { source_object_id, .. }
            if *source_object_id == attacker_id
        )
    });
    assert!(
        !triggered,
        "SelfDealsCombatDamageToPlayer trigger must NOT fire when creature has 0 power"
    );

    // p2's life total should be unchanged.
    assert_eq!(
        state.players.get(&p2).unwrap().life_total,
        40,
        "p2 should not have taken any damage from a 0-power creature"
    );
}

#[test]
/// CR 603.2c — an ability triggers once for each time the trigger event occurs.
/// In multiplayer, two unblocked creatures attacking different players each fire
/// their SelfDealsCombatDamageToPlayer triggers independently. Both go on the
/// stack (CR 510.3a: abilities triggered on damage being dealt are placed on the
/// stack before priority is granted).
fn test_510_3a_combat_damage_trigger_multiplayer_separate_targets() {
    use mtg_engine::{TriggerEvent, TriggeredAbilityDef};

    let p1 = PlayerId(1);
    let p2 = PlayerId(2);
    let p3 = PlayerId(3);

    let make_trigger = || TriggeredAbilityDef {
        counter_filter: None,
        counter_on_self: false,
        once_per_turn: false,
        etb_filter: None,
        death_filter: None,
        combat_damage_filter: None,
        triggering_creature_filter: None,
        targets: vec![],
        trigger_on: TriggerEvent::SelfDealsCombatDamageToPlayer,
        intervening_if: None,
        description: "Whenever ~ deals combat damage to a player, do something".to_string(),
        effect: None,
    };

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .object(ObjectSpec::creature(p1, "Creature A", 2, 2).with_triggered_ability(make_trigger()))
        .object(ObjectSpec::creature(p1, "Creature B", 3, 3).with_triggered_ability(make_trigger()))
        .at_step(Step::DeclareAttackers)
        .active_player(p1)
        .build()
        .unwrap();

    let att_a_id = state
        .objects
        .values()
        .find(|o| o.controller == p1 && o.characteristics.name == "Creature A")
        .unwrap()
        .id;
    let att_b_id = state
        .objects
        .values()
        .find(|o| o.controller == p1 && o.characteristics.name == "Creature B")
        .unwrap()
        .id;

    // Creature A attacks p2; Creature B attacks p3.
    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![
                (att_a_id, AttackTarget::Player(p2)),
                (att_b_id, AttackTarget::Player(p3)),
            ],
            enlist_choices: vec![],
        },
    )
    .expect("declare attackers failed");

    let (state, _) = pass_all(state, &[p1, p2, p3]);

    // Both p2 and p3 declare no blockers.
    let (state, _) = process_command(
        state,
        Command::DeclareBlockers {
            player: p2,
            blockers: vec![],
        },
    )
    .expect("p2 declare blockers failed");

    let (state, _) = process_command(
        state,
        Command::DeclareBlockers {
            player: p3,
            blockers: vec![],
        },
    )
    .expect("p3 declare blockers failed");

    // Pass through DeclareBlockers → CombatDamage.
    let (state, events) = pass_all(state, &[p1, p2, p3]);

    // Both creatures should have fired their triggers.
    let triggered_a = events.iter().any(|e| {
        matches!(
            e,
            GameEvent::AbilityTriggered { source_object_id, .. }
            if *source_object_id == att_a_id
        )
    });
    let triggered_b = events.iter().any(|e| {
        matches!(
            e,
            GameEvent::AbilityTriggered { source_object_id, .. }
            if *source_object_id == att_b_id
        )
    });

    assert!(
        triggered_a,
        "Creature A's SelfDealsCombatDamageToPlayer trigger should fire"
    );
    assert!(
        triggered_b,
        "Creature B's SelfDealsCombatDamageToPlayer trigger should fire"
    );

    // Both triggered abilities should be on the stack.
    assert_eq!(
        state.stack_objects.len(),
        2,
        "Both triggered abilities should be placed on the stack (one per creature)"
    );

    // p2 and p3 both took damage; p1 is untouched.
    assert_eq!(
        state.players.get(&p2).unwrap().life_total,
        38,
        "p2 should have taken 2 damage from Creature A"
    );
    assert_eq!(
        state.players.get(&p3).unwrap().life_total,
        37,
        "p3 should have taken 3 damage from Creature B"
    );
    assert_eq!(
        state.players.get(&p1).unwrap().life_total,
        40,
        "p1 (the attacker) should be untouched"
    );
}

// ---------------------------------------------------------------------------
// Tests: First-strike keyword snapshot (CR 702.7b, 702.7c, 702.4c/d)
// ---------------------------------------------------------------------------

#[test]
/// CR 702.7b — At the start of the first-strike damage step, the engine snapshots
/// which creatures have FirstStrike or DoubleStrike. This snapshot (stored in
/// `CombatState::first_strike_participants`) is used to determine regular-step
/// eligibility — not the creatures' current keywords.
///
/// Scenario: p1 attacks with a 2/1 FirstStrike creature, unblocked.
/// - First-strike step: the FS creature deals 2 damage to p2.
/// - After the step, `first_strike_participants` must contain the attacker's id.
/// - Regular step: the FS creature was in the snapshot → excluded from regular step.
/// - p2 should take exactly 2 damage total (not 4).
fn test_702_7b_first_strike_snapshot_populated_and_excludes_regular_step() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(
            ObjectSpec::creature(p1, "FSAttacker", 2, 1).with_keyword(KeywordAbility::FirstStrike),
        )
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let attacker_id = state
        .objects
        .values()
        .find(|o| o.characteristics.name == "FSAttacker")
        .unwrap()
        .id;

    // Declare the attacker.
    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(attacker_id, AttackTarget::Player(p2))],
            enlist_choices: vec![],
        },
    )
    .unwrap();

    // Pass through DeclareAttackers → DeclareBlockers.
    let (state, _) = pass_all(state, &[p1, p2]);
    // Pass through DeclareBlockers → FirstStrikeDamage step.
    let (state, _) = pass_all(state, &[p1, p2]);
    assert_eq!(
        state.turn.step,
        Step::FirstStrikeDamage,
        "should be in FirstStrikeDamage step"
    );

    // CR 702.7b: After the first-strike step fires, the snapshot must be populated.
    let snapshot = &state.combat.as_ref().unwrap().first_strike_participants;
    assert!(
        snapshot.contains(&attacker_id),
        "CR 702.7b: attacker with FirstStrike must appear in first_strike_participants snapshot"
    );

    // p2 should have taken 2 damage from the first-strike step.
    assert_eq!(
        state.players.get(&p2).unwrap().life_total,
        38,
        "p2 should have taken 2 damage in the first-strike step"
    );

    // Pass through FirstStrikeDamage → CombatDamage step.
    let (state, regular_events) = pass_all(state, &[p1, p2]);
    assert_eq!(state.turn.step, Step::CombatDamage);

    // CR 702.7b/702.7c: FS attacker was in snapshot → excluded from regular step.
    // No damage should be dealt in the regular step.
    let regular_damage: u32 = regular_events
        .iter()
        .filter_map(|e| {
            if let GameEvent::CombatDamageDealt { assignments } = e {
                Some(assignments.iter().map(|a| a.amount).sum::<u32>())
            } else {
                None
            }
        })
        .sum();

    assert_eq!(
        regular_damage, 0,
        "CR 702.7b: FS attacker (in snapshot) must not deal damage in the regular step; \
         got {} damage",
        regular_damage
    );

    // p2 should still have exactly 38 life (2 damage total, not 4).
    assert_eq!(
        state.players.get(&p2).unwrap().life_total,
        38,
        "p2 should have taken exactly 2 total damage (FS step only, not regular step)"
    );
}

#[test]
/// CR 702.7c — A creature that does NOT have first strike or double strike at the
/// start of the first-strike step deals damage in the regular step normally,
/// even if it gains first strike after the snapshot was taken.
///
/// Scenario: p1 attacks with a 3/3 normal creature (no FS) vs. a 2/1 FirstStrike blocker.
/// - First-strike step: blocker (FS, in snapshot) deals 2 to attacker. Attacker not in snapshot.
/// - The blocker's 2 damage is lethal to the 3/3 attacker? No — 3/3 has 3 toughness. Survives.
/// - Regular step: attacker (NOT in snapshot) deals 3 to blocker. Blocker (in snapshot,
///   only FS not DS) does NOT deal damage again.
/// - Result: blocker takes 3 lethal damage; attacker took 2 from FS step.
fn test_702_7c_normal_creature_not_in_snapshot_deals_regular_damage() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(ObjectSpec::creature(p1, "NormalAttacker", 3, 3))
        .object(
            ObjectSpec::creature(p2, "FSBlocker", 2, 1).with_keyword(KeywordAbility::FirstStrike),
        )
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let attacker_id = state
        .objects
        .values()
        .find(|o| o.characteristics.name == "NormalAttacker")
        .unwrap()
        .id;
    let blocker_id = state
        .objects
        .values()
        .find(|o| o.characteristics.name == "FSBlocker")
        .unwrap()
        .id;

    // Declare attackers.
    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(attacker_id, AttackTarget::Player(p2))],
            enlist_choices: vec![],
        },
    )
    .unwrap();

    // Pass through DeclareAttackers → DeclareBlockers.
    let (state, _) = pass_all(state, &[p1, p2]);

    // p2 declares the blocker.
    let (state, _) = process_command(
        state,
        Command::DeclareBlockers {
            player: p2,
            blockers: vec![(blocker_id, attacker_id)],
        },
    )
    .unwrap();

    // Pass through DeclareBlockers → FirstStrikeDamage step.
    let (state, fs_events) = pass_all(state, &[p1, p2]);
    assert_eq!(state.turn.step, Step::FirstStrikeDamage);

    // CR 702.7b: blocker (FS) must be in snapshot; attacker (normal) must NOT be.
    let snapshot = &state.combat.as_ref().unwrap().first_strike_participants;
    assert!(
        snapshot.contains(&blocker_id),
        "CR 702.7b: FS blocker must appear in snapshot"
    );
    assert!(
        !snapshot.contains(&attacker_id),
        "CR 702.7b: normal attacker (no FS/DS) must NOT appear in snapshot"
    );

    // First-strike step: only the blocker deals damage (2 to attacker).
    let fs_damage: u32 = fs_events
        .iter()
        .filter_map(|e| {
            if let GameEvent::CombatDamageDealt { assignments } = e {
                Some(assignments.iter().map(|a| a.amount).sum::<u32>())
            } else {
                None
            }
        })
        .sum();
    assert_eq!(fs_damage, 2, "only FS blocker deals 2 in first-strike step");

    // Pass through FirstStrikeDamage → CombatDamage step.
    let (state, regular_events) = pass_all(state, &[p1, p2]);
    assert_eq!(state.turn.step, Step::CombatDamage);

    // CR 702.7c: normal attacker (NOT in snapshot) deals 3 in regular step.
    // FS blocker (in snapshot, no DS) does NOT deal damage again.
    let regular_damage: u32 = regular_events
        .iter()
        .filter_map(|e| {
            if let GameEvent::CombatDamageDealt { assignments } = e {
                Some(assignments.iter().map(|a| a.amount).sum::<u32>())
            } else {
                None
            }
        })
        .sum();
    assert_eq!(
        regular_damage, 3,
        "CR 702.7c: normal attacker (not in snapshot) deals 3 in regular step; \
         FS blocker (in snapshot) does not deal again"
    );

    // FSBlocker should be dead (took 3, toughness 1).
    let blocker_alive = state.objects.values().any(|o| {
        o.characteristics.name == "FSBlocker" && o.zone == mtg_engine::ZoneId::Battlefield
    });
    assert!(
        !blocker_alive,
        "CR 702.7c: FSBlocker (1 toughness) should be dead after taking 3 regular damage"
    );
}

// ── SR-FS-03: First-strike attacker vs first-strike blocker ───────────────────

#[test]
/// SR-FS-03 / CR 702.7b — When both attacker AND blocker have first strike, both
/// deal their damage simultaneously in the first-strike damage step. Neither
/// creature appears in the regular combat damage step.
///
/// Scenario: p1 attacks with a 2/2 FirstStrike creature; p2 blocks with a 2/2
/// FirstStrike creature. Both deal 2 damage simultaneously in the FS step
/// (both die, since 2 damage ≥ toughness 2). No creature deals damage in the
/// regular step.
///
/// CR 702.7b: "A creature with first strike deals combat damage before creatures
/// without first strike." Since both have first strike, they deal simultaneously
/// in the first-strike step.
fn test_sr_fs03_first_strike_vs_first_strike_damage_only_in_fs_step() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(
            ObjectSpec::creature(p1, "FS Attacker", 2, 2).with_keyword(KeywordAbility::FirstStrike),
        )
        .object(
            ObjectSpec::creature(p2, "FS Blocker", 2, 2).with_keyword(KeywordAbility::FirstStrike),
        )
        .at_step(Step::DeclareAttackers)
        .active_player(p1)
        .build()
        .unwrap();

    let attacker_id = state
        .objects
        .values()
        .find(|o| o.characteristics.name == "FS Attacker")
        .unwrap()
        .id;
    let blocker_id = state
        .objects
        .values()
        .find(|o| o.characteristics.name == "FS Blocker")
        .unwrap()
        .id;

    // Declare attacker.
    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(attacker_id, AttackTarget::Player(p2))],
            enlist_choices: vec![],
        },
    )
    .unwrap();

    // Pass through DeclareAttackers → DeclareBlockers.
    let (state, _) = pass_all(state, &[p1, p2]);

    // p2 declares the FS Blocker as blocker.
    let (state, _) = process_command(
        state,
        Command::DeclareBlockers {
            player: p2,
            blockers: vec![(blocker_id, attacker_id)],
        },
    )
    .unwrap();

    // Pass through DeclareBlockers → FirstStrikeDamage step.
    // Both creatures have first strike → both deal damage in this step.
    let (state, fs_events) = pass_all(state, &[p1, p2]);
    assert_eq!(
        state.turn.step,
        Step::FirstStrikeDamage,
        "SR-FS-03: should be in FirstStrikeDamage step"
    );

    // SR-FS-03: Both deal 2 damage simultaneously in the FS step.
    // Both have 2 toughness → both die from 2 damage.
    let deaths_in_fs_step = fs_events
        .iter()
        .filter(|e| matches!(e, GameEvent::CreatureDied { .. }))
        .count();
    assert_eq!(
        deaths_in_fs_step, 2,
        "SR-FS-03 / CR 702.7b: both first-strike creatures should die in the \
         first-strike step (simultaneous damage); got {}",
        deaths_in_fs_step
    );

    // Verify total first-strike damage was 2+2=4 (each dealing 2 power).
    let fs_damage_total: u32 = fs_events
        .iter()
        .filter_map(|e| {
            if let GameEvent::CombatDamageDealt { assignments } = e {
                Some(assignments.iter().map(|a| a.amount).sum::<u32>())
            } else {
                None
            }
        })
        .sum();
    assert_eq!(
        fs_damage_total, 4,
        "SR-FS-03: total first-strike damage should be 4 (2 from each creature); got {}",
        fs_damage_total
    );

    // Pass through FirstStrikeDamage → CombatDamage.
    // Both creatures are now dead → neither deals regular damage.
    let (state, regular_events) = pass_all(state, &[p1, p2]);
    assert_eq!(
        state.turn.step,
        Step::CombatDamage,
        "SR-FS-03: should advance to CombatDamage step"
    );

    // SR-FS-03: No creature should deal regular damage (both are dead).
    let regular_damage_total: u32 = regular_events
        .iter()
        .filter_map(|e| {
            if let GameEvent::CombatDamageDealt { assignments } = e {
                Some(assignments.iter().map(|a| a.amount).sum::<u32>())
            } else {
                None
            }
        })
        .sum();
    assert_eq!(
        regular_damage_total, 0,
        "SR-FS-03 / CR 702.7b: neither first-strike creature should deal regular \
         combat damage (both died in the first-strike step); got {}",
        regular_damage_total
    );

    // Both creatures must be off the battlefield.
    // Check using original ObjectIds (CR 400.7: zone change creates new object ID).
    // After death, the original IDs are gone from state.objects.
    let attacker_alive = state.objects.values().any(|o| o.id == attacker_id);
    let blocker_alive = state.objects.values().any(|o| o.id == blocker_id);
    assert!(
        !attacker_alive,
        "SR-FS-03: FS Attacker should have died in the first-strike step (original ID {} gone)",
        attacker_id.0
    );
    assert!(
        !blocker_alive,
        "SR-FS-03: FS Blocker should have died in the first-strike step (original ID {} gone)",
        blocker_id.0
    );
}

// ── SR-FS-02: First strike gained between the two combat damage steps ─────────

#[test]
/// SR-FS-02 / CR 702.7c — A creature that gains first strike AFTER the first-strike
/// damage step has begun (i.e., between the two combat damage steps) is not retro-
/// actively added to the first-strike step. It already missed it, and it still deals
/// its damage normally in the regular combat damage step.
///
/// CR 702.7c: "A creature that's removed from combat ... won't assign combat damage
/// ... A creature that gains first strike or double strike after [the first-strike]
/// combat damage step has begun won't [retroactively deal damage in it]." The engine
/// implements this with `first_strike_participants` — a snapshot taken at the start
/// of the first-strike step. Membership, not current keywords, decides regular-step
/// eligibility.
///
/// Scenario: p1 attacks p2 with two unblocked creatures — a 2/2 FirstStrike "Vanguard"
/// (forces a first-strike step and a snapshot) and a 3/3 plain "Latecomer". After the
/// first-strike step resolves (only Vanguard deals damage), a continuous effect grants
/// Latecomer first strike — simulating an instant cast in the gap between the steps.
/// Latecomer must still deal its 3 damage in the regular step (it was never in the
/// snapshot), and Vanguard must not deal damage again.
fn test_sr_fs02_first_strike_gained_between_damage_steps() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(
            ObjectSpec::creature(p1, "Vanguard", 2, 2).with_keyword(KeywordAbility::FirstStrike),
        )
        .object(ObjectSpec::creature(p1, "Latecomer", 3, 3))
        .at_step(Step::DeclareAttackers)
        .active_player(p1)
        .build()
        .unwrap();

    let vanguard_id = state
        .objects
        .values()
        .find(|o| o.characteristics.name == "Vanguard")
        .unwrap()
        .id;
    let latecomer_id = state
        .objects
        .values()
        .find(|o| o.characteristics.name == "Latecomer")
        .unwrap()
        .id;

    // Declare both attackers at p2.
    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![
                (vanguard_id, AttackTarget::Player(p2)),
                (latecomer_id, AttackTarget::Player(p2)),
            ],
            enlist_choices: vec![],
        },
    )
    .unwrap();

    // DeclareAttackers → DeclareBlockers; p2 declares no blockers.
    let (state, _) = pass_all(state, &[p1, p2]);
    let (state, _) = process_command(
        state,
        Command::DeclareBlockers {
            player: p2,
            blockers: vec![],
        },
    )
    .unwrap();

    // DeclareBlockers → FirstStrikeDamage step. Only Vanguard (FS) deals damage.
    let (mut state, _) = pass_all(state, &[p1, p2]);
    assert_eq!(
        state.turn.step,
        Step::FirstStrikeDamage,
        "SR-FS-02: should be in FirstStrikeDamage step"
    );

    // CR 702.7b: the snapshot must contain Vanguard (had FS) but not Latecomer.
    {
        let snapshot = &state.combat.as_ref().unwrap().first_strike_participants;
        assert!(
            snapshot.contains(&vanguard_id),
            "SR-FS-02: Vanguard (FirstStrike) must be in the first-strike snapshot"
        );
        assert!(
            !snapshot.contains(&latecomer_id),
            "SR-FS-02: Latecomer (no FirstStrike at step start) must NOT be in the snapshot"
        );
    }
    assert_eq!(
        state.players.get(&p2).unwrap().life_total,
        38,
        "SR-FS-02: p2 takes only Vanguard's 2 damage in the first-strike step"
    );

    // --- Grant Latecomer first strike BETWEEN the two damage steps ---
    // Simulates an instant-speed grant (e.g. a spell) cast during the priority
    // window after the first-strike step. CR 702.7c: this must NOT retroactively
    // place Latecomer into the first-strike step.
    state.continuous_effects.push_back(ContinuousEffect {
        id: EffectId(9_000_002),
        source: None,
        timestamp: 9_000_002,
        layer: EffectLayer::Ability,
        duration: EffectDuration::UntilEndOfTurn,
        filter: EffectFilter::SingleObject(latecomer_id),
        modification: LayerModification::AddKeyword(KeywordAbility::FirstStrike),
        is_cda: false,
        condition: None,
    });

    // Sanity: the grant is live — Latecomer now has first strike.
    let latecomer_chars = mtg_engine::calculate_characteristics(&state, latecomer_id).unwrap();
    assert!(
        latecomer_chars
            .keywords
            .contains(&KeywordAbility::FirstStrike),
        "SR-FS-02: continuous effect should have granted Latecomer first strike"
    );

    // FirstStrikeDamage → CombatDamage. Latecomer (not in snapshot) still deals
    // its 3 damage here despite now having first strike; Vanguard (in snapshot,
    // FS only) does not deal damage again.
    let (state, regular_events) = pass_all(state, &[p1, p2]);
    assert_eq!(
        state.turn.step,
        Step::CombatDamage,
        "SR-FS-02: should advance to CombatDamage step"
    );

    let regular_damage: u32 = regular_events
        .iter()
        .filter_map(|e| {
            if let GameEvent::CombatDamageDealt { assignments } = e {
                Some(assignments.iter().map(|a| a.amount).sum::<u32>())
            } else {
                None
            }
        })
        .sum();
    assert_eq!(
        regular_damage, 3,
        "SR-FS-02 / CR 702.7c: Latecomer (gained FS after the snapshot) still deals \
         its 3 damage in the regular step; Vanguard does not deal damage again"
    );
    assert_eq!(
        state.players.get(&p2).unwrap().life_total,
        35,
        "SR-FS-02: p2 ends at 40 - 2 (FS step) - 3 (regular step) = 35"
    );
}

// ── MR-M6-13: attacker blocked, all blockers gone before damage, no trample ───

#[test]
/// MR-M6-13 / CR 509.1h — A creature that has been blocked remains blocked even
/// after every creature blocking it leaves combat. Without trample, such an
/// attacker deals NO combat damage: not to its (now-absent) blockers, and not to
/// the defending player.
///
/// Scenario: p1 attacks p2 with a plain 3/3 "Grizzly" (no trample, no first strike).
/// p2 blocks with a 1/1 "Wall". Before the combat damage step, the Wall is destroyed
/// (here, a continuous effect drops its toughness to 0, simulating a removal spell
/// cast during the declare-blockers priority window). In the combat damage step the
/// Grizzly is still "blocked" (CR 509.1h) but has no blockers and no trample, so it
/// deals no damage at all.
fn test_mr_m6_13_blocked_attacker_blockers_removed_no_trample() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(ObjectSpec::creature(p1, "Grizzly", 3, 3))
        .object(ObjectSpec::creature(p2, "Wall", 1, 1))
        .at_step(Step::DeclareAttackers)
        .active_player(p1)
        .build()
        .unwrap();

    let attacker_id = state
        .objects
        .values()
        .find(|o| o.characteristics.name == "Grizzly")
        .unwrap()
        .id;
    let blocker_id = state
        .objects
        .values()
        .find(|o| o.characteristics.name == "Wall")
        .unwrap()
        .id;

    // Declare the attacker at p2.
    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(attacker_id, AttackTarget::Player(p2))],
            enlist_choices: vec![],
        },
    )
    .unwrap();

    // DeclareAttackers → DeclareBlockers.
    let (state, _) = pass_all(state, &[p1, p2]);

    // p2 blocks the Grizzly with the Wall.
    let (mut state, _) = process_command(
        state,
        Command::DeclareBlockers {
            player: p2,
            blockers: vec![(blocker_id, attacker_id)],
        },
    )
    .unwrap();

    // The attacker is now recorded as blocked (CR 509.1h).
    assert!(
        state.combat.as_ref().unwrap().is_blocked(attacker_id),
        "MR-M6-13: Grizzly should be marked blocked after declaration"
    );

    // --- Remove the blocker before the combat damage step ---
    // A continuous effect drops the Wall's toughness to 0; the next SBA check
    // destroys it (CR 704.5f). This simulates a removal spell resolving during
    // the declare-blockers priority window.
    state.continuous_effects.push_back(ContinuousEffect {
        id: EffectId(9_000_013),
        source: None,
        timestamp: 9_000_013,
        layer: EffectLayer::PtModify,
        duration: EffectDuration::UntilEndOfTurn,
        filter: EffectFilter::SingleObject(blocker_id),
        modification: LayerModification::ModifyToughness(-1),
        is_cda: false,
        condition: None,
    });

    // DeclareBlockers → CombatDamage. The SBA fires on the way, destroying the Wall.
    let (state, events) = pass_all(state, &[p1, p2]);
    assert_eq!(
        state.turn.step,
        Step::CombatDamage,
        "MR-M6-13: should advance to the CombatDamage step"
    );

    // The Wall is gone (CR 400.7: original ObjectId no longer on the battlefield).
    let wall_alive = state.objects.values().any(|o| o.id == blocker_id);
    assert!(
        !wall_alive,
        "MR-M6-13: the Wall should have been destroyed by the 0-toughness SBA"
    );

    // CR 509.1h: the Grizzly is still blocked even though no blockers remain.
    assert!(
        state.combat.as_ref().unwrap().is_blocked(attacker_id),
        "MR-M6-13: Grizzly remains blocked after its blocker left combat (CR 509.1h)"
    );

    // No combat damage was dealt to the player — blocked, blockers gone, no trample.
    let player_damage: u32 = events
        .iter()
        .filter_map(|e| {
            if let GameEvent::CombatDamageDealt { assignments } = e {
                Some(
                    assignments
                        .iter()
                        .filter(|a| matches!(a.target, CombatDamageTarget::Player(_)))
                        .map(|a| a.amount)
                        .sum::<u32>(),
                )
            } else {
                None
            }
        })
        .sum();
    assert_eq!(
        player_damage, 0,
        "MR-M6-13 / CR 509.1h: a blocked attacker with no blockers and no trample \
         deals no combat damage to the player"
    );
    assert_eq!(
        state.players.get(&p2).unwrap().life_total,
        40,
        "MR-M6-13: p2 takes no damage (blocked Grizzly, blockers gone, no trample)"
    );
}

// ── SR-TRM-01: planeswalker combat damage removes loyalty counters ────────────

#[test]
/// SR-TRM-01 / CR 120.3c — Combat damage dealt to a planeswalker causes that many
/// loyalty counters to be removed from it. The damage is NOT marked on the
/// planeswalker object as it would be on a creature.
///
/// Scenario: p1 attacks p2's planeswalker (5 loyalty) with a 3/3 creature. After
/// combat damage, the planeswalker has 5 - 3 = 2 loyalty counters and zero marked
/// damage.
fn test_sr_trm01_planeswalker_combat_damage_removes_loyalty() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(ObjectSpec::creature(p1, "Raider", 3, 3))
        .object(ObjectSpec::planeswalker(p2, "Garruk", 5).with_counter(CounterType::Loyalty, 5))
        .at_step(Step::DeclareAttackers)
        .active_player(p1)
        .build()
        .unwrap();

    let attacker_id = state
        .objects
        .values()
        .find(|o| o.characteristics.name == "Raider")
        .unwrap()
        .id;
    let pw_id = state
        .objects
        .values()
        .find(|o| o.characteristics.name == "Garruk")
        .unwrap()
        .id;

    // Declare the attacker at the planeswalker.
    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(attacker_id, AttackTarget::Planeswalker(pw_id))],
            enlist_choices: vec![],
        },
    )
    .unwrap();

    // DeclareAttackers → DeclareBlockers; p2 declares no blockers.
    let (state, _) = pass_all(state, &[p1, p2]);
    let (state, _) = process_command(
        state,
        Command::DeclareBlockers {
            player: p2,
            blockers: vec![],
        },
    )
    .unwrap();

    // DeclareBlockers → CombatDamage (no first strike → single damage step).
    let (state, _) = pass_all(state, &[p1, p2]);
    assert_eq!(
        state.turn.step,
        Step::CombatDamage,
        "SR-TRM-01: should be in the CombatDamage step"
    );

    let pw = state
        .objects
        .get(&pw_id)
        .expect("SR-TRM-01: planeswalker should still be on the battlefield (5 - 3 = 2 loyalty)");

    // CR 120.3c: 3 loyalty counters removed (5 → 2).
    assert_eq!(
        pw.counters.get(&CounterType::Loyalty).copied().unwrap_or(0),
        2,
        "SR-TRM-01 / CR 120.3c: 3 combat damage removes 3 loyalty counters (5 → 2)"
    );
    // The damage must NOT be marked on the planeswalker object.
    assert_eq!(
        pw.damage_marked, 0,
        "SR-TRM-01: combat damage to a planeswalker removes loyalty counters, \
         it is not marked as damage on the object"
    );
}
