//! Recover keyword ability tests (CR 702.59).
//!
//! CR 702.59a: Recover [cost] means "When a creature is put into your graveyard from
//! the battlefield, you may pay [cost]. If you do, return this card from your graveyard
//! to your hand. If you don't, exile this card."
//!
//! Implementation notes:
//! - Recover is a triggered ability on a card in the graveyard.
//! - The trigger fires when a creature is put into the card owner's graveyard from
//!   the battlefield (CR 702.59a). A creature itself may have Recover (it triggers on
//!   its own death if it was also in the graveyard, which is unusual, or it can trigger
//!   from another creature dying).
//! - When the RecoverTrigger resolves, the engine emits RecoverPaymentRequired and
//!   queues the cost in `pending_recover_payments`.
//! - The player responds with Command::PayRecover { pay: true } to return the card to
//!   hand, or Command::PayRecover { pay: false } to send it to exile.
//! - CR 400.7: If the Recover card left the graveyard before the trigger resolves,
//!   the trigger does nothing.

use mtg_engine::{
    check_and_apply_sbas, process_command, AbilityDefinition, CardDefinition, CardId, CardRegistry,
    CardType, Command, GameEvent, GameState, GameStateBuilder, KeywordAbility, ManaCost, ObjectId,
    ObjectSpec, PlayerId, StackObjectKind, Step, TypeLine, ZoneId,
};

// ── Helpers ───────────────────────────────────────────────────────────────────

fn p(n: u64) -> PlayerId {
    PlayerId(n)
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

fn in_graveyard(state: &GameState, name: &str, owner: PlayerId) -> bool {
    find_in_zone(state, name, ZoneId::Graveyard(owner)).is_some()
}

fn in_hand(state: &GameState, name: &str, owner: PlayerId) -> bool {
    find_in_zone(state, name, ZoneId::Hand(owner)).is_some()
}

fn in_exile(state: &GameState, name: &str) -> bool {
    state
        .objects
        .values()
        .any(|obj| obj.characteristics.name == name && matches!(obj.zone, ZoneId::Exile))
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

// ── Card Definitions ──────────────────────────────────────────────────────────

/// A non-creature instant card with Recover {2} in the graveyard.
fn recover_card_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("test-recover-sorcery".into()),
        name: "Test Recover Sorcery".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Sorcery].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Draw a card. Recover {2}".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Recover),
            AbilityDefinition::Recover {
                cost: ManaCost {
                    generic: 2,
                    ..Default::default()
                },
            },
        ],
        ..Default::default()
    }
}

/// A creature card with Recover {1} (for testing self-Recover).
fn recover_creature_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("test-recover-creature".into()),
        name: "Test Recover Creature".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Recover {1}".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Recover),
            AbilityDefinition::Recover {
                cost: ManaCost {
                    generic: 1,
                    ..Default::default()
                },
            },
        ],
        power: Some(2),
        toughness: Some(2),
        ..Default::default()
    }
}

// ── ObjectSpecs ───────────────────────────────────────────────────────────────

/// A Recover sorcery already in p1's graveyard.
fn recover_sorcery_in_graveyard(owner: PlayerId) -> ObjectSpec {
    ObjectSpec::card(owner, "Test Recover Sorcery")
        .in_zone(ZoneId::Graveyard(owner))
        .with_card_id(CardId("test-recover-sorcery".into()))
        .with_keyword(KeywordAbility::Recover)
        .with_types(vec![CardType::Sorcery])
}

/// A 2/2 creature with lethal damage on the battlefield (dies to SBA).
fn dying_creature(owner: PlayerId, name: &str) -> ObjectSpec {
    ObjectSpec::creature(owner, name, 2, 2).with_damage(2)
}

// ── Test 1: Creature dies, Recover triggers ───────────────────────────────────

#[test]
/// CR 702.59a — When a creature is put into the Recover card owner's graveyard
/// from the battlefield, a RecoverTrigger is queued on the stack.
fn test_recover_basic_creature_dies_triggers_recover() {
    let p1 = p(1);
    let p2 = p(2);
    let registry = CardRegistry::new(vec![recover_card_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        // Recover sorcery in p1's graveyard.
        .object(recover_sorcery_in_graveyard(p1))
        // 2/2 creature with 2 damage — will die to SBA (CR 704.5g).
        .object(dying_creature(p1, "Dying Bear"))
        .build()
        .unwrap();

    state.turn.priority_holder = Some(p1);

    // Both players pass → handle_all_passed → SBAs fire → CreatureDied
    // → check_triggers queues RecoverTrigger → flush_pending_triggers puts it on stack.
    let (state, events) = pass_all(state, &[p1, p2]);

    // RecoverTrigger should be on the stack.
    assert!(
        state
            .stack_objects
            .iter()
            .any(|so| matches!(&so.kind, StackObjectKind::RecoverTrigger { .. })),
        "CR 702.59a: RecoverTrigger should be on the stack after creature dies"
    );

    // CreatureDied event should have been emitted.
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::CreatureDied { .. })),
        "CR 702.59a: CreatureDied event should be emitted"
    );
}

// ── Test 2: Pay recover cost → card returns to hand ──────────────────────────

#[test]
/// CR 702.59a — When the RecoverTrigger resolves and the player pays the cost,
/// the Recover card moves from graveyard to hand.
fn test_recover_pay_returns_to_hand() {
    let p1 = p(1);
    let p2 = p(2);
    let registry = CardRegistry::new(vec![recover_card_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(recover_sorcery_in_graveyard(p1))
        .object(dying_creature(p1, "Dying Bear"))
        .build()
        .unwrap();

    state.turn.priority_holder = Some(p1);

    // Both pass → SBAs → RecoverTrigger on stack.
    let (state, _) = pass_all(state, &[p1, p2]);

    // Confirm trigger is on stack.
    let trigger_on_stack = state
        .stack_objects
        .iter()
        .any(|so| matches!(&so.kind, StackObjectKind::RecoverTrigger { .. }));
    assert!(trigger_on_stack, "RecoverTrigger should be on the stack");

    // Confirm card is still in graveyard.
    assert!(
        in_graveyard(&state, "Test Recover Sorcery", p1),
        "Recover card should still be in graveyard before trigger resolves"
    );

    // Both pass → trigger resolves → RecoverPaymentRequired emitted.
    let (mut state, resolve_events) = pass_all(state, &[p1, p2]);

    assert!(
        resolve_events
            .iter()
            .any(|e| matches!(e, GameEvent::RecoverPaymentRequired { .. })),
        "CR 702.59a: RecoverPaymentRequired should be emitted when trigger resolves"
    );

    // Find the recover card id in graveyard.
    let recover_card_id = find_in_zone(&state, "Test Recover Sorcery", ZoneId::Graveyard(p1))
        .expect(
            "Recover card should still be in graveyard after trigger resolution (before payment)",
        );

    // Add mana to pay the cost ({2} generic).
    use mtg_engine::ManaColor;
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 2);

    // Player pays the recover cost.
    let (state, pay_events) = process_command(
        state,
        Command::PayRecover {
            player: p1,
            recover_card: recover_card_id,
            pay: true,
        },
    )
    .expect("PayRecover should succeed");

    // Card should be in hand.
    assert!(
        in_hand(&state, "Test Recover Sorcery", p1),
        "CR 702.59a: Recover card should be in hand after paying the recover cost"
    );

    // Should not be in graveyard.
    assert!(
        !in_graveyard(&state, "Test Recover Sorcery", p1),
        "CR 702.59a: Recover card should no longer be in graveyard after returning to hand"
    );

    // RecoverPaid event emitted.
    assert!(
        pay_events
            .iter()
            .any(|e| matches!(e, GameEvent::RecoverPaid { .. })),
        "CR 702.59a: RecoverPaid event should be emitted when player pays"
    );
}

// ── Test 3: Decline recover cost → card exiled ───────────────────────────────

#[test]
/// CR 702.59a — When the RecoverTrigger resolves and the player declines to pay,
/// the Recover card is exiled.
fn test_recover_decline_exiles_card() {
    let p1 = p(1);
    let p2 = p(2);
    let registry = CardRegistry::new(vec![recover_card_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(recover_sorcery_in_graveyard(p1))
        .object(dying_creature(p1, "Dying Bear"))
        .build()
        .unwrap();

    state.turn.priority_holder = Some(p1);

    // Both pass → SBAs → RecoverTrigger on stack.
    let (state, _) = pass_all(state, &[p1, p2]);

    // Both pass → trigger resolves → RecoverPaymentRequired.
    let (state, resolve_events) = pass_all(state, &[p1, p2]);

    assert!(
        resolve_events
            .iter()
            .any(|e| matches!(e, GameEvent::RecoverPaymentRequired { .. })),
        "CR 702.59a: RecoverPaymentRequired should be emitted"
    );

    let recover_card_id = find_in_zone(&state, "Test Recover Sorcery", ZoneId::Graveyard(p1))
        .expect("Recover card should be in graveyard before payment choice");

    // Player declines.
    let (state, decline_events) = process_command(
        state,
        Command::PayRecover {
            player: p1,
            recover_card: recover_card_id,
            pay: false,
        },
    )
    .expect("PayRecover(decline) should succeed");

    // Card should be in exile.
    assert!(
        in_exile(&state, "Test Recover Sorcery"),
        "CR 702.59a: Recover card should be in exile when player declines to pay"
    );

    // Should not be in graveyard.
    assert!(
        !in_graveyard(&state, "Test Recover Sorcery", p1),
        "CR 702.59a: Recover card should no longer be in graveyard after exile"
    );

    // RecoverDeclined event emitted.
    assert!(
        decline_events
            .iter()
            .any(|e| matches!(e, GameEvent::RecoverDeclined { .. })),
        "CR 702.59a: RecoverDeclined event should be emitted when player declines"
    );
}

// ── Test 4: Recover card left graveyard before trigger resolves → nothing ────

#[test]
/// CR 400.7 — If the Recover card is no longer in the graveyard when the
/// RecoverTrigger resolves, the trigger does nothing (fizzles).
fn test_recover_card_left_graveyard_fizzles() {
    let p1 = p(1);
    let p2 = p(2);
    let registry = CardRegistry::new(vec![recover_card_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(recover_sorcery_in_graveyard(p1))
        .object(dying_creature(p1, "Dying Bear"))
        .build()
        .unwrap();

    state.turn.priority_holder = Some(p1);

    // Both pass → SBAs → RecoverTrigger on stack.
    let (mut state, _) = pass_all(state, &[p1, p2]);

    // Manually move the Recover card from graveyard to exile (simulating another effect
    // that removed it — e.g., an opponent's Rest in Peace effect or Relic of Progenitus).
    let recover_id = find_in_zone(&state, "Test Recover Sorcery", ZoneId::Graveyard(p1))
        .expect("Recover card should be in graveyard");
    if let Some(obj) = state.objects.get_mut(&recover_id) {
        obj.zone = ZoneId::Exile;
    }

    // Confirm the card is gone from graveyard.
    assert!(
        !in_graveyard(&state, "Test Recover Sorcery", p1),
        "test setup: card should have been moved out of graveyard"
    );

    // Both pass → RecoverTrigger resolves.
    // CR 400.7: card is no longer in graveyard → trigger fizzles, no RecoverPaymentRequired.
    let (state, resolve_events) = pass_all(state, &[p1, p2]);

    assert!(
        !resolve_events
            .iter()
            .any(|e| matches!(e, GameEvent::RecoverPaymentRequired { .. })),
        "CR 400.7: RecoverPaymentRequired should NOT be emitted if card left graveyard"
    );

    // No pending recover payments.
    assert!(
        state.pending_recover_payments.is_empty(),
        "CR 400.7: no pending recover payments when trigger fizzles"
    );
}

// ── Test 5: Creature with Recover dies → its own Recover triggers ─────────────

#[test]
/// CR 702.59a — A creature with Recover that dies goes to the graveyard, and its
/// Recover ability triggers (referencing itself in the graveyard).
fn test_recover_creature_with_recover_dies_triggers_self() {
    let p1 = p(1);
    let p2 = p(2);
    let registry = CardRegistry::new(vec![recover_creature_def()]);

    // A 2/2 creature with Recover {1} on the battlefield, at lethal damage.
    let dying_recover_creature = ObjectSpec::creature(p1, "Test Recover Creature", 2, 2)
        .with_damage(2)
        .in_zone(ZoneId::Battlefield)
        .with_card_id(CardId("test-recover-creature".into()))
        .with_keyword(KeywordAbility::Recover)
        .with_types(vec![CardType::Creature]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(dying_recover_creature)
        .build()
        .unwrap();

    state.turn.priority_holder = Some(p1);

    // Both pass → SBA kills creature → creature goes to p1's graveyard with Recover
    // → Recover triggers on itself.
    let (state, events) = pass_all(state, &[p1, p2]);

    // CreatureDied should be emitted.
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::CreatureDied { .. })),
        "CR 702.59a: CreatureDied event should be emitted"
    );

    // RecoverTrigger should be on the stack.
    let has_recover_trigger = state
        .stack_objects
        .iter()
        .any(|so| matches!(&so.kind, StackObjectKind::RecoverTrigger { .. }));
    assert!(
        has_recover_trigger,
        "CR 702.59a: RecoverTrigger should be on the stack when creature with Recover dies"
    );
}

// ── Test 6: No Recover card in graveyard → no trigger ────────────────────────

#[test]
/// CR 702.59a — If there is no card with Recover in the graveyard when a creature
/// dies, no RecoverTrigger is queued.
fn test_recover_no_recover_card_in_graveyard_no_trigger() {
    let p1 = p(1);
    let p2 = p(2);

    // No Recover card in graveyard — just a vanilla creature dying.
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(dying_creature(p1, "Dying Bear"))
        .build()
        .unwrap();

    state.turn.priority_holder = Some(p1);

    let (state, _) = pass_all(state, &[p1, p2]);

    // No RecoverTrigger on the stack.
    assert!(
        !state
            .stack_objects
            .iter()
            .any(|so| matches!(&so.kind, StackObjectKind::RecoverTrigger { .. })),
        "CR 702.59a: no RecoverTrigger when there is no Recover card in graveyard"
    );
}

// ── Test 7: Opponent's creature dying does not trigger your Recover ────────────

#[test]
/// CR 702.59a — Recover only triggers when a creature goes into the Recover card
/// owner's graveyard. An opponent's creature dying does not trigger it.
fn test_recover_opponents_creature_death_no_trigger() {
    let p1 = p(1);
    let p2 = p(2);
    let registry = CardRegistry::new(vec![recover_card_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        // p1's Recover card in p1's graveyard.
        .object(recover_sorcery_in_graveyard(p1))
        // p2's creature dying (goes to p2's graveyard, not p1's).
        .object(dying_creature(p2, "Opponent Bear"))
        .build()
        .unwrap();

    state.turn.priority_holder = Some(p1);

    let (state, events) = pass_all(state, &[p1, p2]);

    // CreatureDied should be emitted (p2's creature died).
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::CreatureDied { .. })),
        "p2's creature should have died"
    );

    // No RecoverTrigger on the stack — p2's creature went to p2's graveyard, not p1's.
    assert!(
        !state
            .stack_objects
            .iter()
            .any(|so| matches!(&so.kind, StackObjectKind::RecoverTrigger { .. })),
        "CR 702.59a: Recover should not trigger when opponent's creature dies into opponent's graveyard"
    );
}

// ── Test 8: SBA-based approach: check_and_apply_sbas queues trigger ───────────

#[test]
/// CR 702.59a — When `check_and_apply_sbas` runs (from an external call), the
/// CreatureDied event causes check_triggers to queue a RecoverTrigger in
/// `state.pending_triggers` (ready to be flushed to the stack).
fn test_recover_sba_queues_pending_trigger() {
    let p1 = p(1);
    let p2 = p(2);
    let registry = CardRegistry::new(vec![recover_card_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(recover_sorcery_in_graveyard(p1))
        .object(dying_creature(p1, "Dying Bear"))
        .build()
        .unwrap();

    // Run SBAs directly.
    let events = check_and_apply_sbas(&mut state);

    // CreatureDied emitted.
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::CreatureDied { .. })),
        "CreatureDied should be emitted by check_and_apply_sbas"
    );

    // RecoverTrigger should be queued in pending_triggers.
    let has_recover_pending = state
        .pending_triggers
        .iter()
        .any(|t| t.kind == mtg_engine::state::stubs::PendingTriggerKind::Recover);
    assert!(
        has_recover_pending,
        "CR 702.59a: Recover PendingTrigger should be queued after creature dies via SBA"
    );
}
