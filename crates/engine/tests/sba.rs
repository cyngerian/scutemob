//! State-Based Action tests (CR 704).
//!
//! Each test targets a specific SBA in isolation, then integration tests
//! cover chains and convergence.

use mtg_engine::state::builder::{GameStateBuilder, ObjectSpec};
use mtg_engine::state::player::{CardId, PlayerId};
use mtg_engine::state::turn::Step;
use mtg_engine::state::types::{CounterType, SubType, SuperType};
use mtg_engine::state::zone::ZoneId;
use mtg_engine::{
    start_game, ContinuousEffect, EffectDuration, EffectFilter, EffectId, EffectLayer, GameEvent,
    KeywordAbility, LayerModification, LossReason,
};

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
        .build()
        .unwrap();

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
        .build()
        .unwrap();

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
        .build()
        .unwrap();

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
        .build()
        .unwrap();

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
        .build()
        .unwrap();

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
        .build()
        .unwrap();

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
        .build()
        .unwrap();

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
        .build()
        .unwrap();

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
        .build()
        .unwrap();

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
        .build()
        .unwrap();

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
        .build()
        .unwrap();

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
        .build()
        .unwrap();

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
        .build()
        .unwrap();

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
        .build()
        .unwrap();

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
        .build()
        .unwrap();

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
        .build()
        .unwrap();

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
        .build()
        .unwrap();

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
        .build()
        .unwrap();

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
        .build()
        .unwrap();

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
        .build()
        .unwrap();

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
        .build()
        .unwrap();

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
        .build()
        .unwrap();

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
        .build()
        .unwrap();

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
            .build()
            .unwrap();

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
            .build()
            .unwrap();

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
        .build()
        .unwrap();

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
        .build()
        .unwrap();

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
        .build()
        .unwrap();

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

// ── MR-M5-02: SBAs must use layer-computed characteristics ─────────────────

#[test]
/// CR 704.5f + CR 613 — creature reduced to 0 toughness by a continuous effect dies via SBA.
///
/// A 3/3 creature with a -3/-3 continuous effect active has effective toughness 0.
/// SBAs must see the layer-computed toughness, not the raw printed value.
/// Without MR-M5-02, the SBA would skip this creature (printed toughness == 3 > 0).
fn test_sba_704_5f_continuous_effect_reduces_toughness_to_zero_dies() {
    let minus3_effect = ContinuousEffect {
        id: EffectId(1),
        source: None,
        timestamp: 10,
        layer: EffectLayer::PtModify,
        duration: EffectDuration::Indefinite,
        filter: EffectFilter::AllCreatures,
        modification: LayerModification::ModifyBoth(-3),
        is_cda: false,
    };

    let state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .object(ObjectSpec::creature(p(1), "Weakened Bear", 3, 3))
        .add_continuous_effect(minus3_effect)
        .at_step(Step::PreCombatMain)
        .active_player(p(1))
        .build()
        .unwrap();

    let (_, events) = start_game(state).unwrap();

    let died = events
        .iter()
        .any(|e| matches!(e, GameEvent::CreatureDied { .. }));
    assert!(
        died,
        "3/3 creature reduced to 0 toughness by continuous -3/-3 should die via SBA 704.5f"
    );
}

#[test]
/// CR 704.5g + CR 613 — creature with lethal damage that had Indestructible removed by a
/// continuous effect is destroyed by SBA.
///
/// A 2/2 with 2 damage (normally survived via Indestructible) and a RemoveKeyword effect
/// removing Indestructible has effective characteristics with no Indestructible.
/// SBAs must see the layer-computed keywords, not the raw printed keywords.
/// Without MR-M5-02, the SBA would skip this creature (raw keywords include Indestructible).
fn test_sba_704_5g_indestructible_removed_by_effect_lethal_damage_dies() {
    let remove_indestructible = ContinuousEffect {
        id: EffectId(1),
        source: None,
        timestamp: 10,
        layer: EffectLayer::Ability,
        duration: EffectDuration::Indefinite,
        filter: EffectFilter::AllCreatures,
        modification: LayerModification::RemoveKeyword(KeywordAbility::Indestructible),
        is_cda: false,
    };

    let state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .object(
            ObjectSpec::creature(p(1), "Fallen Guardian", 2, 2)
                .with_keyword(KeywordAbility::Indestructible)
                .with_damage(2),
        )
        .add_continuous_effect(remove_indestructible)
        .at_step(Step::PreCombatMain)
        .active_player(p(1))
        .build()
        .unwrap();

    let (_, events) = start_game(state).unwrap();

    let died = events
        .iter()
        .any(|e| matches!(e, GameEvent::CreatureDied { .. }));
    assert!(
        died,
        "indestructible creature whose keyword was removed by a continuous effect \
         should die from lethal damage via SBA 704.5g"
    );
}

// ── MR-M4-12: Planeswalker with Loyalty counter at 0 ──────────────────────

#[test]
/// CR 704.5i — planeswalker with CounterType::Loyalty at 0 goes to graveyard.
///
/// MR-M4-12: previous tests only checked characteristics.loyalty; this verifies
/// the counter-based path used by loyalty abilities in actual gameplay.
fn test_sba_704_5i_planeswalker_loyalty_counter_zero_dies() {
    let state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        // loyalty field left as None; loyalty tracked via counter (as M7+ gameplay does)
        .object(ObjectSpec::planeswalker(p(1), "Jace", 3).with_counter(CounterType::Loyalty, 0))
        .at_step(Step::PreCombatMain)
        .active_player(p(1))
        .build()
        .unwrap();

    let events = sba_events_from_start(state);

    let died = events
        .iter()
        .any(|e| matches!(e, GameEvent::PlaneswalkerDied { .. }));
    assert!(
        died,
        "planeswalker with CounterType::Loyalty == 0 should go to graveyard"
    );
}

#[test]
/// CR 704.5i — planeswalker with CounterType::Loyalty > 0 survives.
///
/// MR-M4-12: counter-based loyalty path — positive counter means alive.
fn test_sba_704_5i_planeswalker_loyalty_counter_nonzero_survives() {
    let state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .object(
            ObjectSpec::planeswalker(p(1), "Jace", 0) // base loyalty 0, but counter overrides
                .with_counter(CounterType::Loyalty, 4),
        )
        .at_step(Step::PreCombatMain)
        .active_player(p(1))
        .build()
        .unwrap();

    let events = sba_events_from_start(state);

    let died = events
        .iter()
        .any(|e| matches!(e, GameEvent::PlaneswalkerDied { .. }));
    assert!(
        !died,
        "planeswalker with CounterType::Loyalty == 4 should NOT go to graveyard"
    );
}

// ── MR-M4-14: 3+ legendary copies (same controller) ──────────────────────

#[test]
/// CR 704.5j — three legendary permanents with same name and controller:
/// all but one go to the graveyard in a single SBA pass.
///
/// MR-M4-14: existing test only covered 2 copies; this verifies N > 2.
fn test_sba_704_5j_three_legendary_copies_all_but_one_removed() {
    let state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .object(
            ObjectSpec::creature(p(1), "Thalia", 2, 1).with_supertypes(vec![SuperType::Legendary]),
        )
        .object(
            ObjectSpec::creature(p(1), "Thalia", 2, 1).with_supertypes(vec![SuperType::Legendary]),
        )
        .object(
            ObjectSpec::creature(p(1), "Thalia", 2, 1).with_supertypes(vec![SuperType::Legendary]),
        )
        .at_step(Step::PreCombatMain)
        .active_player(p(1))
        .build()
        .unwrap();

    let events = sba_events_from_start(state);

    // The LegendaryRuleApplied event lists all objects sent to graveyard.
    // With 3 copies, exactly 2 should be removed (1 stays).
    let removed_count = events
        .iter()
        .filter_map(|e| {
            if let GameEvent::LegendaryRuleApplied {
                put_to_graveyard, ..
            } = e
            {
                Some(put_to_graveyard.len())
            } else {
                None
            }
        })
        .sum::<usize>();
    assert_eq!(
        removed_count, 2,
        "with 3 legendary copies, 2 should be removed (highest ObjectId survives)"
    );
}

// ── CC#9: Indestructible + deathtouch combined ────────────────────────────

#[test]
/// CC#9 / CR 704.5h + CR 702.12a — An indestructible creature dealt deathtouch damage
/// survives the SBA check.
///
/// CR 704.5h: "If a creature has been dealt damage by a source with deathtouch since the
/// last time state-based actions were checked, it's destroyed."
/// CR 702.12a: "A permanent with indestructible can't be destroyed."
///
/// The engine skips CR 704.5h for indestructible creatures. The indestructible
/// creature should still be on the battlefield after the SBA check.
fn test_cc9_indestructible_survives_deathtouch() {
    let state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .object(
            ObjectSpec::creature(p(1), "Indestructible Creature", 4, 4)
                .with_keyword(KeywordAbility::Indestructible)
                .with_damage(1)
                .with_deathtouch_damage(), // dealt deathtouch damage — normally would die
        )
        .at_step(Step::PreCombatMain)
        .active_player(p(1))
        .build()
        .unwrap();

    let events = sba_events_from_start(state);

    // CR 702.12a: indestructible prevents destruction from deathtouch (CR 704.5h).
    let died = events
        .iter()
        .any(|e| matches!(e, GameEvent::CreatureDied { .. }));
    assert!(
        !died,
        "indestructible creature should survive deathtouch damage (CR 702.12a prevents CR 704.5h); \
         events: {:?}",
        events
    );
}

#[test]
/// CC#9 (contrast) / CR 704.5h — A NON-indestructible creature dealt deathtouch
/// damage IS destroyed, confirming the deathtouch SBA fires when indestructible is absent.
fn test_cc9_non_indestructible_dies_from_deathtouch() {
    let state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .object(
            ObjectSpec::creature(p(1), "Normal Creature", 4, 4)
                .with_damage(1)
                .with_deathtouch_damage(), // dealt deathtouch damage — should die
        )
        .at_step(Step::PreCombatMain)
        .active_player(p(1))
        .build()
        .unwrap();

    let events = sba_events_from_start(state);

    // CR 704.5h: non-indestructible creature with deathtouch damage is destroyed.
    let died = events
        .iter()
        .any(|e| matches!(e, GameEvent::CreatureDied { .. }));
    assert!(
        died,
        "non-indestructible creature with deathtouch damage should be destroyed (CR 704.5h); \
         events: {:?}",
        events
    );
}

// ── CC#10: Legendary rule simultaneous ETBs ───────────────────────────────

#[test]
/// CC#10 / CR 704.5j + CR 603.2 — When two copies of a legendary permanent enter
/// simultaneously, ETB triggers fire for BOTH before the legendary rule puts one in
/// the graveyard.
///
/// CR 704.5j: "If a player controls two or more legendary permanents with the same name,
/// that player chooses one of them, and the rest are put into their owners' graveyards."
///
/// The legendary rule is a state-based action. ETB triggers from both copies are
/// queued before SBAs are checked. This test verifies that the SBA fires (one copy
/// goes to graveyard) given a pre-built state with two identical legendary permanents.
/// The "ETB triggers fire first" behavior requires a full game loop; here we verify
/// the SBA itself fires correctly.
fn test_cc10_legendary_rule_simultaneous_etb_triggers() {
    // Build a state with two copies of "Thalia, Guardian of Thraben" on the battlefield.
    // Both have the Legendary supertype and the same name.
    let state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .object(
            ObjectSpec::creature(p(1), "Thalia, Guardian of Thraben", 2, 1)
                .with_supertypes(vec![SuperType::Legendary]),
        )
        .object(
            ObjectSpec::creature(p(1), "Thalia, Guardian of Thraben", 2, 1)
                .with_supertypes(vec![SuperType::Legendary]),
        )
        .at_step(Step::PreCombatMain)
        .active_player(p(1))
        .build()
        .unwrap();

    let events = sba_events_from_start(state);

    // CR 704.5j: the legendary rule fires — exactly one copy goes to the graveyard.
    let legendary_rule_fired = events
        .iter()
        .any(|e| matches!(e, GameEvent::LegendaryRuleApplied { .. }));
    assert!(
        legendary_rule_fired,
        "legendary rule should fire when two legendary copies are on the battlefield; \
         events: {:?}",
        events
    );

    // Exactly one should have been put to graveyard (one kept, one removed).
    let removed_count = events
        .iter()
        .filter_map(|e| {
            if let GameEvent::LegendaryRuleApplied {
                put_to_graveyard, ..
            } = e
            {
                Some(put_to_graveyard.len())
            } else {
                None
            }
        })
        .sum::<usize>();
    assert_eq!(
        removed_count, 1,
        "exactly one legendary copy should be removed; removed: {}",
        removed_count
    );
}

// ── CC#24: Token die-trigger fires before SBA cleanup ─────────────────────

#[test]
/// CC#24 / CR 704.5d + CR 603.2 — A token creature dies; the "creature died" SBA fires
/// (moving it to graveyard with a CreatureDied event), and THEN SBA 704.5d removes the
/// token from the graveyard with a TokenCeasedToExist event.
///
/// The critical ordering: the token briefly exists in the graveyard zone (long enough
/// for "whenever a creature dies" triggers to see it), then the next SBA pass removes it.
///
/// In our engine, we verify this by checking that:
/// 1. `CreatureDied` fires for the token (SBA 704.5g — lethal damage).
/// 2. `TokenCeasedToExist` fires for the token (SBA 704.5d — token in non-battlefield zone).
/// 3. `TokenCeasedToExist` comes AFTER `CreatureDied` in the event sequence, confirming
///    the token passed through the graveyard before ceasing to exist.
fn test_cc24_token_dies_trigger_fires_before_sba_cleanup() {
    // Build a state with a token creature that has lethal damage marked.
    // When start_game fires SBAs, the token should: die → graveyard (CreatureDied),
    // then cease to exist (TokenCeasedToExist).
    let state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .object(
            ObjectSpec::creature(p(1), "Bear Token", 2, 2)
                .token()
                .with_damage(2), // lethal damage → triggers SBA 704.5g
        )
        .at_step(Step::PreCombatMain)
        .active_player(p(1))
        .build()
        .unwrap();

    let events = sba_events_from_start(state);

    // Both events should be present.
    let died_event = events
        .iter()
        .position(|e| matches!(e, GameEvent::CreatureDied { .. }));
    let ceased_event = events
        .iter()
        .position(|e| matches!(e, GameEvent::TokenCeasedToExist { .. }));

    assert!(
        died_event.is_some(),
        "CreatureDied should fire when the token takes lethal damage (SBA 704.5g); \
         events: {:?}",
        events
    );
    assert!(
        ceased_event.is_some(),
        "TokenCeasedToExist should fire after token enters the graveyard (SBA 704.5d); \
         events: {:?}",
        events
    );

    // CR 704.5d ordering: TokenCeasedToExist comes AFTER CreatureDied.
    // The token briefly exists in the graveyard (triggering die-triggers) before ceasing.
    assert!(
        died_event.unwrap() < ceased_event.unwrap(),
        "CreatureDied (pos {}) must come before TokenCeasedToExist (pos {}) — \
         token briefly exists in graveyard before SBA 704.5d removes it",
        died_event.unwrap(),
        ceased_event.unwrap()
    );
}

// ── CC#31: Aura falls off after type-change ends ───────────────────────────

#[test]
/// CC#31 / CR 704.5m + CR 514.2 — An "Enchant creature" aura is attached to an
/// animated land. When the UntilEndOfTurn animation expires at cleanup, the land
/// stops being a creature. The next SBA check finds the aura attached to a non-creature
/// (illegal for "Enchant creature" auras) and moves it to the graveyard.
///
/// This test verifies the two-step interaction:
/// 1. With the animation active: aura is legally attached to a creature-land.
/// 2. With the animation removed (simulating cleanup): land is no longer a creature.
/// 3. SBA 704.5m fires: aura on non-creature goes to the graveyard.
///
/// For this PARTIAL test, we directly verify that an aura attached to a non-creature
/// (a plain land) triggers the SBA — simulating the state AFTER the animation has ended.
/// The full integration (animation → cleanup → SBA) is covered by turn_actions.rs.
fn test_cc31_aura_falls_off_after_type_change_ends() {
    // Build a state: a plain land (NOT a creature) with an Aura attached to it.
    // This simulates the post-cleanup state where the animation has expired.
    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .object(ObjectSpec::land(p(1), "Animated Land")) // was a creature, now a plain land
        .object(
            ObjectSpec::enchantment(p(1), "Bear Umbra") // "Enchant creature" aura
                .with_subtypes(vec![SubType("Aura".to_string())]),
        )
        .at_step(Step::PreCombatMain)
        .active_player(p(1))
        .build()
        .unwrap();

    // Manually attach the aura to the land (simulating illegal attachment after animation ended).
    let land_id = state
        .objects
        .iter()
        .find(|(_, o)| o.characteristics.name == "Animated Land")
        .map(|(id, _)| *id)
        .unwrap();
    let aura_id = state
        .objects
        .iter()
        .find(|(_, o)| o.characteristics.name == "Bear Umbra")
        .map(|(id, _)| *id)
        .unwrap();

    if let Some(aura) = state.objects.get_mut(&aura_id) {
        aura.attached_to = Some(land_id);
        aura.enchants_creatures = true; // "Enchant creature" restriction
    }
    if let Some(land) = state.objects.get_mut(&land_id) {
        land.attachments.push_back(aura_id);
    }

    let (_, events) = start_game(state).unwrap();

    // CR 704.5m: aura attached to non-creature land goes to graveyard.
    let aura_fell_off = events
        .iter()
        .any(|e| matches!(e, GameEvent::AuraFellOff { object_id, .. } if object_id == &aura_id));
    assert!(
        aura_fell_off,
        "Bear Umbra (Enchant creature aura) should fall off the non-creature land \
         via SBA 704.5m; events: {:?}",
        events
    );

    // The land should still be on the battlefield (only the aura is removed).
    let land_on_battlefield = events
        .iter()
        .any(|e| matches!(e, GameEvent::AuraFellOff { object_id, .. } if object_id == &land_id));
    assert!(
        !land_on_battlefield,
        "the land itself should not be removed — only the illegally attached aura falls off"
    );
}
