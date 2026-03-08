//! Commander damage tracking tests (CR 903.10a / CR 704.6c).
//!
//! Session 4: Commander Damage & Partner Commanders.
//!
//! Tests verify:
//! - 21 combat damage from one commander triggers SBA loss (CR 704.6c)
//! - 20 damage from one commander is not enough
//! - Damage from two different commanders does NOT add up (tracked per-commander)
//! - A copy (clone) of a commander does NOT deal commander damage (CR 903.3)
//! - Commander damage survives zone changes (CardId-based, not ObjectId-based)

use mtg_engine::{
    check_and_apply_sbas, process_command, AttackTarget, CardId, Command, GameEvent, GameState,
    GameStateBuilder, ObjectSpec, PlayerId, Step, ZoneId,
};

// ── Helpers ───────────────────────────────────────────────────────────────────

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

fn cid(s: &str) -> CardId {
    CardId(s.to_string())
}

/// Pass priority for every player in sequence (one full round).
fn pass_all(state: GameState, players: &[PlayerId]) -> (GameState, Vec<GameEvent>) {
    let mut all_events = Vec::new();
    let mut current = state;
    for &player in players {
        let (s, ev) = process_command(current, Command::PassPriority { player })
            .unwrap_or_else(|e| panic!("PassPriority by {:?} failed: {:?}", player, e));
        current = s;
        all_events.extend(ev);
    }
    (current, all_events)
}

/// Run a single unblocked combat where `attacker` (owned by `attacker_player`)
/// attacks `defending_player`. Returns the state after combat damage resolves.
fn run_one_unblocked_combat(
    state: GameState,
    attacker_player: PlayerId,
    defending_player: PlayerId,
    attacker_id: mtg_engine::ObjectId,
    all_players: &[PlayerId],
) -> (GameState, Vec<GameEvent>) {
    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: attacker_player,
            attackers: vec![(attacker_id, AttackTarget::Player(defending_player))],
            enlist_choices: vec![],
        },
    )
    .unwrap();

    let (state, _) = pass_all(state, all_players);

    let (state, _) = process_command(
        state,
        Command::DeclareBlockers {
            player: defending_player,
            blockers: vec![],
        },
    )
    .unwrap();

    pass_all(state, all_players)
}

// ── CR 704.6c: Commander damage SBA loss ─────────────────────────────────────

#[test]
/// CR 704.6c / CR 903.10a — Player who has received exactly 21 combat damage
/// from a single commander loses the game as a state-based action.
fn test_commander_damage_21_from_one_commander_kills() {
    let p1 = p(1);
    let p2 = p(2);
    let cmd_id = cid("big-commander");

    // Pre-set p2 as having received 20 commander damage already, then deal 1 more.
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .player_commander(p1, cmd_id.clone())
        // 7/7 commander — will deal 1 damage this combat to push total to 21
        .object(ObjectSpec::creature(p1, "Big Commander", 7, 7).with_card_id(cmd_id.clone()))
        .at_step(Step::DeclareAttackers)
        .active_player(p1)
        .build()
        .unwrap();

    // Pre-set 20 damage already received from this commander.
    {
        let p2_state = state.players.get_mut(&p2).unwrap();
        let inner = im::OrdMap::from(vec![(cmd_id.clone(), 20u32)]);
        p2_state.commander_damage_received.insert(p1, inner);
    }

    let attacker_id = state
        .objects
        .values()
        .find(|o| o.controller == p1)
        .unwrap()
        .id;

    let (state, _) = run_one_unblocked_combat(state, p1, p2, attacker_id, &[p1, p2]);

    // After combat damage, p2 has 27 damage total (20 + 7), which triggers SBA loss.
    // SBA is checked after damage (before priority). Check p2's loss state.
    let p2_state = state.players.get(&p2).unwrap();
    assert!(
        p2_state.has_lost,
        "p2 should have lost due to 27 commander damage (>= 21); \
         commander_damage_received = {:?}",
        p2_state.commander_damage_received
    );
}

#[test]
/// CR 903.10a — Exactly 20 combat damage from one commander does NOT cause loss.
/// The threshold is 21 or more.
fn test_commander_damage_20_from_one_commander_no_loss() {
    let p1 = p(1);
    let p2 = p(2);
    let cmd_id = cid("medium-commander");

    // Pre-set p2 as having received 15 commander damage; a 5/5 commander deals 5 more = 20.
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .player_commander(p1, cmd_id.clone())
        .object(ObjectSpec::creature(p1, "Medium Commander", 5, 5).with_card_id(cmd_id.clone()))
        .at_step(Step::DeclareAttackers)
        .active_player(p1)
        .build()
        .unwrap();

    {
        let p2_state = state.players.get_mut(&p2).unwrap();
        let inner = im::OrdMap::from(vec![(cmd_id.clone(), 15u32)]);
        p2_state.commander_damage_received.insert(p1, inner);
    }

    let attacker_id = state
        .objects
        .values()
        .find(|o| o.controller == p1)
        .unwrap()
        .id;

    let (state, _) = run_one_unblocked_combat(state, p1, p2, attacker_id, &[p1, p2]);

    let p2_state = state.players.get(&p2).unwrap();
    // 15 + 5 = 20: still not enough to lose.
    let total_damage = p2_state
        .commander_damage_received
        .get(&p1)
        .and_then(|m| m.get(&cmd_id))
        .copied()
        .unwrap_or(0);
    assert_eq!(
        total_damage, 20,
        "p2 should have accumulated 20 commander damage"
    );
    assert!(
        !p2_state.has_lost,
        "p2 should NOT have lost with only 20 commander damage"
    );
}

#[test]
/// CR 903.10a — Commander damage is tracked per-commander.
/// 10 damage from commander A + 11 damage from commander B = no loss.
/// Neither individual total reaches 21.
fn test_commander_damage_10_from_a_plus_11_from_b_no_loss() {
    let p1 = p(1);
    let p2 = p(2);
    let p3 = p(3);
    let cmd_a = cid("commander-a");
    let cmd_b = cid("commander-b");

    // Pre-set p3 as having received 10 from p1's commander and 11 from p2's commander.
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .player_commander(p1, cmd_a.clone())
        .player_commander(p2, cmd_b.clone())
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();

    {
        let p3_state = state.players.get_mut(&p3).unwrap();
        let from_p1 = im::OrdMap::from(vec![(cmd_a.clone(), 10u32)]);
        let from_p2 = im::OrdMap::from(vec![(cmd_b.clone(), 11u32)]);
        p3_state.commander_damage_received.insert(p1, from_p1);
        p3_state.commander_damage_received.insert(p2, from_p2);
    }

    // Run SBA check — neither commander's total is >= 21.
    let events = check_and_apply_sbas(&mut state);

    let p3_state = state.players.get(&p3).unwrap();
    assert!(
        !p3_state.has_lost,
        "p3 should NOT have lost: 10 from A + 11 from B are tracked separately; events: {:?}",
        events
    );
}

#[test]
/// CR 903.3 — A copy (clone) of a commander does NOT deal commander damage.
///
/// The commander designation is on the physical card (CardId), not copiable.
/// A copy of a commander has no CardId in any player's commander_ids, so
/// combat damage from the copy is NOT tracked as commander damage.
fn test_commander_damage_from_copy_does_not_count() {
    let p1 = p(1);
    let p2 = p(2);
    let cmd_id = cid("original-commander");

    // p1's commander is registered under cmd_id.
    // The "copy" creature has NO card_id — simulating a copied object.
    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .player_commander(p1, cmd_id.clone())
        // The original commander (registered) — NOT in this test's attack.
        .object(ObjectSpec::creature(p1, "Original Commander", 6, 6).with_card_id(cmd_id.clone()))
        // The copy — same power/toughness but NO card_id registered as commander.
        .object(ObjectSpec::creature(p1, "Copy of Commander", 6, 6))
        .at_step(Step::DeclareAttackers)
        .active_player(p1)
        .build()
        .unwrap();

    // Find the copy (no card_id).
    let copy_id = state
        .objects
        .values()
        .find(|o| o.controller == p1 && o.card_id.is_none())
        .unwrap()
        .id;

    // Declare the COPY as attacker (not the original).
    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(copy_id, AttackTarget::Player(p2))],
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

    // The copy dealt 6 damage to p2 as normal combat damage (life total reduced).
    let p2_life = state.players.get(&p2).unwrap().life_total;
    assert_eq!(
        p2_life, 34,
        "p2 should have taken 6 normal damage from the copy"
    );

    // But NO commander damage was tracked for p2.
    let p2_cmd_damage = state
        .players
        .get(&p2)
        .unwrap()
        .commander_damage_received
        .get(&p1)
        .and_then(|m| m.get(&cmd_id))
        .copied()
        .unwrap_or(0);
    assert_eq!(
        p2_cmd_damage, 0,
        "copy damage should NOT count as commander damage (CR 903.3)"
    );
}

#[test]
/// CR 903.10a — Commander damage accumulates across zone changes.
///
/// A commander that dies (and returns to the command zone via SBA), then
/// is re-cast (new ObjectId, same CardId), continues to accumulate commander
/// damage toward the same total for the defending player.
fn test_commander_damage_survives_zone_change() {
    let p1 = p(1);
    let p2 = p(2);
    let cmd_id = cid("persistent-commander");

    // Start: commander is on the battlefield.
    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .player_commander(p1, cmd_id.clone())
        .object(ObjectSpec::creature(p1, "Persistent Commander", 7, 7).with_card_id(cmd_id.clone()))
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

    // Round 1: deal 7 commander damage to p2.
    let (state, _) = run_one_unblocked_combat(state, p1, p2, attacker_id, &[p1, p2]);

    // Verify 7 damage accumulated.
    let damage_after_r1 = state
        .players
        .get(&p2)
        .unwrap()
        .commander_damage_received
        .get(&p1)
        .and_then(|m| m.get(&cmd_id))
        .copied()
        .unwrap_or(0);
    assert_eq!(
        damage_after_r1, 7,
        "p2 should have 7 commander damage after round 1"
    );

    // Simulate the commander dying: manually move it to the graveyard.
    // Also reset combat state so the second combat can proceed cleanly.
    let mut state = state;
    // Clear the combat state from round 1 (would be cleared by EndOfCombat in a real game).
    state.combat = None;
    let commander_obj_id = state
        .objects
        .values()
        .find(|o| o.card_id.as_ref() == Some(&cmd_id))
        .map(|o| o.id)
        .expect("commander should be on battlefield");

    state
        .move_object_to_zone(commander_obj_id, ZoneId::Graveyard(p1))
        .expect("move to graveyard failed");

    // SBA: commander in graveyard → emits CommanderZoneReturnChoiceRequired (MR-M9-01 fix).
    let sba_events = check_and_apply_sbas(&mut state);
    assert!(
        sba_events
            .iter()
            .any(|e| matches!(e, GameEvent::CommanderZoneReturnChoiceRequired { .. })),
        "SBA should emit CommanderZoneReturnChoiceRequired; events: {sba_events:?}"
    );

    // Owner resolves choice: return commander to command zone.
    let choice_event = sba_events.iter().find(
        |e| matches!(e, GameEvent::CommanderZoneReturnChoiceRequired { owner, .. } if *owner == p1),
    );
    let graveyard_obj_id = match choice_event.unwrap() {
        GameEvent::CommanderZoneReturnChoiceRequired { object_id, .. } => *object_id,
        _ => unreachable!(),
    };
    let (new_state, _) = process_command(
        state,
        Command::ReturnCommanderToCommandZone {
            player: p1,
            object_id: graveyard_obj_id,
        },
    )
    .expect("return commander to command zone");
    let mut state = new_state;

    // Commander is now in command zone.
    assert_eq!(
        state.objects_in_zone(&ZoneId::Command(p1)).len(),
        1,
        "commander should be in command zone after choice resolved"
    );

    // "Re-cast": add a NEW object with the same card_id to the battlefield
    // (simulating casting from command zone; the new object has a new ObjectId).
    state
        .add_object(
            mtg_engine::state::game_object::GameObject {
                id: mtg_engine::ObjectId(0), // will be assigned
                card_id: Some(cmd_id.clone()),
                characteristics: mtg_engine::state::game_object::Characteristics {
                    name: "Persistent Commander".to_string(),
                    card_types: [mtg_engine::CardType::Creature].into_iter().collect(),
                    power: Some(7),
                    toughness: Some(7),
                    ..Default::default()
                },
                controller: p1,
                owner: p1,
                zone: ZoneId::Battlefield,
                status: mtg_engine::ObjectStatus::default(),
                counters: im::OrdMap::new(),
                attachments: im::Vector::new(),
                attached_to: None,
                damage_marked: 0,
                deathtouch_damage: false,
                is_token: false,
                timestamp: 0,
                has_summoning_sickness: false,
                goaded_by: im::Vector::new(),
                kicker_times_paid: 0,
                cast_alt_cost: None,
                is_bestowed: false,
                is_foretold: false,
                foretold_turn: 0,
                was_unearthed: false,
                myriad_exile_at_eoc: false,
                decayed_sacrifice_at_eoc: false,
                is_suspended: false,
                exiled_by_hideaway: None,
                is_renowned: false,
                is_suspected: false,
                encore_sacrifice_at_end_step: false,
                encore_must_attack: None,
                encore_activated_by: None,
                is_plotted: false,
                plotted_turn: 0,
                is_prototyped: false,
                was_bargained: false,
                echo_pending: false,
                phased_out_indirectly: false,
                phased_out_controller: None,
                creatures_devoured: 0,
                champion_exiled_card: None,
                paired_with: None,
                tribute_was_paid: false,
                x_value: 0,
                evidence_collected: false,
                squad_count: 0,
                offspring_paid: false,
                gift_was_given: false,
                gift_opponent: None,
                is_saddled: false,
            },
            ZoneId::Battlefield,
        )
        .expect("add new commander object failed");

    // Find the new commander ObjectId.
    let new_commander_id = state
        .objects
        .values()
        .find(|o| o.card_id.as_ref() == Some(&cmd_id) && o.zone == ZoneId::Battlefield)
        .map(|o| o.id)
        .expect("new commander should be on battlefield");

    // New combat from the DeclareAttackers step.
    state.turn.step = mtg_engine::Step::DeclareAttackers;
    state.turn.priority_holder = Some(p1);
    state.turn.players_passed = im::OrdSet::new();

    // Round 2: deal 7 more commander damage to p2 with the NEW object.
    let (state, _) = run_one_unblocked_combat(state, p1, p2, new_commander_id, &[p1, p2]);

    // Verify cumulative damage is 14 (7 + 7), confirming zone-change survival.
    let damage_after_r2 = state
        .players
        .get(&p2)
        .unwrap()
        .commander_damage_received
        .get(&p1)
        .and_then(|m| m.get(&cmd_id))
        .copied()
        .unwrap_or(0);
    assert_eq!(
        damage_after_r2, 14,
        "p2 should have 14 cumulative commander damage across zone change (7+7)"
    );
}

#[test]
/// CR 510.2 — Combat damage is dealt even without an explicit DeclareBlockers
/// command. When all players pass through the DeclareBlockers step, the engine
/// advances to CombatDamage and deals damage normally.
fn test_combat_damage_no_declare_blockers_command() {
    let p1 = p(1);
    let p2 = p(2);
    let players = [p(1), p(2), p(3), p(4)];

    let state = GameStateBuilder::four_player()
        .at_step(Step::BeginningOfCombat)
        .active_player(p1)
        .object(ObjectSpec::creature(p1, "Elf", 1, 1))
        .build()
        .unwrap();

    // Pass all through BeginningOfCombat → DeclareAttackers
    let (state, _) = pass_all(state, &players);
    assert_eq!(state.turn.step, Step::DeclareAttackers);

    // Declare the elf attacking p2
    let elf_id = state
        .objects_in_zone(&ZoneId::Battlefield)
        .first()
        .unwrap()
        .id;
    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(elf_id, AttackTarget::Player(p2))],
            enlist_choices: vec![],
        },
    )
    .unwrap();

    // Pass all → DeclareBlockers
    let (state, _) = pass_all(state, &players);
    assert_eq!(state.turn.step, Step::DeclareBlockers);

    // All pass WITHOUT sending DeclareBlockers command → CombatDamage
    let (state, _) = pass_all(state, &players);

    // P2 should have taken 1 combat damage
    assert_eq!(
        state.players.get(&p2).unwrap().life_total,
        39,
        "P2 should have taken 1 combat damage (40 - 1 = 39)"
    );
}
