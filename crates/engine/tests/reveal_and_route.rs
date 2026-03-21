/// Tests for Effect::RevealAndRoute (CR 701.16a) and Effect::Flicker (CR 400.7).
///
/// RevealAndRoute reveals the top N cards of a player's library, then routes
/// matching cards to one zone and non-matching cards to another.
///
/// Flicker exiles a target permanent and returns it to the battlefield under
/// its owner's control as a new object.
use mtg_engine::cards::card_definition::{EffectTarget, TargetFilter, ZoneTarget};
use mtg_engine::effects::{execute_effect, EffectContext};
use mtg_engine::rules::events::GameEvent;
use mtg_engine::state::game_object::ObjectId;
use mtg_engine::state::targeting::{SpellTarget, Target};
use mtg_engine::state::types::CardType;
use mtg_engine::state::types::SubType;
use mtg_engine::state::zone::ZoneId;
use mtg_engine::state::{GameStateBuilder, ObjectSpec, PlayerId};
use mtg_engine::{Effect, EffectAmount, LibraryPosition, PlayerTarget};

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

fn ec(controller: PlayerId, source: ObjectId) -> EffectContext {
    EffectContext::new(controller, source, vec![])
}

fn ec_with_target(controller: PlayerId, source: ObjectId, target_id: ObjectId) -> EffectContext {
    EffectContext::new(
        controller,
        source,
        vec![SpellTarget {
            target: Target::Object(target_id),
            zone_at_cast: Some(ZoneId::Battlefield),
        }],
    )
}

// ── RevealAndRoute Tests ─────────────────────────────────────────────────────

#[test]
/// CR 701.16a — all cards match the filter: all go to matched_dest.
/// Source: Goblin Ringleader pattern where library top 4 are all Goblins.
fn test_reveal_and_route_all_match() {
    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .object(
            ObjectSpec::card(p(1), "Goblin A")
                .in_zone(ZoneId::Library(p(1)))
                .with_subtypes(vec![SubType("Goblin".to_string())])
                .with_types(vec![CardType::Creature]),
        )
        .object(
            ObjectSpec::card(p(1), "Goblin B")
                .in_zone(ZoneId::Library(p(1)))
                .with_subtypes(vec![SubType("Goblin".to_string())])
                .with_types(vec![CardType::Creature]),
        )
        .build()
        .unwrap();

    let source_id = ObjectId(999);
    let mut ctx = ec(p(1), source_id);

    let effect = Effect::RevealAndRoute {
        player: PlayerTarget::Controller,
        count: EffectAmount::Fixed(4),
        filter: TargetFilter {
            has_subtype: Some(SubType("Goblin".to_string())),
            ..Default::default()
        },
        matched_dest: ZoneTarget::Hand {
            owner: PlayerTarget::Controller,
        },
        unmatched_dest: ZoneTarget::Library {
            owner: PlayerTarget::Controller,
            position: LibraryPosition::Bottom,
        },
    };

    let events = execute_effect(&mut state, &effect, &mut ctx);

    // Both cards should be in hand now.
    let hand_count = state
        .objects
        .values()
        .filter(|o| o.zone == ZoneId::Hand(p(1)))
        .count();
    assert_eq!(hand_count, 2, "all Goblins should be routed to hand");

    // Library should be empty (only had 2 cards, both matched).
    let lib_count = state
        .zones
        .get(&ZoneId::Library(p(1)))
        .map(|z| z.len())
        .unwrap_or(0);
    assert_eq!(lib_count, 0, "library should be empty");

    // Should have ObjectReturnedToHand events.
    let hand_events = events
        .iter()
        .filter(|e| matches!(e, GameEvent::ObjectReturnedToHand { .. }))
        .count();
    assert_eq!(hand_events, 2, "should emit 2 ObjectReturnedToHand events");
}

#[test]
/// CR 701.16a — no cards match the filter: all go to unmatched_dest.
/// Source: Goblin Ringleader pattern where library top has no Goblins.
fn test_reveal_and_route_none_match() {
    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .object(
            ObjectSpec::card(p(1), "Elf A")
                .in_zone(ZoneId::Library(p(1)))
                .with_subtypes(vec![SubType("Elf".to_string())])
                .with_types(vec![CardType::Creature]),
        )
        .object(
            ObjectSpec::card(p(1), "Elf B")
                .in_zone(ZoneId::Library(p(1)))
                .with_subtypes(vec![SubType("Elf".to_string())])
                .with_types(vec![CardType::Creature]),
        )
        .build()
        .unwrap();

    let source_id = ObjectId(999);
    let mut ctx = ec(p(1), source_id);

    let effect = Effect::RevealAndRoute {
        player: PlayerTarget::Controller,
        count: EffectAmount::Fixed(4),
        filter: TargetFilter {
            has_subtype: Some(SubType("Goblin".to_string())),
            ..Default::default()
        },
        matched_dest: ZoneTarget::Hand {
            owner: PlayerTarget::Controller,
        },
        unmatched_dest: ZoneTarget::Library {
            owner: PlayerTarget::Controller,
            position: LibraryPosition::Bottom,
        },
    };

    let events = execute_effect(&mut state, &effect, &mut ctx);

    // Both cards should remain in library (moved to bottom).
    let lib_count = state
        .zones
        .get(&ZoneId::Library(p(1)))
        .map(|z| z.len())
        .unwrap_or(0);
    assert_eq!(lib_count, 2, "non-matching cards should stay in library");

    // Hand should be empty.
    let hand_count = state
        .objects
        .values()
        .filter(|o| o.zone == ZoneId::Hand(p(1)))
        .count();
    assert_eq!(hand_count, 0, "no cards should be in hand");

    // Should have ObjectPutOnLibrary events.
    let lib_events = events
        .iter()
        .filter(|e| matches!(e, GameEvent::ObjectPutOnLibrary { .. }))
        .count();
    assert_eq!(lib_events, 2, "should emit 2 ObjectPutOnLibrary events");
}

#[test]
/// CR 701.16a — partial match: matching cards go to one zone, rest to another.
/// Source: Goblin Ringleader ETB with mixed library top.
fn test_reveal_and_route_partial_match() {
    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .object(
            ObjectSpec::card(p(1), "Goblin A")
                .in_zone(ZoneId::Library(p(1)))
                .with_subtypes(vec![SubType("Goblin".to_string())])
                .with_types(vec![CardType::Creature]),
        )
        .object(
            ObjectSpec::card(p(1), "Elf A")
                .in_zone(ZoneId::Library(p(1)))
                .with_subtypes(vec![SubType("Elf".to_string())])
                .with_types(vec![CardType::Creature]),
        )
        .object(
            ObjectSpec::card(p(1), "Goblin B")
                .in_zone(ZoneId::Library(p(1)))
                .with_subtypes(vec![SubType("Goblin".to_string())])
                .with_types(vec![CardType::Creature]),
        )
        .object(
            ObjectSpec::card(p(1), "Elf B")
                .in_zone(ZoneId::Library(p(1)))
                .with_subtypes(vec![SubType("Elf".to_string())])
                .with_types(vec![CardType::Creature]),
        )
        .build()
        .unwrap();

    let source_id = ObjectId(999);
    let mut ctx = ec(p(1), source_id);

    let effect = Effect::RevealAndRoute {
        player: PlayerTarget::Controller,
        count: EffectAmount::Fixed(4),
        filter: TargetFilter {
            has_subtype: Some(SubType("Goblin".to_string())),
            ..Default::default()
        },
        matched_dest: ZoneTarget::Hand {
            owner: PlayerTarget::Controller,
        },
        unmatched_dest: ZoneTarget::Library {
            owner: PlayerTarget::Controller,
            position: LibraryPosition::Bottom,
        },
    };

    execute_effect(&mut state, &effect, &mut ctx);

    // 2 Goblins should be in hand.
    let hand_count = state
        .objects
        .values()
        .filter(|o| o.zone == ZoneId::Hand(p(1)))
        .count();
    assert_eq!(hand_count, 2, "2 Goblins should be in hand");

    // 2 Elves should be in library (bottom).
    let lib_count = state
        .zones
        .get(&ZoneId::Library(p(1)))
        .map(|z| z.len())
        .unwrap_or(0);
    assert_eq!(lib_count, 2, "2 non-Goblins should be in library");
}

#[test]
/// CR 701.16a — empty library: effect does nothing.
fn test_reveal_and_route_empty_library() {
    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .build()
        .unwrap();

    let source_id = ObjectId(999);
    let mut ctx = ec(p(1), source_id);

    let effect = Effect::RevealAndRoute {
        player: PlayerTarget::Controller,
        count: EffectAmount::Fixed(4),
        filter: TargetFilter {
            has_subtype: Some(SubType("Goblin".to_string())),
            ..Default::default()
        },
        matched_dest: ZoneTarget::Hand {
            owner: PlayerTarget::Controller,
        },
        unmatched_dest: ZoneTarget::Library {
            owner: PlayerTarget::Controller,
            position: LibraryPosition::Bottom,
        },
    };

    let events = execute_effect(&mut state, &effect, &mut ctx);
    assert!(events.is_empty(), "empty library should produce no events");
}

#[test]
/// CR 701.16a + CR 110.4a — Chaos Warp pattern: reveal top 1, permanent card
/// goes to battlefield, non-permanent stays on top.
/// Source: Chaos Warp card definition pattern.
fn test_reveal_and_route_permanent_card_to_battlefield() {
    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .object(
            ObjectSpec::card(p(1), "Creature Card")
                .in_zone(ZoneId::Library(p(1)))
                .with_types(vec![CardType::Creature]),
        )
        .build()
        .unwrap();

    let source_id = ObjectId(999);
    let mut ctx = ec(p(1), source_id);

    let effect = Effect::RevealAndRoute {
        player: PlayerTarget::Controller,
        count: EffectAmount::Fixed(1),
        filter: TargetFilter {
            has_card_types: vec![
                CardType::Artifact,
                CardType::Creature,
                CardType::Enchantment,
                CardType::Land,
                CardType::Planeswalker,
            ],
            ..Default::default()
        },
        matched_dest: ZoneTarget::Battlefield { tapped: false },
        unmatched_dest: ZoneTarget::Library {
            owner: PlayerTarget::Controller,
            position: LibraryPosition::Top,
        },
    };

    let events = execute_effect(&mut state, &effect, &mut ctx);

    // Creature should be on the battlefield.
    let bf_count = state
        .objects
        .values()
        .filter(|o| o.zone == ZoneId::Battlefield)
        .count();
    assert_eq!(bf_count, 1, "permanent card should be on battlefield");

    // Should emit PermanentEnteredBattlefield.
    let etb_events = events
        .iter()
        .filter(|e| matches!(e, GameEvent::PermanentEnteredBattlefield { .. }))
        .count();
    assert_eq!(etb_events, 1, "should emit PermanentEnteredBattlefield");
}

// ── Flicker Tests ────────────────────────────────────────────────────────────

#[test]
/// CR 400.7 — basic flicker: exile and return to battlefield as new object.
fn test_flicker_basic_exile_and_return() {
    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .object(
            ObjectSpec::card(p(1), "Target Creature")
                .in_zone(ZoneId::Battlefield)
                .with_types(vec![CardType::Creature]),
        )
        .build()
        .unwrap();

    let original_id = state
        .objects
        .iter()
        .find(|(_, o)| o.zone == ZoneId::Battlefield)
        .map(|(id, _)| *id)
        .unwrap();

    let source_id = ObjectId(999);
    let mut ctx = ec_with_target(p(1), source_id, original_id);

    let effect = Effect::Flicker {
        target: EffectTarget::DeclaredTarget { index: 0 },
        return_tapped: false,
    };

    let events = execute_effect(&mut state, &effect, &mut ctx);

    // The original object should no longer be on the battlefield (it was exiled
    // then returned as a NEW object per CR 400.7).
    assert!(
        !state.objects.contains_key(&original_id),
        "original object should no longer exist"
    );

    // There should be exactly one object on the battlefield (the returned creature).
    let bf_objects: Vec<_> = state
        .objects
        .iter()
        .filter(|(_, o)| o.zone == ZoneId::Battlefield)
        .collect();
    assert_eq!(bf_objects.len(), 1, "one creature should be on battlefield");

    // The returned creature should NOT be tapped.
    let (_, returned_obj) = bf_objects[0];
    assert!(
        !returned_obj.status.tapped,
        "flickered creature should return untapped"
    );

    // Should have both ObjectExiled and PermanentEnteredBattlefield events.
    let exile_events = events
        .iter()
        .filter(|e| matches!(e, GameEvent::ObjectExiled { .. }))
        .count();
    let etb_events = events
        .iter()
        .filter(|e| matches!(e, GameEvent::PermanentEnteredBattlefield { .. }))
        .count();
    assert_eq!(exile_events, 1, "should emit ObjectExiled");
    assert_eq!(etb_events, 1, "should emit PermanentEnteredBattlefield");
}

#[test]
/// CR 400.7 — flicker with return_tapped: permanent returns tapped.
fn test_flicker_return_tapped() {
    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .object(
            ObjectSpec::card(p(1), "Target Creature")
                .in_zone(ZoneId::Battlefield)
                .with_types(vec![CardType::Creature]),
        )
        .build()
        .unwrap();

    let original_id = state
        .objects
        .iter()
        .find(|(_, o)| o.zone == ZoneId::Battlefield)
        .map(|(id, _)| *id)
        .unwrap();

    let source_id = ObjectId(999);
    let mut ctx = ec_with_target(p(1), source_id, original_id);

    let effect = Effect::Flicker {
        target: EffectTarget::DeclaredTarget { index: 0 },
        return_tapped: true,
    };

    execute_effect(&mut state, &effect, &mut ctx);

    let bf_objects: Vec<_> = state
        .objects
        .iter()
        .filter(|(_, o)| o.zone == ZoneId::Battlefield)
        .collect();
    assert_eq!(bf_objects.len(), 1, "one creature should be on battlefield");

    let (_, returned_obj) = bf_objects[0];
    assert!(
        returned_obj.status.tapped,
        "flickered creature should return tapped"
    );
}

#[test]
/// CR 400.7 — flicker target not on battlefield: does nothing.
fn test_flicker_target_not_on_battlefield() {
    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .object(
            ObjectSpec::card(p(1), "Hand Card")
                .in_zone(ZoneId::Hand(p(1)))
                .with_types(vec![CardType::Creature]),
        )
        .build()
        .unwrap();

    let hand_id = state
        .objects
        .iter()
        .find(|(_, o)| o.zone == ZoneId::Hand(p(1)))
        .map(|(id, _)| *id)
        .unwrap();

    let source_id = ObjectId(999);
    let mut ctx = EffectContext::new(
        p(1),
        source_id,
        vec![SpellTarget {
            target: Target::Object(hand_id),
            zone_at_cast: Some(ZoneId::Hand(p(1))),
        }],
    );

    let effect = Effect::Flicker {
        target: EffectTarget::DeclaredTarget { index: 0 },
        return_tapped: false,
    };

    let events = execute_effect(&mut state, &effect, &mut ctx);

    // No events should be emitted — target wasn't on battlefield.
    assert!(
        events.is_empty(),
        "flicker of non-battlefield object should do nothing"
    );

    // The object should still be in hand.
    let still_in_hand = state
        .objects
        .get(&hand_id)
        .map(|o| o.zone == ZoneId::Hand(p(1)))
        .unwrap_or(false);
    assert!(still_in_hand, "card should remain in hand");
}
