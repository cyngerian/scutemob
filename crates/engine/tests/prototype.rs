//! Prototype keyword ability tests (CR 702.160 / CR 718).
//!
//! Prototype is NOT an alternative cost (CR 118.9, ruling 2022-10-14). It is a static
//! ability that allows the player to choose, at cast time, whether to cast the card
//! using its prototype mana cost, power, and toughness instead of the printed values.
//!
//! Key rules verified:
//! - Cast prototyped: pay prototype cost, permanent has prototype P/T and color (CR 718.3, 718.3b).
//! - Cast normally: pay printed cost, permanent has printed P/T and color (CR 718.4).
//! - Prototype color comes from prototype mana cost symbols (CR 718.3b, 105.2).
//! - Mana value of prototyped spell/permanent = prototype cost MV (ruling 2022-10-14, Fateful Handoff).
//! - Prototype characteristics apply only on stack and battlefield; other zones use printed chars (CR 718.4).
//! - Abilities, name, types, and subtypes are unchanged by prototype (CR 718.5).
//! - Card without Prototype ability cannot be cast with prototype: true (CR 702.160a).
//! - SBAs apply to prototype P/T (CR 704.5f).

use mtg_engine::state::types::AltCostKind;
use mtg_engine::{
    process_command, AbilityDefinition, AltCastDetails, CardDefinition, CardId, CardRegistry,
    CardType, Color, Command, GameEvent, GameState, GameStateBuilder, KeywordAbility, ManaColor,
    ManaCost, ObjectId, ObjectSpec, PlayerId, Step, TypeLine, ZoneId,
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

fn in_graveyard(state: &GameState, name: &str, owner: PlayerId) -> bool {
    find_in_zone(state, name, ZoneId::Graveyard(owner)).is_some()
}

/// Pass priority for all listed players once (resolves top of stack if all pass).
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

/// Blitz Automaton (mock):
/// Artifact Creature — Construct {7} (colorless), 6/4.
/// Prototype {2}{R} — 3/2.
/// (Also has Haste via the Prototype ability on the real card, but omitted for simplicity.)
fn blitz_automaton_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("blitz-automaton".to_string()),
        name: "Blitz Automaton".to_string(),
        mana_cost: Some(ManaCost {
            generic: 7,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Artifact, CardType::Creature]
                .into_iter()
                .collect(),
            ..Default::default()
        },
        oracle_text: "Prototype {2}{R} — 3/2".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Prototype),
            AbilityDefinition::AltCastAbility {
                kind: AltCostKind::Prototype,
                cost: ManaCost {
                    generic: 2,
                    red: 1,
                    ..Default::default()
                },
                details: Some(AltCastDetails::Prototype {
                    power: 3,
                    toughness: 2,
                }),
            },
        ],
        power: Some(6),
        toughness: Some(4),
        ..Default::default()
    }
}

/// Blitz Automaton in hand.
fn automaton_in_hand(owner: PlayerId) -> ObjectSpec {
    let mut spec = ObjectSpec::card(owner, "Blitz Automaton")
        .in_zone(ZoneId::Hand(owner))
        .with_card_id(CardId("blitz-automaton".to_string()))
        .with_types(vec![CardType::Artifact, CardType::Creature])
        .with_keyword(KeywordAbility::Prototype)
        .with_mana_cost(ManaCost {
            generic: 7,
            ..Default::default()
        });
    spec.power = Some(6);
    spec.toughness = Some(4);
    spec
}

/// Plain Construct (mock): no Prototype ability.
fn plain_construct_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("plain-construct".to_string()),
        name: "Plain Construct".to_string(),
        mana_cost: Some(ManaCost {
            generic: 3,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Artifact, CardType::Creature]
                .into_iter()
                .collect(),
            ..Default::default()
        },
        oracle_text: "".to_string(),
        abilities: vec![],
        power: Some(2),
        toughness: Some(2),
        ..Default::default()
    }
}

fn plain_construct_in_hand(owner: PlayerId) -> ObjectSpec {
    let mut spec = ObjectSpec::card(owner, "Plain Construct")
        .in_zone(ZoneId::Hand(owner))
        .with_card_id(CardId("plain-construct".to_string()))
        .with_types(vec![CardType::Artifact, CardType::Creature])
        .with_mana_cost(ManaCost {
            generic: 3,
            ..Default::default()
        });
    spec.power = Some(2);
    spec.toughness = Some(2);
    spec
}

// ── Test 1: Basic prototype cast ───────────────────────────────────────────────

/// CR 702.160a / CR 718.3 / CR 718.3b — Cast Blitz Automaton as a prototyped spell.
/// Verify: prototype cost {2}{R} is paid; permanent enters with prototype P/T (3/2);
/// permanent is red (from {R} in prototype cost).
#[test]
fn test_prototype_basic_cast() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![blitz_automaton_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(automaton_in_hand(p1))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Pay {2}{R} — prototype cost.
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

    let card_id = find_object(&state, "Blitz Automaton");

    // Cast as prototyped spell.
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
            alt_cost: None,
            prototype: true,
            modes_chosen: vec![],
            x_value: 0,
            face_down_kind: None,
            additional_costs: vec![],
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    )
    .unwrap_or_else(|e| panic!("CastSpell with prototype failed: {:?}", e));

    // Spell is on the stack with was_prototyped = true.
    assert_eq!(
        state.stack_objects.len(),
        1,
        "CR 718.3: prototyped spell should be on the stack"
    );
    assert!(
        state.stack_objects[0].was_prototyped,
        "CR 718.3b: was_prototyped should be true on stack object"
    );

    // Mana consumed: {2}{R} = 3 mana total (not {7}).
    let pool = &state.players[&p1].mana_pool;
    assert_eq!(
        pool.white + pool.blue + pool.black + pool.red + pool.green + pool.colorless,
        0,
        "CR 718.3a: prototype cost {{2}}{{R}} should be fully deducted"
    );

    // Resolve the spell.
    let (state, _) = pass_all(state, &[p1, p2]);

    // Permanent is on the battlefield.
    assert!(
        on_battlefield(&state, "Blitz Automaton"),
        "CR 718.3b: prototyped permanent should be on battlefield"
    );

    let bf_id = find_in_zone(&state, "Blitz Automaton", ZoneId::Battlefield).unwrap();
    let obj = &state.objects[&bf_id];

    // Prototype P/T.
    assert_eq!(
        obj.characteristics.power,
        Some(3),
        "CR 718.3b: prototype power should be 3"
    );
    assert_eq!(
        obj.characteristics.toughness,
        Some(2),
        "CR 718.3b: prototype toughness should be 2"
    );

    // Prototype color: red (from {R} in {2}{R}).
    assert!(
        obj.characteristics.colors.contains(&Color::Red),
        "CR 718.3b / CR 105.2: prototyped permanent should be red"
    );
    assert_eq!(
        obj.characteristics.colors.len(),
        1,
        "CR 718.3b: should be only red (not colorless)"
    );

    // is_prototyped flag set.
    assert!(
        obj.is_prototyped,
        "CR 718.3b: is_prototyped flag should be true on battlefield permanent"
    );
}

// ── Test 2: Normal (non-prototype) cast ─────────────────────────────────────────

/// CR 718.4 — Cast Blitz Automaton normally for {7}.
/// Verify: full cost paid; permanent has printed P/T (6/4); colorless (no colored mana in {7}).
#[test]
fn test_prototype_normal_cast() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![blitz_automaton_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(automaton_in_hand(p1))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Pay {7} — normal cost (all colorless).
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 7);
    state.turn.priority_holder = Some(p1);

    let card_id = find_object(&state, "Blitz Automaton");

    // Cast without prototype.
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
            alt_cost: None,
            prototype: false,
            modes_chosen: vec![],
            x_value: 0,
            face_down_kind: None,
            additional_costs: vec![],
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    )
    .unwrap_or_else(|e| panic!("CastSpell normally failed: {:?}", e));

    // was_prototyped = false on stack.
    assert!(
        !state.stack_objects[0].was_prototyped,
        "CR 718.4: was_prototyped should be false for normal cast"
    );

    // Mana consumed: {7}.
    let pool = &state.players[&p1].mana_pool;
    assert_eq!(
        pool.white + pool.blue + pool.black + pool.red + pool.green + pool.colorless,
        0,
        "CR 718.4: normal cost {{7}} should be fully deducted"
    );

    // Resolve.
    let (state, _) = pass_all(state, &[p1, p2]);

    assert!(
        on_battlefield(&state, "Blitz Automaton"),
        "CR 718.4: normally-cast permanent should be on battlefield"
    );

    let bf_id = find_in_zone(&state, "Blitz Automaton", ZoneId::Battlefield).unwrap();
    let obj = &state.objects[&bf_id];

    // Printed P/T.
    assert_eq!(
        obj.characteristics.power,
        Some(6),
        "CR 718.4: normally-cast permanent should have printed power 6"
    );
    assert_eq!(
        obj.characteristics.toughness,
        Some(4),
        "CR 718.4: normally-cast permanent should have printed toughness 4"
    );

    // Colorless (no colored mana in {7}).
    assert!(
        obj.characteristics.colors.is_empty(),
        "CR 718.4: normally-cast Blitz Automaton should be colorless"
    );

    // is_prototyped = false.
    assert!(
        !obj.is_prototyped,
        "CR 718.4: is_prototyped should be false for normally-cast permanent"
    );
}

// ── Test 3: Prototype color from mana cost ────────────────────────────────────

/// CR 718.3b / CR 105.2 — Prototype {2}{R} makes the permanent red.
/// Non-prototyped Blitz Automaton ({7}) is colorless.
/// This test isolates the color inference logic.
#[test]
fn test_prototype_color_change() {
    let p1 = p(1);
    let p2 = p(2);

    // ── Prototyped cast: should be red ──
    let registry = CardRegistry::new(vec![blitz_automaton_def()]);
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(automaton_in_hand(p1))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

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

    let card_id = find_object(&state, "Blitz Automaton");
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
            alt_cost: None,
            prototype: true,
            modes_chosen: vec![],
            x_value: 0,
            face_down_kind: None,
            additional_costs: vec![],
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    )
    .unwrap();
    let (state, _) = pass_all(state, &[p1, p2]);

    let bf_id = find_in_zone(&state, "Blitz Automaton", ZoneId::Battlefield).unwrap();
    assert!(
        state.objects[&bf_id]
            .characteristics
            .colors
            .contains(&Color::Red),
        "CR 718.3b / CR 105.2: prototype {{2}}{{R}} should make permanent red"
    );
    assert!(
        !state.objects[&bf_id]
            .characteristics
            .colors
            .contains(&Color::Blue),
        "CR 718.3b: prototype {{2}}{{R}} should not be blue"
    );
    assert!(
        !state.objects[&bf_id]
            .characteristics
            .colors
            .contains(&Color::White),
        "CR 718.3b: prototype {{2}}{{R}} should not be white"
    );
}

// ── Test 4: Mana value uses prototype cost ────────────────────────────────────

/// CR 718.3b (ruling 2022-10-14, Fateful Handoff) — The mana value of a prototyped
/// permanent is based on the prototype mana cost, not the printed mana cost.
/// Prototype {2}{R} has MV 3 (2 + 1 = 3), not 7.
#[test]
fn test_prototype_mana_value() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![blitz_automaton_def()]);
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(automaton_in_hand(p1))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

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

    let card_id = find_object(&state, "Blitz Automaton");
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
            alt_cost: None,
            prototype: true,
            modes_chosen: vec![],
            x_value: 0,
            face_down_kind: None,
            additional_costs: vec![],
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    )
    .unwrap();
    let (state, _) = pass_all(state, &[p1, p2]);

    let bf_id = find_in_zone(&state, "Blitz Automaton", ZoneId::Battlefield).unwrap();
    let mana_cost = state.objects[&bf_id]
        .characteristics
        .mana_cost
        .as_ref()
        .expect("prototyped permanent should have a mana cost");

    assert_eq!(
        mana_cost.mana_value(),
        3,
        "CR 718.3b: mana value of prototyped {{2}}{{R}} permanent should be 3 (not 7)"
    );
    assert_eq!(
        mana_cost.red, 1,
        "CR 718.3b: prototype cost should have 1 red pip"
    );
    assert_eq!(
        mana_cost.generic, 2,
        "CR 718.3b: prototype cost should have 2 generic"
    );
}

// ── Test 5: Leaves battlefield — resumes normal characteristics ───────────────

/// CR 718.4 (ruling 2022-10-14) — When a prototyped permanent leaves the battlefield
/// and enters another zone, it resumes its normal characteristics in that zone.
/// A prototyped Blitz Automaton bounced to hand is a {7}, colorless, 6/4 card.
#[test]
fn test_prototype_leaves_battlefield_resumes_normal() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![blitz_automaton_def()]);
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(automaton_in_hand(p1))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Cast prototyped.
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

    let card_id = find_object(&state, "Blitz Automaton");
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
            alt_cost: None,
            prototype: true,
            modes_chosen: vec![],
            x_value: 0,
            face_down_kind: None,
            additional_costs: vec![],
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    )
    .unwrap();
    let (state, _) = pass_all(state, &[p1, p2]);

    // Verify prototyped on battlefield.
    let bf_id = find_in_zone(&state, "Blitz Automaton", ZoneId::Battlefield).unwrap();
    assert!(
        state.objects[&bf_id].is_prototyped,
        "precondition: permanent should be prototyped on battlefield"
    );
    assert_eq!(
        state.objects[&bf_id].characteristics.power,
        Some(3),
        "precondition: prototype P/T should be 3/2 on battlefield"
    );

    // Bounce to hand: simulate zone change by using BounceToHand command indirectly.
    // We use the state's zone-move infrastructure via move_object which resets flags (CR 400.7).
    // Instead, directly test: after zone change, the NEW object in hand should have
    // printed characteristics (the builder.rs / mod.rs reset sets is_prototyped=false,
    // and card definitions provide the printed power/toughness).
    // We verify this by creating a fresh hand object (simulating the card returning to hand):
    let hand_state = {
        let mut s = state.clone();
        // Move from battlefield to hand: this triggers CR 400.7 — new object, is_prototyped cleared.
        // Use the engine's move_object_to_zone which resets flags.
        let result = s.move_object_to_zone(bf_id, ZoneId::Hand(p1));
        assert!(result.is_ok(), "move_object_to_zone to hand should succeed");
        s
    };

    // The object in hand should have is_prototyped = false (flag cleared on zone change).
    let hand_id = find_in_zone(&hand_state, "Blitz Automaton", ZoneId::Hand(p1))
        .expect("Blitz Automaton should be in p1's hand after bounce");
    assert!(
        !hand_state.objects[&hand_id].is_prototyped,
        "CR 718.4: is_prototyped should be false after zone change to hand (CR 400.7)"
    );
    // CR 718.4: characteristics should revert to printed values after zone change.
    assert_eq!(
        hand_state.objects[&hand_id].characteristics.power,
        Some(6),
        "CR 718.4: power should revert to printed value 6 in hand"
    );
    assert_eq!(
        hand_state.objects[&hand_id].characteristics.toughness,
        Some(4),
        "CR 718.4: toughness should revert to printed value 4 in hand"
    );
    let mc = hand_state.objects[&hand_id]
        .characteristics
        .mana_cost
        .as_ref()
        .unwrap();
    assert_eq!(
        mc.mana_value(),
        7,
        "CR 718.4: mana value should revert to 7 in hand"
    );
    assert!(
        hand_state.objects[&hand_id]
            .characteristics
            .colors
            .is_empty(),
        "CR 718.4: colors should revert to colorless in hand"
    );
}

// ── Test 6: Graveyard has normal characteristics ──────────────────────────────

/// CR 718.4 — In every zone except the stack or battlefield (when cast as prototyped),
/// a prototype card has only its normal characteristics.
/// A Blitz Automaton in the graveyard is {7}, colorless, 6/4.
/// We test this by placing the card directly in the graveyard and verifying the spec.
#[test]
fn test_prototype_in_graveyard_normal_chars() {
    let p1 = p(1);
    let p2 = p(2);

    // Place a Blitz Automaton directly in the graveyard (simulating it was there from start,
    // not cast prototyped). The object spec should have printed characteristics.
    let spec = ObjectSpec::card(p1, "Blitz Automaton")
        .in_zone(ZoneId::Graveyard(p1))
        .with_card_id(CardId("blitz-automaton".to_string()))
        .with_types(vec![CardType::Artifact, CardType::Creature])
        .with_keyword(KeywordAbility::Prototype)
        .with_mana_cost(ManaCost {
            generic: 7,
            ..Default::default()
        });

    let registry = CardRegistry::new(vec![blitz_automaton_def()]);
    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object({
            let mut s = spec;
            s.power = Some(6);
            s.toughness = Some(4);
            s
        })
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let gy_id = find_in_zone(&state, "Blitz Automaton", ZoneId::Graveyard(p1))
        .expect("Blitz Automaton should be in graveyard");
    let obj = &state.objects[&gy_id];

    // CR 718.4: graveyard object has normal (printed) characteristics.
    assert!(
        !obj.is_prototyped,
        "CR 718.4: card in graveyard should not have is_prototyped = true"
    );
    assert_eq!(
        obj.characteristics.power,
        Some(6),
        "CR 718.4: graveyard card should have printed power 6"
    );
    assert_eq!(
        obj.characteristics.toughness,
        Some(4),
        "CR 718.4: graveyard card should have printed toughness 4"
    );
    // Mana cost in graveyard = normal {7}.
    let mc = obj.characteristics.mana_cost.as_ref().unwrap();
    assert_eq!(
        mc.mana_value(),
        7,
        "CR 718.4: graveyard card should have normal mana value 7"
    );
    // Colorless (no colored mana in {7}).
    assert!(
        obj.characteristics.colors.is_empty(),
        "CR 718.4: graveyard card should be colorless"
    );
}

// ── Test 7: Abilities unchanged by prototype ──────────────────────────────────

/// CR 718.5 — A prototype card's abilities, name, types, and subtypes remain the same
/// whether cast as a prototyped spell or normally. Only mana cost, MV, color,
/// power, and toughness change.
#[test]
fn test_prototype_retains_keyword_ability() {
    let p1 = p(1);
    let p2 = p(2);

    // Use a card that also has Haste (added for this test).
    let haste_automaton_def = CardDefinition {
        card_id: CardId("haste-automaton".to_string()),
        name: "Haste Automaton".to_string(),
        mana_cost: Some(ManaCost {
            generic: 7,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Artifact, CardType::Creature]
                .into_iter()
                .collect(),
            ..Default::default()
        },
        oracle_text: "Prototype {2}{R} — 3/2\nHaste".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Prototype),
            AbilityDefinition::AltCastAbility {
                kind: AltCostKind::Prototype,
                cost: ManaCost {
                    generic: 2,
                    red: 1,
                    ..Default::default()
                },
                details: Some(AltCastDetails::Prototype {
                    power: 3,
                    toughness: 2,
                }),
            },
            AbilityDefinition::Keyword(KeywordAbility::Haste),
        ],
        power: Some(6),
        toughness: Some(4),
        ..Default::default()
    };

    let registry = CardRegistry::new(vec![haste_automaton_def]);

    let mut spec = ObjectSpec::card(p1, "Haste Automaton")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("haste-automaton".to_string()))
        .with_types(vec![CardType::Artifact, CardType::Creature])
        .with_keyword(KeywordAbility::Prototype)
        .with_keyword(KeywordAbility::Haste)
        .with_mana_cost(ManaCost {
            generic: 7,
            ..Default::default()
        });
    spec.power = Some(6);
    spec.toughness = Some(4);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(spec)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

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

    let card_id = find_object(&state, "Haste Automaton");
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
            alt_cost: None,
            prototype: true,
            modes_chosen: vec![],
            x_value: 0,
            face_down_kind: None,
            additional_costs: vec![],
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    )
    .unwrap();
    let (state, _) = pass_all(state, &[p1, p2]);

    let bf_id = find_in_zone(&state, "Haste Automaton", ZoneId::Battlefield).unwrap();
    let obj = &state.objects[&bf_id];

    // CR 718.5: Haste ability should still be present after prototyped cast.
    assert!(
        obj.characteristics
            .keywords
            .contains(&KeywordAbility::Haste),
        "CR 718.5: prototyped permanent should retain Haste keyword"
    );

    // CR 718.5: Prototype keyword should still be present (it's an ability on the card).
    assert!(
        obj.characteristics
            .keywords
            .contains(&KeywordAbility::Prototype),
        "CR 718.5: prototyped permanent should retain Prototype keyword"
    );

    // P/T should still be prototype values (3/2).
    assert_eq!(
        obj.characteristics.power,
        Some(3),
        "CR 718.3b: prototype P/T should be 3/2 even though Haste is present"
    );
}

// ── Test 8: Card without Prototype ability cannot use prototype: true ─────────

/// CR 702.160a — Only cards with the Prototype ability can be cast with prototype: true.
/// Attempting to cast a plain creature with prototype: true returns an error.
#[test]
fn test_prototype_negative_not_prototype_keyword() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![plain_construct_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(plain_construct_in_hand(p1))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 3);
    state.turn.priority_holder = Some(p1);

    let card_id = find_object(&state, "Plain Construct");

    // Attempting prototype: true on a card without Prototype ability should fail.
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
            prototype: true,
            modes_chosen: vec![],
            x_value: 0,
            face_down_kind: None,
            additional_costs: vec![],
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    );

    assert!(
        result.is_err(),
        "CR 702.160a: casting with prototype: true should fail if card lacks Prototype ability"
    );
}

// ── Test 9: SBA applies to prototype toughness ────────────────────────────────

/// CR 704.5f — A creature with toughness 0 is put into the graveyard as a state-based action.
/// A prototyped 3/2 receiving -0/-2 (net 3/0) should die to SBAs.
/// We test by placing a pre-damaged prototyped creature and triggering SBAs.
#[test]
fn test_prototype_sba_toughness_check() {
    use mtg_engine::check_and_apply_sbas;

    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![blitz_automaton_def()]);

    // Place a Blitz Automaton on the battlefield as if it was cast prototyped (3/2, red).
    let mut spec = ObjectSpec::card(p1, "Blitz Automaton")
        .in_zone(ZoneId::Battlefield)
        .with_card_id(CardId("blitz-automaton".to_string()))
        .with_types(vec![CardType::Artifact, CardType::Creature])
        .with_keyword(KeywordAbility::Prototype)
        .with_mana_cost(ManaCost {
            generic: 2,
            red: 1,
            ..Default::default()
        });
    // Set to prototype P/T and mark is_prototyped.
    spec.power = Some(3);
    spec.toughness = Some(2);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(spec)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Mark the battlefield object as prototyped and set toughness to 0 (simulate -2/-2 damage).
    let bf_id = find_in_zone(&state, "Blitz Automaton", ZoneId::Battlefield)
        .expect("should be on battlefield");
    {
        let obj = state.objects.get_mut(&bf_id).unwrap();
        obj.is_prototyped = true;
        // Set toughness to 0 — simulates a -2 effect on a 2-toughness prototyped creature.
        obj.characteristics.toughness = Some(0);
    }

    // Run SBAs — CR 704.5f: creature with toughness 0 is put in graveyard.
    let _events = check_and_apply_sbas(&mut state);

    // Blitz Automaton should now be in the graveyard.
    assert!(
        !on_battlefield(&state, "Blitz Automaton"),
        "CR 704.5f: prototyped creature with toughness 0 should leave the battlefield"
    );
    assert!(
        in_graveyard(&state, "Blitz Automaton", p1),
        "CR 704.5f: prototyped creature with toughness 0 should go to graveyard"
    );
}

// ── Test 10: on_stack spell has prototype characteristics ─────────────────────

/// CR 718.3b — While on the stack as a prototyped spell, the spell has the
/// prototype mana cost, power, toughness, and color (not the printed ones).
#[test]
fn test_prototype_stack_characteristics() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![blitz_automaton_def()]);
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(automaton_in_hand(p1))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

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

    let card_id = find_object(&state, "Blitz Automaton");
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
            alt_cost: None,
            prototype: true,
            modes_chosen: vec![],
            x_value: 0,
            face_down_kind: None,
            additional_costs: vec![],
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    )
    .unwrap();

    // The spell is now on the stack. Verify the source object's characteristics.
    assert_eq!(
        state.stack_objects.len(),
        1,
        "precondition: one spell on the stack"
    );

    // The stack_object flag.
    assert!(
        state.stack_objects[0].was_prototyped,
        "CR 718.3b: was_prototyped should be true on the StackObject"
    );

    // The source card on the stack should have prototype characteristics.
    // source_object is inside StackObjectKind::Spell.
    let source_id = match &state.stack_objects[0].kind {
        mtg_engine::StackObjectKind::Spell { source_object } => *source_object,
        other => panic!("expected Spell on stack, got {:?}", other),
    };
    if let Some(source_obj) = state.objects.get(&source_id) {
        // Mana cost should be prototype cost {2}{R}.
        if let Some(mc) = &source_obj.characteristics.mana_cost {
            assert_eq!(
                mc.mana_value(),
                3,
                "CR 718.3b: spell on stack should have prototype MV 3"
            );
            assert_eq!(mc.red, 1, "CR 718.3b: spell on stack should have 1 red pip");
        }
        // Colors should be red.
        assert!(
            source_obj.characteristics.colors.contains(&Color::Red),
            "CR 718.3b: spell on stack should be red"
        );
        // P/T should be prototype values.
        assert_eq!(
            source_obj.characteristics.power,
            Some(3),
            "CR 718.3b: spell on stack should have prototype power 3"
        );
        assert_eq!(
            source_obj.characteristics.toughness,
            Some(2),
            "CR 718.3b: spell on stack should have prototype toughness 2"
        );
    }
}
