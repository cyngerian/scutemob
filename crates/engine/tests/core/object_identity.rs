//! Tests for object identity rules per CR 400.7.
//!
//! CR 400.7: "An object that moves from one zone to another becomes a new
//! object with no memory of, or relation to, its previous existence."

use mtg_engine::state::test_util;
use mtg_engine::state::*;

#[test]
/// CR 400.7 — zone change produces a new ObjectId
fn test_400_7_zone_change_new_object_id() {
    let p1 = PlayerId(1);
    let mut state = GameStateBuilder::four_player()
        .object(ObjectSpec::creature(p1, "Grizzly Bears", 2, 2))
        .build()
        .unwrap();

    // Find the bear on the battlefield
    let battlefield_objs = state.objects_in_zone(&ZoneId::Battlefield);
    assert_eq!(battlefield_objs.len(), 1);
    let old_id = battlefield_objs[0].id;

    // Move to graveyard (creature dies)
    let (new_id, _old_snapshot) =
        test_util::move_object_to_zone(&mut state, old_id, ZoneId::Graveyard(p1)).unwrap();

    // New ObjectId is different from old
    assert_ne!(old_id, new_id);

    // Old ID no longer exists in the state
    assert!(state.object(old_id).is_err());

    // New ID exists and is in the graveyard
    let new_obj = state.object(new_id).unwrap();
    assert_eq!(new_obj.zone, ZoneId::Graveyard(p1));
}

#[test]
/// CR 400.7 — old snapshot is preserved for trigger processing
fn test_400_7_old_snapshot_preserved() {
    let p1 = PlayerId(1);
    let mut state = GameStateBuilder::four_player()
        .object(
            ObjectSpec::creature(p1, "Savannah Lions", 2, 1)
                .controlled_by(PlayerId(2))
                .tapped()
                .with_counter(CounterType::PlusOnePlusOne, 3),
        )
        .build()
        .unwrap();

    let battlefield_objs = state.objects_in_zone(&ZoneId::Battlefield);
    let old_id = battlefield_objs[0].id;

    let (_new_id, old_snapshot) =
        test_util::move_object_to_zone(&mut state, old_id, ZoneId::Graveyard(p1)).unwrap();

    // Old snapshot preserves original state
    assert_eq!(old_snapshot.id, old_id);
    assert_eq!(old_snapshot.characteristics.name, "Savannah Lions");
    assert_eq!(old_snapshot.controller, PlayerId(2)); // Was being controlled by p2
    assert!(old_snapshot.status.tapped);
    assert_eq!(
        old_snapshot.counters.get(&CounterType::PlusOnePlusOne),
        Some(&3)
    );
    assert_eq!(old_snapshot.zone, ZoneId::Battlefield);
}

#[test]
/// CR 400.7 — characteristics preserved across zone change
fn test_400_7_characteristics_preserved() {
    let p1 = PlayerId(1);
    let mut state = GameStateBuilder::four_player()
        .object(
            ObjectSpec::creature(p1, "Tarmogoyf", 0, 1)
                .with_colors(vec![Color::Green])
                .with_subtypes(vec![SubType("Lhurgoyf".to_string())])
                .with_mana_cost(ManaCost {
                    generic: 1,
                    green: 1,
                    ..ManaCost::default()
                }),
        )
        .build()
        .unwrap();

    let battlefield_objs = state.objects_in_zone(&ZoneId::Battlefield);
    let old_id = battlefield_objs[0].id;

    let (new_id, _) =
        test_util::move_object_to_zone(&mut state, old_id, ZoneId::Graveyard(p1)).unwrap();

    let new_obj = state.object(new_id).unwrap();
    assert_eq!(new_obj.characteristics.name, "Tarmogoyf");
    assert!(new_obj.characteristics.colors.contains(&Color::Green));
    assert!(new_obj
        .characteristics
        .subtypes
        .contains(&SubType("Lhurgoyf".to_string())));
    assert_eq!(
        new_obj
            .characteristics
            .mana_cost
            .as_ref()
            .unwrap()
            .mana_value(),
        2
    );
    assert_eq!(new_obj.characteristics.power, Some(0));
    assert_eq!(new_obj.characteristics.toughness, Some(1));
}

#[test]
/// CR 400.7 — status, counters, and attachments reset on zone change
fn test_400_7_status_counters_attachments_reset() {
    let p1 = PlayerId(1);
    let mut state = GameStateBuilder::four_player()
        .object(
            ObjectSpec::creature(p1, "Tapped Bear", 2, 2)
                .tapped()
                .with_counter(CounterType::PlusOnePlusOne, 5),
        )
        .build()
        .unwrap();

    let old_id = state.objects_in_zone(&ZoneId::Battlefield)[0].id;

    // Verify pre-move state
    let old_obj = state.object(old_id).unwrap();
    assert!(old_obj.status.tapped);
    assert_eq!(old_obj.counters.get(&CounterType::PlusOnePlusOne), Some(&5));

    let (new_id, _) =
        test_util::move_object_to_zone(&mut state, old_id, ZoneId::Graveyard(p1)).unwrap();

    let new_obj = state.object(new_id).unwrap();
    // Status reset
    assert!(!new_obj.status.tapped);
    assert!(!new_obj.status.flipped);
    assert!(!new_obj.status.face_down);
    assert!(!new_obj.status.phased_out);
    // Counters reset
    assert!(new_obj.counters.is_empty());
    // Attachments reset
    assert!(new_obj.attachments.is_empty());
    assert!(new_obj.attached_to.is_none());
    // Damage reset
    assert_eq!(new_obj.damage_marked, 0);
}

#[test]
/// CR 400.7 — controller resets to owner on zone change
fn test_400_7_controller_resets_to_owner() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let mut state = GameStateBuilder::four_player()
        .object(ObjectSpec::creature(p1, "Stolen Bear", 2, 2).controlled_by(p2))
        .build()
        .unwrap();

    let old_id = state.objects_in_zone(&ZoneId::Battlefield)[0].id;

    // Verify controller is different from owner
    let old_obj = state.object(old_id).unwrap();
    assert_eq!(old_obj.owner, p1);
    assert_eq!(old_obj.controller, p2);

    let (new_id, _) =
        test_util::move_object_to_zone(&mut state, old_id, ZoneId::Graveyard(p1)).unwrap();

    let new_obj = state.object(new_id).unwrap();
    assert_eq!(new_obj.owner, p1);
    assert_eq!(new_obj.controller, p1, "controller should reset to owner");
}

#[test]
/// CR 400.7 — card_id persists across zone changes (physical card identity)
fn test_400_7_card_id_persists() {
    let p1 = PlayerId(1);
    let card_id = CardId("sol-ring-oracle-id".to_string());

    let mut state = GameStateBuilder::four_player()
        .object(ObjectSpec::artifact(p1, "Sol Ring").with_card_id(card_id.clone()))
        .build()
        .unwrap();

    let old_id = state.objects_in_zone(&ZoneId::Battlefield)[0].id;

    let (new_id, _) =
        test_util::move_object_to_zone(&mut state, old_id, ZoneId::Graveyard(p1)).unwrap();

    let new_obj = state.object(new_id).unwrap();
    assert_eq!(
        new_obj.card_id,
        Some(card_id),
        "card_id must persist across zone changes for commander tracking"
    );
}

#[test]
/// CR 400.7 — token status preserved across zone change
fn test_400_7_token_status_preserved() {
    let p1 = PlayerId(1);
    let mut state = GameStateBuilder::four_player()
        .object(ObjectSpec::creature(p1, "Saproling", 1, 1).token())
        .build()
        .unwrap();

    let old_id = state.objects_in_zone(&ZoneId::Battlefield)[0].id;

    let (new_id, _) =
        test_util::move_object_to_zone(&mut state, old_id, ZoneId::Graveyard(p1)).unwrap();

    let new_obj = state.object(new_id).unwrap();
    assert!(new_obj.is_token, "token status should persist");
}

#[test]
/// CR 400.7 — new object gets a fresh timestamp
fn test_400_7_new_timestamp() {
    let p1 = PlayerId(1);
    let mut state = GameStateBuilder::four_player()
        .object(ObjectSpec::creature(p1, "Bear", 2, 2))
        .build()
        .unwrap();

    let old_id = state.objects_in_zone(&ZoneId::Battlefield)[0].id;
    let old_timestamp = state.object(old_id).unwrap().timestamp;

    let (new_id, _) =
        test_util::move_object_to_zone(&mut state, old_id, ZoneId::Graveyard(p1)).unwrap();

    let new_obj = state.object(new_id).unwrap();
    assert!(
        new_obj.timestamp > old_timestamp,
        "new object should have a newer timestamp"
    );
}

#[test]
/// Move an object that doesn't exist returns error
fn test_move_nonexistent_object_errors() {
    let mut state = GameStateBuilder::four_player().build().unwrap();
    let result =
        test_util::move_object_to_zone(&mut state, ObjectId(999), ZoneId::Graveyard(PlayerId(1)));
    assert!(result.is_err());
}

#[test]
/// MR-M1-19 / CR 400.7 — a same-zone move (battlefield → battlefield) **keeps the
/// same `ObjectId`**, because no zone change happened.
///
/// # This test is INVERTED by PB-DX15a (`scutemob-216`, closes `OOS-DP9-11`)
///
/// It used to be `test_400_7_same_zone_move_produces_new_id`, asserting
/// `assert_ne!(old_id, new_id)` on the reasoning that *"the zone-change event creates a
/// new object regardless of the source and destination zones being the same."*
///
/// **That reasoning inverts the rule it cites.** CR 400.7 reads, in full: *"An object
/// that moves from one zone to another becomes a new object with no memory of, or
/// relation to, its previous existence."* The antecedent is a move **from one zone to
/// another**. An object whose destination is the zone it is already in has not moved
/// from one zone to another, so the rule does not fire and the object is the same
/// object. The old assertion was a pin on a primitive's behaviour dressed as a rules
/// claim, and it was the reason `OOS-DP9-11` stayed open: a fix at the helper level
/// reddened it, so every earlier reader concluded the helper was right.
///
/// Its consequences were real, not theoretical. `next_object_id()` is
/// `timestamp_counter += 1`, and `timestamp_counter` is the seed source for every
/// `Zone::shuffle` and coin flip (`rules/commander.rs:832-835`), so a card that never
/// left its library both lost its identity and moved the game's PRNG. Seventeen
/// deck-legal `Complete` defs reached that path — see
/// `crates/engine/tests/core/pb_dx15a_same_zone_identity_roster.rs`, which prints the
/// population rather than transcribing it.
///
/// The battlefield is an **unordered** zone (`Zone::new_unordered()`,
/// `state/builder.rs:302`), so there is no order to permute either: the correct
/// behaviour here is a total no-op, which is what this now asserts.
fn test_400_7_same_zone_move_keeps_the_same_id() {
    let p1 = PlayerId(1);
    let mut state = GameStateBuilder::four_player()
        .object(ObjectSpec::creature(p1, "Blinking Bear", 2, 2))
        .build()
        .unwrap();

    let old_id = state.objects_in_zone(&ZoneId::Battlefield)[0].id;
    let counter_before = state.current_timestamp();

    // "Move" to the zone it is already in. At the primitive level this is a reposition,
    // not a zone change.
    let (new_id, _old_snapshot) =
        test_util::move_object_to_zone(&mut state, old_id, ZoneId::Battlefield).unwrap();

    // CR 400.7: no move from one zone to another, so no new object.
    assert_eq!(
        old_id, new_id,
        "a same-zone move must NOT produce a new ObjectId -- CR 400.7's antecedent is \
         'moves from one zone to another', and this object did not"
    );

    // The original id is still live.
    assert!(
        state.object(old_id).is_ok(),
        "the original ObjectId must still resolve after a same-zone move"
    );

    // And no `timestamp_counter` value was burned. This is the half that matters beyond
    // identity: the counter seeds `Zone::shuffle` and every coin flip, so a same-zone
    // move used to perturb the game's PRNG for free.
    assert_eq!(
        state.current_timestamp(),
        counter_before,
        "a same-zone move must consume NO timestamp_counter value -- it is the shuffle \
         and coin-flip seed source"
    );

    // Only one object on the battlefield still.
    assert_eq!(
        state.objects_in_zone(&ZoneId::Battlefield).len(),
        1,
        "battlefield should still have exactly 1 object after a same-zone move"
    );
}

#[test]
/// MR-M1-20 / CR 400.7 — moving a valid object to a non-existent player zone
/// returns an error. The object must not be modified or destroyed.
fn test_move_to_invalid_zone_returns_error() {
    let p1 = PlayerId(1);
    let mut state = GameStateBuilder::four_player()
        .object(ObjectSpec::creature(p1, "Stubborn Bear", 2, 2))
        .build()
        .unwrap();

    let bear_id = state.objects_in_zone(&ZoneId::Battlefield)[0].id;

    // PlayerId(999) does not exist in the state.
    let result =
        test_util::move_object_to_zone(&mut state, bear_id, ZoneId::Graveyard(PlayerId(999)));
    assert!(
        result.is_err(),
        "moving to a non-existent player's graveyard should return an error"
    );

    // The object must still exist on the battlefield, unmodified.
    assert!(
        state.object(bear_id).is_ok(),
        "object should still exist after failed zone move"
    );
    assert_eq!(
        state.object(bear_id).unwrap().zone,
        ZoneId::Battlefield,
        "object should still be on the battlefield after failed zone move"
    );
}

#[test]
/// Multiple zone transitions produce unique ObjectIds each time
fn test_multiple_zone_transitions() {
    let p1 = PlayerId(1);
    let mut state = GameStateBuilder::four_player()
        .object(ObjectSpec::creature(p1, "Phoenix", 2, 2))
        .build()
        .unwrap();

    let id1 = state.objects_in_zone(&ZoneId::Battlefield)[0].id;

    // Battlefield → Graveyard
    let (id2, _) = test_util::move_object_to_zone(&mut state, id1, ZoneId::Graveyard(p1)).unwrap();
    assert_ne!(id1, id2);

    // Graveyard → Exile
    let (id3, _) = test_util::move_object_to_zone(&mut state, id2, ZoneId::Exile).unwrap();
    assert_ne!(id2, id3);
    assert_ne!(id1, id3);

    // Only the latest object exists
    assert!(state.object(id1).is_err());
    assert!(state.object(id2).is_err());
    assert!(state.object(id3).is_ok());
    assert_eq!(state.object(id3).unwrap().zone, ZoneId::Exile);
    assert_eq!(state.total_objects(), 1);
}
