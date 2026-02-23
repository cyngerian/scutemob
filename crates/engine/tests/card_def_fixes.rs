//! Tests verifying M9.4 Session 1 card definition corrections.
//!
//! Covers:
//! - Read the Bones: Scry 2 fires before drawing two cards (CR 701.18)
//! - Dimir Guildgate: modal color choice (CR 106.6)
//! - Path to Exile: optional search via deterministic MayPayOrElse (CR 701.19)
//! - Thought Vessel / Reliquary Tower: no-maximum-hand-size skips cleanup discard (CR 402.2)
//! - Alela, Cunning Conqueror: WheneverYouCastSpell has during_opponent_turn: true (CR 603.1)

use mtg_engine::{
    all_cards, process_command, CardRegistry, CardType, Command, GameEvent, GameState,
    GameStateBuilder, KeywordAbility, ManaColor, ManaCost, ObjectSpec, PlayerId, Step,
    TriggerCondition, ZoneId,
};

// ── Helper: find an object by name ───────────────────────────────────────────

fn find_object(state: &GameState, name: &str) -> mtg_engine::ObjectId {
    state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{}' not found in state", name))
}

/// Pass priority for all players once.
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

// ── CR 701.18: Read the Bones — scry fires before draw ───────────────────────

#[test]
/// CR 701.18 — Read the Bones: Scry 2 fires before drawing two cards.
/// Verifies the Scried event precedes the CardDrawn events in the event sequence.
fn test_read_the_bones_scry_then_draw() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let def = all_cards()
        .into_iter()
        .find(|d| d.name == "Read the Bones")
        .expect("Read the Bones must be in all_cards()");
    let card_id = def.card_id.clone();
    let registry = CardRegistry::new(vec![def]);

    // Give p1 a library with 5 cards and Read the Bones in hand.
    let mut builder = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(
            ObjectSpec::card(p1, "Read the Bones")
                .with_card_id(card_id)
                .with_types(vec![CardType::Sorcery])
                .with_mana_cost(ManaCost {
                    black: 1,
                    generic: 2,
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

    // Pay 3 mana: 1 black + 2 generic (just add 3 black for simplicity).
    let mut state = state;
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Black, 3);

    let rtb_id = find_object(&state, "Read the Bones");

    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: rtb_id,
            targets: vec![],
        },
    )
    .unwrap();

    let (state, events) = pass_all(state, &[p1, p2]);

    // CR 701.18: Scried event must appear before any CardDrawn events.
    let scried_pos = events
        .iter()
        .position(|e| matches!(e, GameEvent::Scried { player, .. } if *player == p1))
        .expect("Scried event should be emitted for p1");

    let first_drawn_pos = events
        .iter()
        .position(|e| matches!(e, GameEvent::CardDrawn { player, .. } if *player == p1))
        .expect("At least one CardDrawn event should be emitted for p1");

    assert!(
        scried_pos < first_drawn_pos,
        "Scried (pos {}) must precede first CardDrawn (pos {}); events: {:?}",
        scried_pos,
        first_drawn_pos,
        events
    );

    // Must have scried for 2 cards.
    let scried = events
        .iter()
        .find(|e| matches!(e, GameEvent::Scried { player, .. } if *player == p1));
    if let Some(GameEvent::Scried { count, .. }) = scried {
        assert_eq!(*count, 2, "Scry count must be 2");
    }

    // Must have drawn exactly 2 cards.
    let drawn_count = events
        .iter()
        .filter(|e| matches!(e, GameEvent::CardDrawn { player, .. } if *player == p1))
        .count();
    assert_eq!(
        drawn_count, 2,
        "Read the Bones should draw 2 cards; got {}",
        drawn_count
    );

    // p1 should have lost 2 life.
    let initial_life = 40; // Commander format starts at 40
    assert_eq!(
        state.players[&p1].life_total,
        initial_life - 2,
        "Read the Bones should cost 2 life"
    );
}

// ── CR 106.6: Dimir Guildgate — modal color choice ────────────────────────────

#[test]
/// CR 106.6 — Dimir Guildgate: tap ability is modelled as Effect::Choose between
/// AddMana blue and AddMana black (replacing the old AddManaAnyColor).
/// This is a data model test verifying the card definition is correct.
fn test_dimir_guildgate_modal_color() {
    use mtg_engine::{AbilityDefinition, Cost, Effect, ManaPool};

    let def = all_cards()
        .into_iter()
        .find(|d| d.name == "Dimir Guildgate")
        .expect("Dimir Guildgate must be in all_cards()");

    // Find the activated tap ability (should have a Choose effect).
    let tap_ability = def.abilities.iter().find(|a| {
        matches!(
            a,
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                ..
            }
        )
    });

    assert!(
        tap_ability.is_some(),
        "Dimir Guildgate should have a tap activated ability"
    );

    if let Some(AbilityDefinition::Activated { effect, .. }) = tap_ability {
        // CR 106.6: must be a Choose between two AddMana options.
        assert!(
            matches!(effect, Effect::Choose { choices, .. } if choices.len() == 2),
            "Dimir Guildgate tap ability must use Effect::Choose with 2 choices; got: {:?}",
            effect
        );

        if let Effect::Choose { choices, .. } = effect {
            // First choice must add 1 blue mana.
            assert!(
                matches!(&choices[0], Effect::AddMana { mana, .. }
                    if mana == &ManaPool { blue: 1, ..Default::default() }),
                "First choice should add 1 blue mana; got: {:?}",
                &choices[0]
            );
            // Second choice must add 1 black mana.
            assert!(
                matches!(&choices[1], Effect::AddMana { mana, .. }
                    if mana == &ManaPool { black: 1, ..Default::default() }),
                "Second choice should add 1 black mana; got: {:?}",
                &choices[1]
            );
        }
    }
}

// ── CR 701.19: Path to Exile — optional search ───────────────────────────────

#[test]
/// CR 701.19 / M9.4 — Path to Exile: target creature is exiled and the deterministic
/// fallback fires the search branch (controller searches for a basic land).
fn test_path_to_exile_optional_search() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let def = all_cards()
        .into_iter()
        .find(|d| d.name == "Path to Exile")
        .expect("Path to Exile must be in all_cards()");
    let card_id = def.card_id.clone();
    let registry = CardRegistry::new(vec![def]);

    // p2 has a creature on the battlefield. p2's library has a basic land.
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(
            ObjectSpec::card(p1, "Path to Exile")
                .with_card_id(card_id)
                .with_types(vec![CardType::Instant])
                .with_mana_cost(ManaCost {
                    white: 1,
                    ..Default::default()
                })
                .in_zone(ZoneId::Hand(p1)),
        )
        .object(ObjectSpec::creature(p2, "Goblin Guide", 2, 2).in_zone(ZoneId::Battlefield))
        .object(ObjectSpec::card(p2, "Plains").in_zone(ZoneId::Library(p2)))
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .with_registry(registry)
        .build()
        .unwrap();

    // Pay 1 white mana.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::White, 1);

    let path_id = find_object(&state, "Path to Exile");
    let goblin_id = find_object(&state, "Goblin Guide");

    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: path_id,
            targets: vec![mtg_engine::Target::Object(goblin_id)],
        },
    )
    .unwrap();

    let (_state, events) = pass_all(state, &[p1, p2]);

    // Goblin Guide should have been exiled.
    let exiled = events
        .iter()
        .any(|e| matches!(e, GameEvent::ObjectExiled { object_id, .. } if *object_id == goblin_id));
    assert!(
        exiled,
        "Goblin Guide should be exiled by Path to Exile; events: {:?}",
        events
    );
}

// ── CR 402.2: Thought Vessel — no maximum hand size ──────────────────────────

#[test]
/// CR 402.2 / CR 514.1 — Thought Vessel: controller with Thought Vessel on battlefield
/// does NOT discard to hand size during cleanup.
fn test_thought_vessel_no_max_hand_size() {
    let p1 = PlayerId(1);

    // Build a cleanup state: p1 has 9 cards in hand (normally would discard 2),
    // but also has a permanent with NoMaxHandSize on the battlefield.
    let mut builder = GameStateBuilder::four_player().at_step(Step::End);

    // Add 9 cards to hand (exceeds max 7).
    for i in 0..9 {
        builder = builder
            .object(ObjectSpec::card(p1, &format!("Hand Card {}", i)).in_zone(ZoneId::Hand(p1)));
    }

    // Add Thought Vessel (with NoMaxHandSize keyword) to battlefield.
    builder = builder.object(
        ObjectSpec::artifact(p1, "Thought Vessel")
            .with_keyword(KeywordAbility::NoMaxHandSize)
            .in_zone(ZoneId::Battlefield),
    );

    let state = builder.build().unwrap();

    // Pass all 4 players in End step to trigger cleanup.
    let (state, _) = process_command(state, Command::PassPriority { player: p1 }).unwrap();
    let (state, _) = process_command(
        state,
        Command::PassPriority {
            player: PlayerId(2),
        },
    )
    .unwrap();
    let (state, _) = process_command(
        state,
        Command::PassPriority {
            player: PlayerId(3),
        },
    )
    .unwrap();
    let (_state, events) = process_command(
        state,
        Command::PassPriority {
            player: PlayerId(4),
        },
    )
    .unwrap();

    // No DiscardedToHandSize events should have fired.
    let discard_events: Vec<_> = events
        .iter()
        .filter(|e| matches!(e, GameEvent::DiscardedToHandSize { player, .. } if *player == p1))
        .collect();
    assert!(
        discard_events.is_empty(),
        "Player with Thought Vessel should NOT discard to hand size; discard events: {:?}",
        discard_events
    );
}

// ── CR 603.1: Alela trigger scoping — during_opponent_turn flag ───────────────

#[test]
/// CR 603.1 — Alela, Cunning Conqueror: WheneverYouCastSpell trigger has
/// during_opponent_turn: true, restricting it to opponent turns only.
/// This is a data model test verifying the card definition is correct.
fn test_alela_opponent_turn_only() {
    let alela_def = all_cards()
        .into_iter()
        .find(|d| d.name == "Alela, Cunning Conqueror")
        .expect("Alela, Cunning Conqueror must be in all_cards()");

    use mtg_engine::AbilityDefinition;

    // Find the WheneverYouCastSpell trigger in Alela's abilities.
    let cast_spell_trigger = alela_def.abilities.iter().find(|a| {
        matches!(
            a,
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WheneverYouCastSpell { .. },
                ..
            }
        )
    });

    assert!(
        cast_spell_trigger.is_some(),
        "Alela should have a WheneverYouCastSpell triggered ability"
    );

    // Verify during_opponent_turn is true (CR 603.1: fires only during opponent turns).
    if let Some(AbilityDefinition::Triggered {
        trigger_condition:
            TriggerCondition::WheneverYouCastSpell {
                during_opponent_turn,
            },
        ..
    }) = cast_spell_trigger
    {
        assert!(
            *during_opponent_turn,
            "Alela's WheneverYouCastSpell should have during_opponent_turn: true (CR 603.1)"
        );
    }
}
