//! Cumulative Upkeep keyword ability tests (CR 702.24).
//!
//! CR 702.24a: Cumulative upkeep [cost] means "At the beginning of your upkeep,
//! if this permanent is on the battlefield, put an age counter on this permanent.
//! Then you may pay [cost] for each age counter on it. If you don't, sacrifice it."
//!
//! Test setup note: Tests that verify the upkeep trigger start at Step::Untap with
//! priority manually set. Passing priority for all players at Untap causes the engine
//! to advance to Upkeep via handle_all_passed -> enter_step, which calls upkeep_actions
//! and queues the CumulativeUpkeepTrigger. This mirrors the echo.rs test pattern.

use mtg_engine::{
    process_command, AbilityDefinition, CardDefinition, CardId, CardRegistry, CardType, Command,
    CounterType, CumulativeUpkeepCost, GameEvent, GameState, GameStateBuilder, KeywordAbility,
    ManaColor, ManaCost, ObjectId, ObjectSpec, PlayerId, StackObjectKind, Step, TypeLine, ZoneId,
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
    state.objects.iter().find_map(|(&id, obj)| {
        if obj.characteristics.name == name && obj.zone == zone {
            Some(id)
        } else {
            None
        }
    })
}

fn on_battlefield(state: &GameState, name: &str) -> bool {
    find_in_zone(state, name, ZoneId::Battlefield).is_some()
}

fn in_graveyard(state: &GameState, name: &str, owner: PlayerId) -> bool {
    find_in_zone(state, name, ZoneId::Graveyard(owner)).is_some()
}

fn age_counters_on(state: &GameState, obj_id: ObjectId) -> u32 {
    state
        .objects
        .get(&obj_id)
        .and_then(|obj| obj.counters.get(&CounterType::Age).copied())
        .unwrap_or(0)
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

// ── Card definitions ──────────────────────────────────────────────────────────

/// CU enchantment with mana cost {1} per counter (Mystic Remora pattern).
fn cu_mana_1_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("test-cu-mana-1".into()),
        name: "Test CU Mana 1".to_string(),
        mana_cost: Some(ManaCost {
            blue: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Enchantment].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Cumulative upkeep {1}".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::CumulativeUpkeep(
                CumulativeUpkeepCost::Mana(ManaCost {
                    generic: 1,
                    ..Default::default()
                }),
            )),
            AbilityDefinition::CumulativeUpkeep {
                cost: CumulativeUpkeepCost::Mana(ManaCost {
                    generic: 1,
                    ..Default::default()
                }),
            },
        ],
        ..Default::default()
    }
}

/// CU permanent with life cost 2 per counter (Glacial Chasm pattern).
fn cu_life_2_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("test-cu-life-2".into()),
        name: "Test CU Life 2".to_string(),
        mana_cost: Some(ManaCost::default()),
        types: TypeLine {
            card_types: [CardType::Land].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Cumulative upkeep -- Pay 2 life".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::CumulativeUpkeep(
                CumulativeUpkeepCost::Life(2),
            )),
            AbilityDefinition::CumulativeUpkeep {
                cost: CumulativeUpkeepCost::Life(2),
            },
        ],
        ..Default::default()
    }
}

/// ObjectSpec for the mana CU permanent already on the battlefield.
fn cu_mana_1_on_battlefield(owner: PlayerId) -> ObjectSpec {
    ObjectSpec::card(owner, "Test CU Mana 1")
        .in_zone(ZoneId::Battlefield)
        .with_card_id(CardId("test-cu-mana-1".into()))
        .with_keyword(KeywordAbility::CumulativeUpkeep(
            CumulativeUpkeepCost::Mana(ManaCost {
                generic: 1,
                ..Default::default()
            }),
        ))
        .with_types(vec![CardType::Enchantment])
}

/// ObjectSpec for the life CU permanent already on the battlefield.
fn cu_life_2_on_battlefield(owner: PlayerId) -> ObjectSpec {
    ObjectSpec::card(owner, "Test CU Life 2")
        .in_zone(ZoneId::Battlefield)
        .with_card_id(CardId("test-cu-life-2".into()))
        .with_keyword(KeywordAbility::CumulativeUpkeep(
            CumulativeUpkeepCost::Life(2),
        ))
        .with_types(vec![CardType::Land])
}

// ── Test 1: Basic trigger fires and adds age counter ─────────────────────────

#[test]
/// CR 702.24a — At the beginning of the controller's upkeep, a CumulativeUpkeepTrigger
/// is queued on the stack and an age counter is added on resolution.
fn test_cumulative_upkeep_basic_age_counter_added() {
    let p1 = p(1);
    let p2 = p(2);
    let registry = CardRegistry::new(vec![cu_mana_1_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .active_player(p1)
        .at_step(Step::Untap)
        .object(cu_mana_1_on_battlefield(p1))
        .build()
        .unwrap();

    state.turn.priority_holder = Some(p1);

    // Advance Untap -> Upkeep (enter_step calls upkeep_actions, queuing trigger).
    let (state, _) = pass_all(state, &[p1, p2]);
    assert_eq!(state.turn.step, Step::Upkeep);

    // CumulativeUpkeepTrigger should be on the stack.
    assert!(
        state.stack_objects.iter().any(|so| matches!(
            &so.kind,
            StackObjectKind::KeywordTrigger {
                keyword: KeywordAbility::CumulativeUpkeep(_),
                data: mtg_engine::TriggerData::UpkeepCost { .. },
                ..
            }
        )),
        "CR 702.24a: CumulativeUpkeepTrigger should be on the stack at controller's upkeep"
    );

    // Both pass -> trigger resolves -> age counter added + payment required.
    let (state, events) = pass_all(state, &[p1, p2]);

    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::CumulativeUpkeepPaymentRequired { .. })),
        "CR 702.24a: CumulativeUpkeepPaymentRequired event should be emitted"
    );

    // Age counter should be on the permanent.
    let obj_id = find_in_zone(&state, "Test CU Mana 1", ZoneId::Battlefield)
        .expect("permanent should be on battlefield");
    assert_eq!(
        age_counters_on(&state, obj_id),
        1,
        "CR 702.24a: one age counter should be added on the first upkeep"
    );
}

// ── Test 2: Paying the mana cost keeps the permanent ─────────────────────────

#[test]
/// CR 702.24a — When the CU trigger resolves and the controller pays the mana cost
/// (per_counter_cost x age_count), the permanent stays on the battlefield.
fn test_cumulative_upkeep_pay_mana_keeps_permanent() {
    let p1 = p(1);
    let p2 = p(2);
    let registry = CardRegistry::new(vec![cu_mana_1_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .active_player(p1)
        .at_step(Step::Untap)
        .object(cu_mana_1_on_battlefield(p1))
        .build()
        .unwrap();

    state.turn.priority_holder = Some(p1);

    // Advance to Upkeep.
    let (state, _) = pass_all(state, &[p1, p2]);
    assert_eq!(state.turn.step, Step::Upkeep);

    // Resolve the trigger (adds 1 age counter, emits payment required).
    let (mut state, _) = pass_all(state, &[p1, p2]);

    let perm_id = find_in_zone(&state, "Test CU Mana 1", ZoneId::Battlefield)
        .expect("permanent should be on battlefield");

    // Give p1 {1} generic mana to pay 1x{1}.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 1);

    // Pay cumulative upkeep.
    let (state, pay_events) = process_command(
        state,
        Command::PayCumulativeUpkeep {
            player: p1,
            permanent: perm_id,
            pay: true,
        },
    )
    .expect("PayCumulativeUpkeep should succeed");

    assert!(
        on_battlefield(&state, "Test CU Mana 1"),
        "CR 702.24a: permanent should stay on battlefield when CU cost is paid"
    );

    assert!(
        pay_events
            .iter()
            .any(|e| matches!(e, GameEvent::CumulativeUpkeepPaid { .. })),
        "CR 702.24a: CumulativeUpkeepPaid event should be emitted"
    );
}

// ── Test 3: Declining payment sacrifices the permanent ───────────────────────

#[test]
/// CR 702.24a — When the CU trigger resolves and the controller declines payment,
/// the permanent is sacrificed (bypassing indestructible, CR 701.17a).
fn test_cumulative_upkeep_decline_payment_sacrifices() {
    let p1 = p(1);
    let p2 = p(2);
    let registry = CardRegistry::new(vec![cu_mana_1_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .active_player(p1)
        .at_step(Step::Untap)
        .object(cu_mana_1_on_battlefield(p1))
        .build()
        .unwrap();

    state.turn.priority_holder = Some(p1);

    // Advance to Upkeep.
    let (state, _) = pass_all(state, &[p1, p2]);
    assert_eq!(state.turn.step, Step::Upkeep);

    // Resolve the trigger.
    let (state, _) = pass_all(state, &[p1, p2]);

    let perm_id = find_in_zone(&state, "Test CU Mana 1", ZoneId::Battlefield)
        .expect("permanent should be on battlefield");

    // Decline payment.
    let (state, _) = process_command(
        state,
        Command::PayCumulativeUpkeep {
            player: p1,
            permanent: perm_id,
            pay: false,
        },
    )
    .expect("PayCumulativeUpkeep decline should succeed");

    assert!(
        !on_battlefield(&state, "Test CU Mana 1"),
        "CR 702.24a: permanent should leave battlefield when CU cost is not paid"
    );

    assert!(
        in_graveyard(&state, "Test CU Mana 1", p1),
        "CR 702.24a: permanent should go to graveyard when CU cost is not paid"
    );
}

// ── Test 4: Escalating cost over multiple upkeeps ────────────────────────────

#[test]
/// CR 702.24a — The cost increases each upkeep:
/// Turn 1: 1 age counter added, pay 1x{1} = {1}.
/// Turn 2: 2 age counters total, pay 2x{1} = {2}.
/// Turn 3: 3 age counters total, pay 3x{1} = {3}.
fn test_cumulative_upkeep_escalating_cost() {
    let p1 = p(1);
    let p2 = p(2);
    let registry = CardRegistry::new(vec![cu_mana_1_def()]);

    // Manually construct two upkeep resolution cycles.
    // Cycle 1: start at Untap, advance to Upkeep, resolve trigger, pay {1}.
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .active_player(p1)
        .at_step(Step::Untap)
        .object(cu_mana_1_on_battlefield(p1))
        .build()
        .unwrap();
    state.turn.priority_holder = Some(p1);

    // --- Upkeep 1 ---
    let (state, _) = pass_all(state, &[p1, p2]);
    assert_eq!(state.turn.step, Step::Upkeep);
    let (mut state, _) = pass_all(state, &[p1, p2]);

    let perm_id = find_in_zone(&state, "Test CU Mana 1", ZoneId::Battlefield)
        .expect("on battlefield after upkeep 1");
    assert_eq!(
        age_counters_on(&state, perm_id),
        1,
        "CR 702.24a: 1 age counter after first upkeep"
    );

    // Pay {1} (age_count=1, cost={1}).
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 1);
    let (state, _) = process_command(
        state,
        Command::PayCumulativeUpkeep {
            player: p1,
            permanent: perm_id,
            pay: true,
        },
    )
    .expect("Upkeep 1 payment");

    assert!(on_battlefield(&state, "Test CU Mana 1"));

    // Advance to next turn's upkeep by advancing past the full turn.
    // We simulate by directly constructing a new state at Untap with the permanent
    // now having 1 age counter.
    let perm_id2 = find_in_zone(&state, "Test CU Mana 1", ZoneId::Battlefield).unwrap();
    let age_after_turn1 = age_counters_on(&state, perm_id2);
    assert_eq!(
        age_after_turn1, 1,
        "CR 702.24a: 1 age counter remains after paying upkeep 1"
    );

    // --- Upkeep 2: age counter was already 1, trigger adds 1 more = 2 total ---
    // Advance to upkeep again by manipulating step and priority.
    let mut state2 = state;
    // Move to Untap of the same player (simulate next turn start).
    state2.turn.step = Step::Untap;
    state2.turn.priority_holder = Some(p1);

    let (state2, _) = pass_all(state2, &[p1, p2]);
    assert_eq!(state2.turn.step, Step::Upkeep);
    let (mut state2, _) = pass_all(state2, &[p1, p2]);

    let perm_id3 = find_in_zone(&state2, "Test CU Mana 1", ZoneId::Battlefield)
        .expect("on battlefield after upkeep 2 trigger");
    assert_eq!(
        age_counters_on(&state2, perm_id3),
        2,
        "CR 702.24a: 2 age counters after second upkeep trigger resolves"
    );

    // CumulativeUpkeepPaymentRequired should show age_counter_count = 2.
    assert!(
        state2
            .pending_cumulative_upkeep_payments
            .iter()
            .any(|(_, obj, _)| *obj == perm_id3),
        "CR 702.24a: pending payment entry should exist for upkeep 2"
    );

    // Pay {2} (age_count=2, cost=2x{1}).
    state2
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 2);
    let (state2, _) = process_command(
        state2,
        Command::PayCumulativeUpkeep {
            player: p1,
            permanent: perm_id3,
            pay: true,
        },
    )
    .expect("Upkeep 2 payment");

    assert!(
        on_battlefield(&state2, "Test CU Mana 1"),
        "CR 702.24a: permanent stays after paying upkeep 2"
    );

    let perm_id4 = find_in_zone(&state2, "Test CU Mana 1", ZoneId::Battlefield).unwrap();
    assert_eq!(
        age_counters_on(&state2, perm_id4),
        2,
        "CR 702.24a: 2 age counters remain after paying upkeep 2"
    );
}

// ── Test 5: Life cost variant ─────────────────────────────────────────────────

#[test]
/// CR 702.24a — Life-based cumulative upkeep (Glacial Chasm pattern): paying the life
/// cost keeps the permanent; the player's life total is reduced accordingly.
fn test_cumulative_upkeep_pay_life_cost() {
    let p1 = p(1);
    let p2 = p(2);
    let registry = CardRegistry::new(vec![cu_life_2_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .active_player(p1)
        .at_step(Step::Untap)
        .object(cu_life_2_on_battlefield(p1))
        .build()
        .unwrap();

    state.turn.priority_holder = Some(p1);

    let initial_life = state.players[&p1].life_total;

    // Advance to Upkeep.
    let (state, _) = pass_all(state, &[p1, p2]);
    assert_eq!(state.turn.step, Step::Upkeep);

    // Resolve trigger (adds 1 age counter).
    let (state, _) = pass_all(state, &[p1, p2]);

    let perm_id = find_in_zone(&state, "Test CU Life 2", ZoneId::Battlefield)
        .expect("permanent should be on battlefield");

    // Pay 2 life (1 age counter x 2 life each = 2 life total).
    let (state, pay_events) = process_command(
        state,
        Command::PayCumulativeUpkeep {
            player: p1,
            permanent: perm_id,
            pay: true,
        },
    )
    .expect("PayCumulativeUpkeep (life) should succeed");

    assert!(
        on_battlefield(&state, "Test CU Life 2"),
        "CR 702.24a: permanent should stay on battlefield when life cost is paid"
    );

    assert_eq!(
        state.players[&p1].life_total,
        initial_life - 2,
        "CR 702.24a: player should lose 2 life paying cumulative upkeep (1 counter * 2 life)"
    );

    assert!(
        pay_events
            .iter()
            .any(|e| matches!(e, GameEvent::LifeLost { .. })),
        "CR 702.24a: LifeLost event should be emitted for life-based cumulative upkeep"
    );
}

// ── Test 6: Permanent left battlefield before trigger resolves ────────────────

#[test]
/// CR 400.7 — If the permanent left the battlefield before the CumulativeUpkeepTrigger
/// resolves, the trigger does nothing (the age counter is not added to any object).
fn test_cumulative_upkeep_permanent_left_battlefield() {
    let p1 = p(1);
    let p2 = p(2);
    let registry = CardRegistry::new(vec![cu_mana_1_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .active_player(p1)
        .at_step(Step::Untap)
        .object(cu_mana_1_on_battlefield(p1))
        .build()
        .unwrap();

    state.turn.priority_holder = Some(p1);

    // Advance to Upkeep -- trigger is queued.
    let (state, _) = pass_all(state, &[p1, p2]);
    assert_eq!(state.turn.step, Step::Upkeep);

    // Trigger is on stack. Before resolving, remove the permanent from the battlefield
    // by manually moving it to the graveyard.
    let obj_id = find_object(&state, "Test CU Mana 1");
    let mut state2 = state;
    if let Some(obj) = state2.objects.get_mut(&obj_id) {
        obj.zone = ZoneId::Graveyard(p1);
    }

    // Now resolve the trigger (both players pass).
    let (state2, events) = pass_all(state2, &[p1, p2]);

    // No payment required event -- permanent was not on battlefield.
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, GameEvent::CumulativeUpkeepPaymentRequired { .. })),
        "CR 400.7: no payment required when permanent left battlefield before trigger resolved"
    );

    // No pending payments.
    assert!(
        state2.pending_cumulative_upkeep_payments.is_empty(),
        "CR 400.7: no pending payment entry when permanent left battlefield"
    );
}

// ── Test 7: Only fires on the controller's upkeep ─────────────────────────────

#[test]
/// CR 702.24a — "At the beginning of YOUR upkeep": the trigger only fires when
/// the active player is the permanent's controller.
fn test_cumulative_upkeep_multiplayer_only_controller_upkeep() {
    let p1 = p(1);
    let p2 = p(2);
    let registry = CardRegistry::new(vec![cu_mana_1_def()]);

    // P1 controls the CU permanent but it is P2's upkeep.
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .active_player(p2) // p2 is active -- NOT the controller of the CU permanent
        .at_step(Step::Untap)
        .object(cu_mana_1_on_battlefield(p1)) // owned+controlled by p1
        .build()
        .unwrap();

    state.turn.priority_holder = Some(p2);

    // Advance p2's Untap -> Upkeep.
    let (state, _) = pass_all(state, &[p2, p1]);
    assert_eq!(state.turn.step, Step::Upkeep);

    // No CumulativeUpkeepTrigger should be on the stack (it's p2's upkeep, not p1's).
    assert!(
        !state.stack_objects.iter().any(|so| matches!(
            &so.kind,
            StackObjectKind::KeywordTrigger {
                keyword: KeywordAbility::CumulativeUpkeep(_),
                data: mtg_engine::TriggerData::UpkeepCost { .. },
                ..
            }
        )),
        "CR 702.24a: CumulativeUpkeepTrigger should NOT fire on a non-controller's upkeep"
    );

    // No pending triggers for CumulativeUpkeep.
    assert!(
        state.pending_triggers.iter().all(|t| !matches!(
            t.kind,
            mtg_engine::state::stubs::PendingTriggerKind::KeywordTrigger {
                keyword: KeywordAbility::CumulativeUpkeep(_),
                ..
            }
        )),
        "CR 702.24a: no CumulativeUpkeep trigger pending on non-controller's upkeep"
    );
}

// ── Test 8: Multiple CU instances trigger separately, share age counters ──────

#[test]
/// CR 702.24b — Two instances of cumulative upkeep on the same permanent trigger
/// separately. Each trigger adds one age counter when it resolves. When both have
/// resolved, the permanent has 2 age counters.
fn test_cumulative_upkeep_multiple_instances_share_counters() {
    let p1 = p(1);
    let p2 = p(2);
    let registry = CardRegistry::new(vec![cu_mana_1_def()]);

    // Object with TWO instances of CumulativeUpkeep({1}).
    // im::OrdSet deduplicates equal values -- use Mana({1}) and Mana({2})
    // to represent two distinct instances (per gotchas-infra.md).
    let obj_two_cu = ObjectSpec::card(p1, "Test CU Mana 1")
        .in_zone(ZoneId::Battlefield)
        .with_card_id(CardId("test-cu-mana-1".into()))
        .with_keyword(KeywordAbility::CumulativeUpkeep(
            CumulativeUpkeepCost::Mana(ManaCost {
                generic: 1,
                ..Default::default()
            }),
        ))
        .with_keyword(KeywordAbility::CumulativeUpkeep(
            CumulativeUpkeepCost::Mana(ManaCost {
                generic: 2,
                ..Default::default()
            }),
        ))
        .with_types(vec![CardType::Enchantment]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .active_player(p1)
        .at_step(Step::Untap)
        .object(obj_two_cu)
        .build()
        .unwrap();

    state.turn.priority_holder = Some(p1);

    // Advance to Upkeep -- both triggers should be queued.
    let (state, _) = pass_all(state, &[p1, p2]);
    assert_eq!(state.turn.step, Step::Upkeep);

    let cu_trigger_count = state
        .stack_objects
        .iter()
        .filter(|so| {
            matches!(
                &so.kind,
                StackObjectKind::KeywordTrigger {
                    keyword: KeywordAbility::CumulativeUpkeep(_),
                    data: mtg_engine::TriggerData::UpkeepCost { .. },
                    ..
                }
            )
        })
        .count();

    assert_eq!(
        cu_trigger_count, 2,
        "CR 702.24b: two CU instances should produce two separate triggers on the stack"
    );
}
