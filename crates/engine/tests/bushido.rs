//! Bushido keyword ability tests (CR 702.45).
//!
//! Bushido is a triggered ability: "Whenever this creature blocks or becomes
//! blocked, it gets +N/+N until end of turn." (CR 702.45a)
//!
//! Key rules verified:
//! - Blocker with Bushido triggers on SelfBlocks and gets +N/+N (CR 702.45a).
//! - Attacker with Bushido triggers on SelfBecomesBlocked and gets +N/+N (CR 702.45a).
//! - A creature cannot both block AND become blocked — exactly one trigger per combat.
//! - Multiple instances of Bushido each trigger separately (CR 702.45b).
//! - The bonus expires at end of turn (CR 514.2).
//! - Attacker blocked by multiple creatures triggers only once (CR 509.3c).
//! - Multiplayer: triggers fire for creatures across defenders (CR 702.45a).

use mtg_engine::{
    calculate_characteristics, process_command, AbilityDefinition, AttackTarget, CardDefinition,
    CardId, CardRegistry, CardType, Command, GameEvent, GameState, GameStateBuilder,
    KeywordAbility, ObjectId, ObjectSpec, PlayerId, Step, TypeLine, ZoneId,
};

// ── Helpers ────────────────────────────────────────────────────────────────────

fn find_object(state: &GameState, name: &str) -> ObjectId {
    state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{}' not found", name))
}

fn pass_all(state: GameState, players: &[PlayerId]) -> (GameState, Vec<GameEvent>) {
    let mut all_events = Vec::new();
    let mut current = state;
    for &p in players {
        let (s, ev) = process_command(current, Command::PassPriority { player: p })
            .unwrap_or_else(|e| panic!("PassPriority by {:?} failed: {:?}", p, e));
        current = s;
        all_events.extend(ev);
    }
    (current, all_events)
}

// ── Test 1: Bushido blocker gets +N/+N ────────────────────────────────────────

#[test]
/// CR 702.45a — A creature with Bushido 1 that blocks gets +1/+1 until end of
/// turn when the trigger resolves. The blocker fires on SelfBlocks.
fn test_702_45a_bushido_blocker_gets_bonus() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(ObjectSpec::creature(p1, "Attacker", 3, 3))
        .object(
            ObjectSpec::creature(p2, "Bushido Blocker", 2, 2)
                .with_keyword(KeywordAbility::Bushido(1)),
        )
        .at_step(Step::DeclareAttackers)
        .active_player(p1)
        .build()
        .unwrap();

    let attacker_id = find_object(&state, "Attacker");
    let blocker_id = find_object(&state, "Bushido Blocker");

    // Declare attacker.
    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(attacker_id, AttackTarget::Player(p2))],
            enlist_choices: vec![],
        },
    )
    .expect("declare attackers failed");

    // Advance to DeclareBlockers.
    let (state, _) = pass_all(state, &[p1, p2]);

    // P2 blocks with the Bushido creature. The SelfBlocks trigger fires.
    let (state, declare_events) = process_command(
        state,
        Command::DeclareBlockers {
            player: p2,
            blockers: vec![(blocker_id, attacker_id)],
        },
    )
    .expect("declare blockers failed");

    // AbilityTriggered event from the blocker.
    let triggered = declare_events.iter().any(|e| {
        matches!(
            e,
            GameEvent::AbilityTriggered { source_object_id, .. }
            if *source_object_id == blocker_id
        )
    });
    assert!(
        triggered,
        "CR 702.45a: AbilityTriggered should fire for the Bushido blocker"
    );

    // One trigger on the stack.
    assert_eq!(
        state.stack_objects.len(),
        1,
        "CR 702.45a: Bushido blocker trigger should be on the stack"
    );

    // Resolve the trigger.
    let (state, _) = pass_all(state, &[p1, p2]);

    // Bushido Blocker is now 3/3 (2+1 / 2+1).
    let chars = calculate_characteristics(&state, blocker_id)
        .expect("Bushido Blocker should still be on battlefield");
    assert_eq!(
        chars.power,
        Some(3),
        "CR 702.45a: blocker power should be 3 after Bushido 1 trigger resolves"
    );
    assert_eq!(
        chars.toughness,
        Some(3),
        "CR 702.45a: blocker toughness should be 3 after Bushido 1 trigger resolves"
    );
}

// ── Test 2: Bushido attacker becomes blocked ───────────────────────────────────

#[test]
/// CR 702.45a — A creature with Bushido 1 that attacks and becomes blocked gets
/// +1/+1 until end of turn when the trigger resolves. The attacker fires on
/// SelfBecomesBlocked.
fn test_702_45a_bushido_attacker_becomes_blocked() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(
            ObjectSpec::creature(p1, "Bushido Attacker", 2, 2)
                .with_keyword(KeywordAbility::Bushido(1)),
        )
        .object(ObjectSpec::creature(p2, "Blocker", 3, 3))
        .at_step(Step::DeclareAttackers)
        .active_player(p1)
        .build()
        .unwrap();

    let attacker_id = find_object(&state, "Bushido Attacker");
    let blocker_id = find_object(&state, "Blocker");

    // Declare attacker.
    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(attacker_id, AttackTarget::Player(p2))],
            enlist_choices: vec![],
        },
    )
    .expect("declare attackers failed");

    // Advance to DeclareBlockers.
    let (state, _) = pass_all(state, &[p1, p2]);

    // P2 blocks with a 3/3. The Bushido attacker's SelfBecomesBlocked trigger fires.
    let (state, declare_events) = process_command(
        state,
        Command::DeclareBlockers {
            player: p2,
            blockers: vec![(blocker_id, attacker_id)],
        },
    )
    .expect("declare blockers failed");

    // AbilityTriggered event from the attacker.
    let triggered = declare_events.iter().any(|e| {
        matches!(
            e,
            GameEvent::AbilityTriggered { source_object_id, .. }
            if *source_object_id == attacker_id
        )
    });
    assert!(
        triggered,
        "CR 702.45a: AbilityTriggered should fire for the Bushido attacker"
    );

    // One trigger on the stack.
    assert_eq!(
        state.stack_objects.len(),
        1,
        "CR 702.45a: Bushido attacker trigger should be on the stack"
    );

    // Resolve the trigger.
    let (state, _) = pass_all(state, &[p1, p2]);

    // Bushido Attacker is now 3/3 (2+1 / 2+1).
    let chars = calculate_characteristics(&state, attacker_id)
        .expect("Bushido Attacker should still be on battlefield");
    assert_eq!(
        chars.power,
        Some(3),
        "CR 702.45a: attacker power should be 3 after Bushido 1 trigger resolves"
    );
    assert_eq!(
        chars.toughness,
        Some(3),
        "CR 702.45a: attacker toughness should be 3 after Bushido 1 trigger resolves"
    );
}

// ── Test 3: Bushido does not double-trigger for same creature ─────────────────

#[test]
/// CR 702.45a — A creature either blocks (SelfBlocks fires) OR becomes blocked
/// (SelfBecomesBlocked fires), never both. When a Bushido attacker is blocked by
/// a Bushido blocker, each gets exactly one trigger (not two each).
fn test_702_45a_bushido_does_not_double_trigger() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(
            ObjectSpec::creature(p1, "Bushido Attacker", 2, 2)
                .with_keyword(KeywordAbility::Bushido(1)),
        )
        .object(
            ObjectSpec::creature(p2, "Bushido Blocker", 1, 1)
                .with_keyword(KeywordAbility::Bushido(1)),
        )
        .at_step(Step::DeclareAttackers)
        .active_player(p1)
        .build()
        .unwrap();

    let attacker_id = find_object(&state, "Bushido Attacker");
    let blocker_id = find_object(&state, "Bushido Blocker");

    // Declare attacker.
    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(attacker_id, AttackTarget::Player(p2))],
            enlist_choices: vec![],
        },
    )
    .expect("declare attackers failed");

    // Advance to DeclareBlockers.
    let (state, _) = pass_all(state, &[p1, p2]);

    // P2 blocks with Bushido blocker. Each creature triggers ONCE.
    let (state, declare_events) = process_command(
        state,
        Command::DeclareBlockers {
            player: p2,
            blockers: vec![(blocker_id, attacker_id)],
        },
    )
    .expect("declare blockers failed");

    // Exactly 2 AbilityTriggered events: one for attacker, one for blocker.
    let triggered_count = declare_events
        .iter()
        .filter(|e| matches!(e, GameEvent::AbilityTriggered { .. }))
        .count();
    assert_eq!(
        triggered_count, 2,
        "CR 702.45a: exactly 2 triggers (attacker + blocker), not 4"
    );

    // Attacker triggers on SelfBecomesBlocked, blocker triggers on SelfBlocks.
    let attacker_triggered = declare_events.iter().any(|e| {
        matches!(
            e,
            GameEvent::AbilityTriggered { source_object_id, .. }
            if *source_object_id == attacker_id
        )
    });
    let blocker_triggered = declare_events.iter().any(|e| {
        matches!(
            e,
            GameEvent::AbilityTriggered { source_object_id, .. }
            if *source_object_id == blocker_id
        )
    });
    assert!(
        attacker_triggered,
        "CR 702.45a: attacker should trigger once"
    );
    assert!(blocker_triggered, "CR 702.45a: blocker should trigger once");

    // Exactly 2 items on the stack.
    assert_eq!(
        state.stack_objects.len(),
        2,
        "CR 702.45a: exactly 2 triggers on the stack"
    );
}

// ── Test 4: Multiple Bushido instances trigger separately ─────────────────────

#[test]
/// CR 702.45b — Multiple instances of Bushido on the same creature each trigger
/// separately. A creature with Bushido 1 and Bushido 2 should get two separate
/// triggers, resulting in +1/+1 + +2/+2 = +3/+3 total.
fn test_702_45b_bushido_multiple_instances() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    // Build a CardDefinition with two Bushido entries (Bushido 1 + Bushido 2).
    let double_bushido_def = CardDefinition {
        card_id: CardId("double-bushido-creature".to_string()),
        name: "Double Bushido Creature".to_string(),
        mana_cost: None,
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Bushido 1\nBushido 2".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Bushido(1)),
            AbilityDefinition::Keyword(KeywordAbility::Bushido(2)),
        ],
        power: Some(1),
        toughness: Some(1),
        color_indicator: None,
        back_face: None,
        spell_cost_modifiers: vec![],
        self_cost_reduction: None,
        starting_loyalty: None,
        adventure_face: None,
        meld_pair: None,
        activated_ability_cost_reductions: vec![],
    };

    let registry = CardRegistry::new(vec![double_bushido_def]);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(
            ObjectSpec::creature(p1, "Double Bushido Creature", 1, 1)
                .with_keyword(KeywordAbility::Bushido(1))
                .with_keyword(KeywordAbility::Bushido(2))
                .with_card_id(CardId("double-bushido-creature".to_string())),
        )
        .object(ObjectSpec::creature(p2, "Tough Blocker", 5, 5))
        .at_step(Step::DeclareAttackers)
        .active_player(p1)
        .build()
        .unwrap();

    let attacker_id = find_object(&state, "Double Bushido Creature");
    let blocker_id = find_object(&state, "Tough Blocker");

    // Declare attacker.
    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(attacker_id, AttackTarget::Player(p2))],
            enlist_choices: vec![],
        },
    )
    .expect("declare attackers failed");

    // Advance to DeclareBlockers.
    let (state, _) = pass_all(state, &[p1, p2]);

    // P2 blocks. Two SelfBecomesBlocked triggers from the attacker's two Bushido instances.
    let (state, declare_events) = process_command(
        state,
        Command::DeclareBlockers {
            player: p2,
            blockers: vec![(blocker_id, attacker_id)],
        },
    )
    .expect("declare blockers failed");

    // 2 AbilityTriggered events from the attacker alone (one per Bushido instance).
    let attacker_triggered_count = declare_events
        .iter()
        .filter(|e| {
            matches!(
                e,
                GameEvent::AbilityTriggered { source_object_id, .. }
                if *source_object_id == attacker_id
            )
        })
        .count();
    assert_eq!(
        attacker_triggered_count, 2,
        "CR 702.45b: two Bushido instances should generate two separate triggers"
    );

    // 2 triggers on the stack from the attacker (plus 0 from the blocker which has no Bushido).
    assert_eq!(
        state.stack_objects.len(),
        2,
        "CR 702.45b: 2 Bushido triggers on the stack from the attacker"
    );

    // Resolve first trigger.
    let (state, _) = pass_all(state, &[p1, p2]);
    assert_eq!(state.stack_objects.len(), 1, "one trigger should remain");

    // Resolve second trigger.
    let (state, _) = pass_all(state, &[p1, p2]);
    assert!(state.stack_objects.is_empty(), "stack should be empty");

    // Attacker should be 4/4 (1 + 1 + 2 = 4 power/toughness).
    let chars = calculate_characteristics(&state, attacker_id)
        .expect("attacker should still be on battlefield");
    assert_eq!(
        chars.power,
        Some(4),
        "CR 702.45b: attacker power should be 4 (1 base + 1 + 2 from Bushido)"
    );
    assert_eq!(
        chars.toughness,
        Some(4),
        "CR 702.45b: attacker toughness should be 4 (1 base + 1 + 2 from Bushido)"
    );
}

// ── Test 5: Bushido bonus expires at end of turn ──────────────────────────────

#[test]
/// CR 702.45a ("until end of turn"), CR 514.2 — The +N/+N from Bushido expires
/// during the Cleanup step. After expire_end_of_turn_effects, the creature
/// returns to its printed P/T.
fn test_702_45a_bushido_bonus_expires_eot() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(ObjectSpec::creature(p1, "Attacker", 3, 3))
        .object(
            ObjectSpec::creature(p2, "Bushido Blocker", 2, 2)
                .with_keyword(KeywordAbility::Bushido(1)),
        )
        .at_step(Step::DeclareAttackers)
        .active_player(p1)
        .build()
        .unwrap();

    let attacker_id = find_object(&state, "Attacker");
    let blocker_id = find_object(&state, "Bushido Blocker");

    // Declare attacker.
    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(attacker_id, AttackTarget::Player(p2))],
            enlist_choices: vec![],
        },
    )
    .expect("declare attackers failed");

    // Advance to DeclareBlockers.
    let (state, _) = pass_all(state, &[p1, p2]);

    // P2 blocks. Bushido trigger fires.
    let (state, _) = process_command(
        state,
        Command::DeclareBlockers {
            player: p2,
            blockers: vec![(blocker_id, attacker_id)],
        },
    )
    .expect("declare blockers failed");

    // Resolve trigger — blocker is now 3/3.
    let (state, _) = pass_all(state, &[p1, p2]);

    let chars = calculate_characteristics(&state, blocker_id).unwrap();
    assert_eq!(
        chars.power,
        Some(3),
        "Bushido effect active — blocker should be 3/3 before cleanup"
    );
    assert_eq!(
        chars.toughness,
        Some(3),
        "Bushido effect active — blocker should be 3/3 before cleanup"
    );

    // Simulate cleanup: expire all UntilEndOfTurn effects (CR 514.2).
    let mut state = state;
    mtg_engine::rules::layers::expire_end_of_turn_effects(&mut state);

    // After cleanup, blocker returns to 2/2.
    let chars = calculate_characteristics(&state, blocker_id).unwrap();
    assert_eq!(
        chars.power,
        Some(2),
        "CR 514.2: Bushido +1 power should expire at cleanup, returning blocker to 2"
    );
    assert_eq!(
        chars.toughness,
        Some(2),
        "CR 514.2: Bushido +1 toughness should expire at cleanup, returning blocker to 2"
    );
}

// ── Test 6: Attacker blocked by multiple creatures triggers only once ──────────

#[test]
/// CR 509.3c — An attacking creature with Bushido becomes blocked once, even if
/// multiple creatures are declared as blockers. The SelfBecomesBlocked trigger
/// should fire exactly once regardless of the number of blockers.
fn test_702_45a_bushido_attacker_blocked_by_multiple() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(
            ObjectSpec::creature(p1, "Bushido Attacker", 3, 3)
                .with_keyword(KeywordAbility::Bushido(1))
                .with_keyword(KeywordAbility::Menace),
        )
        .object(ObjectSpec::creature(p2, "Blocker A", 1, 1))
        .object(ObjectSpec::creature(p2, "Blocker B", 1, 1))
        .at_step(Step::DeclareAttackers)
        .active_player(p1)
        .build()
        .unwrap();

    let attacker_id = find_object(&state, "Bushido Attacker");
    let blocker_a = find_object(&state, "Blocker A");
    let blocker_b = find_object(&state, "Blocker B");

    // Declare attacker.
    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(attacker_id, AttackTarget::Player(p2))],
            enlist_choices: vec![],
        },
    )
    .expect("declare attackers failed");

    // Advance to DeclareBlockers.
    let (state, _) = pass_all(state, &[p1, p2]);

    // P2 blocks with TWO creatures (Menace requires >= 2 blockers).
    // The attacker should trigger only ONCE (it becomes blocked once).
    let (state, declare_events) = process_command(
        state,
        Command::DeclareBlockers {
            player: p2,
            blockers: vec![(blocker_a, attacker_id), (blocker_b, attacker_id)],
        },
    )
    .expect("declare blockers failed");

    // Exactly ONE AbilityTriggered event from the Bushido attacker.
    let attacker_triggered_count = declare_events
        .iter()
        .filter(|e| {
            matches!(
                e,
                GameEvent::AbilityTriggered { source_object_id, .. }
                if *source_object_id == attacker_id
            )
        })
        .count();
    assert_eq!(
        attacker_triggered_count, 1,
        "CR 509.3c: Bushido attacker blocked by 2 creatures should trigger only ONCE"
    );
}

// ── Test 7: Multiplayer — triggers fire across defenders ───────────────────────

#[test]
/// CR 702.45a, multiplayer — In a 4-player game, Bushido triggers fire correctly
/// when blocking creatures from different defending players are involved.
/// P1 attacks P2 and P3 with Bushido creatures. Both defenders block.
/// Both Bushido triggers fire.
fn test_702_45a_bushido_multiplayer() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);
    let p3 = PlayerId(3);
    let p4 = PlayerId(4);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .add_player(p4)
        // P1 attacks P2 with a Bushido 1 creature.
        .object(
            ObjectSpec::creature(p1, "Bushido Attacker A", 2, 2)
                .with_keyword(KeywordAbility::Bushido(1)),
        )
        // P1 attacks P3 with a non-Bushido creature.
        .object(ObjectSpec::creature(p1, "Plain Attacker B", 2, 2))
        // P2 blocks with a Bushido 1 creature.
        .object(
            ObjectSpec::creature(p2, "P2 Bushido Blocker", 1, 1)
                .with_keyword(KeywordAbility::Bushido(1)),
        )
        // P3 blocks with a plain creature.
        .object(ObjectSpec::creature(p3, "P3 Plain Blocker", 1, 1))
        .at_step(Step::DeclareAttackers)
        .active_player(p1)
        .build()
        .unwrap();

    let attacker_a = find_object(&state, "Bushido Attacker A");
    let plain_attacker_b = find_object(&state, "Plain Attacker B");
    let p2_blocker = find_object(&state, "P2 Bushido Blocker");
    let p3_blocker = find_object(&state, "P3 Plain Blocker");

    // Declare attackers: A attacks P2, B attacks P3.
    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![
                (attacker_a, AttackTarget::Player(p2)),
                (plain_attacker_b, AttackTarget::Player(p3)),
            ],
            enlist_choices: vec![],
        },
    )
    .expect("declare attackers failed");

    // Advance to DeclareBlockers.
    let (state, _) = pass_all(state, &[p1, p2, p3, p4]);

    // P2 blocks Attacker A with their Bushido creature.
    // Two Bushido triggers: attacker_a (SelfBecomesBlocked) + p2_blocker (SelfBlocks).
    let (state, p2_declare_events) = process_command(
        state,
        Command::DeclareBlockers {
            player: p2,
            blockers: vec![(p2_blocker, attacker_a)],
        },
    )
    .expect("p2 declare blockers failed");

    // P3 blocks Plain Attacker B with a plain creature (no Bushido triggers).
    let (state, p3_declare_events) = process_command(
        state,
        Command::DeclareBlockers {
            player: p3,
            blockers: vec![(p3_blocker, plain_attacker_b)],
        },
    )
    .expect("p3 declare blockers failed");

    // P2's declaration should generate 2 Bushido triggers.
    let p2_trigger_count = p2_declare_events
        .iter()
        .filter(|e| matches!(e, GameEvent::AbilityTriggered { .. }))
        .count();
    assert_eq!(
        p2_trigger_count, 2,
        "CR 702.45a multiplayer: P2's DeclareBlockers should fire 2 Bushido triggers \
         (attacker_a SelfBecomesBlocked + p2_blocker SelfBlocks)"
    );

    // P3's declaration should generate 0 Bushido triggers (no Bushido in that combat).
    let p3_trigger_count = p3_declare_events
        .iter()
        .filter(|e| matches!(e, GameEvent::AbilityTriggered { .. }))
        .count();
    assert_eq!(
        p3_trigger_count, 0,
        "CR 702.45a multiplayer: P3's DeclareBlockers should fire 0 Bushido triggers"
    );

    // Two triggers on the stack total.
    assert_eq!(
        state.stack_objects.len(),
        2,
        "CR 702.45a multiplayer: 2 Bushido triggers on the stack after both declare-blockers"
    );

    // Resolve both triggers.
    let (state, _) = pass_all(state, &[p1, p2, p3, p4]);
    let (state, _) = pass_all(state, &[p1, p2, p3, p4]);

    assert!(
        state.stack_objects.is_empty(),
        "stack should be empty after both triggers resolve"
    );

    // attacker_a should be 3/3 (2+1 / 2+1) from its Bushido trigger.
    let attacker_a_chars = calculate_characteristics(&state, attacker_a)
        .expect("Bushido Attacker A should still be on battlefield");
    assert_eq!(
        attacker_a_chars.power,
        Some(3),
        "CR 702.45a multiplayer: Bushido Attacker A power should be 3 after trigger"
    );
    assert_eq!(
        attacker_a_chars.toughness,
        Some(3),
        "CR 702.45a multiplayer: Bushido Attacker A toughness should be 3 after trigger"
    );

    // p2_blocker should be 2/2 (1+1 / 1+1) from its Bushido trigger.
    let p2_blocker_chars = calculate_characteristics(&state, p2_blocker)
        .expect("P2 Bushido Blocker should still be on battlefield");
    assert_eq!(
        p2_blocker_chars.power,
        Some(2),
        "CR 702.45a multiplayer: P2 Bushido Blocker power should be 2 after trigger"
    );
    assert_eq!(
        p2_blocker_chars.toughness,
        Some(2),
        "CR 702.45a multiplayer: P2 Bushido Blocker toughness should be 2 after trigger"
    );
}
