//! Gravestorm keyword ability tests (CR 702.69).
//!
//! Gravestorm is a triggered ability: "When you cast this spell, copy it for each
//! permanent that was put into a graveyard from the battlefield this turn. If the
//! spell has any targets, you may choose new targets for any of the copies."
//!
//! Key rules verified:
//! - Casting a gravestorm spell creates a GravestormTrigger on the stack (CR 702.69a).
//! - Resolving the trigger creates N copies (N = permanents_put_into_graveyard_this_turn).
//! - Zero count: trigger still fires but creates 0 copies (CR 702.69a).
//! - Copies have `is_copy: true` and do NOT increment `spells_cast_this_turn` (CR 707.10).
//! - Counter increments when a permanent moves from Battlefield to Graveyard (CR 702.69a).
//! - Counter does NOT increment for non-battlefield-to-graveyard moves (CR 702.69a).
//! - Counter resets to 0 at the start of each new turn (CR 702.69a "this turn").
//! - Counter includes permanents from ALL players (ruling 2024-02-02).

use mtg_engine::{
    process_command, AbilityDefinition, CardDefinition, CardId, CardRegistry, CardType, Command,
    Effect, EffectAmount, GameEvent, GameState, GameStateBuilder, KeywordAbility, ManaColor,
    ManaCost, ObjectId, ObjectSpec, PlayerId, PlayerTarget, StackObjectKind, Step, TypeLine,
    ZoneId,
};

// ── Helpers ───────────────────────────────────────────────────────────────────

fn p1() -> PlayerId {
    PlayerId(1)
}
fn p2() -> PlayerId {
    PlayerId(2)
}

fn find_object(state: &GameState, name: &str) -> ObjectId {
    state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{}' not found", name))
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

/// Cast a spell (no cost — mana is pre-loaded).
fn cast_spell(state: GameState, player: PlayerId, card: ObjectId) -> (GameState, Vec<GameEvent>) {
    process_command(
        state,
        Command::CastSpell {
            player,
            card,
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
    .unwrap_or_else(|e| panic!("CastSpell failed: {:?}", e))
}

// ── Card definitions ──────────────────────────────────────────────────────────

/// Synthetic Gravestorm sorcery: "Gravestorm. You gain 1 life."
///
/// Mana cost: {1} (generic). Gravestorm is the keyword being tested;
/// the GainLife effect is irrelevant to copy mechanics.
fn gravestorm_sorcery_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("gravestorm-test".to_string()),
        name: "Test Gravestorm Sorcery".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Sorcery].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Gravestorm (When you cast this spell, copy it for each permanent \
                      that was put into a graveyard from the battlefield this turn.)\n\
                      You gain 1 life."
            .to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Gravestorm),
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
        ..Default::default()
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

/// CR 702.69a — Gravestorm with count 3: casting a gravestorm spell when 3 permanents
/// have been put into a graveyard from the battlefield this turn creates 3 copies.
///
/// After trigger resolves: 1 original + 3 copies = 4 stack objects.
/// Each copy emits a SpellCopied event, NOT a SpellCast event.
#[test]
fn test_gravestorm_basic_creates_copies() {
    let p1 = p1();
    let p2 = p2();

    let def = gravestorm_sorcery_def();
    let card_id = def.card_id.clone();
    let registry = CardRegistry::new(vec![def]);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(
            ObjectSpec::card(p1, "Test Gravestorm Sorcery")
                .with_card_id(card_id)
                .with_types(vec![CardType::Sorcery])
                .with_mana_cost(ManaCost {
                    generic: 1,
                    ..Default::default()
                })
                .with_keyword(KeywordAbility::Gravestorm)
                .in_zone(ZoneId::Hand(p1)),
        )
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .with_registry(registry)
        .build()
        .unwrap();

    // Directly set permanents_put_into_graveyard_this_turn = 3 to simulate 3 deaths.
    let mut state = state;
    state.permanents_put_into_graveyard_this_turn = 3;
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 1);

    let spell_id = find_object(&state, "Test Gravestorm Sorcery");
    let (state, cast_events) = cast_spell(state, p1, spell_id);

    // After casting: stack has [gravestorm spell (bottom), gravestorm trigger (top)] = 2 objects.
    assert_eq!(
        state.stack_objects.len(),
        2,
        "After cast: gravestorm spell + trigger on stack; got {}",
        state.stack_objects.len()
    );

    // Verify the trigger has the correct gravestorm_count.
    let trigger = state.stack_objects.back().expect("trigger expected");
    match &trigger.kind {
        StackObjectKind::KeywordTrigger {
            keyword: KeywordAbility::Gravestorm,
            data: mtg_engine::state::stack::TriggerData::SpellCopy { copy_count, .. },
            ..
        } => {
            assert_eq!(
                *copy_count, 3,
                "Gravestorm KeywordTrigger should capture copy_count 3; got {}",
                copy_count
            );
        }
        other => panic!("Expected Gravestorm KeywordTrigger, got {:?}", other),
    }

    // SpellCast events: exactly 1 SpellCast (the original), 0 SpellCopied at cast time.
    let spell_cast_count = cast_events
        .iter()
        .filter(|e| matches!(e, GameEvent::SpellCast { .. }))
        .count();
    assert_eq!(
        spell_cast_count, 1,
        "Exactly 1 SpellCast at cast time; got {}",
        spell_cast_count
    );

    // Resolve the gravestorm trigger (both players pass).
    let (state, resolve_events) = pass_all(state, &[p1, p2]);

    // CR 702.69a: 3 permanents → 3 copies created.
    // Stack: 1 original + 3 copies = 4 objects.
    assert_eq!(
        state.stack_objects.len(),
        4,
        "After trigger resolves: 1 original + 3 copies = 4; got {}",
        state.stack_objects.len()
    );

    // Verify 3 SpellCopied events (not SpellCast).
    let copied_count = resolve_events
        .iter()
        .filter(|e| matches!(e, GameEvent::SpellCopied { .. }))
        .count();
    assert_eq!(
        copied_count, 3,
        "Exactly 3 SpellCopied events from trigger; got {}",
        copied_count
    );

    // Verify no additional SpellCast events (copies are NOT cast — CR 707.10).
    let new_cast_count = resolve_events
        .iter()
        .filter(|e| matches!(e, GameEvent::SpellCast { .. }))
        .count();
    assert_eq!(
        new_cast_count, 0,
        "No SpellCast events during trigger resolution (copies not cast); got {}",
        new_cast_count
    );

    // Verify all 3 copies have is_copy = true.
    let copy_count_on_stack = state.stack_objects.iter().filter(|s| s.is_copy).count();
    assert_eq!(
        copy_count_on_stack, 3,
        "3 stack objects should have is_copy=true; got {}",
        copy_count_on_stack
    );
}

/// CR 702.69a — Gravestorm with count 0: trigger still fires but creates 0 copies.
///
/// When no permanents have been put into a graveyard from the battlefield this turn,
/// the gravestorm trigger is created but resolves to produce no copies.
/// Stack after trigger resolves: 1 original spell only.
#[test]
fn test_gravestorm_zero_count_no_copies() {
    let p1 = p1();
    let p2 = p2();

    let def = gravestorm_sorcery_def();
    let card_id = def.card_id.clone();
    let registry = CardRegistry::new(vec![def]);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(
            ObjectSpec::card(p1, "Test Gravestorm Sorcery")
                .with_card_id(card_id)
                .with_types(vec![CardType::Sorcery])
                .with_mana_cost(ManaCost {
                    generic: 1,
                    ..Default::default()
                })
                .with_keyword(KeywordAbility::Gravestorm)
                .in_zone(ZoneId::Hand(p1)),
        )
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .with_registry(registry)
        .build()
        .unwrap();

    // permanents_put_into_graveyard_this_turn = 0 (default).
    let mut state = state;
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 1);

    let spell_id = find_object(&state, "Test Gravestorm Sorcery");
    let (state, _) = cast_spell(state, p1, spell_id);

    // Trigger is on the stack (count=0, but trigger was still created).
    assert_eq!(
        state.stack_objects.len(),
        2,
        "After cast: spell + trigger on stack (count=0); got {}",
        state.stack_objects.len()
    );

    let trigger = state.stack_objects.back().expect("trigger");
    match &trigger.kind {
        StackObjectKind::KeywordTrigger {
            keyword: KeywordAbility::Gravestorm,
            data: mtg_engine::state::stack::TriggerData::SpellCopy { copy_count, .. },
            ..
        } => {
            assert_eq!(
                *copy_count, 0,
                "Gravestorm KeywordTrigger copy_count should be 0; got {}",
                copy_count
            );
        }
        other => panic!("Expected GravestormTrigger, got {:?}", other),
    }

    // Resolve trigger (both players pass).
    let (state, resolve_events) = pass_all(state, &[p1, p2]);

    // Zero copies created — stack has only the original spell.
    assert_eq!(
        state.stack_objects.len(),
        1,
        "After trigger resolves with count=0: only original on stack; got {}",
        state.stack_objects.len()
    );

    let copied_count = resolve_events
        .iter()
        .filter(|e| matches!(e, GameEvent::SpellCopied { .. }))
        .count();
    assert_eq!(
        copied_count, 0,
        "No SpellCopied events when count=0; got {}",
        copied_count
    );
}

/// CR 702.69a — Counter increments when a permanent moves from Battlefield to Graveyard.
///
/// Moving a creature directly from the battlefield to its owner's graveyard via
/// `move_object_to_zone` should increment `permanents_put_into_graveyard_this_turn`.
#[test]
fn test_gravestorm_count_increments_on_permanent_dying() {
    let p1 = p1();
    let p2 = p2();

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(ObjectSpec::creature(p1, "Test Creature", 2, 2))
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();

    assert_eq!(
        state.permanents_put_into_graveyard_this_turn, 0,
        "Counter starts at 0"
    );

    let creature_id = find_object(&state, "Test Creature");
    let mut state = state;

    // Simulate death by moving from Battlefield to Graveyard.
    state
        .move_object_to_zone(creature_id, ZoneId::Graveyard(p1))
        .expect("move_object_to_zone should succeed");

    assert_eq!(
        state.permanents_put_into_graveyard_this_turn, 1,
        "Counter should be 1 after one permanent dies; got {}",
        state.permanents_put_into_graveyard_this_turn
    );
}

/// CR 702.69a + ruling 2024-02-02 — Tokens going from battlefield to graveyard count.
///
/// Tokens briefly exist in the graveyard (CR 704.5d) before ceasing to exist as an SBA.
/// The counter must increment when the token enters the graveyard, before SBA removal.
#[test]
fn test_gravestorm_count_includes_tokens() {
    let p1 = p1();
    let p2 = p2();

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(ObjectSpec::creature(p1, "Token Creature", 1, 1).token())
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();

    let token_id = find_object(&state, "Token Creature");
    let mut state = state;

    state
        .move_object_to_zone(token_id, ZoneId::Graveyard(p1))
        .expect("token move should succeed");

    assert_eq!(
        state.permanents_put_into_graveyard_this_turn, 1,
        "Token entering graveyard from battlefield should increment counter; got {}",
        state.permanents_put_into_graveyard_this_turn
    );
}

/// CR 702.69a + ruling 2024-02-02 — Counter includes permanents from ALL players.
///
/// In a multiplayer game, permanents from any player going to any graveyard from
/// the battlefield all count toward the gravestorm total.
#[test]
fn test_gravestorm_count_includes_all_players() {
    let p1 = p1();
    let p2 = p2();

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(ObjectSpec::creature(p1, "P1 Creature", 2, 2))
        .object(ObjectSpec::creature(p2, "P2 Creature", 2, 2))
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();

    let p1_creature_id = find_object(&state, "P1 Creature");
    let p2_creature_id = find_object(&state, "P2 Creature");
    let mut state = state;

    // Move p1's creature to p1's graveyard.
    state
        .move_object_to_zone(p1_creature_id, ZoneId::Graveyard(p1))
        .expect("p1 creature move should succeed");

    assert_eq!(
        state.permanents_put_into_graveyard_this_turn, 1,
        "After p1 creature dies: counter = 1"
    );

    // Move p2's creature to p2's graveyard.
    state
        .move_object_to_zone(p2_creature_id, ZoneId::Graveyard(p2))
        .expect("p2 creature move should succeed");

    assert_eq!(
        state.permanents_put_into_graveyard_this_turn, 2,
        "After p2 creature also dies: counter = 2 (all players); got {}",
        state.permanents_put_into_graveyard_this_turn
    );
}

/// CR 702.69a "this turn" — Counter resets to 0 at the start of each turn.
///
/// The `reset_turn_state` function in `turn_actions.rs` resets
/// `permanents_put_into_graveyard_this_turn` at the start of each new turn.
/// Verified by advancing priority to trigger the untap/upkeep/draw steps.
#[test]
fn test_gravestorm_count_resets_each_turn() {
    let p1 = p1();
    let p2 = p2();

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();

    // Manually set the counter to simulate 2 permanents dying this turn.
    let mut state = state;
    state.permanents_put_into_graveyard_this_turn = 2;

    assert_eq!(
        state.permanents_put_into_graveyard_this_turn, 2,
        "Counter is 2 before turn end"
    );

    // Advance through p1's remaining phases to end turn, then p2 starts their turn.
    // Pass priority through all phases to advance the turn.
    // Phases: PreCombatMain → Combat → PostCombatMain → End → Cleanup → p2 Untap+Upkeep+Draw
    // Each requires both players to pass priority.
    let players = [p1, p2];
    let mut current = state;

    // Advance until it is p2's turn (p2 becomes active player).
    // We pass priority repeatedly until the active player changes.
    let mut turn_changed = false;
    for _ in 0..30 {
        let active = current.turn.active_player;
        let (next, _) = pass_all(current, &players);
        current = next;
        if current.turn.active_player != active {
            turn_changed = true;
            // Check counter reset happened.
            assert_eq!(
                current.permanents_put_into_graveyard_this_turn, 0,
                "Counter should reset to 0 at start of p2's turn; got {}",
                current.permanents_put_into_graveyard_this_turn
            );
            break;
        }
    }

    assert!(turn_changed, "Turn should advance to p2 within 30 passes");
}

/// CR 702.69a — Copies are NOT cast; they do not trigger SpellCast events.
///
/// Copies have is_copy=true. They do not trigger "whenever you cast a spell"
/// abilities and do not increment `spells_cast_this_turn` (CR 707.10).
/// This is inherently handled by `create_storm_copies` / `copy_spell_on_stack`.
#[test]
fn test_gravestorm_copies_not_cast() {
    let p1 = p1();
    let p2 = p2();

    let def = gravestorm_sorcery_def();
    let card_id = def.card_id.clone();
    let registry = CardRegistry::new(vec![def]);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(
            ObjectSpec::card(p1, "Test Gravestorm Sorcery")
                .with_card_id(card_id)
                .with_types(vec![CardType::Sorcery])
                .with_mana_cost(ManaCost {
                    generic: 1,
                    ..Default::default()
                })
                .with_keyword(KeywordAbility::Gravestorm)
                .in_zone(ZoneId::Hand(p1)),
        )
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .with_registry(registry)
        .build()
        .unwrap();

    let mut state = state;
    state.permanents_put_into_graveyard_this_turn = 2;
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 1);

    let spells_before = state.players.get(&p1).unwrap().spells_cast_this_turn;

    let spell_id = find_object(&state, "Test Gravestorm Sorcery");
    let (state, _) = cast_spell(state, p1, spell_id);

    // Resolve trigger.
    let (state, resolve_events) = pass_all(state, &[p1, p2]);

    // 2 copies should be on stack.
    assert_eq!(
        state.stack_objects.len(),
        3,
        "1 original + 2 copies = 3 stack objects; got {}",
        state.stack_objects.len()
    );

    // No SpellCast events during trigger resolution.
    let resolve_cast_events = resolve_events
        .iter()
        .filter(|e| matches!(e, GameEvent::SpellCast { .. }))
        .count();
    assert_eq!(
        resolve_cast_events, 0,
        "No SpellCast during trigger resolution (copies not cast); got {}",
        resolve_cast_events
    );

    // spells_cast_this_turn should only have incremented by 1 (the original cast).
    let spells_after = state.players.get(&p1).unwrap().spells_cast_this_turn;
    assert_eq!(
        spells_after,
        spells_before + 1,
        "spells_cast_this_turn should increment only for the original cast; got before={}, after={}",
        spells_before,
        spells_after
    );
}

/// CR 702.69a — Only battlefield-to-graveyard moves count; not other zone moves.
///
/// Discarding a card from hand to the graveyard must NOT increment the counter.
/// Moving a card from the library to the graveyard must NOT increment the counter.
/// Only `from == Battlefield` and `to == Graveyard(_)` increments the counter.
#[test]
fn test_gravestorm_count_does_not_include_non_battlefield_to_graveyard() {
    let p1 = p1();
    let p2 = p2();

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(
            ObjectSpec::card(p1, "Hand Card")
                .with_types(vec![CardType::Sorcery])
                .in_zone(ZoneId::Hand(p1)),
        )
        .object(
            ObjectSpec::card(p1, "Library Card")
                .with_types(vec![CardType::Sorcery])
                .in_zone(ZoneId::Library(p1)),
        )
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();

    let hand_card_id = find_object(&state, "Hand Card");
    let library_card_id = find_object(&state, "Library Card");
    let mut state = state;

    // Move from Hand → Graveyard (discard): should NOT increment counter.
    state
        .move_object_to_zone(hand_card_id, ZoneId::Graveyard(p1))
        .expect("hand-to-graveyard move should succeed");

    assert_eq!(
        state.permanents_put_into_graveyard_this_turn, 0,
        "Discarding from hand should NOT increment gravestorm counter; got {}",
        state.permanents_put_into_graveyard_this_turn
    );

    // Move from Library → Graveyard (mill): should NOT increment counter.
    state
        .move_object_to_zone(library_card_id, ZoneId::Graveyard(p1))
        .expect("library-to-graveyard move should succeed");

    assert_eq!(
        state.permanents_put_into_graveyard_this_turn, 0,
        "Milling from library should NOT increment gravestorm counter; got {}",
        state.permanents_put_into_graveyard_this_turn
    );
}

/// CR 702.69a — Multiple permanents dying in the same turn accumulate correctly.
///
/// Each permanent entering a graveyard from the battlefield increments the counter by 1.
/// Three creatures dying in one turn should produce a count of 3.
#[test]
fn test_gravestorm_count_accumulates_across_multiple_deaths() {
    let p1 = p1();
    let p2 = p2();

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(ObjectSpec::creature(p1, "Creature A", 1, 1))
        .object(ObjectSpec::creature(p1, "Creature B", 1, 1))
        .object(ObjectSpec::creature(p2, "Creature C", 1, 1))
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();

    let a_id = find_object(&state, "Creature A");
    let b_id = find_object(&state, "Creature B");
    let c_id = find_object(&state, "Creature C");
    let mut state = state;

    state
        .move_object_to_zone(a_id, ZoneId::Graveyard(p1))
        .unwrap();
    assert_eq!(state.permanents_put_into_graveyard_this_turn, 1);

    state
        .move_object_to_zone(b_id, ZoneId::Graveyard(p1))
        .unwrap();
    assert_eq!(state.permanents_put_into_graveyard_this_turn, 2);

    state
        .move_object_to_zone(c_id, ZoneId::Graveyard(p2))
        .unwrap();
    assert_eq!(
        state.permanents_put_into_graveyard_this_turn, 3,
        "Three permanents (including from p2) should produce count=3; got {}",
        state.permanents_put_into_graveyard_this_turn
    );
}
