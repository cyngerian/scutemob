//! State-Based Action tests (CR 704).
//!
//! Each test targets a specific SBA in isolation, then integration tests
//! cover chains and convergence.

use mtg_engine::state::builder::{GameStateBuilder, ObjectSpec};
use mtg_engine::state::player::{CardId, PlayerId};
use mtg_engine::state::turn::Step;
use mtg_engine::state::types::{CounterType, SubType, SuperType};
use mtg_engine::state::zone::ZoneId;
use mtg_engine::{start_game, GameEvent, LossReason};

// ── Helpers ──────────────────────────────────────────────────────────────────

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

/// Collect SBA-related events from a `start_game` call on a state that has SBA conditions.
fn sba_events_from_start(state: mtg_engine::state::GameState) -> Vec<GameEvent> {
    start_game(state).unwrap().1
}

// ── CR 704.5a: Life total ≤ 0 ─────────────────────────────────────────────

#[test]
/// CR 704.5a — player at exactly 0 life loses as SBA
fn test_sba_704_5a_player_at_zero_life_loses() {
    let state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .player_life(p(1), 0)
        .at_step(Step::PreCombatMain)
        .active_player(p(1))
        .build();

    let events = sba_events_from_start(state);

    let lost = events.iter().any(|e| {
        matches!(e, GameEvent::PlayerLost { player, reason }
            if *player == p(1) && *reason == LossReason::LifeTotal)
    });
    assert!(lost, "player at 0 life should lose via SBA");
}

#[test]
/// CR 704.5a — player at negative life loses as SBA
fn test_sba_704_5a_player_at_negative_life_loses() {
    let state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .player_life(p(1), -3)
        .at_step(Step::PreCombatMain)
        .active_player(p(1))
        .build();

    let events = sba_events_from_start(state);

    let lost = events.iter().any(|e| {
        matches!(e, GameEvent::PlayerLost { player, reason }
            if *player == p(1) && *reason == LossReason::LifeTotal)
    });
    assert!(lost, "player at -3 life should lose via SBA");
}

#[test]
/// CR 704.5a — player at 1 life does NOT lose
fn test_sba_704_5a_player_at_one_life_survives() {
    let state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .player_life(p(1), 1)
        .at_step(Step::PreCombatMain)
        .active_player(p(1))
        .build();

    let events = sba_events_from_start(state);

    let lost = events
        .iter()
        .any(|e| matches!(e, GameEvent::PlayerLost { player, .. } if *player == p(1)));
    assert!(!lost, "player at 1 life should NOT lose");
}

#[test]
/// CR 704.5a — multiple players at 0 life simultaneously all lose
fn test_sba_704_5a_multiple_players_lose_simultaneously() {
    let state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .add_player(p(3))
        .player_life(p(1), 0)
        .player_life(p(2), -1)
        .player_life(p(3), 40)
        .at_step(Step::PreCombatMain)
        .active_player(p(1))
        .build();

    let events = sba_events_from_start(state);

    let p1_lost = events.iter().any(|e| {
        matches!(e, GameEvent::PlayerLost { player, reason }
            if *player == p(1) && *reason == LossReason::LifeTotal)
    });
    let p2_lost = events.iter().any(|e| {
        matches!(e, GameEvent::PlayerLost { player, reason }
            if *player == p(2) && *reason == LossReason::LifeTotal)
    });
    let p3_lost = events
        .iter()
        .any(|e| matches!(e, GameEvent::PlayerLost { player, .. } if *player == p(3)));

    assert!(p1_lost, "player 1 at 0 life should lose");
    assert!(p2_lost, "player 2 at -1 life should lose");
    assert!(!p3_lost, "player 3 at 40 life should NOT lose");
}

// ── CR 704.5c: Poison counters ─────────────────────────────────────────────

#[test]
/// CR 704.5c — player with 10 poison counters loses
fn test_sba_704_5c_ten_poison_loses() {
    let state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .player_poison(p(1), 10)
        .at_step(Step::PreCombatMain)
        .active_player(p(1))
        .build();

    let events = sba_events_from_start(state);

    let lost = events.iter().any(|e| {
        matches!(e, GameEvent::PlayerLost { player, reason }
            if *player == p(1) && *reason == LossReason::PoisonCounters)
    });
    assert!(lost, "player with 10 poison should lose");
}

#[test]
/// CR 704.5c — player with 9 poison counters does NOT lose
fn test_sba_704_5c_nine_poison_survives() {
    let state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .player_poison(p(1), 9)
        .at_step(Step::PreCombatMain)
        .active_player(p(1))
        .build();

    let events = sba_events_from_start(state);

    let lost = events
        .iter()
        .any(|e| matches!(e, GameEvent::PlayerLost { player, .. } if *player == p(1)));
    assert!(!lost, "player with 9 poison should NOT lose");
}

// ── CR 704.5d: Token in non-battlefield zone ───────────────────────────────

#[test]
/// CR 704.5d — token in graveyard ceases to exist
fn test_sba_704_5d_token_in_graveyard_ceases_to_exist() {
    let state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .object(
            ObjectSpec::creature(p(1), "Bear Token", 2, 2)
                .token()
                .in_zone(ZoneId::Graveyard(p(1))),
        )
        .at_step(Step::PreCombatMain)
        .active_player(p(1))
        .build();

    let events = sba_events_from_start(state);

    let ceased = events
        .iter()
        .any(|e| matches!(e, GameEvent::TokenCeasedToExist { .. }));
    assert!(ceased, "token in graveyard should cease to exist");
}

#[test]
/// CR 704.5d — token on battlefield is NOT removed
fn test_sba_704_5d_token_on_battlefield_stays() {
    let state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .object(ObjectSpec::creature(p(1), "Bear Token", 2, 2).token())
        .at_step(Step::PreCombatMain)
        .active_player(p(1))
        .build();

    let events = sba_events_from_start(state);

    let ceased = events
        .iter()
        .any(|e| matches!(e, GameEvent::TokenCeasedToExist { .. }));
    assert!(!ceased, "token on battlefield should NOT cease to exist");
}

// ── CR 704.5f: Creature with toughness ≤ 0 ────────────────────────────────

#[test]
/// CR 704.5f — creature with 0 toughness goes to graveyard
fn test_sba_704_5f_zero_toughness_creature_dies() {
    let state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .object(ObjectSpec::creature(p(1), "Dying Creature", 2, 0))
        .at_step(Step::PreCombatMain)
        .active_player(p(1))
        .build();

    let events = sba_events_from_start(state);

    let died = events
        .iter()
        .any(|e| matches!(e, GameEvent::CreatureDied { .. }));
    assert!(died, "0 toughness creature should go to graveyard via SBA");
}

#[test]
/// CR 704.5f — creature with -1 toughness goes to graveyard
fn test_sba_704_5f_negative_toughness_creature_dies() {
    let state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .object(ObjectSpec::creature(p(1), "Dying Creature", 5, -1))
        .at_step(Step::PreCombatMain)
        .active_player(p(1))
        .build();

    let events = sba_events_from_start(state);

    let died = events
        .iter()
        .any(|e| matches!(e, GameEvent::CreatureDied { .. }));
    assert!(died, "-1 toughness creature should go to graveyard via SBA");
}

#[test]
/// CR 704.5f — creature with 1 toughness and no damage survives
fn test_sba_704_5f_positive_toughness_creature_survives() {
    let state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .object(ObjectSpec::creature(p(1), "Survivor", 1, 1))
        .at_step(Step::PreCombatMain)
        .active_player(p(1))
        .build();

    let events = sba_events_from_start(state);

    let died = events
        .iter()
        .any(|e| matches!(e, GameEvent::CreatureDied { .. }));
    assert!(!died, "1/1 with no damage should NOT die via SBA");
}

// ── CR 704.5g: Creature with lethal damage ─────────────────────────────────

#[test]
/// CR 704.5g — creature with damage equal to toughness is destroyed
fn test_sba_704_5g_lethal_damage_destroys_creature() {
    let state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .object(ObjectSpec::creature(p(1), "Damaged Bear", 2, 2).with_damage(2))
        .at_step(Step::PreCombatMain)
        .active_player(p(1))
        .build();

    let events = sba_events_from_start(state);

    let died = events
        .iter()
        .any(|e| matches!(e, GameEvent::CreatureDied { .. }));
    assert!(died, "creature with lethal damage should be destroyed");
}

#[test]
/// CR 704.5g — creature with less-than-lethal damage survives
fn test_sba_704_5g_nonlethal_damage_survives() {
    let state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .object(ObjectSpec::creature(p(1), "Wounded Bear", 2, 3).with_damage(2))
        .at_step(Step::PreCombatMain)
        .active_player(p(1))
        .build();

    let events = sba_events_from_start(state);

    let died = events
        .iter()
        .any(|e| matches!(e, GameEvent::CreatureDied { .. }));
    assert!(
        !died,
        "creature with 2 damage on 3 toughness should NOT die"
    );
}

// ── CR 704.5h: Creature with deathtouch damage ─────────────────────────────

#[test]
/// CR 704.5h — creature with deathtouch damage (even 1) is destroyed
fn test_sba_704_5h_deathtouch_damage_destroys_creature() {
    let state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .object(
            ObjectSpec::creature(p(1), "Poisoned Creature", 2, 5)
                .with_damage(1)
                .with_deathtouch_damage(),
        )
        .at_step(Step::PreCombatMain)
        .active_player(p(1))
        .build();

    let events = sba_events_from_start(state);

    let died = events
        .iter()
        .any(|e| matches!(e, GameEvent::CreatureDied { .. }));
    assert!(died, "creature with deathtouch damage should be destroyed");
}

// ── CR 704.5i: Planeswalker with 0 loyalty ─────────────────────────────────

#[test]
/// CR 704.5i — planeswalker at 0 loyalty goes to graveyard
fn test_sba_704_5i_planeswalker_zero_loyalty_dies() {
    let state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .object(ObjectSpec::planeswalker(p(1), "Dying Walker", 0))
        .at_step(Step::PreCombatMain)
        .active_player(p(1))
        .build();

    let events = sba_events_from_start(state);

    let died = events
        .iter()
        .any(|e| matches!(e, GameEvent::PlaneswalkerDied { .. }));
    assert!(died, "planeswalker at 0 loyalty should go to graveyard");
}

#[test]
/// CR 704.5i — planeswalker at 1 loyalty survives
fn test_sba_704_5i_planeswalker_nonzero_loyalty_survives() {
    let state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .object(ObjectSpec::planeswalker(p(1), "Jace", 3))
        .at_step(Step::PreCombatMain)
        .active_player(p(1))
        .build();

    let events = sba_events_from_start(state);

    let died = events
        .iter()
        .any(|e| matches!(e, GameEvent::PlaneswalkerDied { .. }));
    assert!(!died, "planeswalker at 3 loyalty should NOT die");
}

// ── CR 704.5j: Legendary rule ──────────────────────────────────────────────

#[test]
/// CR 704.5j — two legendary permanents with same name: one goes to graveyard
fn test_sba_704_5j_legendary_rule_removes_duplicate() {
    let state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .object(
            ObjectSpec::creature(p(1), "Thalia", 2, 1).with_supertypes(vec![SuperType::Legendary]),
        )
        .object(
            ObjectSpec::creature(p(1), "Thalia", 2, 1).with_supertypes(vec![SuperType::Legendary]),
        )
        .at_step(Step::PreCombatMain)
        .active_player(p(1))
        .build();

    let events = sba_events_from_start(state);

    let applied = events.iter().any(|e| {
        matches!(e, GameEvent::LegendaryRuleApplied { put_to_graveyard, .. }
            if put_to_graveyard.len() == 1)
    });
    assert!(
        applied,
        "legendary rule should remove one of two duplicates"
    );
}

#[test]
/// CR 704.5j — two legendaries with different names: no SBA
fn test_sba_704_5j_different_name_legendaries_no_sba() {
    let state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .object(
            ObjectSpec::creature(p(1), "Thalia", 2, 1).with_supertypes(vec![SuperType::Legendary]),
        )
        .object(
            ObjectSpec::creature(p(1), "Gaddock Teeg", 2, 2)
                .with_supertypes(vec![SuperType::Legendary]),
        )
        .at_step(Step::PreCombatMain)
        .active_player(p(1))
        .build();

    let events = sba_events_from_start(state);

    let applied = events
        .iter()
        .any(|e| matches!(e, GameEvent::LegendaryRuleApplied { .. }));
    assert!(
        !applied,
        "different-named legendaries should NOT trigger legendary rule"
    );
}

#[test]
/// CR 704.5j — two legendaries with same name controlled by different players: no SBA
fn test_sba_704_5j_same_name_different_controllers_no_sba() {
    let state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .object(
            ObjectSpec::creature(p(1), "Thalia", 2, 1).with_supertypes(vec![SuperType::Legendary]),
        )
        .object(
            ObjectSpec::creature(p(2), "Thalia", 2, 1).with_supertypes(vec![SuperType::Legendary]),
        )
        .at_step(Step::PreCombatMain)
        .active_player(p(1))
        .build();

    let events = sba_events_from_start(state);

    let applied = events
        .iter()
        .any(|e| matches!(e, GameEvent::LegendaryRuleApplied { .. }));
    assert!(
        !applied,
        "same-named legendaries under different controllers should NOT trigger rule"
    );
}

// ── CR 704.5m: Aura attached to illegal object ─────────────────────────────

#[test]
/// CR 704.5m — aura with no attached_to goes to graveyard
fn test_sba_704_5m_unattached_aura_goes_to_graveyard() {
    let state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .object(
            ObjectSpec::enchantment(p(1), "Rancor")
                .with_subtypes(vec![SubType("Aura".to_string())]),
        )
        .at_step(Step::PreCombatMain)
        .active_player(p(1))
        .build();

    let events = sba_events_from_start(state);

    let fell_off = events
        .iter()
        .any(|e| matches!(e, GameEvent::AuraFellOff { .. }));
    assert!(
        fell_off,
        "unattached aura on battlefield should go to graveyard"
    );
}

// ── CR 704.5n: Equipment attached illegally ────────────────────────────────

#[test]
/// CR 704.5n — equipment attached to non-creature becomes unattached
fn test_sba_704_5n_equipment_on_non_creature_unattaches() {
    // Build a state with equipment attached to a land (illegal).
    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .object(
            ObjectSpec::artifact(p(1), "Sword of Stuff")
                .with_subtypes(vec![SubType("Equipment".to_string())]),
        )
        .object(ObjectSpec::land(p(1), "Forest"))
        .at_step(Step::PreCombatMain)
        .active_player(p(1))
        .build();

    // Manually attach sword to the land (simulating illegal attachment).
    let sword_id = state
        .objects
        .iter()
        .find(|(_, o)| o.characteristics.name == "Sword of Stuff")
        .map(|(id, _)| *id)
        .unwrap();
    let land_id = state
        .objects
        .iter()
        .find(|(_, o)| o.characteristics.name == "Forest")
        .map(|(id, _)| *id)
        .unwrap();

    if let Some(sword) = state.objects.get_mut(&sword_id) {
        sword.attached_to = Some(land_id);
    }
    if let Some(land) = state.objects.get_mut(&land_id) {
        land.attachments.push_back(sword_id);
    }

    let (_, events) = start_game(state).unwrap();

    let unattached: bool = events.iter().any(
        |e| matches!(e, GameEvent::EquipmentUnattached { object_id } if object_id == &sword_id),
    );
    assert!(
        unattached,
        "equipment on non-creature should become unattached"
    );
}

// ── CR 704.5q: Counter annihilation ────────────────────────────────────────

#[test]
/// CR 704.5q — equal +1/+1 and -1/-1 counters annihilate completely
fn test_sba_704_5q_equal_counters_annihilate() {
    let state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .object(
            ObjectSpec::creature(p(1), "Modular Bear", 2, 2)
                .with_counter(CounterType::PlusOnePlusOne, 2)
                .with_counter(CounterType::MinusOneMinusOne, 2),
        )
        .at_step(Step::PreCombatMain)
        .active_player(p(1))
        .build();

    let (new_state, events) = start_game(state).unwrap();

    let annihilated: bool = events
        .iter()
        .any(|e| matches!(e, GameEvent::CountersAnnihilated { amount, .. } if *amount == 2));
    assert!(annihilated, "equal counter pairs should annihilate");

    // Verify counters are gone from state.
    let bear = new_state
        .objects
        .values()
        .find(|o| o.characteristics.name == "Modular Bear")
        .unwrap();
    assert_eq!(bear.counters.get(&CounterType::PlusOnePlusOne), None);
    assert_eq!(bear.counters.get(&CounterType::MinusOneMinusOne), None);
}

#[test]
/// CR 704.5q — 3 +1/+1 and 2 -1/-1 → 1 +1/+1 remains
fn test_sba_704_5q_unequal_counters_partial_annihilation() {
    let state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .object(
            ObjectSpec::creature(p(1), "Persist Bear", 2, 2)
                .with_counter(CounterType::PlusOnePlusOne, 3)
                .with_counter(CounterType::MinusOneMinusOne, 2),
        )
        .at_step(Step::PreCombatMain)
        .active_player(p(1))
        .build();

    let (new_state, events) = start_game(state).unwrap();

    let annihilated: bool = events
        .iter()
        .any(|e| matches!(e, GameEvent::CountersAnnihilated { amount, .. } if *amount == 2));
    assert!(annihilated, "2 pairs should be annihilated");

    let bear = new_state
        .objects
        .values()
        .find(|o| o.characteristics.name == "Persist Bear")
        .unwrap();
    assert_eq!(
        bear.counters.get(&CounterType::PlusOnePlusOne).copied(),
        Some(1)
    );
    assert_eq!(bear.counters.get(&CounterType::MinusOneMinusOne), None);
}

// ── CR 704.5u: Commander damage ────────────────────────────────────────────

#[test]
/// CR 704.5u — player who received 21 combat damage from one commander loses
fn test_sba_704_5u_commander_damage_21_loses() {
    use im::OrdMap;

    let commander_card = CardId("thalia-card".to_string());

    let state = {
        let mut b = GameStateBuilder::new()
            .add_player(p(1))
            .add_player(p(2))
            .player_commander(p(2), commander_card.clone())
            .at_step(Step::PreCombatMain)
            .active_player(p(1))
            .build();

        // Set player 1's commander damage received from player 2's commander to 21.
        if let Some(player1) = b.players.get_mut(&p(1)) {
            let mut inner: OrdMap<CardId, u32> = OrdMap::new();
            inner.insert(commander_card.clone(), 21);
            player1.commander_damage_received.insert(p(2), inner);
        }
        b
    };

    let events = sba_events_from_start(state);

    let lost = events.iter().any(|e| {
        matches!(e, GameEvent::PlayerLost { player, reason }
            if *player == p(1) && *reason == LossReason::CommanderDamage)
    });
    assert!(lost, "player with 21 commander damage should lose");
}

#[test]
/// CR 704.5u — player with 20 commander damage does NOT lose
fn test_sba_704_5u_commander_damage_20_survives() {
    use im::OrdMap;

    let commander_card = CardId("thalia-card".to_string());

    let state = {
        let mut b = GameStateBuilder::new()
            .add_player(p(1))
            .add_player(p(2))
            .player_commander(p(2), commander_card.clone())
            .at_step(Step::PreCombatMain)
            .active_player(p(1))
            .build();

        if let Some(player1) = b.players.get_mut(&p(1)) {
            let mut inner: OrdMap<CardId, u32> = OrdMap::new();
            inner.insert(commander_card.clone(), 20);
            player1.commander_damage_received.insert(p(2), inner);
        }
        b
    };

    let events = sba_events_from_start(state);

    let lost = events
        .iter()
        .any(|e| matches!(e, GameEvent::PlayerLost { player, .. } if *player == p(1)));
    assert!(!lost, "player with 20 commander damage should NOT lose");
}

// ── SBA fixed-point / chaining ─────────────────────────────────────────────

#[test]
/// SBA fixed-point: SBAs run until no more apply.
/// Setup: two creatures, one at 0 toughness, one healthy.
/// Only the one with 0 toughness should die.
fn test_sba_convergence_only_applicable_sbas_fire() {
    let state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .object(ObjectSpec::creature(p(1), "Doomed", 2, 0))
        .object(ObjectSpec::creature(p(1), "Healthy", 3, 3))
        .at_step(Step::PreCombatMain)
        .active_player(p(1))
        .build();

    let (new_state, events) = start_game(state).unwrap();

    let died_count: usize = events
        .iter()
        .filter(|e| matches!(e, GameEvent::CreatureDied { .. }))
        .count();
    assert_eq!(died_count, 1, "exactly one creature should die");

    // Verify the healthy creature is still on the battlefield.
    let healthy_on_battlefield = new_state
        .objects
        .values()
        .any(|o| o.characteristics.name == "Healthy" && o.zone == ZoneId::Battlefield);
    assert!(
        healthy_on_battlefield,
        "healthy creature should still be on battlefield"
    );
}

#[test]
/// SBA interaction: SBAs do not re-trigger on the same event.
/// A creature goes to 0 toughness exactly once — no infinite loop.
fn test_sba_no_infinite_loop_on_repeated_sba() {
    // A single 0-toughness creature. Should fire SBA once and stop.
    let state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .object(ObjectSpec::creature(p(1), "Doomed", 1, 0))
        .at_step(Step::PreCombatMain)
        .active_player(p(1))
        .build();

    // This should complete without hanging.
    let (_, events) = start_game(state).unwrap();

    let died_count: usize = events
        .iter()
        .filter(|e| matches!(e, GameEvent::CreatureDied { .. }))
        .count();
    assert_eq!(died_count, 1, "creature should die exactly once");
}

// ── SBA trigger integration ───────────────────────────────────────────────

#[test]
/// SBAs fire before priority is granted at step start.
/// A 0-toughness creature on the battlefield when a new step begins → dies before any priority.
fn test_sba_fire_before_first_priority_grant() {
    let state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .object(ObjectSpec::creature(p(1), "Pre-dead", 1, 0))
        .at_step(Step::PreCombatMain)
        .active_player(p(1))
        .build();

    let (_, events) = start_game(state).unwrap();

    // CreatureDied should come BEFORE the first PriorityGiven.
    let died_pos: Option<usize> = events
        .iter()
        .position(|e| matches!(e, GameEvent::CreatureDied { .. }));
    let priority_pos: Option<usize> = events
        .iter()
        .position(|e| matches!(e, GameEvent::PriorityGiven { .. }));

    assert!(died_pos.is_some(), "creature should die");
    assert!(priority_pos.is_some(), "priority should be given");
    assert!(
        died_pos.unwrap() < priority_pos.unwrap(),
        "CreatureDied should come before first PriorityGiven"
    );
}
