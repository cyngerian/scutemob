//! Echo keyword ability tests (CR 702.30).
//!
//! CR 702.30a: Echo [cost] means "At the beginning of your upkeep, if this permanent
//! came under your control since the beginning of your last upkeep, sacrifice it unless
//! you pay [cost]."
//!
//! The `echo_pending` flag on `GameObject` models the "came under your control since
//! the beginning of your last upkeep" condition. It is set true at ETB and cleared
//! only when the echo trigger resolves (either by paying or by sacrificing). If the
//! trigger is countered (e.g., Stifle), the flag remains set so the trigger refires
//! next upkeep.
//!
//! Test setup note: Tests that verify the upkeep trigger start at Step::Untap with
//! priority manually set. Passing priority for all players at Untap causes the engine
//! to advance to Upkeep via handle_all_passed -> enter_step, which calls upkeep_actions
//! and queues the EchoTrigger. This avoids the need to pass through Draw (which would
//! fail with an empty library).

use mtg_engine::{
    process_command, AbilityDefinition, CardDefinition, CardId, CardRegistry, CardType, Command,
    GameEvent, GameState, GameStateBuilder, KeywordAbility, ManaColor, ManaCost, ObjectId,
    ObjectSpec, PlayerId, StackObjectKind, Step, TypeLine, ZoneId,
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

// ── Card definitions ───────────────────────────────────────────────────────────

/// Echo creature: {2}{R}, 2/2, Echo {2}{R}.
fn echo_rr_creature_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("test-echo-rr".into()),
        name: "Test Echo RR Creature".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            red: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Echo {2}{R}".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Echo(ManaCost {
                generic: 2,
                red: 1,
                ..Default::default()
            })),
            AbilityDefinition::Echo {
                cost: ManaCost {
                    generic: 2,
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

/// Echo creature with DIFFERENT echo cost: {1}{W}, 2/2, Echo {3}{W}{W}.
/// Models Karmic Guide-style cards (CR 702.30b).
fn echo_different_cost_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("test-echo-diff".into()),
        name: "Test Echo Different Cost".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            white: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Echo {3}{W}{W}".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Echo(ManaCost {
                generic: 3,
                white: 2,
                ..Default::default()
            })),
            AbilityDefinition::Echo {
                cost: ManaCost {
                    generic: 3,
                    white: 2,
                    ..Default::default()
                },
            },
        ],
        power: Some(2),
        toughness: Some(2),
        ..Default::default()
    }
}

/// ObjectSpec for the Echo RR creature already on the battlefield.
fn echo_rr_on_battlefield(owner: PlayerId) -> ObjectSpec {
    ObjectSpec::card(owner, "Test Echo RR Creature")
        .in_zone(ZoneId::Battlefield)
        .with_card_id(CardId("test-echo-rr".into()))
        .with_keyword(KeywordAbility::Echo(ManaCost {
            generic: 2,
            red: 1,
            ..Default::default()
        }))
        .with_types(vec![CardType::Creature])
}

// ── Test 1: ETB sets echo_pending ─────────────────────────────────────────────

#[test]
/// CR 702.30a — When a permanent with Echo is cast and enters the battlefield via
/// spell resolution, `echo_pending` is set to true.
fn test_echo_etb_sets_pending() {
    let p1 = p(1);
    let p2 = p(2);
    let registry = CardRegistry::new(vec![echo_rr_creature_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(
            ObjectSpec::card(p1, "Test Echo RR Creature")
                .in_zone(ZoneId::Hand(p1))
                .with_card_id(CardId("test-echo-rr".into()))
                .with_keyword(KeywordAbility::Echo(ManaCost {
                    generic: 2,
                    red: 1,
                    ..Default::default()
                }))
                .with_types(vec![CardType::Creature])
                .with_mana_cost(ManaCost {
                    generic: 2,
                    red: 1,
                    ..Default::default()
                }),
        )
        .build()
        .unwrap();

    // Give p1 mana to cast.
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
        .add(ManaColor::Colorless, 2);
    state.turn.priority_holder = Some(p1);

    let card_id = find_object(&state, "Test Echo RR Creature");

    // Cast the creature.
    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: card_id,
            targets: vec![],
            kicker_times: 0,
            alt_cost: None,
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
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
    .expect("CastSpell should succeed");

    // Resolve by passing priority for both players.
    let (state, _) = pass_all(state, &[p1, p2]);

    // CR 702.30a: permanent should be on battlefield with echo_pending = true.
    assert!(
        on_battlefield(&state, "Test Echo RR Creature"),
        "CR 702.30a: Echo creature should be on battlefield after resolving"
    );

    let obj_id =
        find_in_zone(&state, "Test Echo RR Creature", ZoneId::Battlefield).expect("on battlefield");
    assert!(
        state.objects[&obj_id].echo_pending,
        "CR 702.30a: echo_pending should be true after the permanent enters the battlefield"
    );
}

// ── Test 2: echo_pending is false for a non-Echo permanent ───────────────────

#[test]
/// CR 702.30a — A permanent without Echo should have echo_pending = false.
fn test_echo_pending_false_without_echo() {
    let p1 = p(1);
    let p2 = p(2);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(ObjectSpec::creature(p1, "Vanilla Bear", 2, 2))
        .build()
        .unwrap();

    let obj_id = find_object(&state, "Vanilla Bear");
    assert!(
        !state.objects[&obj_id].echo_pending,
        "CR 702.30a: echo_pending must be false for a permanent without Echo"
    );
}

// ── Test 3: Upkeep trigger queues EchoTrigger ────────────────────────────────

#[test]
/// CR 702.30a — At the beginning of the controller's upkeep, if echo_pending is true
/// and the permanent has Echo, an EchoTrigger is queued on the stack.
fn test_echo_upkeep_trigger_fires() {
    let p1 = p(1);
    let p2 = p(2);
    let registry = CardRegistry::new(vec![echo_rr_creature_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .active_player(p1)
        .at_step(Step::Untap)
        .object(echo_rr_on_battlefield(p1))
        .build()
        .unwrap();

    // Manually set echo_pending = true (simulating ETB from spell resolution).
    let obj_id = find_object(&state, "Test Echo RR Creature");
    if let Some(obj) = state.objects.get_mut(&obj_id) {
        obj.echo_pending = true;
    }
    state.turn.priority_holder = Some(p1);

    // Advance Untap -> Upkeep (enter_step calls upkeep_actions, queuing EchoTrigger).
    let (state, _) = pass_all(state, &[p1, p2]);
    assert_eq!(
        state.turn.step,
        Step::Upkeep,
        "should be at Upkeep after advancing from Untap"
    );

    // EchoTrigger should be on the stack.
    assert!(
        state
            .stack_objects
            .iter()
            .any(|so| matches!(&so.kind, StackObjectKind::EchoTrigger { .. })),
        "CR 702.30a: EchoTrigger should be on the stack at controller's upkeep"
    );
}

// ── Test 4: Paying the echo cost keeps the permanent ─────────────────────────

#[test]
/// CR 702.30a — When the echo trigger resolves and the controller pays the echo cost,
/// the permanent stays on the battlefield and echo_pending is cleared.
fn test_echo_pay_cost_keeps_permanent() {
    let p1 = p(1);
    let p2 = p(2);
    let registry = CardRegistry::new(vec![echo_rr_creature_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .active_player(p1)
        .at_step(Step::Untap)
        .object(echo_rr_on_battlefield(p1))
        .build()
        .unwrap();

    // Set echo_pending.
    let obj_id = find_object(&state, "Test Echo RR Creature");
    if let Some(obj) = state.objects.get_mut(&obj_id) {
        obj.echo_pending = true;
    }
    state.turn.priority_holder = Some(p1);

    // Advance Untap -> Upkeep -> EchoTrigger queued.
    let (state, _) = pass_all(state, &[p1, p2]);
    assert_eq!(state.turn.step, Step::Upkeep);

    // Both pass -> EchoTrigger resolves -> EchoPaymentRequired emitted.
    let (mut state, events) = pass_all(state, &[p1, p2]);

    let perm_id =
        find_in_zone(&state, "Test Echo RR Creature", ZoneId::Battlefield).expect("on battlefield");

    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::EchoPaymentRequired { .. })),
        "CR 702.30a: EchoPaymentRequired event should be emitted when trigger resolves"
    );

    // Give p1 mana to pay the echo cost NOW (after step transitions which clear mana pools).
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
        .add(ManaColor::Colorless, 2);

    // Player pays the echo cost.
    let (state, pay_events) = process_command(
        state,
        Command::PayEcho {
            player: p1,
            permanent: perm_id,
            pay: true,
        },
    )
    .expect("PayEcho should succeed");

    // Permanent stays on battlefield.
    assert!(
        on_battlefield(&state, "Test Echo RR Creature"),
        "CR 702.30a: permanent should stay on battlefield when echo cost is paid"
    );

    // echo_pending cleared.
    let obj_id_after =
        find_in_zone(&state, "Test Echo RR Creature", ZoneId::Battlefield).expect("on battlefield");
    assert!(
        !state.objects[&obj_id_after].echo_pending,
        "CR 702.30a: echo_pending should be false after paying echo cost"
    );

    // EchoPaid event emitted.
    assert!(
        pay_events
            .iter()
            .any(|e| matches!(e, GameEvent::EchoPaid { .. })),
        "CR 702.30a: EchoPaid event should be emitted when player pays"
    );
}

// ── Test 5: Declining payment sacrifices the permanent ───────────────────────

#[test]
/// CR 702.30a — When the echo trigger resolves and the controller declines to pay,
/// the permanent is sacrificed (sent to the graveyard).
fn test_echo_decline_payment_sacrifices() {
    let p1 = p(1);
    let p2 = p(2);
    let registry = CardRegistry::new(vec![echo_rr_creature_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .active_player(p1)
        .at_step(Step::Untap)
        .object(echo_rr_on_battlefield(p1))
        .build()
        .unwrap();

    // Set echo_pending = true but give NO mana (can't pay, will decline).
    let obj_id = find_object(&state, "Test Echo RR Creature");
    if let Some(obj) = state.objects.get_mut(&obj_id) {
        obj.echo_pending = true;
    }
    state.turn.priority_holder = Some(p1);

    // Advance Untap -> Upkeep -> EchoTrigger queued.
    let (state, _) = pass_all(state, &[p1, p2]);
    assert_eq!(state.turn.step, Step::Upkeep);

    // Both pass -> trigger resolves -> EchoPaymentRequired.
    let (state, _) = pass_all(state, &[p1, p2]);

    let perm_id =
        find_in_zone(&state, "Test Echo RR Creature", ZoneId::Battlefield).expect("on battlefield");

    // Player declines to pay.
    let (state, sac_events) = process_command(
        state,
        Command::PayEcho {
            player: p1,
            permanent: perm_id,
            pay: false,
        },
    )
    .expect("PayEcho (decline) should succeed");

    // Permanent is no longer on battlefield.
    assert!(
        !on_battlefield(&state, "Test Echo RR Creature"),
        "CR 702.30a: permanent should be sacrificed when echo cost is declined"
    );

    // Permanent moved to owner's graveyard.
    assert!(
        in_graveyard(&state, "Test Echo RR Creature", p1),
        "CR 702.30a: sacrificed Echo permanent should be in owner's graveyard"
    );

    // CreatureDied event emitted (sacrifice uses the zone change path).
    assert!(
        sac_events
            .iter()
            .any(|e| matches!(e, GameEvent::CreatureDied { .. })),
        "CR 702.30a: CreatureDied event should be emitted when echo permanent is sacrificed"
    );
}

// ── Test 6: No trigger after echo has been paid ───────────────────────────────

#[test]
/// CR 702.30a — After the echo cost has been paid (echo_pending = false), the next
/// upkeep should NOT queue another EchoTrigger.
fn test_echo_no_trigger_after_paid() {
    let p1 = p(1);
    let p2 = p(2);
    let registry = CardRegistry::new(vec![echo_rr_creature_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .active_player(p1)
        .at_step(Step::Untap)
        .object(echo_rr_on_battlefield(p1))
        .build()
        .unwrap();

    // Set echo_pending = true.
    let obj_id = find_object(&state, "Test Echo RR Creature");
    if let Some(obj) = state.objects.get_mut(&obj_id) {
        obj.echo_pending = true;
    }
    state.turn.priority_holder = Some(p1);

    // === First upkeep: echo fires and is paid ===
    let (state, _) = pass_all(state, &[p1, p2]);
    assert_eq!(state.turn.step, Step::Upkeep);

    let (mut state, _) = pass_all(state, &[p1, p2]); // trigger resolves -> EchoPaymentRequired

    let perm_id =
        find_in_zone(&state, "Test Echo RR Creature", ZoneId::Battlefield).expect("on battlefield");

    // Give p1 mana to pay NOW (step transitions clear mana pools).
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
        .add(ManaColor::Colorless, 2);

    let (state, _) = process_command(
        state,
        Command::PayEcho {
            player: p1,
            permanent: perm_id,
            pay: true,
        },
    )
    .expect("PayEcho should succeed");

    // echo_pending should be false now.
    let obj_id_after =
        find_in_zone(&state, "Test Echo RR Creature", ZoneId::Battlefield).expect("on battlefield");
    assert!(
        !state.objects[&obj_id_after].echo_pending,
        "echo_pending should be false after paying"
    );

    // === Second upkeep: echo must NOT fire ===
    let mut state = state;
    state.turn.step = Step::Untap;
    state.turn.priority_holder = Some(p1);

    let (state, _) = pass_all(state, &[p1, p2]);
    assert_eq!(state.turn.step, Step::Upkeep);

    // No EchoTrigger on stack this time.
    assert!(
        !state
            .stack_objects
            .iter()
            .any(|so| matches!(&so.kind, StackObjectKind::EchoTrigger { .. })),
        "CR 702.30a: EchoTrigger must NOT fire on second upkeep after echo was paid"
    );
}

// ── Test 7: Different echo cost (Karmic Guide pattern) ───────────────────────

#[test]
/// CR 702.30b — Echo cost can differ from the card's mana cost. The EchoTrigger
/// should carry the ECHO cost, not the mana cost.
fn test_echo_different_cost() {
    let p1 = p(1);
    let p2 = p(2);
    let registry = CardRegistry::new(vec![echo_different_cost_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .active_player(p1)
        .at_step(Step::Untap)
        .object(
            ObjectSpec::card(p1, "Test Echo Different Cost")
                .in_zone(ZoneId::Battlefield)
                .with_card_id(CardId("test-echo-diff".into()))
                .with_keyword(KeywordAbility::Echo(ManaCost {
                    generic: 3,
                    white: 2,
                    ..Default::default()
                }))
                .with_types(vec![CardType::Creature]),
        )
        .build()
        .unwrap();

    let obj_id = find_object(&state, "Test Echo Different Cost");
    if let Some(obj) = state.objects.get_mut(&obj_id) {
        obj.echo_pending = true;
    }
    state.turn.priority_holder = Some(p1);

    // Advance to upkeep.
    let (state, _) = pass_all(state, &[p1, p2]);
    assert_eq!(state.turn.step, Step::Upkeep);

    // EchoTrigger should be on the stack.
    assert!(
        state
            .stack_objects
            .iter()
            .any(|so| matches!(&so.kind, StackObjectKind::EchoTrigger { .. })),
        "CR 702.30b: EchoTrigger should fire for card with different echo cost"
    );

    // Verify the trigger carries the ECHO cost ({3}{W}{W}), not the mana cost ({1}{W}).
    let echo_trigger = state
        .stack_objects
        .iter()
        .find(|so| matches!(&so.kind, StackObjectKind::EchoTrigger { .. }))
        .expect("EchoTrigger must be on stack");

    if let StackObjectKind::EchoTrigger { echo_cost, .. } = &echo_trigger.kind {
        assert_eq!(
            echo_cost.generic, 3,
            "CR 702.30b: echo cost generic should be 3 (not 1)"
        );
        assert_eq!(
            echo_cost.white, 2,
            "CR 702.30b: echo cost white should be 2 (not 1)"
        );
    } else {
        panic!("expected EchoTrigger");
    }
}

// ── Test 8: Multiplayer — only controller's upkeep triggers echo ──────────────

#[test]
/// CR 702.30a "your upkeep" — In multiplayer, Echo only triggers on the controller's
/// upkeep, not on other players' upkeeps.
fn test_echo_multiplayer_only_controller_upkeep() {
    let p1 = p(1);
    let p2 = p(2);
    let registry = CardRegistry::new(vec![echo_rr_creature_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .active_player(p2) // p2 is active, p1 controls the echo creature
        .at_step(Step::Untap)
        .object(echo_rr_on_battlefield(p1)) // p1's creature
        .build()
        .unwrap();

    // Set echo_pending = true for p1's creature.
    let obj_id = find_object(&state, "Test Echo RR Creature");
    if let Some(obj) = state.objects.get_mut(&obj_id) {
        obj.echo_pending = true;
    }
    state.turn.priority_holder = Some(p2);

    // Advance p2's Untap -> p2's Upkeep.
    let (state, _) = pass_all(state, &[p2, p1]);
    assert_eq!(state.turn.step, Step::Upkeep);

    // No EchoTrigger for p1's permanent during p2's upkeep.
    assert!(
        !state
            .stack_objects
            .iter()
            .any(|so| matches!(&so.kind, StackObjectKind::EchoTrigger { .. })),
        "CR 702.30a: EchoTrigger must not fire on a non-controller's upkeep"
    );

    // echo_pending still true (hasn't been consumed).
    let obj_id_check = find_object(&state, "Test Echo RR Creature");
    assert!(
        state.objects[&obj_id_check].echo_pending,
        "CR 702.30a: echo_pending should remain true during a non-controller's upkeep"
    );
}

// ── Test 9: Permanent that left battlefield before trigger resolves ────────────

#[test]
/// CR 400.7 — If the Echo permanent leaves the battlefield before the echo trigger
/// resolves, the trigger should do nothing (not add to pending_echo_payments).
fn test_echo_permanent_left_battlefield() {
    let p1 = p(1);
    let p2 = p(2);
    let registry = CardRegistry::new(vec![echo_rr_creature_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .active_player(p1)
        .at_step(Step::Untap)
        .object(echo_rr_on_battlefield(p1))
        .build()
        .unwrap();

    // Set echo_pending = true.
    let obj_id = find_object(&state, "Test Echo RR Creature");
    if let Some(obj) = state.objects.get_mut(&obj_id) {
        obj.echo_pending = true;
    }
    state.turn.priority_holder = Some(p1);

    // Advance to Upkeep -> EchoTrigger queued.
    let (state, _) = pass_all(state, &[p1, p2]);
    assert_eq!(state.turn.step, Step::Upkeep);

    // Verify EchoTrigger is on stack.
    assert!(
        state
            .stack_objects
            .iter()
            .any(|so| matches!(&so.kind, StackObjectKind::EchoTrigger { .. })),
        "EchoTrigger should be on stack"
    );

    // Simulate the permanent leaving the battlefield BEFORE the trigger resolves
    // by finding the trigger's echo_permanent and removing the object from the battlefield.
    let echo_trigger = state
        .stack_objects
        .iter()
        .find(|so| matches!(&so.kind, StackObjectKind::EchoTrigger { .. }))
        .expect("EchoTrigger must be on stack");

    let perm_id = if let StackObjectKind::EchoTrigger { echo_permanent, .. } = &echo_trigger.kind {
        *echo_permanent
    } else {
        panic!("expected EchoTrigger");
    };

    // Move the permanent to its graveyard to simulate it leaving.
    let mut state = state;
    if let Some(obj) = state.objects.get_mut(&perm_id) {
        let old_zone = obj.zone;
        obj.zone = ZoneId::Graveyard(p1);
        // Update the zone collections.
        if let Some(zone) = state.zones.get_mut(&old_zone) {
            zone.remove(&perm_id);
        }
        if let Some(zone) = state.zones.get_mut(&ZoneId::Graveyard(p1)) {
            zone.insert(perm_id);
        }
    }

    // Trigger resolves: permanent not on battlefield -> no EchoPaymentRequired.
    let (state, events) = pass_all(state, &[p1, p2]);

    assert!(
        !events
            .iter()
            .any(|e| matches!(e, GameEvent::EchoPaymentRequired { .. })),
        "CR 400.7: EchoPaymentRequired must not be emitted if permanent left battlefield"
    );

    // No pending echo payments.
    assert!(
        state.pending_echo_payments.is_empty(),
        "CR 400.7: pending_echo_payments should be empty if permanent left battlefield"
    );
}
