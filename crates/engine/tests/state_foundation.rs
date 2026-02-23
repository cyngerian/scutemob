//! Tests for basic GameState construction, field defaults, and accessors.

use mtg_engine::state::*;

#[test]
fn test_four_player_construction_defaults() {
    let state = GameStateBuilder::four_player().build().unwrap();

    // 4 players created
    assert_eq!(state.players.len(), 4);

    // Each player has correct defaults
    for pid in 1..=4u64 {
        let player = state.player(PlayerId(pid)).unwrap();
        assert_eq!(player.life_total, 40, "Commander starting life is 40");
        assert!(player.mana_pool.is_empty());
        assert_eq!(player.poison_counters, 0);
        assert_eq!(player.land_plays_remaining, 1);
        assert!(!player.has_drawn_for_turn);
        assert!(!player.has_lost);
        assert!(!player.has_conceded);
        assert!(player.commander_ids.is_empty());
        assert!(player.commander_tax.is_empty());
        assert!(player.commander_damage_received.is_empty());
    }
}

#[test]
fn test_four_player_turn_state_defaults() {
    let state = GameStateBuilder::four_player().build().unwrap();

    assert_eq!(state.turn.active_player, PlayerId(1));
    assert_eq!(state.turn.priority_holder, Some(PlayerId(1)));
    assert_eq!(state.turn.phase, Phase::PreCombatMain);
    assert_eq!(state.turn.step, Step::PreCombatMain);
    assert_eq!(state.turn.turn_number, 1);
    assert!(state.turn.players_passed.is_empty());
    assert!(state.turn.extra_turns.is_empty());

    // Turn order is 1, 2, 3, 4
    let order: Vec<PlayerId> = state.turn.turn_order.iter().copied().collect();
    assert_eq!(
        order,
        vec![PlayerId(1), PlayerId(2), PlayerId(3), PlayerId(4)]
    );
}

#[test]
fn test_zones_created_for_each_player() {
    let state = GameStateBuilder::four_player().build().unwrap();

    // Shared zones exist
    assert!(state.zone(&ZoneId::Battlefield).is_ok());
    assert!(state.zone(&ZoneId::Stack).is_ok());
    assert!(state.zone(&ZoneId::Exile).is_ok());

    // Per-player zones exist for each player
    for pid in 1..=4u64 {
        let id = PlayerId(pid);
        assert!(state.zone(&ZoneId::Library(id)).is_ok());
        assert!(state.zone(&ZoneId::Hand(id)).is_ok());
        assert!(state.zone(&ZoneId::Graveyard(id)).is_ok());
        assert!(state.zone(&ZoneId::Command(id)).is_ok());
    }

    // Total zones: 3 shared + 4 per-player * 4 players = 19
    assert_eq!(state.zones.len(), 19);
}

#[test]
fn test_empty_state_has_no_objects() {
    let state = GameStateBuilder::four_player().build().unwrap();
    assert_eq!(state.total_objects(), 0);
    assert!(state.objects.is_empty());
}

#[test]
fn test_player_accessor_not_found() {
    let state = GameStateBuilder::four_player().build().unwrap();
    let result = state.player(PlayerId(99));
    assert!(result.is_err());
}

#[test]
fn test_object_accessor_not_found() {
    let state = GameStateBuilder::four_player().build().unwrap();
    let result = state.object(ObjectId(99));
    assert!(result.is_err());
}

#[test]
fn test_zone_accessor_not_found() {
    let state = GameStateBuilder::four_player().build().unwrap();
    // Player 99 doesn't exist, so their library zone doesn't exist
    let result = state.zone(&ZoneId::Library(PlayerId(99)));
    assert!(result.is_err());
}

#[test]
fn test_custom_player_life() {
    let state = GameStateBuilder::new()
        .add_player_with(PlayerId(1), |p| p.life(20))
        .add_player_with(PlayerId(2), |p| p.life(30))
        .build()
        .unwrap();

    assert_eq!(state.player(PlayerId(1)).unwrap().life_total, 20);
    assert_eq!(state.player(PlayerId(2)).unwrap().life_total, 30);
}

#[test]
fn test_mana_pool_operations() {
    let mut pool = ManaPool::default();
    assert!(pool.is_empty());
    assert_eq!(pool.total(), 0);

    pool.add(ManaColor::White, 2);
    pool.add(ManaColor::Blue, 1);
    pool.add(ManaColor::Colorless, 3);

    assert!(!pool.is_empty());
    assert_eq!(pool.total(), 6);
    assert_eq!(pool.white, 2);
    assert_eq!(pool.blue, 1);
    assert_eq!(pool.colorless, 3);

    pool.empty();
    assert!(pool.is_empty());
    assert_eq!(pool.total(), 0);
}

#[test]
fn test_mana_cost_mana_value() {
    let cost = ManaCost {
        generic: 3,
        white: 1,
        blue: 1,
        ..ManaCost::default()
    };
    // {3}{W}{U} = mana value 5
    assert_eq!(cost.mana_value(), 5);

    let free = ManaCost::default();
    assert_eq!(free.mana_value(), 0);
}

#[test]
fn test_active_players_excludes_lost() {
    let mut state = GameStateBuilder::four_player().build().unwrap();

    // All 4 active initially
    assert_eq!(state.active_players().len(), 4);

    // Player 2 loses
    state.player_mut(PlayerId(2)).unwrap().has_lost = true;
    assert_eq!(state.active_players().len(), 3);
    assert!(!state.active_players().contains(&PlayerId(2)));

    // Player 4 concedes
    state.player_mut(PlayerId(4)).unwrap().has_conceded = true;
    assert_eq!(state.active_players().len(), 2);
    let active = state.active_players();
    assert!(active.contains(&PlayerId(1)));
    assert!(active.contains(&PlayerId(3)));
}

#[test]
fn test_turn_number_configurable() {
    let state = GameStateBuilder::four_player()
        .turn_number(5)
        .build()
        .unwrap();
    assert_eq!(state.turn.turn_number, 5);
}
