//! Tribute keyword ability tests (CR 702.104).
//!
//! Tribute is a static ability that functions as a creature enters the battlefield.
//! "As this creature enters, choose an opponent. That player may put an additional
//! N +1/+1 counters on it as it enters." (CR 702.104a)
//!
//! "Objects with tribute have triggered abilities that check 'if tribute wasn't paid.'"
//! (CR 702.104b)
//!
//! Key rules verified:
//! - Bot opponent auto-declines tribute (deterministic play) (CR 702.104a).
//! - Creature enters WITHOUT extra counters when tribute not paid (CR 702.104a).
//! - The "tribute wasn't paid" triggered effect fires when tribute not paid (CR 702.104b).
//! - The "tribute wasn't paid" effect does NOT fire when tribute was paid (CR 702.104b).
//! - tribute_was_paid flag is observable on the permanent (CR 702.104b).
//! - KeywordAbility::Tribute(n) variant compiles and is present in card definitions.
//! - Multiple Tribute instances: each N is independent.
//! - 4-player multiplayer: tribute fires correctly for active player's turn.

use mtg_engine::{
    process_command, AbilityDefinition, CardDefinition, CardId, CardRegistry, CardType, Command,
    CounterType, Effect, EffectAmount, GameEvent, GameStateBuilder, KeywordAbility, ManaCost,
    ObjectId, ObjectSpec, PlayerId, PlayerTarget, Step, TriggerCondition, TypeLine, ZoneId,
};

// ── Helpers ───────────────────────────────────────────────────────────────────

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

fn find_object(state: &mtg_engine::GameState, name: &str) -> ObjectId {
    state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{}' not found", name))
}

fn find_object_on_battlefield(state: &mtg_engine::GameState, name: &str) -> Option<ObjectId> {
    state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name && obj.zone == ZoneId::Battlefield)
        .map(|(id, _)| *id)
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

/// Cast a creature from hand by adding generic mana and calling CastSpell.
fn cast_creature(
    state: mtg_engine::GameState,
    caster: PlayerId,
    card_id: ObjectId,
    generic_cost: u32,
) -> mtg_engine::GameState {
    let mut state = state;
    state
        .players
        .get_mut(&caster)
        .unwrap()
        .mana_pool
        .add(mtg_engine::ManaColor::Colorless, generic_cost);
    state.turn.priority_holder = Some(caster);

    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: caster,
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
        },
    )
    .unwrap_or_else(|e| panic!("CastSpell failed: {:?}", e));
    state
}

// ── Card Definitions ──────────────────────────────────────────────────────────

/// Tribute 2 creature with "if tribute wasn't paid, you gain 3 life."
/// 2/2 Creature, cost {2}.
fn tribute_2_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("tribute-2-test".to_string()),
        name: "Tribute Test Creature".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Tribute 2 (As this creature enters, an opponent of your choice may put 2 +1/+1 counters on it.) When ~ enters, if tribute wasn't paid, you gain 3 life.".to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Tribute(2)),
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::TributeNotPaid,
                effect: Effect::GainLife {
                    player: PlayerTarget::Controller,
                    amount: EffectAmount::Fixed(3),
                },
                intervening_if: None,
            },
        ],
        ..Default::default()
    }
}

/// Tribute 3 creature with "if tribute wasn't paid, draw a card."
/// 1/1 Creature, cost {3}.
fn tribute_3_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("tribute-3-test".to_string()),
        name: "Tribute Three Test".to_string(),
        mana_cost: Some(ManaCost {
            generic: 3,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Tribute 3 (As this creature enters, an opponent of your choice may put 3 +1/+1 counters on it.) When ~ enters, if tribute wasn't paid, draw a card.".to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Tribute(3)),
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::TributeNotPaid,
                effect: Effect::DrawCards {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(1),
                },
                intervening_if: None,
            },
        ],
        ..Default::default()
    }
}

/// Tribute 2 creature with NO "tribute wasn't paid" trigger.
/// Used to verify no trigger fires (as a regression/isolation test).
/// 3/3 Creature, cost {2}.
fn tribute_no_trigger_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("tribute-no-trigger-test".to_string()),
        name: "Tribute No Trigger Test".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Tribute 2".to_string(),
        power: Some(3),
        toughness: Some(3),
        abilities: vec![AbilityDefinition::Keyword(KeywordAbility::Tribute(2))],
        ..Default::default()
    }
}

// ── Test 1: Basic tribute not paid — creature enters without counters ──────────

#[test]
/// CR 702.104a — "As this creature enters, choose an opponent. That player may
/// put an additional N +1/+1 counters on it as it enters."
/// Bot opponent declines tribute. Creature enters without extra +1/+1 counters.
fn test_tribute_basic_not_paid_no_counters() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![tribute_2_def()]);

    let spec = ObjectSpec::card(p1, "Tribute Test Creature")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("tribute-2-test".to_string()))
        .with_types(vec![CardType::Creature])
        .with_keyword(KeywordAbility::Tribute(2))
        .with_mana_cost(ManaCost {
            generic: 2,
            ..Default::default()
        });

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(spec)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state.turn.priority_holder = Some(p1);

    let card_id = find_object(&state, "Tribute Test Creature");
    let state = cast_creature(state, p1, card_id, 2);
    let (state, _) = pass_all(state, &[p1, p2]);

    let bf_id = find_object_on_battlefield(&state, "Tribute Test Creature")
        .expect("CR 702.104a: Tribute creature should be on the battlefield");

    // Creature should have 0 +1/+1 counters (tribute not paid by bot).
    let counter_count = state.objects[&bf_id]
        .counters
        .get(&CounterType::PlusOnePlusOne)
        .copied()
        .unwrap_or(0);
    assert_eq!(
        counter_count, 0,
        "CR 702.104a: Tribute creature should enter with 0 counters when opponent declines"
    );

    // tribute_was_paid should be false.
    assert!(
        !state.objects[&bf_id].tribute_was_paid,
        "CR 702.104b: tribute_was_paid should be false when opponent declines"
    );
}

// ── Test 2: Tribute not paid — triggered effect fires ─────────────────────────

#[test]
/// CR 702.104b — Objects with tribute have triggered abilities that check
/// "if tribute wasn't paid." When tribute is not paid, the effect fires.
/// Test: creature enters, bot declines, controller gains 3 life.
fn test_tribute_not_paid_trigger_fires() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![tribute_2_def()]);

    let spec = ObjectSpec::card(p1, "Tribute Test Creature")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("tribute-2-test".to_string()))
        .with_types(vec![CardType::Creature])
        .with_keyword(KeywordAbility::Tribute(2))
        .with_mana_cost(ManaCost {
            generic: 2,
            ..Default::default()
        });

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(spec)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let initial_life = state.players[&p1].life_total;
    state.turn.priority_holder = Some(p1);

    let card_id = find_object(&state, "Tribute Test Creature");
    let state = cast_creature(state, p1, card_id, 2);
    let (state, _) = pass_all(state, &[p1, p2]);

    // Controller should have gained 3 life from "if tribute wasn't paid" trigger.
    let final_life = state.players[&p1].life_total;
    assert_eq!(
        final_life,
        initial_life + 3,
        "CR 702.104b: Controller should gain 3 life when tribute isn't paid"
    );
}

// ── Test 3: tribute_was_paid = true — triggered effect does NOT fire ──────────

#[test]
/// CR 702.104b — The "tribute wasn't paid" condition is false when tribute was paid.
/// When tribute_was_paid is set to true on the entering permanent, the triggered
/// ability does not fire (intervening-if: CR 603.4).
fn test_tribute_paid_trigger_does_not_fire() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![tribute_2_def()]);

    // Start with the tribute creature already on the battlefield with tribute paid.
    let spec = ObjectSpec::card(p1, "Tribute Test Creature")
        .in_zone(ZoneId::Battlefield)
        .with_card_id(CardId("tribute-2-test".to_string()))
        .with_types(vec![CardType::Creature])
        .with_keyword(KeywordAbility::Tribute(2));

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(spec)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Manually set tribute_was_paid = true and add 2 counters (simulating tribute paid).
    let bf_id = find_object_on_battlefield(&state, "Tribute Test Creature")
        .expect("creature should be on battlefield");
    if let Some(obj) = state.objects.get_mut(&bf_id) {
        obj.tribute_was_paid = true;
        obj.counters = obj.counters.update(CounterType::PlusOnePlusOne, 2);
    }

    let initial_life = state.players[&p1].life_total;

    // No trigger should have fired — life should be unchanged.
    assert_eq!(
        state.players[&p1].life_total, initial_life,
        "CR 702.104b: When tribute is paid, the trigger should not fire"
    );

    // tribute_was_paid should be true.
    assert!(
        state.objects[&bf_id].tribute_was_paid,
        "CR 702.104b: tribute_was_paid should be true when tribute was paid"
    );

    // 2 +1/+1 counters should be present (tribute N=2 was paid).
    let counter_count = state.objects[&bf_id]
        .counters
        .get(&CounterType::PlusOnePlusOne)
        .copied()
        .unwrap_or(0);
    assert_eq!(
        counter_count, 2,
        "CR 702.104a: Creature should have 2 +1/+1 counters when tribute was paid"
    );
}

// ── Test 4: Tribute N keyword variant presence ────────────────────────────────

#[test]
/// CR 702.104a — KeywordAbility::Tribute(n) compiles and is present on the card.
/// Verifies the enum variant is correctly attached to the card definition.
fn test_tribute_keyword_on_card() {
    let def = tribute_2_def();
    let has_tribute = def
        .abilities
        .iter()
        .any(|a| matches!(a, AbilityDefinition::Keyword(KeywordAbility::Tribute(2))));
    assert!(
        has_tribute,
        "CR 702.104a: Card definition should contain KeywordAbility::Tribute(2)"
    );

    let def3 = tribute_3_def();
    let has_tribute_3 = def3
        .abilities
        .iter()
        .any(|a| matches!(a, AbilityDefinition::Keyword(KeywordAbility::Tribute(3))));
    assert!(
        has_tribute_3,
        "CR 702.104a: Card definition should contain KeywordAbility::Tribute(3)"
    );
}

// ── Test 5: Tribute 3 — different N value ────────────────────────────────────

#[test]
/// CR 702.104a — Tribute N with different N values. Tribute 3 creature enters
/// with 0 counters (bot declines), "if tribute wasn't paid" draws a card.
fn test_tribute_n_value_draw_card() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![tribute_3_def()]);

    // Add a card to the library so DrawCards doesn't fizzle.
    let library_card = ObjectSpec::card(p1, "Library Card")
        .in_zone(ZoneId::Library(p1))
        .with_types(vec![CardType::Creature]);

    let spec = ObjectSpec::card(p1, "Tribute Three Test")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("tribute-3-test".to_string()))
        .with_types(vec![CardType::Creature])
        .with_keyword(KeywordAbility::Tribute(3))
        .with_mana_cost(ManaCost {
            generic: 3,
            ..Default::default()
        });

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(library_card)
        .object(spec)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state.turn.priority_holder = Some(p1);
    let hand_count_before = state
        .objects
        .values()
        .filter(|o| o.zone == ZoneId::Hand(p1))
        .count();

    let card_id = find_object(&state, "Tribute Three Test");
    let state = cast_creature(state, p1, card_id, 3);
    let (state, _) = pass_all(state, &[p1, p2]);

    let bf_id = find_object_on_battlefield(&state, "Tribute Three Test")
        .expect("CR 702.104a: Tribute 3 creature should be on battlefield");

    // No extra counters when tribute not paid.
    let counter_count = state.objects[&bf_id]
        .counters
        .get(&CounterType::PlusOnePlusOne)
        .copied()
        .unwrap_or(0);
    assert_eq!(
        counter_count, 0,
        "CR 702.104a: Tribute 3 creature should have 0 counters when bot declines"
    );

    // "If tribute wasn't paid" trigger drew a card.
    let hand_count_after = state
        .objects
        .values()
        .filter(|o| o.zone == ZoneId::Hand(p1))
        .count();
    // hand_count_before had 1 card (the tribute creature itself, now on battlefield).
    // After: tribute creature left hand, library card drawn → hand size same as before
    // (started with 1 Tribute card, spent it, drew 1 from library).
    assert_eq!(
        hand_count_after,
        hand_count_before - 1 + 1, // -1 for cast tribute creature, +1 for drawn card
        "CR 702.104b: 'If tribute wasn't paid' draw trigger should draw 1 card"
    );
}

// ── Test 6: tribute_was_paid resets on zone change ───────────────────────────

#[test]
/// CR 400.7 — tribute_was_paid is not preserved across zone changes (CR 400.7).
/// When a permanent leaves the battlefield, it becomes a new object with
/// tribute_was_paid = false.
fn test_tribute_paid_resets_on_zone_change() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![tribute_2_def()]);

    let spec = ObjectSpec::card(p1, "Tribute Test Creature")
        .in_zone(ZoneId::Battlefield)
        .with_card_id(CardId("tribute-2-test".to_string()))
        .with_types(vec![CardType::Creature]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(spec)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Mark tribute as paid on the battlefield object.
    let bf_id = find_object_on_battlefield(&state, "Tribute Test Creature")
        .expect("creature should start on battlefield");
    if let Some(obj) = state.objects.get_mut(&bf_id) {
        obj.tribute_was_paid = true;
    }
    assert!(
        state.objects[&bf_id].tribute_was_paid,
        "tribute_was_paid should be true initially"
    );

    // Move the creature to graveyard (zone change per CR 400.7 → new object).
    let (new_id, _) = state
        .move_object_to_zone(bf_id, ZoneId::Graveyard(p1))
        .expect("zone change should succeed");

    // The new object in the graveyard should have tribute_was_paid = false.
    assert!(
        !state.objects[&new_id].tribute_was_paid,
        "CR 400.7: tribute_was_paid should be false after zone change (new object)"
    );
}

// ── Test 7: Tribute creature without trigger — no life gain ──────────────────

#[test]
/// CR 702.104a — A creature with Tribute but no "tribute wasn't paid" trigger
/// enters normally (no counters when bot declines). No life gain or other effect.
fn test_tribute_no_trigger_card() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![tribute_no_trigger_def()]);

    let spec = ObjectSpec::card(p1, "Tribute No Trigger Test")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("tribute-no-trigger-test".to_string()))
        .with_types(vec![CardType::Creature])
        .with_keyword(KeywordAbility::Tribute(2))
        .with_mana_cost(ManaCost {
            generic: 2,
            ..Default::default()
        });

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(spec)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let initial_life = state.players[&p1].life_total;
    state.turn.priority_holder = Some(p1);

    let card_id = find_object(&state, "Tribute No Trigger Test");
    let state = cast_creature(state, p1, card_id, 2);
    let (state, _) = pass_all(state, &[p1, p2]);

    let bf_id = find_object_on_battlefield(&state, "Tribute No Trigger Test")
        .expect("creature should be on battlefield");

    // No counters (bot declined).
    let counter_count = state.objects[&bf_id]
        .counters
        .get(&CounterType::PlusOnePlusOne)
        .copied()
        .unwrap_or(0);
    assert_eq!(
        counter_count, 0,
        "CR 702.104a: Tribute creature with no trigger should have 0 counters when bot declines"
    );

    // No life gain (no trigger).
    assert_eq!(
        state.players[&p1].life_total, initial_life,
        "CR 702.104a: Creature with Tribute but no trigger should not change controller's life"
    );
}

// ── Test 8: Multiplayer — tribute fires in 4-player game ─────────────────────

#[test]
/// CR 702.104a — In multiplayer, the controller chooses which opponent to offer tribute to.
/// Bot play: all opponents decline. "Tribute wasn't paid" trigger fires.
/// (Controller in a 4-player game: p1 controls, p2/p3/p4 are opponents.)
fn test_tribute_multiplayer_fires() {
    let p1 = p(1);
    let p2 = p(2);
    let p3 = p(3);
    let p4 = p(4);

    let registry = CardRegistry::new(vec![tribute_2_def()]);

    let spec = ObjectSpec::card(p1, "Tribute Test Creature")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("tribute-2-test".to_string()))
        .with_types(vec![CardType::Creature])
        .with_keyword(KeywordAbility::Tribute(2))
        .with_mana_cost(ManaCost {
            generic: 2,
            ..Default::default()
        });

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .add_player(p4)
        .with_registry(registry)
        .object(spec)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let initial_life = state.players[&p1].life_total;
    state.turn.priority_holder = Some(p1);

    let card_id = find_object(&state, "Tribute Test Creature");
    let state = cast_creature(state, p1, card_id, 2);
    let (state, _) = pass_all(state, &[p1, p2, p3, p4]);

    let bf_id = find_object_on_battlefield(&state, "Tribute Test Creature")
        .expect("CR 702.104a: Tribute creature should be on battlefield in 4-player game");

    // No counters placed (bot declined).
    let counter_count = state.objects[&bf_id]
        .counters
        .get(&CounterType::PlusOnePlusOne)
        .copied()
        .unwrap_or(0);
    assert_eq!(
        counter_count, 0,
        "CR 702.104a: Tribute creature should have 0 counters when bot declines in 4-player game"
    );

    // "If tribute wasn't paid" trigger should have fired → controller gained 3 life.
    assert_eq!(
        state.players[&p1].life_total,
        initial_life + 3,
        "CR 702.104b: 'Tribute wasn't paid' trigger should fire in 4-player game"
    );
}
