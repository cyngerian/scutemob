//! The decisions a **human** seat is offered (M11-local Session 8, plan items 2, 3
//! and 4).
//!
//! Three separate claims, each with its own section below:
//!
//! * **item 2** — the four "invisible optional decisions" the plan lists. Three of
//!   them (Echo CR 702.30, Cumulative Upkeep CR 702.24, Recover CR 702.58) turned
//!   out to be **already surfaced**; the fourth (blocker damage-assignment order,
//!   CR 509.2) was not and now is. See the premise-correction note below.
//! * **item 3** — `Command::Concede` (CR 104.3a) is reachable from a human seat.
//! * **item 4** — the error-surfacing audit's positive claim: a rejection reaches
//!   the caller and never silently becomes a `PassPriority`.
//!
//! # Premise correction on plan item 2
//!
//! The session plan (written 2026-07-26) lists Echo, Cumulative Upkeep and Recover
//! as decisions that "need new `LegalAction` variants". They do not, any more:
//! **PB-DP4 (`scutemob-152`) shipped `LegalAction::PayEcho` /
//! `PayCumulativeUpkeep` / `PayRecover` on 2026-07-26**, after the plan was
//! written, complete with the SR-38 affordability gating. `params.rs` maps all
//! three, `heuristic_bot.rs` scores them, and `tools/play-server/src/view.rs`
//! labels them. The tests in this file's first section therefore *verify the
//! existing surface reaches a human through `LocalGame`* rather than adding one —
//! which is the half `legal_actions.rs`'s own in-source tests do not cover, since
//! they call `StubProvider::legal_actions` directly.
//!
//! # Why the two new actions are human-only
//!
//! `Concede` and `OrderBlockers` are appended by
//! `mtg_simulator::local_game::human_only_actions`, not by `StubProvider`. Adding
//! anything to the provider's list shifts every `RandomBot` RNG draw downstream of
//! it and so changes what every recorded `mtg-fuzzer` seed reproduces (plan §8
//! R11); and a bot must never auto-concede. `test_s8_bot_seat_is_never_offered_*`
//! below pins both directions.

use std::collections::{BTreeSet, HashMap};

use mtg_engine::{
    AttackTarget, Command, GameState, GameStateBuilder, ManaCost, ManaPool, ObjectId, ObjectSpec,
    PlayerId, Step, ZoneId,
};
use mtg_simulator::{
    human_only_actions, ActionParams, AdvanceOutcome, Bot, HumanChoice, LegalAction,
    LegalActionProvider, LocalGame, LocalGameError, LocalGameLimits, StubProvider,
};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);

fn limits() -> LocalGameLimits {
    LocalGameLimits {
        max_turns: 5,
        max_commands: 1000,
        max_consecutive_passes: 500,
        record_journal: true,
    }
}

fn id_of(state: &GameState, name: &str) -> ObjectId {
    state
        .objects()
        .iter()
        .find(|(_, o)| o.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("no object named {name:?}"))
}

/// A two-player state with `P1` active and holding priority in `step`, plus whatever
/// `objects` the caller wants.
fn two_player_state(step: Step, objects: Vec<ObjectSpec>) -> GameState {
    let mut builder = GameStateBuilder::new()
        .add_player(P1)
        .add_player(P2)
        .active_player(P1);
    for spec in objects {
        builder = builder.object(spec);
    }
    let mut state = builder.build().expect("state builds");
    state.turn_mut().step = step;
    state.turn_mut().priority_holder = Some(P1);
    state
}

/// Start a `LocalGame` with `P1` human and no bots, so `advance()` can only ever stop
/// at `P1`'s decision or halt.
fn start_human_game(state: GameState) -> LocalGame<StubProvider> {
    let human_seats: BTreeSet<PlayerId> = [P1].into_iter().collect();
    let bots: HashMap<PlayerId, Box<dyn Bot>> = HashMap::new();
    let (game, _events) = LocalGame::start(state, 1, StubProvider, bots, human_seats, limits(), true)
        .expect("game should start");
    game
}

/// Exactly the action list `LocalGame::advance()` hands a human seat at a priority
/// window: the provider's own enumeration, then S8's human-only augmentation.
///
/// Composed here rather than driven through `advance()` because `start_game` resets
/// the turn (`reset_turn_state`, `step = Untap`), so a fixture cannot begin in
/// `Step::DeclareBlockers` with a populated `CombatState` or mid-upkeep with an
/// outstanding payment — the state would be wiped before the first `advance()`.
/// `human_only_actions`' own doc records that, and `advance()` composes these two
/// calls and nothing else on this path.
fn human_action_list(state: &GameState, player: PlayerId) -> Vec<LegalAction> {
    let mut actions = StubProvider.legal_actions(state, player);
    actions.extend(human_only_actions(state, player, false));
    actions
}

/// The decision `advance()` yields, or a panic naming what it did instead.
fn expect_decision(game: &mut LocalGame<StubProvider>) -> mtg_simulator::PendingDecision {
    match game.advance() {
        AdvanceOutcome::AwaitingHuman(d) => d,
        other => panic!("expected AwaitingHuman, got {other:?}"),
    }
}

fn index_of(actions: &[LegalAction], pred: impl Fn(&LegalAction) -> bool) -> usize {
    actions
        .iter()
        .position(pred)
        .unwrap_or_else(|| panic!("no matching action in {actions:?}"))
}

// ── Item 2: the payment decisions already reach a human ───────────────────────

/// CR 702.30a — an outstanding echo payment reaches a human seat, with both
/// branches, and the `pay: true` branch is one the engine accepts (SR-38).
#[test]
fn test_s8_echo_payment_reaches_a_human_seat() {
    let mut state = two_player_state(
        Step::Upkeep,
        vec![ObjectSpec::creature(P1, "Echo Permanent", 2, 2).in_zone(ZoneId::Battlefield)],
    );
    let perm = id_of(&state, "Echo Permanent");
    state.pending_echo_payments_mut().push_back((
        P1,
        perm,
        ManaCost {
            generic: 2,
            ..Default::default()
        },
    ));
    state.players_mut().get_mut(&P1).unwrap().mana_pool = ManaPool {
        colorless: 2,
        ..Default::default()
    };

    let actions = human_action_list(&state, P1);
    assert!(
        actions.iter().any(
            |a| matches!(a, LegalAction::PayEcho { permanent, pay: true } if *permanent == perm)
        ),
        "CR 702.30a: an affordable echo payment must be offered; got {actions:?}"
    );
    assert!(
        actions.iter().any(
            |a| matches!(a, LegalAction::PayEcho { permanent, pay: false } if *permanent == perm)
        ),
        "CR 118.12a: declining is always legal"
    );

    // The engine really does accept the `pay: true` branch the human was offered —
    // the SR-38 standard, checked rather than assumed.
    let action = actions
        .iter()
        .find(|a| matches!(a, LegalAction::PayEcho { pay: true, .. }))
        .unwrap();
    let command = mtg_simulator::action_to_command_with_params(
        &state,
        P1,
        action,
        &ActionParams::default(),
    )
    .expect("PayEcho takes no params");
    let (after, _events) =
        mtg_engine::process_command(state, command).expect("the engine must accept it");
    assert!(
        after.pending_echo_payments().is_empty(),
        "the payment is discharged"
    );
}

/// CR 702.24a — cumulative upkeep, same shape.
#[test]
fn test_s8_cumulative_upkeep_payment_reaches_a_human_seat() {
    let mut state = two_player_state(
        Step::Upkeep,
        vec![ObjectSpec::creature(P1, "Upkeep Permanent", 2, 2).in_zone(ZoneId::Battlefield)],
    );
    let perm = id_of(&state, "Upkeep Permanent");
    state
        .pending_cumulative_upkeep_payments_mut()
        .push_back((P1, perm, mtg_engine::CumulativeUpkeepCost::Life(1)));

    let actions = human_action_list(&state, P1);
    assert!(
        actions.iter().any(|a| matches!(
            a,
            LegalAction::PayCumulativeUpkeep { permanent, pay: true } if *permanent == perm
        )),
        "CR 702.24a: an affordable payment must be offered; got {actions:?}"
    );
    assert!(
        actions.iter().any(|a| matches!(
            a,
            LegalAction::PayCumulativeUpkeep { permanent, pay: false } if *permanent == perm
        )),
        "CR 118.12a: declining is always legal"
    );
}

/// CR 702.59a — recover, same shape.
#[test]
fn test_s8_recover_payment_reaches_a_human_seat() {
    let mut state = two_player_state(
        Step::Upkeep,
        vec![ObjectSpec::card(P1, "Recover Card").in_zone(ZoneId::Graveyard(P1))],
    );
    let card = id_of(&state, "Recover Card");
    state
        .pending_recover_payments_mut()
        .push_back((P1, card, ManaCost::default()));

    let actions = human_action_list(&state, P1);
    assert!(
        actions.iter().any(|a| matches!(
            a,
            LegalAction::PayRecover { recover_card, .. } if *recover_card == card
        )),
        "CR 702.59a: the payment must be offered; got {actions:?}"
    );
}

// ── Item 2: OrderBlockers (CR 509.2), the one that was genuinely missing ──────

/// Build a declared combat: `P1` attacks `P2` with one creature, blocked by two.
fn combat_state() -> GameState {
    let mut state = two_player_state(
        Step::DeclareBlockers,
        vec![
            ObjectSpec::creature(P1, "Attacker", 4, 4).in_zone(ZoneId::Battlefield),
            ObjectSpec::creature(P2, "Blocker A", 1, 1).in_zone(ZoneId::Battlefield),
            ObjectSpec::creature(P2, "Blocker B", 1, 1).in_zone(ZoneId::Battlefield),
        ],
    );
    let attacker = id_of(&state, "Attacker");
    let blocker_a = id_of(&state, "Blocker A");
    let blocker_b = id_of(&state, "Blocker B");

    let mut combat = mtg_engine::CombatState::new(P1);
    combat.attackers.insert(attacker, AttackTarget::Player(P2));
    combat.blockers.insert(blocker_a, attacker);
    combat.blockers.insert(blocker_b, attacker);
    *state.combat_mut() = Some(combat);
    state
}

/// The `OrderBlockers` option a human attacker is offered, and its candidate list.
fn order_action(state: &GameState) -> LegalAction {
    human_action_list(state, P1)
        .into_iter()
        .find(|a| matches!(a, LegalAction::OrderBlockers { .. }))
        .unwrap_or_else(|| panic!("no OrderBlockers offered in {:?}", human_action_list(state, P1)))
}

/// CR 509.2: the attacking human is offered a damage-assignment order for an
/// attacker with two blockers, and a submitted permutation is recorded front-to-back.
#[test]
fn test_s8_order_blockers_is_offered_to_a_human_attacker() {
    let state = combat_state();
    let attacker = id_of(&state, "Attacker");
    let blocker_a = id_of(&state, "Blocker A");
    let blocker_b = id_of(&state, "Blocker B");

    let action = order_action(&state);
    match &action {
        LegalAction::OrderBlockers {
            attacker: a,
            blockers,
        } => {
            assert_eq!(*a, attacker);
            assert_eq!(blockers.len(), 2, "both blockers are candidates (CR 509.2)");
            assert!(blockers.contains(&blocker_a) && blockers.contains(&blocker_b));
        }
        other => panic!("expected OrderBlockers, got {other:?}"),
    }

    // Submit the REVERSE of the engine's default order, so the assertion below
    // cannot pass by the default being written back.
    let chosen = vec![blocker_b, blocker_a];
    let command = mtg_simulator::action_to_command_with_params(
        &state,
        P1,
        &action,
        &ActionParams {
            blocker_order: chosen.clone(),
            ..ActionParams::default()
        },
    )
    .expect("blocker_order is a supported param on this action");
    let (after, _events) = mtg_engine::process_command(state, command)
        .expect("the engine must accept an order over exactly this attacker's blockers");

    assert_eq!(
        after
            .combat()
            .as_ref()
            .and_then(|c| c.damage_assignment_order.get(&attacker))
            .cloned(),
        Some(chosen),
        "CR 509.2: the submitted order is recorded front-to-back"
    );
}

/// An empty `blocker_order` means "keep the engine's default", and `params.rs`
/// submits the candidate list verbatim rather than an empty vector — which the
/// engine would refuse with `IncompleteBlockerOrder`.
#[test]
fn test_s8_default_blocker_order_submits_the_candidate_list_verbatim() {
    let state = combat_state();
    let attacker = id_of(&state, "Attacker");
    let action = order_action(&state);
    let LegalAction::OrderBlockers { blockers, .. } = &action else {
        unreachable!()
    };
    let candidates = blockers.clone();

    let command = mtg_simulator::action_to_command_with_params(
        &state,
        P1,
        &action,
        &ActionParams::default(),
    )
    .expect("default params are valid");
    let (after, _events) =
        mtg_engine::process_command(state, command).expect("the default order is complete");
    assert_eq!(
        after
            .combat()
            .as_ref()
            .and_then(|c| c.damage_assignment_order.get(&attacker))
            .cloned(),
        Some(candidates),
        "an empty blocker_order records the candidate list, not an empty order"
    );
}

/// CR 509.2 termination property: once an attacker's order is set, it is not offered
/// again. Without this, a client that always takes the first non-pass action would be
/// re-offered the identical action forever — answering `OrderBlockers` consumes no
/// priority and `handle_order_blockers` accepts it any number of times.
#[test]
fn test_s8_order_blockers_is_not_reoffered_once_set() {
    let mut state = combat_state();
    let attacker = id_of(&state, "Attacker");
    let blocker_a = id_of(&state, "Blocker A");
    let blocker_b = id_of(&state, "Blocker B");
    if let Some(combat) = state.combat_mut().as_mut() {
        combat
            .damage_assignment_order
            .insert(attacker, vec![blocker_a, blocker_b]);
    }

    let actions = human_action_list(&state, P1);
    assert!(
        !actions
            .iter()
            .any(|a| matches!(a, LegalAction::OrderBlockers { .. })),
        "an attacker whose order is set must not be offered again; got {actions:?}"
    );
}

/// An attacker with a single blocker has exactly one permutation, so CR 509.2 has
/// nothing to decide and no action is offered.
#[test]
fn test_s8_order_blockers_is_not_offered_for_a_single_blocker() {
    let mut state = combat_state();
    let blocker_b = id_of(&state, "Blocker B");
    if let Some(combat) = state.combat_mut().as_mut() {
        combat.blockers.remove(&blocker_b);
    }

    let actions = human_action_list(&state, P1);
    assert!(
        !actions
            .iter()
            .any(|a| matches!(a, LegalAction::OrderBlockers { .. })),
        "one blocker is one permutation; got {actions:?}"
    );
}

/// Only the attacking player orders blockers (CR 509.2) — the defender is not asked.
#[test]
fn test_s8_order_blockers_is_not_offered_to_the_defending_player() {
    let state = combat_state();
    let actions = human_only_actions(&state, P2, false);
    assert!(
        !actions
            .iter()
            .any(|a| matches!(a, LegalAction::OrderBlockers { .. })),
        "CR 509.2: only the attacking player orders; got {actions:?}"
    );
}

/// The provider itself must never emit `OrderBlockers` — that is what keeps the
/// fuzzer's `RandomBot` draw sequence unchanged.
#[test]
fn test_s8_bot_seat_is_never_offered_order_blockers() {
    let state = combat_state();
    let actions = StubProvider.legal_actions(&state, P1);
    assert!(
        !actions
            .iter()
            .any(|a| matches!(a, LegalAction::OrderBlockers { .. })),
        "StubProvider must not emit OrderBlockers; got {actions:?}"
    );
}

/// While a `BlockingDecision` is outstanding the engine's admission gate refuses
/// `OrderBlockers`, so it must not be offered then — but `Concede` still is, because
/// the gate exempts it by name.
#[test]
fn test_s8_only_concede_is_offered_while_a_decision_blocks() {
    let state = combat_state();
    let actions = human_only_actions(&state, P1, true);
    assert_eq!(
        actions.len(),
        1,
        "only Concede survives a blocking decision; got {actions:?}"
    );
    assert!(matches!(actions[0], LegalAction::Concede));
}

// ── Item 3: Concede (CR 104.3a) ───────────────────────────────────────────────

/// CR 104.3a — a human seat is offered `Concede` at a plain priority window, and
/// submitting it through the real `LocalGame` removes the player and ends a
/// two-player game with the opponent as winner. This is the one end-to-end path in
/// this file: it needs no pre-`start_game` fixture, so nothing is wiped.
#[test]
fn test_s8_human_can_concede_and_the_game_reports_game_over() {
    let state = two_player_state(Step::PreCombatMain, Vec::new());
    let mut game = start_human_game(state);
    let decision = expect_decision(&mut game);

    let idx = index_of(&decision.actions, |a| matches!(a, LegalAction::Concede));
    game.submit(
        decision.seq,
        HumanChoice {
            action_index: idx,
            params: ActionParams::default(),
        },
    )
    .expect("CR 104.3a: conceding is always legal");

    assert!(
        game.state()
            .player(P1)
            .is_ok_and(|p| p.has_conceded || p.has_lost),
        "the conceding player has left the game"
    );

    // With one player left, `advance()` reports the CR 104.2a conclusion, and the
    // `GameOverView` the play server renders is built from exactly this `GameResult`
    // (`tools/play-server/src/view.rs::game_over_view`).
    match game.advance() {
        AdvanceOutcome::GameOver(result) => {
            assert_eq!(result.winner, Some(P2), "the last player standing wins");
            assert_eq!(result.turn_count, game.state().turn().turn_number);
            // `violations` is deliberately NOT asserted empty here. `check_all` runs
            // after the concede command, at which point the ACTIVE player is the one
            // who just conceded — `check_player_consistency` reports that, correctly,
            // and the game ends before any turn passes so nothing clears it. That is
            // a property of conceding on your own turn, not a defect.
        }
        other => panic!("expected GameOver after the only opponent conceded, got {other:?}"),
    }
}

/// The provider itself must never emit `Concede` — `legal_actions.rs` says so in
/// prose ("bots should never auto-concede"); this makes it checkable.
#[test]
fn test_s8_bot_seat_is_never_offered_concede() {
    let state = two_player_state(Step::PreCombatMain, Vec::new());
    let actions = StubProvider.legal_actions(&state, P1);
    assert!(
        !actions.iter().any(|a| matches!(a, LegalAction::Concede)),
        "StubProvider must not emit Concede; got {actions:?}"
    );
}

// ── Item 4: the error-surfacing audit's positive claim ────────────────────────

/// A param the action has no channel for is a `BadParams` — refused by name, not
/// silently discarded and not reported as an engine rejection.
///
/// The other half of the audit (an *engine* rejection reaching the caller with the
/// state byte-for-byte unchanged, no command counted, no journal entry and the
/// pending decision intact) is already pinned end to end by S3's
/// `local_game.rs::test_human_illegal_target_is_rejected_without_state_change`, so
/// it is cited rather than duplicated here.
#[test]
fn test_s8_an_unsupported_param_is_refused_not_discarded() {
    let state = two_player_state(Step::PreCombatMain, Vec::new());
    let mut game = start_human_game(state);
    let decision = expect_decision(&mut game);
    let idx = index_of(&decision.actions, |a| matches!(a, LegalAction::PassPriority));

    let commands_before = game.command_count();
    let err = game
        .submit(
            decision.seq,
            HumanChoice {
                action_index: idx,
                params: ActionParams {
                    // `PassPriority` has no blocker-order channel.
                    blocker_order: vec![ObjectId(1)],
                    ..ActionParams::default()
                },
            },
        )
        .expect_err("announcing a param this action cannot carry must be refused");

    match &err {
        LocalGameError::BadParams(message) => assert!(
            message.contains("blocker_order"),
            "the message must name the offending field; got {message:?}"
        ),
        other => panic!("expected BadParams, got {other:?}"),
    }
    assert_eq!(
        game.command_count(),
        commands_before,
        "no command was applied — in particular no PassPriority fallback"
    );
    assert!(
        !game
            .journal()
            .iter()
            .any(|r| matches!(r.command, Command::PassPriority { .. })),
        "no PassPriority was ever issued on the human's behalf"
    );
    assert_eq!(
        game.pending_decision().map(|d| d.seq),
        Some(decision.seq),
        "the decision survives its own refused answer"
    );
}
