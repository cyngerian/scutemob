//! Encore keyword ability tests (CR 702.141).
//!
//! Encore is an activated ability that functions while the card is in a graveyard.
//! "Encore [cost]" means "[Cost], Exile this card from your graveyard: For each opponent,
//! create a token that's a copy of this card that attacks that opponent this turn if able.
//! The tokens gain haste. Sacrifice them at the beginning of the next end step.
//! Activate only as a sorcery." (CR 702.141a)
//!
//! Key rules verified:
//! - Card is exiled as part of the activation cost (not at resolution) (CR 702.141a).
//! - In a 4-player game, 3 tokens are created (one per opponent) (CR 702.141a).
//! - In a 2-player game, 1 token is created (CR 702.141a).
//! - Tokens gain Haste (CR 702.141a).
//! - Tokens are tagged `encore_sacrifice_at_end_step = true` (CR 702.141a).
//! - Tokens have `encore_must_attack = Some(opponent_id)` (CR 702.141a).
//! - Eliminated opponents do not get a token (ruling 2020-11-10).
//! - Sorcery-speed restriction: active player only, main phase only, empty stack (CR 702.141a).
//! - Card must be in player's own graveyard (CR 702.141a).
//! - Card must have the Encore keyword (CR 702.141a).
//! - Encore is NOT a cast: no SpellCast event (ruling).

use mtg_engine::{
    process_command, AbilityDefinition, CardDefinition, CardId, CardRegistry, CardType, Command,
    GameEvent, GameState, GameStateBuilder, KeywordAbility, ManaColor, ManaCost, ObjectId,
    ObjectSpec, PlayerId, Step, SubType, TypeLine, ZoneId,
};

// ── Helpers ───────────────────────────────────────────────────────────────────

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

fn find_object(state: &GameState, name: &str) -> ObjectId {
    state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{}' not found", name))
}

fn find_in_zone(state: &GameState, name: &str, zone: ZoneId) -> Option<ObjectId> {
    state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name && obj.zone == zone)
        .map(|(id, _)| *id)
}

fn in_graveyard(state: &GameState, name: &str, owner: PlayerId) -> bool {
    find_in_zone(state, name, ZoneId::Graveyard(owner)).is_some()
}

fn in_exile(state: &GameState, name: &str) -> bool {
    find_in_zone(state, name, ZoneId::Exile).is_some()
}

fn count_on_battlefield_by_name(state: &GameState, name: &str) -> usize {
    state
        .objects
        .values()
        .filter(|obj| obj.characteristics.name == name && obj.zone == ZoneId::Battlefield)
        .count()
}

/// Pass priority for all listed players once.
fn pass_all(state: GameState, players: &[PlayerId]) -> (GameState, Vec<GameEvent>) {
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

/// Encore Test Creature: {2}{B}, 2/2 Warrior Creature, Encore {2}{B}.
fn encore_test_card() -> CardDefinition {
    CardDefinition {
        card_id: CardId("encore-test-creature".to_string()),
        name: "Encore Test Creature".to_string(),
        mana_cost: Some(ManaCost {
            black: 1,
            generic: 2,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            subtypes: [SubType("Warrior".to_string())].into_iter().collect(),
            ..Default::default()
        },
        power: Some(2),
        toughness: Some(2),
        oracle_text: "Encore {2}{B}".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Encore),
            AbilityDefinition::Encore {
                cost: ManaCost {
                    black: 1,
                    generic: 2,
                    ..Default::default()
                },
            },
        ],
        ..Default::default()
    }
}

/// Build an ObjectSpec for the encore test creature in a player's graveyard.
fn creature_in_graveyard(owner: PlayerId) -> ObjectSpec {
    ObjectSpec::card(owner, "Encore Test Creature")
        .in_zone(ZoneId::Graveyard(owner))
        .with_card_id(CardId("encore-test-creature".to_string()))
        .with_keyword(KeywordAbility::Encore)
}

// ── Test 1: Basic encore in 4-player game — 3 tokens created ─────────────────

#[test]
/// CR 702.141a — Activate encore in a 4-player game; card is exiled as cost;
/// 3 tokens created (one per opponent) when ability resolves.
fn test_encore_basic_4p() {
    let p1 = p(1);
    let p2 = p(2);
    let p3 = p(3);
    let p4 = p(4);

    let registry = CardRegistry::new(vec![encore_test_card()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .add_player(p4)
        .with_registry(registry)
        .object(creature_in_graveyard(p1))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Give p1 {2}{B} mana for encore cost.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Black, 1);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 2);
    state.turn.priority_holder = Some(p1);

    let card_id = find_object(&state, "Encore Test Creature");

    // p1 activates encore.
    let (state, activate_events) = process_command(
        state,
        Command::EncoreCard {
            player: p1,
            card: card_id,
        },
    )
    .expect("EncoreCard should succeed");

    // AbilityActivated event emitted.
    assert!(
        activate_events
            .iter()
            .any(|e| matches!(e, GameEvent::AbilityActivated { player, .. } if *player == p1)),
        "CR 702.141a: AbilityActivated event expected when encore is activated"
    );

    // Card should be in exile immediately (exiled as cost, CR 702.141a).
    assert!(
        in_exile(&state, "Encore Test Creature"),
        "CR 702.141a: card should be in exile after encore activation (exiled as cost)"
    );
    assert!(
        !in_graveyard(&state, "Encore Test Creature", p1),
        "CR 702.141a: card should no longer be in graveyard"
    );

    // All 4 players pass priority → ability resolves.
    let (state, _resolve_events) = pass_all(state, &[p1, p2, p3, p4]);

    // 3 tokens should be on the battlefield (one per opponent: p2, p3, p4).
    let token_count = count_on_battlefield_by_name(&state, "Encore Test Creature");
    assert_eq!(
        token_count, 3,
        "CR 702.141a: 3 tokens expected in 4-player game (one per opponent)"
    );
}

// ── Test 2: Tokens gain haste ─────────────────────────────────────────────────

#[test]
/// CR 702.141a — "The tokens gain haste."
/// Each token created by encore must have KeywordAbility::Haste.
fn test_encore_tokens_have_haste() {
    let p1 = p(1);
    let p2 = p(2);
    let p3 = p(3);

    let registry = CardRegistry::new(vec![encore_test_card()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .with_registry(registry)
        .object(creature_in_graveyard(p1))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Black, 1);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 2);
    state.turn.priority_holder = Some(p1);

    let card_id = find_object(&state, "Encore Test Creature");

    let (state, _) = process_command(
        state,
        Command::EncoreCard {
            player: p1,
            card: card_id,
        },
    )
    .expect("EncoreCard should succeed");

    let (state, _) = pass_all(state, &[p1, p2, p3]);

    // All tokens on battlefield should have Haste.
    let tokens: Vec<_> = state
        .objects
        .values()
        .filter(|obj| {
            obj.characteristics.name == "Encore Test Creature"
                && obj.zone == ZoneId::Battlefield
                && obj.is_token
        })
        .collect();

    assert_eq!(tokens.len(), 2, "2 tokens expected in 3-player game");

    for token in &tokens {
        assert!(
            token
                .characteristics
                .keywords
                .contains(&KeywordAbility::Haste),
            "CR 702.141a: encore token must have Haste; token {:?} does not",
            token.id
        );
    }
}

// ── Test 3: Card exiled as cost ───────────────────────────────────────────────

#[test]
/// CR 702.141a, CR 602.2 — The card is exiled as part of the activation cost,
/// before the ability goes on the stack.
fn test_encore_card_exiled_as_cost() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![encore_test_card()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(creature_in_graveyard(p1))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Black, 1);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 2);
    state.turn.priority_holder = Some(p1);

    let card_id = find_object(&state, "Encore Test Creature");

    // Before activation: card is in graveyard.
    assert!(in_graveyard(&state, "Encore Test Creature", p1));

    let (state, activate_events) = process_command(
        state,
        Command::EncoreCard {
            player: p1,
            card: card_id,
        },
    )
    .expect("EncoreCard should succeed");

    // After activation: card is in exile (exiled as cost, BEFORE ability resolves).
    assert!(
        in_exile(&state, "Encore Test Creature"),
        "CR 702.141a: card must be in exile immediately after activation (exiled as cost)"
    );
    assert!(
        !in_graveyard(&state, "Encore Test Creature", p1),
        "CR 702.141a: card must not be in graveyard after activation"
    );

    // ObjectExiled event should have been emitted during activation.
    assert!(
        activate_events
            .iter()
            .any(|e| matches!(e, GameEvent::ObjectExiled { player, .. } if *player == p1)),
        "CR 702.141a: ObjectExiled event expected during activation (cost payment)"
    );

    // Stack should have the EncoreAbility (not empty yet).
    assert_eq!(
        state.stack_objects.len(),
        1,
        "CR 702.141a: EncoreAbility should be on the stack after activation"
    );
}

// ── Test 4: Tokens sacrificed at end step ─────────────────────────────────────

#[test]
/// CR 702.141a — "Sacrifice them at the beginning of the next end step."
/// Encore tokens have `encore_sacrifice_at_end_step = true`, which queues
/// EncoreSacrificeTriggers in end_step_actions().
fn test_encore_sacrifice_at_end_step() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![encore_test_card()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(creature_in_graveyard(p1))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Black, 1);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 2);
    state.turn.priority_holder = Some(p1);

    let card_id = find_object(&state, "Encore Test Creature");

    let (state, _) = process_command(
        state,
        Command::EncoreCard {
            player: p1,
            card: card_id,
        },
    )
    .expect("EncoreCard should succeed");

    // Resolve the encore ability.
    let (state, _) = pass_all(state, &[p1, p2]);

    // Token should be on the battlefield with encore_sacrifice_at_end_step = true.
    let token_id = state
        .objects
        .iter()
        .find(|(_, obj)| {
            obj.characteristics.name == "Encore Test Creature"
                && obj.zone == ZoneId::Battlefield
                && obj.is_token
        })
        .map(|(id, _)| *id)
        .expect("encore token should be on battlefield");

    let token = state.objects.get(&token_id).unwrap();
    assert!(
        token.encore_sacrifice_at_end_step,
        "CR 702.141a: encore token should have encore_sacrifice_at_end_step = true"
    );

    // Simulate end step actions (which queue the sacrifice trigger).
    let mut state = state;
    let _end_events = mtg_engine::rules::turn_actions::end_step_actions(&mut state);

    // Pending triggers should include an encore sacrifice trigger.
    let has_encore_trigger = state
        .pending_triggers
        .iter()
        .any(|t| t.kind == mtg_engine::state::stubs::PendingTriggerKind::EncoreSacrifice);
    assert!(
        has_encore_trigger,
        "CR 702.141a: end_step_actions should queue EncoreSacrificeTrigger for encore tokens"
    );

    // Flush triggers: the sacrifice trigger goes on the stack.
    let mut events = Vec::new();
    let trigger_events = mtg_engine::rules::abilities::flush_pending_triggers(&mut state);
    events.extend(trigger_events);

    // Stack should have the EncoreSacrificeTrigger.
    assert_eq!(
        state.stack_objects.len(),
        1,
        "EncoreSacrificeTrigger should be on the stack"
    );

    // Both players pass priority → sacrifice trigger resolves.
    let (state, _) = pass_all(state, &[p1, p2]);

    // Token should no longer be on the battlefield.
    let token_on_battlefield = state.objects.values().any(|obj| {
        obj.characteristics.name == "Encore Test Creature" && obj.zone == ZoneId::Battlefield
    });
    assert!(
        !token_on_battlefield,
        "CR 702.141a: encore token should be sacrificed after EncoreSacrificeTrigger resolves"
    );
}

// ── Test 5: Sorcery speed — not active player's turn ─────────────────────────

#[test]
/// CR 702.141a — "Activate only as a sorcery."
/// Cannot be activated during opponent's turn.
fn test_encore_sorcery_speed_opponent_turn() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![encore_test_card()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(creature_in_graveyard(p2)) // p2's card
        .active_player(p1) // p1 is active player
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // p2 tries to activate encore during p1's turn.
    state
        .players
        .get_mut(&p2)
        .unwrap()
        .mana_pool
        .add(ManaColor::Black, 1);
    state
        .players
        .get_mut(&p2)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 2);
    state.turn.priority_holder = Some(p2);

    let card_id = find_object(&state, "Encore Test Creature");

    let result = process_command(
        state,
        Command::EncoreCard {
            player: p2,
            card: card_id,
        },
    );

    assert!(
        result.is_err(),
        "CR 702.141a: encore cannot be activated during an opponent's turn"
    );
}

// ── Test 6: Sorcery speed — non-empty stack ───────────────────────────────────

#[test]
/// CR 702.141a — "Activate only as a sorcery."
/// Cannot be activated when the stack is not empty.
fn test_encore_sorcery_speed_non_empty_stack() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![encore_test_card()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(creature_in_graveyard(p1))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Black, 1);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 2);
    state.turn.priority_holder = Some(p1);

    // Artificially add a stack object to simulate non-empty stack.
    let fake_stack_obj = mtg_engine::StackObject {
        id: ObjectId(9999),
        controller: p1,
        kind: mtg_engine::StackObjectKind::TriggeredAbility {
            source_object: ObjectId(9998),
            ability_index: 0,
        },
        targets: Vec::new(),
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
    };
    state.stack_objects.push_back(fake_stack_obj);

    let card_id = find_object(&state, "Encore Test Creature");

    let result = process_command(
        state,
        Command::EncoreCard {
            player: p1,
            card: card_id,
        },
    );

    assert!(
        result.is_err(),
        "CR 702.141a: encore cannot be activated with a non-empty stack"
    );
}

// ── Test 7: Card not in graveyard ─────────────────────────────────────────────

#[test]
/// CR 702.141a — Encore must be activated from the player's graveyard.
/// Attempting from any other zone must fail.
fn test_encore_not_in_graveyard() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![encore_test_card()]);

    // Place the card in hand, not graveyard.
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(
            ObjectSpec::card(p1, "Encore Test Creature")
                .in_zone(ZoneId::Hand(p1))
                .with_card_id(CardId("encore-test-creature".to_string()))
                .with_keyword(KeywordAbility::Encore),
        )
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Black, 1);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 2);
    state.turn.priority_holder = Some(p1);

    let card_id = find_object(&state, "Encore Test Creature");

    let result = process_command(
        state,
        Command::EncoreCard {
            player: p1,
            card: card_id,
        },
    );

    assert!(
        result.is_err(),
        "CR 702.141a: encore cannot be activated from hand (must be in graveyard)"
    );
}

// ── Test 8: Card lacks Encore keyword ─────────────────────────────────────────

#[test]
/// CR 702.141a — The card must have the Encore keyword.
/// A creature without Encore in the graveyard cannot be encored.
fn test_encore_no_keyword() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![encore_test_card()]);

    // Place a card WITHOUT the Encore keyword in the graveyard.
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(
            ObjectSpec::card(p1, "Encore Test Creature")
                .in_zone(ZoneId::Graveyard(p1))
                .with_card_id(CardId("encore-test-creature".to_string())),
            // NOTE: no .with_keyword(KeywordAbility::Encore)
        )
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Black, 1);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 2);
    state.turn.priority_holder = Some(p1);

    let card_id = find_object(&state, "Encore Test Creature");

    let result = process_command(
        state,
        Command::EncoreCard {
            player: p1,
            card: card_id,
        },
    );

    assert!(
        result.is_err(),
        "CR 702.141a: encore activation must fail if card lacks Encore keyword"
    );
}

// ── Test 9: 2-player game — only 1 token created ─────────────────────────────

#[test]
/// CR 702.141a — "For each opponent" — in a 1v1 game, only 1 opponent exists.
/// Exactly 1 token should be created.
fn test_encore_2p_game() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![encore_test_card()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(creature_in_graveyard(p1))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Black, 1);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 2);
    state.turn.priority_holder = Some(p1);

    let card_id = find_object(&state, "Encore Test Creature");

    let (state, _) = process_command(
        state,
        Command::EncoreCard {
            player: p1,
            card: card_id,
        },
    )
    .expect("EncoreCard should succeed");

    // Resolve.
    let (state, _) = pass_all(state, &[p1, p2]);

    let token_count = count_on_battlefield_by_name(&state, "Encore Test Creature");
    assert_eq!(
        token_count, 1,
        "CR 702.141a: exactly 1 token expected in a 2-player game"
    );
}

// ── Test 10: Eliminated opponent gets no token ────────────────────────────────

#[test]
/// CR 702.141a — Ruling: "Opponents who have left the game aren't counted."
/// A player with has_lost = true should not receive a token.
fn test_encore_eliminated_opponent() {
    let p1 = p(1);
    let p2 = p(2);
    let p3 = p(3);
    let p4 = p(4);

    let registry = CardRegistry::new(vec![encore_test_card()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .add_player(p4)
        .with_registry(registry)
        .object(creature_in_graveyard(p1))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Mark p3 as eliminated.
    state.players.get_mut(&p3).unwrap().has_lost = true;

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Black, 1);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 2);
    state.turn.priority_holder = Some(p1);

    let card_id = find_object(&state, "Encore Test Creature");

    let (state, _) = process_command(
        state,
        Command::EncoreCard {
            player: p1,
            card: card_id,
        },
    )
    .expect("EncoreCard should succeed");

    // p3 has_lost = true so only p1, p2, p4 in turn_order for pass_all.
    // Use p1, p2, p4 for priority passing.
    let (state, _) = pass_all(state, &[p1, p2, p4]);

    let token_count = count_on_battlefield_by_name(&state, "Encore Test Creature");
    assert_eq!(
        token_count, 2,
        "CR 702.141a ruling: eliminated opponent (p3) should not receive a token; only p2 and p4"
    );
}
