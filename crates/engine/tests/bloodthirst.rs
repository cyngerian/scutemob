//! Bloodthirst keyword ability tests (CR 702.54).
//!
//! Bloodthirst is a static ability that functions as an ETB replacement effect.
//! "If an opponent was dealt damage this turn, this permanent enters with N +1/+1
//! counters on it." (CR 702.54a)
//!
//! Key rules verified:
//! - ETB with N counters when any opponent was dealt damage this turn (CR 702.54a).
//! - No counters when no opponent was dealt damage this turn (CR 702.54a).
//! - N value is fixed (not dependent on how much damage was dealt) (CR 702.54a).
//! - Multiple instances each apply separately (CR 702.54c).
//! - Multiplayer: any single opponent being damaged is sufficient (CR 702.54a).
//! - Self-damage does not satisfy Bloodthirst (CR 702.54a: "an opponent").
//! - Eliminated opponents are not opponents (CR 800.4a / CR 102.3).
//! - CounterAdded event is emitted when counters are placed.

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
            collect_evidence_cards: vec![],
            squad_count: 0,
            offspring_paid: false,
        },
    )
    .unwrap_or_else(|e| panic!("CastSpell failed: {:?}", e));
    state
}

// ── Card Definitions ──────────────────────────────────────────────────────────

/// Bloodthirst 2 creature. 1/1 Creature - Berserker, cost {1}{R}.
fn bloodthirst_2_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("bloodthirst-2-test".to_string()),
        name: "Bloodthirst Test Creature".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Bloodthirst 2".to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![AbilityDefinition::Keyword(KeywordAbility::Bloodthirst(2))],
        ..Default::default()
    }
}

/// Bloodthirst 3 creature. 2/2 Creature, cost {3}.
fn bloodthirst_3_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("bloodthirst-3-test".to_string()),
        name: "Bloodthirst Three Test".to_string(),
        mana_cost: Some(ManaCost {
            generic: 3,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Bloodthirst 3".to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![AbilityDefinition::Keyword(KeywordAbility::Bloodthirst(3))],
        ..Default::default()
    }
}

/// Creature with Bloodthirst 1 and Bloodthirst 2 (CR 702.54c test). 1/1 Creature, cost {2}.
fn bloodthirst_dual_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("bloodthirst-dual-test".to_string()),
        name: "Bloodthirst Dual Test".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Bloodthirst 1, Bloodthirst 2".to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Bloodthirst(1)),
            AbilityDefinition::Keyword(KeywordAbility::Bloodthirst(2)),
        ],
        ..Default::default()
    }
}

// ── Test 1: Basic Bloodthirst 2 with opponent damaged ─────────────────────────

#[test]
/// CR 702.54a — "If an opponent was dealt damage this turn, this permanent enters
/// with N +1/+1 counters on it." Bloodthirst 2 → 2 counters when opponent was damaged.
fn test_bloodthirst_basic_opponent_damaged() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![bloodthirst_2_def()]);

    let bt_spec = ObjectSpec::card(p1, "Bloodthirst Test Creature")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("bloodthirst-2-test".to_string()))
        .with_types(vec![CardType::Creature])
        .with_keyword(KeywordAbility::Bloodthirst(2))
        .with_mana_cost(ManaCost {
            generic: 2,
            ..Default::default()
        });

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(bt_spec)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Set up: opponent p2 was dealt damage this turn.
    state
        .players
        .get_mut(&p2)
        .unwrap()
        .damage_received_this_turn = 3;
    state.turn.priority_holder = Some(p1);

    let bt_id = find_object(&state, "Bloodthirst Test Creature");
    let state = cast_creature(state, p1, bt_id, 2);
    let (state, _) = pass_all(state, &[p1, p2]);

    let bf_id = find_object_on_battlefield(&state, "Bloodthirst Test Creature")
        .expect("CR 702.54a: Bloodthirst creature should be on the battlefield");

    let counter_count = state.objects[&bf_id]
        .counters
        .get(&CounterType::PlusOnePlusOne)
        .copied()
        .unwrap_or(0);
    assert_eq!(
        counter_count, 2,
        "CR 702.54a: Bloodthirst 2 should place 2 +1/+1 counters when an opponent was damaged"
    );
}

// ── Test 2: No damage dealt — no counters ─────────────────────────────────────

#[test]
/// CR 702.54a — Bloodthirst creature enters when no opponent was dealt damage this turn.
/// Expect 0 +1/+1 counters; creature enters at its printed P/T.
fn test_bloodthirst_no_damage_dealt() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![bloodthirst_2_def()]);

    let bt_spec = ObjectSpec::card(p1, "Bloodthirst Test Creature")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("bloodthirst-2-test".to_string()))
        .with_types(vec![CardType::Creature])
        .with_keyword(KeywordAbility::Bloodthirst(2))
        .with_mana_cost(ManaCost {
            generic: 2,
            ..Default::default()
        });

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(bt_spec)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // No damage dealt to anyone: damage_received_this_turn defaults to 0.
    state.turn.priority_holder = Some(p1);

    let bt_id = find_object(&state, "Bloodthirst Test Creature");
    let state = cast_creature(state, p1, bt_id, 2);
    let (state, _) = pass_all(state, &[p1, p2]);

    let bf_id = find_object_on_battlefield(&state, "Bloodthirst Test Creature")
        .expect("CR 702.54a: Bloodthirst creature should be on the battlefield");

    let counter_count = state.objects[&bf_id]
        .counters
        .get(&CounterType::PlusOnePlusOne)
        .copied()
        .unwrap_or(0);
    assert_eq!(
        counter_count, 0,
        "CR 702.54a: Bloodthirst creature should have 0 counters when no opponent was damaged"
    );
}

// ── Test 3: Bloodthirst N multiplier ─────────────────────────────────────────

#[test]
/// CR 702.54a — Bloodthirst 3 creature enters after opponent took damage → exactly 3 counters.
/// The N value is fixed, not proportional to how much damage was dealt.
fn test_bloodthirst_n_multiplier() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![bloodthirst_3_def()]);

    let bt_spec = ObjectSpec::card(p1, "Bloodthirst Three Test")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("bloodthirst-3-test".to_string()))
        .with_types(vec![CardType::Creature])
        .with_keyword(KeywordAbility::Bloodthirst(3))
        .with_mana_cost(ManaCost {
            generic: 3,
            ..Default::default()
        });

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(bt_spec)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Opponent took only 1 damage, but Bloodthirst 3 still gives 3 counters.
    state
        .players
        .get_mut(&p2)
        .unwrap()
        .damage_received_this_turn = 1;
    state.turn.priority_holder = Some(p1);

    let bt_id = find_object(&state, "Bloodthirst Three Test");
    let state = cast_creature(state, p1, bt_id, 3);
    let (state, _) = pass_all(state, &[p1, p2]);

    let bf_id = find_object_on_battlefield(&state, "Bloodthirst Three Test")
        .expect("Bloodthirst 3 creature should be on battlefield");

    let counter_count = state.objects[&bf_id]
        .counters
        .get(&CounterType::PlusOnePlusOne)
        .copied()
        .unwrap_or(0);
    assert_eq!(
        counter_count, 3,
        "CR 702.54a: Bloodthirst 3 should place exactly 3 counters (N is fixed, not proportional)"
    );
}

// ── Test 4: Multiple Bloodthirst instances work separately ────────────────────

#[test]
/// CR 702.54c — "If a creature has multiple instances of bloodthirst, each applies separately."
/// Creature with Bloodthirst 1 + Bloodthirst 2 → 1 + 2 = 3 counters total.
fn test_bloodthirst_multiple_instances() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![bloodthirst_dual_def()]);

    let bt_spec = ObjectSpec::card(p1, "Bloodthirst Dual Test")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("bloodthirst-dual-test".to_string()))
        .with_types(vec![CardType::Creature])
        .with_keyword(KeywordAbility::Bloodthirst(1))
        .with_keyword(KeywordAbility::Bloodthirst(2))
        .with_mana_cost(ManaCost {
            generic: 2,
            ..Default::default()
        });

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(bt_spec)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p2)
        .unwrap()
        .damage_received_this_turn = 5;
    state.turn.priority_holder = Some(p1);

    let bt_id = find_object(&state, "Bloodthirst Dual Test");
    let state = cast_creature(state, p1, bt_id, 2);
    let (state, _) = pass_all(state, &[p1, p2]);

    let bf_id = find_object_on_battlefield(&state, "Bloodthirst Dual Test")
        .expect("Bloodthirst dual creature should be on battlefield");

    let counter_count = state.objects[&bf_id]
        .counters
        .get(&CounterType::PlusOnePlusOne)
        .copied()
        .unwrap_or(0);
    assert_eq!(
        counter_count, 3,
        "CR 702.54c: Bloodthirst 1 + Bloodthirst 2 should produce 3 counters (1 + 2)"
    );
}

// ── Test 5: Multiplayer — multiple opponents damaged ──────────────────────────

#[test]
/// CR 702.54a — Bloodthirst checks "an opponent" (any single one is sufficient).
/// In multiplayer, if multiple opponents were damaged, the condition is still binary.
/// Bloodthirst N still places exactly N counters (not per-opponent).
fn test_bloodthirst_multiple_opponents_damaged() {
    let p1 = p(1);
    let p2 = p(2);
    let p3 = p(3);

    let registry = CardRegistry::new(vec![bloodthirst_2_def()]);

    let bt_spec = ObjectSpec::card(p1, "Bloodthirst Test Creature")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("bloodthirst-2-test".to_string()))
        .with_types(vec![CardType::Creature])
        .with_keyword(KeywordAbility::Bloodthirst(2))
        .with_mana_cost(ManaCost {
            generic: 2,
            ..Default::default()
        });

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .with_registry(registry)
        .object(bt_spec)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Both opponents took damage.
    state
        .players
        .get_mut(&p2)
        .unwrap()
        .damage_received_this_turn = 2;
    state
        .players
        .get_mut(&p3)
        .unwrap()
        .damage_received_this_turn = 3;
    state.turn.priority_holder = Some(p1);

    let bt_id = find_object(&state, "Bloodthirst Test Creature");
    let state = cast_creature(state, p1, bt_id, 2);
    let (state, _) = pass_all(state, &[p1, p2, p3]);

    let bf_id = find_object_on_battlefield(&state, "Bloodthirst Test Creature")
        .expect("Bloodthirst creature should be on battlefield");

    let counter_count = state.objects[&bf_id]
        .counters
        .get(&CounterType::PlusOnePlusOne)
        .copied()
        .unwrap_or(0);
    assert_eq!(
        counter_count, 2,
        "CR 702.54a: Bloodthirst 2 should place exactly 2 counters even if multiple opponents were damaged"
    );
}

// ── Test 6: Only controller damaged — no counters ─────────────────────────────

#[test]
/// CR 702.54a — Bloodthirst requires "an opponent" was dealt damage.
/// If only the controller was dealt damage (not an opponent), no counters are placed.
fn test_bloodthirst_only_controller_damaged() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![bloodthirst_2_def()]);

    let bt_spec = ObjectSpec::card(p1, "Bloodthirst Test Creature")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("bloodthirst-2-test".to_string()))
        .with_types(vec![CardType::Creature])
        .with_keyword(KeywordAbility::Bloodthirst(2))
        .with_mana_cost(ManaCost {
            generic: 2,
            ..Default::default()
        });

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(bt_spec)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Only the controller (p1) was dealt damage, not an opponent.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .damage_received_this_turn = 5;
    // p2 (opponent) took no damage.
    state.turn.priority_holder = Some(p1);

    let bt_id = find_object(&state, "Bloodthirst Test Creature");
    let state = cast_creature(state, p1, bt_id, 2);
    let (state, _) = pass_all(state, &[p1, p2]);

    let bf_id = find_object_on_battlefield(&state, "Bloodthirst Test Creature")
        .expect("Bloodthirst creature should be on battlefield");

    let counter_count = state.objects[&bf_id]
        .counters
        .get(&CounterType::PlusOnePlusOne)
        .copied()
        .unwrap_or(0);
    assert_eq!(
        counter_count, 0,
        "CR 702.54a: Bloodthirst should not trigger when only the controller was damaged (not an opponent)"
    );
}

// ── Test 7: Eliminated opponent not counted ───────────────────────────────────

#[test]
/// CR 800.4a / CR 102.3 — Eliminated players are not opponents.
/// An eliminated opponent's damage does not satisfy Bloodthirst.
fn test_bloodthirst_eliminated_opponent_not_counted() {
    let p1 = p(1);
    let p2 = p(2);
    let p3 = p(3);

    let registry = CardRegistry::new(vec![bloodthirst_2_def()]);

    let bt_spec = ObjectSpec::card(p1, "Bloodthirst Test Creature")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("bloodthirst-2-test".to_string()))
        .with_types(vec![CardType::Creature])
        .with_keyword(KeywordAbility::Bloodthirst(2))
        .with_mana_cost(ManaCost {
            generic: 2,
            ..Default::default()
        });

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .with_registry(registry)
        .object(bt_spec)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // p2 has been eliminated (has_lost = true), but was dealt damage this turn.
    // p3 (living opponent) took no damage.
    state.players.get_mut(&p2).unwrap().has_lost = true;
    state
        .players
        .get_mut(&p2)
        .unwrap()
        .damage_received_this_turn = 10;
    state.turn.priority_holder = Some(p1);

    let bt_id = find_object(&state, "Bloodthirst Test Creature");
    let state = cast_creature(state, p1, bt_id, 2);
    let (state, _) = pass_all(state, &[p1, p3]);

    let bf_id = find_object_on_battlefield(&state, "Bloodthirst Test Creature")
        .expect("Bloodthirst creature should be on battlefield");

    let counter_count = state.objects[&bf_id]
        .counters
        .get(&CounterType::PlusOnePlusOne)
        .copied()
        .unwrap_or(0);
    assert_eq!(
        counter_count, 0,
        "CR 800.4a: Eliminated opponents are not opponents; their damage should not satisfy Bloodthirst"
    );
}

// ── Test 8: CounterAdded event emitted ───────────────────────────────────────

#[test]
/// CR 702.54a — Verify that CounterAdded event is emitted with the correct count
/// when a Bloodthirst creature enters with counters.
fn test_bloodthirst_counter_added_event() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![bloodthirst_2_def()]);

    let bt_spec = ObjectSpec::card(p1, "Bloodthirst Test Creature")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("bloodthirst-2-test".to_string()))
        .with_types(vec![CardType::Creature])
        .with_keyword(KeywordAbility::Bloodthirst(2))
        .with_mana_cost(ManaCost {
            generic: 2,
            ..Default::default()
        });

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(bt_spec)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p2)
        .unwrap()
        .damage_received_this_turn = 4;
    state.turn.priority_holder = Some(p1);

    let bt_id = find_object(&state, "Bloodthirst Test Creature");
    let state = cast_creature(state, p1, bt_id, 2);
    let (state, resolve_events) = pass_all(state, &[p1, p2]);

    // Creature should be on the battlefield.
    let bf_id = find_object_on_battlefield(&state, "Bloodthirst Test Creature")
        .expect("CR 702.54a: Bloodthirst creature should be on battlefield");

    // Verify 2 counters placed.
    let counter_count = state.objects[&bf_id]
        .counters
        .get(&CounterType::PlusOnePlusOne)
        .copied()
        .unwrap_or(0);
    assert_eq!(counter_count, 2, "CR 702.54a: should have 2 +1/+1 counters");

    // Verify CounterAdded event emitted with count = 2.
    let has_counter_event = resolve_events.iter().any(|ev| {
        matches!(
            ev,
            GameEvent::CounterAdded {
                counter: CounterType::PlusOnePlusOne,
                count: 2,
                ..
            }
        )
    });
    assert!(
        has_counter_event,
        "CR 702.54a: CounterAdded event (count=2) should be emitted for Bloodthirst ETB counters"
    );
}
