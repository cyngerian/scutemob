//! Delayed triggered ability tests (CR 603.7, CR 610.3).
//!
//! PB-33: Copy/Clone + Exile/Flicker timing.
//!
//! Key rules verified:
//! - CR 603.7 / PB-33: sacrifice_at_end_step flag (Mobilize pattern)
//! - CR 603.7 / PB-33: exile_at_end_step flag (Chandra Flamecaller pattern)
//! - CR 603.7 / PB-33: return_to_hand_at_end_step flag (Locust God pattern)
//! - CR 603.7b: AtNextEndStep delayed trigger fires only once
//! - CR 707.9a: CreateTokenCopy + gains_haste
//! - CR 707.9b: CreateTokenCopy + except_not_legendary
//! - CR 610.3: WhenSourceLeavesBattlefield delayed trigger

use mtg_engine::state::game_object::ObjectId;
use mtg_engine::state::types::SuperType;
use mtg_engine::{
    process_command, CardType, Command, GameEvent, GameState, GameStateBuilder, KeywordAbility,
    PlayerId, Step, ZoneId,
};

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

fn find_in_zone(state: &GameState, name: &str, zone: ZoneId) -> Option<ObjectId> {
    state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name && obj.zone == zone)
        .map(|(id, _)| *id)
}

fn on_battlefield(state: &GameState, name: &str) -> bool {
    find_in_zone(state, name, ZoneId::Battlefield).is_some()
}

#[allow(dead_code)]
fn count_on_battlefield(state: &GameState, name: &str) -> usize {
    state
        .objects
        .values()
        .filter(|obj| obj.characteristics.name == name && obj.zone == ZoneId::Battlefield)
        .count()
}

fn in_exile(state: &GameState, name: &str) -> bool {
    state
        .objects
        .iter()
        .any(|(_, obj)| obj.characteristics.name == name && obj.zone == ZoneId::Exile)
}

fn in_hand(state: &GameState, name: &str, owner: PlayerId) -> bool {
    find_in_zone(state, name, ZoneId::Hand(owner)).is_some()
}

/// Pass priority for all listed players once.
fn pass_all(state: GameState, players: &[PlayerId]) -> (GameState, Vec<GameEvent>) {
    let mut all_events = Vec::new();
    let mut current = state;
    for &pl in players {
        let (s, ev) = process_command(current, Command::PassPriority { player: pl })
            .unwrap_or_else(|e| panic!("PassPriority failed: {:?}", e));
        current = s;
        all_events.extend(ev);
    }
    (current, all_events)
}

/// Drain the stack completely.
fn drain_stack(mut state: GameState, players: &[PlayerId]) -> (GameState, Vec<GameEvent>) {
    let mut all_events = Vec::new();
    let mut limit = 200;
    while !state.stack_objects.is_empty() && limit > 0 {
        let (s, ev) = pass_all(state, players);
        state = s;
        all_events.extend(ev);
        limit -= 1;
    }
    (state, all_events)
}

/// Advance to a specific step by passing priority repeatedly.
fn advance_to_step(
    mut state: GameState,
    target: Step,
    players: &[PlayerId],
) -> (GameState, Vec<GameEvent>) {
    let mut all_events = Vec::new();
    let mut limit = 200;
    while state.turn.step != target && limit > 0 {
        let (s, ev) = pass_all(state, players);
        state = s;
        all_events.extend(ev);
        limit -= 1;
    }
    (state, all_events)
}

/// Build a minimal `GameObject` for testing (creature, default fields).
fn make_test_creature(
    name: &str,
    controller: PlayerId,
) -> mtg_engine::state::game_object::GameObject {
    use mtg_engine::state::game_object::{Characteristics, GameObject, ObjectStatus};
    use mtg_engine::state::types::CardType as CT;

    let mut chars = Characteristics {
        name: name.to_string(),
        power: Some(2),
        toughness: Some(2),
        ..Characteristics::default()
    };
    chars.card_types.insert(CT::Creature);

    GameObject {
        id: ObjectId(0),
        card_id: None,
        characteristics: chars,
        controller,
        owner: controller,
        zone: ZoneId::Battlefield,
        status: ObjectStatus::default(),
        counters: im::OrdMap::new(),
        attachments: im::Vector::new(),
        attached_to: None,
        damage_marked: 0,
        deathtouch_damage: false,
        is_token: false,
        is_emblem: false,
        timestamp: 0,
        has_summoning_sickness: false,
        goaded_by: im::Vector::new(),
        kicker_times_paid: 0,
        cast_alt_cost: None,
        foretold_turn: 0,
        was_unearthed: false,
        myriad_exile_at_eoc: false,
        decayed_sacrifice_at_eoc: false,
        ring_block_sacrifice_at_eoc: false,
        exiled_by_hideaway: None,
        encore_sacrifice_at_end_step: false,
        encore_must_attack: None,
        encore_activated_by: None,
        sacrifice_at_end_step: false,
        exile_at_end_step: false,
        return_to_hand_at_end_step: false,
        is_plotted: false,
        plotted_turn: 0,
        is_prototyped: false,
        was_bargained: false,
        evidence_collected: false,
        phased_out_indirectly: false,
        phased_out_controller: None,
        creatures_devoured: 0,
        champion_exiled_card: None,
        paired_with: None,
        tribute_was_paid: false,
        x_value: 0,
        squad_count: 0,
        offspring_paid: false,
        gift_was_given: false,
        gift_opponent: None,
        encoded_cards: im::Vector::new(),
        haunting_target: None,
        merged_components: im::Vector::new(),
        is_transformed: false,
        last_transform_timestamp: 0,
        was_cast_disturbed: false,
        was_cast: false,
        abilities_activated_this_turn: 0,
        craft_exiled_cards: im::Vector::new(),
        chosen_creature_type: None,
        chosen_color: None,
        face_down_as: None,
        loyalty_ability_activated_this_turn: false,
        class_level: 0,
        designations: mtg_engine::state::game_object::Designations::default(),
        adventure_exiled_by: None,
        meld_component: None,
        entered_turn: None,
    }
}

// ── Test: sacrifice_at_end_step ─────────────────────────────────────────────

/// CR 603.7 / PB-33: Tokens with sacrifice_at_end_step=true are sacrificed at next end step.
///
/// Simulates the Mobilize pattern (Voice of Victory, Zurgo Stormrender).
#[test]
fn test_sacrifice_at_end_step_mobilize_token() {
    let p1 = p(1);
    let p2 = p(2);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let token_spec = mtg_engine::cards::card_definition::TokenSpec {
        name: "Warrior".to_string(),
        card_types: [CardType::Creature].into_iter().collect(),
        subtypes: [mtg_engine::state::types::SubType("Warrior".to_string())]
            .into_iter()
            .collect(),
        colors: [mtg_engine::state::types::Color::Red].into_iter().collect(),
        power: 1,
        toughness: 1,
        count: 1,
        sacrifice_at_end_step: true,
        ..Default::default()
    };
    let token_obj = mtg_engine::effects::make_token(&token_spec, p1);
    state
        .add_object(token_obj, ZoneId::Battlefield)
        .expect("add token");

    assert!(
        on_battlefield(&state, "Warrior"),
        "token on battlefield before end step"
    );

    let (state, _) = advance_to_step(state, Step::End, &[p1, p2]);
    let (state, _) = drain_stack(state, &[p1, p2]);

    assert!(
        !on_battlefield(&state, "Warrior"),
        "CR 603.7b: Mobilize token should be sacrificed at next end step"
    );
}

/// CR 603.7 / PB-33: Tokens with exile_at_end_step=true are exiled at next end step.
///
/// Simulates Chandra Flamecaller's +1 (3/1 Elementals with haste, exile at end step).
#[test]
fn test_exile_at_end_step_token() {
    let p1 = p(1);
    let p2 = p(2);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let token_spec = mtg_engine::cards::card_definition::TokenSpec {
        name: "Elemental".to_string(),
        card_types: [CardType::Creature].into_iter().collect(),
        subtypes: [mtg_engine::state::types::SubType("Elemental".to_string())]
            .into_iter()
            .collect(),
        colors: [mtg_engine::state::types::Color::Red].into_iter().collect(),
        power: 3,
        toughness: 1,
        count: 1,
        keywords: [KeywordAbility::Haste].into_iter().collect(),
        exile_at_end_step: true,
        ..Default::default()
    };
    let token_obj = mtg_engine::effects::make_token(&token_spec, p1);
    state
        .add_object(token_obj, ZoneId::Battlefield)
        .expect("add token");

    assert!(
        on_battlefield(&state, "Elemental"),
        "token on battlefield before end step"
    );

    let (state, _) = advance_to_step(state, Step::End, &[p1, p2]);
    let (state, _) = drain_stack(state, &[p1, p2]);

    assert!(
        !on_battlefield(&state, "Elemental"),
        "CR 603.7b: Elemental token should be exiled at next end step"
    );
    // Note: CR 704.5d causes tokens in exile to cease to exist (SBA removes them).
    // So the token won't persist in exile — it simply no longer exists.
    assert!(
        state
            .objects
            .values()
            .find(|obj| obj.characteristics.name == "Elemental")
            .is_none(),
        "Elemental token should no longer exist (exiled then SBA removed per CR 704.5d)"
    );
}

/// CR 603.7 / PB-33: return_to_hand_at_end_step on graveyard object.
///
/// Simulates The Locust God's death trigger (return to hand at next end step).
#[test]
fn test_return_to_hand_at_end_step() {
    let p1 = p(1);
    let p2 = p(2);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Add a creature in the graveyard with return_to_hand_at_end_step=true.
    let mut god = make_test_creature("Locust God", p1);
    god.return_to_hand_at_end_step = true;
    let _god_id = state
        .add_object(god, ZoneId::Graveyard(p1))
        .expect("add to graveyard");

    assert!(
        !in_hand(&state, "Locust God", p1),
        "god starts in graveyard"
    );

    let (state, _) = advance_to_step(state, Step::End, &[p1, p2]);
    let (state, _) = drain_stack(state, &[p1, p2]);

    assert!(
        in_hand(&state, "Locust God", p1),
        "CR 603.7: Locust God should return to hand at next end step"
    );
}

/// CR 603.7b: AtNextEndStep delayed trigger fires exactly once.
///
/// After firing, the trigger is cleaned up and does not re-fire on later end steps.
#[test]
fn test_delayed_trigger_fires_only_once() {
    use mtg_engine::state::stubs::{DelayedTrigger, DelayedTriggerAction, DelayedTriggerTiming};

    let p1 = p(1);
    let p2 = p(2);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Add a creature in exile that the delayed trigger will return.
    let bear = make_test_creature("Bear", p1);
    let bear_id = state
        .add_object(bear, ZoneId::Exile)
        .expect("add bear to exile");

    // Register a delayed trigger to return at next end step.
    state.delayed_triggers.push_back(DelayedTrigger {
        source: ObjectId(999), // dummy source (unused for AtNextEndStep)
        controller: p1,
        target_object: bear_id,
        action: DelayedTriggerAction::ReturnFromExileToBattlefield { tapped: false },
        timing: DelayedTriggerTiming::AtNextEndStep,
        fired: false,
    });

    assert_eq!(
        state.delayed_triggers.len(),
        1,
        "one pending delayed trigger"
    );
    assert!(in_exile(&state, "Bear"), "bear starts in exile");

    // Advance to end step — trigger fires.
    let (state, _) = advance_to_step(state, Step::End, &[p1, p2]);
    let (state, _) = drain_stack(state, &[p1, p2]);

    assert!(
        on_battlefield(&state, "Bear"),
        "CR 603.7b: bear returned to battlefield"
    );
    assert_eq!(
        state.delayed_triggers.len(),
        0,
        "CR 603.7b: fired trigger should be cleaned up"
    );
}

/// CR 707.9a: CreateTokenCopy with gains_haste creates a token with Haste.
#[test]
fn test_create_token_copy_with_haste() {
    use mtg_engine::cards::card_definition::{Effect, EffectTarget};
    use mtg_engine::effects::{execute_effect, EffectContext};
    use mtg_engine::state::targeting::{SpellTarget, Target};

    let p1 = p(1);
    let p2 = p(2);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let source = make_test_creature("Source Creature", p1);
    let source_id = state
        .add_object(source, ZoneId::Battlefield)
        .expect("add source creature");

    let effect = Effect::CreateTokenCopy {
        source: EffectTarget::DeclaredTarget { index: 0 },
        enters_tapped_and_attacking: false,
        except_not_legendary: false,
        gains_haste: true,
        delayed_action: None,
    };

    let target = SpellTarget {
        target: Target::Object(source_id),
        zone_at_cast: Some(ZoneId::Battlefield),
    };
    let mut ctx = EffectContext::new(p1, source_id, vec![target]);
    let _events = execute_effect(&mut state, &effect, &mut ctx);

    let token = state
        .objects
        .values()
        .find(|obj| {
            obj.characteristics.name == "Source Creature"
                && obj.is_token
                && obj.zone == ZoneId::Battlefield
        })
        .expect("token should exist");

    let chars = mtg_engine::rules::layers::calculate_characteristics(&state, token.id)
        .unwrap_or_else(|| token.characteristics.clone());

    assert!(
        chars.keywords.contains(&KeywordAbility::Haste),
        "CR 707.9a: token copy should have Haste"
    );
}

/// CR 707.9b: CreateTokenCopy with except_not_legendary removes Legendary supertype.
#[test]
fn test_create_token_copy_not_legendary() {
    use mtg_engine::cards::card_definition::{Effect, EffectTarget};
    use mtg_engine::effects::{execute_effect, EffectContext};
    use mtg_engine::state::targeting::{SpellTarget, Target};

    let p1 = p(1);
    let p2 = p(2);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let mut source = make_test_creature("Legendary Dragon", p1);
    source
        .characteristics
        .supertypes
        .insert(SuperType::Legendary);
    source.characteristics.power = Some(5);
    source.characteristics.toughness = Some(5);
    let source_id = state
        .add_object(source, ZoneId::Battlefield)
        .expect("add legendary creature");

    let effect = Effect::CreateTokenCopy {
        source: EffectTarget::DeclaredTarget { index: 0 },
        enters_tapped_and_attacking: false,
        except_not_legendary: true,
        gains_haste: false,
        delayed_action: None,
    };

    let target = SpellTarget {
        target: Target::Object(source_id),
        zone_at_cast: Some(ZoneId::Battlefield),
    };
    let mut ctx = EffectContext::new(p1, source_id, vec![target]);
    let _events = execute_effect(&mut state, &effect, &mut ctx);

    let token = state
        .objects
        .values()
        .find(|obj| {
            obj.characteristics.name == "Legendary Dragon"
                && obj.is_token
                && obj.zone == ZoneId::Battlefield
        })
        .expect("token should exist");

    let chars = mtg_engine::rules::layers::calculate_characteristics(&state, token.id)
        .unwrap_or_else(|| token.characteristics.clone());

    assert!(
        !chars.supertypes.contains(&SuperType::Legendary),
        "CR 707.9b: token copy should NOT be Legendary"
    );

    // Source should still be Legendary.
    let source = state.objects.get(&source_id).expect("source still exists");
    let source_chars = mtg_engine::rules::layers::calculate_characteristics(&state, source_id)
        .unwrap_or_else(|| source.characteristics.clone());
    assert!(
        source_chars.supertypes.contains(&SuperType::Legendary),
        "CR 707.9b: source should still be Legendary"
    );
}

/// CR 610.3: WhenSourceLeavesBattlefield delayed trigger returns exiled creature when source LTBs.
///
/// Tests the Brutal Cathar "exile until this leaves" pattern.
#[test]
fn test_exile_until_source_leaves() {
    use mtg_engine::state::stubs::{DelayedTrigger, DelayedTriggerAction, DelayedTriggerTiming};

    let p1 = p(1);
    let p2 = p(2);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // p1 has Cathar (source) on battlefield.
    let cathar = make_test_creature("Cathar", p1);
    let cathar_id = state
        .add_object(cathar, ZoneId::Battlefield)
        .expect("add cathar");

    // p2 has a creature exiled.
    let mut prisoner = make_test_creature("Prisoner", p2);
    prisoner.owner = p2;
    prisoner.controller = p2;
    let prisoner_id = state
        .add_object(prisoner, ZoneId::Exile)
        .expect("add prisoner to exile");

    // Register a WhenSourceLeavesBattlefield delayed trigger.
    state.delayed_triggers.push_back(DelayedTrigger {
        source: cathar_id,
        controller: p1,
        target_object: prisoner_id,
        action: DelayedTriggerAction::ReturnFromExileToBattlefield { tapped: false },
        timing: DelayedTriggerTiming::WhenSourceLeavesBattlefield,
        fired: false,
    });

    assert!(in_exile(&state, "Prisoner"), "prisoner starts in exile");
    assert!(on_battlefield(&state, "Cathar"), "cathar on battlefield");

    // Destroy the Cathar (source leaves battlefield).
    let (grave_id, _) = state
        .move_object_to_zone(cathar_id, ZoneId::Graveyard(p1))
        .expect("destroy cathar");

    // Simulate check_triggers for CreatureDied event.
    let events = vec![mtg_engine::GameEvent::CreatureDied {
        object_id: cathar_id,
        new_grave_id: grave_id,
        controller: p1,
        pre_death_counters: im::OrdMap::new(),
    }];
    let triggers = mtg_engine::rules::abilities::check_triggers(&state, &events);
    for t in triggers {
        state.pending_triggers.push_back(t);
    }

    // Flush and drain stack.
    let _evts = mtg_engine::rules::abilities::flush_pending_triggers(&mut state);
    let (state, _) = drain_stack(state, &[p1, p2]);

    assert!(
        !in_exile(&state, "Prisoner"),
        "CR 610.3: Prisoner should return when Cathar leaves battlefield"
    );
    assert!(
        on_battlefield(&state, "Prisoner"),
        "CR 610.3: Prisoner should be on battlefield under owner's control"
    );
}
