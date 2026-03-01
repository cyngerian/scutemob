//! Foretell keyword ability tests (CR 702.143).
//!
//! Foretell is a keyword that functions while the card is in a player's hand.
//! "Any time a player has priority during their turn, that player may pay {2}
//! and exile a card with foretell from their hand face down. That player may
//! cast that card after the current turn has ended by paying any foretell cost
//! it has rather than paying that spell's mana cost." (CR 702.143a)
//!
//! Key rules verified:
//! - ForetellCard special action: pay {2}, exile face-down (CR 702.143a / CR 116.2h)
//! - Cannot cast foretold card on the same turn it was foretold (CR 702.143a)
//! - Can cast foretold card on a later turn for foretell cost (CR 702.143a)
//! - Foretell is a special action -- does NOT use the stack (CR 702.143b)
//! - Can foretell at any time with priority during own turn, not sorcery speed (CR 116.2h)
//! - Cannot foretell during opponent's turn (CR 116.2h)
//! - Cannot foretell without Foretell keyword (CR 702.143a)
//! - Cannot foretell without {2} mana (cost validation)
//! - Foretell cast is an alternative cost -- mutual exclusion with other alt costs (CR 118.9a)
//! - Foretell does NOT bypass timing restrictions for sorceries (ruling 2021-02-05)
//! - Instants can be cast from exile at instant speed (ruling 2021-02-05)
//! - CR 400.7: new ObjectId after zone change

use mtg_engine::{
    process_command, AbilityDefinition, CardDefinition, CardId, CardRegistry, CardType, Command,
    GameEvent, GameStateBuilder, KeywordAbility, ManaColor, ManaCost, ObjectSpec, PlayerId, Step,
    ZoneId,
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

fn find_object_in_zone(
    state: &mtg_engine::GameState,
    name: &str,
    zone: ZoneId,
) -> Option<mtg_engine::ObjectId> {
    state.objects.iter().find_map(|(&id, obj)| {
        if obj.characteristics.name == name && obj.zone == zone {
            Some(id)
        } else {
            None
        }
    })
}

fn hand_count(state: &mtg_engine::GameState, player: PlayerId) -> usize {
    state
        .objects
        .values()
        .filter(|o| o.zone == ZoneId::Hand(player))
        .count()
}

fn exile_count(state: &mtg_engine::GameState) -> usize {
    state
        .objects
        .values()
        .filter(|o| o.zone == ZoneId::Exile)
        .count()
}

fn graveyard_count(state: &mtg_engine::GameState, player: PlayerId) -> usize {
    state
        .objects
        .values()
        .filter(|o| o.zone == ZoneId::Graveyard(player))
        .count()
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

// ── Card definitions ──────────────────────────────────────────────────────────

/// Saw It Coming: Instant {1}{U}{U}. Counter target spell. Foretell {1}{U}.
fn saw_it_coming_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("saw-it-coming".to_string()),
        name: "Saw It Coming".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            blue: 2,
            ..Default::default()
        }),
        types: mtg_engine::TypeLine {
            card_types: [CardType::Instant].into_iter().collect(),
            ..Default::default()
        },
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Foretell),
            AbilityDefinition::Foretell {
                cost: ManaCost {
                    generic: 1,
                    blue: 1,
                    ..Default::default()
                },
            },
        ],
        ..Default::default()
    }
}

/// Behold the Multiverse: Instant {3}{U}. Foretell {1}{U}. Draw 2.
fn behold_the_multiverse_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("behold-the-multiverse".to_string()),
        name: "Behold the Multiverse".to_string(),
        mana_cost: Some(ManaCost {
            generic: 3,
            blue: 1,
            ..Default::default()
        }),
        types: mtg_engine::TypeLine {
            card_types: [CardType::Instant].into_iter().collect(),
            ..Default::default()
        },
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Foretell),
            AbilityDefinition::Foretell {
                cost: ManaCost {
                    generic: 1,
                    blue: 1,
                    ..Default::default()
                },
            },
        ],
        ..Default::default()
    }
}

/// Glorious End: Sorcery {2}{R}. Foretell {1}{R}.
fn glorious_end_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("glorious-end".to_string()),
        name: "Glorious End".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            red: 1,
            ..Default::default()
        }),
        types: mtg_engine::TypeLine {
            card_types: [CardType::Sorcery].into_iter().collect(),
            ..Default::default()
        },
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Foretell),
            AbilityDefinition::Foretell {
                cost: ManaCost {
                    generic: 1,
                    red: 1,
                    ..Default::default()
                },
            },
        ],
        ..Default::default()
    }
}

/// A plain non-foretell instant for testing flashback exclusion.
fn lightning_bolt_def() -> CardDefinition {
    use mtg_engine::cards::card_definition::EffectAmount;
    CardDefinition {
        card_id: CardId("lightning-bolt".to_string()),
        name: "Lightning Bolt".to_string(),
        mana_cost: Some(ManaCost {
            red: 1,
            ..Default::default()
        }),
        types: mtg_engine::TypeLine {
            card_types: [CardType::Instant].into_iter().collect(),
            ..Default::default()
        },
        abilities: vec![AbilityDefinition::Spell {
            effect: mtg_engine::cards::card_definition::Effect::DealDamage {
                target: mtg_engine::cards::card_definition::EffectTarget::DeclaredTarget {
                    index: 0,
                },
                amount: EffectAmount::Fixed(3),
            },
            targets: vec![mtg_engine::cards::card_definition::TargetRequirement::TargetPlayer],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}

fn build_registry(defs: Vec<CardDefinition>) -> std::sync::Arc<CardRegistry> {
    CardRegistry::new(defs)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

/// CR 702.143a / CR 116.2h — Foretell basic: pay {2}, card exiled face-down.
#[test]
fn test_foretell_basic_exile_face_down() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);
    let registry = build_registry(vec![saw_it_coming_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(PlayerId(2))
        .with_registry(registry)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(
            ObjectSpec::card(p1, "Saw It Coming")
                .with_card_id(CardId("saw-it-coming".to_string()))
                .with_keyword(KeywordAbility::Foretell)
                .with_types(vec![CardType::Instant])
                .in_zone(ZoneId::Hand(p1)),
        )
        .build()
        .unwrap();

    // Give player 1 enough mana to foretell ({2}).
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 2);
    state.turn.priority_holder = Some(p1);

    let card_id = find_object(&state, "Saw It Coming");
    let initial_hand_count = hand_count(&state, p1);
    assert_eq!(initial_hand_count, 1);

    let (state, events) = process_command(
        state,
        Command::ForetellCard {
            player: p1,
            card: card_id,
        },
    )
    .unwrap();

    // Card should now be in exile.
    assert_eq!(
        hand_count(&state, p1),
        0,
        "hand should be empty after foretell"
    );
    assert_eq!(exile_count(&state), 1, "one card should be in exile");

    // The exile object should be face-down and marked as foretold.
    let exile_obj = state
        .objects
        .values()
        .find(|o| o.zone == ZoneId::Exile)
        .expect("exile object should exist");
    assert!(
        exile_obj.status.face_down,
        "foretold card should be face-down (CR 702.143a)"
    );
    assert!(
        exile_obj.is_foretold,
        "foretold card should have is_foretold=true (CR 702.143a)"
    );
    assert_eq!(
        exile_obj.foretold_turn, state.turn.turn_number,
        "foretold_turn should equal current turn (CR 702.143a)"
    );

    // Events: ManaCostPaid + CardForetold.
    let has_mana_paid = events
        .iter()
        .any(|e| matches!(e, GameEvent::ManaCostPaid { .. }));
    let has_foretold = events
        .iter()
        .any(|e| matches!(e, GameEvent::CardForetold { player, .. } if *player == p1));
    assert!(has_mana_paid, "ManaCostPaid event should be emitted");
    assert!(
        has_foretold,
        "CardForetold event should be emitted (CR 702.143a)"
    );

    let _ = p2; // avoid unused warning
}

/// CR 702.143b — Foretell does NOT use the stack (special action).
#[test]
fn test_foretell_does_not_use_stack() {
    let p1 = PlayerId(1);
    let registry = build_registry(vec![saw_it_coming_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(PlayerId(2))
        .with_registry(registry)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(
            ObjectSpec::card(p1, "Saw It Coming")
                .with_card_id(CardId("saw-it-coming".to_string()))
                .with_keyword(KeywordAbility::Foretell)
                .with_types(vec![CardType::Instant])
                .in_zone(ZoneId::Hand(p1)),
        )
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 2);
    state.turn.priority_holder = Some(p1);

    let card_id = find_object(&state, "Saw It Coming");

    let (state, _) = process_command(
        state,
        Command::ForetellCard {
            player: p1,
            card: card_id,
        },
    )
    .unwrap();

    // Stack must be empty — foretell is a special action, not a spell (CR 702.143b).
    assert!(
        state.stack_objects.is_empty(),
        "foretell must not use the stack (CR 702.143b)"
    );
}

/// CR 702.143a — Cannot cast foretold card on the same turn it was foretold.
#[test]
fn test_foretell_cannot_cast_same_turn() {
    let p1 = PlayerId(1);
    let registry = build_registry(vec![saw_it_coming_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(PlayerId(2))
        .with_registry(registry)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .turn_number(5)
        .object(
            ObjectSpec::card(p1, "Saw It Coming")
                .with_card_id(CardId("saw-it-coming".to_string()))
                .with_keyword(KeywordAbility::Foretell)
                .with_types(vec![CardType::Instant])
                .in_zone(ZoneId::Hand(p1)),
        )
        .build()
        .unwrap();

    // Give mana for foretell action ({2}).
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 3);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Blue, 2);
    state.turn.priority_holder = Some(p1);

    let card_id = find_object(&state, "Saw It Coming");

    // Foretell the card (turn 5).
    let (state, _) = process_command(
        state,
        Command::ForetellCard {
            player: p1,
            card: card_id,
        },
    )
    .unwrap();

    // Find the foretold card in exile.
    let exile_id = state
        .objects
        .iter()
        .find(|(_, o)| o.zone == ZoneId::Exile && o.is_foretold)
        .map(|(&id, _)| id)
        .expect("foretold card should be in exile");

    // Attempt to cast it on the same turn (turn 5) — must fail.
    let result = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: exile_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            cast_with_evoke: false,
            cast_with_bestow: false,
            cast_with_miracle: false,
            cast_with_escape: false,
            escape_exile_cards: vec![],
            cast_with_foretell: true,
            cast_with_buyback: false,
            cast_with_overload: false,
            retrace_discard_land: None,
            cast_with_jump_start: false,
            jump_start_discard: None,
            cast_with_aftermath: false,
        },
    );

    assert!(
        result.is_err(),
        "casting foretold card on the same turn must fail (CR 702.143a)"
    );
    let err = result.unwrap_err();
    let err_str = format!("{:?}", err);
    assert!(
        err_str.contains("same turn") || err_str.contains("current turn"),
        "error should mention same-turn restriction: {}",
        err_str
    );
}

/// CR 702.143a — Foretold card can be cast on a later turn for foretell cost.
#[test]
fn test_foretell_cast_from_exile_on_later_turn() {
    let p1 = PlayerId(1);
    let registry = build_registry(vec![behold_the_multiverse_def()]);

    // Start on turn 3, foretell the card.
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(PlayerId(2))
        .with_registry(registry)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .turn_number(3)
        .object(
            ObjectSpec::card(p1, "Behold the Multiverse")
                .with_card_id(CardId("behold-the-multiverse".to_string()))
                .with_keyword(KeywordAbility::Foretell)
                .with_types(vec![CardType::Instant])
                .in_zone(ZoneId::Hand(p1)),
        )
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 2);
    state.turn.priority_holder = Some(p1);

    let card_id = find_object(&state, "Behold the Multiverse");

    let (mut state, _) = process_command(
        state,
        Command::ForetellCard {
            player: p1,
            card: card_id,
        },
    )
    .unwrap();

    // Advance to turn 4 (next turn).
    state.turn.turn_number = 4;
    state.turn.priority_holder = Some(p1);

    // Give mana for foretell cost ({1}{U}).
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 1);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Blue, 1);

    let exile_id = state
        .objects
        .iter()
        .find(|(_, o)| o.zone == ZoneId::Exile && o.is_foretold)
        .map(|(&id, _)| id)
        .expect("foretold card should be in exile");

    // Cast from exile for foretell cost -- should succeed.
    let result = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: exile_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            cast_with_evoke: false,
            cast_with_bestow: false,
            cast_with_miracle: false,
            cast_with_escape: false,
            escape_exile_cards: vec![],
            cast_with_foretell: true,
            cast_with_buyback: false,
            cast_with_overload: false,
            retrace_discard_land: None,
            cast_with_jump_start: false,
            jump_start_discard: None,
            cast_with_aftermath: false,
        },
    );

    assert!(
        result.is_ok(),
        "casting foretold card on later turn should succeed (CR 702.143a): {:?}",
        result.err()
    );

    let (state, events) = result.unwrap();

    // The card should now be on the stack.
    assert!(
        !state.stack_objects.is_empty(),
        "spell should be on the stack"
    );
    // No longer in exile.
    assert_eq!(exile_count(&state), 0, "card should have left exile");
    // SpellCast event should be emitted.
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::SpellCast { .. })),
        "SpellCast event should be emitted"
    );
}

/// CR 116.2h — Cannot foretell during opponent's turn.
#[test]
fn test_foretell_requires_player_turn() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);
    let registry = build_registry(vec![saw_it_coming_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(PlayerId(2))
        .with_registry(registry)
        .active_player(p2) // p2 is the active player
        .at_step(Step::PreCombatMain)
        .object(
            ObjectSpec::card(p1, "Saw It Coming")
                .with_card_id(CardId("saw-it-coming".to_string()))
                .with_keyword(KeywordAbility::Foretell)
                .with_types(vec![CardType::Instant])
                .in_zone(ZoneId::Hand(p1)),
        )
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 2);
    // Priority goes to p1 during p2's turn (response window).
    state.turn.priority_holder = Some(p1);

    let card_id = find_object(&state, "Saw It Coming");

    // P1 tries to foretell during p2's turn -- should fail.
    let result = process_command(
        state,
        Command::ForetellCard {
            player: p1,
            card: card_id,
        },
    );

    assert!(
        result.is_err(),
        "foretell must not be allowed during opponent's turn (CR 116.2h)"
    );
    let err = format!("{:?}", result.unwrap_err());
    assert!(
        err.contains("own turn") || err.contains("your turn") || err.contains("active"),
        "error should mention turn restriction: {}",
        err
    );
}

/// CR 702.143a — Cannot foretell a card without the Foretell keyword.
#[test]
fn test_foretell_requires_foretell_keyword() {
    let p1 = PlayerId(1);
    let registry = build_registry(vec![lightning_bolt_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(PlayerId(2))
        .with_registry(registry)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(
            ObjectSpec::card(p1, "Lightning Bolt")
                .with_card_id(CardId("lightning-bolt".to_string()))
                .in_zone(ZoneId::Hand(p1)),
        )
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 2);
    state.turn.priority_holder = Some(p1);

    let card_id = find_object(&state, "Lightning Bolt");

    let result = process_command(
        state,
        Command::ForetellCard {
            player: p1,
            card: card_id,
        },
    );

    assert!(
        result.is_err(),
        "foretell must be rejected for cards without Foretell keyword (CR 702.143a)"
    );
}

/// CR 702.143a — Cannot foretell a card not in hand.
#[test]
fn test_foretell_requires_card_in_hand() {
    let p1 = PlayerId(1);
    let registry = build_registry(vec![saw_it_coming_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(PlayerId(2))
        .with_registry(registry)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(
            ObjectSpec::card(p1, "Saw It Coming")
                .with_card_id(CardId("saw-it-coming".to_string()))
                .with_keyword(KeywordAbility::Foretell)
                .with_types(vec![CardType::Instant])
                .in_zone(ZoneId::Graveyard(p1)), // in graveyard, not hand
        )
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 2);
    state.turn.priority_holder = Some(p1);

    let card_id = find_object(&state, "Saw It Coming");

    let result = process_command(
        state,
        Command::ForetellCard {
            player: p1,
            card: card_id,
        },
    );

    assert!(
        result.is_err(),
        "foretell must reject cards not in hand (CR 702.143a)"
    );
}

/// Insufficient mana — cannot foretell without {2}.
#[test]
fn test_foretell_insufficient_mana() {
    let p1 = PlayerId(1);
    let registry = build_registry(vec![saw_it_coming_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(PlayerId(2))
        .with_registry(registry)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(
            ObjectSpec::card(p1, "Saw It Coming")
                .with_card_id(CardId("saw-it-coming".to_string()))
                .with_keyword(KeywordAbility::Foretell)
                .with_types(vec![CardType::Instant])
                .in_zone(ZoneId::Hand(p1)),
        )
        .build()
        .unwrap();

    // Only {1} mana -- not enough for {2} foretell action cost.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 1);
    state.turn.priority_holder = Some(p1);

    let card_id = find_object(&state, "Saw It Coming");

    let result = process_command(
        state,
        Command::ForetellCard {
            player: p1,
            card: card_id,
        },
    );

    assert!(
        result.is_err(),
        "foretell must be rejected with insufficient mana"
    );
    let err = format!("{:?}", result.unwrap_err());
    assert!(
        err.contains("Mana") || err.contains("mana"),
        "error should indicate mana issue: {}",
        err
    );
}

/// CR 116.2h — Foretell can be used during combat (any time with priority during own turn).
#[test]
fn test_foretell_during_combat() {
    let p1 = PlayerId(1);
    let registry = build_registry(vec![saw_it_coming_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(PlayerId(2))
        .with_registry(registry)
        .active_player(p1)
        .at_step(Step::DeclareAttackers) // combat step, not main phase
        .object(
            ObjectSpec::card(p1, "Saw It Coming")
                .with_card_id(CardId("saw-it-coming".to_string()))
                .with_keyword(KeywordAbility::Foretell)
                .with_types(vec![CardType::Instant])
                .in_zone(ZoneId::Hand(p1)),
        )
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 2);
    state.turn.priority_holder = Some(p1);

    let card_id = find_object(&state, "Saw It Coming");

    // Should succeed -- foretell has no sorcery-speed restriction (CR 116.2h).
    let result = process_command(
        state,
        Command::ForetellCard {
            player: p1,
            card: card_id,
        },
    );

    assert!(
        result.is_ok(),
        "foretell should succeed during combat (CR 116.2h: any time with priority during own turn): {:?}",
        result.err()
    );

    let (state, _) = result.unwrap();
    assert_eq!(
        exile_count(&state),
        1,
        "card should be in exile after foretelling during combat"
    );
}

/// CR 118.9a — Cannot combine foretell with escape (mutual exclusion).
#[test]
fn test_foretell_mutual_exclusion_with_escape() {
    // We test this indirectly by trying to cast a foretold card from exile
    // without the foretell flag, checking that the "not in hand" error fires
    // (since foretell is required to bypass the zone check for exile).
    // A foretold card in exile without cast_with_foretell: true should fail zone validation.
    let p1 = PlayerId(1);
    let registry = build_registry(vec![saw_it_coming_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(PlayerId(2))
        .with_registry(registry)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .turn_number(3)
        .object(
            ObjectSpec::card(p1, "Saw It Coming")
                .with_card_id(CardId("saw-it-coming".to_string()))
                .with_keyword(KeywordAbility::Foretell)
                .with_types(vec![CardType::Instant])
                .in_zone(ZoneId::Hand(p1)),
        )
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 2);
    state.turn.priority_holder = Some(p1);

    let card_id = find_object(&state, "Saw It Coming");

    let (mut state, _) = process_command(
        state,
        Command::ForetellCard {
            player: p1,
            card: card_id,
        },
    )
    .unwrap();

    // Advance to turn 4.
    state.turn.turn_number = 4;
    state.turn.priority_holder = Some(p1);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Blue, 2);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 1);

    let exile_id = state
        .objects
        .iter()
        .find(|(_, o)| o.zone == ZoneId::Exile && o.is_foretold)
        .map(|(&id, _)| id)
        .expect("foretold card should be in exile");

    // Attempting to cast a foretell card with BOTH cast_with_foretell AND cast_with_escape
    // should fail with mutual exclusion (CR 118.9a).
    let result = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: exile_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            cast_with_evoke: false,
            cast_with_bestow: false,
            cast_with_miracle: false,
            cast_with_escape: true, // conflicting alternative cost
            escape_exile_cards: vec![],
            cast_with_foretell: true,
            cast_with_buyback: false,
            cast_with_overload: false,
            retrace_discard_land: None,
            cast_with_jump_start: false,
            jump_start_discard: None,
            cast_with_aftermath: false,
        },
    );

    assert!(
        result.is_err(),
        "combining foretell with escape should fail (CR 118.9a)"
    );
    let err_str = format!("{:?}", result.unwrap_err());
    assert!(
        err_str.contains("foretell") || err_str.contains("escape"),
        "error should mention mutual exclusion: {}",
        err_str
    );
}

/// CR 118.9a — Cannot combine foretell with evoke (mutual exclusion).
#[test]
fn test_foretell_mutual_exclusion_with_evoke() {
    let p1 = PlayerId(1);
    let registry = build_registry(vec![saw_it_coming_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(PlayerId(2))
        .with_registry(registry)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .turn_number(3)
        .object(
            ObjectSpec::card(p1, "Saw It Coming")
                .with_card_id(CardId("saw-it-coming".to_string()))
                .with_keyword(KeywordAbility::Foretell)
                .with_types(vec![CardType::Instant])
                .in_zone(ZoneId::Hand(p1)),
        )
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 2);
    state.turn.priority_holder = Some(p1);

    let card_id = find_object(&state, "Saw It Coming");

    let (mut state, _) = process_command(
        state,
        Command::ForetellCard {
            player: p1,
            card: card_id,
        },
    )
    .unwrap();

    state.turn.turn_number = 4;
    state.turn.priority_holder = Some(p1);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Blue, 2);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 1);

    let exile_id = state
        .objects
        .iter()
        .find(|(_, o)| o.zone == ZoneId::Exile && o.is_foretold)
        .map(|(&id, _)| id)
        .expect("foretold card should be in exile");

    let result = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: exile_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            cast_with_evoke: true, // conflicting alternative cost
            cast_with_bestow: false,
            cast_with_miracle: false,
            cast_with_escape: false,
            escape_exile_cards: vec![],
            cast_with_foretell: true,
            cast_with_buyback: false,
            cast_with_overload: false,
            retrace_discard_land: None,
            cast_with_jump_start: false,
            jump_start_discard: None,
            cast_with_aftermath: false,
        },
    );

    assert!(
        result.is_err(),
        "combining foretell with evoke should fail (CR 118.9a)"
    );
}

/// Ruling 2021-02-05 — Foretelled sorcery cannot be cast at instant speed.
#[test]
fn test_foretell_sorcery_timing_restriction() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);
    let registry = build_registry(vec![glorious_end_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(PlayerId(2))
        .with_registry(registry)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .turn_number(3)
        .object(
            ObjectSpec::card(p1, "Glorious End")
                .with_card_id(CardId("glorious-end".to_string()))
                .with_keyword(KeywordAbility::Foretell)
                .with_types(vec![CardType::Sorcery])
                .in_zone(ZoneId::Hand(p1)),
        )
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 2);
    state.turn.priority_holder = Some(p1);

    let card_id = find_object(&state, "Glorious End");

    let (mut state, _) = process_command(
        state,
        Command::ForetellCard {
            player: p1,
            card: card_id,
        },
    )
    .unwrap();

    // Turn 4 but it's p2's turn -- p1 cannot cast a sorcery at instant speed.
    state.turn.turn_number = 4;
    state.turn.active_player = p2; // p2's turn
    state.turn.priority_holder = Some(p1); // p1 has priority (response window)

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Red, 2);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 1);

    let exile_id = state
        .objects
        .iter()
        .find(|(_, o)| o.zone == ZoneId::Exile && o.is_foretold)
        .map(|(&id, _)| id)
        .expect("foretold card should be in exile");

    // Cast sorcery at instant speed (during opponent's turn) -- must fail.
    let result = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: exile_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            cast_with_evoke: false,
            cast_with_bestow: false,
            cast_with_miracle: false,
            cast_with_escape: false,
            escape_exile_cards: vec![],
            cast_with_foretell: true,
            cast_with_buyback: false,
            cast_with_overload: false,
            retrace_discard_land: None,
            cast_with_jump_start: false,
            jump_start_discard: None,
            cast_with_aftermath: false,
        },
    );

    assert!(
        result.is_err(),
        "foretelled sorcery must follow sorcery timing (ruling 2021-02-05): should fail"
    );
}

/// Ruling 2021-02-05 — Foretelled instant can be cast at instant speed.
#[test]
fn test_foretell_instant_timing() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);
    let registry = build_registry(vec![saw_it_coming_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(PlayerId(2))
        .with_registry(registry)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .turn_number(3)
        .object(
            ObjectSpec::card(p1, "Saw It Coming")
                .with_card_id(CardId("saw-it-coming".to_string()))
                .with_keyword(KeywordAbility::Foretell)
                .with_types(vec![CardType::Instant])
                .in_zone(ZoneId::Hand(p1)),
        )
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 2);
    state.turn.priority_holder = Some(p1);

    let card_id = find_object(&state, "Saw It Coming");

    let (mut state, _) = process_command(
        state,
        Command::ForetellCard {
            player: p1,
            card: card_id,
        },
    )
    .unwrap();

    // Turn 4, opponent's turn with a spell on the stack -- instant speed.
    state.turn.turn_number = 4;
    state.turn.active_player = p2;
    state.turn.priority_holder = Some(p1);

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 1);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Blue, 1);

    let exile_id = state
        .objects
        .iter()
        .find(|(_, o)| o.zone == ZoneId::Exile && o.is_foretold)
        .map(|(&id, _)| id)
        .expect("foretold card should be in exile");

    // An instant should cast just fine during opponent's turn (ruling 2021-02-05).
    let result = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: exile_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            cast_with_evoke: false,
            cast_with_bestow: false,
            cast_with_miracle: false,
            cast_with_escape: false,
            escape_exile_cards: vec![],
            cast_with_foretell: true,
            cast_with_buyback: false,
            cast_with_overload: false,
            retrace_discard_land: None,
            cast_with_jump_start: false,
            jump_start_discard: None,
            cast_with_aftermath: false,
        },
    );

    assert!(
        result.is_ok(),
        "foretelled instant should cast at instant speed (ruling 2021-02-05): {:?}",
        result.err()
    );
}

/// CR 400.7 — After foretelling, the old ObjectId is dead; new ID in exile.
#[test]
fn test_foretell_card_identity_tracking() {
    let p1 = PlayerId(1);
    let registry = build_registry(vec![saw_it_coming_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(PlayerId(2))
        .with_registry(registry)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(
            ObjectSpec::card(p1, "Saw It Coming")
                .with_card_id(CardId("saw-it-coming".to_string()))
                .with_keyword(KeywordAbility::Foretell)
                .with_types(vec![CardType::Instant])
                .in_zone(ZoneId::Hand(p1)),
        )
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 2);
    state.turn.priority_holder = Some(p1);

    let old_card_id = find_object(&state, "Saw It Coming");

    let (state, events) = process_command(
        state,
        Command::ForetellCard {
            player: p1,
            card: old_card_id,
        },
    )
    .unwrap();

    // Old ObjectId should no longer exist (CR 400.7).
    assert!(
        !state.objects.contains_key(&old_card_id),
        "old ObjectId should not exist after zone change (CR 400.7)"
    );

    // New object should be in exile.
    let exile_obj = find_object_in_zone(&state, "Saw It Coming", ZoneId::Exile);
    assert!(
        exile_obj.is_some(),
        "card should exist in exile under new ObjectId (CR 400.7)"
    );

    // CardForetold event should reference both old and new IDs.
    let foretold_event = events
        .iter()
        .find(|e| matches!(e, GameEvent::CardForetold { .. }));
    assert!(
        foretold_event.is_some(),
        "CardForetold event should be emitted"
    );

    if let Some(GameEvent::CardForetold {
        object_id,
        new_exile_id,
        ..
    }) = foretold_event
    {
        assert_eq!(
            *object_id, old_card_id,
            "object_id should be the old (pre-exile) id"
        );
        assert_ne!(
            *object_id, *new_exile_id,
            "new_exile_id should be different (CR 400.7)"
        );
    }
}

/// CR 118.9c — Foretell cost is the PAID cost; card retains printed mana value.
#[test]
fn test_foretell_mana_value_unchanged() {
    let p1 = PlayerId(1);
    let registry = build_registry(vec![saw_it_coming_def()]);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(PlayerId(2))
        .with_registry(registry)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(
            ObjectSpec::card(p1, "Saw It Coming")
                .with_card_id(CardId("saw-it-coming".to_string()))
                .with_keyword(KeywordAbility::Foretell)
                .with_types(vec![CardType::Instant])
                .with_mana_cost(ManaCost {
                    generic: 1,
                    blue: 2,
                    ..Default::default()
                })
                .in_zone(ZoneId::Hand(p1)),
        )
        .build()
        .unwrap();

    // The card in hand should have the printed mana cost {1}{U}{U}.
    let card_obj = state
        .objects
        .values()
        .find(|o| o.characteristics.name == "Saw It Coming")
        .expect("card should be in state");

    let mana_cost = card_obj.characteristics.mana_cost.as_ref();
    assert!(
        mana_cost.is_some(),
        "card should have a mana cost (CR 118.9c)"
    );

    let cost = mana_cost.unwrap();
    // Printed cost: {1}{U}{U} = MV 3
    assert_eq!(
        cost.mana_value(),
        3,
        "printed mana value should be 3 for Saw It Coming (CR 118.9c)"
    );
}

/// Cannot cast a foreteld card without cast_with_foretell (zone check fails).
#[test]
fn test_foretell_card_requires_cast_with_foretell_flag() {
    let p1 = PlayerId(1);
    let registry = build_registry(vec![saw_it_coming_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(PlayerId(2))
        .with_registry(registry)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .turn_number(3)
        .object(
            ObjectSpec::card(p1, "Saw It Coming")
                .with_card_id(CardId("saw-it-coming".to_string()))
                .with_keyword(KeywordAbility::Foretell)
                .with_types(vec![CardType::Instant])
                .in_zone(ZoneId::Hand(p1)),
        )
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 2);
    state.turn.priority_holder = Some(p1);

    let card_id = find_object(&state, "Saw It Coming");

    let (mut state, _) = process_command(
        state,
        Command::ForetellCard {
            player: p1,
            card: card_id,
        },
    )
    .unwrap();

    state.turn.turn_number = 4;
    state.turn.priority_holder = Some(p1);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Blue, 2);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 1);

    let exile_id = state
        .objects
        .iter()
        .find(|(_, o)| o.zone == ZoneId::Exile && o.is_foretold)
        .map(|(&id, _)| id)
        .expect("foretold card should be in exile");

    // Attempt to cast without cast_with_foretell: true -- must fail (card is in exile, not hand).
    let result = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: exile_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            cast_with_evoke: false,
            cast_with_bestow: false,
            cast_with_miracle: false,
            cast_with_escape: false,
            escape_exile_cards: vec![],
            cast_with_foretell: false, // NOT requesting foretell cast
            cast_with_buyback: false,
            cast_with_overload: false,
            retrace_discard_land: None,
            cast_with_jump_start: false,
            jump_start_discard: None,
            cast_with_aftermath: false,
        },
    );

    assert!(
        result.is_err(),
        "casting foreteld card without cast_with_foretell should fail (card not in hand)"
    );
}

/// Reveals hidden info test — CardForetold event is marked as revealing hidden info.
#[test]
fn test_foretell_reveals_hidden_info() {
    let p1 = PlayerId(1);
    let registry = build_registry(vec![saw_it_coming_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(PlayerId(2))
        .with_registry(registry)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(
            ObjectSpec::card(p1, "Saw It Coming")
                .with_card_id(CardId("saw-it-coming".to_string()))
                .with_keyword(KeywordAbility::Foretell)
                .with_types(vec![CardType::Instant])
                .in_zone(ZoneId::Hand(p1)),
        )
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 2);
    state.turn.priority_holder = Some(p1);

    let card_id = find_object(&state, "Saw It Coming");

    let (_state, events) = process_command(
        state,
        Command::ForetellCard {
            player: p1,
            card: card_id,
        },
    )
    .unwrap();

    // Find the CardForetold event and verify it reveals hidden info.
    let foretold_ev = events
        .iter()
        .find(|e| matches!(e, GameEvent::CardForetold { .. }))
        .expect("CardForetold event should exist");

    assert!(
        foretold_ev.reveals_hidden_info(),
        "CardForetold must reveal hidden info (CR 702.143a: opponent sees exile but not card identity)"
    );
}

/// Cannot cast a foretold card that was not actually foretold (is_foretold == false).
#[test]
fn test_foretell_requires_is_foretold_flag() {
    let p1 = PlayerId(1);
    let registry = build_registry(vec![saw_it_coming_def()]);

    // Place the card directly in exile without going through foretell action.
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(PlayerId(2))
        .with_registry(registry)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .turn_number(5)
        .object(
            ObjectSpec::card(p1, "Saw It Coming")
                .with_card_id(CardId("saw-it-coming".to_string()))
                .with_types(vec![CardType::Instant])
                .in_zone(ZoneId::Exile), // exile but NOT foretold
        )
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Blue, 2);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 1);
    state.turn.priority_holder = Some(p1);

    let card_id = find_object(&state, "Saw It Coming");

    // is_foretold should be false by default.
    assert!(
        !state.objects[&card_id].is_foretold,
        "card placed directly in exile should not be foretold"
    );

    let result = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: card_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            cast_with_evoke: false,
            cast_with_bestow: false,
            cast_with_miracle: false,
            cast_with_escape: false,
            escape_exile_cards: vec![],
            cast_with_foretell: true,
            cast_with_buyback: false,
            cast_with_overload: false,
            retrace_discard_land: None,
            cast_with_jump_start: false,
            jump_start_discard: None,
            cast_with_aftermath: false,
        },
    );

    assert!(
        result.is_err(),
        "casting a non-foretold exile card with cast_with_foretell should fail (CR 702.143a)"
    );
    let err = format!("{:?}", result.unwrap_err());
    assert!(
        err.contains("foretold") || err.contains("not foretold"),
        "error should mention foretold status: {}",
        err
    );
}
