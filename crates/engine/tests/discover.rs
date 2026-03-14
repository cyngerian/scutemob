//! Discover keyword action tests (CR 701.57).
//!
//! Discover is a keyword action (CR 701.57), not a triggered ability like Cascade
//! (CR 702.85). Cards perform discover via their own ETB or other triggered
//! abilities which include `Effect::Discover { player, n }` as their effect.
//!
//! Key differences from Cascade:
//! - MV threshold is `<= N` (Cascade uses `< spell_MV`)
//! - Declined card goes to hand (Cascade puts non-cast cards on library bottom)
//! - Uses a fixed N parameter rather than the spell's own MV
//!
//! Test strategy: use a creature with a WhenEntersBattlefield triggered ability
//! that includes Effect::Discover, then cast it, resolve the ETB trigger, and
//! observe the discover outcomes.
//!
//! # Known gap: DiscoverToHand not tested
//!
//! CR 701.57a allows the player to decline casting the discovered card and put it
//! into hand instead. The deterministic engine always casts (matching Cascade's
//! policy), so the DiscoverToHand path is unreachable in tests. The event and
//! hand-move logic exist in copy.rs and are structurally correct, but cannot be
//! exercised until M10 adds player-choice infrastructure. When that lands, add a
//! test that declines to cast and verifies the card moves to hand via DiscoverToHand.

use mtg_engine::{
    process_command, AbilityDefinition, CardDefinition, CardId, CardRegistry, CardType, Command,
    Effect, EffectAmount, GameEvent, GameState, GameStateBuilder, KeywordAbility, ManaCost,
    ObjectSpec, PlayerId, PlayerTarget, StackObjectKind, Step, TriggerCondition, TypeLine, ZoneId,
};

fn p1() -> PlayerId {
    PlayerId(1)
}
fn p2() -> PlayerId {
    PlayerId(2)
}

/// Pass priority for all listed players once (stack resolves when all pass).
fn pass_all(state: GameState, players: &[PlayerId]) -> (GameState, Vec<GameEvent>) {
    let mut all_events = Vec::new();
    let mut current = state;
    for &p in players {
        let (s, ev) = process_command(current, Command::PassPriority { player: p })
            .unwrap_or_else(|e| panic!("PassPriority by {:?} failed: {:?}", p, e));
        current = s;
        all_events.extend(ev);
    }
    (current, all_events)
}

/// A creature with "When this enters, Discover N."
///
/// Uses the Discover keyword marker plus a WhenEntersBattlefield trigger
/// whose effect is Effect::Discover { player: Controller, n }.
fn discover_creature(id: &str, name: &str, mv: u32, discover_n: u32) -> CardDefinition {
    CardDefinition {
        card_id: CardId(id.into()),
        name: name.into(),
        mana_cost: Some(ManaCost {
            generic: mv,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].iter().cloned().collect(),
            ..Default::default()
        },
        oracle_text: format!("When this creature enters, discover {discover_n}."),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Discover),
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                effect: Effect::Discover {
                    player: PlayerTarget::Controller,
                    n: discover_n,
                },
                intervening_if: None,
            },
        ],
        power: Some(2),
        toughness: Some(2),
        color_indicator: None,
        back_face: None,
    }
}

/// A plain sorcery with a specific mana value (for discover to find).
fn plain_sorcery(id: &str, name: &str, mv: u32) -> CardDefinition {
    CardDefinition {
        card_id: CardId(id.into()),
        name: name.into(),
        mana_cost: Some(ManaCost {
            generic: mv,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Sorcery].iter().cloned().collect(),
            ..Default::default()
        },
        oracle_text: "Plain sorcery — gain 1 life.".into(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::GainLife {
                player: PlayerTarget::Controller,
                amount: EffectAmount::Fixed(1),
            },
            targets: vec![],
            modes: None,
            cant_be_countered: false,
        }],
        power: None,
        toughness: None,
        color_indicator: None,
        back_face: None,
    }
}

/// A basic land definition (lands are skipped during discover — CR 701.57a).
fn basic_land(id: &str, name: &str) -> CardDefinition {
    CardDefinition {
        card_id: CardId(id.into()),
        name: name.into(),
        mana_cost: None,
        types: TypeLine {
            card_types: [CardType::Land].iter().cloned().collect(),
            ..Default::default()
        },
        oracle_text: "{T}: Add {G}.".into(),
        abilities: vec![],
        power: None,
        toughness: None,
        color_indicator: None,
        back_face: None,
    }
}

/// Cast the named spell from p1's hand and give it enough mana, then return
/// the state after the CastSpell command (the ETB trigger is not yet resolved).
fn cast_discover_creature(state: GameState, name: &str) -> (GameState, Vec<GameEvent>) {
    let card_id = state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name && obj.zone == ZoneId::Hand(p1()))
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("Card '{}' not found in p1's hand", name));

    process_command(
        state,
        Command::CastSpell {
            player: p1(),
            card: card_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: None,
            prototype: false,
            modes_chosen: vec![],
            x_value: 0,
            face_down_kind: None,
            additional_costs: vec![],
        },
    )
    .unwrap_or_else(|e| panic!("CastSpell '{}' failed: {:?}", name, e))
}

// ── Test 1: Basic discover finds and casts a qualifying card ──────────────────

/// CR 701.57a — Discover exiles lands, then finds and casts the first nonland
/// card with MV <= N.
///
/// Setup: Discover 3. Library (top to bottom): Forest (land, skip), Sorcery MV=2
/// (MV 2 <= 3, qualifies). After discover resolves: sorcery is on the stack,
/// the forest is at the library bottom, DiscoverExiled and DiscoverCast events fire.
#[test]
fn test_discover_basic_finds_and_casts_card() {
    let p1 = p1();
    let p2 = p2();

    let creature_def = discover_creature("disc-creature-1", "Disc Creature 1", 3, 3);
    let sorcery_def = plain_sorcery("disc-sorcery-mv2", "Small Sorcery", 2);
    let land_def = basic_land("disc-forest-1", "Forest 1");

    let registry = CardRegistry::new(vec![creature_def, sorcery_def, land_def]);

    // Library (top to bottom): Forest 1 (top = last pushed), Small Sorcery (bottom = first pushed).
    // Push order: Sorcery first (bottom), Forest last (top).
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(
            ObjectSpec::card(p1, "Disc Creature 1")
                .with_card_id(CardId("disc-creature-1".into()))
                .with_types(vec![CardType::Creature])
                .with_mana_cost(ManaCost {
                    generic: 3,
                    ..Default::default()
                })
                .with_keyword(KeywordAbility::Discover)
                .in_zone(ZoneId::Hand(p1)),
        )
        // Bottom of library (first pushed).
        .object(
            ObjectSpec::card(p1, "Small Sorcery")
                .with_card_id(CardId("disc-sorcery-mv2".into()))
                .with_types(vec![CardType::Sorcery])
                .with_mana_cost(ManaCost {
                    generic: 2,
                    ..Default::default()
                })
                .in_zone(ZoneId::Library(p1)),
        )
        // Top of library (last pushed = last = top).
        .object(
            ObjectSpec::card(p1, "Forest 1")
                .with_card_id(CardId("disc-forest-1".into()))
                .with_types(vec![CardType::Land])
                .in_zone(ZoneId::Library(p1)),
        )
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .with_registry(CardRegistry::new(vec![
            discover_creature("disc-creature-1", "Disc Creature 1", 3, 3),
            plain_sorcery("disc-sorcery-mv2", "Small Sorcery", 2),
            basic_land("disc-forest-1", "Forest 1"),
        ]))
        .build()
        .unwrap();

    state.players.get_mut(&p1).unwrap().mana_pool.colorless = 3;

    // Cast the discover creature.
    let (state, _) = cast_discover_creature(state, "Disc Creature 1");

    // Both players pass priority → creature spell resolves → ETB trigger queued on stack.
    let (state, _) = pass_all(state, &[p1, p2]);

    // CR 603.3: ETB trigger is on the stack. Both players pass priority again to resolve it.
    // Discover 3 executes: Forest exiled, Small Sorcery discovered and cast.
    let (state, resolve_events) = pass_all(state, &[p1, p2]);

    // DiscoverExiled should fire (Forest exiled), DiscoverCast should fire (Small Sorcery cast).
    let exiled_count = resolve_events
        .iter()
        .filter(|e| matches!(e, GameEvent::DiscoverExiled { .. }))
        .count();
    assert!(
        exiled_count >= 1,
        "Expected at least 1 DiscoverExiled event; got {}",
        exiled_count
    );

    let cast_count = resolve_events
        .iter()
        .filter(|e| matches!(e, GameEvent::DiscoverCast { .. }))
        .count();
    assert_eq!(
        cast_count, 1,
        "Expected 1 DiscoverCast event; got {}",
        cast_count
    );

    // The Small Sorcery should now be on the stack (Disc Creature still resolves separately).
    let sorcery_on_stack = state.stack_objects.iter().any(|so| {
        if let StackObjectKind::Spell { source_object } = so.kind {
            state
                .objects
                .get(&source_object)
                .map(|obj| obj.characteristics.name == "Small Sorcery")
                .unwrap_or(false)
        } else {
            false
        }
    });
    assert!(
        sorcery_on_stack,
        "Small Sorcery should be on the stack after discover casts it"
    );

    // Forest should NOT be in the library (it was exiled then put to library bottom,
    // so it returns to library — but the stack hasn't resolved yet, so it may still
    // be at the bottom of the library).
    let forest_in_library = state
        .objects
        .values()
        .any(|obj| obj.characteristics.name == "Forest 1" && obj.zone == ZoneId::Library(p1));
    assert!(
        forest_in_library,
        "Forest 1 should be at the bottom of the library after discover (exiled then returned)"
    );
}

// ── Test 2: Discover uses <= N (unlike Cascade which uses < N) ────────────────

/// CR 701.57a — A card with MV exactly equal to N qualifies for discover
/// (MV <= N, inclusive). This is the key difference from Cascade (CR 702.85a)
/// which uses strictly-less-than (MV < spell_MV).
#[test]
fn test_discover_mv_equal_to_n_is_valid() {
    let p1 = p1();
    let p2 = p2();

    // Discover 3. Library top = a MV=3 sorcery. MV 3 <= 3, so it qualifies.
    let creature_def = discover_creature("disc-creature-eq", "Disc Creature Eq", 4, 3);
    let equal_mv_def = plain_sorcery("disc-sorcery-mv3", "Equal MV Sorcery", 3);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![creature_def, equal_mv_def]))
        .object(
            ObjectSpec::card(p1, "Disc Creature Eq")
                .with_card_id(CardId("disc-creature-eq".into()))
                .with_types(vec![CardType::Creature])
                .with_mana_cost(ManaCost {
                    generic: 4,
                    ..Default::default()
                })
                .with_keyword(KeywordAbility::Discover)
                .in_zone(ZoneId::Hand(p1)),
        )
        // Library top = MV=3 sorcery (exact match).
        .object(
            ObjectSpec::card(p1, "Equal MV Sorcery")
                .with_card_id(CardId("disc-sorcery-mv3".into()))
                .with_types(vec![CardType::Sorcery])
                .with_mana_cost(ManaCost {
                    generic: 3,
                    ..Default::default()
                })
                .in_zone(ZoneId::Library(p1)),
        )
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();

    state.players.get_mut(&p1).unwrap().mana_pool.colorless = 4;

    let (state, _) = cast_discover_creature(state, "Disc Creature Eq");
    // Resolve the creature spell.
    let (state, _) = pass_all(state, &[p1, p2]);
    // CR 603.3: Resolve the ETB discover trigger.
    let (state, resolve_events) = pass_all(state, &[p1, p2]);

    // MV=3 <= Discover 3: the card should be cast, not skipped.
    let cast_count = resolve_events
        .iter()
        .filter(|e| matches!(e, GameEvent::DiscoverCast { .. }))
        .count();
    assert_eq!(
        cast_count, 1,
        "MV=3 card should be cast by Discover 3 (MV <= N); got {} DiscoverCast events",
        cast_count
    );

    let sorcery_on_stack = state.stack_objects.iter().any(|so| {
        if let StackObjectKind::Spell { source_object } = so.kind {
            state
                .objects
                .get(&source_object)
                .map(|obj| obj.characteristics.name == "Equal MV Sorcery")
                .unwrap_or(false)
        } else {
            false
        }
    });
    assert!(
        sorcery_on_stack,
        "Equal MV Sorcery (MV=3) should be on the stack after Discover 3"
    );
}

// ── Test 3: Empty library — discover completes without error ──────────────────

/// CR 701.57b — "A player has 'discovered' after the process described in
/// 701.57a is complete, even if some or all of those actions were impossible."
/// Empty library: no cards are exiled, no card is found. Discover completes
/// silently without error or state corruption.
#[test]
fn test_discover_empty_library() {
    let p1 = p1();
    let p2 = p2();

    let creature_def = discover_creature("disc-creature-empty", "Disc Creature Empty", 3, 3);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![creature_def]))
        .object(
            ObjectSpec::card(p1, "Disc Creature Empty")
                .with_card_id(CardId("disc-creature-empty".into()))
                .with_types(vec![CardType::Creature])
                .with_mana_cost(ManaCost {
                    generic: 3,
                    ..Default::default()
                })
                .with_keyword(KeywordAbility::Discover)
                .in_zone(ZoneId::Hand(p1)),
        )
        // No cards in library.
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();

    state.players.get_mut(&p1).unwrap().mana_pool.colorless = 3;

    // Should not panic or error.
    let (state, _) = cast_discover_creature(state, "Disc Creature Empty");
    let (state, resolve_events) = pass_all(state, &[p1, p2]);

    // No DiscoverCast and no DiscoverToHand — nothing was found.
    let cast_count = resolve_events
        .iter()
        .filter(|e| matches!(e, GameEvent::DiscoverCast { .. }))
        .count();
    let to_hand_count = resolve_events
        .iter()
        .filter(|e| matches!(e, GameEvent::DiscoverToHand { .. }))
        .count();
    assert_eq!(cast_count, 0, "No DiscoverCast with empty library");
    assert_eq!(to_hand_count, 0, "No DiscoverToHand with empty library");

    // Engine state must still be valid: creature should have entered the battlefield.
    let creature_on_bf = state.objects.values().any(|obj| {
        obj.characteristics.name == "Disc Creature Empty" && obj.zone == ZoneId::Battlefield
    });
    assert!(
        creature_on_bf,
        "Disc Creature Empty should be on the battlefield after the spell resolves"
    );
}

// ── Test 4: All lands in library — no qualifying card found ───────────────────

/// CR 701.57a — If the library contains only lands, all are exiled but no
/// qualifying nonland card is found. All exiled lands go to the library bottom.
/// DiscoverExiled fires (listing all lands), no DiscoverCast or DiscoverToHand.
#[test]
fn test_discover_all_lands_in_library() {
    let p1 = p1();
    let p2 = p2();

    let creature_def = discover_creature("disc-creature-lands", "Disc Creature Lands", 3, 3);
    let forest1_def = basic_land("disc-land-1", "Forest A");
    let forest2_def = basic_land("disc-land-2", "Forest B");

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![
            creature_def,
            forest1_def,
            forest2_def,
        ]))
        .object(
            ObjectSpec::card(p1, "Disc Creature Lands")
                .with_card_id(CardId("disc-creature-lands".into()))
                .with_types(vec![CardType::Creature])
                .with_mana_cost(ManaCost {
                    generic: 3,
                    ..Default::default()
                })
                .with_keyword(KeywordAbility::Discover)
                .in_zone(ZoneId::Hand(p1)),
        )
        .object(
            ObjectSpec::card(p1, "Forest A")
                .with_card_id(CardId("disc-land-1".into()))
                .with_types(vec![CardType::Land])
                .in_zone(ZoneId::Library(p1)),
        )
        .object(
            ObjectSpec::card(p1, "Forest B")
                .with_card_id(CardId("disc-land-2".into()))
                .with_types(vec![CardType::Land])
                .in_zone(ZoneId::Library(p1)),
        )
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();

    state.players.get_mut(&p1).unwrap().mana_pool.colorless = 3;

    let (state, _) = cast_discover_creature(state, "Disc Creature Lands");
    // Resolve the creature spell.
    let (state, _) = pass_all(state, &[p1, p2]);
    // CR 603.3: Resolve the ETB discover trigger.
    let (state, resolve_events) = pass_all(state, &[p1, p2]);

    // No cast, no hand — all lands skipped.
    let cast_count = resolve_events
        .iter()
        .filter(|e| matches!(e, GameEvent::DiscoverCast { .. }))
        .count();
    let to_hand_count = resolve_events
        .iter()
        .filter(|e| matches!(e, GameEvent::DiscoverToHand { .. }))
        .count();
    assert_eq!(cast_count, 0, "No DiscoverCast when only lands in library");
    assert_eq!(
        to_hand_count, 0,
        "No DiscoverToHand when only lands in library"
    );

    // DiscoverExiled fires listing the exiled lands.
    let exiled_ev = resolve_events
        .iter()
        .find(|e| matches!(e, GameEvent::DiscoverExiled { .. }));
    assert!(
        exiled_ev.is_some(),
        "DiscoverExiled should fire even when no qualifying card is found"
    );
    if let Some(GameEvent::DiscoverExiled { cards_exiled, .. }) = exiled_ev {
        assert_eq!(
            cards_exiled.len(),
            2,
            "Both lands should be listed in DiscoverExiled; got {}",
            cards_exiled.len()
        );
    }

    // Both lands should be back in the library (exiled then put on bottom).
    let lands_in_lib = state
        .objects
        .values()
        .filter(|obj| {
            obj.zone == ZoneId::Library(p1)
                && (obj.characteristics.name == "Forest A"
                    || obj.characteristics.name == "Forest B")
        })
        .count();
    assert_eq!(
        lands_in_lib, 2,
        "Both lands should be returned to the library bottom after discover"
    );
}

// ── Test 5: Remaining exiled cards go to library bottom ───────────────────────

/// CR 701.57a — "Put the remaining exiled cards on the bottom of your library
/// in a random order." After discovering a qualifying card (and casting it),
/// the other exiled cards (lands, too-high MV cards) go to the library bottom.
///
/// Setup: Library (top to bottom): Forest (skip), MV=2 sorcery (cast), MV=1 sorcery.
/// After discover 3: MV=2 sorcery is cast (removed from exiled set), Forest
/// goes to library bottom.
#[test]
fn test_discover_remaining_cards_go_to_library_bottom() {
    let p1 = p1();
    let p2 = p2();

    let creature_def = discover_creature("disc-creature-rem", "Disc Creature Rem", 4, 3);
    let sorcery_mv2_def = plain_sorcery("disc-sorcery-rem-mv2", "Mid Sorcery", 2);
    let sorcery_mv1_def = plain_sorcery("disc-sorcery-rem-mv1", "Bottom Sorcery", 1);
    let land_def = basic_land("disc-land-rem", "Forest Rem");

    // Library (top to bottom): Forest Rem (top), Mid Sorcery, Bottom Sorcery.
    // Discover 3: Forest exiled (land, skip), Mid Sorcery exiled (MV=2 <= 3, cast).
    // Remaining exiled = [Forest Rem]. Bottom Sorcery stays in library (never exiled).
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![
            creature_def,
            sorcery_mv2_def,
            sorcery_mv1_def,
            land_def,
        ]))
        .object(
            ObjectSpec::card(p1, "Disc Creature Rem")
                .with_card_id(CardId("disc-creature-rem".into()))
                .with_types(vec![CardType::Creature])
                .with_mana_cost(ManaCost {
                    generic: 4,
                    ..Default::default()
                })
                .with_keyword(KeywordAbility::Discover)
                .in_zone(ZoneId::Hand(p1)),
        )
        // Pushed in order: Bottom Sorcery (very bottom), Mid Sorcery, Forest Rem (top).
        .object(
            ObjectSpec::card(p1, "Bottom Sorcery")
                .with_card_id(CardId("disc-sorcery-rem-mv1".into()))
                .with_types(vec![CardType::Sorcery])
                .with_mana_cost(ManaCost {
                    generic: 1,
                    ..Default::default()
                })
                .in_zone(ZoneId::Library(p1)),
        )
        .object(
            ObjectSpec::card(p1, "Mid Sorcery")
                .with_card_id(CardId("disc-sorcery-rem-mv2".into()))
                .with_types(vec![CardType::Sorcery])
                .with_mana_cost(ManaCost {
                    generic: 2,
                    ..Default::default()
                })
                .in_zone(ZoneId::Library(p1)),
        )
        .object(
            ObjectSpec::card(p1, "Forest Rem")
                .with_card_id(CardId("disc-land-rem".into()))
                .with_types(vec![CardType::Land])
                .in_zone(ZoneId::Library(p1)),
        )
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();

    state.players.get_mut(&p1).unwrap().mana_pool.colorless = 4;

    let (state, _) = cast_discover_creature(state, "Disc Creature Rem");
    // Resolve the creature spell.
    let (state, _) = pass_all(state, &[p1, p2]);
    // CR 603.3: Resolve the ETB discover trigger.
    let (state, resolve_events) = pass_all(state, &[p1, p2]);

    // Mid Sorcery (MV=2) should have been discovered and cast.
    let cast_count = resolve_events
        .iter()
        .filter(|e| matches!(e, GameEvent::DiscoverCast { .. }))
        .count();
    assert_eq!(cast_count, 1, "Mid Sorcery should be discovered and cast");

    // Forest Rem should be back in the library (not in exile).
    let forest_in_lib = state
        .objects
        .values()
        .any(|obj| obj.characteristics.name == "Forest Rem" && obj.zone == ZoneId::Library(p1));
    assert!(
        forest_in_lib,
        "Forest Rem should be returned to library bottom after discover"
    );

    let forest_in_exile = state
        .objects
        .values()
        .any(|obj| obj.characteristics.name == "Forest Rem" && obj.zone == ZoneId::Exile);
    assert!(
        !forest_in_exile,
        "Forest Rem should NOT remain in exile after discover"
    );

    // Bottom Sorcery should still be in the library (was never touched).
    let bottom_in_lib = state
        .objects
        .values()
        .any(|obj| obj.characteristics.name == "Bottom Sorcery" && obj.zone == ZoneId::Library(p1));
    assert!(
        bottom_in_lib,
        "Bottom Sorcery should remain in library (discover stopped at Mid Sorcery)"
    );
}

// ── Test 6: Discover vs Cascade MV threshold difference ───────────────────────

/// CR 701.57a vs CR 702.85a — Discover uses `<=` while Cascade uses `<`.
///
/// A card with MV=3 should be found by "Discover 3" (3 <= 3 is true)
/// but should NOT be found by a "Cascade" spell with MV=3 (3 < 3 is false).
///
/// This test verifies discover specifically: it finds a card whose MV equals N.
/// (See also test_discover_mv_equal_to_n_is_valid which tests the same invariant
/// more directly.)
#[test]
fn test_discover_vs_cascade_mv_threshold() {
    let p1 = p1();
    let p2 = p2();

    // Discover 3 with a MV=3 card on top of library → should find it.
    let creature_def = discover_creature("disc-creature-thr", "Disc Creature Thr", 4, 3);
    let mv3_sorcery_def = plain_sorcery("disc-sorcery-thr-mv3", "Threshold Sorcery", 3);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![creature_def, mv3_sorcery_def]))
        .object(
            ObjectSpec::card(p1, "Disc Creature Thr")
                .with_card_id(CardId("disc-creature-thr".into()))
                .with_types(vec![CardType::Creature])
                .with_mana_cost(ManaCost {
                    generic: 4,
                    ..Default::default()
                })
                .with_keyword(KeywordAbility::Discover)
                .in_zone(ZoneId::Hand(p1)),
        )
        // Library top: MV=3 sorcery.
        .object(
            ObjectSpec::card(p1, "Threshold Sorcery")
                .with_card_id(CardId("disc-sorcery-thr-mv3".into()))
                .with_types(vec![CardType::Sorcery])
                .with_mana_cost(ManaCost {
                    generic: 3,
                    ..Default::default()
                })
                .in_zone(ZoneId::Library(p1)),
        )
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();

    state.players.get_mut(&p1).unwrap().mana_pool.colorless = 4;

    let (state, _) = cast_discover_creature(state, "Disc Creature Thr");
    // Resolve the creature spell.
    let (state, _) = pass_all(state, &[p1, p2]);
    // CR 603.3: Resolve the ETB discover trigger.
    let (_state, resolve_events) = pass_all(state, &[p1, p2]);

    // Discover 3 MUST find a MV=3 card (3 <= 3).
    let cast_count = resolve_events
        .iter()
        .filter(|e| matches!(e, GameEvent::DiscoverCast { .. }))
        .count();
    assert_eq!(
        cast_count, 1,
        "Discover 3 must cast MV=3 card (3 <= 3 is true); got {} DiscoverCast events",
        cast_count
    );
}

// ── Test 7: MV > N skips the card (discover only exiles qualifying cards) ─────

/// CR 701.57a — Cards with MV > N are skipped and exiled during the search,
/// eventually reaching the library bottom if no qualifying card is found.
///
/// Setup: Library top = MV=5 sorcery (MV 5 > 3, skip). No other cards.
/// Discover 3: MV=5 sorcery is exiled (skipped), library empties. The MV=5
/// sorcery goes to library bottom. No DiscoverCast or DiscoverToHand.
///
/// Note: A nonland card is still exiled even if its MV > N (the exiling
/// is mandatory; only the cast is conditional on MV <= N).
#[test]
fn test_discover_high_mv_card_goes_to_library_bottom() {
    let p1 = p1();
    let p2 = p2();

    let creature_def = discover_creature("disc-creature-highmv", "Disc Creature HighMV", 3, 3);
    let high_mv_def = plain_sorcery("disc-sorcery-highmv", "High MV Sorcery", 5);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![creature_def, high_mv_def]))
        .object(
            ObjectSpec::card(p1, "Disc Creature HighMV")
                .with_card_id(CardId("disc-creature-highmv".into()))
                .with_types(vec![CardType::Creature])
                .with_mana_cost(ManaCost {
                    generic: 3,
                    ..Default::default()
                })
                .with_keyword(KeywordAbility::Discover)
                .in_zone(ZoneId::Hand(p1)),
        )
        .object(
            ObjectSpec::card(p1, "High MV Sorcery")
                .with_card_id(CardId("disc-sorcery-highmv".into()))
                .with_types(vec![CardType::Sorcery])
                .with_mana_cost(ManaCost {
                    generic: 5,
                    ..Default::default()
                })
                .in_zone(ZoneId::Library(p1)),
        )
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();

    state.players.get_mut(&p1).unwrap().mana_pool.colorless = 3;

    let (state, _) = cast_discover_creature(state, "Disc Creature HighMV");
    let (state, resolve_events) = pass_all(state, &[p1, p2]);

    // No DiscoverCast (MV=5 > N=3, so it's not the qualifying card).
    let cast_count = resolve_events
        .iter()
        .filter(|e| matches!(e, GameEvent::DiscoverCast { .. }))
        .count();
    assert_eq!(
        cast_count, 0,
        "MV=5 card should NOT be cast by Discover 3 (5 > 3); got {} DiscoverCast events",
        cast_count
    );

    // The MV=5 sorcery is exiled (skip) then goes to library bottom.
    let sorcery_in_lib = state.objects.values().any(|obj| {
        obj.characteristics.name == "High MV Sorcery" && obj.zone == ZoneId::Library(p1)
    });
    assert!(
        sorcery_in_lib,
        "High MV Sorcery should be at library bottom after being exiled and skipped"
    );
}
