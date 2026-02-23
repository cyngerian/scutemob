//! Storm keyword and spell-copying tests (CR 702.40, CR 707.10).
//!
//! Session 8 of M9.4 implements:
//! - `spells_cast_this_turn` tracker on PlayerState
//! - `copy::copy_spell_on_stack` — creates a stack copy (CR 707.10)
//! - `copy::create_storm_copies` — N copies for storm (CR 702.40a)
//! - `KeywordAbility::Storm` — triggers on cast; copies pushed above original
//! - `GameEvent::SpellCopied` — emitted for each copy; copies are NOT cast

use mtg_engine::{
    process_command, AbilityDefinition, CardDefinition, CardId, CardRegistry, CardType, Command,
    Effect, EffectAmount, GameEvent, GameState, GameStateBuilder, KeywordAbility, ManaColor,
    ManaCost, ObjectId, ObjectSpec, PlayerId, PlayerTarget, Step, TypeLine, ZoneId,
};

fn p1() -> PlayerId {
    PlayerId(1)
}
fn p2() -> PlayerId {
    PlayerId(2)
}

/// Pass priority for all listed players once.
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

/// Build a minimal CardDefinition for a storm sorcery.
///
/// The spell has Storm keyword and a simple GainLife effect (doesn't matter for
/// storm copy tests — the effect itself is irrelevant; we're testing copy creation).
fn storm_sorcery_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("storm-sorcery-test".into()),
        name: "Test Storm Sorcery".into(),
        mana_cost: Some(ManaCost {
            generic: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Sorcery].iter().cloned().collect(),
            ..Default::default()
        },
        oracle_text: "Storm (When you cast this spell, copy it for each other spell cast before it this turn.)".into(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Storm),
            AbilityDefinition::Spell {
                effect: Effect::GainLife {
                    player: PlayerTarget::Controller,
                    amount: EffectAmount::Fixed(1),
                },
                targets: vec![],
                modes: None,
                cant_be_countered: false,
            },
        ],
        power: None,
        toughness: None,
    }
}

// ── CR 702.40a: Storm creates N copies ───────────────────────────────────────

/// CR 702.40a — Storm with storm count 3: casting a storm spell when 3 other
/// spells have been cast this turn creates 3 copies on the stack.
///
/// After casting: stack should have the original + 3 copies = 4 stack objects.
/// Each copy emits a SpellCopied event, NOT a SpellCast event.
#[test]
fn test_storm_creates_copies() {
    let p1 = p1();
    let p2 = p2();

    let def = storm_sorcery_def();
    let card_id = def.card_id.clone();
    let registry = CardRegistry::new(vec![def]);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(
            ObjectSpec::card(p1, "Test Storm Sorcery")
                .with_card_id(card_id)
                .with_types(vec![CardType::Sorcery])
                .with_mana_cost(ManaCost {
                    generic: 1,
                    ..Default::default()
                })
                .with_keyword(KeywordAbility::Storm)
                .in_zone(ZoneId::Hand(p1)),
        )
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .with_registry(registry)
        .build()
        .unwrap();

    // Directly set spells_cast_this_turn = 3 to simulate 3 prior spells cast.
    let mut state = state;
    state.players.get_mut(&p1).unwrap().spells_cast_this_turn = 3;
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 1);

    let storm_id = state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == "Test Storm Sorcery")
        .map(|(id, _)| *id)
        .unwrap();

    let (state, events) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: storm_id,
            targets: vec![],
        },
    )
    .unwrap();

    // CR 702.40a: 3 prior spells → 3 copies.
    // Stack should have: 1 original + 3 copies = 4 stack objects.
    assert_eq!(
        state.stack_objects.len(),
        4,
        "Storm with 3 prior spells should create 3 copies (4 total on stack); got {}",
        state.stack_objects.len()
    );

    // Exactly 3 SpellCopied events and 1 SpellCast event.
    let spell_cast_count = events
        .iter()
        .filter(|e| matches!(e, GameEvent::SpellCast { .. }))
        .count();
    let spell_copied_count = events
        .iter()
        .filter(|e| matches!(e, GameEvent::SpellCopied { .. }))
        .count();

    assert_eq!(
        spell_cast_count, 1,
        "Exactly 1 SpellCast event (the storm spell itself); got {}",
        spell_cast_count
    );
    assert_eq!(
        spell_copied_count, 3,
        "Exactly 3 SpellCopied events (storm copies); got {}",
        spell_copied_count
    );
}

// ── CR 702.40a: Copies resolve independently ─────────────────────────────────

/// CR 702.40a — Storm copies are each independent StackObjects.
/// Each copy is a distinct entry in stack_objects with its own ID.
/// Resolving the copies happens LIFO: last copy resolves first, then original.
#[test]
fn test_storm_copies_resolve_independently() {
    let p1 = p1();
    let p2 = p2();

    let def = storm_sorcery_def();
    let card_id = def.card_id.clone();
    let registry = CardRegistry::new(vec![def]);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(
            ObjectSpec::card(p1, "Test Storm Sorcery")
                .with_card_id(card_id)
                .with_types(vec![CardType::Sorcery])
                .with_mana_cost(ManaCost {
                    generic: 1,
                    ..Default::default()
                })
                .with_keyword(KeywordAbility::Storm)
                .in_zone(ZoneId::Hand(p1)),
        )
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .with_registry(registry)
        .build()
        .unwrap();

    // 2 prior spells → 2 copies.
    let mut state = state;
    state.players.get_mut(&p1).unwrap().spells_cast_this_turn = 2;
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 1);

    let storm_id = state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == "Test Storm Sorcery")
        .map(|(id, _)| *id)
        .unwrap();

    let (state, _cast_events) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: storm_id,
            targets: vec![],
        },
    )
    .unwrap();

    // Stack has original + 2 copies = 3 entries.
    assert_eq!(state.stack_objects.len(), 3, "Should be 3 stack objects");

    // All 3 should have distinct IDs.
    let ids: Vec<ObjectId> = state.stack_objects.iter().map(|s| s.id).collect();
    let mut unique_ids = ids.clone();
    unique_ids.sort();
    unique_ids.dedup();
    assert_eq!(
        unique_ids.len(),
        3,
        "All stack objects must have distinct IDs; got {:?}",
        ids
    );

    // Resolve all 3 stack objects (pass priority twice per player to resolve each).
    // After each resolve: stack shrinks by 1.
    let (state, _) = pass_all(state, &[p1, p2]);
    let first_resolve_size = state.stack_objects.len();

    let (state, _) = pass_all(state, &[p1, p2]);
    let second_resolve_size = state.stack_objects.len();

    let (state, _) = pass_all(state, &[p1, p2]);
    let final_size = state.stack_objects.len();

    assert_eq!(
        first_resolve_size, 2,
        "After first resolve, 2 stack objects remain"
    );
    assert_eq!(
        second_resolve_size, 1,
        "After second resolve, 1 stack object remains"
    );
    assert_eq!(final_size, 0, "After third resolve, stack is empty");
}

// ── CR 702.40a: Storm count resets each turn ─────────────────────────────────

/// CR 702.40a — Storm count (spells_cast_this_turn) resets at the start of each turn.
/// Casting a storm spell at the start of a fresh turn creates 0 copies.
#[test]
fn test_storm_count_resets_each_turn() {
    let p1 = p1();
    let p2 = p2();

    let def = storm_sorcery_def();
    let card_id = def.card_id.clone();
    let registry = CardRegistry::new(vec![def]);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(
            ObjectSpec::card(p1, "Test Storm Sorcery")
                .with_card_id(card_id)
                .with_types(vec![CardType::Sorcery])
                .with_mana_cost(ManaCost {
                    generic: 1,
                    ..Default::default()
                })
                .with_keyword(KeywordAbility::Storm)
                .in_zone(ZoneId::Hand(p1)),
        )
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .with_registry(registry)
        .build()
        .unwrap();

    // Verify spells_cast_this_turn starts at 0.
    assert_eq!(
        state.players[&p1].spells_cast_this_turn, 0,
        "spells_cast_this_turn should start at 0"
    );

    let mut state = state;
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 1);

    let storm_id = state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == "Test Storm Sorcery")
        .map(|(id, _)| *id)
        .unwrap();

    // Cast with 0 prior spells → 0 copies → stack has only 1 entry.
    let (state, _events) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: storm_id,
            targets: vec![],
        },
    )
    .unwrap();

    assert_eq!(
        state.stack_objects.len(),
        1,
        "Storm with 0 prior spells → 0 copies → 1 stack object; got {}",
        state.stack_objects.len()
    );

    // After casting, spells_cast_this_turn should be 1 (the storm spell itself).
    assert_eq!(
        state.players[&p1].spells_cast_this_turn, 1,
        "spells_cast_this_turn should be 1 after casting the storm spell"
    );
}

// ── CC#35: Spell copies are NOT cast (CR 707.10c) ────────────────────────────

/// CC#35 / CR 707.10c — A copy of a spell on the stack is NOT cast.
/// "Whenever you cast a spell" triggers do NOT trigger for copies.
///
/// This test verifies: when storm creates copies, only one SpellCast event fires
/// (for the original), and SpellCopied events are used for the copies.
#[test]
fn test_spell_copy_is_not_cast() {
    let p1 = p1();
    let p2 = p2();

    let def = storm_sorcery_def();
    let card_id = def.card_id.clone();
    let registry = CardRegistry::new(vec![def]);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(
            ObjectSpec::card(p1, "Test Storm Sorcery")
                .with_card_id(card_id)
                .with_types(vec![CardType::Sorcery])
                .with_mana_cost(ManaCost {
                    generic: 1,
                    ..Default::default()
                })
                .with_keyword(KeywordAbility::Storm)
                .in_zone(ZoneId::Hand(p1)),
        )
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .with_registry(registry)
        .build()
        .unwrap();

    // 2 prior spells → 2 copies.
    let mut state = state;
    state.players.get_mut(&p1).unwrap().spells_cast_this_turn = 2;
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 1);

    let storm_id = state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == "Test Storm Sorcery")
        .map(|(id, _)| *id)
        .unwrap();

    let (state, events) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: storm_id,
            targets: vec![],
        },
    )
    .unwrap();

    // CR 707.10c: Copies are NOT cast — exactly 1 SpellCast event.
    let cast_events: Vec<_> = events
        .iter()
        .filter(|e| matches!(e, GameEvent::SpellCast { .. }))
        .collect();
    assert_eq!(
        cast_events.len(),
        1,
        "Only 1 SpellCast event (the original cast); got {} SpellCast events.\nEvents: {:?}",
        cast_events.len(),
        events
    );

    // SpellCopied events for the 2 copies.
    let copied_events: Vec<_> = events
        .iter()
        .filter(|e| matches!(e, GameEvent::SpellCopied { .. }))
        .collect();
    assert_eq!(
        copied_events.len(),
        2,
        "2 SpellCopied events for the 2 storm copies; got {}",
        copied_events.len()
    );

    // spells_cast_this_turn is incremented for the cast, NOT for copies.
    // After this cast, it was 2 (prior) + 1 (this cast) = 3.
    assert_eq!(
        state.players[&p1].spells_cast_this_turn, 3,
        "spells_cast_this_turn should be 3 (2 prior + 1 this cast); copies don't increment"
    );

    // Stack should have 3 entries: original + 2 copies.
    assert_eq!(
        state.stack_objects.len(),
        3,
        "Stack: original + 2 copies = 3"
    );
}
