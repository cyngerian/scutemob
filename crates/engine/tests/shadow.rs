//! Shadow evasion ability enforcement tests (CR 702.28).
//!
//! Shadow is a bidirectional evasion ability: a creature with shadow can't be
//! blocked by creatures without shadow, and a creature without shadow can't be
//! blocked by creatures with shadow.
//!
//! Tests:
//! - Shadow attacker cannot be blocked by non-shadow (CR 702.28b, first half)
//! - Shadow attacker can be blocked by shadow (CR 702.28b, both have shadow)
//! - Non-shadow attacker cannot be blocked by shadow (CR 702.28b, second half -- unique to shadow)
//! - Non-shadow can block non-shadow (baseline -- neither has shadow)
//! - Shadow + Flying: both evasion abilities must be satisfied (CR 702.28b + CR 702.9a)
//! - Shadow + Flying: blocker with shadow + flying satisfies both (CR 702.28b + CR 702.9a)
//! - Shadow + Flying: blocker with shadow + reach satisfies both (CR 702.28b + CR 702.9a + CR 702.17)

use mtg_engine::{
    process_command, AttackTarget, Command, GameStateBuilder, KeywordAbility, ObjectSpec, PlayerId,
    Step,
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

// ─── CR 702.28: Shadow ────────────────────────────────────────────────────────

#[test]
/// CR 702.28b — A creature with shadow can't be blocked by creatures without shadow.
fn test_702_28_shadow_creature_cannot_be_blocked_by_non_shadow() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(
            ObjectSpec::creature(p1, "Shadow Attacker", 2, 2).with_keyword(KeywordAbility::Shadow),
        )
        .object(ObjectSpec::creature(p2, "Normal Blocker", 2, 2))
        .at_step(Step::DeclareBlockers)
        .active_player(p1)
        .build()
        .unwrap();

    let attacker_id = find_object(&state, "Shadow Attacker");
    let blocker_id = find_object(&state, "Normal Blocker");

    let mut state = state;
    state.combat = Some({
        let mut cs = mtg_engine::CombatState::new(p1);
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
        "A creature without shadow should not be able to block a shadow attacker (CR 702.28b)"
    );
}

#[test]
/// CR 702.28b — A creature with shadow CAN be blocked by another creature with shadow.
fn test_702_28_shadow_creature_can_be_blocked_by_shadow() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(
            ObjectSpec::creature(p1, "Shadow Attacker", 2, 2).with_keyword(KeywordAbility::Shadow),
        )
        .object(
            ObjectSpec::creature(p2, "Shadow Blocker", 2, 2).with_keyword(KeywordAbility::Shadow),
        )
        .at_step(Step::DeclareBlockers)
        .active_player(p1)
        .build()
        .unwrap();

    let attacker_id = find_object(&state, "Shadow Attacker");
    let blocker_id = find_object(&state, "Shadow Blocker");

    let mut state = state;
    state.combat = Some({
        let mut cs = mtg_engine::CombatState::new(p1);
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
        "A shadow creature should be able to block a shadow attacker (CR 702.28b): {:?}",
        result.err()
    );
}

#[test]
/// CR 702.28b — A creature without shadow can't be blocked by creatures with shadow.
/// This is the second half of the bidirectional restriction, unique to shadow
/// (unlike Flying/Fear which are one-directional).
fn test_702_28_non_shadow_creature_cannot_be_blocked_by_shadow() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(ObjectSpec::creature(p1, "Normal Attacker", 2, 2))
        .object(
            ObjectSpec::creature(p2, "Shadow Blocker", 2, 2).with_keyword(KeywordAbility::Shadow),
        )
        .at_step(Step::DeclareBlockers)
        .active_player(p1)
        .build()
        .unwrap();

    let attacker_id = find_object(&state, "Normal Attacker");
    let blocker_id = find_object(&state, "Shadow Blocker");

    let mut state = state;
    state.combat = Some({
        let mut cs = mtg_engine::CombatState::new(p1);
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
        "A shadow creature should not be able to block a non-shadow attacker (CR 702.28b)"
    );
}

#[test]
/// CR 702.28b — Baseline: two non-shadow creatures can block each other normally.
fn test_702_28_non_shadow_can_block_non_shadow() {
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
        let mut cs = mtg_engine::CombatState::new(p1);
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
        "A non-shadow creature should be able to block a non-shadow attacker (CR 702.28b): {:?}",
        result.err()
    );
}

#[test]
/// CR 702.28b + CR 702.9a — Multiple evasion abilities must ALL be satisfied.
/// A shadow creature with flying can only be blocked by a creature that has BOTH
/// shadow AND (flying or reach). A shadow-only blocker satisfies shadow but not flying.
fn test_702_28_shadow_plus_flying_both_must_be_satisfied() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(
            ObjectSpec::creature(p1, "Shadow Flying Attacker", 2, 2)
                .with_keyword(KeywordAbility::Shadow)
                .with_keyword(KeywordAbility::Flying),
        )
        .object(
            // Has shadow (satisfies shadow restriction) but no flying or reach
            ObjectSpec::creature(p2, "Shadow Ground Blocker", 2, 2)
                .with_keyword(KeywordAbility::Shadow),
        )
        .at_step(Step::DeclareBlockers)
        .active_player(p1)
        .build()
        .unwrap();

    let attacker_id = find_object(&state, "Shadow Flying Attacker");
    let blocker_id = find_object(&state, "Shadow Ground Blocker");

    let mut state = state;
    state.combat = Some({
        let mut cs = mtg_engine::CombatState::new(p1);
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
        "A shadow-only ground creature should not block a shadow+flying attacker (CR 702.28b + CR 702.9a)"
    );
}

#[test]
/// CR 702.28b + CR 702.9a — A blocker with both shadow and flying satisfies both
/// evasion restrictions of a shadow+flying attacker.
fn test_702_28_shadow_plus_flying_satisfied_by_shadow_flying() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(
            ObjectSpec::creature(p1, "Shadow Flying Attacker", 2, 2)
                .with_keyword(KeywordAbility::Shadow)
                .with_keyword(KeywordAbility::Flying),
        )
        .object(
            // Both shadow and flying satisfy the two restrictions
            ObjectSpec::creature(p2, "Shadow Flying Blocker", 2, 2)
                .with_keyword(KeywordAbility::Shadow)
                .with_keyword(KeywordAbility::Flying),
        )
        .at_step(Step::DeclareBlockers)
        .active_player(p1)
        .build()
        .unwrap();

    let attacker_id = find_object(&state, "Shadow Flying Attacker");
    let blocker_id = find_object(&state, "Shadow Flying Blocker");

    let mut state = state;
    state.combat = Some({
        let mut cs = mtg_engine::CombatState::new(p1);
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
        "A shadow+flying creature should block a shadow+flying attacker (CR 702.28b + CR 702.9a): {:?}",
        result.err()
    );
}

#[test]
/// CR 702.28b + CR 702.9a + CR 702.17 — Reach can substitute for flying.
/// A blocker with shadow+reach satisfies a shadow+flying attacker's restrictions.
fn test_702_28_shadow_plus_flying_satisfied_by_shadow_reach() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(
            ObjectSpec::creature(p1, "Shadow Flying Attacker", 2, 2)
                .with_keyword(KeywordAbility::Shadow)
                .with_keyword(KeywordAbility::Flying),
        )
        .object(
            // Shadow satisfies shadow restriction; reach satisfies flying restriction
            ObjectSpec::creature(p2, "Shadow Reach Blocker", 2, 2)
                .with_keyword(KeywordAbility::Shadow)
                .with_keyword(KeywordAbility::Reach),
        )
        .at_step(Step::DeclareBlockers)
        .active_player(p1)
        .build()
        .unwrap();

    let attacker_id = find_object(&state, "Shadow Flying Attacker");
    let blocker_id = find_object(&state, "Shadow Reach Blocker");

    let mut state = state;
    state.combat = Some({
        let mut cs = mtg_engine::CombatState::new(p1);
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
        "A shadow+reach creature should block a shadow+flying attacker (CR 702.28b + CR 702.9a + CR 702.17): {:?}",
        result.err()
    );
}
