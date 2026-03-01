//! Afflict keyword ability tests (CR 702.130).
//!
//! Afflict is a triggered ability: "Whenever this creature becomes blocked,
//! defending player loses N life."
//!
//! Key rules verified:
//! - Trigger fires when creature becomes blocked (CR 702.130a).
//! - Trigger does NOT fire when creature is unblocked (CR 509.3c).
//! - Multiple blockers on the same attacker still produce exactly one trigger (CR 509.3c).
//! - Multiple instances of afflict trigger separately (CR 702.130b).
//! - Multiplayer: each trigger targets the correct defending player (CR 508.5, 702.130a).
//! - Life loss is NOT damage — no DamageDealt event is emitted (rulings 2017-07-14).

use mtg_engine::{
    process_command, AttackTarget, CardRegistry, Command, GameEvent, GameState, GameStateBuilder,
    KeywordAbility, ObjectId, ObjectSpec, PlayerId, Step, ZoneId,
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

/// Pass priority for all listed players once (resolves the top stack item).
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

/// Get the life total of a player.
fn life_total(state: &GameState, player: PlayerId) -> i32 {
    state
        .players
        .get(&player)
        .map(|p| p.life_total)
        .unwrap_or_else(|| panic!("player {:?} not found", player))
}

// ── Test 1: Basic life loss when blocked ──────────────────────────────────────

#[test]
/// CR 702.130a — "Afflict N" means "Whenever this creature becomes blocked,
/// defending player loses N life." P1 attacks P2 with Afflict 2. P2 declares
/// a blocker. Afflict trigger fires and resolves — P2 loses 2 life.
fn test_702_130a_afflict_basic_life_loss() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let attacker = ObjectSpec::creature(p1, "Afflict Creature", 2, 2)
        .with_keyword(KeywordAbility::Afflict(2))
        .in_zone(ZoneId::Battlefield);
    let blocker = ObjectSpec::creature(p2, "P2 Blocker", 2, 2).in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(attacker)
        .object(blocker)
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let attacker_id = find_object(&state, "Afflict Creature");
    let blocker_id = find_object(&state, "P2 Blocker");

    let p2_life_before = life_total(&state, p2);

    // P1 declares attacker.
    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(attacker_id, AttackTarget::Player(p2))],
        },
    )
    .expect("DeclareAttackers should succeed");

    // No trigger yet — afflict fires on becomes blocked, not on attacking.
    assert!(
        state.stack_objects.is_empty(),
        "CR 702.130a: afflict trigger should NOT fire at DeclareAttackers — only at DeclareBlockers"
    );

    // Pass priority to advance to DeclareBlockers.
    let (state, _) = pass_all(state, &[p1, p2]);

    // P2 declares a blocker — afflict trigger fires here.
    let (state, events) = process_command(
        state,
        Command::DeclareBlockers {
            player: p2,
            blockers: vec![(blocker_id, attacker_id)],
        },
    )
    .expect("DeclareBlockers should succeed");

    // AbilityTriggered event should be emitted.
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::AbilityTriggered { .. })),
        "CR 702.130a: AbilityTriggered event expected when creature becomes blocked"
    );
    assert_eq!(
        state.stack_objects.len(),
        1,
        "CR 702.130a: afflict trigger should be on the stack"
    );

    // Resolve the afflict trigger — both players pass.
    let (state, _) = pass_all(state, &[p1, p2]);

    // P2 should have lost 2 life.
    assert_eq!(
        life_total(&state, p2),
        p2_life_before - 2,
        "CR 702.130a: defending player should lose N life when afflict trigger resolves"
    );
    // P1 should be unaffected.
    assert_eq!(
        life_total(&state, p1),
        40,
        "CR 702.130a: attacker's controller should NOT lose life from afflict"
    );
}

// ── Test 2: No trigger when unblocked ────────────────────────────────────────

#[test]
/// CR 702.130a, CR 509.3c — Afflict only triggers when the creature "becomes blocked."
/// A creature with no blockers declared against it does NOT become blocked,
/// so afflict does NOT trigger.
fn test_702_130a_afflict_not_blocked_no_trigger() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let attacker = ObjectSpec::creature(p1, "Afflict Creature", 2, 2)
        .with_keyword(KeywordAbility::Afflict(2))
        .in_zone(ZoneId::Battlefield);

    // P2 has no creatures — no blocking possible.
    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(attacker)
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let attacker_id = find_object(&state, "Afflict Creature");
    let p2_life_before = life_total(&state, p2);

    // P1 declares attacker.
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
    let (state, events) = process_command(
        state,
        Command::DeclareBlockers {
            player: p2,
            blockers: vec![],
        },
    )
    .expect("DeclareBlockers with no blockers should succeed");

    // No AbilityTriggered event — creature was not blocked.
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, GameEvent::AbilityTriggered { .. })),
        "CR 509.3c: afflict should NOT trigger when creature is unblocked"
    );
    assert!(
        state.stack_objects.is_empty(),
        "CR 509.3c: stack should be empty — no afflict trigger when unblocked"
    );

    // P2's life total should be unchanged.
    assert_eq!(
        life_total(&state, p2),
        p2_life_before,
        "CR 509.3c: defending player should NOT lose life when creature is unblocked"
    );
}

// ── Test 3: Multiple blockers — single trigger (CR 509.3c) ───────────────────

#[test]
/// CR 509.3c — "Whenever [a creature] becomes blocked" triggers only once per
/// combat per attacking creature, even if multiple creatures block it.
/// A creature with Afflict 3 blocked by 2 creatures still only loses the defender
/// 3 life (one trigger), not 6 life (two triggers).
fn test_509_3c_afflict_multiple_blockers_single_trigger() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let attacker = ObjectSpec::creature(p1, "Afflict Creature", 5, 5)
        .with_keyword(KeywordAbility::Afflict(3))
        .in_zone(ZoneId::Battlefield);
    let blocker1 = ObjectSpec::creature(p2, "P2 Blocker 1", 1, 2).in_zone(ZoneId::Battlefield);
    let blocker2 = ObjectSpec::creature(p2, "P2 Blocker 2", 1, 2).in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(attacker)
        .object(blocker1)
        .object(blocker2)
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let attacker_id = find_object(&state, "Afflict Creature");
    let blocker1_id = find_object(&state, "P2 Blocker 1");
    let blocker2_id = find_object(&state, "P2 Blocker 2");
    let p2_life_before = life_total(&state, p2);

    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(attacker_id, AttackTarget::Player(p2))],
        },
    )
    .expect("DeclareAttackers should succeed");

    let (state, _) = pass_all(state, &[p1, p2]);

    // P2 declares TWO blockers against the same attacker.
    let (state, events) = process_command(
        state,
        Command::DeclareBlockers {
            player: p2,
            blockers: vec![(blocker1_id, attacker_id), (blocker2_id, attacker_id)],
        },
    )
    .expect("DeclareBlockers should succeed");

    // Exactly ONE AbilityTriggered event (from the single SelfBecomesBlocked trigger).
    let triggered_count = events
        .iter()
        .filter(|e| matches!(e, GameEvent::AbilityTriggered { .. }))
        .count();
    assert_eq!(
        triggered_count, 1,
        "CR 509.3c: afflict should trigger exactly once regardless of how many creatures block it"
    );
    assert_eq!(
        state.stack_objects.len(),
        1,
        "CR 509.3c: exactly one afflict trigger on the stack for two blockers"
    );

    // Resolve the single trigger.
    let (state, _) = pass_all(state, &[p1, p2]);

    // P2 should have lost exactly 3 life — not 6.
    assert_eq!(
        life_total(&state, p2),
        p2_life_before - 3,
        "CR 509.3c: defending player loses 3 life (not 6) — one trigger regardless of blocker count"
    );
}

// ── Test 4: Multiple instances trigger separately (CR 702.130b) ──────────────

#[test]
/// CR 702.130b — "If a creature has multiple instances of afflict, each triggers
/// separately." A creature with Afflict 2 AND Afflict 1 generates two separate
/// triggers when blocked. After both resolve, P2 loses 2 + 1 = 3 life total.
fn test_702_130b_afflict_multiple_instances_trigger_separately() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    // Creature with TWO afflict keyword instances.
    let attacker = ObjectSpec::creature(p1, "Double Afflict", 3, 3)
        .with_keyword(KeywordAbility::Afflict(2))
        .with_keyword(KeywordAbility::Afflict(1))
        .in_zone(ZoneId::Battlefield);
    let blocker = ObjectSpec::creature(p2, "P2 Blocker", 3, 3).in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(attacker)
        .object(blocker)
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let attacker_id = find_object(&state, "Double Afflict");
    let blocker_id = find_object(&state, "P2 Blocker");
    let p2_life_before = life_total(&state, p2);

    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(attacker_id, AttackTarget::Player(p2))],
        },
    )
    .expect("DeclareAttackers should succeed");

    let (state, _) = pass_all(state, &[p1, p2]);

    let (state, events) = process_command(
        state,
        Command::DeclareBlockers {
            player: p2,
            blockers: vec![(blocker_id, attacker_id)],
        },
    )
    .expect("DeclareBlockers should succeed");

    // Two AbilityTriggered events — one per afflict instance.
    let triggered_count = events
        .iter()
        .filter(|e| matches!(e, GameEvent::AbilityTriggered { .. }))
        .count();
    assert_eq!(
        triggered_count, 2,
        "CR 702.130b: two afflict instances should generate two separate triggers"
    );
    assert_eq!(
        state.stack_objects.len(),
        2,
        "CR 702.130b: two afflict triggers should be on the stack"
    );

    // Resolve first trigger.
    let (state, _) = pass_all(state, &[p1, p2]);
    // Resolve second trigger.
    let (state, _) = pass_all(state, &[p1, p2]);

    // P2 should have lost 2 + 1 = 3 life total.
    assert_eq!(
        life_total(&state, p2),
        p2_life_before - 3,
        "CR 702.130b: P2 should lose 3 life total (2 + 1) from two afflict triggers"
    );
}

// ── Test 5: Multiplayer — correct defending player ────────────────────────────

#[test]
/// CR 508.5, CR 702.130a — In multiplayer, each afflict trigger must target the
/// correct defending player. P1 attacks P2 with an afflict creature and P3 with
/// a non-afflict creature. Only P2 (the correct defending player) loses life.
fn test_afflict_multiplayer_correct_defending_player() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);
    let p3 = PlayerId(3);
    let p4 = PlayerId(4);

    let afflict_attacker = ObjectSpec::creature(p1, "Afflict Attacker", 2, 2)
        .with_keyword(KeywordAbility::Afflict(2))
        .in_zone(ZoneId::Battlefield);
    let plain_attacker =
        ObjectSpec::creature(p1, "Plain Attacker", 2, 2).in_zone(ZoneId::Battlefield);

    let p2_blocker = ObjectSpec::creature(p2, "P2 Blocker", 2, 2).in_zone(ZoneId::Battlefield);
    let p3_blocker = ObjectSpec::creature(p3, "P3 Blocker", 2, 2).in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .add_player(p4)
        .with_registry(CardRegistry::new(vec![]))
        .object(afflict_attacker)
        .object(plain_attacker)
        .object(p2_blocker)
        .object(p3_blocker)
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let afflict_id = find_object(&state, "Afflict Attacker");
    let plain_id = find_object(&state, "Plain Attacker");
    let p2_blocker_id = find_object(&state, "P2 Blocker");
    let p3_blocker_id = find_object(&state, "P3 Blocker");

    let p2_life_before = life_total(&state, p2);
    let p3_life_before = life_total(&state, p3);

    // P1 attacks P2 with afflict creature and P3 with plain creature.
    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![
                (afflict_id, AttackTarget::Player(p2)),
                (plain_id, AttackTarget::Player(p3)),
            ],
        },
    )
    .expect("DeclareAttackers should succeed");

    // Advance to DeclareBlockers for P2.
    let (state, _) = pass_all(state, &[p1, p2, p3, p4]);

    // P2 blocks the afflict creature.
    let (state, events) = process_command(
        state,
        Command::DeclareBlockers {
            player: p2,
            blockers: vec![(p2_blocker_id, afflict_id)],
        },
    )
    .expect("P2 DeclareBlockers should succeed");

    // Afflict trigger fires for P2's block.
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::AbilityTriggered { .. })),
        "CR 702.130a: afflict trigger should fire when P2's blocker blocks"
    );

    // Resolve P2's afflict trigger.
    let (state, _) = pass_all(state, &[p1, p2, p3, p4]);

    // P2 loses 2 life from afflict.
    assert_eq!(
        life_total(&state, p2),
        p2_life_before - 2,
        "CR 508.5: P2 (defending player for the afflict creature) should lose 2 life"
    );

    // P3 has NOT blocked yet, advance.
    let (state, events) = process_command(
        state,
        Command::DeclareBlockers {
            player: p3,
            blockers: vec![(p3_blocker_id, plain_id)],
        },
    )
    .expect("P3 DeclareBlockers should succeed");

    // No AbilityTriggered for P3's block (plain attacker has no afflict).
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, GameEvent::AbilityTriggered { .. })),
        "CR 702.130a: no afflict trigger for plain (non-afflict) attacker being blocked"
    );

    // P3 should not have lost any life from afflict.
    assert_eq!(
        life_total(&state, p3),
        p3_life_before,
        "CR 508.5: P3 should NOT lose life (attacked by non-afflict creature)"
    );
}

// ── Test 6: Life loss is not damage ──────────────────────────────────────────

#[test]
/// Rulings 2017-07-14 — Afflict causes life loss, NOT damage. The engine should
/// emit a LifeChanged event, but NOT a DamageDealt event. This verifies that
/// afflict bypasses damage prevention and does not interact with lifelink.
fn test_afflict_life_loss_not_damage() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let attacker = ObjectSpec::creature(p1, "Afflict Creature", 2, 2)
        .with_keyword(KeywordAbility::Afflict(3))
        .in_zone(ZoneId::Battlefield);
    let blocker = ObjectSpec::creature(p2, "P2 Blocker", 2, 2).in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(attacker)
        .object(blocker)
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let attacker_id = find_object(&state, "Afflict Creature");
    let blocker_id = find_object(&state, "P2 Blocker");
    let p2_life_before = life_total(&state, p2);

    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(attacker_id, AttackTarget::Player(p2))],
        },
    )
    .expect("DeclareAttackers should succeed");

    let (state, _) = pass_all(state, &[p1, p2]);

    let (state, _) = process_command(
        state,
        Command::DeclareBlockers {
            player: p2,
            blockers: vec![(blocker_id, attacker_id)],
        },
    )
    .expect("DeclareBlockers should succeed");

    // Resolve the afflict trigger.
    let (state, resolution_events) = pass_all(state, &[p1, p2]);

    // Life should have decreased.
    assert_eq!(
        life_total(&state, p2),
        p2_life_before - 3,
        "Afflict: P2 should lose 3 life"
    );

    // LifeLost event should be emitted.
    assert!(
        resolution_events
            .iter()
            .any(|e| matches!(e, GameEvent::LifeLost { .. })),
        "Afflict: LifeLost event should be emitted when life is lost"
    );

    // DamageDealt event should NOT be emitted by afflict resolution.
    // (DamageDealt is for actual damage; afflict causes life loss directly.)
    assert!(
        !resolution_events
            .iter()
            .any(|e| matches!(e, GameEvent::DamageDealt { .. })),
        "Rulings 2017-07-14: afflict life loss should NOT produce a DamageDealt event"
    );
}
