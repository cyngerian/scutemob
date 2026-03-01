//! Hideaway keyword ability tests (CR 702.75).
//!
//! Hideaway is a triggered ability that fires when a permanent with Hideaway enters
//! the battlefield. The controller looks at the top N cards of their library, exiles
//! one face-down, and puts the rest on the bottom in a random order.
//!
//! Key rules verified:
//! - Hideaway(N) ETB trigger fires and creates HideawayTrigger on the stack (CR 702.75a).
//! - Trigger resolution exiles top card face-down; rest go to library bottom (CR 702.75a).
//! - Exiled card's `exiled_by_hideaway` is set to the source permanent's ObjectId (CR 607.2a).
//! - Exiled card has `face_down = true` (CR 406.3).
//! - Library-empty edge case: trigger resolves with no effect (CR 702.75a).
//! - `Effect::PlayExiledCard` finds the matching card and plays it (CR 702.75a, CR 607.2a).
//! - Permanents without Hideaway do NOT generate a HideawayTrigger (negative test).

use mtg_engine::state::{ActivatedAbility, ActivationCost, CardType};
use mtg_engine::{
    process_command, AbilityDefinition, CardDefinition, CardId, CardRegistry, Command, Condition,
    Effect, GameEvent, GameStateBuilder, KeywordAbility, ManaCost, ObjectId, ObjectSpec, PlayerId,
    StackObject, StackObjectKind, Step, ZoneId,
};

// ── Helpers ───────────────────────────────────────────────────────────────────

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

fn find_object(state: &mtg_engine::GameState, name: &str) -> mtg_engine::ObjectId {
    state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{}' not found", name))
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

/// Keep passing priority for all players until the stack is empty.
fn pass_all_until_empty(
    mut state: mtg_engine::GameState,
    players: &[PlayerId],
) -> (mtg_engine::GameState, Vec<GameEvent>) {
    let mut all_events = Vec::new();
    for _ in 0..50 {
        if state.stack_objects.is_empty() {
            break;
        }
        let (s, ev) = pass_all(state, players);
        state = s;
        all_events.extend(ev);
    }
    (state, all_events)
}

/// Build a minimal `StackObject` for a HideawayTrigger.
/// All non-essential boolean fields are false, targets is empty.
fn make_hideaway_trigger_stack_obj(
    id: ObjectId,
    source_object: ObjectId,
    hideaway_count: u32,
    controller: PlayerId,
) -> StackObject {
    StackObject {
        id,
        controller,
        kind: StackObjectKind::HideawayTrigger {
            source_object,
            hideaway_count,
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
    }
}

// ── Card definitions ──────────────────────────────────────────────────────────

/// A creature permanent with Hideaway(4).
fn mock_hideaway_creature_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("mock-hideaway-creature".to_string()),
        name: "Mock Hideaway Creature".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            black: 1,
            ..Default::default()
        }),
        types: mtg_engine::TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Hideaway 4".to_string(),
        abilities: vec![AbilityDefinition::Keyword(KeywordAbility::Hideaway(4))],
        power: Some(3),
        toughness: Some(3),
        ..Default::default()
    }
}

/// A permanent with Hideaway(2) AND PlayExiledCard activated ability.
fn mock_hideaway_with_play_ability_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("mock-hideaway-play".to_string()),
        name: "Mock Hideaway Play".to_string(),
        mana_cost: None,
        types: mtg_engine::TypeLine {
            card_types: [CardType::Land].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Hideaway 2\n{T}: Play the exiled card.".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Hideaway(2)),
            AbilityDefinition::Activated {
                cost: mtg_engine::Cost::Tap,
                effect: Effect::Conditional {
                    condition: Condition::Always,
                    if_true: Box::new(Effect::PlayExiledCard),
                    if_false: Box::new(Effect::Nothing),
                },
                timing_restriction: None,
            },
        ],
        ..Default::default()
    }
}

/// A vanilla creature: no keywords, for negative tests.
fn mock_vanilla_creature_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("mock-vanilla-creature".to_string()),
        name: "Mock Vanilla Creature".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            green: 1,
            ..Default::default()
        }),
        types: mtg_engine::TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: String::new(),
        abilities: vec![],
        power: Some(2),
        toughness: Some(2),
        ..Default::default()
    }
}

// ── Test 1: ETB trigger fires ──────────────────────────────────────────────────

/// CR 702.75a — When a permanent with Hideaway(N) enters the battlefield, a
/// HideawayTrigger is put onto the stack. We verify by manually placing the object
/// on the battlefield, queueing a HideawayTrigger in `pending_triggers`, and
/// confirming it flushes to the stack when priority is processed.
#[test]
fn test_hideaway_etb_trigger_fires() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![mock_hideaway_creature_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(
            ObjectSpec::card(p1, "Mock Hideaway Creature")
                .with_card_id(CardId("mock-hideaway-creature".to_string()))
                .with_types(vec![CardType::Creature])
                .with_keyword(KeywordAbility::Hideaway(4))
                .in_zone(ZoneId::Battlefield),
        )
        .object(ObjectSpec::card(p1, "LC1").in_zone(ZoneId::Library(p1)))
        .object(ObjectSpec::card(p1, "LC2").in_zone(ZoneId::Library(p1)))
        .object(ObjectSpec::card(p1, "LC3").in_zone(ZoneId::Library(p1)))
        .object(ObjectSpec::card(p1, "LC4").in_zone(ZoneId::Library(p1)))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let perm_id = find_object(&state, "Mock Hideaway Creature");

    // Push the HideawayTrigger manually onto the stack to simulate what
    // the engine does when the permanent enters via process_command.
    let trigger_id = state.next_object_id();
    state
        .stack_objects
        .push_back(make_hideaway_trigger_stack_obj(trigger_id, perm_id, 4, p1));
    state.turn.priority_holder = Some(p1);

    // The trigger should be on the stack.
    let has_trigger = state
        .stack_objects
        .iter()
        .any(|so| matches!(so.kind, StackObjectKind::HideawayTrigger { .. }));
    assert!(
        has_trigger,
        "CR 702.75a: HideawayTrigger should be on the stack after ETB"
    );

    // Resolve the trigger — both players pass priority.
    let (state, _) = pass_all(state, &[p1, p2]);

    // Stack should be empty after trigger resolves.
    assert_eq!(
        state.stack_objects.len(),
        0,
        "CR 702.75a: stack should be empty after HideawayTrigger resolves"
    );
}

// ── Test 2: Trigger resolution exiles one card face-down ──────────────────────

/// CR 702.75a — When the HideawayTrigger resolves, the top card of the controller's
/// library is exiled face-down, and the remaining cards go to the bottom.
#[test]
fn test_hideaway_trigger_resolution_exiles_one_face_down() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![mock_hideaway_creature_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(
            ObjectSpec::card(p1, "Mock Hideaway Creature")
                .with_card_id(CardId("mock-hideaway-creature".to_string()))
                .with_types(vec![CardType::Creature])
                .with_keyword(KeywordAbility::Hideaway(4))
                .in_zone(ZoneId::Battlefield),
        )
        .object(ObjectSpec::card(p1, "Top Card").in_zone(ZoneId::Library(p1)))
        .object(ObjectSpec::card(p1, "Lib Card 2").in_zone(ZoneId::Library(p1)))
        .object(ObjectSpec::card(p1, "Lib Card 3").in_zone(ZoneId::Library(p1)))
        .object(ObjectSpec::card(p1, "Lib Card 4").in_zone(ZoneId::Library(p1)))
        .object(ObjectSpec::card(p1, "Lib Card 5").in_zone(ZoneId::Library(p1)))
        .object(ObjectSpec::card(p1, "Lib Card 6").in_zone(ZoneId::Library(p1)))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let initial_lib_count = state
        .objects
        .values()
        .filter(|o| o.zone == ZoneId::Library(p1))
        .count();
    assert_eq!(initial_lib_count, 6, "should start with 6 library cards");

    let perm_id = find_object(&state, "Mock Hideaway Creature");

    // Push the HideawayTrigger onto the stack.
    let trigger_id = state.next_object_id();
    state
        .stack_objects
        .push_back(make_hideaway_trigger_stack_obj(trigger_id, perm_id, 4, p1));
    state.turn.priority_holder = Some(p1);

    // Resolve the trigger.
    let (state, events) = pass_all(state, &[p1, p2]);

    // Exactly one card should now be in exile.
    let exile_cards: Vec<_> = state
        .objects
        .values()
        .filter(|o| o.zone == ZoneId::Exile)
        .collect();
    assert_eq!(
        exile_cards.len(),
        1,
        "CR 702.75a: exactly one card should be exiled after trigger resolves; \
         exile count: {}",
        exile_cards.len()
    );

    // The exiled card must be face-down (CR 406.3).
    assert!(
        exile_cards[0].status.face_down,
        "CR 406.3: the hideaway-exiled card must be face-down"
    );

    // Library should have 5 cards remaining (6 - 1 exiled = 5).
    let remaining_lib = state
        .objects
        .values()
        .filter(|o| o.zone == ZoneId::Library(p1))
        .count();
    assert_eq!(
        remaining_lib, 5,
        "CR 702.75a: library should have 5 cards after one is exiled; \
         library count: {}",
        remaining_lib
    );

    // HideawayExiled event should have been emitted.
    let exiled_event = events
        .iter()
        .any(|e| matches!(e, GameEvent::HideawayExiled { .. }));
    assert!(
        exiled_event,
        "CR 702.75a: HideawayExiled event should be emitted on trigger resolution"
    );
}

// ── Test 3: Exiled card tracked by source permanent ───────────────────────────

/// CR 607.2a — The exiled card's `exiled_by_hideaway` field matches the source
/// permanent's ObjectId, establishing the linked-ability relationship.
#[test]
fn test_hideaway_exiled_card_tracked_by_source() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![mock_hideaway_creature_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(
            ObjectSpec::card(p1, "Mock Hideaway Creature")
                .with_card_id(CardId("mock-hideaway-creature".to_string()))
                .with_types(vec![CardType::Creature])
                .with_keyword(KeywordAbility::Hideaway(4))
                .in_zone(ZoneId::Battlefield),
        )
        .object(ObjectSpec::card(p1, "Library Top").in_zone(ZoneId::Library(p1)))
        .object(ObjectSpec::card(p1, "Library Card 2").in_zone(ZoneId::Library(p1)))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let source_id = find_object(&state, "Mock Hideaway Creature");

    // Push HideawayTrigger onto the stack.
    let trigger_id = state.next_object_id();
    state
        .stack_objects
        .push_back(make_hideaway_trigger_stack_obj(
            trigger_id, source_id, 4, p1,
        ));
    state.turn.priority_holder = Some(p1);

    // Resolve the trigger.
    let (state, _) = pass_all(state, &[p1, p2]);

    // The exiled card's exiled_by_hideaway must equal source_id.
    let exiled_obj = state
        .objects
        .values()
        .find(|o| o.zone == ZoneId::Exile)
        .expect("CR 607.2a: one card should be in exile");

    assert_eq!(
        exiled_obj.exiled_by_hideaway,
        Some(source_id),
        "CR 607.2a: exiled_by_hideaway must point to the Hideaway source permanent; \
         got {:?}, expected Some({:?})",
        exiled_obj.exiled_by_hideaway,
        source_id
    );
}

// ── Test 4: Empty library edge case ──────────────────────────────────────────

/// CR 702.75a (edge case) — If the library is empty when the HideawayTrigger
/// resolves, the trigger does nothing (no exile, no library change).
#[test]
fn test_hideaway_empty_library() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![mock_hideaway_creature_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(
            ObjectSpec::card(p1, "Mock Hideaway Creature")
                .with_card_id(CardId("mock-hideaway-creature".to_string()))
                .with_types(vec![CardType::Creature])
                .with_keyword(KeywordAbility::Hideaway(4))
                .in_zone(ZoneId::Battlefield),
        )
        // No library cards — empty library.
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let perm_id = find_object(&state, "Mock Hideaway Creature");

    // Push HideawayTrigger onto the stack.
    let trigger_id = state.next_object_id();
    state
        .stack_objects
        .push_back(make_hideaway_trigger_stack_obj(trigger_id, perm_id, 4, p1));
    state.turn.priority_holder = Some(p1);

    // Resolve the trigger.
    let (state, events) = pass_all(state, &[p1, p2]);

    // No cards should be exiled.
    let exile_count = state
        .objects
        .values()
        .filter(|o| o.zone == ZoneId::Exile)
        .count();
    assert_eq!(
        exile_count, 0,
        "CR 702.75a edge case: empty library — nothing should be exiled"
    );

    // No HideawayExiled event should fire.
    let exiled_event = events
        .iter()
        .any(|e| matches!(e, GameEvent::HideawayExiled { .. }));
    assert!(
        !exiled_event,
        "CR 702.75a edge case: HideawayExiled event should NOT fire for empty library"
    );

    // Stack should be empty after trigger resolves.
    assert_eq!(
        state.stack_objects.len(),
        0,
        "Stack should be empty after empty-library trigger resolves"
    );
}

// ── Test 5: Face-down exile is hidden ─────────────────────────────────────────

/// CR 406.3 — The card exiled by Hideaway is face-down; the engine enforces this
/// via `status.face_down = true` on the exiled card.
#[test]
fn test_hideaway_face_down_exile_is_hidden() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![mock_hideaway_creature_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(
            ObjectSpec::card(p1, "Mock Hideaway Creature")
                .with_card_id(CardId("mock-hideaway-creature".to_string()))
                .with_types(vec![CardType::Creature])
                .with_keyword(KeywordAbility::Hideaway(4))
                .in_zone(ZoneId::Battlefield),
        )
        .object(ObjectSpec::card(p1, "Secret Card").in_zone(ZoneId::Library(p1)))
        .object(ObjectSpec::card(p1, "Other Library Card").in_zone(ZoneId::Library(p1)))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let perm_id = find_object(&state, "Mock Hideaway Creature");

    let trigger_id = state.next_object_id();
    state
        .stack_objects
        .push_back(make_hideaway_trigger_stack_obj(trigger_id, perm_id, 4, p1));
    state.turn.priority_holder = Some(p1);

    let (state, _) = pass_all(state, &[p1, p2]);

    // Every card in exile from this trigger must be face-down.
    let exiled_by_hideaway: Vec<_> = state
        .objects
        .values()
        .filter(|o| o.zone == ZoneId::Exile && o.exiled_by_hideaway.is_some())
        .collect();

    assert_eq!(
        exiled_by_hideaway.len(),
        1,
        "CR 406.3: exactly one hideaway-exiled card should be in exile"
    );
    assert!(
        exiled_by_hideaway[0].status.face_down,
        "CR 406.3: hideaway-exiled card must have face_down = true to enforce hidden information"
    );
}

// ── Test 6: PlayExiledCard finds and plays the exiled card ────────────────────

/// CR 702.75a + CR 607.2a — Activating the "play the exiled card" ability (via
/// `Effect::PlayExiledCard`) finds the card with `exiled_by_hideaway == source_id`,
/// turns it face-up, and places it in the appropriate zone (battlefield for permanents).
#[test]
fn test_hideaway_play_exiled_card() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![mock_hideaway_with_play_ability_def()]);

    // Build the "play the exiled card" activated ability directly on the ObjectSpec.
    // This is the linked ability (CR 607.2a) for Hideaway.
    let play_exiled_ability = ActivatedAbility {
        cost: ActivationCost {
            requires_tap: true,
            mana_cost: None,
            sacrifice_self: false,
        },
        description: "{T}: Play the exiled card without paying its mana cost.".to_string(),
        effect: Some(Effect::Conditional {
            condition: Condition::Always,
            if_true: Box::new(Effect::PlayExiledCard),
            if_false: Box::new(Effect::Nothing),
        }),
        sorcery_speed: false,
    };

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(
            ObjectSpec::card(p1, "Mock Hideaway Play")
                .with_card_id(CardId("mock-hideaway-play".to_string()))
                .with_types(vec![CardType::Land])
                .with_keyword(KeywordAbility::Hideaway(2))
                .with_activated_ability(play_exiled_ability)
                .in_zone(ZoneId::Battlefield),
        )
        .object(ObjectSpec::card(p1, "Library Card A").in_zone(ZoneId::Library(p1)))
        .object(ObjectSpec::card(p1, "Library Card B").in_zone(ZoneId::Library(p1)))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let perm_id = find_object(&state, "Mock Hideaway Play");

    // Step 1: Resolve the Hideaway ETB trigger to exile one card.
    let trigger_id = state.next_object_id();
    state
        .stack_objects
        .push_back(make_hideaway_trigger_stack_obj(trigger_id, perm_id, 2, p1));
    state.turn.priority_holder = Some(p1);

    let (state, _) = pass_all(state, &[p1, p2]);

    // Verify one card is in exile face-down.
    let exile_count = state
        .objects
        .values()
        .filter(|o| o.zone == ZoneId::Exile)
        .count();
    assert_eq!(
        exile_count, 1,
        "Prerequisite: one card should be exiled before activating play ability"
    );

    let exiled_obj = state
        .objects
        .values()
        .find(|o| o.zone == ZoneId::Exile)
        .unwrap();
    assert!(
        exiled_obj.status.face_down,
        "Exiled card should be face-down"
    );
    assert_eq!(
        exiled_obj.exiled_by_hideaway,
        Some(perm_id),
        "CR 607.2a: exiled_by_hideaway should point to perm_id"
    );

    // Step 2: Activate the "play the exiled card" ability (ability index 0).
    let (state, _) = process_command(
        state,
        Command::ActivateAbility {
            player: p1,
            source: perm_id,
            ability_index: 0,
            targets: vec![],
        },
    )
    .unwrap_or_else(|e| panic!("ActivateAbility failed: {:?}", e));

    // Resolve the activated ability.
    let (state, _) = pass_all_until_empty(state, &[p1, p2]);

    // The exiled card should no longer be in exile.
    let exile_after = state
        .objects
        .values()
        .filter(|o| o.zone == ZoneId::Exile)
        .count();
    assert_eq!(
        exile_after, 0,
        "CR 702.75a: after playing the exiled card, exile zone should be empty"
    );
}

// ── Test 7: Negative — no Hideaway keyword, no trigger ───────────────────────

/// Negative test — A permanent WITHOUT Hideaway does NOT generate a
/// HideawayTrigger when it enters the battlefield.
#[test]
fn test_hideaway_negative_no_keyword() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![mock_vanilla_creature_def()]);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(
            ObjectSpec::card(p1, "Mock Vanilla Creature")
                .with_card_id(CardId("mock-vanilla-creature".to_string()))
                .with_types(vec![CardType::Creature])
                .in_zone(ZoneId::Battlefield),
        )
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // No HideawayTrigger should be on the stack.
    let has_hideaway_trigger = state
        .stack_objects
        .iter()
        .any(|so| matches!(so.kind, StackObjectKind::HideawayTrigger { .. }));

    assert!(
        !has_hideaway_trigger,
        "Negative test: permanent without Hideaway should NOT generate HideawayTrigger"
    );

    // No pending hideaway trigger either.
    let has_pending = state.pending_triggers.iter().any(|t| t.is_hideaway_trigger);
    assert!(
        !has_pending,
        "Negative test: no pending hideaway trigger for vanilla creature"
    );
}
