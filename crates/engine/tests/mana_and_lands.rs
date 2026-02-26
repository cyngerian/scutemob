//! Tests for M3-A: stack foundation, mana ability activation, and land play.
//!
//! CR 305.1 — playing a land
//! CR 605   — mana abilities

use mtg_engine::{
    Command, GameEvent, GameStateBuilder, GameStateError, ManaAbility, ManaColor, ManaPool,
    ObjectSpec, PlayerId, Step, ZoneId,
};

// ══════════════════════════════════════════════════════════════════════════════
// PlayLand tests
// ══════════════════════════════════════════════════════════════════════════════

#[test]
/// CR 305.1 — playing a land during your main phase puts it on the battlefield.
fn test_play_land_enters_battlefield() {
    let state = GameStateBuilder::new()
        .add_player(PlayerId(1))
        .add_player(PlayerId(2))
        .at_step(Step::PreCombatMain)
        .active_player(PlayerId(1))
        .object(ObjectSpec::land(PlayerId(1), "Forest").in_zone(ZoneId::Hand(PlayerId(1))))
        .build()
        .unwrap();

    let hand_count = state.objects_in_zone(&ZoneId::Hand(PlayerId(1))).len();
    let bf_count = state.objects_in_zone(&ZoneId::Battlefield).len();

    // Find the land's ObjectId
    let land_id = state.objects_in_zone(&ZoneId::Hand(PlayerId(1)))[0].id;

    let (new_state, events) = mtg_engine::process_command(
        state,
        Command::PlayLand {
            player: PlayerId(1),
            card: land_id,
        },
    )
    .unwrap();

    // Hand shrank by 1, battlefield grew by 1
    assert_eq!(
        new_state.objects_in_zone(&ZoneId::Hand(PlayerId(1))).len(),
        hand_count - 1
    );
    assert_eq!(
        new_state.objects_in_zone(&ZoneId::Battlefield).len(),
        bf_count + 1
    );

    // LandPlayed event emitted
    assert!(events.iter().any(|e| matches!(
        e,
        GameEvent::LandPlayed {
            player: PlayerId(1),
            ..
        }
    )));
}

#[test]
/// CR 305.1 — land plays remaining is decremented after playing a land.
fn test_play_land_decrements_land_plays() {
    let state = GameStateBuilder::new()
        .add_player(PlayerId(1))
        .add_player(PlayerId(2))
        .at_step(Step::PreCombatMain)
        .active_player(PlayerId(1))
        .object(ObjectSpec::land(PlayerId(1), "Forest").in_zone(ZoneId::Hand(PlayerId(1))))
        .build()
        .unwrap();

    assert_eq!(state.player(PlayerId(1)).unwrap().land_plays_remaining, 1);

    let land_id = state.objects_in_zone(&ZoneId::Hand(PlayerId(1)))[0].id;

    let (new_state, _) = mtg_engine::process_command(
        state,
        Command::PlayLand {
            player: PlayerId(1),
            card: land_id,
        },
    )
    .unwrap();

    assert_eq!(
        new_state.player(PlayerId(1)).unwrap().land_plays_remaining,
        0
    );
}

#[test]
/// CR 305.1 — playing a land resets players_passed (game action occurred).
/// Active player retains priority.
fn test_play_land_resets_priority_round() {
    let state = GameStateBuilder::new()
        .add_player(PlayerId(1))
        .add_player(PlayerId(2))
        .at_step(Step::PreCombatMain)
        .active_player(PlayerId(1))
        .object(ObjectSpec::land(PlayerId(1), "Forest").in_zone(ZoneId::Hand(PlayerId(1))))
        .build()
        .unwrap();

    let land_id = state.objects_in_zone(&ZoneId::Hand(PlayerId(1)))[0].id;

    let (new_state, _) = mtg_engine::process_command(
        state,
        Command::PlayLand {
            player: PlayerId(1),
            card: land_id,
        },
    )
    .unwrap();

    // Active player still has priority
    assert_eq!(new_state.turn.priority_holder, Some(PlayerId(1)));
    // Priority round reset
    assert!(new_state.turn.players_passed.is_empty());
}

#[test]
/// CR 305.1 — cannot play a land when not the active player.
fn test_play_land_wrong_player_fails() {
    let state = GameStateBuilder::new()
        .add_player(PlayerId(1))
        .add_player(PlayerId(2))
        .at_step(Step::PreCombatMain)
        .active_player(PlayerId(1))
        .object(ObjectSpec::land(PlayerId(2), "Forest").in_zone(ZoneId::Hand(PlayerId(2))))
        .build()
        .unwrap();

    // Give priority to player 2 so the NotPriorityHolder check doesn't fire first
    // (actually player 1 has priority here, so player 2 can't issue any command)
    let land_id = state.objects_in_zone(&ZoneId::Hand(PlayerId(2)))[0].id;

    let result = mtg_engine::process_command(
        state,
        Command::PlayLand {
            player: PlayerId(2),
            card: land_id,
        },
    );

    assert!(matches!(
        result,
        Err(GameStateError::NotPriorityHolder { .. })
    ));
}

#[test]
/// CR 305.1 — cannot play a land during a non-main-phase step.
fn test_play_land_non_main_phase_fails() {
    let state = GameStateBuilder::new()
        .add_player(PlayerId(1))
        .add_player(PlayerId(2))
        .at_step(Step::Upkeep)
        .active_player(PlayerId(1))
        .object(ObjectSpec::land(PlayerId(1), "Forest").in_zone(ZoneId::Hand(PlayerId(1))))
        .build()
        .unwrap();

    let land_id = state.objects_in_zone(&ZoneId::Hand(PlayerId(1)))[0].id;

    let result = mtg_engine::process_command(
        state,
        Command::PlayLand {
            player: PlayerId(1),
            card: land_id,
        },
    );

    assert!(matches!(result, Err(GameStateError::NotMainPhase)));
}

#[test]
/// CR 305.1 — cannot play a land when land_plays_remaining is 0.
fn test_play_land_no_plays_remaining_fails() {
    let state = GameStateBuilder::new()
        .add_player(PlayerId(1))
        .add_player(PlayerId(2))
        .at_step(Step::PreCombatMain)
        .active_player(PlayerId(1))
        .add_player_with(PlayerId(1), |p| p.land_plays(0))
        .object(ObjectSpec::land(PlayerId(1), "Forest").in_zone(ZoneId::Hand(PlayerId(1))))
        .build()
        .unwrap();

    let land_id = state.objects_in_zone(&ZoneId::Hand(PlayerId(1)))[0].id;

    let result = mtg_engine::process_command(
        state,
        Command::PlayLand {
            player: PlayerId(1),
            card: land_id,
        },
    );

    assert!(matches!(
        result,
        Err(GameStateError::NoLandPlaysRemaining(PlayerId(1)))
    ));
}

#[test]
/// CR 305.1 — cannot play a non-land card as a land.
fn test_play_land_non_land_fails() {
    let state = GameStateBuilder::new()
        .add_player(PlayerId(1))
        .add_player(PlayerId(2))
        .at_step(Step::PreCombatMain)
        .active_player(PlayerId(1))
        .object(ObjectSpec::card(PlayerId(1), "Lightning Bolt").in_zone(ZoneId::Hand(PlayerId(1))))
        .build()
        .unwrap();

    let card_id = state.objects_in_zone(&ZoneId::Hand(PlayerId(1)))[0].id;

    let result = mtg_engine::process_command(
        state,
        Command::PlayLand {
            player: PlayerId(1),
            card: card_id,
        },
    );

    assert!(matches!(result, Err(GameStateError::InvalidCommand(_))));
}

#[test]
/// CR 305.1 — cannot play a land when the stack is non-empty.
fn test_play_land_stack_nonempty_fails() {
    use mtg_engine::{StackObject, StackObjectKind};

    let state = GameStateBuilder::new()
        .add_player(PlayerId(1))
        .add_player(PlayerId(2))
        .at_step(Step::PreCombatMain)
        .active_player(PlayerId(1))
        .object(ObjectSpec::land(PlayerId(1), "Forest").in_zone(ZoneId::Hand(PlayerId(1))))
        .build()
        .unwrap();

    let land_id = state.objects_in_zone(&ZoneId::Hand(PlayerId(1)))[0].id;

    // Manually push a fake StackObject to simulate a non-empty stack.
    let mut state = state;
    let fake_id = state.next_object_id();
    state.stack_objects.push_back(StackObject {
        id: fake_id,
        controller: PlayerId(1),
        kind: StackObjectKind::Spell {
            source_object: fake_id,
        },
        targets: vec![],
        cant_be_countered: false,
        is_copy: false,
        cast_with_flashback: false,
        kicker_times_paid: 0,
    });

    let result = mtg_engine::process_command(
        state,
        Command::PlayLand {
            player: PlayerId(1),
            card: land_id,
        },
    );

    assert!(matches!(result, Err(GameStateError::StackNotEmpty)));
}

// ══════════════════════════════════════════════════════════════════════════════
// TapForMana tests
// ══════════════════════════════════════════════════════════════════════════════

#[test]
/// CR 605 — tapping a land with a mana ability adds mana to the pool.
fn test_tap_for_mana_adds_to_pool() {
    let forest_ability = ManaAbility::tap_for(ManaColor::Green);

    let state = GameStateBuilder::new()
        .add_player(PlayerId(1))
        .add_player(PlayerId(2))
        .at_step(Step::PreCombatMain)
        .active_player(PlayerId(1))
        .object(ObjectSpec::land(PlayerId(1), "Forest").with_mana_ability(forest_ability))
        .build()
        .unwrap();

    assert_eq!(state.player(PlayerId(1)).unwrap().mana_pool.green, 0);

    let land_id = state.objects_in_zone(&ZoneId::Battlefield)[0].id;

    let (new_state, events) = mtg_engine::process_command(
        state,
        Command::TapForMana {
            player: PlayerId(1),
            source: land_id,
            ability_index: 0,
        },
    )
    .unwrap();

    assert_eq!(new_state.player(PlayerId(1)).unwrap().mana_pool.green, 1);

    // ManaAdded event emitted
    assert!(events.iter().any(|e| matches!(
        e,
        GameEvent::ManaAdded {
            player: PlayerId(1),
            color: ManaColor::Green,
            amount: 1,
        }
    )));
}

#[test]
/// CR 701.21 — tapping a land for mana sets its tapped status.
fn test_tap_for_mana_taps_permanent() {
    let state = GameStateBuilder::new()
        .add_player(PlayerId(1))
        .add_player(PlayerId(2))
        .at_step(Step::PreCombatMain)
        .active_player(PlayerId(1))
        .object(
            ObjectSpec::land(PlayerId(1), "Island")
                .with_mana_ability(ManaAbility::tap_for(ManaColor::Blue)),
        )
        .build()
        .unwrap();

    let land_id = state.objects_in_zone(&ZoneId::Battlefield)[0].id;

    assert!(!state.object(land_id).unwrap().status.tapped);

    let (new_state, events) = mtg_engine::process_command(
        state,
        Command::TapForMana {
            player: PlayerId(1),
            source: land_id,
            ability_index: 0,
        },
    )
    .unwrap();

    assert!(new_state.object(land_id).unwrap().status.tapped);
    assert!(events.iter().any(|e| matches!(
        e,
        GameEvent::PermanentTapped {
            player: PlayerId(1),
            object_id: _,
        }
    )));
}

#[test]
/// CR 605 — tapping an already-tapped land for mana is illegal.
fn test_tap_already_tapped_land_fails() {
    let state = GameStateBuilder::new()
        .add_player(PlayerId(1))
        .add_player(PlayerId(2))
        .at_step(Step::PreCombatMain)
        .active_player(PlayerId(1))
        .object(
            ObjectSpec::land(PlayerId(1), "Forest")
                .with_mana_ability(ManaAbility::tap_for(ManaColor::Green))
                .tapped(),
        )
        .build()
        .unwrap();

    let land_id = state.objects_in_zone(&ZoneId::Battlefield)[0].id;

    let result = mtg_engine::process_command(
        state,
        Command::TapForMana {
            player: PlayerId(1),
            source: land_id,
            ability_index: 0,
        },
    );

    assert!(matches!(
        result,
        Err(GameStateError::PermanentAlreadyTapped(_))
    ));
}

#[test]
/// CR 605 — a player cannot tap an opponent's permanent for mana.
fn test_tap_for_mana_opponent_land_fails() {
    let state = GameStateBuilder::new()
        .add_player(PlayerId(1))
        .add_player(PlayerId(2))
        .at_step(Step::PreCombatMain)
        .active_player(PlayerId(1))
        .object(
            ObjectSpec::land(PlayerId(2), "Forest")
                .with_mana_ability(ManaAbility::tap_for(ManaColor::Green)),
        )
        .build()
        .unwrap();

    let land_id = state.objects_in_zone(&ZoneId::Battlefield)[0].id;

    // Player 2 owns the land; player 1 has priority and tries to tap it.
    let result = mtg_engine::process_command(
        state,
        Command::TapForMana {
            player: PlayerId(1),
            source: land_id,
            ability_index: 0,
        },
    );

    assert!(matches!(result, Err(GameStateError::NotController { .. })));
}

#[test]
/// CR 605.5 — mana ability activation does not change who holds priority.
fn test_tap_for_mana_player_retains_priority() {
    let state = GameStateBuilder::new()
        .add_player(PlayerId(1))
        .add_player(PlayerId(2))
        .at_step(Step::PreCombatMain)
        .active_player(PlayerId(1))
        .object(
            ObjectSpec::land(PlayerId(1), "Forest")
                .with_mana_ability(ManaAbility::tap_for(ManaColor::Green)),
        )
        .build()
        .unwrap();

    assert_eq!(state.turn.priority_holder, Some(PlayerId(1)));

    let land_id = state.objects_in_zone(&ZoneId::Battlefield)[0].id;

    let (new_state, _) = mtg_engine::process_command(
        state,
        Command::TapForMana {
            player: PlayerId(1),
            source: land_id,
            ability_index: 0,
        },
    )
    .unwrap();

    // Priority is still with player 1
    assert_eq!(new_state.turn.priority_holder, Some(PlayerId(1)));
}

#[test]
/// CR 605 — tapping multiple lands accumulates mana in the pool.
fn test_tap_multiple_lands_accumulates_mana() {
    let state = GameStateBuilder::new()
        .add_player(PlayerId(1))
        .add_player(PlayerId(2))
        .at_step(Step::PreCombatMain)
        .active_player(PlayerId(1))
        .object(
            ObjectSpec::land(PlayerId(1), "Forest 1")
                .with_mana_ability(ManaAbility::tap_for(ManaColor::Green)),
        )
        .object(
            ObjectSpec::land(PlayerId(1), "Forest 2")
                .with_mana_ability(ManaAbility::tap_for(ManaColor::Green)),
        )
        .object(
            ObjectSpec::land(PlayerId(1), "Island")
                .with_mana_ability(ManaAbility::tap_for(ManaColor::Blue)),
        )
        .build()
        .unwrap();

    let bf = state.objects_in_zone(&ZoneId::Battlefield);
    let id0 = bf[0].id;
    let id1 = bf[1].id;
    let id2 = bf[2].id;

    let (s1, _) = mtg_engine::process_command(
        state,
        Command::TapForMana {
            player: PlayerId(1),
            source: id0,
            ability_index: 0,
        },
    )
    .unwrap();
    let (s2, _) = mtg_engine::process_command(
        s1,
        Command::TapForMana {
            player: PlayerId(1),
            source: id1,
            ability_index: 0,
        },
    )
    .unwrap();
    let (s3, _) = mtg_engine::process_command(
        s2,
        Command::TapForMana {
            player: PlayerId(1),
            source: id2,
            ability_index: 0,
        },
    )
    .unwrap();

    let pool = &s3.player(PlayerId(1)).unwrap().mana_pool;
    assert_eq!(pool.green, 2);
    assert_eq!(pool.blue, 1);
    assert_eq!(pool.white, 0);
}

#[test]
/// CR 305.1 — only one land play per turn by default; second attempt fails.
fn test_play_land_only_one_per_turn() {
    let state = GameStateBuilder::new()
        .add_player(PlayerId(1))
        .add_player(PlayerId(2))
        .at_step(Step::PreCombatMain)
        .active_player(PlayerId(1))
        .object(ObjectSpec::land(PlayerId(1), "Forest 1").in_zone(ZoneId::Hand(PlayerId(1))))
        .object(ObjectSpec::land(PlayerId(1), "Forest 2").in_zone(ZoneId::Hand(PlayerId(1))))
        .build()
        .unwrap();

    let hand = state.objects_in_zone(&ZoneId::Hand(PlayerId(1)));
    let id0 = hand[0].id;
    let id1 = hand[1].id;

    // Play first land — succeeds
    let (s1, _) = mtg_engine::process_command(
        state,
        Command::PlayLand {
            player: PlayerId(1),
            card: id0,
        },
    )
    .unwrap();

    // Play second land — fails: no plays remaining
    let result = mtg_engine::process_command(
        s1,
        Command::PlayLand {
            player: PlayerId(1),
            card: id1,
        },
    );

    assert!(matches!(
        result,
        Err(GameStateError::NoLandPlaysRemaining(PlayerId(1)))
    ));
}

#[test]
/// CR 305.1 — a land can be played during postcombat main phase too.
fn test_play_land_postcombat_main_is_legal() {
    let state = GameStateBuilder::new()
        .add_player(PlayerId(1))
        .add_player(PlayerId(2))
        .at_step(Step::PostCombatMain)
        .active_player(PlayerId(1))
        .object(ObjectSpec::land(PlayerId(1), "Forest").in_zone(ZoneId::Hand(PlayerId(1))))
        .build()
        .unwrap();

    let land_id = state.objects_in_zone(&ZoneId::Hand(PlayerId(1)))[0].id;

    let result = mtg_engine::process_command(
        state,
        Command::PlayLand {
            player: PlayerId(1),
            card: land_id,
        },
    );

    assert!(result.is_ok());
}

#[test]
/// CR 605 — using an ability_index that doesn't exist on the source returns an error.
fn test_tap_for_mana_invalid_ability_index_fails() {
    let state = GameStateBuilder::new()
        .add_player(PlayerId(1))
        .add_player(PlayerId(2))
        .at_step(Step::PreCombatMain)
        .active_player(PlayerId(1))
        // Forest has ability at index 0 only
        .object(
            ObjectSpec::land(PlayerId(1), "Forest")
                .with_mana_ability(ManaAbility::tap_for(ManaColor::Green)),
        )
        .build()
        .unwrap();

    let land_id = state.objects_in_zone(&ZoneId::Battlefield)[0].id;

    let result = mtg_engine::process_command(
        state,
        Command::TapForMana {
            player: PlayerId(1),
            source: land_id,
            ability_index: 5, // out of range
        },
    );

    assert!(matches!(
        result,
        Err(GameStateError::InvalidAbilityIndex { .. })
    ));
}

#[test]
/// CR 605 — mana ability on a non-battlefield permanent is illegal.
fn test_tap_for_mana_non_battlefield_fails() {
    let state = GameStateBuilder::new()
        .add_player(PlayerId(1))
        .add_player(PlayerId(2))
        .at_step(Step::PreCombatMain)
        .active_player(PlayerId(1))
        // Forest is in hand, not battlefield
        .object(
            ObjectSpec::land(PlayerId(1), "Forest")
                .in_zone(ZoneId::Hand(PlayerId(1)))
                .with_mana_ability(ManaAbility::tap_for(ManaColor::Green)),
        )
        .build()
        .unwrap();

    let land_id = state.objects_in_zone(&ZoneId::Hand(PlayerId(1)))[0].id;

    let result = mtg_engine::process_command(
        state,
        Command::TapForMana {
            player: PlayerId(1),
            source: land_id,
            ability_index: 0,
        },
    );

    assert!(matches!(
        result,
        Err(GameStateError::ObjectNotOnBattlefield(_))
    ));
}

#[test]
/// CR 305.1 — mana pool is preserved after playing a land (pool is not cleared).
fn test_play_land_does_not_clear_mana_pool() {
    let state = GameStateBuilder::new()
        .add_player(PlayerId(1))
        .add_player(PlayerId(2))
        .at_step(Step::PreCombatMain)
        .active_player(PlayerId(1))
        .player_mana(
            PlayerId(1),
            ManaPool {
                green: 3,
                ..Default::default()
            },
        )
        .object(ObjectSpec::land(PlayerId(1), "Forest").in_zone(ZoneId::Hand(PlayerId(1))))
        .build()
        .unwrap();

    let land_id = state.objects_in_zone(&ZoneId::Hand(PlayerId(1)))[0].id;

    let (new_state, _) = mtg_engine::process_command(
        state,
        Command::PlayLand {
            player: PlayerId(1),
            card: land_id,
        },
    )
    .unwrap();

    // Mana pool should still have the 3 green (PlayLand doesn't clear pools)
    assert_eq!(new_state.player(PlayerId(1)).unwrap().mana_pool.green, 3);
}
