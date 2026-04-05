//! Extra turn tests.
//!
//! Covers:
//! - CR 500.7: Extra turn LIFO ordering, designated player, resumption, multi-stack
//!   (existing tests).
//! - Effect::ExtraTurn dispatch: grant N extra turns to controller or opponent via Effect.
//! - GiftType::ExtraTurn: CR 702.174g — gift an extra turn to an opponent.
//! - self_exile_on_resolution: Temporal Trespass/Mastery exile themselves on resolution.
//! - self_shuffle_on_resolution: Nexus of Fate shuffles into library on resolution.

use im::Vector;
use mtg_engine::cards::card_definition::GiftType;
use mtg_engine::rules::engine::process_command;
use mtg_engine::{
    AbilityDefinition, AdditionalCost, CardDefinition, CardId, CardRegistry, CardType, Command,
    Effect, EffectAmount, GameEvent, GameState, GameStateBuilder, KeywordAbility, ManaColor,
    ManaCost, ObjectSpec, PlayerId, PlayerTarget, Step, TypeLine, ZoneId,
};

fn pass(state: GameState, player: PlayerId) -> (GameState, Vec<GameEvent>) {
    process_command(state, Command::PassPriority { player }).unwrap()
}

/// Build a 4-player state with enough library cards that nobody decks out.
fn four_player_with_libraries(step: Step) -> GameState {
    let mut builder = GameStateBuilder::four_player().at_step(step);
    for pid in 1..=4 {
        let player = PlayerId(pid);
        for i in 0..10 {
            builder = builder.object(
                ObjectSpec::card(player, &format!("Card {} P{}", i, pid))
                    .in_zone(ZoneId::Library(player)),
            );
        }
    }
    builder.build().unwrap()
}

/// Complete the current turn by passing priority until the turn number increases.
fn complete_turn(mut state: GameState) -> (GameState, Vec<GameEvent>) {
    let mut all_events = Vec::new();
    let start_turn = state.turn.turn_number;
    loop {
        let holder = state.turn.priority_holder.expect("no priority holder");
        let (new_state, events) = pass(state, holder);
        all_events.extend(events);
        state = new_state;
        if state.turn.turn_number > start_turn {
            return (state, all_events);
        }
    }
}

#[test]
/// Extra turns are LIFO — most recently added goes first
fn test_extra_turns_lifo() {
    let mut state = four_player_with_libraries(Step::End);

    // Add extra turns: P2 first, then P3
    state.turn.extra_turns = Vector::from(vec![PlayerId(2), PlayerId(3)]);

    // Complete current turn (P1's End step)
    let (state, _) = complete_turn(state);

    // P3 should get the next turn (LIFO — last added goes first)
    assert_eq!(state.turn.active_player, PlayerId(3));
}

#[test]
/// Extra turn: designated player becomes active
fn test_extra_turn_designated_player_active() {
    let mut state = four_player_with_libraries(Step::End);

    state.turn.extra_turns.push_back(PlayerId(3));

    let (state, events) = complete_turn(state);

    assert_eq!(state.turn.active_player, PlayerId(3));
    assert!(events.iter().any(|e| matches!(
        e,
        GameEvent::TurnStarted { player, .. } if *player == PlayerId(3)
    )));
}

#[test]
/// After an extra turn, normal turn order resumes
fn test_extra_turn_normal_order_resumes() {
    let mut state = four_player_with_libraries(Step::End);

    // P1's turn, with an extra turn for P3 queued
    state.turn.extra_turns.push_back(PlayerId(3));

    // Complete P1's turn → P3's extra turn
    let (state, _) = complete_turn(state);
    assert_eq!(state.turn.active_player, PlayerId(3));

    // Complete P3's extra turn → normal order resumes (P2 is next after P1)
    let (state, _) = complete_turn(state);
    assert_eq!(state.turn.active_player, PlayerId(2));
}

#[test]
/// Multiple extra turns stack LIFO
fn test_multiple_extra_turns_stack() {
    let mut state = four_player_with_libraries(Step::End);

    // Stack three extra turns
    state.turn.extra_turns.push_back(PlayerId(2));
    state.turn.extra_turns.push_back(PlayerId(3));
    state.turn.extra_turns.push_back(PlayerId(4));

    // P4 goes first (LIFO)
    let (state, _) = complete_turn(state);
    assert_eq!(state.turn.active_player, PlayerId(4));

    // Then P3
    let (state, _) = complete_turn(state);
    assert_eq!(state.turn.active_player, PlayerId(3));

    // Then P2
    let (state, _) = complete_turn(state);
    assert_eq!(state.turn.active_player, PlayerId(2));

    // Then back to normal order (P2 is after P1)
    let (state, _) = complete_turn(state);
    assert_eq!(state.turn.active_player, PlayerId(2));
}

// ── Effect::ExtraTurn dispatch tests ──────────────────────────────────────────

/// Synthetic instant that grants one extra turn to its controller.
/// Used to test Effect::ExtraTurn { player: Controller, count: Fixed(1) }.
fn extra_turn_instant_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("extra-turn-instant".to_string()),
        name: "Extra Turn Instant".to_string(),
        mana_cost: Some(ManaCost {
            blue: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Instant].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Take an extra turn after this one.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::ExtraTurn {
                player: PlayerTarget::Controller,
                count: EffectAmount::Fixed(1),
            },
            targets: vec![],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}

/// Synthetic instant that grants two extra turns to its controller (Teferi -10 analog).
fn two_extra_turns_instant_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("two-extra-turns-instant".to_string()),
        name: "Two Extra Turns Instant".to_string(),
        mana_cost: Some(ManaCost {
            blue: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Instant].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Take two extra turns after this one.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::ExtraTurn {
                player: PlayerTarget::Controller,
                count: EffectAmount::Fixed(2),
            },
            targets: vec![],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}

/// Synthetic instant that grants one extra turn with "Gift an extra turn" (GiftType::ExtraTurn).
fn gift_extra_turn_instant_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("gift-extra-turn-instant".to_string()),
        name: "Gift Extra Turn Instant".to_string(),
        mana_cost: Some(ManaCost { blue: 1, ..Default::default() }),
        types: TypeLine {
            card_types: [CardType::Instant].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Gift an extra turn (You may choose an opponent as you cast this spell. If you do, that player takes an extra turn after this one.)".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Gift),
            AbilityDefinition::Gift {
                gift_type: GiftType::ExtraTurn,
            },
            AbilityDefinition::Spell {
                effect: Effect::Nothing,
                targets: vec![],
                modes: None,
                cant_be_countered: false,
            },
        ],
        ..Default::default()
    }
}

/// Synthetic sorcery with self_exile_on_resolution and ExtraTurn effect.
/// Models Temporal Trespass behavior.
fn temporal_trespass_analog_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("temporal-trespass-test".to_string()),
        name: "Temporal Trespass Test".to_string(),
        mana_cost: Some(ManaCost {
            blue: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Sorcery].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Take an extra turn after this one. Exile this spell.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::ExtraTurn {
                player: PlayerTarget::Controller,
                count: EffectAmount::Fixed(1),
            },
            targets: vec![],
            modes: None,
            cant_be_countered: false,
        }],
        self_exile_on_resolution: true,
        ..Default::default()
    }
}

/// Synthetic instant with self_shuffle_on_resolution and ExtraTurn effect.
/// Models Nexus of Fate behavior.
fn nexus_of_fate_analog_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("nexus-of-fate-test".to_string()),
        name: "Nexus of Fate Test".to_string(),
        mana_cost: Some(ManaCost {
            blue: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Instant].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Take an extra turn after this one. Shuffle this into library.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::ExtraTurn {
                player: PlayerTarget::Controller,
                count: EffectAmount::Fixed(1),
            },
            targets: vec![],
            modes: None,
            cant_be_countered: false,
        }],
        self_shuffle_on_resolution: true,
        ..Default::default()
    }
}

/// Helper: build a 2-player game with the given spell in p1's hand.
/// P1 has {U} mana available. Returns (state, p1, p2, card_object_id).
/// If `extra_keywords` is non-empty, they are added to the spell object's characteristics.
fn setup_extra_turn_state_with_keywords(
    spell_def: CardDefinition,
    extra_keywords: Vec<KeywordAbility>,
) -> (GameState, PlayerId, PlayerId, mtg_engine::ObjectId) {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);
    let card_id = spell_def.card_id.clone();
    let registry = CardRegistry::new(vec![spell_def]);

    let mut spell_spec = ObjectSpec::card(p1, "Extra Turn Spell")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(card_id)
        .with_mana_cost(ManaCost {
            blue: 1,
            ..Default::default()
        })
        .with_types(vec![CardType::Instant]);
    for kw in extra_keywords {
        spell_spec = spell_spec.with_keyword(kw);
    }

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(spell_spec)
        // Library cards so nobody decks out
        .object(
            ObjectSpec::card(p1, "P1 Lib")
                .in_zone(ZoneId::Library(p1))
                .with_mana_cost(ManaCost::default()),
        )
        .object(
            ObjectSpec::card(p2, "P2 Lib")
                .in_zone(ZoneId::Library(p2))
                .with_mana_cost(ManaCost::default()),
        )
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let mut state = state;
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Blue, 1);
    state.turn.priority_holder = Some(p1);

    let spell_obj_id = state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == "Extra Turn Spell")
        .map(|(id, _)| *id)
        .expect("spell should be in hand");

    (state, p1, p2, spell_obj_id)
}

/// Convenience wrapper with no extra keywords.
fn setup_extra_turn_state(
    spell_def: CardDefinition,
) -> (GameState, PlayerId, PlayerId, mtg_engine::ObjectId) {
    setup_extra_turn_state_with_keywords(spell_def, vec![])
}

/// Helper: cast an extra-turn spell (no targets, no additional costs).
fn cast_extra_turn_spell(
    state: GameState,
    player: PlayerId,
    card_obj_id: mtg_engine::ObjectId,
) -> (GameState, Vec<GameEvent>) {
    process_command(
        state,
        Command::CastSpell {
            player,
            card: card_obj_id,
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
    .unwrap_or_else(|e| panic!("CastSpell failed: {:?}", e))
}

/// Helper: cast a gift extra-turn spell choosing the given opponent.
fn cast_gift_extra_turn(
    state: GameState,
    player: PlayerId,
    card_obj_id: mtg_engine::ObjectId,
    gift_opponent: PlayerId,
) -> (GameState, Vec<GameEvent>) {
    process_command(
        state,
        Command::CastSpell {
            player,
            card: card_obj_id,
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
            additional_costs: vec![AdditionalCost::Gift {
                opponent: gift_opponent,
            }],
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    )
    .unwrap_or_else(|e| panic!("CastSpell (gift) failed: {:?}", e))
}

/// Helper: resolve the top of the stack by passing priority for all players.
fn resolve_top(mut state: GameState, players: &[PlayerId]) -> (GameState, Vec<GameEvent>) {
    let mut all_events = vec![];
    for &pl in players {
        let (s, evs) = process_command(state, Command::PassPriority { player: pl }).unwrap();
        all_events.extend(evs);
        state = s;
        if state.stack_objects.is_empty() {
            break;
        }
    }
    (state, all_events)
}

/// CR 500.7 — Effect::ExtraTurn grants one extra turn to controller.
/// The turn queue should contain the controller's PlayerId after resolution.
#[test]
fn test_effect_extra_turn_basic() {
    let (state, p1, p2, spell_id) = setup_extra_turn_state(extra_turn_instant_def());

    // Cast the spell.
    let (state, _) = cast_extra_turn_spell(state, p1, spell_id);
    assert_eq!(state.stack_objects.len(), 1, "spell should be on stack");

    // Resolve: p1 then p2 pass priority.
    let (state, events) = resolve_top(state, &[p1, p2]);

    assert!(
        state.stack_objects.is_empty(),
        "stack should be empty after resolution"
    );

    // Extra turn queue should have p1 in it.
    assert_eq!(
        state.turn.extra_turns,
        Vector::from(vec![p1]),
        "CR 500.7: controller should have one extra turn queued"
    );

    // ExtraTurnAdded event should have been emitted.
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::ExtraTurnAdded { player } if *player == p1)),
        "CR 500.7: ExtraTurnAdded event should be emitted for the controller"
    );
}

/// CR 500.7 — Effect::ExtraTurn with count=2 adds two extra turns (Teferi -10 analog).
/// Both turns are added for the same player; LIFO means the second push_back is taken first.
#[test]
fn test_effect_extra_turn_two_turns() {
    let (state, p1, p2, spell_id) = setup_extra_turn_state(two_extra_turns_instant_def());

    let (state, _) = cast_extra_turn_spell(state, p1, spell_id);
    let (state, events) = resolve_top(state, &[p1, p2]);

    assert!(state.stack_objects.is_empty());

    // Should have two entries for p1 in the extra_turns queue.
    assert_eq!(
        state.turn.extra_turns.len(),
        2,
        "CR 500.7: two extra turns should be queued for count=2"
    );
    assert!(
        state.turn.extra_turns.iter().all(|&pid| pid == p1),
        "CR 500.7: both extra turns should be for the controller"
    );

    // Two ExtraTurnAdded events.
    let added_events: Vec<_> = events
        .iter()
        .filter(|e| matches!(e, GameEvent::ExtraTurnAdded { player } if *player == p1))
        .collect();
    assert_eq!(
        added_events.len(),
        2,
        "CR 500.7: two ExtraTurnAdded events for count=2"
    );
}

/// CR 702.174g — GiftType::ExtraTurn: chosen opponent takes an extra turn.
#[test]
fn test_gift_extra_turn() {
    let (state, p1, p2, spell_id) = setup_extra_turn_state_with_keywords(
        gift_extra_turn_instant_def(),
        vec![KeywordAbility::Gift],
    );

    // Cast choosing p2 as gift recipient.
    let (state, _) = cast_gift_extra_turn(state, p1, spell_id, p2);
    assert_eq!(state.stack_objects.len(), 1, "spell should be on stack");

    // Resolve.
    let (state, events) = resolve_top(state, &[p1, p2]);
    assert!(state.stack_objects.is_empty());

    // p2 should have an extra turn queued (the gift recipient, not the caster).
    assert_eq!(
        state.turn.extra_turns,
        Vector::from(vec![p2]),
        "CR 702.174g: the gift recipient (p2) should have an extra turn queued"
    );

    // ExtraTurnAdded event should name p2.
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::ExtraTurnAdded { player } if *player == p2)),
        "CR 702.174g: ExtraTurnAdded event should name the gift recipient"
    );
}

/// self_exile_on_resolution — Temporal Trespass analog: card goes to exile, not graveyard.
#[test]
fn test_self_exile_on_resolution() {
    let (state, p1, p2, spell_id) = setup_extra_turn_state(temporal_trespass_analog_def());

    let (state, _) = cast_extra_turn_spell(state, p1, spell_id);
    let (state, _) = resolve_top(state, &[p1, p2]);

    // The spell card should now be in exile, NOT in the graveyard.
    let in_exile = state
        .objects
        .values()
        .any(|o| o.zone == ZoneId::Exile && o.characteristics.name == "Extra Turn Spell");
    let in_graveyard = state
        .objects
        .values()
        .any(|o| o.zone == ZoneId::Graveyard(p1) && o.characteristics.name == "Extra Turn Spell");

    assert!(
        in_exile,
        "self_exile_on_resolution: spell should be in exile after resolving"
    );
    assert!(
        !in_graveyard,
        "self_exile_on_resolution: spell should NOT be in graveyard"
    );

    // The extra turn should still have been granted.
    assert!(
        !state.turn.extra_turns.is_empty(),
        "extra turn should still be granted when spell self-exiles"
    );
}

/// self_shuffle_on_resolution — Nexus of Fate analog: card goes to library, not graveyard.
#[test]
fn test_self_shuffle_on_resolution() {
    let (state, p1, p2, spell_id) = setup_extra_turn_state(nexus_of_fate_analog_def());

    let (state, _) = cast_extra_turn_spell(state, p1, spell_id);
    let (state, _) = resolve_top(state, &[p1, p2]);

    // The spell card should now be in the owner's library, NOT in the graveyard.
    let in_library = state
        .objects
        .values()
        .any(|o| o.zone == ZoneId::Library(p1) && o.characteristics.name == "Extra Turn Spell");
    let in_graveyard = state
        .objects
        .values()
        .any(|o| o.zone == ZoneId::Graveyard(p1) && o.characteristics.name == "Extra Turn Spell");

    assert!(
        in_library,
        "self_shuffle_on_resolution: spell should be in library after resolving (Nexus of Fate)"
    );
    assert!(
        !in_graveyard,
        "self_shuffle_on_resolution: spell should NOT be in graveyard"
    );

    // The extra turn should still have been granted.
    assert!(
        !state.turn.extra_turns.is_empty(),
        "extra turn should still be granted when spell shuffles into library"
    );
}

/// CR 500.7 — Extra turn resolves and is taken: cast spell → resolve → take extra turn.
/// After taking the extra turn, normal turn order resumes.
#[test]
fn test_effect_extra_turn_resolves_and_taken() {
    // Use 4-player setup so turn order is P1 → P2 → P3 → P4 → P1...
    let mut state = four_player_with_libraries(Step::PreCombatMain);
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    // Add extra turn spell to p1's hand via state manipulation.
    // Push the extra turn for p1 directly onto extra_turns.
    state.turn.extra_turns.push_back(p1);

    // Complete p1's current turn. The extra turn should fire next.
    let (state, _) = complete_turn(state);
    assert_eq!(
        state.turn.active_player, p1,
        "CR 500.7: p1 takes the extra turn"
    );

    // After p1's extra turn completes, normal order resumes (p2 is next after p1).
    let (state, _) = complete_turn(state);
    assert_eq!(
        state.turn.active_player, p2,
        "CR 500.7: after extra turn, normal order resumes (p2 next after p1)"
    );
}
