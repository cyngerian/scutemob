//! Surveil keyword action tests (CR 701.25).
//!
//! Surveil is a keyword action (like Scry), not a keyword ability. Cards say
//! "Surveil N" as part of a spell effect or triggered/activated ability effect.
//!
//! Key rules verified:
//! - Surveil N moves the top N cards of the library to the graveyard (CR 701.25a).
//!   Deterministic fallback: all looked-at cards go to graveyard.
//! - Surveil 0 produces no event (CR 701.25c).
//! - Surveil with empty library still emits event (CR 701.25d).
//! - Surveil with fewer than N cards in library handles partial fill (CR 701.25d).
//! - Surveil then draw sequence works correctly (common card pattern).
//! - "Whenever you surveil" triggers fire via TriggerEvent::ControllerSurveils (CR 701.25d).
//! - Surveil 0 does NOT fire "whenever you surveil" triggers (CR 701.25c).

use mtg_engine::state::CardType;
use mtg_engine::{
    process_command, AbilityDefinition, CardDefinition, CardId, CardRegistry, Command, CounterType,
    Effect, EffectAmount, GameEvent, GameStateBuilder, ManaColor, ManaCost, ObjectSpec, PlayerId,
    PlayerTarget, Step, TriggerEvent, TriggeredAbilityDef, TypeLine, ZoneId,
};

// ── Helpers ───────────────────────────────────────────────────────────────────

fn find_object(state: &mtg_engine::GameState, name: &str) -> mtg_engine::ObjectId {
    state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{}' not found", name))
}

fn find_by_name_in_zone(
    state: &mtg_engine::GameState,
    name: &str,
    zone: ZoneId,
) -> Option<mtg_engine::ObjectId> {
    state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name && obj.zone == zone)
        .map(|(id, _)| *id)
}

/// Pass priority for all listed players once.
fn pass_all(
    state: mtg_engine::GameState,
    players: &[PlayerId],
) -> (mtg_engine::GameState, Vec<GameEvent>) {
    let mut all_events = Vec::new();
    let mut current = state;
    for &pl in players {
        let (s, ev) = process_command(current, Command::PassPriority { player: pl })
            .unwrap_or_else(|e| panic!("PassPriority by {:?} failed: {:?}", pl, e));
        current = s;
        all_events.extend(ev);
    }
    (current, all_events)
}

/// Count objects in a zone.
fn count_in_zone(
    state: &mtg_engine::GameState,
    player: PlayerId,
    zone_fn: impl Fn(PlayerId) -> ZoneId,
) -> usize {
    let zone = zone_fn(player);
    state
        .objects
        .values()
        .filter(|obj| obj.zone == zone)
        .count()
}

/// Build a "Surveil N" sorcery card definition (no targets).
fn surveil_spell_def(n: u32) -> CardDefinition {
    CardDefinition {
        card_id: CardId(format!("surveil-{}", n)),
        name: format!("Surveil {}", n),
        mana_cost: Some(ManaCost {
            blue: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Sorcery].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: format!("Surveil {}.", n),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::Surveil {
                player: PlayerTarget::Controller,
                count: EffectAmount::Fixed(n as i32),
            },
            targets: vec![],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}

/// Build a "Surveil N, then draw a card." sorcery card definition.
fn surveil_then_draw_def(n: u32) -> CardDefinition {
    CardDefinition {
        card_id: CardId(format!("surveil-draw-{}", n)),
        name: format!("Surveil Draw {}", n),
        mana_cost: Some(ManaCost {
            blue: 1,
            generic: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Sorcery].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: format!("Surveil {}, then draw a card.", n),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::Sequence(vec![
                Effect::Surveil {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(n as i32),
                },
                Effect::DrawCards {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(1),
                },
            ]),
            targets: vec![],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}

// ── Test 1: Basic surveil — cards move to graveyard ──────────────────────────

#[test]
/// CR 701.25a — Surveil N moves the top N cards of the library to the graveyard.
/// Deterministic fallback: all looked-at cards go to graveyard.
fn test_surveil_basic_cards_go_to_graveyard() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let def = surveil_spell_def(2);
    let registry = CardRegistry::new(vec![def]);

    let mut builder = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(
            ObjectSpec::card(p1, "Surveil 2")
                .with_card_id(CardId("surveil-2".to_string()))
                .with_types(vec![CardType::Sorcery])
                .with_mana_cost(ManaCost {
                    blue: 1,
                    ..Default::default()
                })
                .in_zone(ZoneId::Hand(p1)),
        )
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .with_registry(registry);

    for i in 0..5 {
        builder = builder.object(
            ObjectSpec::card(p1, &format!("Library Card {}", i)).in_zone(ZoneId::Library(p1)),
        );
    }

    let state = builder.build().unwrap();

    let initial_lib_count = count_in_zone(&state, p1, ZoneId::Library);
    let initial_grave_count = count_in_zone(&state, p1, ZoneId::Graveyard);

    let mut state = state;
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Blue, 1);

    let spell_id = find_object(&state, "Surveil 2");

    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: spell_id,
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
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    )
    .unwrap();

    let (state, events) = pass_all(state, &[p1, p2]);

    // CR 701.25a: 2 cards should have moved from library to graveyard.
    let final_lib_count = count_in_zone(&state, p1, ZoneId::Library);
    let final_grave_count = count_in_zone(&state, p1, ZoneId::Graveyard);

    assert_eq!(
        final_lib_count,
        initial_lib_count - 2,
        "Library should have 2 fewer cards after Surveil 2"
    );
    // Graveyard gains 2 library cards + the spell itself goes to graveyard.
    assert!(
        final_grave_count >= initial_grave_count + 2,
        "Graveyard should have at least 2 more cards after Surveil 2; before={}, after={}",
        initial_grave_count,
        final_grave_count
    );

    // CR 701.25: Surveilled event should be emitted.
    let surveilled = events
        .iter()
        .find(|e| matches!(e, GameEvent::Surveilled { player, .. } if *player == p1));
    assert!(
        surveilled.is_some(),
        "Surveilled event must be emitted for p1"
    );

    if let Some(GameEvent::Surveilled { count, .. }) = surveilled {
        assert_eq!(*count, 2, "Surveil count must be 2");
    }
}

// ── Test 2: Surveil 0 produces no event ──────────────────────────────────────

#[test]
/// CR 701.25c — Surveil 0 produces no event. Library and graveyard are unchanged.
fn test_surveil_zero_no_event() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let def = surveil_spell_def(0);
    let registry = CardRegistry::new(vec![def]);

    let mut builder = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(
            ObjectSpec::card(p1, "Surveil 0")
                .with_card_id(CardId("surveil-0".to_string()))
                .with_types(vec![CardType::Sorcery])
                .with_mana_cost(ManaCost {
                    blue: 1,
                    ..Default::default()
                })
                .in_zone(ZoneId::Hand(p1)),
        )
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .with_registry(registry);

    for i in 0..3 {
        builder = builder
            .object(ObjectSpec::card(p1, &format!("Lib Card {}", i)).in_zone(ZoneId::Library(p1)));
    }

    let state = builder.build().unwrap();

    let initial_lib_count = count_in_zone(&state, p1, ZoneId::Library);
    let initial_grave_count = count_in_zone(&state, p1, ZoneId::Graveyard);

    let mut state = state;
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Blue, 1);

    let spell_id = find_object(&state, "Surveil 0");

    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: spell_id,
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
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    )
    .unwrap();

    let (state, events) = pass_all(state, &[p1, p2]);

    // CR 701.25c: no Surveilled event.
    let surveilled = events
        .iter()
        .find(|e| matches!(e, GameEvent::Surveilled { .. }));
    assert!(
        surveilled.is_none(),
        "Surveilled event must NOT be emitted when surveilling 0; events: {:?}",
        events
            .iter()
            .filter(|e| matches!(e, GameEvent::Surveilled { .. }))
            .collect::<Vec<_>>()
    );

    // Library size unchanged (spell itself still goes to graveyard, but no library cards move).
    let final_lib_count = count_in_zone(&state, p1, ZoneId::Library);
    assert_eq!(
        final_lib_count, initial_lib_count,
        "Library size must be unchanged after Surveil 0; before={}, after={}",
        initial_lib_count, final_lib_count
    );

    // Graveyard only gains the spell itself, not any library cards.
    let final_grave_count = count_in_zone(&state, p1, ZoneId::Graveyard);
    assert_eq!(
        final_grave_count,
        initial_grave_count + 1, // +1 for the spell itself
        "Graveyard should only gain the spell itself after Surveil 0"
    );
}

// ── Test 3: Surveil with empty library still emits event ─────────────────────

#[test]
/// CR 701.25d — Surveil with an empty library still emits Surveilled event
/// with count=0 (surveil happened, but no cards were available to look at).
/// This differs from Surveil 0, where no event is emitted at all.
fn test_surveil_empty_library_still_emits_event() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let def = surveil_spell_def(2);
    let registry = CardRegistry::new(vec![def]);

    // No library cards added — empty library.
    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(
            ObjectSpec::card(p1, "Surveil 2")
                .with_card_id(CardId("surveil-2".to_string()))
                .with_types(vec![CardType::Sorcery])
                .with_mana_cost(ManaCost {
                    blue: 1,
                    ..Default::default()
                })
                .in_zone(ZoneId::Hand(p1)),
        )
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .with_registry(registry)
        .build()
        .unwrap();

    let mut state = state;
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Blue, 1);

    let spell_id = find_object(&state, "Surveil 2");

    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: spell_id,
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
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    )
    .unwrap();

    let (_, events) = pass_all(state, &[p1, p2]);

    // CR 701.25d: event fires even when library is empty (0 cards actually moved).
    let surveilled = events
        .iter()
        .find(|e| matches!(e, GameEvent::Surveilled { player, .. } if *player == p1));
    assert!(
        surveilled.is_some(),
        "Surveilled event must fire even with empty library (CR 701.25d)"
    );

    if let Some(GameEvent::Surveilled { count, .. }) = surveilled {
        assert_eq!(
            *count, 0,
            "Surveil count must be 0 when library is empty; got {}",
            count
        );
    }
}

// ── Test 4: Surveil with fewer than N cards ───────────────────────────────────

#[test]
/// CR 701.25d — If the library has fewer than N cards, the player surveys
/// whatever is available. The event fires with the actual count, not N.
fn test_surveil_library_fewer_than_n() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let def = surveil_spell_def(3);
    let registry = CardRegistry::new(vec![def]);

    let mut builder = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(
            ObjectSpec::card(p1, "Surveil 3")
                .with_card_id(CardId("surveil-3".to_string()))
                .with_types(vec![CardType::Sorcery])
                .with_mana_cost(ManaCost {
                    blue: 1,
                    ..Default::default()
                })
                .in_zone(ZoneId::Hand(p1)),
        )
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .with_registry(registry);

    // Only 1 card in library (surveilling 3 but only 1 available).
    builder = builder.object(ObjectSpec::card(p1, "Solo Card").in_zone(ZoneId::Library(p1)));

    let state = builder.build().unwrap();
    let mut state = state;
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Blue, 1);

    let spell_id = find_object(&state, "Surveil 3");

    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: spell_id,
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
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    )
    .unwrap();

    let (state, events) = pass_all(state, &[p1, p2]);

    // CR 701.25d: event fires with actual count (1, not 3).
    let surveilled = events
        .iter()
        .find(|e| matches!(e, GameEvent::Surveilled { player, .. } if *player == p1));
    assert!(
        surveilled.is_some(),
        "Surveilled event must be emitted when library has fewer than N cards"
    );

    if let Some(GameEvent::Surveilled { count, .. }) = surveilled {
        assert_eq!(
            *count, 1,
            "Surveil count must be 1 (actual cards available), not 3 (requested); got {}",
            count
        );
    }

    // Library is now empty.
    let final_lib_count = count_in_zone(&state, p1, ZoneId::Library);
    assert_eq!(
        final_lib_count, 0,
        "Library must be empty after surveilling the only card"
    );

    // The one card moved to graveyard.
    assert!(
        find_by_name_in_zone(&state, "Solo Card", ZoneId::Graveyard(p1)).is_some(),
        "Solo Card must be in p1's graveyard after being surveilled"
    );
}

// ── Test 5: Surveil then draw sequence ────────────────────────────────────────

#[test]
/// CR 701.25a — Common card pattern: "Surveil N, then draw a card."
/// Surveilled event must precede CardDrawn event, and draw uses the
/// card below the surveilled cards (3rd card in a 5-card library after
/// surveilling 2).
fn test_surveil_then_draw_sequence() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let def = surveil_then_draw_def(2);
    let registry = CardRegistry::new(vec![def]);

    let mut builder = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(
            ObjectSpec::card(p1, "Surveil Draw 2")
                .with_card_id(CardId("surveil-draw-2".to_string()))
                .with_types(vec![CardType::Sorcery])
                .with_mana_cost(ManaCost {
                    blue: 1,
                    generic: 1,
                    ..Default::default()
                })
                .in_zone(ZoneId::Hand(p1)),
        )
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .with_registry(registry);

    for i in 0..5 {
        builder = builder
            .object(ObjectSpec::card(p1, &format!("Lib Card {}", i)).in_zone(ZoneId::Library(p1)));
    }

    let state = builder.build().unwrap();

    let initial_lib_count = count_in_zone(&state, p1, ZoneId::Library);
    let initial_hand_count = count_in_zone(&state, p1, ZoneId::Hand);

    let mut state = state;
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Blue, 2); // 2 mana: 1B + 1 generic

    let spell_id = find_object(&state, "Surveil Draw 2");

    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: spell_id,
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
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    )
    .unwrap();

    let (state, events) = pass_all(state, &[p1, p2]);

    // Surveilled event must appear before CardDrawn.
    let surveilled_pos = events
        .iter()
        .position(|e| matches!(e, GameEvent::Surveilled { player, .. } if *player == p1))
        .expect("Surveilled event must be emitted");

    let card_drawn_pos = events
        .iter()
        .position(|e| matches!(e, GameEvent::CardDrawn { player, .. } if *player == p1))
        .expect("CardDrawn event must be emitted");

    assert!(
        surveilled_pos < card_drawn_pos,
        "Surveilled (pos {}) must precede CardDrawn (pos {})",
        surveilled_pos,
        card_drawn_pos
    );

    // Library shrinks by 2 (surveilled) + 1 (drawn) = 3 total.
    let final_lib_count = count_in_zone(&state, p1, ZoneId::Library);
    assert_eq!(
        final_lib_count,
        initial_lib_count - 3,
        "Library must shrink by 3 (2 surveilled + 1 drawn)"
    );

    // Hand gains 1 drawn card (spell itself was cast, so net count stays same or +1).
    let final_hand_count = count_in_zone(&state, p1, ZoneId::Hand);
    // Spell was cast (removed from hand), then +1 drawn: net = initial - 1 + 1 = initial.
    // But the hand count at cast time was initial_hand_count - 1 (spell was removed).
    // After resolution, +1 card drawn, so final = initial_hand_count - 1 + 1 = initial_hand_count.
    assert_eq!(
        final_hand_count, initial_hand_count,
        "Hand size should be unchanged (cast one, drew one)"
    );

    // 2 library cards moved to graveyard.
    let final_grave_count = count_in_zone(&state, p1, ZoneId::Graveyard);
    assert!(
        final_grave_count >= 2, // at least 2 surveilled cards + spell itself in graveyard
        "Graveyard must contain at least 2 surveilled cards; got {}",
        final_grave_count
    );
}

// ── Test 6: "Whenever you surveil" trigger fires ──────────────────────────────

#[test]
/// CR 701.25d — "Whenever you surveil" trigger fires when a player surveils.
/// The trigger fires once per surveil action, regardless of how many cards
/// were put into the graveyard.
/// Validated via a creature that gains a +1/+1 counter on surveil.
fn test_whenever_you_surveil_trigger() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let def = surveil_spell_def(2);
    let registry = CardRegistry::new(vec![def]);

    // Build a creature with WheneverYouSurveil -> AddCounter(+1/+1).
    let surveil_creature = ObjectSpec::creature(p1, "Spybug", 1, 1)
        .with_triggered_ability(TriggeredAbilityDef {
            etb_filter: None,
            death_filter: None,
            combat_damage_filter: None,
            targets: vec![],
            trigger_on: TriggerEvent::ControllerSurveils,
            intervening_if: None,
            description: "Whenever you surveil, Spybug gets a +1/+1 counter (CR 701.25d)"
                .to_string(),
            effect: Some(Effect::AddCounter {
                target: mtg_engine::CardEffectTarget::Source,
                counter: CounterType::PlusOnePlusOne,
                count: 1,
            }),
        })
        .in_zone(ZoneId::Battlefield);

    let mut builder = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(surveil_creature)
        .object(
            ObjectSpec::card(p1, "Surveil 2")
                .with_card_id(CardId("surveil-2".to_string()))
                .with_types(vec![CardType::Sorcery])
                .with_mana_cost(ManaCost {
                    blue: 1,
                    ..Default::default()
                })
                .in_zone(ZoneId::Hand(p1)),
        )
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .with_registry(registry);

    for i in 0..3 {
        builder = builder
            .object(ObjectSpec::card(p1, &format!("Lib Card {}", i)).in_zone(ZoneId::Library(p1)));
    }

    let state = builder.build().unwrap();
    let mut state = state;
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Blue, 1);

    let spell_id = find_object(&state, "Surveil 2");

    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: spell_id,
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
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    )
    .unwrap();

    // First pass_all: resolves the surveil spell + queues trigger.
    let (state, _) = pass_all(state.clone(), &[p1, p2]);

    // Second pass_all: resolves the "whenever you surveil" trigger.
    let (state, _) = pass_all(state, &[p1, p2]);

    // CR 701.25d: Spybug should have a +1/+1 counter.
    let spybug = state
        .objects
        .values()
        .find(|obj| obj.characteristics.name == "Spybug" && obj.zone == ZoneId::Battlefield)
        .expect("Spybug must still be on the battlefield");

    let counter_count = spybug
        .counters
        .get(&CounterType::PlusOnePlusOne)
        .copied()
        .unwrap_or(0);

    assert_eq!(
        counter_count, 1,
        "Spybug must have exactly 1 +1/+1 counter after surveil trigger resolved; got {}",
        counter_count
    );
}

// ── Test 7: Surveil 0 does not fire "whenever you surveil" trigger ────────────

#[test]
/// CR 701.25c — Surveil 0 does not emit a Surveilled event, so "whenever you surveil"
/// triggers do NOT fire.
fn test_surveil_zero_does_not_fire_trigger() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let def = surveil_spell_def(0);
    let registry = CardRegistry::new(vec![def]);

    // Build a creature with WheneverYouSurveil -> AddCounter(+1/+1).
    let surveil_creature = ObjectSpec::creature(p1, "Spybug", 1, 1)
        .with_triggered_ability(TriggeredAbilityDef {
            etb_filter: None,
            death_filter: None,
            combat_damage_filter: None,
            targets: vec![],
            trigger_on: TriggerEvent::ControllerSurveils,
            intervening_if: None,
            description: "Whenever you surveil, Spybug gets a +1/+1 counter (CR 701.25d)"
                .to_string(),
            effect: Some(Effect::AddCounter {
                target: mtg_engine::CardEffectTarget::Source,
                counter: CounterType::PlusOnePlusOne,
                count: 1,
            }),
        })
        .in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(surveil_creature)
        .object(
            ObjectSpec::card(p1, "Surveil 0")
                .with_card_id(CardId("surveil-0".to_string()))
                .with_types(vec![CardType::Sorcery])
                .with_mana_cost(ManaCost {
                    blue: 1,
                    ..Default::default()
                })
                .in_zone(ZoneId::Hand(p1)),
        )
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .with_registry(registry)
        .build()
        .unwrap();

    let mut state = state;
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Blue, 1);

    let spell_id = find_object(&state, "Surveil 0");

    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: spell_id,
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
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    )
    .unwrap();

    // pass_all twice to ensure any would-be trigger resolves.
    let (state, events1) = pass_all(state.clone(), &[p1, p2]);
    let (state, events2) = pass_all(state, &[p1, p2]);

    let all_events: Vec<_> = events1.iter().chain(events2.iter()).collect();

    // CR 701.25c: no Surveilled event should have been emitted.
    let surveilled = all_events
        .iter()
        .find(|e| matches!(e, GameEvent::Surveilled { .. }));
    assert!(
        surveilled.is_none(),
        "Surveilled event must NOT fire for Surveil 0"
    );

    // CR 701.25c: Spybug must have NO +1/+1 counters.
    let spybug = state
        .objects
        .values()
        .find(|obj| obj.characteristics.name == "Spybug" && obj.zone == ZoneId::Battlefield)
        .expect("Spybug must still be on the battlefield");

    let counter_count = spybug
        .counters
        .get(&CounterType::PlusOnePlusOne)
        .copied()
        .unwrap_or(0);

    assert_eq!(
        counter_count, 0,
        "Spybug must have NO +1/+1 counters when Surveil 0 was cast (trigger must not fire)"
    );
}
