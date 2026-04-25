//! 6-player game tests: priority rotation, combat, APNAP ordering, elimination, concession.
//!
//! Session 6: `reveals_hidden_info()` & 6-Player Tests.
//!
//! These tests validate that the engine's multiplayer systems (already designed
//! for N players) work correctly at N=6, not just N=4.
//!
//! Also includes `reveals_hidden_info()` classification tests for the network
//! layer (M10 safe-rewind checkpoint identification).

use mtg_engine::{
    process_command, register_commander_zone_replacements, AttackTarget, CardId, Command,
    GameEvent, GameState, GameStateBuilder, ObjectSpec, PlayerId, Step, ZoneId,
};

// ── Helpers ───────────────────────────────────────────────────────────────────

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

fn cid(s: &str) -> CardId {
    CardId(s.to_string())
}

/// Pass priority until the step or turn advances (handles variable player counts).
fn pass_until_advance(mut state: GameState) -> (GameState, Vec<GameEvent>) {
    let mut all_events = Vec::new();
    while let Some(holder) = state.turn.priority_holder {
        let (new_state, events) = process_command(state, Command::PassPriority { player: holder })
            .unwrap_or_else(|e| panic!("PassPriority by {:?} failed: {:?}", holder, e));
        let advanced = events.iter().any(|e| {
            matches!(
                e,
                GameEvent::StepChanged { .. } | GameEvent::TurnStarted { .. }
            )
        });
        all_events.extend(events);
        state = new_state;
        if advanced {
            return (state, all_events);
        }
    }
    (state, all_events)
}

/// Build a 6-player state at the given step, each player having library cards.
fn six_player_at(step: Step) -> GameState {
    let mut builder = GameStateBuilder::six_player().at_step(step);
    for pid in 1..=6u64 {
        let player = PlayerId(pid);
        for i in 0..10 {
            builder = builder.object(
                ObjectSpec::card(player, &format!("Card {} P{}", i, pid))
                    .in_zone(ZoneId::Library(player)),
            );
        }
    }
    builder.build().unwrap()
}

// ── Item 3: 6-player tests ────────────────────────────────────────────────────

#[test]
/// CR 116.3 — priority passes through all 6 players in APNAP order before
/// the stack resolves or the step advances.
///
/// Player 1 is active. Each of the 6 players must pass in order 1→2→3→4→5→6
/// before the step can advance.
fn test_six_player_priority_rotation() {
    let state = six_player_at(Step::PreCombatMain);

    // Player 1 (active) has priority first.
    assert_eq!(state.turn.priority_holder, Some(p(1)));

    // P1 passes → P2
    let (state, ev) = process_command(state, Command::PassPriority { player: p(1) }).unwrap();
    assert!(ev
        .iter()
        .any(|e| matches!(e, GameEvent::PriorityPassed { player } if *player == p(1))));
    assert_eq!(state.turn.priority_holder, Some(p(2)));

    // P2 → P3
    let (state, _) = process_command(state, Command::PassPriority { player: p(2) }).unwrap();
    assert_eq!(state.turn.priority_holder, Some(p(3)));

    // P3 → P4
    let (state, _) = process_command(state, Command::PassPriority { player: p(3) }).unwrap();
    assert_eq!(state.turn.priority_holder, Some(p(4)));

    // P4 → P5
    let (state, _) = process_command(state, Command::PassPriority { player: p(4) }).unwrap();
    assert_eq!(state.turn.priority_holder, Some(p(5)));

    // P5 → P6
    let (state, _) = process_command(state, Command::PassPriority { player: p(5) }).unwrap();
    assert_eq!(state.turn.priority_holder, Some(p(6)));

    // P6 passes → all 6 have passed → step advances (AllPlayersPassed emitted)
    let (state, ev) = process_command(state, Command::PassPriority { player: p(6) }).unwrap();
    assert!(
        ev.iter().any(|e| matches!(e, GameEvent::AllPlayersPassed)),
        "AllPlayersPassed should be emitted after all 6 players pass"
    );
    // Step should have advanced beyond PreCombatMain
    assert_ne!(
        state.turn.step,
        Step::PreCombatMain,
        "step should advance after all 6 players pass"
    );
}

#[test]
/// CR 508.1 / CR 509.1 — in multiplayer, each defending player declares
/// blockers independently. With 6 players, the active player can attack
/// multiple defending players, and each declares blockers separately.
fn test_six_player_combat_five_defenders() {
    let mut builder = GameStateBuilder::six_player().at_step(Step::DeclareAttackers);
    // Give P1 five 2/2 attackers — one aimed at each defender.
    for i in 0..5 {
        builder = builder.object(ObjectSpec::creature(p(1), &format!("Attacker {}", i), 2, 2));
    }
    // Give each defending player one blocker.
    for pid in 2..=6 {
        builder = builder.object(ObjectSpec::creature(
            p(pid),
            &format!("Blocker P{}", pid),
            2,
            2,
        ));
    }
    let state = builder.build().unwrap();

    // Collect attacker IDs owned by P1 (the 5 attackers).
    let mut attacker_ids: Vec<mtg_engine::ObjectId> = state
        .objects
        .values()
        .filter(|o| o.controller == p(1))
        .map(|o| o.id)
        .collect();
    attacker_ids.sort();

    // Declare each attacker targeting a different player (P2–P6).
    let attackers_decl: Vec<(mtg_engine::ObjectId, AttackTarget)> = attacker_ids
        .iter()
        .zip(2u64..=6)
        .map(|(&id, target_pid)| (id, AttackTarget::Player(p(target_pid))))
        .collect();

    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p(1),
            attackers: attackers_decl,
            enlist_choices: vec![],
        },
    )
    .expect("DeclareAttackers failed");

    // All players pass to advance to DeclareBlockers.
    let (state, _) = pass_until_advance(state);
    assert_eq!(state.turn.step, Step::DeclareBlockers);

    // Each of P2–P6 declares their blocker for the attacker targeting them.
    // The attacker-to-defender mapping: attacker_ids[i] attacks p(i+2).
    let mut state = state;
    for (i, &attacker_id) in attacker_ids.iter().enumerate() {
        let def_pid = p(i as u64 + 2);
        // Find this player's blocker (the creature they own).
        let blocker_id = state
            .objects
            .values()
            .find(|o| o.controller == def_pid && o.zone == ZoneId::Battlefield)
            .map(|o| o.id)
            .expect("blocker should exist");

        let (new_state, _) = process_command(
            state,
            Command::DeclareBlockers {
                player: def_pid,
                blockers: vec![(blocker_id, attacker_id)],
            },
        )
        .expect("DeclareBlockers failed");
        state = new_state;
    }

    // Pass to CombatDamage step — all attackers are blocked by 2/2s.
    let (state, events) = pass_until_advance(state);

    // CombatDamageDealt should be emitted during the step transitions.
    // Each 2/2 attacker is blocked by a 2/2: both die. No damage to players.
    let damage_dealt = events
        .iter()
        .any(|e| matches!(e, GameEvent::CombatDamageDealt { .. }));
    assert!(
        damage_dealt,
        "CombatDamageDealt should have been emitted; events = {:?}",
        events
            .iter()
            .filter(|e| !matches!(e, GameEvent::PriorityGiven { .. }))
            .collect::<Vec<_>>()
    );

    // No defending player should have lost life (all damage was absorbed by blockers).
    for pid in 2..=6u64 {
        let life = state.players.get(&p(pid)).unwrap().life_total;
        assert_eq!(
            life, 40,
            "P{} should not have taken damage (was blocked)",
            pid
        );
    }
}

#[test]
/// CR 101.4 — APNAP (Active Player, Non-Active Player) ordering cycles through
/// all 6 players in turn order. After P1's turn, P2 becomes active; then P3, etc.
fn test_six_player_apnap_ordering() {
    let mut builder = GameStateBuilder::six_player().at_step(Step::End);
    // Library cards so nobody decks out during draw steps.
    for pid in 1..=6u64 {
        let player = PlayerId(pid);
        for i in 0..20 {
            builder = builder.object(
                ObjectSpec::card(player, &format!("Card {} P{}", i, pid))
                    .in_zone(ZoneId::Library(player)),
            );
        }
    }
    let state = builder.build().unwrap();

    // Verify turn order is 1→2→3→4→5→6.
    let turn_order: Vec<PlayerId> = state.turn.turn_order.iter().copied().collect();
    assert_eq!(
        turn_order,
        vec![p(1), p(2), p(3), p(4), p(5), p(6)],
        "turn order should be P1→P2→P3→P4→P5→P6"
    );

    // Advance through P1's End step → P2 becomes active.
    let (state, events) = pass_until_advance(state);
    let turn_started = events
        .iter()
        .any(|e| matches!(e, GameEvent::TurnStarted { player, .. } if *player == p(2)));
    assert!(
        turn_started,
        "P2's turn should have started after P1's End step"
    );
    assert_eq!(
        state.turn.active_player,
        p(2),
        "P2 should be active after P1's turn"
    );

    // Advance through P2's End step → P3.
    let (state, events) = pass_until_advance(state);
    // P2 is in Upkeep now; advance to End and beyond.
    // Keep passing until P3's turn starts.
    let mut state = state;
    let mut found_p3 = events
        .iter()
        .any(|e| matches!(e, GameEvent::TurnStarted { player, .. } if *player == p(3)));
    while !found_p3 {
        let (new_state, new_events) = pass_until_advance(state);
        found_p3 = new_events
            .iter()
            .any(|e| matches!(e, GameEvent::TurnStarted { player, .. } if *player == p(3)));
        state = new_state;
        if found_p3 {
            break;
        }
        // Safety: if we've gone past P3 starting, check current active player.
        if state.turn.active_player == p(3) {
            found_p3 = true;
            break;
        }
    }
    assert!(
        found_p3 || state.turn.active_player == p(3),
        "P3 should become active after P2's turn"
    );
}

#[test]
/// CR 800.4 — when a player loses, their turn is removed from the turn order.
/// Player 3 is eliminated; turns should sequence P1→P2→P4→P5→P6, skipping P3.
fn test_six_player_turn_advancement_skips_eliminated() {
    let state = six_player_at(Step::PreCombatMain);

    // P3 concedes (simulating elimination).
    let (state, _) = process_command(state, Command::Concede { player: p(3) }).unwrap();

    // Verify P3 has conceded.
    let p3_state = state.players.get(&p(3)).unwrap();
    assert!(
        p3_state.has_conceded || p3_state.has_lost,
        "P3 should be eliminated after conceding"
    );

    // P1 passes → should skip P3 → P4 (or directly check that P3 is not priority holder).
    let (state, _) = process_command(state, Command::PassPriority { player: p(1) }).unwrap();
    assert_eq!(
        state.turn.priority_holder,
        Some(p(2)),
        "after P1 passes, P2 should hold priority (P3 is eliminated)"
    );

    let (state, _) = process_command(state, Command::PassPriority { player: p(2) }).unwrap();
    // P3 is skipped
    assert_eq!(
        state.turn.priority_holder,
        Some(p(4)),
        "after P2 passes, P4 should hold priority (P3 is skipped)"
    );

    let (state, _) = process_command(state, Command::PassPriority { player: p(4) }).unwrap();
    assert_eq!(
        state.turn.priority_holder,
        Some(p(5)),
        "after P4 passes, P5 should hold priority"
    );

    let (state, _) = process_command(state, Command::PassPriority { player: p(5) }).unwrap();
    assert_eq!(
        state.turn.priority_holder,
        Some(p(6)),
        "after P5 passes, P6 should hold priority"
    );
}

#[test]
/// CR 800.4 — player concedes during another player's turn; priority and turn
/// order adjust correctly. The conceding player is removed from priority
/// rotation immediately.
fn test_six_player_concession_mid_game() {
    let state = six_player_at(Step::PreCombatMain);

    // P1 is active with priority. P4 concedes (not active, not the priority holder).
    let (state, concede_events) =
        process_command(state, Command::Concede { player: p(4) }).unwrap();

    // Concession event should be emitted.
    assert!(
        concede_events
            .iter()
            .any(|e| matches!(e, GameEvent::PlayerConceded { player } if *player == p(4))),
        "PlayerConceded should be emitted for P4"
    );

    // P1 still holds priority (concession during P1's turn doesn't change active player).
    assert_eq!(
        state.turn.priority_holder,
        Some(p(1)),
        "P1 should still hold priority after P4 concedes"
    );

    // P1 passes → P2 (P3 is next, then P4 would be skipped, then P5, P6).
    let (state, _) = process_command(state, Command::PassPriority { player: p(1) }).unwrap();
    assert_eq!(
        state.turn.priority_holder,
        Some(p(2)),
        "P2 should hold priority after P1 passes"
    );

    let (state, _) = process_command(state, Command::PassPriority { player: p(2) }).unwrap();
    assert_eq!(
        state.turn.priority_holder,
        Some(p(3)),
        "P3 should hold priority after P2 passes"
    );

    // P3 passes → P4 is conceded, should skip to P5.
    let (state, _) = process_command(state, Command::PassPriority { player: p(3) }).unwrap();
    assert_eq!(
        state.turn.priority_holder,
        Some(p(5)),
        "P5 should hold priority after P3 passes (P4 is conceded/eliminated)"
    );
}

#[test]
/// CR 903.6 / CR 903.7 — in a 6-player Commander game, each player starts with
/// their commander in the command zone and 40 life. Commander zone return
/// replacements are registered for each player.
fn test_six_player_game_start_all_commanders_correct() {
    let commander_ids: Vec<CardId> = (1..=6).map(|i| cid(&format!("commander-{}", i))).collect();

    let mut builder = GameStateBuilder::six_player();
    for (i, cid_val) in commander_ids.iter().enumerate() {
        let pid = p(i as u64 + 1);
        builder = builder.player_commander(pid, cid_val.clone());
        builder = builder.object(
            ObjectSpec::card(pid, &format!("Commander {}", i + 1))
                .with_card_id(cid_val.clone())
                .with_types(vec![mtg_engine::CardType::Creature])
                .with_supertypes(vec![mtg_engine::SuperType::Legendary])
                .in_zone(ZoneId::Command(pid)),
        );
    }
    let mut state = builder.build().unwrap();

    // Register commander zone replacements (CR 903.9b: hand/library → command zone).
    register_commander_zone_replacements(&mut state);

    // Verify: each player starts at 40 life.
    for pid in 1..=6u64 {
        let life = state.players.get(&p(pid)).unwrap().life_total;
        assert_eq!(life, 40, "P{} should start at 40 life (CR 903.7)", pid);
    }

    // Verify: each player's commander is in their command zone.
    for (i, cid_val) in commander_ids.iter().enumerate() {
        let pid = p(i as u64 + 1);
        let in_command = state
            .objects_in_zone(&ZoneId::Command(pid))
            .iter()
            .any(|obj| obj.card_id.as_ref().map(|c| c == cid_val).unwrap_or(false));
        assert!(
            in_command,
            "P{}'s commander should be in command zone (CR 903.6)",
            i + 1
        );
    }

    // Verify: commander_ids are registered for each player.
    for (i, cid_val) in commander_ids.iter().enumerate() {
        let pid = p(i as u64 + 1);
        let player_state = state.players.get(&pid).unwrap();
        assert!(
            player_state.commander_ids.contains(cid_val),
            "P{} should have their commander registered in commander_ids",
            i + 1
        );
    }

    // Verify: replacement effects are registered (2 per commander: hand + library redirect).
    // 6 players × 2 effects = 12 replacement effects.
    assert_eq!(
        state.replacement_effects.len(),
        12,
        "should have 12 commander zone replacement effects (2 per player × 6 players)"
    );
}

// ── Item 4: reveals_hidden_info tests ────────────────────────────────────────

#[test]
/// `GameEvent::reveals_hidden_info()` returns `true` for `CardDrawn`.
///
/// Drawing a card moves a card from the hidden library zone to the player's
/// hand, revealing which card was drawn. This is hidden information that the
/// M10 network layer must not allow rewinding past.
fn test_reveals_hidden_info_card_drawn_true() {
    let event = GameEvent::CardDrawn {
        player: p(1),
        new_object_id: mtg_engine::ObjectId(42),
    };
    assert!(
        event.reveals_hidden_info(),
        "CardDrawn should reveal hidden info"
    );
}

#[test]
/// `GameEvent::reveals_hidden_info()` returns `false` for `PriorityGiven`.
///
/// Granting priority involves only public information (who has priority).
/// The network layer can safely rewind to before a PriorityGiven event.
fn test_reveals_hidden_info_priority_given_false() {
    let event = GameEvent::PriorityGiven { player: p(1) };
    assert!(
        !event.reveals_hidden_info(),
        "PriorityGiven should NOT reveal hidden info"
    );
}
