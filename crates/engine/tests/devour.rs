//! Devour keyword ability tests (CR 702.82).
//!
//! Devour is a static ability that functions as an ETB replacement effect.
//! "As this object enters, you may sacrifice any number of creatures. This
//! permanent enters with N +1/+1 counters on it for each creature sacrificed
//! this way." (CR 702.82a)
//!
//! Key rules verified:
//! - ETB: creature enters with N * sacrifice_count +1/+1 counters (CR 702.82a).
//! - Zero sacrifices → 0 counters (optional sacrifice, CR 702.82a).
//! - N multiplier: Devour 3 × 2 creatures = 6 counters (CR 702.82a).
//! - Sacrificed creatures go to graveyard (CR 702.82a).
//! - `creatures_devoured` field tracks count (CR 702.82b).
//! - Multiple Devour instances work separately (CR 702.82c).
//! - Only controller's own creatures can be sacrificed (CR 702.82a + multiplayer).
//! - Cannot sacrifice the creature being cast.

use mtg_engine::{
    process_command, AbilityDefinition, CardDefinition, CardId, CardRegistry, CardType, Command,
    CounterType, GameEvent, GameStateBuilder, KeywordAbility, ManaCost, ObjectId, ObjectSpec,
    PlayerId, Step, TypeLine, ZoneId,
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

fn find_object_in_graveyard(
    state: &mtg_engine::GameState,
    player: PlayerId,
    name: &str,
) -> Option<ObjectId> {
    state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name && obj.zone == ZoneId::Graveyard(player))
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

/// Cast the named Devour creature from hand with the given sacrifice list.
fn cast_devour_creature(
    state: mtg_engine::GameState,
    caster: PlayerId,
    card_id: ObjectId,
    generic_cost: u32,
    devour_sacrifices: Vec<ObjectId>,
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
            devour_sacrifices,
            modes_chosen: vec![],
            fuse: false,
            x_value: 0,
        },
    )
    .unwrap_or_else(|e| panic!("CastSpell failed: {:?}", e));
    state
}

// ── Card Definitions ──────────────────────────────────────────────────────────

/// Devour 1 creature (like Mycoloth). 4/4 Creature - Fungus.
fn devour_1_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("devour-1-test".to_string()),
        name: "Devour One".to_string(),
        mana_cost: Some(ManaCost {
            generic: 3,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Devour 1".to_string(),
        power: Some(4),
        toughness: Some(4),
        abilities: vec![AbilityDefinition::Keyword(KeywordAbility::Devour(1))],
        ..Default::default()
    }
}

/// Devour 3 creature (like Thunder-Thrash Elder). 1/1 Creature - Lizard Warrior.
fn devour_3_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("devour-3-test".to_string()),
        name: "Devour Three".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Devour 3".to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![AbilityDefinition::Keyword(KeywordAbility::Devour(3))],
        ..Default::default()
    }
}

/// Creature with both Devour 1 and Devour 2 (artificial test case). 1/1 Creature.
fn devour_dual_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("devour-dual-test".to_string()),
        name: "Devour Dual Test".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Devour 1, Devour 2".to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Devour(1)),
            AbilityDefinition::Keyword(KeywordAbility::Devour(2)),
        ],
        ..Default::default()
    }
}

/// Simple 1/1 creature for sacrifice fodder.
fn fodder_def(name: &str, id: &str) -> CardDefinition {
    CardDefinition {
        card_id: CardId(id.to_string()),
        name: name.to_string(),
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
    }
}

// ── Test 1: Devour basic — sacrifice 1 creature ────────────────────────────────

#[test]
/// CR 702.82a — "As this object enters, you may sacrifice any number of creatures.
/// This permanent enters with N +1/+1 counters on it for each creature sacrificed."
/// Devour 1 creature enters, sacrifice 1 creature → 1 counter.
fn test_devour_basic_one_sacrifice() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![
        devour_1_def(),
        fodder_def("Fodder Alpha", "fodder-alpha"),
    ]);

    let devour_spec = ObjectSpec::card(p1, "Devour One")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("devour-1-test".to_string()))
        .with_types(vec![CardType::Creature])
        .with_keyword(KeywordAbility::Devour(1))
        .with_mana_cost(ManaCost {
            generic: 3,
            ..Default::default()
        });

    let fodder_spec = ObjectSpec::card(p1, "Fodder Alpha")
        .in_zone(ZoneId::Battlefield)
        .with_card_id(CardId("fodder-alpha".to_string()))
        .with_types(vec![CardType::Creature]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(devour_spec)
        .object(fodder_spec)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state.turn.priority_holder = Some(p1);

    let devour_id = find_object(&state, "Devour One");
    let fodder_id = find_object(&state, "Fodder Alpha");

    // Cast with Fodder Alpha as the sacrifice.
    let state = cast_devour_creature(state, p1, devour_id, 3, vec![fodder_id]);

    // Resolve: both players pass priority.
    let (state, resolve_events) = pass_all(state, &[p1, p2]);

    // Devour One should be on the battlefield.
    let bf_id = find_object_on_battlefield(&state, "Devour One")
        .expect("CR 702.82a: Devour creature should be on the battlefield");

    // Verify: exactly 1 +1/+1 counter (Devour 1 × 1 creature).
    let counter_count = state.objects[&bf_id]
        .counters
        .get(&CounterType::PlusOnePlusOne)
        .copied()
        .unwrap_or(0);
    assert_eq!(
        counter_count, 1,
        "CR 702.82a: Devour 1 × 1 sacrifice should yield 1 counter"
    );

    // Verify CounterAdded event was emitted.
    let counter_event = resolve_events.iter().any(|ev| {
        matches!(
            ev,
            GameEvent::CounterAdded {
                counter: CounterType::PlusOnePlusOne,
                count: 1,
                ..
            }
        )
    });
    assert!(
        counter_event,
        "CR 702.82a: CounterAdded event should be emitted"
    );

    // Verify: the sacrificed creature is in the graveyard.
    let in_graveyard = find_object_in_graveyard(&state, p1, "Fodder Alpha");
    assert!(
        in_graveyard.is_some(),
        "CR 702.82a: Sacrificed creature should be in the graveyard"
    );
}

// ── Test 2: Devour — sacrifice multiple creatures ──────────────────────────────

#[test]
/// CR 702.82a — Devour 1 creature enters, sacrifice 2 creatures → 2 counters.
fn test_devour_multiple_sacrifices() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![
        devour_1_def(),
        fodder_def("Fodder Alpha", "fodder-alpha"),
        fodder_def("Fodder Beta", "fodder-beta"),
    ]);

    let devour_spec = ObjectSpec::card(p1, "Devour One")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("devour-1-test".to_string()))
        .with_types(vec![CardType::Creature])
        .with_keyword(KeywordAbility::Devour(1))
        .with_mana_cost(ManaCost {
            generic: 3,
            ..Default::default()
        });

    let fodder_a = ObjectSpec::card(p1, "Fodder Alpha")
        .in_zone(ZoneId::Battlefield)
        .with_card_id(CardId("fodder-alpha".to_string()))
        .with_types(vec![CardType::Creature]);

    let fodder_b = ObjectSpec::card(p1, "Fodder Beta")
        .in_zone(ZoneId::Battlefield)
        .with_card_id(CardId("fodder-beta".to_string()))
        .with_types(vec![CardType::Creature]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(devour_spec)
        .object(fodder_a)
        .object(fodder_b)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state.turn.priority_holder = Some(p1);

    let devour_id = find_object(&state, "Devour One");
    let fodder_a_id = find_object(&state, "Fodder Alpha");
    let fodder_b_id = find_object(&state, "Fodder Beta");

    let state = cast_devour_creature(state, p1, devour_id, 3, vec![fodder_a_id, fodder_b_id]);
    let (state, _) = pass_all(state, &[p1, p2]);

    let bf_id = find_object_on_battlefield(&state, "Devour One")
        .expect("CR 702.82a: Devour creature should be on the battlefield");

    let counter_count = state.objects[&bf_id]
        .counters
        .get(&CounterType::PlusOnePlusOne)
        .copied()
        .unwrap_or(0);
    assert_eq!(
        counter_count, 2,
        "CR 702.82a: Devour 1 × 2 sacrifices should yield 2 counters"
    );

    // Both sacrificed creatures should be in graveyard.
    assert!(
        find_object_in_graveyard(&state, p1, "Fodder Alpha").is_some(),
        "CR 702.82a: Fodder Alpha should be in graveyard after sacrifice"
    );
    assert!(
        find_object_in_graveyard(&state, p1, "Fodder Beta").is_some(),
        "CR 702.82a: Fodder Beta should be in graveyard after sacrifice"
    );
}

// ── Test 3: Devour N multiplier ────────────────────────────────────────────────

#[test]
/// CR 702.82a — Devour 3 creature enters, sacrifice 2 creatures → 6 counters (3 × 2).
fn test_devour_n_multiplier() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![
        devour_3_def(),
        fodder_def("Fodder Alpha", "fodder-alpha"),
        fodder_def("Fodder Beta", "fodder-beta"),
    ]);

    let devour_spec = ObjectSpec::card(p1, "Devour Three")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("devour-3-test".to_string()))
        .with_types(vec![CardType::Creature])
        .with_keyword(KeywordAbility::Devour(3))
        .with_mana_cost(ManaCost {
            generic: 2,
            ..Default::default()
        });

    let fodder_a = ObjectSpec::card(p1, "Fodder Alpha")
        .in_zone(ZoneId::Battlefield)
        .with_card_id(CardId("fodder-alpha".to_string()))
        .with_types(vec![CardType::Creature]);

    let fodder_b = ObjectSpec::card(p1, "Fodder Beta")
        .in_zone(ZoneId::Battlefield)
        .with_card_id(CardId("fodder-beta".to_string()))
        .with_types(vec![CardType::Creature]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(devour_spec)
        .object(fodder_a)
        .object(fodder_b)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state.turn.priority_holder = Some(p1);

    let devour_id = find_object(&state, "Devour Three");
    let fodder_a_id = find_object(&state, "Fodder Alpha");
    let fodder_b_id = find_object(&state, "Fodder Beta");

    let state = cast_devour_creature(state, p1, devour_id, 2, vec![fodder_a_id, fodder_b_id]);
    let (state, _) = pass_all(state, &[p1, p2]);

    let bf_id = find_object_on_battlefield(&state, "Devour Three")
        .expect("CR 702.82a: Devour creature should be on the battlefield");

    let counter_count = state.objects[&bf_id]
        .counters
        .get(&CounterType::PlusOnePlusOne)
        .copied()
        .unwrap_or(0);
    assert_eq!(
        counter_count, 6,
        "CR 702.82a: Devour 3 × 2 sacrifices should yield 6 counters"
    );
}

// ── Test 4: Devour — zero sacrifice (optional) ─────────────────────────────────

#[test]
/// CR 702.82a — "you may sacrifice any number of creatures" — zero is valid.
/// Creature enters with 0 counters when player passes empty devour_sacrifices.
fn test_devour_zero_sacrifice() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![
        devour_1_def(),
        fodder_def("Fodder Alpha", "fodder-alpha"),
    ]);

    let devour_spec = ObjectSpec::card(p1, "Devour One")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("devour-1-test".to_string()))
        .with_types(vec![CardType::Creature])
        .with_keyword(KeywordAbility::Devour(1))
        .with_mana_cost(ManaCost {
            generic: 3,
            ..Default::default()
        });

    let fodder_spec = ObjectSpec::card(p1, "Fodder Alpha")
        .in_zone(ZoneId::Battlefield)
        .with_card_id(CardId("fodder-alpha".to_string()))
        .with_types(vec![CardType::Creature]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(devour_spec)
        .object(fodder_spec)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state.turn.priority_holder = Some(p1);

    let devour_id = find_object(&state, "Devour One");

    // Cast with empty sacrifice list (player opts out).
    let state = cast_devour_creature(state, p1, devour_id, 3, vec![]);
    let (state, _) = pass_all(state, &[p1, p2]);

    let bf_id = find_object_on_battlefield(&state, "Devour One")
        .expect("CR 702.82a: Devour creature should still enter the battlefield");

    // No counters should be placed.
    let counter_count = state.objects[&bf_id]
        .counters
        .get(&CounterType::PlusOnePlusOne)
        .copied()
        .unwrap_or(0);
    assert_eq!(counter_count, 0, "CR 702.82a: Zero sacrifices → 0 counters");

    // The fodder creature should still be on the battlefield.
    assert!(
        find_object_on_battlefield(&state, "Fodder Alpha").is_some(),
        "CR 702.82a: Creature not sacrificed should still be on the battlefield"
    );
}

// ── Test 5: Devour — no eligible creatures on battlefield ─────────────────────

#[test]
/// CR 702.82a — If no other creatures are on the battlefield, the devour permanent
/// enters normally with 0 counters. Sacrifice list is empty.
fn test_devour_no_eligible_creatures() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![devour_1_def()]);

    let devour_spec = ObjectSpec::card(p1, "Devour One")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("devour-1-test".to_string()))
        .with_types(vec![CardType::Creature])
        .with_keyword(KeywordAbility::Devour(1))
        .with_mana_cost(ManaCost {
            generic: 3,
            ..Default::default()
        });

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(devour_spec)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state.turn.priority_holder = Some(p1);

    let devour_id = find_object(&state, "Devour One");

    let state = cast_devour_creature(state, p1, devour_id, 3, vec![]);
    let (state, _) = pass_all(state, &[p1, p2]);

    let bf_id = find_object_on_battlefield(&state, "Devour One")
        .expect("CR 702.82a: Devour creature should enter normally");

    let counter_count = state.objects[&bf_id]
        .counters
        .get(&CounterType::PlusOnePlusOne)
        .copied()
        .unwrap_or(0);
    assert_eq!(
        counter_count, 0,
        "CR 702.82a: No eligible creatures → 0 counters"
    );
}

// ── Test 6: Devour — only controller's creatures can be sacrificed ─────────────

#[test]
/// CR 702.82a (multiplayer): Devour only allows the controller to sacrifice creatures
/// they control. Attempting to sacrifice an opponent's creature should be rejected.
fn test_devour_only_own_creatures() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![
        devour_1_def(),
        fodder_def("Opponent Fodder", "opp-fodder"),
    ]);

    let devour_spec = ObjectSpec::card(p1, "Devour One")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("devour-1-test".to_string()))
        .with_types(vec![CardType::Creature])
        .with_keyword(KeywordAbility::Devour(1))
        .with_mana_cost(ManaCost {
            generic: 3,
            ..Default::default()
        });

    // P2 controls this creature — P1 should not be able to sacrifice it.
    let opp_fodder_spec = ObjectSpec::card(p2, "Opponent Fodder")
        .in_zone(ZoneId::Battlefield)
        .with_card_id(CardId("opp-fodder".to_string()))
        .with_types(vec![CardType::Creature]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(devour_spec)
        .object(opp_fodder_spec)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state.turn.priority_holder = Some(p1);

    let devour_id = find_object(&state, "Devour One");
    let opp_fodder_id = find_object(&state, "Opponent Fodder");

    // Attempt to sacrifice P2's creature — should be rejected at cast time.
    let mut mana_state = state.clone();
    mana_state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(mtg_engine::ManaColor::Colorless, 3);
    mana_state.turn.priority_holder = Some(p1);

    let result = process_command(
        mana_state,
        Command::CastSpell {
            player: p1,
            card: devour_id,
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
            devour_sacrifices: vec![opp_fodder_id],
            modes_chosen: vec![],
            fuse: false,
            x_value: 0,
        },
    );

    assert!(
        result.is_err(),
        "CR 702.82a: Sacrificing an opponent's creature should be rejected"
    );
}

// ── Test 7: Devour — cannot sacrifice the creature being cast ──────────────────

#[test]
/// CR 702.82a: The creature being cast cannot target itself as a devour sacrifice
/// (it is on the stack, not the battlefield, when devour sacrifices are validated).
fn test_devour_cannot_sacrifice_self() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![devour_1_def()]);

    let devour_spec = ObjectSpec::card(p1, "Devour One")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("devour-1-test".to_string()))
        .with_types(vec![CardType::Creature])
        .with_keyword(KeywordAbility::Devour(1))
        .with_mana_cost(ManaCost {
            generic: 3,
            ..Default::default()
        });

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(devour_spec)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state.turn.priority_holder = Some(p1);

    let devour_id = find_object(&state, "Devour One");

    // Attempt to use the devour card's own ID as the sacrifice target.
    // At cast time, the card is still in hand (pre-move), but after cast it moves
    // to the stack. The validation in casting.rs checks battlefield membership,
    // so a hand card cannot be a sacrifice target anyway.
    let mut mana_state = state.clone();
    mana_state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(mtg_engine::ManaColor::Colorless, 3);
    mana_state.turn.priority_holder = Some(p1);

    let result = process_command(
        mana_state,
        Command::CastSpell {
            player: p1,
            card: devour_id,
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
            devour_sacrifices: vec![devour_id],
            modes_chosen: vec![],
            fuse: false,
            x_value: 0,
        },
    );

    assert!(
        result.is_err(),
        "CR 702.82a: Cannot sacrifice the creature being cast (it is not on the battlefield)"
    );
}

// ── Test 8: Devour — sacrificed creatures go to graveyard ─────────────────────

#[test]
/// CR 702.82a: The sacrificed creatures should be in their owner's graveyard
/// after the Devour ETB replacement resolves. CreatureDied events should fire.
fn test_devour_creatures_go_to_graveyard() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![
        devour_1_def(),
        fodder_def("Fodder Alpha", "fodder-alpha"),
        fodder_def("Fodder Beta", "fodder-beta"),
        fodder_def("Fodder Gamma", "fodder-gamma"),
    ]);

    let devour_spec = ObjectSpec::card(p1, "Devour One")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("devour-1-test".to_string()))
        .with_types(vec![CardType::Creature])
        .with_keyword(KeywordAbility::Devour(1))
        .with_mana_cost(ManaCost {
            generic: 3,
            ..Default::default()
        });

    let fodder_a = ObjectSpec::card(p1, "Fodder Alpha")
        .in_zone(ZoneId::Battlefield)
        .with_card_id(CardId("fodder-alpha".to_string()))
        .with_types(vec![CardType::Creature]);

    let fodder_b = ObjectSpec::card(p1, "Fodder Beta")
        .in_zone(ZoneId::Battlefield)
        .with_card_id(CardId("fodder-beta".to_string()))
        .with_types(vec![CardType::Creature]);

    let fodder_c = ObjectSpec::card(p1, "Fodder Gamma")
        .in_zone(ZoneId::Battlefield)
        .with_card_id(CardId("fodder-gamma".to_string()))
        .with_types(vec![CardType::Creature]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(devour_spec)
        .object(fodder_a)
        .object(fodder_b)
        .object(fodder_c)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state.turn.priority_holder = Some(p1);

    let devour_id = find_object(&state, "Devour One");
    let fodder_a_id = find_object(&state, "Fodder Alpha");
    let fodder_b_id = find_object(&state, "Fodder Beta");
    // Fodder Gamma is NOT sacrificed.

    let state = cast_devour_creature(state, p1, devour_id, 3, vec![fodder_a_id, fodder_b_id]);
    let (state, resolve_events) = pass_all(state, &[p1, p2]);

    // Sacrificed creatures should be in graveyard.
    assert!(
        find_object_in_graveyard(&state, p1, "Fodder Alpha").is_some(),
        "CR 702.82a: Fodder Alpha should be in graveyard after devour"
    );
    assert!(
        find_object_in_graveyard(&state, p1, "Fodder Beta").is_some(),
        "CR 702.82a: Fodder Beta should be in graveyard after devour"
    );

    // Fodder Gamma should still be on the battlefield.
    assert!(
        find_object_on_battlefield(&state, "Fodder Gamma").is_some(),
        "CR 702.82a: Unselected creature should remain on the battlefield"
    );

    // CreatureDied events should have been emitted for the sacrificed creatures.
    let died_count = resolve_events
        .iter()
        .filter(|ev| matches!(ev, GameEvent::CreatureDied { .. }))
        .count();
    assert!(
        died_count >= 2,
        "CR 702.82a: CreatureDied events should fire for each sacrificed creature (got {})",
        died_count
    );
}

// ── Test 9: Devour — multiple instances (CR 702.82c) ──────────────────────────

#[test]
/// CR 702.82c (by analogy): Creature with Devour 1 + Devour 2, sacrifice 2 creatures.
/// Total counters = (1 × 2) + (2 × 2) = 6 counters (each instance processed separately).
fn test_devour_multiple_instances() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![
        devour_dual_def(),
        fodder_def("Fodder Alpha", "fodder-alpha"),
        fodder_def("Fodder Beta", "fodder-beta"),
    ]);

    let devour_spec = ObjectSpec::card(p1, "Devour Dual Test")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("devour-dual-test".to_string()))
        .with_types(vec![CardType::Creature])
        .with_keyword(KeywordAbility::Devour(1))
        .with_keyword(KeywordAbility::Devour(2))
        .with_mana_cost(ManaCost {
            generic: 2,
            ..Default::default()
        });

    let fodder_a = ObjectSpec::card(p1, "Fodder Alpha")
        .in_zone(ZoneId::Battlefield)
        .with_card_id(CardId("fodder-alpha".to_string()))
        .with_types(vec![CardType::Creature]);

    let fodder_b = ObjectSpec::card(p1, "Fodder Beta")
        .in_zone(ZoneId::Battlefield)
        .with_card_id(CardId("fodder-beta".to_string()))
        .with_types(vec![CardType::Creature]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(devour_spec)
        .object(fodder_a)
        .object(fodder_b)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state.turn.priority_holder = Some(p1);

    let devour_id = find_object(&state, "Devour Dual Test");
    let fodder_a_id = find_object(&state, "Fodder Alpha");
    let fodder_b_id = find_object(&state, "Fodder Beta");

    let state = cast_devour_creature(state, p1, devour_id, 2, vec![fodder_a_id, fodder_b_id]);
    let (state, _) = pass_all(state, &[p1, p2]);

    let bf_id = find_object_on_battlefield(&state, "Devour Dual Test")
        .expect("CR 702.82c: Devour creature should be on the battlefield");

    let counter_count = state.objects[&bf_id]
        .counters
        .get(&CounterType::PlusOnePlusOne)
        .copied()
        .unwrap_or(0);
    assert_eq!(
        counter_count, 6,
        "CR 702.82c: Devour 1+2 × 2 sacrifices = (1×2)+(2×2) = 6 counters"
    );
}

// ── Test 10: Devour — creatures_devoured tracking (CR 702.82b) ─────────────────

#[test]
/// CR 702.82b: The `creatures_devoured` field on the permanent tracks how many
/// creatures were sacrificed. Used by abilities that say "it devoured."
fn test_devour_creatures_devoured_tracking() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![
        devour_1_def(),
        fodder_def("Fodder Alpha", "fodder-alpha"),
        fodder_def("Fodder Beta", "fodder-beta"),
        fodder_def("Fodder Gamma", "fodder-gamma"),
    ]);

    let devour_spec = ObjectSpec::card(p1, "Devour One")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("devour-1-test".to_string()))
        .with_types(vec![CardType::Creature])
        .with_keyword(KeywordAbility::Devour(1))
        .with_mana_cost(ManaCost {
            generic: 3,
            ..Default::default()
        });

    let fodder_a = ObjectSpec::card(p1, "Fodder Alpha")
        .in_zone(ZoneId::Battlefield)
        .with_card_id(CardId("fodder-alpha".to_string()))
        .with_types(vec![CardType::Creature]);

    let fodder_b = ObjectSpec::card(p1, "Fodder Beta")
        .in_zone(ZoneId::Battlefield)
        .with_card_id(CardId("fodder-beta".to_string()))
        .with_types(vec![CardType::Creature]);

    let fodder_c = ObjectSpec::card(p1, "Fodder Gamma")
        .in_zone(ZoneId::Battlefield)
        .with_card_id(CardId("fodder-gamma".to_string()))
        .with_types(vec![CardType::Creature]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(devour_spec)
        .object(fodder_a)
        .object(fodder_b)
        .object(fodder_c)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state.turn.priority_holder = Some(p1);

    let devour_id = find_object(&state, "Devour One");
    let fodder_a_id = find_object(&state, "Fodder Alpha");
    let fodder_b_id = find_object(&state, "Fodder Beta");
    let fodder_c_id = find_object(&state, "Fodder Gamma");

    // Sacrifice all 3 fodder creatures.
    let state = cast_devour_creature(
        state,
        p1,
        devour_id,
        3,
        vec![fodder_a_id, fodder_b_id, fodder_c_id],
    );
    let (state, _) = pass_all(state, &[p1, p2]);

    let bf_id = find_object_on_battlefield(&state, "Devour One")
        .expect("CR 702.82b: Devour creature should be on the battlefield");

    // Verify `creatures_devoured` is set to 3.
    let devoured_count = state.objects[&bf_id].creatures_devoured;
    assert_eq!(
        devoured_count, 3,
        "CR 702.82b: creatures_devoured should track the number of sacrificed creatures"
    );

    // Also verify counter count matches (3 devoured × Devour 1 = 3 counters).
    let counter_count = state.objects[&bf_id]
        .counters
        .get(&CounterType::PlusOnePlusOne)
        .copied()
        .unwrap_or(0);
    assert_eq!(
        counter_count, 3,
        "CR 702.82a: 3 creatures sacrificed × Devour 1 = 3 counters"
    );
}
