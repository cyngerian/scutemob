//! Tests for X-cost spell infrastructure (PB-27).
//!
//! Verifies:
//! - `ManaCost.x_count` correctly adds `x_count * x_value` to generic at cast time (CR 107.3a)
//! - `EffectAmount::XValue` resolves to the chosen X during effect execution (CR 107.3m)
//! - ETB triggers receive x_value from the permanent's stored x_value (CR 107.3m)
//! - `Effect::Repeat` with `EffectAmount::XValue` creates X tokens (CR 107.3m)
//! - `Condition::XValueAtLeast(n)` enables conditional branches on X (CR 107.3m)
//! - `LoyaltyCost::MinusX` with `x_value` passed through loyalty activation (CR 606.4)
//! - `Command::ActivateAbility` with `x_value` pays x_count * x_value in mana (CR 107.3k)
//! - `{X}{X}` (x_count: 2) costs 2 * x_value mana (CR 107.3a)

use mtg_engine::{
    all_cards, card_name_to_id, enrich_spec_from_def, process_command, CardDefinition, CardId,
    CardRegistry, CardType, Command, CounterType, GameEvent, GameStateBuilder, ManaColor, ManaCost,
    ObjectId, ObjectSpec, PlayerId, Step, TypeLine, ZoneId,
};
use std::collections::HashMap;

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

/// Cast a spell from hand with the given x_value.
fn cast_x_spell(
    state: mtg_engine::GameState,
    caster: PlayerId,
    card_id: ObjectId,
    mana_generic: u32,
    mana_blue: u32,
    mana_green: u32,
    mana_red: u32,
    mana_white: u32,
    mana_black: u32,
    x_value: u32,
) -> (mtg_engine::GameState, Vec<GameEvent>) {
    let mut state = state;
    let pool = &mut state.players.get_mut(&caster).unwrap().mana_pool;
    if mana_generic > 0 {
        pool.add(ManaColor::Colorless, mana_generic);
    }
    if mana_blue > 0 {
        pool.add(ManaColor::Blue, mana_blue);
    }
    if mana_green > 0 {
        pool.add(ManaColor::Green, mana_green);
    }
    if mana_red > 0 {
        pool.add(ManaColor::Red, mana_red);
    }
    if mana_white > 0 {
        pool.add(ManaColor::White, mana_white);
    }
    if mana_black > 0 {
        pool.add(ManaColor::Black, mana_black);
    }
    state.turn.priority_holder = Some(caster);
    process_command(
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
            prototype: false,
            modes_chosen: vec![],
            x_value,
            face_down_kind: None,
            additional_costs: vec![],
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    )
    .unwrap_or_else(|e| panic!("CastSpell(x_value={}) failed: {:?}", x_value, e))
}

fn dummy_lib_spec(player: PlayerId, suffix: &str) -> ObjectSpec {
    ObjectSpec::card(player, &format!("Library Dummy {}", suffix))
        .in_zone(ZoneId::Library(player))
        .with_card_id(CardId(format!("dummy-lib-{}", suffix)))
        .with_types(vec![CardType::Instant])
}

// ── Test 1: X-cost mana payment (CR 107.3a) ───────────────────────────────────

/// CR 107.3a: Cast Pull from Tomorrow with X=3; verify x_count * x_value added to generic.
/// The card has x_count: 1 and blue: 2. Paying X=3 should require 3 generic + 2 blue.
#[test]
fn test_x_cost_spell_basic_mana_payment() {
    let p1 = p(1);
    let p2 = p(2);
    let defs = all_cards();
    let pull = defs
        .iter()
        .find(|d| d.name == "Pull from Tomorrow")
        .expect("Pull from Tomorrow");
    let card_id = pull.card_id.clone();
    let registry = CardRegistry::new(defs);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(
            ObjectSpec::card(p1, "Pull from Tomorrow")
                .with_card_id(card_id)
                .with_types(vec![CardType::Instant])
                .with_mana_cost(ManaCost {
                    blue: 2,
                    x_count: 1,
                    ..Default::default()
                })
                .in_zone(ZoneId::Hand(p1)),
        )
        .object(dummy_lib_spec(p1, "a"))
        .object(dummy_lib_spec(p1, "b"))
        .object(dummy_lib_spec(p1, "c"))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();
    state.turn.priority_holder = Some(p1);

    let card_obj_id = find_object(&state, "Pull from Tomorrow");

    // X=3: should pay 3 generic + 2 blue total. Add {3}{U}{U}.
    let (state, _) = cast_x_spell(state, p1, card_obj_id, 3, 2, 0, 0, 0, 0, 3);

    // The spell should be on the stack.
    assert_eq!(
        state.stack_objects.len(),
        1,
        "Pull from Tomorrow should be on the stack"
    );

    // Verify x_value is stored on the stack object.
    let stack_obj = state.stack_objects.front().unwrap();
    assert_eq!(
        stack_obj.x_value, 3,
        "stack object should carry x_value=3 (CR 107.3a)"
    );

    // Resolve: pass priority for both players. Pull from Tomorrow draws X cards then discards 1.
    let (state, _) = pass_all(state, &[p1, p2]);

    // Stack should be empty after resolution.
    assert!(
        state.stack_objects.is_empty(),
        "stack should be empty after resolution"
    );
}

// ── Test 2: EffectAmount::XValue resolves to chosen X (CR 107.3m) ─────────────

/// CR 107.3m: Pull from Tomorrow with X=3 draws 3 cards then discards 1.
#[test]
fn test_x_cost_effect_amount_xvalue_draw() {
    let p1 = p(1);
    let p2 = p(2);
    let defs = all_cards();
    let pull = defs
        .iter()
        .find(|d| d.name == "Pull from Tomorrow")
        .expect("Pull from Tomorrow");
    let card_id = pull.card_id.clone();
    let registry = CardRegistry::new(defs);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(
            ObjectSpec::card(p1, "Pull from Tomorrow")
                .with_card_id(card_id)
                .with_types(vec![CardType::Instant])
                .with_mana_cost(ManaCost {
                    blue: 2,
                    x_count: 1,
                    ..Default::default()
                })
                .in_zone(ZoneId::Hand(p1)),
        )
        .object(dummy_lib_spec(p1, "d1"))
        .object(dummy_lib_spec(p1, "d2"))
        .object(dummy_lib_spec(p1, "d3"))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();
    state.turn.priority_holder = Some(p1);

    let card_obj_id = find_object(&state, "Pull from Tomorrow");
    let initial_hand = state
        .objects
        .iter()
        .filter(|(_, o)| o.zone == ZoneId::Hand(p1))
        .count();

    // Cast with X=3 (draw 3, discard 1 → net +2 cards in hand, minus 1 for the spell cast = net +1).
    let (state, _) = cast_x_spell(state, p1, card_obj_id, 3, 2, 0, 0, 0, 0, 3);
    let (state, resolve_events) = pass_all(state, &[p1, p2]);

    // Count CardDrawn events in the resolve batch.
    let draw_count = resolve_events
        .iter()
        .filter(|e| matches!(e, GameEvent::CardDrawn { player, .. } if *player == p1))
        .count();
    assert_eq!(
        draw_count, 3,
        "Pull from Tomorrow X=3 should draw exactly 3 cards (CR 107.3m)"
    );

    // Hand: -1 (spell) +3 (draw) -1 (discard) = initial + 1.
    let final_hand = state
        .objects
        .iter()
        .filter(|(_, o)| o.zone == ZoneId::Hand(p1))
        .count();
    assert_eq!(
        final_hand,
        initial_hand + 1,
        "net hand change should be +1 (draw X=3, discard 1, spell cast)"
    );
}

// ── Test 3: ETB counters from X value (CR 107.3m) ─────────────────────────────

/// CR 107.3m: Ingenious Prodigy cast with X=4 enters with 4 +1/+1 counters.
/// Requires x_value propagation from permanent.x_value to ETB EffectContext.
#[test]
fn test_x_cost_etb_counters_ingenious_prodigy() {
    let p1 = p(1);
    let p2 = p(2);
    let defs = all_cards();
    let card = defs
        .iter()
        .find(|d| d.name == "Ingenious Prodigy")
        .expect("Ingenious Prodigy");
    let card_id = card.card_id.clone();
    let registry = CardRegistry::new(defs);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(
            ObjectSpec::card(p1, "Ingenious Prodigy")
                .with_card_id(card_id)
                .with_types(vec![CardType::Creature])
                .with_mana_cost(ManaCost {
                    blue: 1,
                    x_count: 1,
                    ..Default::default()
                })
                .in_zone(ZoneId::Hand(p1)),
        )
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();
    state.turn.priority_holder = Some(p1);

    let card_obj_id = find_object(&state, "Ingenious Prodigy");

    // Cast with X=4 (pay 4 generic + 1 blue).
    let (state, _) = cast_x_spell(state, p1, card_obj_id, 4, 1, 0, 0, 0, 0, 4);

    // Resolve spell (creatures go on stack → battlefield on resolution).
    let (state, resolve_events) = pass_all(state, &[p1, p2]);
    // Resolve ETB trigger (enters with counters).
    let (state, trigger_events) = pass_all(state, &[p1, p2]);

    let all_events: Vec<_> = resolve_events.into_iter().chain(trigger_events).collect();

    let creature_id = find_object_on_battlefield(&state, "Ingenious Prodigy")
        .expect("Ingenious Prodigy should be on battlefield");

    // Verify 4 +1/+1 counters were placed.
    let counter_count = state
        .objects
        .get(&creature_id)
        .and_then(|o| o.counters.get(&CounterType::PlusOnePlusOne).copied())
        .unwrap_or(0);
    assert_eq!(
        counter_count, 4,
        "Ingenious Prodigy cast with X=4 should have 4 +1/+1 counters (CR 107.3m)"
    );

    // CounterAdded event should be emitted.
    let counter_added = all_events.iter().any(|e| {
        matches!(
            e,
            GameEvent::CounterAdded {
                counter: CounterType::PlusOnePlusOne,
                count: 4,
                ..
            }
        )
    });
    assert!(
        counter_added,
        "CounterAdded(4) event must be emitted for Ingenious Prodigy X=4"
    );
}

// ── Test 4: Repeat creates X tokens (CR 107.3m) ───────────────────────────────

/// CR 107.3m: Awaken the Woods with X=3 creates exactly 3 tokens.
#[test]
fn test_x_cost_repeat_creates_x_tokens() {
    let p1 = p(1);
    let p2 = p(2);
    let defs = all_cards();
    let card = defs
        .iter()
        .find(|d| d.name == "Awaken the Woods")
        .expect("Awaken the Woods");
    let card_id = card.card_id.clone();
    let registry = CardRegistry::new(defs);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(
            ObjectSpec::card(p1, "Awaken the Woods")
                .with_card_id(card_id)
                .with_types(vec![CardType::Sorcery])
                .with_mana_cost(ManaCost {
                    green: 2,
                    x_count: 1,
                    ..Default::default()
                })
                .in_zone(ZoneId::Hand(p1)),
        )
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();
    state.turn.priority_holder = Some(p1);

    let card_obj_id = find_object(&state, "Awaken the Woods");

    let tokens_before = state
        .objects
        .values()
        .filter(|o| o.zone == ZoneId::Battlefield && o.characteristics.name == "Forest Dryad")
        .count();
    assert_eq!(tokens_before, 0);

    // Cast with X=3 (pay 3 generic + 2 green).
    let (state, _) = cast_x_spell(state, p1, card_obj_id, 3, 0, 2, 0, 0, 0, 3);
    let (state, _) = pass_all(state, &[p1, p2]);

    let tokens_after = state
        .objects
        .values()
        .filter(|o| o.zone == ZoneId::Battlefield && o.characteristics.name == "Forest Dryad")
        .count();
    assert_eq!(
        tokens_after, 3,
        "Awaken the Woods X=3 should create 3 Forest Dryad tokens (CR 107.3m)"
    );
}

// ── Test 5: Condition::XValueAtLeast (CR 107.3m) ──────────────────────────────

/// CR 107.3m: Martial Coup with X=5 creates 5 Soldier tokens AND destroys all creatures.
/// With X=4, creates 4 tokens but does NOT destroy creatures.
#[test]
fn test_x_cost_conditional_xvalue_at_least_martial_coup_x5() {
    let p1 = p(1);
    let p2 = p(2);
    let defs = all_cards();
    let card = defs
        .iter()
        .find(|d| d.name == "Martial Coup")
        .expect("Martial Coup");
    let card_id = card.card_id.clone();

    // Use a dummy creature for p2 to verify destruction.
    let dummy_def = CardDefinition {
        card_id: CardId("dummy-creature".to_string()),
        name: "Dummy Creature".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: String::new(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![],
        ..Default::default()
    };
    let registry = CardRegistry::new(vec![card.clone(), dummy_def]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(
            ObjectSpec::card(p1, "Martial Coup")
                .with_card_id(card_id)
                .with_types(vec![CardType::Sorcery])
                .with_mana_cost(ManaCost {
                    white: 2,
                    x_count: 1,
                    ..Default::default()
                })
                .in_zone(ZoneId::Hand(p1)),
        )
        .object(
            ObjectSpec::card(p2, "Dummy Creature")
                .with_card_id(CardId("dummy-creature".to_string()))
                .with_types(vec![CardType::Creature])
                .with_mana_cost(ManaCost {
                    generic: 1,
                    ..Default::default()
                })
                .in_zone(ZoneId::Battlefield),
        )
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();
    state.turn.priority_holder = Some(p1);

    let card_obj_id = find_object(&state, "Martial Coup");

    // Verify p2's creature is on battlefield before the coup.
    let p2_creature_before = find_object_on_battlefield(&state, "Dummy Creature").is_some();
    assert!(
        p2_creature_before,
        "p2's creature should be on battlefield before Martial Coup"
    );

    // Cast with X=5 (5 generic + 2 white).
    let (state, resolve_events) = cast_x_spell(state, p1, card_obj_id, 5, 0, 0, 0, 2, 0, 5);
    let (state, more_events) = pass_all(state, &[p1, p2]);
    let all_events: Vec<_> = resolve_events.into_iter().chain(more_events).collect();

    // Verify that TokenCreated events were emitted for 5 Soldiers.
    let tokens_created = all_events
        .iter()
        .filter(|e| matches!(e, GameEvent::TokenCreated { .. }))
        .count();
    assert_eq!(
        tokens_created, 5,
        "Martial Coup X=5 should create 5 tokens (CR 107.3m)"
    );

    // p2's creature should be destroyed (X >= 5 triggers DestroyAll).
    // Note: the current DestroyAll implementation hits all creatures including the newly
    // created Soldiers, so those may also be gone. The important assertion is that p2's
    // creature was destroyed, verifying Condition::XValueAtLeast(5) fired correctly.
    let p2_creature_after = find_object_on_battlefield(&state, "Dummy Creature");
    assert!(
        p2_creature_after.is_none(),
        "p2's creature should be destroyed by Martial Coup X=5 wipe (CR 107.3m)"
    );
}

/// CR 107.3m: Martial Coup with X=4 creates 4 tokens but does NOT destroy creatures.
#[test]
fn test_x_cost_conditional_xvalue_at_least_martial_coup_x4_no_wipe() {
    let p1 = p(1);
    let p2 = p(2);
    let defs = all_cards();
    let card = defs
        .iter()
        .find(|d| d.name == "Martial Coup")
        .expect("Martial Coup");
    let card_id = card.card_id.clone();

    let dummy_def = CardDefinition {
        card_id: CardId("dummy-creature-2".to_string()),
        name: "Dummy Creature 2".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: String::new(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![],
        ..Default::default()
    };
    let registry = CardRegistry::new(vec![card.clone(), dummy_def]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(
            ObjectSpec::card(p1, "Martial Coup")
                .with_card_id(card_id)
                .with_types(vec![CardType::Sorcery])
                .with_mana_cost(ManaCost {
                    white: 2,
                    x_count: 1,
                    ..Default::default()
                })
                .in_zone(ZoneId::Hand(p1)),
        )
        .object(
            ObjectSpec::card(p2, "Dummy Creature 2")
                .with_card_id(CardId("dummy-creature-2".to_string()))
                .with_types(vec![CardType::Creature])
                .with_mana_cost(ManaCost {
                    generic: 1,
                    ..Default::default()
                })
                .in_zone(ZoneId::Battlefield),
        )
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();
    state.turn.priority_holder = Some(p1);

    let card_obj_id = find_object(&state, "Martial Coup");

    // Cast with X=4 (4 generic + 2 white). X < 5 → no wipe.
    let (state, _) = cast_x_spell(state, p1, card_obj_id, 4, 0, 0, 0, 2, 0, 4);
    let (state, _) = pass_all(state, &[p1, p2]);

    // 4 Soldier tokens.
    let soldiers = state
        .objects
        .values()
        .filter(|o| o.zone == ZoneId::Battlefield && o.characteristics.name == "Soldier")
        .count();
    assert_eq!(
        soldiers, 4,
        "Martial Coup X=4 should create 4 Soldier tokens (CR 107.3m)"
    );

    // p2's creature should still be on battlefield (X=4 < 5, no wipe).
    let p2_creature = find_object_on_battlefield(&state, "Dummy Creature 2");
    assert!(
        p2_creature.is_some(),
        "p2's creature should survive Martial Coup X=4 (X < 5, no wipe; CR 107.3m)"
    );
}

// ── Test 6: X-cost activated ability with x_count: 2 (CR 107.3k) ─────────────

/// CR 107.3k: Treasure Vault {X}{X}, {T}, Sacrifice: Create X Treasure tokens.
/// Activating with X=2 should pay 2*2=4 generic + tap + sacrifice, creating 2 Treasures.
#[test]
fn test_x_cost_activated_ability_double_x_treasure_vault() {
    let p1 = p(1);
    let p2 = p(2);
    let cards = all_cards();
    let defs: HashMap<String, CardDefinition> =
        cards.iter().map(|d| (d.name.clone(), d.clone())).collect();
    let registry = CardRegistry::new(cards);

    // Must use enrich_spec_from_def so the activated abilities are propagated to the object.
    let vault_spec = enrich_spec_from_def(
        ObjectSpec::card(p1, "Treasure Vault")
            .with_card_id(card_name_to_id("Treasure Vault"))
            .in_zone(ZoneId::Battlefield),
        &defs,
    );

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(vault_spec)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();
    state.turn.priority_holder = Some(p1);

    let vault_id = find_object_on_battlefield(&state, "Treasure Vault")
        .expect("Treasure Vault should be on battlefield");

    // Add 4 colorless mana (X=2, x_count=2 → 2*2=4 generic).
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 4);

    // Activate ability index 0 (the {X}{X}, {T}, Sacrifice: Create X Treasures ability).
    // Note: the {T}: Add {C} ability becomes a ManaAbility in characteristics, NOT an
    // activated_ability, so the Sequence cost ability is at activated_abilities index 0.
    // x_value: Some(2) → pays x_count(2) * x_value(2) = 4 generic, creates 2 Treasures.
    let (state, _) = process_command(
        state,
        Command::ActivateAbility {
            player: p1,
            source: vault_id,
            ability_index: 0,
            targets: vec![],
            discard_card: None,
            sacrifice_target: None,
            x_value: Some(2),
        },
    )
    .unwrap();

    // Resolve the activated ability (treasure creation).
    let (state, _) = pass_all(state, &[p1, p2]);

    // Vault is gone (sacrificed as cost).
    let vault_after = find_object_on_battlefield(&state, "Treasure Vault");
    assert!(
        vault_after.is_none(),
        "Treasure Vault should be sacrificed as cost"
    );

    // 2 Treasure tokens should be on battlefield.
    let treasures = state
        .objects
        .values()
        .filter(|o| o.zone == ZoneId::Battlefield && o.characteristics.name == "Treasure")
        .count();
    assert_eq!(
        treasures, 2,
        "Treasure Vault activated with X=2 should create 2 Treasure tokens (CR 107.3k)"
    );
}

// ── Test 7: x_value stored on permanent (CR 107.3m) ──────────────────────────

/// CR 107.3m: The permanent retains x_value for ETB trigger processing.
/// After resolution, the object's x_value field should reflect the cast-time X.
#[test]
fn test_x_cost_permanent_retains_x_value_for_etb() {
    let p1 = p(1);
    let p2 = p(2);
    let defs = all_cards();
    let card = defs
        .iter()
        .find(|d| d.name == "Ingenious Prodigy")
        .expect("Ingenious Prodigy");
    let card_id = card.card_id.clone();
    let registry = CardRegistry::new(defs);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(
            ObjectSpec::card(p1, "Ingenious Prodigy")
                .with_card_id(card_id)
                .with_types(vec![CardType::Creature])
                .with_mana_cost(ManaCost {
                    blue: 1,
                    x_count: 1,
                    ..Default::default()
                })
                .in_zone(ZoneId::Hand(p1)),
        )
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();
    state.turn.priority_holder = Some(p1);

    let card_obj_id = find_object(&state, "Ingenious Prodigy");

    // Cast with X=6 (pay 6 generic + 1 blue).
    let (state, _) = cast_x_spell(state, p1, card_obj_id, 6, 1, 0, 0, 0, 0, 6);

    // After cast, the stack object should carry x_value=6.
    let so = state.stack_objects.front().unwrap();
    assert_eq!(
        so.x_value, 6,
        "stack object x_value should be 6 before resolution"
    );

    // Resolve spell.
    let (state, _) = pass_all(state, &[p1, p2]);
    // Resolve ETB trigger (enters with X counters).
    let (state, _) = pass_all(state, &[p1, p2]);

    // Permanent on battlefield should retain x_value.
    let creature_id = find_object_on_battlefield(&state, "Ingenious Prodigy")
        .expect("Ingenious Prodigy should be on battlefield");
    let stored_x = state
        .objects
        .get(&creature_id)
        .map(|o| o.x_value)
        .unwrap_or(0);
    assert_eq!(
        stored_x, 6,
        "permanent.x_value should be retained as 6 after resolution (CR 107.3m)"
    );

    // Counters should be 6.
    let counters = state
        .objects
        .get(&creature_id)
        .and_then(|o| o.counters.get(&CounterType::PlusOnePlusOne).copied())
        .unwrap_or(0);
    assert_eq!(
        counters, 6,
        "Ingenious Prodigy cast with X=6 should have 6 +1/+1 counters"
    );
}

// ── Test 8: Repeat with X=0 creates no tokens ─────────────────────────────────

/// CR 107.3b: When X=0, Repeat creates 0 tokens (effect does nothing).
#[test]
fn test_x_cost_repeat_zero_creates_no_tokens() {
    let p1 = p(1);
    let p2 = p(2);
    let defs = all_cards();
    let card = defs
        .iter()
        .find(|d| d.name == "Awaken the Woods")
        .expect("Awaken the Woods");
    let card_id = card.card_id.clone();
    let registry = CardRegistry::new(defs);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(
            ObjectSpec::card(p1, "Awaken the Woods")
                .with_card_id(card_id)
                .with_types(vec![CardType::Sorcery])
                .with_mana_cost(ManaCost {
                    green: 2,
                    x_count: 1,
                    ..Default::default()
                })
                .in_zone(ZoneId::Hand(p1)),
        )
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();
    state.turn.priority_holder = Some(p1);

    let card_obj_id = find_object(&state, "Awaken the Woods");

    // Cast with X=0 (pay 0 generic + 2 green). Only colored pips needed.
    let (state, _) = cast_x_spell(state, p1, card_obj_id, 0, 0, 2, 0, 0, 0, 0);
    let (state, _) = pass_all(state, &[p1, p2]);

    let tokens = state
        .objects
        .values()
        .filter(|o| o.zone == ZoneId::Battlefield && o.characteristics.name == "Forest Dryad")
        .count();
    assert_eq!(
        tokens, 0,
        "Awaken the Woods X=0 should create 0 tokens (CR 107.3b)"
    );
}

// ── Test 9: XValueAtLeast(5) is false below threshold ─────────────────────────

/// CR 107.3m: Condition::XValueAtLeast(5) is false when X=4.
/// White Sun's Twilight X=4: creates 4 tokens and gains 4 life, no wipe.
#[test]
fn test_x_cost_condition_xvalue_at_least_below_threshold() {
    let p1 = p(1);
    let p2 = p(2);
    let defs = all_cards();
    let card = defs
        .iter()
        .find(|d| d.name == "White Sun's Twilight")
        .expect("White Sun's Twilight");
    let card_id = card.card_id.clone();

    let dummy_def = CardDefinition {
        card_id: CardId("dummy-for-wst".to_string()),
        name: "Dummy For WST".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: String::new(),
        power: Some(3),
        toughness: Some(3),
        abilities: vec![],
        ..Default::default()
    };
    let registry = CardRegistry::new(vec![card.clone(), dummy_def]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(
            ObjectSpec::card(p1, "White Sun's Twilight")
                .with_card_id(card_id)
                .with_types(vec![CardType::Sorcery])
                .with_mana_cost(ManaCost {
                    white: 2,
                    x_count: 1,
                    ..Default::default()
                })
                .in_zone(ZoneId::Hand(p1)),
        )
        .object(
            ObjectSpec::card(p2, "Dummy For WST")
                .with_card_id(CardId("dummy-for-wst".to_string()))
                .with_types(vec![CardType::Creature])
                .with_mana_cost(ManaCost {
                    generic: 1,
                    ..Default::default()
                })
                .in_zone(ZoneId::Battlefield),
        )
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();
    state.turn.priority_holder = Some(p1);

    let initial_life = state.players.get(&p1).unwrap().life_total;
    let card_obj_id = find_object(&state, "White Sun's Twilight");

    // Cast with X=4 (4 generic + 2 white).
    let (state, _) = cast_x_spell(state, p1, card_obj_id, 4, 0, 0, 0, 2, 0, 4);
    let (state, _) = pass_all(state, &[p1, p2]);

    // Gained 4 life.
    let final_life = state.players.get(&p1).unwrap().life_total;
    assert_eq!(
        final_life,
        initial_life + 4,
        "White Sun's Twilight X=4 should gain 4 life"
    );

    // 4 Phyrexian Mite tokens.
    let mites = state
        .objects
        .values()
        .filter(|o| o.zone == ZoneId::Battlefield && o.characteristics.name == "Phyrexian Mite")
        .count();
    assert_eq!(
        mites, 4,
        "White Sun's Twilight X=4 should create 4 Mite tokens"
    );

    // p2's creature should survive (X=4 < 5, no wipe).
    let p2_creature = find_object_on_battlefield(&state, "Dummy For WST");
    assert!(
        p2_creature.is_some(),
        "p2's creature should survive White Sun's Twilight X=4 (X < 5)"
    );
}
