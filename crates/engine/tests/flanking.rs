//! Flanking keyword ability enforcement tests (CR 702.25).
//!
//! Flanking is a triggered ability that triggers during the declare blockers step
//! (CR 702.25a): "Whenever this creature becomes blocked by a creature without
//! flanking, the blocking creature gets -1/-1 until end of turn."
//!
//! Tests:
//! - Basic case: flanking creature blocked by non-flanking creature gets -1/-1 (CR 702.25a)
//! - No trigger when blocked by a flanking creature (CR 702.25a)
//! - -1/-1 kills a 1/1 blocker via SBA (CR 702.25a + CR 704.5f)
//! - Multiple instances trigger separately (CR 702.25b)
//! - Multiple blockers each trigger separately (CR 702.25a + CR 509.3d)
//! - Effect expires at end of turn (CR 514.2)
//! - Multiplayer: triggers fire for blockers from different defending players (CR 702.25a)

use mtg_engine::{
    calculate_characteristics, process_command, AbilityDefinition, AttackTarget, CardDefinition,
    CardId, CardRegistry, CardType, Command, GameEvent, GameState, GameStateBuilder,
    KeywordAbility, ObjectId, ObjectSpec, PlayerId, Step, TypeLine, ZoneId,
};

// ── Helper: find object ID by name ───────────────────────────────────────────

fn find_object(state: &GameState, name: &str) -> ObjectId {
    state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{}' not found", name))
}

// ── Helper: pass all priority ─────────────────────────────────────────────────

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

// ── Helper: count objects in a graveyard ────────────────────────────────────

fn count_in_graveyard(state: &GameState, player: PlayerId) -> usize {
    state
        .objects
        .values()
        .filter(|o| o.zone == ZoneId::Graveyard(player))
        .count()
}

// ── CR 702.25: Flanking ───────────────────────────────────────────────────────

#[test]
/// CR 702.25a — Basic case: a creature with Flanking becomes blocked by a
/// creature without Flanking. The blocking creature gets -1/-1 until end of turn.
/// A 2/2 blocker should be 1/1 after the trigger resolves (before combat damage).
///
/// Engine timing: the flanking trigger fires during the DeclareBlockers step
/// and is flushed to the stack before priority is granted. The trigger resolves
/// before combat damage is dealt.
fn test_702_25_flanking_basic_minus_one_minus_one() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(
            ObjectSpec::creature(p1, "Flanking Attacker", 2, 2)
                .with_keyword(KeywordAbility::Flanking),
        )
        .object(ObjectSpec::creature(p2, "Blocker", 2, 2))
        .at_step(Step::DeclareAttackers)
        .active_player(p1)
        .build()
        .unwrap();

    let attacker_id = find_object(&state, "Flanking Attacker");
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

    // P2 blocks with the 2/2. The flanking trigger fires and is flushed to the
    // stack immediately (the engine flushes triggers after DeclareBlockers).
    let (state, declare_events) = process_command(
        state,
        Command::DeclareBlockers {
            player: p2,
            blockers: vec![(blocker_id, attacker_id)],
        },
    )
    .expect("declare blockers failed");

    // AbilityTriggered event should be in the DeclareBlockers events.
    let triggered = declare_events.iter().any(|e| {
        matches!(
            e,
            GameEvent::AbilityTriggered { source_object_id, .. }
            if *source_object_id == attacker_id
        )
    });
    assert!(
        triggered,
        "CR 702.25a: AbilityTriggered event should fire for flanking creature"
    );

    // Flanking trigger should be on the stack immediately after DeclareBlockers.
    assert_eq!(
        state.stack_objects.len(),
        1,
        "CR 702.25a: Flanking trigger should be on the stack after blockers declared"
    );

    // Resolve the trigger (both players pass). No combat damage yet (still in DeclareBlockers step).
    let (state, resolve_events) = pass_all(state, &[p1, p2]);

    // AbilityResolved event should have been emitted.
    let resolved = resolve_events
        .iter()
        .any(|e| matches!(e, GameEvent::AbilityResolved { .. }));
    assert!(
        resolved,
        "CR 702.25a: AbilityResolved event should fire after trigger resolves"
    );

    // The blocker should now have -1/-1 applied (2/2 - 1/-1 = 1/1).
    // This is still in DeclareBlockers step, no combat damage has been dealt yet.
    let blocker_chars = calculate_characteristics(&state, blocker_id)
        .expect("blocker should have calculable characteristics after trigger resolves");
    assert_eq!(
        blocker_chars.power,
        Some(1),
        "CR 702.25a: blocker power should be 1 after flanking trigger resolves"
    );
    assert_eq!(
        blocker_chars.toughness,
        Some(1),
        "CR 702.25a: blocker toughness should be 1 after flanking trigger resolves"
    );

    // Stack should be empty.
    assert!(
        state.stack_objects.is_empty(),
        "stack should be empty after flanking trigger resolves"
    );
}

#[test]
/// CR 702.25a — Flanking does NOT trigger when blocked by a creature that also
/// has flanking. "The blocking creature gets -1/-1" only applies to creatures
/// WITHOUT flanking.
fn test_702_25_flanking_does_not_trigger_on_flanking_blocker() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(
            ObjectSpec::creature(p1, "Flanking Attacker", 2, 2)
                .with_keyword(KeywordAbility::Flanking),
        )
        .object(
            ObjectSpec::creature(p2, "Flanking Blocker", 2, 2)
                .with_keyword(KeywordAbility::Flanking),
        )
        .at_step(Step::DeclareAttackers)
        .active_player(p1)
        .build()
        .unwrap();

    let attacker_id = find_object(&state, "Flanking Attacker");
    let blocker_id = find_object(&state, "Flanking Blocker");

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

    // P2 blocks with flanking creature — NO flanking trigger should fire.
    let (state, declare_events) = process_command(
        state,
        Command::DeclareBlockers {
            player: p2,
            blockers: vec![(blocker_id, attacker_id)],
        },
    )
    .expect("declare blockers failed");

    // No AbilityTriggered event for the flanking attacker.
    let triggered = declare_events.iter().any(|e| {
        matches!(
            e,
            GameEvent::AbilityTriggered { source_object_id, .. }
            if *source_object_id == attacker_id
        )
    });
    assert!(
        !triggered,
        "CR 702.25a: Flanking should NOT trigger when blocked by a creature WITH flanking"
    );

    // Stack should be empty — no flanking trigger fired.
    assert!(
        state.stack_objects.is_empty(),
        "stack should be empty — no flanking trigger fired"
    );

    // Blocker's P/T is unchanged (still 2/2).
    let blocker_chars = calculate_characteristics(&state, blocker_id)
        .expect("blocker should have calculable characteristics");
    assert_eq!(
        blocker_chars.power,
        Some(2),
        "CR 702.25a: blocker power unchanged when blocker also has flanking"
    );
    assert_eq!(
        blocker_chars.toughness,
        Some(2),
        "CR 702.25a: blocker toughness unchanged when blocker also has flanking"
    );
}

#[test]
/// CR 702.25a + CR 704.5f — A 1/1 blocker blocking a creature with Flanking gets
/// -1/-1, making it 0/0. SBAs destroy it (toughness 0 or less: CR 704.5f).
/// The attacker is still "blocked" (CR 509.1h) — no damage to the player.
fn test_702_25_flanking_kills_1_toughness_blocker() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(
            ObjectSpec::creature(p1, "Flanking Attacker", 2, 2)
                .with_keyword(KeywordAbility::Flanking),
        )
        .object(ObjectSpec::creature(p2, "Fragile Blocker", 1, 1))
        .at_step(Step::DeclareAttackers)
        .active_player(p1)
        .build()
        .unwrap();

    let attacker_id = find_object(&state, "Flanking Attacker");
    let blocker_id = find_object(&state, "Fragile Blocker");

    // P2 starts at 40 life.
    assert_eq!(
        state.players.get(&p2).unwrap().life_total,
        40,
        "setup: p2 starts at 40 life"
    );

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

    // P2 blocks with the 1/1. Flanking trigger fires immediately.
    let (state, declare_events) = process_command(
        state,
        Command::DeclareBlockers {
            player: p2,
            blockers: vec![(blocker_id, attacker_id)],
        },
    )
    .expect("declare blockers failed");

    // Trigger fired.
    let triggered = declare_events.iter().any(|e| {
        matches!(
            e,
            GameEvent::AbilityTriggered { source_object_id, .. }
            if *source_object_id == attacker_id
        )
    });
    assert!(triggered, "CR 702.25a: Flanking trigger should have fired");

    // Flanking trigger should be on the stack.
    assert_eq!(
        state.stack_objects.len(),
        1,
        "CR 702.25a: Flanking trigger should be on the stack"
    );

    // Resolve the trigger — blocker becomes 0/0, SBAs kill it (CR 704.5f).
    let (state, _) = pass_all(state, &[p1, p2]);

    // Blocker should be in the graveyard.
    assert_eq!(
        count_in_graveyard(&state, p2),
        1,
        "CR 702.25a + CR 704.5f: 1/1 blocker should die from flanking's -1/-1 (toughness 0)"
    );

    // Blocker object is no longer on the battlefield.
    assert!(
        !state
            .objects
            .values()
            .any(|o| o.id == blocker_id && o.zone == ZoneId::Battlefield),
        "CR 702.25a: blocker should not be on battlefield after dying from flanking"
    );

    // P2 still has 40 life — attacker is blocked, no damage to player without trample.
    // (Combat damage hasn't been dealt yet — we're still in DeclareBlockers step.)
    assert_eq!(
        state.players.get(&p2).unwrap().life_total,
        40,
        "P2 takes no damage in DeclareBlockers step (combat damage not yet dealt)"
    );
}

#[test]
/// CR 702.25b — Multiple instances of Flanking trigger separately.
/// A creature with Flanking x2 (two AbilityDefinition::Keyword(Flanking) entries)
/// generates TWO triggers per qualifying blocker.
/// A 3/3 blocker should be 1/1 after both triggers resolve.
fn test_702_25b_flanking_multiple_instances() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    // Build a CardDefinition with two Flanking keyword entries.
    // The `abilities` Vec (not the runtime OrdSet) is what keyword_count reads.
    let double_flanking_def = CardDefinition {
        card_id: CardId("double-flanking-creature".to_string()),
        name: "Double Flanking Creature".to_string(),
        mana_cost: None,
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Flanking\nFlanking".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flanking),
            AbilityDefinition::Keyword(KeywordAbility::Flanking),
        ],
        power: Some(2),
        toughness: Some(2),
        back_face: None,
    };

    let registry = CardRegistry::new(vec![double_flanking_def]);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        // Single creature with two Flanking instances via card_id.
        .object(
            ObjectSpec::creature(p1, "Double Flanking Creature", 2, 2)
                .with_keyword(KeywordAbility::Flanking)
                .with_card_id(CardId("double-flanking-creature".to_string())),
        )
        .object(ObjectSpec::creature(p2, "Tough Blocker", 3, 3))
        .at_step(Step::DeclareAttackers)
        .active_player(p1)
        .build()
        .unwrap();

    let attacker_id = find_object(&state, "Double Flanking Creature");
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

    // P2 blocks with the 3/3. TWO flanking triggers should fire.
    let (state, declare_events) = process_command(
        state,
        Command::DeclareBlockers {
            player: p2,
            blockers: vec![(blocker_id, attacker_id)],
        },
    )
    .expect("declare blockers failed");

    // Two AbilityTriggered events from the SAME creature (CR 702.25b).
    let triggered_count = declare_events
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
        triggered_count, 2,
        "CR 702.25b: a single creature with two Flanking entries should fire TWO triggers"
    );

    // Two items on the stack.
    assert_eq!(
        state.stack_objects.len(),
        2,
        "CR 702.25b: 2 flanking triggers on the stack from a single creature"
    );

    // Resolve first trigger — blocker should be 2/2.
    let (state, _) = pass_all(state, &[p1, p2]);
    assert_eq!(
        state.stack_objects.len(),
        1,
        "one trigger should remain after resolving the first"
    );
    let blocker_chars = calculate_characteristics(&state, blocker_id).unwrap();
    assert_eq!(
        blocker_chars.power,
        Some(2),
        "blocker should be 2 power after first flanking trigger"
    );
    assert_eq!(
        blocker_chars.toughness,
        Some(2),
        "blocker should be 2 toughness after first flanking trigger"
    );

    // Resolve second trigger — blocker should be 1/1.
    let (state, _) = pass_all(state, &[p1, p2]);
    assert!(
        state.stack_objects.is_empty(),
        "stack should be empty after both triggers resolve"
    );

    let blocker_chars = calculate_characteristics(&state, blocker_id).unwrap();
    assert_eq!(
        blocker_chars.power,
        Some(1),
        "CR 702.25b: blocker should be 1 power after two flanking triggers"
    );
    assert_eq!(
        blocker_chars.toughness,
        Some(1),
        "CR 702.25b: blocker should be 1 toughness after two flanking triggers"
    );
}

#[test]
/// CR 702.25a + CR 509.3d — Each qualifying blocker triggers flanking separately.
/// A flanking+menace attacker blocked by two non-flanking creatures generates two triggers.
fn test_702_25_flanking_multiple_blockers() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(
            ObjectSpec::creature(p1, "Flanking Menace Attacker", 3, 3)
                .with_keyword(KeywordAbility::Flanking)
                .with_keyword(KeywordAbility::Menace),
        )
        .object(ObjectSpec::creature(p2, "Blocker A", 2, 2))
        .object(ObjectSpec::creature(p2, "Blocker B", 2, 2))
        .at_step(Step::DeclareAttackers)
        .active_player(p1)
        .build()
        .unwrap();

    let attacker_id = find_object(&state, "Flanking Menace Attacker");
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

    // P2 blocks with both creatures (menace requires >= 2 blockers).
    // Two flanking triggers should fire immediately — one per qualifying blocker.
    let (state, declare_events) = process_command(
        state,
        Command::DeclareBlockers {
            player: p2,
            blockers: vec![(blocker_a, attacker_id), (blocker_b, attacker_id)],
        },
    )
    .expect("declare blockers failed");

    let triggered_count = declare_events
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
        triggered_count, 2,
        "CR 702.25a + CR 509.3d: two triggers should fire for two qualifying blockers"
    );

    assert_eq!(
        state.stack_objects.len(),
        2,
        "CR 702.25a: 2 flanking triggers on the stack (one per blocker)"
    );

    // Resolve first trigger.
    let (state, _) = pass_all(state, &[p1, p2]);
    assert_eq!(state.stack_objects.len(), 1, "one trigger should remain");

    // Resolve second trigger.
    let (state, _) = pass_all(state, &[p1, p2]);
    assert!(
        state.stack_objects.is_empty(),
        "stack should be empty after both triggers resolve"
    );

    // Each blocker should now be 1/1.
    let chars_a = calculate_characteristics(&state, blocker_a).unwrap();
    assert_eq!(
        chars_a.power,
        Some(1),
        "CR 702.25a: Blocker A power should be 1 after flanking trigger"
    );
    assert_eq!(
        chars_a.toughness,
        Some(1),
        "CR 702.25a: Blocker A toughness should be 1 after flanking trigger"
    );

    let chars_b = calculate_characteristics(&state, blocker_b).unwrap();
    assert_eq!(
        chars_b.power,
        Some(1),
        "CR 702.25a: Blocker B power should be 1 after flanking trigger"
    );
    assert_eq!(
        chars_b.toughness,
        Some(1),
        "CR 702.25a: Blocker B toughness should be 1 after flanking trigger"
    );
}

#[test]
/// CR 702.25a + CR 514.2 — The -1/-1 is "until end of turn" (UntilEndOfTurn).
/// After expire_end_of_turn_effects is called (simulating cleanup), the blocker
/// returns to its original P/T.
fn test_702_25_flanking_effect_expires_at_end_of_turn() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(
            ObjectSpec::creature(p1, "Flanking Attacker", 2, 2)
                .with_keyword(KeywordAbility::Flanking),
        )
        .object(ObjectSpec::creature(p2, "Blocker", 2, 2))
        .at_step(Step::DeclareAttackers)
        .active_player(p1)
        .build()
        .unwrap();

    let attacker_id = find_object(&state, "Flanking Attacker");
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

    // P2 blocks. Flanking trigger fires immediately.
    let (state, _) = process_command(
        state,
        Command::DeclareBlockers {
            player: p2,
            blockers: vec![(blocker_id, attacker_id)],
        },
    )
    .expect("declare blockers failed");

    assert_eq!(
        state.stack_objects.len(),
        1,
        "trigger should be on stack after DeclareBlockers"
    );

    // Resolve trigger — blocker becomes 1/1.
    let (state, _) = pass_all(state, &[p1, p2]);
    assert!(state.stack_objects.is_empty(), "stack should be empty");

    // Blocker should be 1/1 now (after trigger resolved).
    let chars = calculate_characteristics(&state, blocker_id)
        .expect("blocker should be alive — no combat damage yet");
    assert_eq!(
        chars.power,
        Some(1),
        "CR 702.25a: blocker should be 1/1 after flanking trigger resolves"
    );
    assert_eq!(
        chars.toughness,
        Some(1),
        "CR 702.25a: blocker should be 1/1 after flanking trigger resolves"
    );

    // Simulate cleanup: expire all UntilEndOfTurn effects (CR 514.2).
    let mut state = state;
    mtg_engine::rules::layers::expire_end_of_turn_effects(&mut state);

    // After cleanup, blocker is back to its original 2/2.
    let chars = calculate_characteristics(&state, blocker_id).unwrap();
    assert_eq!(
        chars.power,
        Some(2),
        "CR 514.2: flanking -1/-1 should expire at cleanup, returning blocker to 2 power"
    );
    assert_eq!(
        chars.toughness,
        Some(2),
        "CR 514.2: flanking -1/-1 should expire at cleanup, returning blocker to 2 toughness"
    );
}

#[test]
/// CR 702.25a — In a 4-player game, flanking triggers fire correctly for blockers
/// from different defending players.
/// P1 attacks P2 and P3 with different flanking creatures. Each declares a blocker.
/// Both flanking triggers fire and each blocker gets -1/-1.
fn test_702_25_flanking_multiplayer() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);
    let p3 = PlayerId(3);
    let p4 = PlayerId(4);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .add_player(p4)
        .object(
            ObjectSpec::creature(p1, "Flanking Attacker A", 2, 2)
                .with_keyword(KeywordAbility::Flanking),
        )
        .object(
            ObjectSpec::creature(p1, "Flanking Attacker B", 2, 2)
                .with_keyword(KeywordAbility::Flanking),
        )
        .object(ObjectSpec::creature(p2, "P2 Blocker", 2, 2))
        .object(ObjectSpec::creature(p3, "P3 Blocker", 2, 2))
        .at_step(Step::DeclareAttackers)
        .active_player(p1)
        .build()
        .unwrap();

    let attacker_a = find_object(&state, "Flanking Attacker A");
    let attacker_b = find_object(&state, "Flanking Attacker B");
    let p2_blocker = find_object(&state, "P2 Blocker");
    let p3_blocker = find_object(&state, "P3 Blocker");

    // Declare attackers: A attacks P2, B attacks P3.
    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![
                (attacker_a, AttackTarget::Player(p2)),
                (attacker_b, AttackTarget::Player(p3)),
            ],
            enlist_choices: vec![],
        },
    )
    .expect("declare attackers failed");

    // Advance to DeclareBlockers.
    let (state, _) = pass_all(state, &[p1, p2, p3, p4]);

    // P2 blocks Attacker A. One flanking trigger fires.
    let (state, p2_declare_events) = process_command(
        state,
        Command::DeclareBlockers {
            player: p2,
            blockers: vec![(p2_blocker, attacker_a)],
        },
    )
    .expect("p2 declare blockers failed");

    // P3 blocks Attacker B. One more flanking trigger fires.
    let (state, p3_declare_events) = process_command(
        state,
        Command::DeclareBlockers {
            player: p3,
            blockers: vec![(p3_blocker, attacker_b)],
        },
    )
    .expect("p3 declare blockers failed");

    // Combine events from both DeclareBlockers calls.
    let all_declare_events: Vec<_> = p2_declare_events
        .iter()
        .chain(p3_declare_events.iter())
        .collect();

    // Two AbilityTriggered events total (one for each attacker-blocker pair).
    let triggered_count = all_declare_events
        .iter()
        .filter(|e| {
            matches!(
                e,
                GameEvent::AbilityTriggered { source_object_id, .. }
                if *source_object_id == attacker_a || *source_object_id == attacker_b
            )
        })
        .count();
    assert_eq!(
        triggered_count, 2,
        "CR 702.25a: 2 flanking triggers should fire (one for P2's blocker, one for P3's)"
    );

    // Two triggers on the stack (one from each defender's DeclareBlockers).
    assert_eq!(
        state.stack_objects.len(),
        2,
        "CR 702.25a: 2 flanking triggers on the stack"
    );

    // Resolve first trigger.
    let (state, _) = pass_all(state, &[p1, p2, p3, p4]);
    assert_eq!(
        state.stack_objects.len(),
        1,
        "one trigger should remain after first resolves"
    );

    // Resolve second trigger.
    let (state, _) = pass_all(state, &[p1, p2, p3, p4]);
    assert!(
        state.stack_objects.is_empty(),
        "stack should be empty after both triggers resolve"
    );

    // Both blockers should now be 1/1.
    let p2_blocker_chars = calculate_characteristics(&state, p2_blocker).unwrap();
    assert_eq!(
        p2_blocker_chars.power,
        Some(1),
        "CR 702.25a: P2's blocker should be 1 power after flanking trigger"
    );
    assert_eq!(
        p2_blocker_chars.toughness,
        Some(1),
        "CR 702.25a: P2's blocker should be 1 toughness after flanking trigger"
    );

    let p3_blocker_chars = calculate_characteristics(&state, p3_blocker).unwrap();
    assert_eq!(
        p3_blocker_chars.power,
        Some(1),
        "CR 702.25a: P3's blocker should be 1 power after flanking trigger"
    );
    assert_eq!(
        p3_blocker_chars.toughness,
        Some(1),
        "CR 702.25a: P3's blocker should be 1 toughness after flanking trigger"
    );
}
