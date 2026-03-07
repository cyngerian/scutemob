//! Tests for zone integrity: every object in exactly one zone, correct zone
//! types, add/remove operations, and zone creation per player.

use mtg_engine::state::*;

/// Helper: verify that every object in the state is in exactly one zone,
/// and that the zone it reports matches where it's actually stored.
fn assert_zone_integrity(state: &GameState) {
    for (obj_id, obj) in state.objects.iter() {
        // Object's zone field points to a real zone
        let zone = state.zone(&obj.zone).unwrap_or_else(|_| {
            panic!(
                "object {:?} references non-existent zone {:?}",
                obj_id, obj.zone
            )
        });

        // That zone contains this object
        assert!(
            zone.contains(obj_id),
            "object {:?} says it's in zone {:?} but the zone doesn't contain it",
            obj_id,
            obj.zone
        );

        // Object is in exactly one zone (not present in any other zone)
        let mut zone_count = 0;
        for (_zid, z) in state.zones.iter() {
            if z.contains(obj_id) {
                zone_count += 1;
            }
        }
        assert_eq!(
            zone_count, 1,
            "object {:?} found in {} zones, expected 1",
            obj_id, zone_count
        );
    }

    // Converse: every ObjectId in a zone is in the objects map
    for (zone_id, zone) in state.zones.iter() {
        for obj_id in zone.object_ids() {
            assert!(
                state.objects.contains_key(&obj_id),
                "zone {:?} contains {:?} but it's not in the objects map",
                zone_id,
                obj_id
            );
        }
    }
}

#[test]
fn test_zone_integrity_empty_state() {
    let state = GameStateBuilder::four_player().build().unwrap();
    assert_zone_integrity(&state);
}

#[test]
fn test_zone_integrity_with_objects() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let state = GameStateBuilder::four_player()
        .object(ObjectSpec::creature(p1, "Bear", 2, 2))
        .object(ObjectSpec::creature(p2, "Bird", 1, 1))
        .object(ObjectSpec::card(p1, "Lightning Bolt"))
        .object(ObjectSpec::land(p1, "Mountain").in_zone(ZoneId::Library(p1)))
        .build()
        .unwrap();

    assert_zone_integrity(&state);
    assert_eq!(state.total_objects(), 4);
}

#[test]
fn test_zone_type_correctness() {
    let state = GameStateBuilder::four_player().build().unwrap();
    let p1 = PlayerId(1);

    // Ordered zones use Vector
    let library = state.zone(&ZoneId::Library(p1)).unwrap();
    assert!(matches!(library, Zone::Ordered(_)));

    let graveyard = state.zone(&ZoneId::Graveyard(p1)).unwrap();
    assert!(matches!(graveyard, Zone::Ordered(_)));

    let stack = state.zone(&ZoneId::Stack).unwrap();
    assert!(matches!(stack, Zone::Ordered(_)));

    // Unordered zones use OrdSet
    let hand = state.zone(&ZoneId::Hand(p1)).unwrap();
    assert!(matches!(hand, Zone::Unordered(_)));

    let battlefield = state.zone(&ZoneId::Battlefield).unwrap();
    assert!(matches!(battlefield, Zone::Unordered(_)));

    let exile = state.zone(&ZoneId::Exile).unwrap();
    assert!(matches!(exile, Zone::Unordered(_)));

    let command = state.zone(&ZoneId::Command(p1)).unwrap();
    assert!(matches!(command, Zone::Unordered(_)));
}

#[test]
fn test_zone_add_remove() {
    let mut zone = Zone::new_unordered();
    let id1 = ObjectId(1);
    let id2 = ObjectId(2);

    assert!(zone.is_empty());
    assert_eq!(zone.len(), 0);

    zone.insert(id1);
    assert_eq!(zone.len(), 1);
    assert!(zone.contains(&id1));
    assert!(!zone.contains(&id2));

    zone.insert(id2);
    assert_eq!(zone.len(), 2);

    assert!(zone.remove(&id1));
    assert_eq!(zone.len(), 1);
    assert!(!zone.contains(&id1));
    assert!(zone.contains(&id2));

    // Removing non-existent returns false
    assert!(!zone.remove(&id1));
}

#[test]
fn test_ordered_zone_preserves_order() {
    let mut zone = Zone::new_ordered();
    zone.insert(ObjectId(10));
    zone.insert(ObjectId(20));
    zone.insert(ObjectId(30));

    let ids = zone.object_ids();
    assert_eq!(ids, vec![ObjectId(10), ObjectId(20), ObjectId(30)]);

    // Top is the last inserted
    assert_eq!(zone.top(), Some(ObjectId(30)));
}

#[test]
fn test_ordered_zone_insert_at() {
    let mut zone = Zone::new_ordered();
    zone.insert(ObjectId(1));
    zone.insert(ObjectId(3));
    zone.insert_at(1, ObjectId(2)); // Insert in the middle

    let ids = zone.object_ids();
    assert_eq!(ids, vec![ObjectId(1), ObjectId(2), ObjectId(3)]);
}

#[test]
fn test_ordered_zone_remove() {
    let mut zone = Zone::new_ordered();
    zone.insert(ObjectId(10));
    zone.insert(ObjectId(20));
    zone.insert(ObjectId(30));

    zone.remove(&ObjectId(20));
    let ids = zone.object_ids();
    assert_eq!(ids, vec![ObjectId(10), ObjectId(30)]);
}

#[test]
fn test_add_object_to_state() {
    let p1 = PlayerId(1);
    let mut state = GameStateBuilder::four_player().build().unwrap();

    let obj = GameObject {
        id: ObjectId(0),
        card_id: None,
        characteristics: Characteristics::default(),
        controller: p1,
        owner: p1,
        zone: ZoneId::Battlefield,
        status: ObjectStatus::default(),
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
    };

    let id = state.add_object(obj, ZoneId::Battlefield).unwrap();
    assert_zone_integrity(&state);
    assert_eq!(state.total_objects(), 1);
    assert!(state.zone(&ZoneId::Battlefield).unwrap().contains(&id));
    assert_eq!(state.object(id).unwrap().zone, ZoneId::Battlefield);
}

#[test]
fn test_objects_in_zone_query() {
    let p1 = PlayerId(1);
    let state = GameStateBuilder::four_player()
        .object(ObjectSpec::creature(p1, "Bear", 2, 2))
        .object(ObjectSpec::creature(p1, "Bird", 1, 1))
        .object(ObjectSpec::card(p1, "Bolt").in_zone(ZoneId::Hand(p1)))
        .build()
        .unwrap();

    let battlefield_objects = state.objects_in_zone(&ZoneId::Battlefield);
    assert_eq!(battlefield_objects.len(), 2);

    let hand_objects = state.objects_in_zone(&ZoneId::Hand(p1));
    assert_eq!(hand_objects.len(), 1);
    assert_eq!(hand_objects[0].characteristics.name, "Bolt");
}

#[test]
fn test_zone_id_properties() {
    let p1 = PlayerId(1);

    // Zone type
    assert_eq!(ZoneId::Library(p1).zone_type(), ZoneType::Library);
    assert_eq!(ZoneId::Battlefield.zone_type(), ZoneType::Battlefield);
    assert_eq!(ZoneId::Stack.zone_type(), ZoneType::Stack);

    // Owner
    assert_eq!(ZoneId::Library(p1).owner(), Some(p1));
    assert_eq!(ZoneId::Battlefield.owner(), None);
    assert_eq!(ZoneId::Stack.owner(), None);
    assert_eq!(ZoneId::Exile.owner(), None);

    // Ordered
    assert!(ZoneId::Library(p1).is_ordered());
    assert!(ZoneId::Graveyard(p1).is_ordered());
    assert!(ZoneId::Stack.is_ordered());
    assert!(!ZoneId::Hand(p1).is_ordered());
    assert!(!ZoneId::Battlefield.is_ordered());
    assert!(!ZoneId::Exile.is_ordered());
    assert!(!ZoneId::Command(p1).is_ordered());
}

#[test]
fn test_zone_shuffle() {
    use rand::SeedableRng;

    let mut zone = Zone::new_ordered();
    for i in 0..20 {
        zone.insert(ObjectId(i));
    }

    let original_order = zone.object_ids();

    let mut rng = rand::rngs::StdRng::seed_from_u64(42);
    zone.shuffle(&mut rng);

    let shuffled_order = zone.object_ids();
    assert_eq!(shuffled_order.len(), 20);
    // Very unlikely to still be in order after shuffle
    assert_ne!(original_order, shuffled_order);

    // Same seed produces same shuffle (deterministic)
    let mut zone2 = Zone::new_ordered();
    for i in 0..20 {
        zone2.insert(ObjectId(i));
    }
    let mut rng2 = rand::rngs::StdRng::seed_from_u64(42);
    zone2.shuffle(&mut rng2);
    assert_eq!(zone2.object_ids(), shuffled_order);
}
