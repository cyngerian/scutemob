//! PB-DX51 (`scutemob-226`) — CR 509.1a offer soundness (`OOS-DX21-2`): `b1`.
//!
//! `legal_actions.rs`'s `DeclareBlockers` offer gained a conjunct,
//! `!combat.defenders_declared.contains(&player)`, mirroring PB-DX21's attacker-side
//! suppression. `combat::handle_declare_blockers` rejects a second declaration from the
//! same defending player with `GameStateError::AlreadyDeclaredBlockers` (CR 509.1a: each
//! defending player declares blockers exactly once) -- an action the engine will refuse
//! must not be offered (SR-38).
//!
//! This probe asserts on the COUNT/PRESENCE of the `LegalAction::DeclareBlockers`
//! variant specifically, not on the whole action list's length, so it cannot pass
//! vacuously off some unrelated offer changing size.

use mtg_engine::{
    AttackTarget, CombatState, GameState, GameStateBuilder, ObjectId, ObjectSpec, PlayerId, Step,
    ZoneId,
};
use mtg_simulator::{LegalAction, LegalActionProvider, StubProvider};

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

fn count_declare_blockers(actions: &[LegalAction]) -> usize {
    actions
        .iter()
        .filter(|a| matches!(a, LegalAction::DeclareBlockers { .. }))
        .count()
}

#[test]
/// CR 509.1a (PB-DX51, `OOS-DX21-2`): `DeclareBlockers` is offered exactly once to a
/// defending player with a live attacker and an eligible blocker, and NOT offered again
/// once `combat.defenders_declared` already contains that player.
fn b1_declare_blockers_not_offered_once_defenders_declared() {
    let p1 = p(1);
    let p2 = p(2);

    // Tapped, matching CR 508.1f's real post-declaration state for a non-Vigilance
    // attacker -- this also sidesteps a SEPARATE, pre-existing offer-layer gap this
    // probe is not scoped to touch: the DeclareBlockers offer here does not itself
    // check `player == combat.attacking_player` (only `handle_declare_blockers`
    // does), so an UNTAPPED attacking-player creature would otherwise be counted as
    // an "eligible" blocker for the attacking player and defeat assertion (3) below
    // for a reason unrelated to CR 509.1a/OOS-DX21-2. Reported separately in this
    // batch's final report rather than fixed here (out of PB-DX51's scope).
    let attacker = ObjectSpec::creature(p1, "B1 Attacker", 2, 2)
        .in_zone(ZoneId::Battlefield)
        .tapped();
    let blocker = ObjectSpec::creature(p2, "B1 Blocker", 2, 2).in_zone(ZoneId::Battlefield);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .active_player(p1)
        .at_step(Step::DeclareBlockers)
        .object(attacker)
        .object(blocker)
        .build()
        .unwrap();

    let attacker_id = find_by_name(&state, "B1 Attacker");

    let mut combat = CombatState::new(p1);
    combat.add_attacker(attacker_id, AttackTarget::Player(p2));
    *state.combat_mut() = Some(combat);
    // `legal_actions` returns an empty list for anyone who does not currently hold
    // priority (its very first behavioural check, before the DeclareBlockers offer
    // is ever reached) -- CR 509.1a's blocking action is not priority-gated on the
    // ENGINE side, but `StubProvider`'s offer layer only answers the current
    // priority holder, so both players need it in turn to be probed honestly.
    state.turn_mut().priority_holder = Some(p2);

    // (1) Non-vacuity precondition: p2 has an untapped eligible blocker and there is a
    // live attacker to block -- if this were 0, the assertion below would be
    // trivially satisfiable for the wrong reason.
    let actions_before = StubProvider.legal_actions(&state, p2);
    assert_eq!(
        count_declare_blockers(&actions_before),
        1,
        "CR 509.1a: p2 has not yet declared blockers this step and has an eligible \
         untapped blocker plus a live attacker -- DeclareBlockers must be offered \
         exactly once (found {} in {:?})",
        count_declare_blockers(&actions_before),
        actions_before
    );

    // (2) p2 has already declared blockers this step.
    state
        .combat_mut()
        .as_mut()
        .unwrap()
        .defenders_declared
        .insert(p2);

    let actions_after = StubProvider.legal_actions(&state, p2);
    assert_eq!(
        count_declare_blockers(&actions_after),
        0,
        "an action the engine will refuse (GameStateError::AlreadyDeclaredBlockers) must \
         not be offered (found {} in {:?})",
        count_declare_blockers(&actions_after),
        actions_after
    );

    // (3) Sanity: the attacking player p1 was never offered DeclareBlockers at all
    // (CR 509.1a names the DEFENDING player) -- unaffected by defenders_declared, and
    // a companion proving the count-based assertion above is not vacuous in the
    // other direction. Grant p1 priority (see the note above on `legal_actions`
    // requiring the querying player to hold it) so this is a real observation, not
    // another instance of the same early-return.
    state.turn_mut().priority_holder = Some(p1);
    let p1_actions = StubProvider.legal_actions(&state, p1);
    assert_eq!(
        count_declare_blockers(&p1_actions),
        0,
        "the attacking player must never be offered DeclareBlockers"
    );
}
