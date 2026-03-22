//! Dungeon resolution tests — Session 3 (CR 309.4c, CR 704.5t, CR 725.2, CR 725.4).
//!
//! Tests cover:
//! - Room ability resolution: room effects execute through the standard effect path
//! - SBA 704.5t: dungeon removed when on bottommost room + no room ability on stack
//! - SBA 704.5t: dungeon NOT removed while room ability still on stack
//! - Initiative: upkeep venture into Undercity for initiative holder (CR 725.2)
//! - Initiative: combat damage steal transfers initiative (CR 725.2)

use mtg_engine::state::stack::StackObjectKind;
use mtg_engine::{
    check_and_apply_sbas, process_command, AttackTarget, Command, DungeonId, DungeonState,
    GameEvent, GameStateBuilder, ObjectSpec, PlayerId, Step,
};

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

/// Build a minimal 4-player game state for testing.
fn simple_state() -> mtg_engine::GameState {
    GameStateBuilder::four_player()
        .build()
        .expect("build failed")
}

// ── Room Ability Resolution ───────────────────────────────────────────────────

/// CR 309.4c: When the venture marker enters a room, that room's triggered ability goes on
/// the stack and resolves like any other triggered ability. When it resolves, the
/// room effect executes via the standard effect execution path.
///
/// Lost Mine of Phandelver, Room 0 (Cave Entrance): effect is Scry 1.
/// Source: CR 309.4c — "the room ability triggers and goes on the stack as any other
/// triggered ability."
#[test]
fn test_room_ability_resolves_scry() {
    let mut state = simple_state();

    // Venture into the dungeon — pushes Cave Entrance (room 0, Scry 1) onto stack.
    let venture_events = mtg_engine::handle_venture_into_dungeon(&mut state, p(1), false)
        .expect("venture should succeed");

    // VenturedIntoDungeon event should be emitted.
    let ventured = venture_events.iter().any(|e| {
        matches!(
            e,
            GameEvent::VenturedIntoDungeon {
                player,
                dungeon: DungeonId::LostMineOfPhandelver,
                room: 0
            } if *player == p(1)
        )
    });
    assert!(ventured, "VenturedIntoDungeon event should be emitted");

    // A RoomAbility SOK for Cave Entrance (room 0) should be on the stack.
    let room_ability_on_stack = state.stack_objects.iter().any(|so| {
        matches!(
            &so.kind,
            StackObjectKind::RoomAbility { owner, dungeon, room }
            if *owner == p(1) && *dungeon == DungeonId::LostMineOfPhandelver && *room == 0
        )
    });
    assert!(
        room_ability_on_stack,
        "RoomAbility SOK should be on the stack for Cave Entrance"
    );

    // Both players pass priority so the room ability resolves.
    // Cave Entrance does Scry 1 — the player's library doesn't have a specific order
    // in tests, so we just verify the ability resolves without error.
    let (state, resolve_events) = process_command(state, Command::PassPriority { player: p(1) })
        .expect("pass priority by p1 should succeed");

    let (state, resolve_events2) = process_command(state, Command::PassPriority { player: p(2) })
        .expect("pass priority by p2 should succeed");

    let (state, resolve_events3) = process_command(state, Command::PassPriority { player: p(3) })
        .expect("pass priority by p3 should succeed");

    let (state, resolve_events4) = process_command(state, Command::PassPriority { player: p(4) })
        .expect("pass priority by p4 should succeed");

    let all_events: Vec<_> = resolve_events
        .into_iter()
        .chain(resolve_events2)
        .chain(resolve_events3)
        .chain(resolve_events4)
        .collect();

    // The stack should now be empty (room ability resolved and left the stack).
    assert!(
        state.stack_objects.is_empty(),
        "stack should be empty after room ability resolves"
    );

    // The venture marker should still be on room 0 (not advanced — Scry doesn't advance).
    let ds = state
        .dungeon_state
        .get(&p(1))
        .expect("player should still have dungeon state");
    assert_eq!(
        ds.current_room, 0,
        "venture marker should remain at room 0 after room ability resolves"
    );

    // No errors in resolution — the Scry completed successfully.
    let _ = all_events;
}

/// CR 309.4c: Lost Mine of Phandelver, Room 1 (Goblin Lair): effect is "Create a 1/1 red Goblin
/// creature token." Verify the token is created when the room ability resolves.
///
/// Source: CR 309.4c — room ability resolves, executing the Create Token effect.
#[test]
fn test_room_ability_resolves_create_token() {
    let mut state = simple_state();

    // First venture to get to room 0 (Cave Entrance), then advance to room 1 (Goblin Lair).
    // We can set the dungeon state directly and push a RoomAbility for room 1.
    state.dungeon_state.insert(
        p(1),
        DungeonState {
            dungeon: DungeonId::LostMineOfPhandelver,
            current_room: 0,
        },
    );

    // Venture again — advances from room 0 to room 1 (Goblin Lair, first exit from Cave Entrance).
    let venture_events = mtg_engine::handle_venture_into_dungeon(&mut state, p(1), false)
        .expect("second venture should succeed");

    // VenturedIntoDungeon event should be emitted for room 1.
    let ventured_to_room1 = venture_events.iter().any(|e| {
        matches!(
            e,
            GameEvent::VenturedIntoDungeon {
                player,
                dungeon: DungeonId::LostMineOfPhandelver,
                room: 1
            } if *player == p(1)
        )
    });
    assert!(
        ventured_to_room1,
        "VenturedIntoDungeon event for room 1 should be emitted"
    );

    // Room 1 (Goblin Lair) ability should be on the stack.
    let room_ability_on_stack = state.stack_objects.iter().any(|so| {
        matches!(
            &so.kind,
            StackObjectKind::RoomAbility { owner, dungeon, room }
            if *owner == p(1) && *dungeon == DungeonId::LostMineOfPhandelver && *room == 1
        )
    });
    assert!(
        room_ability_on_stack,
        "RoomAbility SOK should be on stack for Goblin Lair"
    );

    // Count tokens before resolution.
    let token_count_before = state
        .objects
        .values()
        .filter(|obj| obj.is_token && obj.controller == p(1))
        .count();

    // Pass all 4 players to resolve the room ability.
    let (state, _) =
        process_command(state, Command::PassPriority { player: p(1) }).expect("pass p1");
    let (state, _) =
        process_command(state, Command::PassPriority { player: p(2) }).expect("pass p2");
    let (state, _) =
        process_command(state, Command::PassPriority { player: p(3) }).expect("pass p3");
    let (state, _) =
        process_command(state, Command::PassPriority { player: p(4) }).expect("pass p4");

    // Count tokens after resolution.
    let token_count_after = state
        .objects
        .values()
        .filter(|obj| obj.is_token && obj.controller == p(1))
        .count();

    assert_eq!(
        token_count_after,
        token_count_before + 1,
        "Goblin Lair should create 1 token: before={}, after={}",
        token_count_before,
        token_count_after
    );

    // Verify the token is a 1/1 red Goblin.
    let goblin_token = state
        .objects
        .values()
        .find(|obj| obj.is_token && obj.controller == p(1));
    let Some(goblin) = goblin_token else {
        panic!("Expected a Goblin token to be on the battlefield");
    };
    assert_eq!(
        goblin.characteristics.power,
        Some(1),
        "Goblin token should have power 1"
    );
    assert_eq!(
        goblin.characteristics.toughness,
        Some(1),
        "Goblin token should have toughness 1"
    );
}

// ── SBA 704.5t ───────────────────────────────────────────────────────────────

/// CR 704.5t / CR 309.6: When a player's venture marker is on the bottommost room
/// and no room ability from that dungeon is on the stack, the dungeon is removed from
/// the game (command zone) and the player completes the dungeon.
///
/// Source: CR 704.5t — "If a player has a dungeon in the command zone and
/// that dungeon's bottommost room has been visited, and that player doesn't have
/// a room ability of that dungeon on the stack, that dungeon is removed from the game."
#[test]
fn test_sba_704_5t_removes_completed_dungeon() {
    let mut state = simple_state();

    // Place player 1 at the bottommost room of Lost Mine of Phandelver (room 6).
    let dungeon_def = mtg_engine::get_dungeon(DungeonId::LostMineOfPhandelver);
    let bottommost = dungeon_def.bottommost_room;
    state.dungeon_state.insert(
        p(1),
        DungeonState {
            dungeon: DungeonId::LostMineOfPhandelver,
            current_room: bottommost,
        },
    );

    // Stack is empty — no room ability on it.
    assert!(state.stack_objects.is_empty(), "stack should be empty");

    // Run SBAs.
    let sba_events = check_and_apply_sbas(&mut state);

    // DungeonCompleted event should be emitted.
    let completed = sba_events.iter().any(|e| {
        matches!(
            e,
            GameEvent::DungeonCompleted {
                player,
                dungeon: DungeonId::LostMineOfPhandelver
            } if *player == p(1)
        )
    });
    assert!(completed, "SBA 704.5t should emit DungeonCompleted event");

    // Dungeon should be removed from player 1's state.
    assert!(
        state.dungeon_state.get(&p(1)).is_none(),
        "dungeon should be removed from command zone by SBA 704.5t"
    );

    // dungeons_completed should be incremented.
    let completed_count = state.players.get(&p(1)).unwrap().dungeons_completed;
    assert_eq!(
        completed_count, 1,
        "dungeons_completed should be 1 after SBA removes the dungeon"
    );

    // DungeonId should be in dungeons_completed_set.
    let in_set = state
        .players
        .get(&p(1))
        .unwrap()
        .dungeons_completed_set
        .contains(&DungeonId::LostMineOfPhandelver);
    assert!(
        in_set,
        "DungeonId should be in dungeons_completed_set after SBA removes the dungeon"
    );
}

/// CR 309.6: If a room ability from the dungeon is still on the stack, the dungeon
/// should NOT be removed by SBA 704.5t — wait until the ability leaves the stack.
///
/// Source: CR 309.6 — "...if a room of that dungeon has been visited, and no abilities
/// of that dungeon are on the stack."
#[test]
fn test_sba_704_5t_waits_for_room_ability() {
    use mtg_engine::state::stack::{StackObject, StackObjectKind};

    let mut state = simple_state();

    // Place player 1 at the bottommost room of Lost Mine (room 6).
    let dungeon_def = mtg_engine::get_dungeon(DungeonId::LostMineOfPhandelver);
    let bottommost = dungeon_def.bottommost_room;
    state.dungeon_state.insert(
        p(1),
        DungeonState {
            dungeon: DungeonId::LostMineOfPhandelver,
            current_room: bottommost,
        },
    );

    // Push a fake RoomAbility for this dungeon onto the stack.
    let room_so = StackObject {
        id: state.next_object_id(),
        controller: p(1),
        kind: StackObjectKind::RoomAbility {
            owner: p(1),
            dungeon: DungeonId::LostMineOfPhandelver,
            room: bottommost,
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
        was_cleaved: false,
        // CR 715.3d: test objects are not adventure casts.
        was_cast_as_adventure: false,
        spliced_effects: vec![],
        spliced_card_ids: vec![],
        modes_chosen: vec![],
        x_value: 0,
        evidence_collected: false,
        is_cast_transformed: false,
        additional_costs: vec![],
    };
    state.stack_objects.push_back(room_so);

    // Run SBAs — dungeon should NOT be removed because room ability is on the stack.
    let sba_events = check_and_apply_sbas(&mut state);

    // No DungeonCompleted event should be emitted.
    let completed = sba_events.iter().any(|e| {
        matches!(
            e,
            GameEvent::DungeonCompleted { player, .. } if *player == p(1)
        )
    });
    assert!(
        !completed,
        "SBA 704.5t should NOT remove dungeon while room ability is on the stack"
    );

    // Dungeon should still be present.
    assert!(
        state.dungeon_state.get(&p(1)).is_some(),
        "dungeon should remain in command zone while room ability is on the stack"
    );
}

// ── Initiative Upkeep Trigger ─────────────────────────────────────────────────

/// CR 725.2: "At the beginning of your upkeep, if you have the initiative,
/// venture into the Undercity." The initiative holder ventures at the start of
/// each of their upkeep steps.
///
/// Source: CR 725.2 — "Whenever a player takes the initiative, that player ventures
/// into the dungeon. At the beginning of that player's upkeep, they venture into
/// the dungeon."
#[test]
fn test_initiative_upkeep_venture() {
    // Build a state where p1 has the initiative and is at the start of their upkeep.
    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .at_step(Step::Upkeep)
        .active_player(p(1))
        .build()
        .expect("build failed");

    // Give p1 the initiative.
    state.has_initiative = Some(p(1));

    // At the start of upkeep, p1 should venture into the Undercity.
    // The upkeep actions are triggered when entering the Upkeep step.
    // We use execute_turn_based_actions directly to avoid re-entering the step.
    use mtg_engine::rules::turn_actions::execute_turn_based_actions;
    let events = execute_turn_based_actions(&mut state).expect("upkeep actions should succeed");

    // VenturedIntoDungeon event should be emitted for The Undercity (room 0).
    let ventured_undercity = events.iter().any(|e| {
        matches!(
            e,
            GameEvent::VenturedIntoDungeon {
                player,
                dungeon: DungeonId::TheUndercity,
                room: 0
            } if *player == p(1)
        )
    });
    assert!(
        ventured_undercity,
        "Initiative holder should venture into The Undercity at upkeep start"
    );

    // Player 1 should now have a dungeon state in The Undercity.
    let ds = state
        .dungeon_state
        .get(&p(1))
        .expect("p1 should have dungeon state after upkeep venture");
    assert_eq!(
        ds.dungeon,
        DungeonId::TheUndercity,
        "initiative upkeep venture should force The Undercity"
    );
}

// ── Initiative Combat Damage Steal ───────────────────────────────────────────

/// CR 725.2: "Whenever a creature deals combat damage to the player who has the
/// initiative, that creature's controller takes the initiative."
///
/// Source: CR 725.2 — "Whenever a creature an opponent controls deals combat damage
/// to the player with the initiative, that creature's controller takes the initiative."
#[test]
fn test_initiative_combat_damage_steal() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(ObjectSpec::creature(p1, "Attacker", 2, 2))
        .at_step(Step::DeclareAttackers)
        .active_player(p1)
        .build()
        .expect("build failed");

    // Give p2 the initiative — p1's creature will deal damage to p2 (initiative holder).
    state.has_initiative = Some(p2);

    // Find the attacker.
    let attacker_id = state
        .objects
        .values()
        .find(|o| o.controller == p1)
        .unwrap()
        .id;

    // Declare attacker targeting p2.
    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(attacker_id, AttackTarget::Player(p2))],
            enlist_choices: vec![],
        },
    )
    .expect("declare attackers failed");

    // Both players pass through DeclareAttackers → DeclareBlockers.
    let (state, _) = process_command(state, Command::PassPriority { player: p1 }).expect("p1 pass");
    let (state, _) = process_command(state, Command::PassPriority { player: p2 }).expect("p2 pass");

    // Declare no blockers.
    let (state, _) = process_command(
        state,
        Command::DeclareBlockers {
            player: p2,
            blockers: vec![],
        },
    )
    .expect("declare blockers failed");

    // Both pass → CombatDamage step executes.
    let (state, events1) =
        process_command(state, Command::PassPriority { player: p1 }).expect("p1 pass");
    let (state, events2) =
        process_command(state, Command::PassPriority { player: p2 }).expect("p2 pass");

    let all_events: Vec<_> = events1.into_iter().chain(events2).collect();

    // Verify combat damage was dealt to p2.
    let damage_to_p2 = all_events.iter().any(|e| {
        matches!(
            e,
            GameEvent::CombatDamageDealt { assignments }
            if assignments.iter().any(|a| {
                matches!(a.target, mtg_engine::CombatDamageTarget::Player(pid) if pid == p2)
                    && a.amount > 0
            })
        )
    });
    assert!(
        damage_to_p2,
        "p1's creature should deal combat damage to p2"
    );

    // Initiative should have transferred to p1 (p2 was the holder, p1's creature attacked).
    assert_eq!(
        state.has_initiative,
        Some(p1),
        "initiative should transfer to p1 after their creature deals combat damage to p2 (the initiative holder)"
    );

    // InitiativeTaken event should be emitted.
    let initiative_taken = all_events.iter().any(|e| {
        matches!(
            e,
            GameEvent::InitiativeTaken { player } if *player == p1
        )
    });
    assert!(
        initiative_taken,
        "InitiativeTaken event should be emitted when p1 takes initiative"
    );

    // p1 should also have ventured into the Undercity (CR 725.2: taking initiative ventures).
    let ventured_undercity = all_events.iter().any(|e| {
        matches!(
            e,
            GameEvent::VenturedIntoDungeon {
                player,
                dungeon: DungeonId::TheUndercity,
                room: 0
            } if *player == p1
        )
    });
    assert!(
        ventured_undercity,
        "Taking initiative should trigger venture into The Undercity"
    );
}
