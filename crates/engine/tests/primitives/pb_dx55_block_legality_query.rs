//! PB-DX55 Half 2 (`OOS-SIM5-3`, and `OOS-DX51-3` inside it) — CR 509.1a-c blocker
//! legality: `combat::check_block_pair`, `combat::validate_block_declaration` and
//! `queries::legal_blocks`.
//!
//! Before this batch `legal_actions.rs`'s `DeclareBlockers` offer mirrored only 5 of
//! the engine's 4 preamble + 26 per-pair + 2 batch guards, and the engine ITSELF held
//! TWO independent hand-rolled copies of the per-pair restriction list inside
//! `handle_declare_blockers` — the per-pair loop and the CR 702.39a provoke
//! satisfiability mirror — which were not identical: the provoke mirror omitted the
//! phased-out check, `CrossPlayerBlock`, and the within-batch/committed duplicate
//! check. Every test below drives the REAL engine (`mtg_engine::rules::combat`), not
//! a re-derived copy of its rules, per this module's own subject.
//!
//! Per predicate this file asserts BOTH halves together on one fixture — the offer
//! (`queries::legal_blocks`) does NOT contain the illegal pair, AND the engine
//! (`handle_declare_blockers`) refuses it — because either half alone is the shape
//! this queue keeps shipping (`memory/primitives/pb-plan-DX55.md` Half 2).

use mtg_engine::rules::combat;
use mtg_engine::state::error::GameStateError;
use mtg_engine::{
    all_cards, enrich_spec_from_def, legal_blocks, process_command, AttackTarget, CardDefinition,
    Command, GameState, GameStateBuilder, KeywordAbility, ObjectId, ObjectSpec, PlayerId, Step,
    ZoneId,
};
use std::collections::HashMap;

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

fn find_by_name(state: &GameState, name: &str) -> ObjectId {
    state
        .objects()
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object {name:?} not found"))
}

fn load_defs() -> HashMap<String, CardDefinition> {
    all_cards()
        .into_iter()
        .map(|d| (d.name.clone(), d))
        .collect()
}

/// Enrich a REAL card def (never a naked `ObjectSpec::card()` — Architecture
/// Invariant 9 / the naked-object gotcha) onto the battlefield.
fn enrich(owner: PlayerId, name: &str, defs: &HashMap<String, CardDefinition>) -> ObjectSpec {
    let def = defs
        .get(name)
        .unwrap_or_else(|| panic!("no real CardDefinition for {name:?}"));
    enrich_spec_from_def(
        ObjectSpec::card(owner, name)
            .in_zone(ZoneId::Battlefield)
            .with_card_id(def.card_id.clone()),
        defs,
    )
}

// ═══════════════════════════════════════════════════════════════════════════════
// CR 509.1a — the attacking player cannot declare blockers (`OOS-DX51-3`)
// ═══════════════════════════════════════════════════════════════════════════════

/// CR 509.1a: "The defending player chooses which creatures they control, if any,
/// will block." The attacking player is not a defending player and controls no
/// blockable declaration, whichever untapped creatures it has.
///
/// Both halves asserted on one fixture: `legal_blocks` returns nothing for the
/// attacking player (so the offer would never even be built), AND submitting the
/// pair anyway through the real engine is refused.
#[test]
fn t1_attacking_player_offer_absent_and_engine_refuses() {
    let p1 = p(1);
    let p2 = p(2);

    let attacker = ObjectSpec::creature(p1, "P1 Attacker", 2, 2)
        .in_zone(ZoneId::Battlefield)
        .tapped();
    // p1's OWN creature, untapped, that did not attack -- a vigilant attacker or any
    // creature held back is exactly the shape `OOS-DX51-3` found reachable.
    let held_back = ObjectSpec::creature(p1, "P1 Held Back", 3, 3).in_zone(ZoneId::Battlefield);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .active_player(p1)
        .at_step(Step::DeclareBlockers)
        .object(attacker)
        .object(held_back)
        .build()
        .unwrap();

    let attacker_id = find_by_name(&state, "P1 Attacker");
    let held_back_id = find_by_name(&state, "P1 Held Back");

    let mut combat = mtg_engine::CombatState::new(p1);
    combat.add_attacker(attacker_id, AttackTarget::Player(p2));
    *state.combat_mut() = Some(combat);

    // (1) Offer: p1 (the attacking player) is offered NOTHING to block with, even
    // though "P1 Held Back" is untapped and would otherwise be a perfectly eligible
    // creature by every OTHER per-pair guard.
    let offer = legal_blocks(&state, p1);
    assert!(
        offer.is_empty(),
        "CR 509.1a: the attacking player must never be offered a legal block, got {offer:?}"
    );
    // Non-vacuity: p2 (the real defending player) DOES get an offer against the same
    // board, so the empty result above is about p1 specifically, not a fixture that
    // offers nobody anything.
    let held_back_obj = state.objects_mut().get_mut(&held_back_id).unwrap();
    held_back_obj.controller = p2;
    let p2_offer = legal_blocks(&state, p2);
    assert!(
        !p2_offer.is_empty(),
        "non-vacuity: with the SAME creature controlled by the defending player p2, \
         it must be offered as a legal blocker: {p2_offer:?}"
    );
    // Restore control to p1 for the refusal half below.
    state
        .objects_mut()
        .get_mut(&held_back_id)
        .unwrap()
        .controller = p1;

    // (2) Engine: submitting the pair anyway is refused with the CR 509.1a message.
    let err = combat::handle_declare_blockers(&mut state, p1, vec![(held_back_id, attacker_id)])
        .expect_err("CR 509.1a: the attacking player cannot declare blockers");
    match err {
        GameStateError::InvalidCommand(msg) => assert!(
            msg.contains("attacking player cannot declare blockers"),
            "wrong refusal message: {msg:?}"
        ),
        other => panic!("expected InvalidCommand, got {other:?}"),
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// CR 509.1a — CrossPlayerBlock (multiplayer: an attacker not attacking this player)
// ═══════════════════════════════════════════════════════════════════════════════

/// CR 509.1a: a defending player may only block an attacker "that's attacking that
/// player, a planeswalker they control, or a battle they protect." A THIRD player's
/// untapped creature is not a legal blocker for an attack it is not party to.
#[test]
fn t2_cross_player_block_offer_absent_and_engine_refuses() {
    let p1 = p(1); // attacking player
    let p2 = p(2); // the actual defender, attacked
    let p3 = p(3); // bystander -- CrossPlayerBlock's subject

    let attacker = ObjectSpec::creature(p1, "P1 Attacker", 2, 2).in_zone(ZoneId::Battlefield);
    let bystander = ObjectSpec::creature(p3, "P3 Bystander", 2, 2).in_zone(ZoneId::Battlefield);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .active_player(p1)
        .at_step(Step::DeclareBlockers)
        .object(attacker)
        .object(bystander)
        .build()
        .unwrap();

    let attacker_id = find_by_name(&state, "P1 Attacker");
    let bystander_id = find_by_name(&state, "P3 Bystander");

    let mut combat = mtg_engine::CombatState::new(p1);
    combat.add_attacker(attacker_id, AttackTarget::Player(p2));
    *state.combat_mut() = Some(combat);

    // (1) Offer: p3 is never offered this attacker as a legal block target -- the
    // attacker isn't attacking p3 at all, so p3 has no legal_blocks entry whatsoever.
    let offer = legal_blocks(&state, p3);
    assert!(
        offer.is_empty(),
        "CR 509.1a: a bystander's creature must not be offered a block against an \
         attack it is not party to, got {offer:?}"
    );
    // Non-vacuity: the REAL defender p2 has no creatures here, so demonstrate the
    // predicate directly instead -- p2 controlling the SAME creature as p3 would be
    // offered it (this is the "checked, not assumed" half: the exclusion is about
    // WHO the attacker is attacking, not about the creature itself).
    let mut check_ok_state = state.clone();
    check_ok_state
        .objects_mut()
        .get_mut(&bystander_id)
        .unwrap()
        .controller = p2;
    let p2_offer = legal_blocks(&check_ok_state, p2);
    assert!(
        !p2_offer.is_empty(),
        "non-vacuity: the SAME creature controlled by the ACTUAL defender must be \
         offered: {p2_offer:?}"
    );

    // (2) Engine: p3 submitting the pair is refused with CrossPlayerBlock.
    let err = combat::handle_declare_blockers(&mut state, p3, vec![(bystander_id, attacker_id)])
        .expect_err("CR 509.1a: CrossPlayerBlock");
    match err {
        GameStateError::CrossPlayerBlock { blocker, attacker } => {
            assert_eq!(blocker, bystander_id);
            assert_eq!(attacker, attacker_id);
        }
        other => panic!("expected CrossPlayerBlock, got {other:?}"),
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// CR 702.9b — flying (synthetic negative/positive pair)
// ═══════════════════════════════════════════════════════════════════════════════

/// CR 509.1b / CR 702.9b: a creature without flying or reach cannot block a creature
/// with flying. Asserted alongside a POSITIVE case (a reach blocker CAN) on the same
/// fixture, so the exclusion is about the missing keyword, not about the attacker
/// being unblockable outright.
#[test]
fn t3_flying_offer_absent_and_engine_refuses() {
    let p1 = p(1);
    let p2 = p(2);

    let flyer = ObjectSpec::creature(p1, "P1 Flyer", 3, 3)
        .in_zone(ZoneId::Battlefield)
        .with_keyword(KeywordAbility::Flying);
    let grounded = ObjectSpec::creature(p2, "P2 Grounded", 4, 4).in_zone(ZoneId::Battlefield);
    let reacher = ObjectSpec::creature(p2, "P2 Reacher", 1, 1)
        .in_zone(ZoneId::Battlefield)
        .with_keyword(KeywordAbility::Reach);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .active_player(p1)
        .at_step(Step::DeclareBlockers)
        .object(flyer)
        .object(grounded)
        .object(reacher)
        .build()
        .unwrap();

    let flyer_id = find_by_name(&state, "P1 Flyer");
    let grounded_id = find_by_name(&state, "P2 Grounded");
    let reacher_id = find_by_name(&state, "P2 Reacher");

    let mut combat = mtg_engine::CombatState::new(p1);
    combat.add_attacker(flyer_id, AttackTarget::Player(p2));
    *state.combat_mut() = Some(combat);

    // (1) Offer: the grounded creature is entirely absent from p2's legal_blocks
    // (it has no OTHER legal attacker to fall back to in this fixture); the reach
    // creature IS offered against the flyer.
    let offer = legal_blocks(&state, p2);
    let grounded_entry = offer.iter().find(|(id, _)| *id == grounded_id);
    assert!(
        grounded_entry.is_none(),
        "CR 702.9b: a creature with neither flying nor reach must not be offered as a \
         legal blocker of a flying attacker: {offer:?}"
    );
    let reacher_entry = offer
        .iter()
        .find(|(id, _)| *id == reacher_id)
        .unwrap_or_else(|| panic!("non-vacuity: reach creature must be offered: {offer:?}"));
    assert!(
        reacher_entry.1.contains(&flyer_id),
        "CR 702.9b: reach lets a creature block a flyer: {reacher_entry:?}"
    );

    // (2) Engine: submitting the grounded pair anyway is refused.
    let err = combat::handle_declare_blockers(&mut state, p2, vec![(grounded_id, flyer_id)])
        .expect_err("CR 509.1b / CR 702.9b: flying");
    match err {
        GameStateError::InvalidCommand(msg) => {
            assert!(msg.contains("flying"), "wrong refusal message: {msg:?}")
        }
        other => panic!("expected InvalidCommand, got {other:?}"),
    }

    // And the reach pair is genuinely ACCEPTED by the engine, not merely offered.
    let mut state2 = state.clone();
    combat::handle_declare_blockers(&mut state2, p2, vec![(reacher_id, flyer_id)])
        .expect("CR 702.9b: a reach creature can legally block a flyer");
}

// ═══════════════════════════════════════════════════════════════════════════════
// CR 702.9b — card-integration: a real corpus Flying creature
// ═══════════════════════════════════════════════════════════════════════════════

/// Card-integration companion to `t3`: the SAME restriction, driven through a real
/// `Complete`, deck-legal card (`all_cards()`) rather than a synthetic keyword grant,
/// through the full `process_command` path (CR 601 casting is skipped; the creature
/// is placed directly on the battlefield, which is the established convention for
/// combat-only fixtures in this test suite).
#[test]
fn t4_flying_card_integration_aven_riftwatcher() {
    let defs = load_defs();
    let p1 = p(1);
    let p2 = p(2);

    let flyer = enrich(p1, "Aven Riftwatcher", &defs);
    let grounded = ObjectSpec::creature(p2, "P2 Grounded", 4, 4).in_zone(ZoneId::Battlefield);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .active_player(p1)
        .at_step(Step::DeclareBlockers)
        .object(flyer)
        .object(grounded)
        .build()
        .unwrap();

    let flyer_id = find_by_name(&state, "Aven Riftwatcher");
    let grounded_id = find_by_name(&state, "P2 Grounded");

    assert!(
        state
            .objects()
            .get(&flyer_id)
            .unwrap()
            .characteristics
            .keywords
            .contains(&KeywordAbility::Flying),
        "Aven Riftwatcher must carry Flying once enriched from its real CardDefinition"
    );

    let mut combat = mtg_engine::CombatState::new(p1);
    combat.add_attacker(flyer_id, AttackTarget::Player(p2));
    *state.combat_mut() = Some(combat);

    let offer = legal_blocks(&state, p2);
    assert!(
        offer.is_empty(),
        "CR 702.9b (card integration): a grounded creature must not be offered \
         against a real Flying creature: {offer:?}"
    );

    let (_state, result) = (
        state.clone(),
        process_command(
            state,
            Command::DeclareBlockers {
                player: p2,
                blockers: vec![(grounded_id, flyer_id)],
            },
        ),
    );
    assert!(
        result.is_err(),
        "CR 702.9b (card integration): the real engine must refuse the declaration"
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// Differential: `legal_blocks` / `check_block_pair` agree with the ENGINE, computed
// ═══════════════════════════════════════════════════════════════════════════════

/// For a fixture mixing several attackers and blockers under a real per-pair
/// restriction (flying), `queries::legal_blocks` and `combat::check_block_pair`
/// agree with what `handle_declare_blockers` actually accepts, pair for pair --
/// computed by trying EVERY combination through the real engine, never a hand-listed
/// expectation (the shape this queue's own history calls out: a hand-listed
/// expectation is a claim, not a measurement).
#[test]
fn t5_differential_legal_blocks_matches_engine_acceptance() {
    let p1 = p(1);
    let p2 = p(2);

    let flyer = ObjectSpec::creature(p1, "D Flyer", 2, 2)
        .in_zone(ZoneId::Battlefield)
        .with_keyword(KeywordAbility::Flying);
    let grounder = ObjectSpec::creature(p1, "D Grounder", 2, 2).in_zone(ZoneId::Battlefield);
    let ground_blocker =
        ObjectSpec::creature(p2, "D Ground Blocker", 2, 2).in_zone(ZoneId::Battlefield);
    let flying_blocker = ObjectSpec::creature(p2, "D Flying Blocker", 2, 2)
        .in_zone(ZoneId::Battlefield)
        .with_keyword(KeywordAbility::Flying);
    let tapped_blocker = ObjectSpec::creature(p2, "D Tapped Blocker", 2, 2)
        .in_zone(ZoneId::Battlefield)
        .tapped();

    let base_state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .active_player(p1)
        .at_step(Step::DeclareBlockers)
        .object(flyer)
        .object(grounder)
        .object(ground_blocker)
        .object(flying_blocker)
        .object(tapped_blocker)
        .build()
        .unwrap();

    let flyer_id = find_by_name(&base_state, "D Flyer");
    let grounder_id = find_by_name(&base_state, "D Grounder");
    let ground_blocker_id = find_by_name(&base_state, "D Ground Blocker");
    let flying_blocker_id = find_by_name(&base_state, "D Flying Blocker");
    let tapped_blocker_id = find_by_name(&base_state, "D Tapped Blocker");

    let mut base_state = base_state;
    let mut combat = mtg_engine::CombatState::new(p1);
    combat.add_attacker(flyer_id, AttackTarget::Player(p2));
    combat.add_attacker(grounder_id, AttackTarget::Player(p2));
    *base_state.combat_mut() = Some(combat);

    let attackers = [flyer_id, grounder_id];
    let blockers = [ground_blocker_id, flying_blocker_id, tapped_blocker_id];

    // Compute the offer.
    let offer = legal_blocks(&base_state, p2);
    let offer_map: HashMap<ObjectId, Vec<ObjectId>> = offer.into_iter().collect();

    let mut checked_pairs = 0usize;
    for &blocker_id in &blockers {
        for &attacker_id in &attackers {
            checked_pairs += 1;
            // `check_block_pair`'s own verdict.
            let pair_ok =
                combat::check_block_pair(&base_state, p2, blocker_id, attacker_id, &[]).is_ok();
            // `legal_blocks`'s verdict for the SAME pair.
            let offer_ok = offer_map
                .get(&blocker_id)
                .is_some_and(|atks| atks.contains(&attacker_id));
            assert_eq!(
                pair_ok, offer_ok,
                "check_block_pair and legal_blocks disagree on ({blocker_id:?}, \
                 {attacker_id:?}): check_block_pair={pair_ok}, legal_blocks={offer_ok}"
            );
            // The REAL engine, on a fresh clone so each single-pair declaration is
            // judged independently of the others.
            let mut trial = base_state.clone();
            let engine_ok =
                combat::handle_declare_blockers(&mut trial, p2, vec![(blocker_id, attacker_id)])
                    .is_ok();
            assert_eq!(
                pair_ok, engine_ok,
                "check_block_pair and the REAL engine disagree on ({blocker_id:?}, \
                 {attacker_id:?}): check_block_pair={pair_ok}, engine={engine_ok}"
            );
        }
    }
    // Non-vacuity: a real 3x2 cross product was actually walked, and it contains at
    // least one legal and one illegal pair (otherwise every assertion above would be
    // trivially satisfied by "always false" or "always true").
    assert_eq!(
        checked_pairs, 6,
        "the full 3x2 cross product must be walked"
    );
    let any_legal = offer_map.values().any(|atks| !atks.is_empty());
    let any_illegal = !offer_map.contains_key(&tapped_blocker_id);
    assert!(
        any_legal && any_illegal,
        "non-vacuity: this fixture must contain both a legal and an illegal pair, \
         offer={offer_map:?}"
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// CR 702.39a / CR 509.1c — the closed divergence: provoke now shares the SAME
// per-pair predicate the real declaration is validated against
// ═══════════════════════════════════════════════════════════════════════════════

/// Before this batch, the provoke satisfiability mirror never checked CR 702.26b
/// phased-out -- a phased-out provoked creature is still `zone == Battlefield`, so
/// the old mirror proceeded past its "is this relevant to this player" gate, found no
/// other reason to skip, and then INCORRECTLY demanded the (impossible) block,
/// rejecting an otherwise-legal declaration. `check_block_pair` DOES check
/// phased-out, so the requirement is now correctly treated as impossible.
#[test]
fn t6_provoke_treats_a_phased_out_provoked_creature_as_impossible() {
    let p1 = p(1);
    let p2 = p(2);

    let attacker = ObjectSpec::creature(p1, "Provoker", 2, 2).in_zone(ZoneId::Battlefield);
    let provoked = ObjectSpec::creature(p2, "Phased Provoked", 3, 3).in_zone(ZoneId::Battlefield);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .active_player(p1)
        .at_step(Step::DeclareBlockers)
        .object(attacker)
        .object(provoked)
        .build()
        .unwrap();

    let attacker_id = find_by_name(&state, "Provoker");
    let provoked_id = find_by_name(&state, "Phased Provoked");

    // CR 702.26b: phase the provoked creature out. It remains `zone == Battlefield`
    // (phasing is not a zone change) -- the exact shape that let the old provoke
    // mirror's controller/zone gate pass it through unfiltered.
    state
        .objects_mut()
        .get_mut(&provoked_id)
        .unwrap()
        .status
        .phased_out = true;

    let mut combat = mtg_engine::CombatState::new(p1);
    combat.add_attacker(attacker_id, AttackTarget::Player(p2));
    combat.forced_blocks.insert(provoked_id, attacker_id);
    *state.combat_mut() = Some(combat);

    // p2 declares NO blockers at all -- if the requirement were (incorrectly) judged
    // possible, this would be illegal ("Creature must block ... provoke
    // requirement"). With the fix, it is legal: phased-out makes blocking
    // impossible, so the requirement is not imposed.
    combat::handle_declare_blockers(&mut state, p2, vec![]).unwrap_or_else(|e| {
        panic!(
            "CR 702.39a / CR 509.1c: a phased-out provoked creature's requirement must \
             be treated as IMPOSSIBLE, not as a violation. Got: {e:?}"
        )
    });
}

/// Companion sanity: an provoked creature that IS able to block, and does not, is
/// still correctly rejected -- so `t6` is about the phased-out exemption
/// specifically, not about provoke enforcement having been weakened generally.
#[test]
fn t6b_provoke_still_enforced_when_the_provoked_creature_is_able() {
    let p1 = p(1);
    let p2 = p(2);

    let attacker = ObjectSpec::creature(p1, "Provoker2", 2, 2).in_zone(ZoneId::Battlefield);
    let provoked = ObjectSpec::creature(p2, "Able Provoked", 3, 3).in_zone(ZoneId::Battlefield);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .active_player(p1)
        .at_step(Step::DeclareBlockers)
        .object(attacker)
        .object(provoked)
        .build()
        .unwrap();

    let attacker_id = find_by_name(&state, "Provoker2");
    let provoked_id = find_by_name(&state, "Able Provoked");

    let mut combat = mtg_engine::CombatState::new(p1);
    combat.add_attacker(attacker_id, AttackTarget::Player(p2));
    combat.forced_blocks.insert(provoked_id, attacker_id);
    *state.combat_mut() = Some(combat);

    let err = combat::handle_declare_blockers(&mut state, p2, vec![])
        .expect_err("an able provoked creature that does not block must be rejected");
    match err {
        GameStateError::InvalidCommand(msg) => assert!(
            msg.contains("provoke requirement"),
            "wrong refusal message: {msg:?}"
        ),
        other => panic!("expected InvalidCommand, got {other:?}"),
    }

    // And actually blocking with it is accepted.
    let mut state2 = state.clone();
    combat::handle_declare_blockers(&mut state2, p2, vec![(provoked_id, attacker_id)])
        .expect("blocking the provoker with the provoked creature must be accepted");
}
