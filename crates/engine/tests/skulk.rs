//! Skulk evasion ability enforcement tests (CR 702.118).
//!
//! Skulk is a **one-directional, power-based** evasion ability: a creature with
//! skulk can't be blocked by creatures with GREATER power. Equal power IS allowed
//! to block. Unlike Shadow (CR 702.28), skulk only restricts what can block the
//! skulk creature — the skulk creature itself can block anything freely.
//!
//! Tests:
//! - Skulk attacker cannot be blocked by creature with greater power (CR 702.118b)
//! - Skulk attacker CAN be blocked by creature with equal power (CR 702.118b)
//! - Skulk attacker CAN be blocked by creature with lesser power (CR 702.118b)
//! - Skulk is one-directional: skulk creature can block a higher-power attacker (CR 702.118b)
//! - Skulk + Flying: both evasion abilities must be satisfied (CR 702.118b + CR 702.9a)
//! - Zero-power skulk attacker: only creatures with power 0 or less may block (ruling 2016-04-08)
//! - Power pump via continuous effect: uses post-layer power for the comparison (CR 702.118b + CR 613)

use mtg_engine::{
    process_command, AttackTarget, CombatState, Command, ContinuousEffect, EffectDuration,
    EffectFilter, EffectId, EffectLayer, GameStateBuilder, KeywordAbility, LayerModification,
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

// ─── CR 702.118: Skulk ────────────────────────────────────────────────────────

#[test]
/// CR 702.118b — A creature with skulk can't be blocked by creatures with greater
/// power. A 2/1 skulk attacker cannot be blocked by a 3/3 creature.
fn test_702_118_skulk_creature_cannot_be_blocked_by_greater_power() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(
            ObjectSpec::creature(p1, "Skulk Attacker", 2, 1).with_keyword(KeywordAbility::Skulk),
        )
        .object(ObjectSpec::creature(p2, "Big Blocker", 3, 3))
        .at_step(Step::DeclareBlockers)
        .active_player(p1)
        .build()
        .unwrap();

    let attacker_id = find_object(&state, "Skulk Attacker");
    let blocker_id = find_object(&state, "Big Blocker");

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
        "A creature with greater power should not be able to block a skulk attacker (CR 702.118b)"
    );
}

#[test]
/// CR 702.118b — Equal power IS allowed: a skulk attacker with power 2 CAN be
/// blocked by a creature with power 2 (strictly greater, not greater-or-equal).
fn test_702_118_skulk_creature_can_be_blocked_by_equal_power() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(
            ObjectSpec::creature(p1, "Skulk Attacker", 2, 1).with_keyword(KeywordAbility::Skulk),
        )
        .object(ObjectSpec::creature(p2, "Equal Blocker", 2, 2))
        .at_step(Step::DeclareBlockers)
        .active_player(p1)
        .build()
        .unwrap();

    let attacker_id = find_object(&state, "Skulk Attacker");
    let blocker_id = find_object(&state, "Equal Blocker");

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
        "A creature with equal power SHOULD be able to block a skulk attacker \
         (CR 702.118b: restriction is strictly greater, not greater-or-equal): {:?}",
        result.err()
    );
}

#[test]
/// CR 702.118b — Lesser power IS allowed: a 3/3 skulk attacker CAN be blocked
/// by a creature with power 1. The skulk check only fires for strictly greater power.
fn test_702_118_skulk_creature_can_be_blocked_by_lesser_power() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(
            ObjectSpec::creature(p1, "Big Skulk Attacker", 3, 3)
                .with_keyword(KeywordAbility::Skulk),
        )
        .object(ObjectSpec::creature(p2, "Small Blocker", 1, 4))
        .at_step(Step::DeclareBlockers)
        .active_player(p1)
        .build()
        .unwrap();

    let attacker_id = find_object(&state, "Big Skulk Attacker");
    let blocker_id = find_object(&state, "Small Blocker");

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
        "A creature with lesser power SHOULD be able to block a skulk attacker \
         (CR 702.118b): {:?}",
        result.err()
    );
}

#[test]
/// CR 702.118b — Skulk is one-directional: it only restricts what can BLOCK the
/// skulk creature. A skulk creature can freely block any attacker, regardless of
/// the attacker's power. A 2/1 skulk creature can block a 5/5 attacker.
fn test_702_118_skulk_is_one_directional() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    // p1 has a high-power attacker, p2 has a skulk blocker with lower power.
    // The skulk restriction only applies when the skulk creature is the ATTACKER.
    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(ObjectSpec::creature(p1, "Big Attacker", 5, 5))
        .object(ObjectSpec::creature(p2, "Skulk Blocker", 2, 1).with_keyword(KeywordAbility::Skulk))
        .at_step(Step::DeclareBlockers)
        .active_player(p1)
        .build()
        .unwrap();

    let attacker_id = find_object(&state, "Big Attacker");
    let blocker_id = find_object(&state, "Skulk Blocker");

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
        "A skulk creature SHOULD be able to block a higher-power attacker \
         (CR 702.118b: skulk only restricts what blocks IT, not what IT blocks): {:?}",
        result.err()
    );
}

#[test]
/// CR 702.118b + CR 702.9a — Multiple evasion abilities must ALL be satisfied.
/// A skulk+flying attacker requires the blocker to have (flying or reach) AND
/// power not greater than the attacker's power. Three sub-cases:
///   1. Blocker has no flying — fails the flying restriction.
///   2. Blocker has flying but power <= attacker — valid.
///   3. Blocker has flying but power > attacker — fails the skulk restriction.
fn test_702_118_skulk_plus_flying_both_must_be_satisfied() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    // Sub-case 1: Blocker has no flying — blocked by flying restriction, not skulk.
    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(
            ObjectSpec::creature(p1, "Skulk Flying Attacker", 2, 1)
                .with_keyword(KeywordAbility::Skulk)
                .with_keyword(KeywordAbility::Flying),
        )
        .object(ObjectSpec::creature(p2, "Ground Blocker", 1, 1))
        .at_step(Step::DeclareBlockers)
        .active_player(p1)
        .build()
        .unwrap();

    let attacker_id = find_object(&state, "Skulk Flying Attacker");
    let blocker_id = find_object(&state, "Ground Blocker");

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
        "A ground creature should not block a skulk+flying attacker (no flying, CR 702.9a)"
    );

    // Sub-case 2: Blocker has flying and power <= attacker — valid.
    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(
            ObjectSpec::creature(p1, "Skulk Flying Attacker", 2, 1)
                .with_keyword(KeywordAbility::Skulk)
                .with_keyword(KeywordAbility::Flying),
        )
        .object(
            ObjectSpec::creature(p2, "Flying Blocker Small", 1, 1)
                .with_keyword(KeywordAbility::Flying),
        )
        .at_step(Step::DeclareBlockers)
        .active_player(p1)
        .build()
        .unwrap();

    let attacker_id = find_object(&state, "Skulk Flying Attacker");
    let blocker_id = find_object(&state, "Flying Blocker Small");

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
        "A flying creature with power <= skulk attacker SHOULD block \
         (CR 702.118b + CR 702.9a: both restrictions satisfied): {:?}",
        result.err()
    );

    // Sub-case 3: Blocker has flying but power > attacker — fails skulk restriction.
    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(
            ObjectSpec::creature(p1, "Skulk Flying Attacker", 2, 1)
                .with_keyword(KeywordAbility::Skulk)
                .with_keyword(KeywordAbility::Flying),
        )
        .object(
            ObjectSpec::creature(p2, "Flying Blocker Big", 3, 3)
                .with_keyword(KeywordAbility::Flying),
        )
        .at_step(Step::DeclareBlockers)
        .active_player(p1)
        .build()
        .unwrap();

    let attacker_id = find_object(&state, "Skulk Flying Attacker");
    let blocker_id = find_object(&state, "Flying Blocker Big");

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
        "A flying creature with power > skulk attacker should NOT block \
         (CR 702.118b: skulk restriction still applies even with flying)"
    );
}

#[test]
/// CR 702.118b, Ruling 2016-04-08 — Zero-power skulk attacker: a creature with
/// skulk and power 0 can only be blocked by creatures with power 0 or less.
/// A creature with power 1 or greater cannot block it.
fn test_702_118_skulk_zero_power_attacker() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    // Sub-case 1: 0/1 skulk attacked by a 1/1 — blocked (blocker power 1 > attacker power 0).
    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(
            ObjectSpec::creature(p1, "Zero Power Skulk", 0, 1).with_keyword(KeywordAbility::Skulk),
        )
        .object(ObjectSpec::creature(p2, "1/1 Blocker", 1, 1))
        .at_step(Step::DeclareBlockers)
        .active_player(p1)
        .build()
        .unwrap();

    let attacker_id = find_object(&state, "Zero Power Skulk");
    let blocker_id = find_object(&state, "1/1 Blocker");

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
        "A 1/1 should not block a 0-power skulk attacker \
         (CR 702.118b, ruling 2016-04-08: blocker power 1 > attacker power 0)"
    );

    // Sub-case 2: 0/1 skulk attacked by a 0/3 — allowed (0 is not greater than 0).
    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(
            ObjectSpec::creature(p1, "Zero Power Skulk", 0, 1).with_keyword(KeywordAbility::Skulk),
        )
        .object(ObjectSpec::creature(p2, "0/3 Blocker", 0, 3))
        .at_step(Step::DeclareBlockers)
        .active_player(p1)
        .build()
        .unwrap();

    let attacker_id = find_object(&state, "Zero Power Skulk");
    let blocker_id = find_object(&state, "0/3 Blocker");

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
        "A 0/3 SHOULD block a 0-power skulk attacker \
         (CR 702.118b: power 0 is not greater than power 0): {:?}",
        result.err()
    );
}

#[test]
/// CR 702.118b + CR 613 — Skulk uses post-layer power from `calculate_characteristics`.
/// A 2/1 skulk creature buffed to 4/1 by a continuous +2/+0 effect has an effective
/// power of 4, so a 3/3 creature (power 3 < 4) CAN block it.
fn test_702_118_skulk_with_power_pump() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    // Build state with a 2/1 skulk attacker and a 3/3 blocker.
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(
            ObjectSpec::creature(p1, "Skulk Attacker", 2, 1).with_keyword(KeywordAbility::Skulk),
        )
        .object(ObjectSpec::creature(p2, "Medium Blocker", 3, 3))
        .at_step(Step::DeclareBlockers)
        .active_player(p1)
        .build()
        .unwrap();

    // Find the attacker's ObjectId so we can target the pump effect at it.
    let attacker_id = find_object(&state, "Skulk Attacker");
    let blocker_id = find_object(&state, "Medium Blocker");

    // Add a +2/+0 continuous effect targeting the skulk attacker (pumps it to 4/1).
    // After this effect, blocker power 3 < attacker power 4 -- block is legal.
    state.continuous_effects.push_back(ContinuousEffect {
        id: EffectId(1),
        source: None,
        timestamp: 10,
        layer: EffectLayer::PtModify,
        duration: EffectDuration::Indefinite,
        filter: EffectFilter::SingleObject(attacker_id),
        modification: LayerModification::ModifyPower(2),
        is_cda: false,
    });

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
        "A 3/3 SHOULD block a skulk attacker pumped to power 4 \
         (CR 702.118b + CR 613: post-layer power 3 < 4, block is legal): {:?}",
        result.err()
    );
}
