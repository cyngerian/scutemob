/// Tests for Effect::RevealAndRoute (CR 701.20a) and Effect::Flicker (CR 400.7).
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
/// CR 121.1: "top" is the end draw_card takes. CR 701.20a (Reveal) — all
/// examined cards match the filter: all go to matched_dest, and a decoy card
/// sitting at the true bottom (below the examined top N) is left completely
/// untouched. Library is 5 cards, `count: 4` — strictly longer than the
/// examined window, so a top/bottom read-end inversion would pull in the
/// decoy (a non-matching card) instead of the true top 4, changing both the
/// hand count and its contents. Source: Goblin Ringleader pattern.
fn test_reveal_and_route_all_match() {
    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        // Bottom decoy: non-Goblin. If the read end were inverted, this card
        // would be examined instead of "Goblin A" and would NOT match,
        // shrinking the hand count from 4 to 3.
        .object(
            ObjectSpec::card(p(1), "Untouched Elf")
                .in_zone(ZoneId::Library(p(1)))
                .with_subtypes(vec![SubType("Elf".to_string())])
                .with_types(vec![CardType::Creature]),
        )
        .object(
            ObjectSpec::card(p(1), "Goblin D")
                .in_zone(ZoneId::Library(p(1)))
                .with_subtypes(vec![SubType("Goblin".to_string())])
                .with_types(vec![CardType::Creature]),
        )
        .object(
            ObjectSpec::card(p(1), "Goblin C")
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
        .object(
            ObjectSpec::card(p(1), "Goblin A")
                .in_zone(ZoneId::Library(p(1)))
                .with_subtypes(vec![SubType("Goblin".to_string())])
                .with_types(vec![CardType::Creature]),
        )
        .build()
        .unwrap();

    let decoy_id = state
        .objects()
        .iter()
        .find(|(_, o)| o.characteristics.name == "Untouched Elf")
        .map(|(id, _)| *id)
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

    // All 4 Goblins (the true top 4) should be in hand -- by name, not just count.
    let mut hand_names: Vec<String> = state
        .objects()
        .values()
        .filter(|o| o.zone == ZoneId::Hand(p(1)))
        .map(|o| o.characteristics.name.clone())
        .collect();
    hand_names.sort();
    assert_eq!(
        hand_names,
        vec![
            "Goblin A".to_string(),
            "Goblin B".to_string(),
            "Goblin C".to_string(),
            "Goblin D".to_string(),
        ],
        "CR 121.1: the true top 4 (all Goblins) should be routed to hand"
    );

    // Library should hold exactly the untouched decoy.
    let lib_ids = state
        .zones()
        .get(&ZoneId::Library(p(1)))
        .unwrap()
        .object_ids();
    assert_eq!(
        lib_ids,
        vec![decoy_id],
        "the bottom decoy must be the only card left in library, with its \
         ObjectId unchanged (never read or moved)"
    );

    let hand_events = events
        .iter()
        .filter(|e| matches!(e, GameEvent::ObjectReturnedToHand { .. }))
        .count();
    assert_eq!(hand_events, 4, "should emit 4 ObjectReturnedToHand events");
}

#[test]
/// CR 121.1, CR 701.20a — no examined card matches the filter: all go to
/// unmatched_dest. The decoy at the true bottom is a Goblin (WOULD match if
/// the read end were inverted and it were pulled into the examined window),
/// while the examined top 4 are Elves (none match). Discriminates a
/// read-end inversion via hand contents, not just counts.
fn test_reveal_and_route_none_match() {
    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        // Bottom decoy: a Goblin. If wrongly examined, it would land in hand.
        .object(
            ObjectSpec::card(p(1), "Untouched Goblin")
                .in_zone(ZoneId::Library(p(1)))
                .with_subtypes(vec![SubType("Goblin".to_string())])
                .with_types(vec![CardType::Creature]),
        )
        .object(
            ObjectSpec::card(p(1), "Elf D")
                .in_zone(ZoneId::Library(p(1)))
                .with_subtypes(vec![SubType("Elf".to_string())])
                .with_types(vec![CardType::Creature]),
        )
        .object(
            ObjectSpec::card(p(1), "Elf C")
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
        .object(
            ObjectSpec::card(p(1), "Elf A")
                .in_zone(ZoneId::Library(p(1)))
                .with_subtypes(vec![SubType("Elf".to_string())])
                .with_types(vec![CardType::Creature]),
        )
        .build()
        .unwrap();

    let decoy_id = state
        .objects()
        .iter()
        .find(|(_, o)| o.characteristics.name == "Untouched Goblin")
        .map(|(id, _)| *id)
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

    // Hand must be empty -- if the decoy Goblin were wrongly examined, it
    // would have ended up here instead.
    let hand_count = state
        .objects()
        .values()
        .filter(|o| o.zone == ZoneId::Hand(p(1)))
        .count();
    assert_eq!(
        hand_count, 0,
        "CR 121.1: the true top 4 (all Elves) don't match; the bottom decoy \
         Goblin must not have been examined"
    );

    // All 5 cards remain in library; the decoy's identity is unchanged
    // (it was never a member of top_ids, only displaced by insertions).
    //
    // CR 121.1 / OOS-RS-1 review gap: the count/membership checks above pass
    // identically whether `unmatched_dest`'s Library{Bottom} branch routes
    // through `move_object_to_bottom_of_zone` (push_front, correct) or
    // through plain `move_object_to_zone` (push_back/append, i.e. bottomed
    // cards silently land on TOP instead). Only a positional assertion on
    // `object_ids()` discriminates the two. `Zone::object_ids()` walks the
    // ordered vector low-to-high index, and index len-1 is the top (`top()` =
    // `v.last()`). The decoy was the library's *sole* card before this
    // effect ran (at index 0, the bottom). Four cards being correctly
    // bottomed BELOW it (via `push_front`, each insert at index 0) shifts it
    // to the last index — i.e. the decoy ends up TOPMOST once four unmatched
    // cards go under it.
    let lib_ids = state
        .zones()
        .get(&ZoneId::Library(p(1)))
        .unwrap()
        .object_ids();
    assert_eq!(lib_ids.len(), 5, "no cards should be lost");
    assert_eq!(
        lib_ids[4], decoy_id,
        "CR 121.1: the decoy — the library's sole card before this effect, \
         hence its bottom — must sit at index 4 (the top) once the 4 \
         unmatched Elves are correctly bottomed BELOW it via push_front. If \
         the bottom-write dispatch silently fell back to a top-append \
         (push_back), the decoy would remain stranded at index 0 instead."
    );
    let elf_ids: std::collections::HashSet<ObjectId> = ["Elf A", "Elf B", "Elf C", "Elf D"]
        .iter()
        .map(|name| {
            state
                .objects()
                .iter()
                .find(|(_, o)| o.characteristics.name == *name)
                .map(|(id, _)| *id)
                .unwrap()
        })
        .collect();
    let bottomed_elves: std::collections::HashSet<ObjectId> =
        lib_ids[0..4].iter().copied().collect();
    assert_eq!(
        bottomed_elves, elf_ids,
        "the 4 bottomed Elves (post-move, CR 400.7 new-object identities) \
         must occupy indices 0..4, all strictly below the decoy at index 4"
    );

    let lib_events = events
        .iter()
        .filter(|e| matches!(e, GameEvent::ObjectPutOnLibrary { .. }))
        .count();
    assert_eq!(lib_events, 4, "should emit 4 ObjectPutOnLibrary events");
}

#[test]
/// CR 121.1, CR 701.20a — partial match: matching cards go to one zone, rest
/// to another, and a non-matching decoy at the true bottom is left alone.
/// Source: Goblin Ringleader ETB with mixed library top.
fn test_reveal_and_route_partial_match() {
    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        // Bottom decoy: non-Goblin, must never be examined or moved.
        .object(
            ObjectSpec::card(p(1), "Untouched Elf")
                .in_zone(ZoneId::Library(p(1)))
                .with_subtypes(vec![SubType("Elf".to_string())])
                .with_types(vec![CardType::Creature]),
        )
        .object(
            ObjectSpec::card(p(1), "Elf D")
                .in_zone(ZoneId::Library(p(1)))
                .with_subtypes(vec![SubType("Elf".to_string())])
                .with_types(vec![CardType::Creature]),
        )
        .object(
            ObjectSpec::card(p(1), "Goblin C")
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
        .object(
            ObjectSpec::card(p(1), "Goblin A")
                .in_zone(ZoneId::Library(p(1)))
                .with_subtypes(vec![SubType("Goblin".to_string())])
                .with_types(vec![CardType::Creature]),
        )
        .build()
        .unwrap();

    let decoy_id = state
        .objects()
        .iter()
        .find(|(_, o)| o.characteristics.name == "Untouched Elf")
        .map(|(id, _)| *id)
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

    // Exactly Goblin A and Goblin C should be in hand.
    let mut hand_names: Vec<String> = state
        .objects()
        .values()
        .filter(|o| o.zone == ZoneId::Hand(p(1)))
        .map(|o| o.characteristics.name.clone())
        .collect();
    hand_names.sort();
    assert_eq!(
        hand_names,
        vec!["Goblin A".to_string(), "Goblin C".to_string()],
        "CR 121.1: exactly the true top 4's Goblins should be in hand"
    );

    // Library holds the decoy plus the 2 bottomed Elves.
    let lib_ids = state
        .zones()
        .get(&ZoneId::Library(p(1)))
        .unwrap()
        .object_ids();
    assert_eq!(lib_ids.len(), 3, "decoy + 2 bottomed Elves");
    assert!(
        lib_ids.contains(&decoy_id),
        "the decoy's ObjectId must still be present (untouched)"
    );
}

#[test]
/// CR 701.20a — empty library: effect does nothing.
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
/// CR 121.1, CR 701.20a + CR 110.4a — Chaos Warp pattern: reveal top 1,
/// permanent card goes to battlefield, non-permanent stays on top. A
/// non-permanent decoy sits at the true bottom (below the examined card) --
/// if the read end were inverted, the decoy (an instant) would be examined
/// instead, and nothing would reach the battlefield.
/// Source: Chaos Warp card definition pattern.
fn test_reveal_and_route_permanent_card_to_battlefield() {
    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        // Bottom decoy: a non-permanent (Instant). Must never be examined.
        .object(
            ObjectSpec::card(p(1), "Untouched Instant")
                .in_zone(ZoneId::Library(p(1)))
                .with_types(vec![CardType::Instant]),
        )
        .object(
            ObjectSpec::card(p(1), "Creature Card")
                .in_zone(ZoneId::Library(p(1)))
                .with_types(vec![CardType::Creature]),
        )
        .build()
        .unwrap();

    let decoy_id = state
        .objects()
        .iter()
        .find(|(_, o)| o.characteristics.name == "Untouched Instant")
        .map(|(id, _)| *id)
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

    // The true top card (Creature Card) should be on the battlefield.
    let bf_objects: Vec<_> = state
        .objects()
        .values()
        .filter(|o| o.zone == ZoneId::Battlefield)
        .collect();
    assert_eq!(
        bf_objects.len(),
        1,
        "permanent card should be on battlefield"
    );
    assert_eq!(
        bf_objects[0].characteristics.name, "Creature Card",
        "CR 121.1: the examined card must be the true top, not the bottom decoy"
    );

    // The decoy must still be in library, untouched (same ObjectId).
    let lib_ids = state
        .zones()
        .get(&ZoneId::Library(p(1)))
        .unwrap()
        .object_ids();
    assert_eq!(
        lib_ids,
        vec![decoy_id],
        "the bottom decoy must remain, unmoved (never examined)"
    );

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
        .objects()
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
        !state.objects().contains_key(&original_id),
        "original object should no longer exist"
    );

    // There should be exactly one object on the battlefield (the returned creature).
    let bf_objects: Vec<_> = state
        .objects()
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
        .objects()
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
        .objects()
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
        .objects()
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
        .objects()
        .get(&hand_id)
        .map(|o| o.zone == ZoneId::Hand(p(1)))
        .unwrap_or(false);
    assert!(still_in_hand, "card should remain in hand");
}
