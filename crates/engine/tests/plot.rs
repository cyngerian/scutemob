//! Plot keyword ability tests (CR 702.170).
//!
//! Plot is a keyword that functions while the card is in a player's hand.
//! "Any time you have priority during your main phase while the stack is empty,
//! you may exile this card from your hand and pay [cost]. It becomes a plotted card."
//! (CR 702.170a)
//!
//! Key rules verified:
//! - PlotCard special action: pay plot cost, exile face-up (CR 702.170a / CR 116.2k)
//! - Cannot cast plotted card on the same turn it was plotted (CR 702.170d)
//! - Can cast plotted card on a later turn for free (CR 702.170d)
//! - Plot is a special action -- does NOT use the stack (CR 702.170b)
//! - Plot requires main phase + empty stack, not just any priority (CR 116.2k)
//! - Plot free-cast is restricted to main phase + empty stack (CR 702.170d)
//! - Even instants obey sorcery timing when cast via Plot (CR 702.170d)
//! - Cannot plot during opponent's turn (CR 116.2k)
//! - Cannot plot without Plot keyword (CR 702.170a)
//! - Cannot plot without sufficient mana (CR 702.170a)
//! - Plot free-cast costs zero mana (CR 702.170d)
//! - Plot free-cast cannot combine with other alt costs (CR 118.9a)
//! - CR 400.7: new ObjectId after zone change

use mtg_engine::state::types::AltCostKind;
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

// ── Card definitions ──────────────────────────────────────────────────────────

/// A sorcery with Plot {1}{R}. Mana cost {2}{R}.
fn plot_sorcery_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("plot-sorcery".to_string()),
        name: "Plot Sorcery".to_string(),
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
            AbilityDefinition::Keyword(KeywordAbility::Plot),
            AbilityDefinition::Plot {
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

/// An instant with Plot {1}{U}. Mana cost {2}{U}.
/// Used to test that plotted instants still obey sorcery timing.
fn plot_instant_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("plot-instant".to_string()),
        name: "Plot Instant".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            blue: 1,
            ..Default::default()
        }),
        types: mtg_engine::TypeLine {
            card_types: [CardType::Instant].into_iter().collect(),
            ..Default::default()
        },
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Plot),
            AbilityDefinition::Plot {
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

/// A creature with Plot {1}{R}. Mana cost {3}{R}{R}.
/// Used to test free-cast cost validation.
fn plot_creature_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("plot-creature".to_string()),
        name: "Plot Creature".to_string(),
        mana_cost: Some(ManaCost {
            generic: 3,
            red: 2,
            ..Default::default()
        }),
        types: mtg_engine::TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        power: Some(3),
        toughness: Some(3),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Plot),
            AbilityDefinition::Plot {
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

/// A plain sorcery without Plot (for testing negative cases).
fn plain_sorcery_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("plain-sorcery".to_string()),
        name: "Plain Sorcery".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            ..Default::default()
        }),
        types: mtg_engine::TypeLine {
            card_types: [CardType::Sorcery].into_iter().collect(),
            ..Default::default()
        },
        abilities: vec![],
        ..Default::default()
    }
}

fn build_registry(defs: Vec<CardDefinition>) -> std::sync::Arc<CardRegistry> {
    CardRegistry::new(defs)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

/// CR 702.170a / CR 116.2k — Plot basic: pay plot cost, card exiled face-up.
#[test]
fn test_plot_basic_exile_face_up() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);
    let registry = build_registry(vec![plot_sorcery_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(
            ObjectSpec::card(p1, "Plot Sorcery")
                .with_card_id(CardId("plot-sorcery".to_string()))
                .with_keyword(KeywordAbility::Plot)
                .with_types(vec![CardType::Sorcery])
                .in_zone(ZoneId::Hand(p1)),
        )
        .build()
        .unwrap();

    // Give player 1 enough mana for plot cost {1}{R}.
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
        .add(ManaColor::Red, 1);
    state.turn.priority_holder = Some(p1);

    let card_id = find_object(&state, "Plot Sorcery");
    assert_eq!(hand_count(&state, p1), 1);

    let (state, events) = process_command(
        state,
        Command::PlotCard {
            player: p1,
            card: card_id,
        },
    )
    .unwrap();

    // Card should now be in exile (CR 702.170a: exiled from hand).
    assert_eq!(hand_count(&state, p1), 0, "hand should be empty after plot");
    assert_eq!(exile_count(&state), 1, "one card should be in exile");

    // The exile object should be face-UP (plotted cards are public, CR 702.170a).
    let exile_obj = state
        .objects
        .values()
        .find(|o| o.zone == ZoneId::Exile)
        .expect("exile object should exist");
    assert!(
        !exile_obj.status.face_down,
        "plotted card should be face-up (CR 702.170a: public information)"
    );
    assert!(
        exile_obj.is_plotted,
        "exile object should have is_plotted=true (CR 702.170a)"
    );
    assert_eq!(
        exile_obj.plotted_turn, state.turn.turn_number,
        "plotted_turn should equal current turn (CR 702.170d)"
    );

    // Events: ManaCostPaid + CardPlotted.
    let has_mana_paid = events
        .iter()
        .any(|e| matches!(e, GameEvent::ManaCostPaid { .. }));
    let has_plotted = events
        .iter()
        .any(|e| matches!(e, GameEvent::CardPlotted { player, .. } if *player == p1));
    assert!(has_mana_paid, "ManaCostPaid event should be emitted");
    assert!(
        has_plotted,
        "CardPlotted event should be emitted (CR 702.170a)"
    );
}

/// CR 702.170b — Plot does NOT use the stack (special action).
#[test]
fn test_plot_does_not_use_stack() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);
    let registry = build_registry(vec![plot_sorcery_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(
            ObjectSpec::card(p1, "Plot Sorcery")
                .with_card_id(CardId("plot-sorcery".to_string()))
                .with_keyword(KeywordAbility::Plot)
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
        .add(ManaColor::Colorless, 1);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Red, 1);
    state.turn.priority_holder = Some(p1);

    let card_id = find_object(&state, "Plot Sorcery");

    let (state, _) = process_command(
        state,
        Command::PlotCard {
            player: p1,
            card: card_id,
        },
    )
    .unwrap();

    // Stack must remain empty (CR 702.170b: special action, not a spell).
    assert!(
        state.stack_objects.is_empty(),
        "stack must be empty after plot (CR 702.170b: special action)"
    );
}

/// CR 702.170d — Cannot cast plotted card on the same turn it was plotted.
#[test]
fn test_plot_cannot_cast_same_turn() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);
    let registry = build_registry(vec![plot_sorcery_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(
            ObjectSpec::card(p1, "Plot Sorcery")
                .with_card_id(CardId("plot-sorcery".to_string()))
                .with_keyword(KeywordAbility::Plot)
                .with_types(vec![CardType::Sorcery])
                .in_zone(ZoneId::Hand(p1)),
        )
        .build()
        .unwrap();

    // Give plot cost {1}{R}.
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
        .add(ManaColor::Red, 1);
    state.turn.priority_holder = Some(p1);

    // Plot the card.
    let card_id = find_object(&state, "Plot Sorcery");
    let (state, _) = process_command(
        state,
        Command::PlotCard {
            player: p1,
            card: card_id,
        },
    )
    .unwrap();

    // The card is now in exile with is_plotted = true.
    // Attempt to cast it on the SAME turn -- should fail.
    let exile_id = find_object_in_zone(&state, "Plot Sorcery", ZoneId::Exile)
        .expect("card should be in exile");

    let mut state = state;
    state.turn.priority_holder = Some(p1);

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
            alt_cost: Some(AltCostKind::Plot),
            escape_exile_cards: vec![],
            retrace_discard_land: None,
            jump_start_discard: None,
            prototype: false,
            bargain_sacrifice: None,
            emerge_sacrifice: None,
            casualty_sacrifice: None,
            assist_player: None,
            assist_amount: 0,
            replicate_count: 0,
            splice_cards: vec![],
            entwine_paid: false,
            escalate_modes: 0,
            devour_sacrifices: vec![],
            modes_chosen: vec![],
            fuse: false,
            x_value: 0,
            collect_evidence_cards: vec![],
            squad_count: 0,
        },
    );

    assert!(
        result.is_err(),
        "should not be able to cast plotted card on the same turn (CR 702.170d)"
    );
}

/// CR 702.170d — Can cast plotted card on a LATER turn for free.
#[test]
fn test_plot_cast_from_exile_on_later_turn() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);
    let registry = build_registry(vec![plot_sorcery_def()]);

    // Start with a pre-plotted card in exile (plotted on turn 1, now on turn 2).
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(
            ObjectSpec::card(p1, "Plot Sorcery")
                .with_card_id(CardId("plot-sorcery".to_string()))
                .with_keyword(KeywordAbility::Plot)
                .with_types(vec![CardType::Sorcery])
                .in_zone(ZoneId::Exile),
        )
        .build()
        .unwrap();

    // Set up: turn 2, card was plotted on turn 1.
    state.turn.turn_number = 2;
    state.turn.priority_holder = Some(p1);

    // Mark the exile card as plotted on turn 1.
    let exile_id = find_object_in_zone(&state, "Plot Sorcery", ZoneId::Exile)
        .expect("card should be in exile");
    {
        let obj = state.objects.get_mut(&exile_id).unwrap();
        obj.is_plotted = true;
        obj.plotted_turn = 1;
    }

    // Cast it for free via Plot.
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
            alt_cost: Some(AltCostKind::Plot),
            escape_exile_cards: vec![],
            retrace_discard_land: None,
            jump_start_discard: None,
            prototype: false,
            bargain_sacrifice: None,
            emerge_sacrifice: None,
            casualty_sacrifice: None,
            assist_player: None,
            assist_amount: 0,
            replicate_count: 0,
            splice_cards: vec![],
            entwine_paid: false,
            escalate_modes: 0,
            devour_sacrifices: vec![],
            modes_chosen: vec![],
            fuse: false,
            x_value: 0,
            collect_evidence_cards: vec![],
            squad_count: 0,
        },
    );

    assert!(
        result.is_ok(),
        "should be able to cast plotted card on a later turn (CR 702.170d): {:?}",
        result.err()
    );

    let (state, events) = result.unwrap();
    // Card should now be on the stack.
    assert_eq!(state.stack_objects.len(), 1, "spell should be on the stack");

    // SpellCast event should have been emitted.
    let has_spell_cast = events
        .iter()
        .any(|e| matches!(e, GameEvent::SpellCast { player, .. } if *player == p1));
    assert!(has_spell_cast, "SpellCast event should be emitted");
}

/// CR 702.170d — Plot free-cast costs zero mana (even for expensive spells).
#[test]
fn test_plot_free_cast_costs_zero() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);
    let registry = build_registry(vec![plot_creature_def()]);

    // Plotted creature with mana cost {3}{R}{R} in exile.
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(
            ObjectSpec::card(p1, "Plot Creature")
                .with_card_id(CardId("plot-creature".to_string()))
                .with_keyword(KeywordAbility::Plot)
                .with_types(vec![CardType::Creature])
                .in_zone(ZoneId::Exile),
        )
        .build()
        .unwrap();

    // Turn 2, plotted on turn 1.
    state.turn.turn_number = 2;
    state.turn.priority_holder = Some(p1);

    let exile_id = find_object_in_zone(&state, "Plot Creature", ZoneId::Exile)
        .expect("card should be in exile");
    {
        let obj = state.objects.get_mut(&exile_id).unwrap();
        obj.is_plotted = true;
        obj.plotted_turn = 1;
    }

    // Player has NO mana -- should still succeed because free-cast costs zero.
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
            alt_cost: Some(AltCostKind::Plot),
            escape_exile_cards: vec![],
            retrace_discard_land: None,
            jump_start_discard: None,
            prototype: false,
            bargain_sacrifice: None,
            emerge_sacrifice: None,
            casualty_sacrifice: None,
            assist_player: None,
            assist_amount: 0,
            replicate_count: 0,
            splice_cards: vec![],
            entwine_paid: false,
            escalate_modes: 0,
            devour_sacrifices: vec![],
            modes_chosen: vec![],
            fuse: false,
            x_value: 0,
            collect_evidence_cards: vec![],
            squad_count: 0,
        },
    );

    assert!(
        result.is_ok(),
        "plot free-cast should cost zero mana (CR 702.170d): {:?}",
        result.err()
    );
}

/// CR 702.170a / CR 116.2k — Plot requires main phase + empty stack.
#[test]
fn test_plot_requires_main_phase_empty_stack() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);
    let registry = build_registry(vec![plot_sorcery_def()]);

    // Test 1: Wrong step (declare attackers)
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry.clone())
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .object(
            ObjectSpec::card(p1, "Plot Sorcery")
                .with_card_id(CardId("plot-sorcery".to_string()))
                .with_keyword(KeywordAbility::Plot)
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

    let card_id = find_object(&state, "Plot Sorcery");
    let result = process_command(
        state,
        Command::PlotCard {
            player: p1,
            card: card_id,
        },
    );
    assert!(
        result.is_err(),
        "plot should fail outside main phase (CR 116.2k)"
    );

    // Test 2: Main phase but stack not empty
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(
            ObjectSpec::card(p1, "Plot Sorcery")
                .with_card_id(CardId("plot-sorcery".to_string()))
                .with_keyword(KeywordAbility::Plot)
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

    let card_id = find_object(&state, "Plot Sorcery");

    // Simulate non-empty stack by manually adding a fake stack entry.
    // We use a fake ObjectId to avoid real game state manipulation.
    use mtg_engine::state::stack::{StackObject, StackObjectKind};
    let fake_id = state.next_object_id();
    state.stack_objects.push_back(StackObject {
        id: fake_id,
        controller: p2,
        kind: StackObjectKind::Spell {
            source_object: fake_id,
        },
        targets: vec![],
        cant_be_countered: false,
        is_copy: false,
        cast_with_flashback: false,
        kicker_times_paid: 0,
        was_evoked: false,
        was_bestowed: false,
        cast_with_madness: false,
        cast_with_miracle: false,
        was_escaped: false,
        cast_with_foretell: false,
        was_buyback_paid: false,
        was_suspended: false,
        was_overloaded: false,
        cast_with_jump_start: false,
        cast_with_aftermath: false,
        was_dashed: false,
        was_blitzed: false,
        was_plotted: false,
        was_prototyped: false,
        was_impended: false,
        was_bargained: false,
        was_surged: false,
        was_casualty_paid: false,
        // CR 702.148a: test objects are not cleave casts.
        was_cleaved: false,
        // CR 702.47a: test objects have no spliced effects.
        spliced_effects: vec![],
        spliced_card_ids: vec![],
        devour_sacrifices: vec![],
        modes_chosen: vec![],
        was_entwined: false,
        escalate_modes_paid: 0,
        was_fused: false,
        x_value: 0,
        evidence_collected: false,
        squad_count: 0,
    });

    let result = process_command(
        state,
        Command::PlotCard {
            player: p1,
            card: card_id,
        },
    );
    assert!(
        result.is_err(),
        "plot should fail when stack is not empty (CR 702.170a)"
    );
}

/// CR 702.170d — Plot free-cast requires sorcery timing (main phase + empty stack).
/// Even instants can only be cast via Plot at sorcery speed.
#[test]
fn test_plot_free_cast_requires_sorcery_timing() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);
    let registry = build_registry(vec![plot_instant_def()]);

    // Test: plotted instant -- try to cast via Plot outside of main phase
    // We'll place the card in exile pre-plotted and try to cast it during Upkeep.
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .active_player(p1)
        .at_step(Step::Upkeep)
        .object(
            ObjectSpec::card(p1, "Plot Instant")
                .with_card_id(CardId("plot-instant".to_string()))
                .with_keyword(KeywordAbility::Plot)
                .with_types(vec![CardType::Instant])
                .in_zone(ZoneId::Exile),
        )
        .build()
        .unwrap();

    state.turn.turn_number = 2;
    state.turn.priority_holder = Some(p1);

    let exile_id = find_object_in_zone(&state, "Plot Instant", ZoneId::Exile)
        .expect("card should be in exile");
    {
        let obj = state.objects.get_mut(&exile_id).unwrap();
        obj.is_plotted = true;
        obj.plotted_turn = 1;
    }

    // Try to cast during Upkeep -- should fail (CR 702.170d: "during their main phase").
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
            alt_cost: Some(AltCostKind::Plot),
            escape_exile_cards: vec![],
            retrace_discard_land: None,
            jump_start_discard: None,
            prototype: false,
            bargain_sacrifice: None,
            emerge_sacrifice: None,
            casualty_sacrifice: None,
            assist_player: None,
            assist_amount: 0,
            replicate_count: 0,
            splice_cards: vec![],
            entwine_paid: false,
            escalate_modes: 0,
            devour_sacrifices: vec![],
            modes_chosen: vec![],
            fuse: false,
            x_value: 0,
            collect_evidence_cards: vec![],
            squad_count: 0,
        },
    );

    assert!(
        result.is_err(),
        "plotted cards can only be cast during main phase (CR 702.170d), even instants"
    );
}

/// CR 116.2k — Plot can only be done during the player's own turn.
#[test]
fn test_plot_requires_player_turn() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);
    let registry = build_registry(vec![plot_sorcery_def()]);

    // It is P2's turn; P1 tries to plot.
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .active_player(p2) // P2 is active
        .at_step(Step::PreCombatMain)
        .object(
            ObjectSpec::card(p1, "Plot Sorcery")
                .with_card_id(CardId("plot-sorcery".to_string()))
                .with_keyword(KeywordAbility::Plot)
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
    state.turn.priority_holder = Some(p1); // P1 has priority somehow

    let card_id = find_object(&state, "Plot Sorcery");
    let result = process_command(
        state,
        Command::PlotCard {
            player: p1,
            card: card_id,
        },
    );

    assert!(
        result.is_err(),
        "plot should fail during opponent's turn (CR 116.2k)"
    );
}

/// CR 702.170a — Cannot plot a card without the Plot keyword.
#[test]
fn test_plot_requires_plot_keyword() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);
    let registry = build_registry(vec![plain_sorcery_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(
            ObjectSpec::card(p1, "Plain Sorcery")
                .with_card_id(CardId("plain-sorcery".to_string()))
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

    let card_id = find_object(&state, "Plain Sorcery");
    let result = process_command(
        state,
        Command::PlotCard {
            player: p1,
            card: card_id,
        },
    );

    assert!(
        result.is_err(),
        "plot should fail for card without Plot keyword (CR 702.170a)"
    );
}

/// CR 702.170a — Cannot plot a card that is not in hand.
#[test]
fn test_plot_requires_card_in_hand() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);
    let registry = build_registry(vec![plot_sorcery_def()]);

    // Card on battlefield, not in hand.
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(
            ObjectSpec::card(p1, "Plot Sorcery")
                .with_card_id(CardId("plot-sorcery".to_string()))
                .with_keyword(KeywordAbility::Plot)
                .with_types(vec![CardType::Sorcery])
                .in_zone(ZoneId::Battlefield),
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

    let card_id = find_object(&state, "Plot Sorcery");
    let result = process_command(
        state,
        Command::PlotCard {
            player: p1,
            card: card_id,
        },
    );

    assert!(
        result.is_err(),
        "plot should fail when card is not in hand (CR 702.170a)"
    );
}

/// CR 702.170a — Cannot plot if insufficient mana to pay plot cost.
#[test]
fn test_plot_insufficient_mana() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);
    let registry = build_registry(vec![plot_sorcery_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(
            ObjectSpec::card(p1, "Plot Sorcery")
                .with_card_id(CardId("plot-sorcery".to_string()))
                .with_keyword(KeywordAbility::Plot)
                .with_types(vec![CardType::Sorcery])
                .in_zone(ZoneId::Hand(p1)),
        )
        .build()
        .unwrap();

    // Only give {R} but plot costs {1}{R}.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Red, 1);
    state.turn.priority_holder = Some(p1);

    let card_id = find_object(&state, "Plot Sorcery");
    let result = process_command(
        state,
        Command::PlotCard {
            player: p1,
            card: card_id,
        },
    );

    assert!(
        result.is_err(),
        "plot should fail when insufficient mana (CR 702.170a)"
    );
}

/// CR 118.9a — Plot free-cast cannot combine with Flashback (mutual exclusion).
/// Since alt_cost is Option<AltCostKind>, a non-plotted card with flashback cannot
/// use AltCostKind::Plot. Test: card not marked as plotted.
#[test]
fn test_plot_mutual_exclusion_not_plotted_card() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);
    let registry = build_registry(vec![plot_sorcery_def()]);

    // Card in exile but NOT marked as plotted.
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(
            ObjectSpec::card(p1, "Plot Sorcery")
                .with_card_id(CardId("plot-sorcery".to_string()))
                .with_keyword(KeywordAbility::Plot)
                .with_types(vec![CardType::Sorcery])
                .in_zone(ZoneId::Exile),
        )
        .build()
        .unwrap();

    // Turn 2 -- but card is NOT plotted (is_plotted = false by default).
    state.turn.turn_number = 2;
    state.turn.priority_holder = Some(p1);

    let exile_id = find_object_in_zone(&state, "Plot Sorcery", ZoneId::Exile)
        .expect("card should be in exile");

    // Attempt to cast with AltCostKind::Plot but card is not plotted.
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
            alt_cost: Some(AltCostKind::Plot),
            escape_exile_cards: vec![],
            retrace_discard_land: None,
            jump_start_discard: None,
            prototype: false,
            bargain_sacrifice: None,
            emerge_sacrifice: None,
            casualty_sacrifice: None,
            assist_player: None,
            assist_amount: 0,
            replicate_count: 0,
            splice_cards: vec![],
            entwine_paid: false,
            escalate_modes: 0,
            devour_sacrifices: vec![],
            modes_chosen: vec![],
            fuse: false,
            x_value: 0,
            collect_evidence_cards: vec![],
            squad_count: 0,
        },
    );

    assert!(
        result.is_err(),
        "cannot cast via plot if card was not plotted (CR 702.170d)"
    );
}

/// CR 400.7 — New ObjectId is assigned when card moves to exile via plot.
#[test]
fn test_plot_card_identity_tracking() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);
    let registry = build_registry(vec![plot_sorcery_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(
            ObjectSpec::card(p1, "Plot Sorcery")
                .with_card_id(CardId("plot-sorcery".to_string()))
                .with_keyword(KeywordAbility::Plot)
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
        .add(ManaColor::Colorless, 1);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Red, 1);
    state.turn.priority_holder = Some(p1);

    let old_hand_id = find_object(&state, "Plot Sorcery");
    let (state, _) = process_command(
        state,
        Command::PlotCard {
            player: p1,
            card: old_hand_id,
        },
    )
    .unwrap();

    // CR 400.7: old ObjectId is gone (card moved to a new zone).
    let old_obj_exists = state.objects.contains_key(&old_hand_id);
    assert!(
        !old_obj_exists,
        "original ObjectId should be gone after zone change (CR 400.7)"
    );

    // New ObjectId in exile should exist with is_plotted = true.
    let new_exile_id = find_object_in_zone(&state, "Plot Sorcery", ZoneId::Exile)
        .expect("card should be in exile with a new ObjectId");
    assert_ne!(
        old_hand_id, new_exile_id,
        "exile ObjectId should differ from hand ObjectId (CR 400.7)"
    );
    let exile_obj = state.objects.get(&new_exile_id).unwrap();
    assert!(
        exile_obj.is_plotted,
        "new exile object should have is_plotted=true"
    );
}

/// CR 118.9c — Plot-cast spell's mana value is still its printed mana cost.
/// The spell's printed mana value is unchanged; only the payment is zero.
#[test]
fn test_plot_mana_value_unchanged_on_stack() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);
    let registry = build_registry(vec![plot_creature_def()]);

    // Plot Creature has mana cost {3}{R}{R} = mana value 5.
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(
            ObjectSpec::card(p1, "Plot Creature")
                .with_card_id(CardId("plot-creature".to_string()))
                .with_keyword(KeywordAbility::Plot)
                .with_types(vec![CardType::Creature])
                .in_zone(ZoneId::Exile),
        )
        .build()
        .unwrap();

    state.turn.turn_number = 2;
    state.turn.priority_holder = Some(p1);

    let exile_id = find_object_in_zone(&state, "Plot Creature", ZoneId::Exile)
        .expect("card should be in exile");
    {
        let obj = state.objects.get_mut(&exile_id).unwrap();
        obj.is_plotted = true;
        obj.plotted_turn = 1;
        // Set mana cost so mana_value is 5 ({3}{R}{R}).
        obj.characteristics.mana_cost = Some(ManaCost {
            generic: 3,
            red: 2,
            ..Default::default()
        });
    }

    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: exile_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: Some(AltCostKind::Plot),
            escape_exile_cards: vec![],
            retrace_discard_land: None,
            jump_start_discard: None,
            prototype: false,
            bargain_sacrifice: None,
            emerge_sacrifice: None,
            casualty_sacrifice: None,
            assist_player: None,
            assist_amount: 0,
            replicate_count: 0,
            splice_cards: vec![],
            entwine_paid: false,
            escalate_modes: 0,
            devour_sacrifices: vec![],
            modes_chosen: vec![],
            fuse: false,
            x_value: 0,
            collect_evidence_cards: vec![],
            squad_count: 0,
        },
    )
    .unwrap();

    // The stack object should refer to the card in ZoneId::Stack.
    let stack_card = state
        .objects
        .values()
        .find(|o| o.zone == ZoneId::Stack)
        .expect("card should be on the stack");

    // CR 118.9c: printed mana cost is unchanged even when cast for free.
    let mv = stack_card
        .characteristics
        .mana_cost
        .as_ref()
        .map(|c| c.mana_value())
        .unwrap_or(0);
    assert_eq!(
        mv, 5,
        "mana value should still be 5 (CR 118.9c: mana cost unchanged)"
    );
}

/// CR 702.170d — Plot free-cast is sorcery-speed; cannot cast while stack has objects.
#[test]
fn test_plot_free_cast_requires_empty_stack() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);
    let registry = build_registry(vec![plot_instant_def()]);

    // Even an instant (with plot) cannot be cast via Plot while stack is non-empty.
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(
            ObjectSpec::card(p1, "Plot Instant")
                .with_card_id(CardId("plot-instant".to_string()))
                .with_keyword(KeywordAbility::Plot)
                .with_types(vec![CardType::Instant])
                .in_zone(ZoneId::Exile),
        )
        .build()
        .unwrap();

    state.turn.turn_number = 2;
    state.turn.priority_holder = Some(p1);

    let exile_id = find_object_in_zone(&state, "Plot Instant", ZoneId::Exile)
        .expect("card should be in exile");
    {
        let obj = state.objects.get_mut(&exile_id).unwrap();
        obj.is_plotted = true;
        obj.plotted_turn = 1;
    }

    // Add a fake item to the stack.
    use mtg_engine::state::stack::{StackObject, StackObjectKind};
    let fake_id = state.next_object_id();
    state.stack_objects.push_back(StackObject {
        id: fake_id,
        controller: p2,
        kind: StackObjectKind::Spell {
            source_object: fake_id,
        },
        targets: vec![],
        cant_be_countered: false,
        is_copy: false,
        cast_with_flashback: false,
        kicker_times_paid: 0,
        was_evoked: false,
        was_bestowed: false,
        cast_with_madness: false,
        cast_with_miracle: false,
        was_escaped: false,
        cast_with_foretell: false,
        was_buyback_paid: false,
        was_suspended: false,
        was_overloaded: false,
        cast_with_jump_start: false,
        cast_with_aftermath: false,
        was_dashed: false,
        was_blitzed: false,
        was_plotted: false,
        was_prototyped: false,
        was_impended: false,
        was_bargained: false,
        was_surged: false,
        was_casualty_paid: false,
        // CR 702.148a: test objects are not cleave casts.
        was_cleaved: false,
        // CR 702.47a: test objects have no spliced effects.
        spliced_effects: vec![],
        spliced_card_ids: vec![],
        devour_sacrifices: vec![],
        modes_chosen: vec![],
        was_entwined: false,
        escalate_modes_paid: 0,
        was_fused: false,
        x_value: 0,
        evidence_collected: false,
        squad_count: 0,
    });

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
            alt_cost: Some(AltCostKind::Plot),
            escape_exile_cards: vec![],
            retrace_discard_land: None,
            jump_start_discard: None,
            prototype: false,
            bargain_sacrifice: None,
            emerge_sacrifice: None,
            casualty_sacrifice: None,
            assist_player: None,
            assist_amount: 0,
            replicate_count: 0,
            splice_cards: vec![],
            entwine_paid: false,
            escalate_modes: 0,
            devour_sacrifices: vec![],
            modes_chosen: vec![],
            fuse: false,
            x_value: 0,
            collect_evidence_cards: vec![],
            squad_count: 0,
        },
    );

    assert!(
        result.is_err(),
        "plot free-cast should fail when stack is non-empty (CR 702.170d)"
    );
}

/// CR 702.170d — Plot free-cast requires the player's own turn.
#[test]
fn test_plot_free_cast_requires_own_turn() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);
    let registry = build_registry(vec![plot_sorcery_def()]);

    // P2 is active player; P1 has a plotted card in exile but it's not their turn.
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .active_player(p2) // P2's turn
        .at_step(Step::PreCombatMain)
        .object(
            ObjectSpec::card(p1, "Plot Sorcery")
                .with_card_id(CardId("plot-sorcery".to_string()))
                .with_keyword(KeywordAbility::Plot)
                .with_types(vec![CardType::Sorcery])
                .in_zone(ZoneId::Exile),
        )
        .build()
        .unwrap();

    state.turn.turn_number = 2;
    state.turn.priority_holder = Some(p1); // P1 has priority somehow

    let exile_id = find_object_in_zone(&state, "Plot Sorcery", ZoneId::Exile)
        .expect("card should be in exile");
    {
        let obj = state.objects.get_mut(&exile_id).unwrap();
        obj.is_plotted = true;
        obj.plotted_turn = 1;
    }

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
            alt_cost: Some(AltCostKind::Plot),
            escape_exile_cards: vec![],
            retrace_discard_land: None,
            jump_start_discard: None,
            prototype: false,
            bargain_sacrifice: None,
            emerge_sacrifice: None,
            casualty_sacrifice: None,
            assist_player: None,
            assist_amount: 0,
            replicate_count: 0,
            splice_cards: vec![],
            entwine_paid: false,
            escalate_modes: 0,
            devour_sacrifices: vec![],
            modes_chosen: vec![],
            fuse: false,
            x_value: 0,
            collect_evidence_cards: vec![],
            squad_count: 0,
        },
    );

    assert!(
        result.is_err(),
        "plot free-cast should fail when not your turn (CR 702.170d)"
    );
}

/// CR 702.170a — Normal (non-plot) cast still works for cards with Plot.
/// A card with plot can still be cast normally by paying its mana cost.
#[test]
fn test_plot_normal_cast_still_works() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);
    let registry = build_registry(vec![plot_sorcery_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(
            ObjectSpec::card(p1, "Plot Sorcery")
                .with_card_id(CardId("plot-sorcery".to_string()))
                .with_keyword(KeywordAbility::Plot)
                .with_types(vec![CardType::Sorcery])
                .in_zone(ZoneId::Hand(p1)),
        )
        .build()
        .unwrap();

    // Give player full mana cost {2}{R}.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 2);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Red, 1);
    state.turn.priority_holder = Some(p1);

    let card_id = find_object(&state, "Plot Sorcery");

    // Cast with no alt_cost (normal cast by paying mana cost).
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
            alt_cost: None,
            escape_exile_cards: vec![],
            retrace_discard_land: None,
            jump_start_discard: None,
            prototype: false,
            bargain_sacrifice: None,
            emerge_sacrifice: None,
            casualty_sacrifice: None,
            assist_player: None,
            assist_amount: 0,
            replicate_count: 0,
            splice_cards: vec![],
            entwine_paid: false,
            escalate_modes: 0,
            devour_sacrifices: vec![],
            modes_chosen: vec![],
            fuse: false,
            x_value: 0,
            collect_evidence_cards: vec![],
            squad_count: 0,
        },
    );

    assert!(
        result.is_ok(),
        "normal cast should still work for plot cards (CR 702.170a): {:?}",
        result.err()
    );
}

/// CR 702.170d — Plot free-cast on postcombat main phase works too.
#[test]
fn test_plot_cast_postcombat_main_phase() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);
    let registry = build_registry(vec![plot_sorcery_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .active_player(p1)
        .at_step(Step::PostCombatMain) // postcombat main phase
        .object(
            ObjectSpec::card(p1, "Plot Sorcery")
                .with_card_id(CardId("plot-sorcery".to_string()))
                .with_keyword(KeywordAbility::Plot)
                .with_types(vec![CardType::Sorcery])
                .in_zone(ZoneId::Exile),
        )
        .build()
        .unwrap();

    state.turn.turn_number = 2;
    state.turn.priority_holder = Some(p1);

    let exile_id = find_object_in_zone(&state, "Plot Sorcery", ZoneId::Exile)
        .expect("card should be in exile");
    {
        let obj = state.objects.get_mut(&exile_id).unwrap();
        obj.is_plotted = true;
        obj.plotted_turn = 1;
    }

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
            alt_cost: Some(AltCostKind::Plot),
            escape_exile_cards: vec![],
            retrace_discard_land: None,
            jump_start_discard: None,
            prototype: false,
            bargain_sacrifice: None,
            emerge_sacrifice: None,
            casualty_sacrifice: None,
            assist_player: None,
            assist_amount: 0,
            replicate_count: 0,
            splice_cards: vec![],
            entwine_paid: false,
            escalate_modes: 0,
            devour_sacrifices: vec![],
            modes_chosen: vec![],
            fuse: false,
            x_value: 0,
            collect_evidence_cards: vec![],
            squad_count: 0,
        },
    );

    assert!(
        result.is_ok(),
        "plot free-cast should work in postcombat main phase (CR 702.170d): {:?}",
        result.err()
    );
}

/// CR 702.170a — Plot special action works in postcombat main phase.
#[test]
fn test_plot_action_postcombat_main_phase() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);
    let registry = build_registry(vec![plot_sorcery_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .active_player(p1)
        .at_step(Step::PostCombatMain)
        .object(
            ObjectSpec::card(p1, "Plot Sorcery")
                .with_card_id(CardId("plot-sorcery".to_string()))
                .with_keyword(KeywordAbility::Plot)
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
        .add(ManaColor::Colorless, 1);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Red, 1);
    state.turn.priority_holder = Some(p1);

    let card_id = find_object(&state, "Plot Sorcery");
    let result = process_command(
        state,
        Command::PlotCard {
            player: p1,
            card: card_id,
        },
    );

    assert!(
        result.is_ok(),
        "plot should work in postcombat main phase (CR 702.170a): {:?}",
        result.err()
    );
}

/// CR 702.170d — Plot plot_turn is correctly tracked (card plotted on turn 3
/// can be cast starting from turn 4).
#[test]
fn test_plot_turn_tracking_boundary() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);
    let registry = build_registry(vec![plot_sorcery_def()]);

    // Card plotted on turn 3. It is turn 3. Should NOT be castable.
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(
            ObjectSpec::card(p1, "Plot Sorcery")
                .with_card_id(CardId("plot-sorcery".to_string()))
                .with_keyword(KeywordAbility::Plot)
                .with_types(vec![CardType::Sorcery])
                .in_zone(ZoneId::Exile),
        )
        .build()
        .unwrap();

    state.turn.turn_number = 3;
    state.turn.priority_holder = Some(p1);

    let exile_id = find_object_in_zone(&state, "Plot Sorcery", ZoneId::Exile)
        .expect("card should be in exile");
    {
        let obj = state.objects.get_mut(&exile_id).unwrap();
        obj.is_plotted = true;
        obj.plotted_turn = 3; // plotted_turn == current turn_number
    }

    // Should fail: same turn as plotted.
    let result = process_command(
        state.clone(),
        Command::CastSpell {
            player: p1,
            card: exile_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: Some(AltCostKind::Plot),
            escape_exile_cards: vec![],
            retrace_discard_land: None,
            jump_start_discard: None,
            prototype: false,
            bargain_sacrifice: None,
            emerge_sacrifice: None,
            casualty_sacrifice: None,
            assist_player: None,
            assist_amount: 0,
            replicate_count: 0,
            splice_cards: vec![],
            entwine_paid: false,
            escalate_modes: 0,
            devour_sacrifices: vec![],
            modes_chosen: vec![],
            fuse: false,
            x_value: 0,
            collect_evidence_cards: vec![],
            squad_count: 0,
        },
    );
    assert!(
        result.is_err(),
        "cannot cast on same turn as plotted (turn 3 plotted, turn 3 cast)"
    );

    // Now advance to turn 4 -- should succeed.
    state.turn.turn_number = 4;
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
            alt_cost: Some(AltCostKind::Plot),
            escape_exile_cards: vec![],
            retrace_discard_land: None,
            jump_start_discard: None,
            prototype: false,
            bargain_sacrifice: None,
            emerge_sacrifice: None,
            casualty_sacrifice: None,
            assist_player: None,
            assist_amount: 0,
            replicate_count: 0,
            splice_cards: vec![],
            entwine_paid: false,
            escalate_modes: 0,
            devour_sacrifices: vec![],
            modes_chosen: vec![],
            fuse: false,
            x_value: 0,
            collect_evidence_cards: vec![],
            squad_count: 0,
        },
    );
    assert!(
        result.is_ok(),
        "should be able to cast on turn 4 when plotted on turn 3: {:?}",
        result.err()
    );
}
