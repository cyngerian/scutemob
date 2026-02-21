//! Tests for the GameStateBuilder and ObjectSpec fluent API.

use mtg_engine::state::*;

#[test]
fn test_builder_creature_on_battlefield() {
    let p1 = PlayerId(1);
    let state = GameStateBuilder::four_player()
        .object(ObjectSpec::creature(p1, "Grizzly Bears", 2, 2))
        .build();

    let objs = state.objects_in_zone(&ZoneId::Battlefield);
    assert_eq!(objs.len(), 1);
    assert_eq!(objs[0].characteristics.name, "Grizzly Bears");
    assert_eq!(objs[0].characteristics.power, Some(2));
    assert_eq!(objs[0].characteristics.toughness, Some(2));
    assert!(objs[0]
        .characteristics
        .card_types
        .contains(&CardType::Creature));
    assert_eq!(objs[0].owner, p1);
    assert_eq!(objs[0].controller, p1);
}

#[test]
fn test_builder_artifact_on_battlefield() {
    let p1 = PlayerId(1);
    let state = GameStateBuilder::four_player()
        .object(ObjectSpec::artifact(p1, "Sol Ring"))
        .build();

    let objs = state.objects_in_zone(&ZoneId::Battlefield);
    assert_eq!(objs.len(), 1);
    assert!(objs[0]
        .characteristics
        .card_types
        .contains(&CardType::Artifact));
    assert_eq!(objs[0].characteristics.power, None);
    assert_eq!(objs[0].characteristics.toughness, None);
}

#[test]
fn test_builder_land_on_battlefield() {
    let p1 = PlayerId(1);
    let state = GameStateBuilder::four_player()
        .object(ObjectSpec::land(p1, "Forest"))
        .build();

    let objs = state.objects_in_zone(&ZoneId::Battlefield);
    assert_eq!(objs.len(), 1);
    assert!(objs[0].characteristics.card_types.contains(&CardType::Land));
}

#[test]
fn test_builder_enchantment_on_battlefield() {
    let p1 = PlayerId(1);
    let state = GameStateBuilder::four_player()
        .object(ObjectSpec::enchantment(p1, "Rhystic Study"))
        .build();

    let objs = state.objects_in_zone(&ZoneId::Battlefield);
    assert_eq!(objs.len(), 1);
    assert!(objs[0]
        .characteristics
        .card_types
        .contains(&CardType::Enchantment));
}

#[test]
fn test_builder_planeswalker_on_battlefield() {
    let p1 = PlayerId(1);
    let state = GameStateBuilder::four_player()
        .object(ObjectSpec::planeswalker(p1, "Jace, the Mind Sculptor", 3))
        .build();

    let objs = state.objects_in_zone(&ZoneId::Battlefield);
    assert_eq!(objs.len(), 1);
    assert!(objs[0]
        .characteristics
        .card_types
        .contains(&CardType::Planeswalker));
    assert_eq!(objs[0].characteristics.loyalty, Some(3));
}

#[test]
fn test_builder_card_in_hand() {
    let p1 = PlayerId(1);
    let state = GameStateBuilder::four_player()
        .object(ObjectSpec::card(p1, "Lightning Bolt"))
        .build();

    let hand_objs = state.objects_in_zone(&ZoneId::Hand(p1));
    assert_eq!(hand_objs.len(), 1);
    assert_eq!(hand_objs[0].characteristics.name, "Lightning Bolt");
    assert_eq!(hand_objs[0].zone, ZoneId::Hand(p1));
}

#[test]
fn test_builder_card_in_library() {
    let p1 = PlayerId(1);
    let state = GameStateBuilder::four_player()
        .object(ObjectSpec::card(p1, "Island").in_zone(ZoneId::Library(p1)))
        .build();

    let library_objs = state.objects_in_zone(&ZoneId::Library(p1));
    assert_eq!(library_objs.len(), 1);
}

#[test]
fn test_builder_card_in_exile() {
    let p1 = PlayerId(1);
    let state = GameStateBuilder::four_player()
        .object(ObjectSpec::card(p1, "Exiled Card").in_zone(ZoneId::Exile))
        .build();

    let exile_objs = state.objects_in_zone(&ZoneId::Exile);
    assert_eq!(exile_objs.len(), 1);
}

#[test]
fn test_builder_card_in_command_zone() {
    let p1 = PlayerId(1);
    let state = GameStateBuilder::four_player()
        .object(
            ObjectSpec::creature(p1, "Atraxa, Praetors' Voice", 4, 4)
                .in_zone(ZoneId::Command(p1))
                .with_supertypes(vec![SuperType::Legendary]),
        )
        .build();

    let cmd_objs = state.objects_in_zone(&ZoneId::Command(p1));
    assert_eq!(cmd_objs.len(), 1);
    assert!(cmd_objs[0]
        .characteristics
        .supertypes
        .contains(&SuperType::Legendary));
}

#[test]
fn test_builder_card_in_graveyard() {
    let p1 = PlayerId(1);
    let state = GameStateBuilder::four_player()
        .object(ObjectSpec::card(p1, "Dead Card").in_zone(ZoneId::Graveyard(p1)))
        .build();

    let gy_objs = state.objects_in_zone(&ZoneId::Graveyard(p1));
    assert_eq!(gy_objs.len(), 1);
}

#[test]
fn test_builder_tapped_permanent() {
    let p1 = PlayerId(1);
    let state = GameStateBuilder::four_player()
        .object(ObjectSpec::land(p1, "Tapped Land").tapped())
        .build();

    let objs = state.objects_in_zone(&ZoneId::Battlefield);
    assert!(objs[0].status.tapped);
}

#[test]
fn test_builder_permanent_with_counters() {
    let p1 = PlayerId(1);
    let state = GameStateBuilder::four_player()
        .object(
            ObjectSpec::creature(p1, "Hydra", 0, 0)
                .with_counter(CounterType::PlusOnePlusOne, 5)
                .with_counter(CounterType::Shield, 2),
        )
        .build();

    let objs = state.objects_in_zone(&ZoneId::Battlefield);
    assert_eq!(objs[0].counters.get(&CounterType::PlusOnePlusOne), Some(&5));
    assert_eq!(objs[0].counters.get(&CounterType::Shield), Some(&2));
}

#[test]
fn test_builder_controller_different_from_owner() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let state = GameStateBuilder::four_player()
        .object(ObjectSpec::creature(p1, "Stolen Creature", 3, 3).controlled_by(p2))
        .build();

    let objs = state.objects_in_zone(&ZoneId::Battlefield);
    assert_eq!(objs[0].owner, p1);
    assert_eq!(objs[0].controller, p2);
}

#[test]
fn test_builder_token() {
    let p1 = PlayerId(1);
    let state = GameStateBuilder::four_player()
        .object(ObjectSpec::creature(p1, "Soldier", 1, 1).token())
        .build();

    let objs = state.objects_in_zone(&ZoneId::Battlefield);
    assert!(objs[0].is_token);
}

#[test]
fn test_builder_with_card_id() {
    let p1 = PlayerId(1);
    let card_id = CardId("sol-ring-uuid".to_string());
    let state = GameStateBuilder::four_player()
        .object(ObjectSpec::artifact(p1, "Sol Ring").with_card_id(card_id.clone()))
        .build();

    let objs = state.objects_in_zone(&ZoneId::Battlefield);
    assert_eq!(objs[0].card_id, Some(card_id));
}

#[test]
fn test_builder_with_colors() {
    let p1 = PlayerId(1);
    let state = GameStateBuilder::four_player()
        .object(
            ObjectSpec::creature(p1, "Niv-Mizzet", 5, 5).with_colors(vec![Color::Blue, Color::Red]),
        )
        .build();

    let objs = state.objects_in_zone(&ZoneId::Battlefield);
    assert!(objs[0].characteristics.colors.contains(&Color::Blue));
    assert!(objs[0].characteristics.colors.contains(&Color::Red));
    assert!(!objs[0].characteristics.colors.contains(&Color::Green));
}

#[test]
fn test_builder_with_subtypes() {
    let p1 = PlayerId(1);
    let state = GameStateBuilder::four_player()
        .object(
            ObjectSpec::creature(p1, "Ragavan", 2, 1).with_subtypes(vec![
                SubType("Monkey".to_string()),
                SubType("Pirate".to_string()),
            ]),
        )
        .build();

    let objs = state.objects_in_zone(&ZoneId::Battlefield);
    assert!(objs[0]
        .characteristics
        .subtypes
        .contains(&SubType("Monkey".to_string())));
    assert!(objs[0]
        .characteristics
        .subtypes
        .contains(&SubType("Pirate".to_string())));
}

#[test]
fn test_builder_with_mana_cost() {
    let p1 = PlayerId(1);
    let state = GameStateBuilder::four_player()
        .object(
            ObjectSpec::creature(p1, "Wrath of God", 2, 2).with_mana_cost(ManaCost {
                generic: 2,
                white: 2,
                ..ManaCost::default()
            }),
        )
        .build();

    let objs = state.objects_in_zone(&ZoneId::Battlefield);
    let cost = objs[0].characteristics.mana_cost.as_ref().unwrap();
    assert_eq!(cost.mana_value(), 4);
    assert_eq!(cost.white, 2);
    assert_eq!(cost.generic, 2);
}

#[test]
fn test_builder_multiple_objects_multiple_zones() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let state = GameStateBuilder::four_player()
        // P1's board
        .object(ObjectSpec::creature(p1, "Bear", 2, 2))
        .object(ObjectSpec::land(p1, "Forest"))
        .object(ObjectSpec::land(p1, "Forest 2").tapped())
        // P1's hand
        .object(ObjectSpec::card(p1, "Giant Growth"))
        .object(ObjectSpec::card(p1, "Lightning Bolt"))
        // P2's board
        .object(ObjectSpec::creature(p2, "Bird", 1, 1))
        // P1's graveyard
        .object(ObjectSpec::card(p1, "Dead Spell").in_zone(ZoneId::Graveyard(p1)))
        // Exile
        .object(ObjectSpec::card(p1, "Exiled Card").in_zone(ZoneId::Exile))
        .build();

    assert_eq!(state.total_objects(), 8);
    assert_eq!(state.objects_in_zone(&ZoneId::Battlefield).len(), 4);
    assert_eq!(state.objects_in_zone(&ZoneId::Hand(p1)).len(), 2);
    assert_eq!(state.objects_in_zone(&ZoneId::Graveyard(p1)).len(), 1);
    assert_eq!(state.objects_in_zone(&ZoneId::Exile).len(), 1);
}

#[test]
fn test_builder_player_with_fluent_config() {
    let state = GameStateBuilder::new()
        .add_player_with(PlayerId(1), |p| {
            p.life(30)
                .poison(3)
                .land_plays(2)
                .commander(CardId("cmd-1".to_string()))
        })
        .add_player(PlayerId(2))
        .build();

    let p1 = state.player(PlayerId(1)).unwrap();
    assert_eq!(p1.life_total, 30);
    assert_eq!(p1.poison_counters, 3);
    assert_eq!(p1.land_plays_remaining, 2);
    assert_eq!(p1.commander_ids.len(), 1);

    let p2 = state.player(PlayerId(2)).unwrap();
    assert_eq!(p2.life_total, 40);
}

#[test]
fn test_builder_unique_object_ids() {
    let p1 = PlayerId(1);
    let state = GameStateBuilder::four_player()
        .object(ObjectSpec::creature(p1, "A", 1, 1))
        .object(ObjectSpec::creature(p1, "B", 2, 2))
        .object(ObjectSpec::creature(p1, "C", 3, 3))
        .build();

    let ids: Vec<ObjectId> = state.objects.keys().cloned().collect();
    // All IDs are unique
    for i in 0..ids.len() {
        for j in (i + 1)..ids.len() {
            assert_ne!(ids[i], ids[j]);
        }
    }
}

#[test]
#[should_panic(expected = "must have at least one player")]
fn test_builder_panics_no_players() {
    GameStateBuilder::new().build();
}

#[test]
fn test_step_phase_mapping() {
    assert_eq!(Step::Untap.phase(), Phase::Beginning);
    assert_eq!(Step::Upkeep.phase(), Phase::Beginning);
    assert_eq!(Step::Draw.phase(), Phase::Beginning);
    assert_eq!(Step::PreCombatMain.phase(), Phase::PreCombatMain);
    assert_eq!(Step::BeginningOfCombat.phase(), Phase::Combat);
    assert_eq!(Step::DeclareAttackers.phase(), Phase::Combat);
    assert_eq!(Step::DeclareBlockers.phase(), Phase::Combat);
    assert_eq!(Step::CombatDamage.phase(), Phase::Combat);
    assert_eq!(Step::FirstStrikeDamage.phase(), Phase::Combat);
    assert_eq!(Step::EndOfCombat.phase(), Phase::Combat);
    assert_eq!(Step::PostCombatMain.phase(), Phase::PostCombatMain);
    assert_eq!(Step::End.phase(), Phase::Ending);
    assert_eq!(Step::Cleanup.phase(), Phase::Ending);
}

#[test]
fn test_step_priority() {
    // Untap and Cleanup normally have no priority
    assert!(!Step::Untap.has_priority());
    assert!(!Step::Cleanup.has_priority());

    // All other steps have priority
    assert!(Step::Upkeep.has_priority());
    assert!(Step::Draw.has_priority());
    assert!(Step::PreCombatMain.has_priority());
    assert!(Step::BeginningOfCombat.has_priority());
    assert!(Step::DeclareAttackers.has_priority());
    assert!(Step::DeclareBlockers.has_priority());
    assert!(Step::CombatDamage.has_priority());
    assert!(Step::FirstStrikeDamage.has_priority());
    assert!(Step::EndOfCombat.has_priority());
    assert!(Step::PostCombatMain.has_priority());
    assert!(Step::End.has_priority());
}
