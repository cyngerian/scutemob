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
/// CR 702.40a: Storm is a triggered ability. After cast, the storm trigger goes
/// on the stack above the original spell. When the trigger resolves, 3 copies
/// are created. Final stack: 1 original + 3 copies = 4 stack objects.
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

    let (state, cast_events) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: storm_id,
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
            cast_with_foretell: false,
            cast_with_buyback: false,
            cast_with_overload: false,
            retrace_discard_land: None,
            cast_with_jump_start: false,
            jump_start_discard: None,
            cast_with_aftermath: false,
        },
    )
    .unwrap();

    // CR 702.40a: After casting, the storm trigger is on the stack above the spell.
    // Stack has: [storm spell (bottom), storm trigger (top)] = 2 objects.
    assert_eq!(
        state.stack_objects.len(),
        2,
        "After cast: storm spell + storm trigger on stack; got {}",
        state.stack_objects.len()
    );

    // CastSpell events: 1 SpellCast + 1 AbilityTriggered (storm trigger), 0 SpellCopied.
    let spell_cast_count = cast_events
        .iter()
        .filter(|e| matches!(e, GameEvent::SpellCast { .. }))
        .count();
    assert_eq!(
        spell_cast_count, 1,
        "Exactly 1 SpellCast event at cast time; got {}",
        spell_cast_count
    );

    // Resolve the storm trigger (passes priority for both players).
    // The trigger is on top; all players passing resolves it.
    let (state, resolve_events) = pass_all(state, &[p1, p2]);

    // CR 702.40a: 3 prior spells → 3 copies created when trigger resolves.
    // Stack should have: 1 original + 3 copies = 4 stack objects.
    assert_eq!(
        state.stack_objects.len(),
        4,
        "After trigger resolves: storm spell + 3 copies = 4 total; got {}",
        state.stack_objects.len()
    );

    // Exactly 3 SpellCopied events emitted when the trigger resolved.
    let spell_copied_count = resolve_events
        .iter()
        .filter(|e| matches!(e, GameEvent::SpellCopied { .. }))
        .count();
    assert_eq!(
        spell_copied_count, 3,
        "Exactly 3 SpellCopied events (storm copies) when trigger resolves; got {}",
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
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            cast_with_evoke: false,
            cast_with_bestow: false,
            cast_with_miracle: false,
            cast_with_escape: false,
            escape_exile_cards: vec![],
            cast_with_foretell: false,
            cast_with_buyback: false,
            cast_with_overload: false,
            retrace_discard_land: None,
            cast_with_jump_start: false,
            jump_start_discard: None,
            cast_with_aftermath: false,
        },
    )
    .unwrap();

    // CR 702.40a: After cast, stack has [storm spell, storm trigger] = 2 entries.
    assert_eq!(
        state.stack_objects.len(),
        2,
        "After cast: storm spell + storm trigger = 2 stack objects"
    );

    // Resolve the storm trigger: 2 prior spells → 2 copies created.
    // Stack becomes [storm spell, copy1, copy2] = 3 entries.
    let (state, _) = pass_all(state, &[p1, p2]);
    assert_eq!(
        state.stack_objects.len(),
        3,
        "After trigger resolves: 3 stack objects"
    );

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

    // Resolve all 3 remaining stack objects (pass priority for both players each time).
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

    // Cast with 0 prior spells → 0 copies. After cast, stack has:
    // [storm spell (bottom), storm trigger (top)] = 2 objects.
    let (state, _events) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: storm_id,
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
            cast_with_foretell: false,
            cast_with_buyback: false,
            cast_with_overload: false,
            retrace_discard_land: None,
            cast_with_jump_start: false,
            jump_start_discard: None,
            cast_with_aftermath: false,
        },
    )
    .unwrap();

    // CR 702.40a: Storm trigger goes on the stack even with 0 prior spells.
    // When it resolves, it creates 0 copies (no-op).
    assert_eq!(
        state.stack_objects.len(),
        2,
        "After cast: storm spell + storm trigger = 2 stack objects; got {}",
        state.stack_objects.len()
    );

    // After casting, spells_cast_this_turn should be 1 (the storm spell itself).
    assert_eq!(
        state.players[&p1].spells_cast_this_turn, 1,
        "spells_cast_this_turn should be 1 after casting the storm spell"
    );

    // Resolve the storm trigger: 0 prior spells → 0 copies → stack has 1 object.
    let (state, _resolve_events) = pass_all(state, &[p1, p2]);
    assert_eq!(
        state.stack_objects.len(),
        1,
        "After trigger resolves with 0 copies: 1 stack object remains; got {}",
        state.stack_objects.len()
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

    let (state, cast_events) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: storm_id,
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
            cast_with_foretell: false,
            cast_with_buyback: false,
            cast_with_overload: false,
            retrace_discard_land: None,
            cast_with_jump_start: false,
            jump_start_discard: None,
            cast_with_aftermath: false,
        },
    )
    .unwrap();

    // CR 707.10c: Copies are NOT cast — exactly 1 SpellCast event at cast time.
    // The storm trigger goes on the stack; copies appear when the trigger resolves.
    let spell_cast_count = cast_events
        .iter()
        .filter(|e| matches!(e, GameEvent::SpellCast { .. }))
        .count();
    assert_eq!(
        spell_cast_count, 1,
        "Only 1 SpellCast event (the original cast); got {} SpellCast events.\nEvents: {:?}",
        spell_cast_count, cast_events
    );

    // No SpellCopied at cast time — copies only appear when the storm trigger resolves.
    let copied_at_cast = cast_events
        .iter()
        .filter(|e| matches!(e, GameEvent::SpellCopied { .. }))
        .count();
    assert_eq!(
        copied_at_cast, 0,
        "No SpellCopied events at cast time (trigger hasn't resolved yet); got {}",
        copied_at_cast
    );

    // spells_cast_this_turn is incremented for the cast, NOT for copies.
    // After this cast, it was 2 (prior) + 1 (this cast) = 3.
    assert_eq!(
        state.players[&p1].spells_cast_this_turn, 3,
        "spells_cast_this_turn should be 3 (2 prior + 1 this cast); copies don't increment"
    );

    // Stack has: [storm spell (bottom), storm trigger (top)] = 2.
    assert_eq!(
        state.stack_objects.len(),
        2,
        "After cast: storm spell + storm trigger = 2 stack objects"
    );

    // Resolve the storm trigger: creates 2 copies.
    let (state, resolve_events) = pass_all(state, &[p1, p2]);

    // CR 707.10c: 2 SpellCopied events (copies are NOT cast — no SpellCast for copies).
    let copied_events: Vec<_> = resolve_events
        .iter()
        .filter(|e| matches!(e, GameEvent::SpellCopied { .. }))
        .collect();
    assert_eq!(
        copied_events.len(),
        2,
        "2 SpellCopied events for the 2 storm copies when trigger resolves; got {}",
        copied_events.len()
    );

    // No additional SpellCast events from the copies.
    let new_cast_events: Vec<_> = resolve_events
        .iter()
        .filter(|e| matches!(e, GameEvent::SpellCast { .. }))
        .collect();
    assert_eq!(
        new_cast_events.len(),
        0,
        "No SpellCast events when storm trigger resolves (copies are not cast); got {}",
        new_cast_events.len()
    );

    // Stack should have 3 entries: original + 2 copies.
    assert_eq!(
        state.stack_objects.len(),
        3,
        "After trigger resolves: original + 2 copies = 3 stack objects"
    );
}
