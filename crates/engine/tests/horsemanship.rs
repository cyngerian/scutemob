//! Horsemanship evasion ability enforcement tests (CR 702.31).
//!
//! Horsemanship is a **unidirectional** evasion ability: a creature with
//! horsemanship can't be blocked by creatures without horsemanship. However,
//! a creature WITH horsemanship CAN block a creature WITHOUT horsemanship.
//! This is the key difference from Shadow (CR 702.28), which is bidirectional.
//!
//! Ruling 2009-10-01: Horsemanship does not interact with flying or reach.
//! A creature with flying (but not horsemanship) cannot block a horsemanship
//! attacker. The two evasion abilities are entirely independent.
//!
//! Tests:
//! - Horsemanship attacker cannot be blocked by non-horsemanship (CR 702.31b)
//! - Horsemanship attacker CAN be blocked by horsemanship (CR 702.31b)
//! - Non-horsemanship attacker CAN be blocked by horsemanship (CR 702.31b, unidirectional!)
//! - Non-horsemanship can block non-horsemanship (baseline)
//! - Flying does not satisfy horsemanship restriction (ruling 2009-10-01)
//! - Horsemanship + Flying: both evasion abilities must be satisfied (CR 702.31b + CR 702.9a)
//! - Horsemanship + Flying: blocker with horsemanship + flying satisfies both (CR 702.31b + CR 702.9a)

use mtg_engine::{
    process_command, AttackTarget, CombatState, Command, GameStateBuilder, KeywordAbility,
    ObjectSpec, PlayerId, Step,
};

// ── Helper: find object ID by name ───────────────────────────────────────────

fn find_object(state: &mtg_engine::GameState, name: &str) -> mtg_engine::ObjectId {
    state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{}' not found", name))
}

// ─── CR 702.31: Horsemanship ─────────────────────────────────────────────────

#[test]
/// CR 702.31b — A creature with horsemanship can't be blocked by creatures
/// without horsemanship. The block is illegal.
fn test_702_31_horsemanship_creature_cannot_be_blocked_by_non_horsemanship() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(
            ObjectSpec::creature(p1, "Horsemanship Attacker", 2, 2)
                .with_keyword(KeywordAbility::Horsemanship),
        )
        .object(ObjectSpec::creature(p2, "Normal Blocker", 2, 2))
        .at_step(Step::DeclareBlockers)
        .active_player(p1)
        .build()
        .unwrap();

    let attacker_id = find_object(&state, "Horsemanship Attacker");
    let blocker_id = find_object(&state, "Normal Blocker");

    let mut state = state;
    state.combat = Some({
        let mut cs = CombatState::new(p1);
        cs.attackers.insert(attacker_id, AttackTarget::Player(p2));
        cs
    });

    let result = process_command(
        state,
        Command::DeclareBlockers {
            player: p2,
            blockers: vec![(blocker_id, attacker_id)],
        },
    );

    assert!(
        result.is_err(),
        "A creature without horsemanship should not be able to block a horsemanship attacker (CR 702.31b)"
    );
}

#[test]
/// CR 702.31b — A creature with horsemanship CAN be blocked by another creature
/// with horsemanship. The block is legal.
fn test_702_31_horsemanship_creature_can_be_blocked_by_horsemanship() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(
            ObjectSpec::creature(p1, "Horsemanship Attacker", 2, 2)
                .with_keyword(KeywordAbility::Horsemanship),
        )
        .object(
            ObjectSpec::creature(p2, "Horsemanship Blocker", 2, 2)
                .with_keyword(KeywordAbility::Horsemanship),
        )
        .at_step(Step::DeclareBlockers)
        .active_player(p1)
        .build()
        .unwrap();

    let attacker_id = find_object(&state, "Horsemanship Attacker");
    let blocker_id = find_object(&state, "Horsemanship Blocker");

    let mut state = state;
    state.combat = Some({
        let mut cs = CombatState::new(p1);
        cs.attackers.insert(attacker_id, AttackTarget::Player(p2));
        cs
    });

    let result = process_command(
        state,
        Command::DeclareBlockers {
            player: p2,
            blockers: vec![(blocker_id, attacker_id)],
        },
    );

    assert!(
        result.is_ok(),
        "A horsemanship creature should be able to block a horsemanship attacker (CR 702.31b): {:?}",
        result.err()
    );
}

#[test]
/// CR 702.31b second sentence: "A creature with horsemanship can block a creature
/// with or without horsemanship." A creature WITH horsemanship CAN block a creature
/// WITHOUT horsemanship. This is the key unidirectional difference from Shadow (CR 702.28b).
fn test_702_31_non_horsemanship_can_be_blocked_by_horsemanship() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(ObjectSpec::creature(p1, "Normal Attacker", 2, 2))
        .object(
            ObjectSpec::creature(p2, "Horsemanship Blocker", 2, 2)
                .with_keyword(KeywordAbility::Horsemanship),
        )
        .at_step(Step::DeclareBlockers)
        .active_player(p1)
        .build()
        .unwrap();

    let attacker_id = find_object(&state, "Normal Attacker");
    let blocker_id = find_object(&state, "Horsemanship Blocker");

    let mut state = state;
    state.combat = Some({
        let mut cs = CombatState::new(p1);
        cs.attackers.insert(attacker_id, AttackTarget::Player(p2));
        cs
    });

    let result = process_command(
        state,
        Command::DeclareBlockers {
            player: p2,
            blockers: vec![(blocker_id, attacker_id)],
        },
    );

    assert!(
        result.is_ok(),
        "A horsemanship creature SHOULD be able to block a non-horsemanship attacker \
         (CR 702.31b: unidirectional restriction -- only attacker-has-horsemanship direction is restricted): {:?}",
        result.err()
    );
}

#[test]
/// CR 702.31b — Baseline: two non-horsemanship creatures can block each other normally.
fn test_702_31_non_horsemanship_can_block_non_horsemanship() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(ObjectSpec::creature(p1, "Normal Attacker", 2, 2))
        .object(ObjectSpec::creature(p2, "Normal Blocker", 2, 2))
        .at_step(Step::DeclareBlockers)
        .active_player(p1)
        .build()
        .unwrap();

    let attacker_id = find_object(&state, "Normal Attacker");
    let blocker_id = find_object(&state, "Normal Blocker");

    let mut state = state;
    state.combat = Some({
        let mut cs = CombatState::new(p1);
        cs.attackers.insert(attacker_id, AttackTarget::Player(p2));
        cs
    });

    let result = process_command(
        state,
        Command::DeclareBlockers {
            player: p2,
            blockers: vec![(blocker_id, attacker_id)],
        },
    );

    assert!(
        result.is_ok(),
        "A non-horsemanship creature should be able to block a non-horsemanship attacker: {:?}",
        result.err()
    );
}

#[test]
/// CR 702.31b, Ruling 2009-10-01 — Flying does NOT satisfy the horsemanship
/// restriction. Horsemanship is independent of flying and reach. A creature with
/// flying (but without horsemanship) cannot block a horsemanship attacker.
fn test_702_31_horsemanship_does_not_interact_with_flying() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(
            ObjectSpec::creature(p1, "Horsemanship Attacker", 2, 2)
                .with_keyword(KeywordAbility::Horsemanship),
        )
        .object(
            // Flying does NOT grant the ability to block horsemanship creatures
            ObjectSpec::creature(p2, "Flying Blocker", 2, 2).with_keyword(KeywordAbility::Flying),
        )
        .at_step(Step::DeclareBlockers)
        .active_player(p1)
        .build()
        .unwrap();

    let attacker_id = find_object(&state, "Horsemanship Attacker");
    let blocker_id = find_object(&state, "Flying Blocker");

    let mut state = state;
    state.combat = Some({
        let mut cs = CombatState::new(p1);
        cs.attackers.insert(attacker_id, AttackTarget::Player(p2));
        cs
    });

    let result = process_command(
        state,
        Command::DeclareBlockers {
            player: p2,
            blockers: vec![(blocker_id, attacker_id)],
        },
    );

    assert!(
        result.is_err(),
        "A flying (non-horsemanship) creature should NOT be able to block a horsemanship attacker \
         (ruling 2009-10-01: horsemanship does not interact with flying or reach)"
    );
}

#[test]
/// CR 702.31b + CR 702.9a — Multiple evasion abilities must ALL be satisfied.
/// A horsemanship creature with flying can only be blocked by a creature that has
/// BOTH horsemanship AND (flying or reach). A horsemanship-only blocker satisfies
/// horsemanship but not flying, so the block is illegal.
fn test_702_31_horsemanship_plus_flying_both_must_be_satisfied() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(
            ObjectSpec::creature(p1, "Horsemanship Flying Attacker", 2, 2)
                .with_keyword(KeywordAbility::Horsemanship)
                .with_keyword(KeywordAbility::Flying),
        )
        .object(
            // Has horsemanship (satisfies horsemanship restriction) but no flying or reach
            ObjectSpec::creature(p2, "Horsemanship Ground Blocker", 2, 2)
                .with_keyword(KeywordAbility::Horsemanship),
        )
        .at_step(Step::DeclareBlockers)
        .active_player(p1)
        .build()
        .unwrap();

    let attacker_id = find_object(&state, "Horsemanship Flying Attacker");
    let blocker_id = find_object(&state, "Horsemanship Ground Blocker");

    let mut state = state;
    state.combat = Some({
        let mut cs = CombatState::new(p1);
        cs.attackers.insert(attacker_id, AttackTarget::Player(p2));
        cs
    });

    let result = process_command(
        state,
        Command::DeclareBlockers {
            player: p2,
            blockers: vec![(blocker_id, attacker_id)],
        },
    );

    assert!(
        result.is_err(),
        "A horsemanship-only ground creature should not block a horsemanship+flying attacker \
         (CR 702.31b + CR 702.9a: both evasion restrictions must be satisfied)"
    );
}

#[test]
/// CR 702.31b + CR 702.9a — A blocker with both horsemanship and flying satisfies
/// both evasion restrictions of a horsemanship+flying attacker. Block is legal.
fn test_702_31_horsemanship_plus_flying_satisfied_by_horsemanship_flying() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(
            ObjectSpec::creature(p1, "Horsemanship Flying Attacker", 2, 2)
                .with_keyword(KeywordAbility::Horsemanship)
                .with_keyword(KeywordAbility::Flying),
        )
        .object(
            // Both horsemanship and flying satisfy the two restrictions
            ObjectSpec::creature(p2, "Horsemanship Flying Blocker", 2, 2)
                .with_keyword(KeywordAbility::Horsemanship)
                .with_keyword(KeywordAbility::Flying),
        )
        .at_step(Step::DeclareBlockers)
        .active_player(p1)
        .build()
        .unwrap();

    let attacker_id = find_object(&state, "Horsemanship Flying Attacker");
    let blocker_id = find_object(&state, "Horsemanship Flying Blocker");

    let mut state = state;
    state.combat = Some({
        let mut cs = CombatState::new(p1);
        cs.attackers.insert(attacker_id, AttackTarget::Player(p2));
        cs
    });

    let result = process_command(
        state,
        Command::DeclareBlockers {
            player: p2,
            blockers: vec![(blocker_id, attacker_id)],
        },
    );

    assert!(
        result.is_ok(),
        "A horsemanship+flying creature should block a horsemanship+flying attacker \
         (CR 702.31b + CR 702.9a): {:?}",
        result.err()
    );
}
