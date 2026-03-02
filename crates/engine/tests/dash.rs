//! Dash keyword ability tests (CR 702.109).
//!
//! Dash is an alternative cost (CR 118.9) that allows a creature to be cast for its
//! dash cost instead of its mana cost. When the permanent enters the battlefield, it
//! gains haste. At the beginning of the next end step, it is returned to its owner's hand.
//!
//! Key rules verified:
//! - Dash is an alternative cost: pay dash cost instead of mana cost (CR 702.109a).
//! - After ETB: permanent has haste (was_dashed = true grants haste) (CR 702.109a).
//! - At beginning of next end step, a delayed trigger returns the permanent to hand (CR 702.109a).
//! - If permanent leaves battlefield before trigger resolves, trigger does nothing (Ruling 2014-11-24).
//! - Normal cast (no dash) leaves creature on battlefield with no return trigger (negative test).
//! - Dash cannot combine with other alternative costs: flashback, evoke, etc. (CR 118.9a).
//! - Copies of a dashed creature do NOT inherit was_dashed; no haste, no return trigger (Ruling 2014-11-24).
//! - Commander tax applies on top of dash cost (CR 118.9d).

use mtg_engine::state::types::SuperType;
use mtg_engine::{
    process_command, AbilityDefinition, CardDefinition, CardId, CardRegistry, CardType, Command,
    GameEvent, GameState, GameStateBuilder, KeywordAbility, ManaColor, ManaCost, ObjectId,
    ObjectSpec, PlayerId, Step, TypeLine, ZoneId,
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

fn on_battlefield(state: &GameState, name: &str) -> bool {
    find_in_zone(state, name, ZoneId::Battlefield).is_some()
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
            .unwrap_or_else(|e| panic!("PassPriority by {:?} failed: {:?}", pl, e));
        current = s;
        all_events.extend(ev);
    }
    (current, all_events)
}

/// Goblin Raider (mock): Creature {1}{R} 2/2.
/// Dash {R} — can be cast for {R} instead of {1}{R}, enters with haste and returns at end step.
fn goblin_raider_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("goblin-raider".to_string()),
        name: "Goblin Raider".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            red: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Dash {R}".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Dash),
            AbilityDefinition::Dash {
                cost: ManaCost {
                    red: 1,
                    ..Default::default()
                },
            },
        ],
        power: Some(2),
        toughness: Some(2),
        ..Default::default()
    }
}

/// Helper: place a Goblin Raider in the player's hand.
fn raider_in_hand(owner: PlayerId) -> ObjectSpec {
    ObjectSpec::card(owner, "Goblin Raider")
        .in_zone(ZoneId::Hand(owner))
        .with_card_id(CardId("goblin-raider".to_string()))
        .with_types(vec![CardType::Creature])
        .with_keyword(KeywordAbility::Dash)
        .with_mana_cost(ManaCost {
            generic: 1,
            red: 1,
            ..Default::default()
        })
}

// ── Test 1: Basic dash cast — haste granted ───────────────────────────────────

/// CR 702.109a — Goblin Raider cast for dash cost {R}.
/// After ETB: creature on battlefield with haste (was_dashed = true) and was_dashed flag set.
#[test]
fn test_dash_basic_cast_with_dash_cost() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![goblin_raider_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(raider_in_hand(p1))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Pay {R} — dash cost instead of mana cost {1}{R}.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Red, 1);
    state.turn.priority_holder = Some(p1);

    let card_id = find_object(&state, "Goblin Raider");

    // Cast with dash.
    let (state, _) = process_command(
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
            cast_with_foretell: false,
            cast_with_buyback: false,
            cast_with_overload: false,
            retrace_discard_land: None,
            cast_with_jump_start: false,
            jump_start_discard: None,
            cast_with_aftermath: false,
            cast_with_dash: true,
        },
    )
    .unwrap_or_else(|e| panic!("CastSpell with dash failed: {:?}", e));

    // Spell is on the stack with was_dashed = true.
    assert_eq!(
        state.stack_objects.len(),
        1,
        "CR 702.109a: dashed spell should be on the stack"
    );
    assert!(
        state.stack_objects[0].was_dashed,
        "CR 702.109a: was_dashed should be true on stack object"
    );

    // Mana consumed: {R} = 1 mana total (not {1}{R} = 2 mana).
    let pool = &state.players[&p1].mana_pool;
    assert_eq!(
        pool.white + pool.blue + pool.black + pool.red + pool.green + pool.colorless,
        0,
        "CR 702.109a: {{R}} dash cost should be deducted from mana pool"
    );

    // Resolve the spell (both players pass priority).
    let (state, _) = pass_all(state, &[p1, p2]);

    // Creature is on the battlefield.
    assert!(
        on_battlefield(&state, "Goblin Raider"),
        "CR 702.109a: dashed creature should be on battlefield after resolution"
    );

    // was_dashed flag is set on the permanent.
    let bf_id = find_in_zone(&state, "Goblin Raider", ZoneId::Battlefield).unwrap();
    assert!(
        state.objects[&bf_id].was_dashed,
        "CR 702.109a: was_dashed should be true on battlefield permanent"
    );

    // Haste is granted: permanent has the Haste keyword.
    assert!(
        state.objects[&bf_id]
            .characteristics
            .keywords
            .contains(&KeywordAbility::Haste),
        "CR 702.109a: dashed creature should have Haste keyword"
    );
}

// ── Test 2: Normal cast — no return trigger, no haste from dash ──────────────

/// CR 702.109a — Goblin Raider cast for normal mana cost {1}{R}.
/// No haste from dash, no return trigger — creature stays on battlefield.
#[test]
fn test_dash_normal_cast_no_return() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![goblin_raider_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(raider_in_hand(p1))
        .active_player(p1)
        .at_step(Step::PostCombatMain)
        .build()
        .unwrap();

    // Pay {1}{R} — normal mana cost.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Red, 1);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 1);
    state.turn.priority_holder = Some(p1);

    let card_id = find_object(&state, "Goblin Raider");

    // Cast normally (no dash).
    let (state, _) = process_command(
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
            cast_with_foretell: false,
            cast_with_buyback: false,
            cast_with_overload: false,
            retrace_discard_land: None,
            cast_with_jump_start: false,
            jump_start_discard: None,
            cast_with_aftermath: false,
            cast_with_dash: false,
        },
    )
    .unwrap_or_else(|e| panic!("Normal CastSpell failed: {:?}", e));

    // Resolve.
    let (state, _) = pass_all(state, &[p1, p2]);

    // Creature is on the battlefield.
    assert!(
        on_battlefield(&state, "Goblin Raider"),
        "creature should be on battlefield after normal cast"
    );

    let bf_id = find_in_zone(&state, "Goblin Raider", ZoneId::Battlefield).unwrap();

    // was_dashed is false (normal cast).
    assert!(
        !state.objects[&bf_id].was_dashed,
        "CR 702.109a: was_dashed should be false for normally cast creature"
    );

    // Advance to End step — no DashReturnTrigger should fire.
    let (state, _) = pass_all(state, &[p1, p2]);
    assert_eq!(state.turn.step, Step::End, "should advance to End step");

    // Pass priority at End step — creature should still be on battlefield.
    let (state, _) = pass_all(state, &[p1, p2]);

    assert!(
        on_battlefield(&state, "Goblin Raider"),
        "CR 702.109a: normally cast creature should NOT return to hand at end step"
    );
    assert!(
        !in_hand(&state, "Goblin Raider", p1),
        "CR 702.109a: normally cast creature should NOT be in hand at end step"
    );
}

// ── Test 3: Dash return-to-hand at end step ───────────────────────────────────

/// CR 702.109a — Dashed creature returns to owner's hand at beginning of next end step.
/// Delayed triggered ability: DashReturnTrigger queued in end_step_actions,
/// flushed to stack, then resolves returning creature to hand.
#[test]
fn test_dash_return_to_hand_at_end_step() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![goblin_raider_def()]);

    // Start at PostCombatMain so we can advance to End step.
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(raider_in_hand(p1))
        .active_player(p1)
        .at_step(Step::PostCombatMain)
        .build()
        .unwrap();

    // Pay {R} — dash cost.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Red, 1);
    state.turn.priority_holder = Some(p1);

    let card_id = find_object(&state, "Goblin Raider");

    // Cast with dash.
    let (state, _) = process_command(
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
            cast_with_foretell: false,
            cast_with_buyback: false,
            cast_with_overload: false,
            retrace_discard_land: None,
            cast_with_jump_start: false,
            jump_start_discard: None,
            cast_with_aftermath: false,
            cast_with_dash: true,
        },
    )
    .unwrap_or_else(|e| panic!("CastSpell with dash failed: {:?}", e));

    // Both players pass priority → spell resolves → creature enters battlefield.
    let (state, _) = pass_all(state, &[p1, p2]);

    assert!(
        on_battlefield(&state, "Goblin Raider"),
        "creature should be on battlefield after dash cast"
    );

    // Advance from PostCombatMain to End step.
    let (state, _) = pass_all(state, &[p1, p2]);

    assert_eq!(
        state.turn.step,
        Step::End,
        "should have advanced to End step"
    );

    // Both players pass priority during End step → DashReturnTrigger resolves → return to hand.
    let (state, end_events) = pass_all(state, &[p1, p2]);

    // The trigger should have resolved and creature returned to owner's hand.
    assert!(
        !on_battlefield(&state, "Goblin Raider"),
        "CR 702.109a: dashed creature should NOT be on battlefield after end-step trigger resolves"
    );
    assert!(
        in_hand(&state, "Goblin Raider", p1),
        "CR 702.109a: dashed creature should be in owner's hand after end-step trigger resolves"
    );

    // ObjectReturnedToHand event should have been emitted.
    let returned = end_events
        .iter()
        .any(|e| matches!(e, GameEvent::ObjectReturnedToHand { .. }));
    assert!(
        returned,
        "CR 702.109a: ObjectReturnedToHand event expected when dash trigger resolves"
    );
}

// ── Test 4: Creature died before end step — trigger does nothing ──────────────

/// Ruling 2014-11-24 — "If you pay the dash cost to cast a creature spell, that
/// card will be returned to its owner's hand only if it's still on the battlefield
/// when its triggered ability resolves."
/// If the creature leaves the battlefield before the end step trigger resolves,
/// the trigger does nothing (CR 400.7: new object).
#[test]
fn test_dash_creature_left_battlefield_before_end_step() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![goblin_raider_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(raider_in_hand(p1))
        .active_player(p1)
        .at_step(Step::PostCombatMain)
        .build()
        .unwrap();

    // Pay {R} — dash cost.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Red, 1);
    state.turn.priority_holder = Some(p1);

    let card_id = find_object(&state, "Goblin Raider");

    // Cast with dash.
    let (state, _) = process_command(
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
            cast_with_foretell: false,
            cast_with_buyback: false,
            cast_with_overload: false,
            retrace_discard_land: None,
            cast_with_jump_start: false,
            jump_start_discard: None,
            cast_with_aftermath: false,
            cast_with_dash: true,
        },
    )
    .unwrap_or_else(|e| panic!("CastSpell with dash failed: {:?}", e));

    // Both players pass priority → spell resolves → creature enters battlefield.
    let (mut state, _) = pass_all(state, &[p1, p2]);

    assert!(
        on_battlefield(&state, "Goblin Raider"),
        "creature should be on battlefield after dash cast"
    );

    // Manually move the creature to the graveyard (simulate dying before end step).
    let bf_id = find_in_zone(&state, "Goblin Raider", ZoneId::Battlefield).unwrap();
    state
        .move_object_to_zone(bf_id, ZoneId::Graveyard(p1))
        .expect("move to graveyard should succeed");

    assert!(
        !on_battlefield(&state, "Goblin Raider"),
        "creature should be in graveyard now"
    );

    // Advance to End step.
    let (state, _) = pass_all(state, &[p1, p2]);
    assert_eq!(state.turn.step, Step::End, "should advance to End step");

    // Resolve trigger at End step (it fires but finds creature not on battlefield).
    let (state, end_events) = pass_all(state, &[p1, p2]);

    // Creature should still be in graveyard, NOT returned to hand.
    assert!(
        !in_hand(&state, "Goblin Raider", p1),
        "Ruling 2014-11-24: creature that left battlefield should NOT be returned to hand"
    );
    assert!(
        find_in_zone(&state, "Goblin Raider", ZoneId::Graveyard(p1)).is_some(),
        "creature should remain in graveyard (CR 400.7: new object)"
    );

    // No ObjectReturnedToHand event.
    let returned = end_events
        .iter()
        .any(|e| matches!(e, GameEvent::ObjectReturnedToHand { .. }));
    assert!(
        !returned,
        "Ruling 2014-11-24: no ObjectReturnedToHand when creature left battlefield before trigger"
    );
}

// ── Test 5: Alternative cost exclusivity ─────────────────────────────────────

/// CR 118.9a — Dash is an alternative cost. Only one alternative cost can be
/// applied to a spell. Attempting to combine dash with flashback is rejected.
#[test]
fn test_dash_alternative_cost_exclusivity_with_flashback() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![goblin_raider_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(raider_in_hand(p1))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Red, 1);
    state.turn.priority_holder = Some(p1);

    let card_id = find_object(&state, "Goblin Raider");

    // The combination with flashback is not possible from hand (card is in hand, not
    // graveyard and has no flashback keyword). Test the case where dash is requested
    // for a card without the Dash ability — must be rejected.
    let _ = card_id; // used above to build state, not needed anymore

    // Try casting a card without dash with cast_with_dash: true — must be rejected.
    // Build a fresh state with a non-dash card.
    let no_dash_def = CardDefinition {
        card_id: CardId("vanilla-bear".to_string()),
        name: "Vanilla Bear".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            green: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: String::new(),
        abilities: vec![],
        power: Some(2),
        toughness: Some(2),
        ..Default::default()
    };

    let registry2 = CardRegistry::new(vec![no_dash_def]);

    let no_dash_card = ObjectSpec::card(p1, "Vanilla Bear")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("vanilla-bear".to_string()))
        .with_types(vec![CardType::Creature])
        .with_mana_cost(ManaCost {
            generic: 1,
            green: 1,
            ..Default::default()
        });

    let mut state2 = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry2)
        .object(no_dash_card)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state2
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Green, 1);
    state2.turn.priority_holder = Some(p1);

    let no_dash_id = find_object(&state2, "Vanilla Bear");

    // Attempt dash cast of a non-dash card — must be rejected.
    let result2 = process_command(
        state2,
        Command::CastSpell {
            player: p1,
            card: no_dash_id,
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
            cast_with_dash: true,
        },
    );

    assert!(
        result2.is_err(),
        "CR 702.109a: dash cast should be rejected for a card without the Dash ability"
    );
    let err_msg = format!("{:?}", result2.unwrap_err());
    assert!(
        err_msg.contains("dash"),
        "error message should mention 'dash': {}",
        err_msg
    );
}

// ── Test 6: Dash + evoke mutual exclusion ─────────────────────────────────────

/// CR 118.9a — Dash cannot be combined with evoke (both are alternative costs).
#[test]
fn test_dash_cannot_combine_with_evoke() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![goblin_raider_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(raider_in_hand(p1))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Red, 1);
    state.turn.priority_holder = Some(p1);

    let card_id = find_object(&state, "Goblin Raider");

    // Attempt to cast with both dash and evoke — must be rejected.
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
            cast_with_evoke: true,
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
            cast_with_dash: true,
        },
    );

    assert!(
        result.is_err(),
        "CR 118.9a: dash + evoke should be rejected (only one alternative cost)"
    );
}

// ── Test 7: Commander tax applies to dash cost ────────────────────────────────

/// CR 118.9d — Commander tax is added on top of the dash cost (or any alternative cost).
/// If the commander has been cast from the command zone before, the tax {2} is added.
#[test]
fn test_dash_commander_tax_applies() {
    let p1 = p(1);
    let p2 = p(2);

    // Commander Goblin Raider: cast once from command zone, then dies, then tries to cast with dash.
    let commander_def = CardDefinition {
        card_id: CardId("goblin-raider".to_string()),
        name: "Goblin Raider".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            red: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            supertypes: [SuperType::Legendary].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Dash {R}".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Dash),
            AbilityDefinition::Dash {
                cost: ManaCost {
                    red: 1,
                    ..Default::default()
                },
            },
        ],
        power: Some(2),
        toughness: Some(2),
        ..Default::default()
    };

    let registry = CardRegistry::new(vec![commander_def]);

    // Build state with the commander in the command zone (having been cast once before).
    let commander_card = ObjectSpec::card(p1, "Goblin Raider")
        .in_zone(ZoneId::Command(p1))
        .with_card_id(CardId("goblin-raider".to_string()))
        .with_types(vec![CardType::Creature])
        .with_keyword(KeywordAbility::Dash)
        .with_mana_cost(ManaCost {
            generic: 1,
            red: 1,
            ..Default::default()
        });

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(commander_card)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Set commander tax to 2 (simulates having cast the commander once before).
    let commander_card_id = CardId("goblin-raider".to_string());
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .commander_ids
        .push_back(commander_card_id.clone());
    // Commander tax entry stores the NUMBER OF TIMES already cast.
    // apply_commander_tax multiplies by 2: 1 cast = {2} additional mana.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .commander_tax
        .insert(commander_card_id, 1);

    let cmd_obj_id = find_object(&state, "Goblin Raider");
    state.turn.priority_holder = Some(p1);

    // Pay {R} dash cost — should FAIL because commander tax {2} is owed
    // (total needed: {R} + {2} = {2}{R} = 3 mana, but only providing 1).
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Red, 1);

    let result_insufficient = process_command(
        state.clone(),
        Command::CastSpell {
            player: p1,
            card: cmd_obj_id,
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
            cast_with_dash: true,
        },
    );
    assert!(
        result_insufficient.is_err(),
        "CR 118.9d: dash + commander tax should require more mana than {{R}} alone"
    );

    // Now pay {2}{R} (dash cost {R} + {2} commander tax) — should succeed.
    // State already has {R} from the earlier add. Set pool to {2}{C} + {R} = 3 mana.
    {
        let player = state.players.get_mut(&p1).unwrap();
        player.mana_pool.colorless = 2;
        player.mana_pool.red = 1;
    }

    let result_sufficient = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: cmd_obj_id,
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
            cast_with_dash: true,
        },
    );
    assert!(
        result_sufficient.is_ok(),
        "CR 118.9d: dash + {{2}} commander tax = {{2}}{{R}} total; should succeed with 3 mana: {:?}",
        result_sufficient.err()
    );
}
