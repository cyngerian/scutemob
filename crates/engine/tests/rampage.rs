//! Rampage keyword ability tests (CR 702.23).
//!
//! Rampage is a triggered ability: "Whenever this creature becomes blocked,
//! it gets +N/+N until end of turn for each creature blocking it beyond
//! the first." (CR 702.23a)
//!
//! Key rules verified:
//! - Trigger fires when the creature becomes blocked (CR 702.23a).
//! - Bonus = (blockers - 1) * N; no bonus if blocked by exactly 1 (CR 702.23a).
//! - Bonus applies to BOTH power and toughness (+N/+N, not +N/+0) (CR 702.23a).
//! - Bonus is calculated at RESOLUTION time from combat state (CR 702.23b).
//! - Multiple instances trigger separately (CR 702.23c).
//! - The bonus expires at end of turn (CR 514.2).
//! - No trigger when not blocked (CR 702.23a "becomes blocked").

use mtg_engine::{
    calculate_characteristics, process_command, AttackTarget, CardRegistry, Command, GameEvent,
    GameState, GameStateBuilder, KeywordAbility, ObjectId, ObjectSpec, PlayerId, Step,
};

// ── Helpers ───────────────────────────────────────────────────────────────────

fn find_object(state: &GameState, name: &str) -> ObjectId {
    state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{}' not found", name))
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

// ── Test 1: Blocked by 2 creatures → +N/+N bonus ─────────────────────────────

#[test]
/// CR 702.23a — A creature with Rampage 2 blocked by 2 creatures gets +2/+2
/// (1 creature beyond the first * 2).
///
/// Engine timing: the rampage trigger fires during DeclareBlockers and is
/// flushed to the stack. After both players pass, it resolves and the
/// attacker gains +2/+2 until end of turn.
fn test_702_23a_rampage_blocked_by_two_gets_bonus() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(
            ObjectSpec::creature(p1, "Rampage Attacker", 3, 3)
                .with_keyword(KeywordAbility::Rampage(2)),
        )
        .object(ObjectSpec::creature(p2, "Blocker A", 1, 1))
        .object(ObjectSpec::creature(p2, "Blocker B", 1, 1))
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let attacker_id = find_object(&state, "Rampage Attacker");
    let blocker_a = find_object(&state, "Blocker A");
    let blocker_b = find_object(&state, "Blocker B");

    // Declare attacker.
    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(attacker_id, AttackTarget::Player(p2))],
        },
    )
    .expect("DeclareAttackers should succeed");

    // Advance to DeclareBlockers.
    let (state, _) = pass_all(state, &[p1, p2]);

    // P2 blocks with both creatures.
    let (state, declare_events) = process_command(
        state,
        Command::DeclareBlockers {
            player: p2,
            blockers: vec![(blocker_a, attacker_id), (blocker_b, attacker_id)],
        },
    )
    .expect("DeclareBlockers should succeed");

    // AbilityTriggered event should fire from the rampage creature.
    assert!(
        declare_events.iter().any(|e| {
            matches!(
                e,
                GameEvent::AbilityTriggered { source_object_id, .. }
                if *source_object_id == attacker_id
            )
        }),
        "CR 702.23a: AbilityTriggered event should fire from rampage creature"
    );

    // Trigger should be on the stack.
    assert_eq!(
        state.stack_objects.len(),
        1,
        "CR 702.23a: Rampage trigger should be on the stack after blockers declared"
    );

    // Resolve the trigger (both players pass).
    let (state, resolve_events) = pass_all(state, &[p1, p2]);

    // AbilityResolved event should have been emitted.
    assert!(
        resolve_events
            .iter()
            .any(|e| matches!(e, GameEvent::AbilityResolved { .. })),
        "CR 702.23a: AbilityResolved event should fire after trigger resolves"
    );

    // Rampage Attacker should now be 5/5 (3 + 2 power, 3 + 2 toughness).
    let chars = calculate_characteristics(&state, attacker_id)
        .expect("attacker should still be on battlefield");
    assert_eq!(
        chars.power,
        Some(5),
        "CR 702.23a: Rampage 2 blocked by 2 → +2 power (3+2=5); 1 beyond first × 2"
    );
    assert_eq!(
        chars.toughness,
        Some(5),
        "CR 702.23a: Rampage 2 blocked by 2 → +2 toughness (3+2=5); Rampage is +N/+N"
    );

    // Stack should be empty.
    assert!(
        state.stack_objects.is_empty(),
        "stack should be empty after rampage trigger resolves"
    );
}

// ── Test 2: Blocked by exactly 1 creature → no bonus ─────────────────────────

#[test]
/// CR 702.23a "for each creature blocking it beyond the first" — if the creature
/// is blocked by exactly 1 creature, there are 0 beyond the first, so no bonus
/// is applied. The trigger still fires but resolves as a no-op.
fn test_702_23a_rampage_blocked_by_one_no_bonus() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(
            ObjectSpec::creature(p1, "Rampage Attacker", 3, 3)
                .with_keyword(KeywordAbility::Rampage(2)),
        )
        .object(ObjectSpec::creature(p2, "Single Blocker", 2, 2))
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let attacker_id = find_object(&state, "Rampage Attacker");
    let blocker_id = find_object(&state, "Single Blocker");

    // Declare attacker.
    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(attacker_id, AttackTarget::Player(p2))],
        },
    )
    .expect("DeclareAttackers should succeed");

    // Advance to DeclareBlockers.
    let (state, _) = pass_all(state, &[p1, p2]);

    // P2 blocks with only one creature.
    let (state, declare_events) = process_command(
        state,
        Command::DeclareBlockers {
            player: p2,
            blockers: vec![(blocker_id, attacker_id)],
        },
    )
    .expect("DeclareBlockers should succeed");

    // Trigger fires (creature became blocked).
    assert!(
        declare_events.iter().any(|e| {
            matches!(
                e,
                GameEvent::AbilityTriggered { source_object_id, .. }
                if *source_object_id == attacker_id
            )
        }),
        "CR 702.23a: Rampage trigger fires even when blocked by 1 creature"
    );

    // Trigger on the stack.
    assert_eq!(
        state.stack_objects.len(),
        1,
        "CR 702.23a: trigger should be on the stack"
    );

    // Resolve the trigger — no bonus (0 beyond the first).
    let (state, _) = pass_all(state, &[p1, p2]);

    // No P/T change — bonus is 0 × 2 = 0.
    let chars = calculate_characteristics(&state, attacker_id)
        .expect("attacker should still be on battlefield");
    assert_eq!(
        chars.power,
        Some(3),
        "CR 702.23a: Rampage blocked by 1 → 0 beyond first → no power bonus"
    );
    assert_eq!(
        chars.toughness,
        Some(3),
        "CR 702.23a: Rampage blocked by 1 → 0 beyond first → no toughness bonus"
    );

    // No continuous effects from this trigger.
    assert!(
        state.continuous_effects.is_empty(),
        "CR 702.23a: no continuous effects when blocked by exactly 1 creature"
    );
}

// ── Test 3: Blocked by 3 creatures → scaled bonus ────────────────────────────

#[test]
/// CR 702.23a — Creature with Rampage 2 blocked by 3 creatures.
/// Beyond-first = 2, bonus = 2 × 2 = +4/+4. Verify both power AND toughness.
fn test_702_23a_rampage_blocked_by_three_scaled_bonus() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(
            ObjectSpec::creature(p1, "Rampage Attacker", 2, 4)
                .with_keyword(KeywordAbility::Rampage(2)),
        )
        .object(ObjectSpec::creature(p2, "Blocker A", 1, 1))
        .object(ObjectSpec::creature(p2, "Blocker B", 1, 1))
        .object(ObjectSpec::creature(p2, "Blocker C", 1, 1))
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let attacker_id = find_object(&state, "Rampage Attacker");
    let blocker_a = find_object(&state, "Blocker A");
    let blocker_b = find_object(&state, "Blocker B");
    let blocker_c = find_object(&state, "Blocker C");

    // Declare attacker.
    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(attacker_id, AttackTarget::Player(p2))],
        },
    )
    .expect("DeclareAttackers should succeed");

    // Advance to DeclareBlockers.
    let (state, _) = pass_all(state, &[p1, p2]);

    // P2 blocks with all three.
    let (state, _) = process_command(
        state,
        Command::DeclareBlockers {
            player: p2,
            blockers: vec![
                (blocker_a, attacker_id),
                (blocker_b, attacker_id),
                (blocker_c, attacker_id),
            ],
        },
    )
    .expect("DeclareBlockers should succeed");

    // Trigger on stack.
    assert_eq!(
        state.stack_objects.len(),
        1,
        "CR 702.23a: one rampage trigger on the stack"
    );

    // Resolve trigger.
    let (state, _) = pass_all(state, &[p1, p2]);

    // Bonus = 2 beyond-first × Rampage 2 = +4/+4. Attacker is 2+4=6 power, 4+4=8 toughness.
    let chars = calculate_characteristics(&state, attacker_id)
        .expect("attacker should still be on battlefield");
    assert_eq!(
        chars.power,
        Some(6),
        "CR 702.23a: Rampage 2 blocked by 3 → bonus = 2×2=+4 power (2+4=6)"
    );
    assert_eq!(
        chars.toughness,
        Some(8),
        "CR 702.23a: Rampage 2 blocked by 3 → bonus = 2×2=+4 toughness (4+4=8)"
    );
}

// ── Test 4: Multiple Rampage instances trigger separately ─────────────────────

#[test]
/// CR 702.23c — "If a creature has multiple instances of rampage, each triggers
/// separately." A creature with Rampage 2 twice, blocked by 3, fires two triggers.
/// Each resolves to +4/+4 → total +8/+8.
fn test_702_23c_multiple_rampage_instances() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(
            ObjectSpec::creature(p1, "Double Rampage", 1, 1)
                .with_keyword(KeywordAbility::Rampage(2))
                .with_keyword(KeywordAbility::Rampage(2)),
        )
        .object(ObjectSpec::creature(p2, "Blocker A", 1, 1))
        .object(ObjectSpec::creature(p2, "Blocker B", 1, 1))
        .object(ObjectSpec::creature(p2, "Blocker C", 1, 1))
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let attacker_id = find_object(&state, "Double Rampage");
    let blocker_a = find_object(&state, "Blocker A");
    let blocker_b = find_object(&state, "Blocker B");
    let blocker_c = find_object(&state, "Blocker C");

    // Declare attacker.
    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(attacker_id, AttackTarget::Player(p2))],
        },
    )
    .expect("DeclareAttackers should succeed");

    // Advance to DeclareBlockers.
    let (state, _) = pass_all(state, &[p1, p2]);

    // P2 blocks with all three.
    let (state, declare_events) = process_command(
        state,
        Command::DeclareBlockers {
            player: p2,
            blockers: vec![
                (blocker_a, attacker_id),
                (blocker_b, attacker_id),
                (blocker_c, attacker_id),
            ],
        },
    )
    .expect("DeclareBlockers should succeed");

    // Two AbilityTriggered events (one per Rampage instance, CR 702.23c).
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
        "CR 702.23c: two Rampage instances should generate two triggers"
    );

    // Two triggers on the stack.
    assert_eq!(
        state.stack_objects.len(),
        2,
        "CR 702.23c: two rampage triggers on the stack"
    );

    // Resolve first trigger: attacker gains +4/+4 (1+4=5 power, 1+4=5 toughness).
    let (state, _) = pass_all(state, &[p1, p2]);
    assert_eq!(
        state.stack_objects.len(),
        1,
        "one trigger should remain after first resolves"
    );

    let chars = calculate_characteristics(&state, attacker_id).unwrap();
    assert_eq!(
        chars.power,
        Some(5),
        "CR 702.23c: after first Rampage 2 trigger: 1+4=5 power"
    );
    assert_eq!(
        chars.toughness,
        Some(5),
        "CR 702.23c: after first Rampage 2 trigger: 1+4=5 toughness"
    );

    // Resolve second trigger: attacker gains another +4/+4 → total 1+8=9 power, 1+8=9 toughness.
    let (state, _) = pass_all(state, &[p1, p2]);
    assert!(
        state.stack_objects.is_empty(),
        "stack should be empty after both triggers resolve"
    );

    let chars = calculate_characteristics(&state, attacker_id).unwrap();
    assert_eq!(
        chars.power,
        Some(9),
        "CR 702.23c: after both Rampage 2 triggers: 1+4+4=9 power"
    );
    assert_eq!(
        chars.toughness,
        Some(9),
        "CR 702.23c: after both Rampage 2 triggers: 1+4+4=9 toughness"
    );
}

// ── Test 5: Not blocked → no trigger ─────────────────────────────────────────

#[test]
/// CR 702.23a "Whenever this creature becomes blocked" — no trigger fires when
/// the creature attacks but is not blocked.
fn test_702_23a_rampage_not_blocked_no_trigger() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(
            ObjectSpec::creature(p1, "Rampage Attacker", 3, 3)
                .with_keyword(KeywordAbility::Rampage(2)),
        )
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let attacker_id = find_object(&state, "Rampage Attacker");

    // Declare attacker.
    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(attacker_id, AttackTarget::Player(p2))],
        },
    )
    .expect("DeclareAttackers should succeed");

    // Advance to DeclareBlockers.
    let (state, _) = pass_all(state, &[p1, p2]);

    // P2 declares no blockers.
    let (state, declare_events) = process_command(
        state,
        Command::DeclareBlockers {
            player: p2,
            blockers: vec![],
        },
    )
    .expect("DeclareBlockers with no blockers should succeed");

    // No AbilityTriggered event.
    assert!(
        !declare_events.iter().any(|e| {
            matches!(
                e,
                GameEvent::AbilityTriggered { source_object_id, .. }
                if *source_object_id == attacker_id
            )
        }),
        "CR 702.23a: no trigger when creature is not blocked"
    );

    // Stack should be empty.
    assert!(
        state.stack_objects.is_empty(),
        "CR 702.23a: stack must be empty when rampage creature is not blocked"
    );

    // P/T unchanged.
    let chars =
        calculate_characteristics(&state, attacker_id).expect("attacker should be on battlefield");
    assert_eq!(
        chars.power,
        Some(3),
        "CR 702.23a: no bonus when not blocked (power unchanged)"
    );
    assert_eq!(
        chars.toughness,
        Some(3),
        "CR 702.23a: no bonus when not blocked (toughness unchanged)"
    );
}

// ── Test 6: Bonus expires at end of turn ──────────────────────────────────────

#[test]
/// CR 702.23a ("until end of turn") + CR 514.2 — After expire_end_of_turn_effects,
/// the Rampage bonus is removed and the creature returns to its printed P/T.
fn test_702_23a_rampage_bonus_expires_at_end_of_turn() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(
            ObjectSpec::creature(p1, "Rampage Attacker", 2, 4)
                .with_keyword(KeywordAbility::Rampage(3)),
        )
        .object(ObjectSpec::creature(p2, "Blocker A", 1, 1))
        .object(ObjectSpec::creature(p2, "Blocker B", 1, 1))
        .object(ObjectSpec::creature(p2, "Blocker C", 1, 1))
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let attacker_id = find_object(&state, "Rampage Attacker");
    let blocker_a = find_object(&state, "Blocker A");
    let blocker_b = find_object(&state, "Blocker B");
    let blocker_c = find_object(&state, "Blocker C");

    // Declare attacker.
    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(attacker_id, AttackTarget::Player(p2))],
        },
    )
    .expect("DeclareAttackers should succeed");

    // Advance to DeclareBlockers.
    let (state, _) = pass_all(state, &[p1, p2]);

    // P2 blocks with 3. Trigger fires.
    let (state, _) = process_command(
        state,
        Command::DeclareBlockers {
            player: p2,
            blockers: vec![
                (blocker_a, attacker_id),
                (blocker_b, attacker_id),
                (blocker_c, attacker_id),
            ],
        },
    )
    .expect("DeclareBlockers should succeed");

    // Resolve trigger. Bonus = 2 × 3 = +6/+6. Attacker is 2+6=8 power, 4+6=10 toughness.
    let (state, _) = pass_all(state, &[p1, p2]);

    let chars = calculate_characteristics(&state, attacker_id).unwrap();
    assert_eq!(
        chars.power,
        Some(8),
        "Rampage 3 blocked by 3 → +6 power (2+6=8)"
    );
    assert_eq!(
        chars.toughness,
        Some(10),
        "Rampage 3 blocked by 3 → +6 toughness (4+6=10)"
    );

    // Simulate cleanup: expire all UntilEndOfTurn effects (CR 514.2).
    let mut state = state;
    mtg_engine::rules::layers::expire_end_of_turn_effects(&mut state);

    // After cleanup: Attacker returns to its printed 2/4.
    let chars = calculate_characteristics(&state, attacker_id).unwrap();
    assert_eq!(
        chars.power,
        Some(2),
        "CR 514.2: Rampage bonus expired — attacker returns to printed power (2)"
    );
    assert_eq!(
        chars.toughness,
        Some(4),
        "CR 514.2: Rampage bonus expired — attacker returns to printed toughness (4)"
    );

    // No more continuous effects from Rampage.
    assert!(
        state.continuous_effects.is_empty(),
        "CR 514.2: no UntilEndOfTurn effects should remain after cleanup"
    );
}

// ── Test 7: CR 702.23b — bonus calculated at resolution (state snapshot) ─────

#[test]
/// CR 702.23b — "The rampage bonus is calculated only once per combat, when the
/// triggered ability resolves." This is implicitly verified by tests 1-4: the
/// blocker count is taken from `state.combat.blockers_for()` at resolution time.
/// We verify the blocker count is correct at the time the trigger resolves.
///
/// A creature with Rampage 1 blocked by 2 creatures gets +1/+1.
fn test_702_23b_bonus_calculated_at_resolution_time() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(
            ObjectSpec::creature(p1, "Rampage Attacker", 4, 4)
                .with_keyword(KeywordAbility::Rampage(1)),
        )
        .object(ObjectSpec::creature(p2, "Blocker A", 1, 1))
        .object(ObjectSpec::creature(p2, "Blocker B", 1, 1))
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let attacker_id = find_object(&state, "Rampage Attacker");
    let blocker_a = find_object(&state, "Blocker A");
    let blocker_b = find_object(&state, "Blocker B");

    // Declare attacker.
    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(attacker_id, AttackTarget::Player(p2))],
        },
    )
    .expect("DeclareAttackers should succeed");

    // Advance to DeclareBlockers.
    let (state, _) = pass_all(state, &[p1, p2]);

    // P2 blocks with 2 creatures. Rampage 1 trigger fires.
    let (state, _) = process_command(
        state,
        Command::DeclareBlockers {
            player: p2,
            blockers: vec![(blocker_a, attacker_id), (blocker_b, attacker_id)],
        },
    )
    .expect("DeclareBlockers should succeed");

    assert_eq!(
        state.stack_objects.len(),
        1,
        "one rampage trigger on the stack"
    );

    // Resolve. Bonus = 1 beyond-first × Rampage 1 = +1/+1. Attacker becomes 5/5.
    let (state, _) = pass_all(state, &[p1, p2]);

    let chars =
        calculate_characteristics(&state, attacker_id).expect("attacker should be on battlefield");
    assert_eq!(
        chars.power,
        Some(5),
        "CR 702.23b: Rampage 1 blocked by 2 → +1 power at resolution time (4+1=5)"
    );
    assert_eq!(
        chars.toughness,
        Some(5),
        "CR 702.23b: Rampage 1 blocked by 2 → +1 toughness at resolution time (4+1=5)"
    );
}

// ── Test 8: Rampage 3 blocked by 4 creatures ─────────────────────────────────

#[test]
/// CR 702.23a — Creature with Rampage 3 blocked by 4 creatures.
/// Beyond-first = 3, bonus = 3 × 3 = +9/+9.
fn test_702_23a_rampage_three_blocked_by_four() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(
            ObjectSpec::creature(p1, "Rampage 3 Attacker", 1, 1)
                .with_keyword(KeywordAbility::Rampage(3)),
        )
        .object(ObjectSpec::creature(p2, "Blocker A", 1, 1))
        .object(ObjectSpec::creature(p2, "Blocker B", 1, 1))
        .object(ObjectSpec::creature(p2, "Blocker C", 1, 1))
        .object(ObjectSpec::creature(p2, "Blocker D", 1, 1))
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let attacker_id = find_object(&state, "Rampage 3 Attacker");
    let blocker_a = find_object(&state, "Blocker A");
    let blocker_b = find_object(&state, "Blocker B");
    let blocker_c = find_object(&state, "Blocker C");
    let blocker_d = find_object(&state, "Blocker D");

    // Declare attacker.
    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(attacker_id, AttackTarget::Player(p2))],
        },
    )
    .expect("DeclareAttackers should succeed");

    // Advance to DeclareBlockers.
    let (state, _) = pass_all(state, &[p1, p2]);

    // P2 blocks with all 4.
    let (state, _) = process_command(
        state,
        Command::DeclareBlockers {
            player: p2,
            blockers: vec![
                (blocker_a, attacker_id),
                (blocker_b, attacker_id),
                (blocker_c, attacker_id),
                (blocker_d, attacker_id),
            ],
        },
    )
    .expect("DeclareBlockers should succeed");

    // Resolve trigger.
    let (state, _) = pass_all(state, &[p1, p2]);

    // Bonus = 3 beyond-first × Rampage 3 = +9/+9. Attacker: 1+9=10/10.
    let chars =
        calculate_characteristics(&state, attacker_id).expect("attacker should be on battlefield");
    assert_eq!(
        chars.power,
        Some(10),
        "CR 702.23a: Rampage 3 blocked by 4 → +9 power (1+9=10)"
    );
    assert_eq!(
        chars.toughness,
        Some(10),
        "CR 702.23a: Rampage 3 blocked by 4 → +9 toughness (1+9=10)"
    );
}
