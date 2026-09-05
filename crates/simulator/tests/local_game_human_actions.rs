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
    let (game, _events) =
        LocalGame::start(state, 1, StubProvider, bots, human_seats, limits(), true)
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

/// Pass priority until the human is offered an action matching `pred`, or give up.
///
/// `start_game` resets the turn to `Step::Untap` (`reset_turn_state`), so a fixture
/// cannot begin in a main phase — and `StubProvider` only offers `CastSpell` for a
/// sorcery in a main phase with an empty stack (CR 307.1). Walking there through real
/// priority passes is the honest way to reach it.
fn drive_until(
    game: &mut LocalGame<StubProvider>,
    pred: impl Fn(&LegalAction) -> bool,
) -> mtg_simulator::PendingDecision {
    for _ in 0..200 {
        let decision = expect_decision(game);
        if decision.actions.iter().any(&pred) {
            return decision;
        }
        let pass = index_of(&decision.actions, |a| {
            matches!(a, LegalAction::PassPriority)
        });
        game.submit(
            decision.seq,
            HumanChoice {
                action_index: pass,
                params: ActionParams::default(),
            },
        )
        .expect("passing priority is always legal at a priority window");
    }
    panic!("no decision offered a matching action within 200 priority windows");
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
    let command =
        mtg_simulator::action_to_command_with_params(&state, P1, action, &ActionParams::default())
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
    state.pending_cumulative_upkeep_payments_mut().push_back((
        P1,
        perm,
        mtg_engine::CumulativeUpkeepCost::Life(1),
    ));

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
        .unwrap_or_else(|| {
            panic!(
                "no OrderBlockers offered in {:?}",
                human_action_list(state, P1)
            )
        })
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

    let command =
        mtg_simulator::action_to_command_with_params(&state, P1, &action, &ActionParams::default())
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

// ── Review MR-M11-09: the repeat cap is per COMBAT, not per turn ─────────────

/// CR 506.5 / 509.1 — `HeuristicBot`'s S8 repeat cap must not stop it blocking in a
/// second combat phase of the same turn.
///
/// The first version of the cap keyed on `turn_number` alone, so a bot that had
/// already declared blockers in combat 1 scored `DeclareBlockers` at 0 in combat 2 —
/// below `PassPriority` — and silently declined to block for the rest of the turn.
/// `aurelia_the_warleader` is `Complete` and deck-legal and grants exactly that extra
/// combat, so this was reachable in ordinary play: a quiet play-quality regression
/// introduced by the fix for a loud stall (review MR-M11-09).
///
/// **Re-scoped from `DeclareAttackers` to `DeclareBlockers` by PB-DX21.** This test
/// used to drive the bot's own `RepeatKey::DeclareAttackers` cap, which no longer
/// exists — CR 508.1's once-per-combat legality is now enforced by the engine
/// (`GameStateError::AlreadyDeclaredAttackers`) and suppressed at the offer layer
/// (`legal_actions.rs`), so `heuristic_bot.rs` has nothing left to guard on the
/// attacker side. `DeclareBlockers` is the surviving combat-scoped `RepeatKey`, and
/// MR-M11-09's finding — the cap must reset on combat-phase entry, not on turn
/// number — applies to it identically, so this probe now exercises that key instead.
///
/// Driven through `Bot::choose_action` directly rather than through a game, because
/// staging a real extra combat needs Aurelia to trigger and resolve; what is under test
/// is the bot's *scope*, and `refresh_repeat_scope` reads exactly two things off the
/// state — the turn number and whether a `CombatState` exists.
#[test]
fn test_mr_m11_09_repeat_cap_resets_on_each_combat_phase() {
    use mtg_simulator::HeuristicBot;

    let blocker_action = LegalAction::DeclareBlockers {
        eligible: vec![ObjectId(1)],
        attackers: vec![ObjectId(2)],
        legal_blocks: vec![(ObjectId(1), vec![ObjectId(2)])],
    };
    let legal = vec![LegalAction::PassPriority, blocker_action];

    // Combat 1: a `CombatState` exists.
    let mut in_combat = two_player_state(Step::DeclareBlockers, Vec::new());
    *in_combat.combat_mut() = Some(mtg_engine::CombatState::new(P2));
    // Between combats: `turn_actions.rs` sets `state.combat = None` at end of combat.
    let mut between = two_player_state(Step::PostCombatMain, Vec::new());
    *between.combat_mut() = None;

    let mut bot = HeuristicBot::new(7, "Bot".to_string());

    // Combat 1 — the bot blocks.
    let first = bot.choose_action(&in_combat, P1, &legal);
    assert!(
        matches!(first, Command::DeclareBlockers { .. }),
        "the bot must block in the first combat; got {first:?}"
    );
    // Still combat 1 — the cap holds, which is the whole point of it.
    let second = bot.choose_action(&in_combat, P1, &legal);
    assert!(
        matches!(second, Command::PassPriority { .. }),
        "CR 509.1: a second declaration in the SAME combat must not be preferred; got {second:?}"
    );

    // Combat ends, then a new combat phase begins on the same turn (CR 506.5).
    let _ = bot.choose_action(&between, P1, &[LegalAction::PassPriority]);
    let third = bot.choose_action(&in_combat, P1, &legal);
    assert!(
        matches!(third, Command::DeclareBlockers { .. }),
        "CR 506.5: the cap must reset on combat-phase entry, so the bot blocks again \
         in the extra combat; got {third:?}"
    );
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

// ── Item 2 pickup: OOS-M11-8, `{X}` is paid for ──────────────────────────────

/// CR 107.3 / 601.2b — **OOS-M11-8**, filed by S7 and closed here.
///
/// `LocalGame::auto_tap_commands_for` read the spell's *printed* `mana_cost` and knew
/// nothing about the announced `x_value`, so casting an `{X}` spell with `auto_tap`
/// tapped for the base cost and the engine then refused the whole cast. S7 observed it
/// as a `422 "player does not have enough mana to pay the cost"`. The S8 workstream
/// handoff routed the seed into this session's item-2 audit; it is the same family as
/// `OOS-M11-2`.
///
/// **This test covers the HUMAN path only, and that mattered** (SIM-2, `scutemob-176`):
/// `LocalGame::advance`'s bot seat had its own `solve_mana_payment` call on the taxed
/// printed cost, so the seed was closed for `submit` and open for `advance` — latent,
/// since no shipped bot announces a non-zero X. The bot half is now closed by `advance`
/// calling this same helper, and pinned separately by
/// `sim2_mana_intelligence::t21_bot_auto_tap_includes_the_announced_x`.
///
/// The fixture is a `{X}{1}` sorcery with four one-mana sources. X = 2 needs three
/// mana, which is **more than the printed cost and less than the board**, so a pass
/// that tapped only for the printed `{1}` would leave the cast unpayable and a pass
/// that ignored the pool entirely would be indistinguishable from success.
#[test]
fn test_s8_x_value_is_included_in_the_auto_tap_plan() {
    use mtg_engine::{
        AbilityDefinition, CardDefinition, CardId, CardRegistry, CardType, Effect, EffectAmount,
        ManaAbility, ManaColor, ManaCost, PlayerTarget, TypeLine,
    };

    let x_cost = ManaCost {
        generic: 1,
        x_count: 1,
        ..ManaCost::default()
    };
    let def = CardDefinition {
        name: "S8 Fireball".to_string(),
        card_id: CardId("s8-fireball".to_string()),
        mana_cost: Some(x_cost.clone()),
        types: TypeLine {
            card_types: [CardType::Sorcery].into_iter().collect(),
            ..Default::default()
        },
        abilities: vec![AbilityDefinition::Spell {
            // Targetless on purpose: this test is about paying, not about CR 601.2c.
            effect: Effect::GainLife {
                player: PlayerTarget::Controller,
                amount: EffectAmount::Fixed(1),
            },
            targets: vec![],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    };

    let mut builder = GameStateBuilder::new()
        .add_player(P1)
        .add_player(P2)
        .with_registry(CardRegistry::new(vec![def.clone()]))
        .active_player(P1)
        .object(
            ObjectSpec::card(P1, &def.name)
                .with_card_id(def.card_id.clone())
                .with_types(vec![CardType::Sorcery])
                .with_mana_cost(x_cost)
                .in_zone(ZoneId::Hand(P1)),
        );
    // Four sources: enough for X = 2 (three mana) with one to spare, so the test
    // distinguishes "tapped what the cost needs" from "tapped everything".
    for i in 0..4 {
        builder = builder.object(
            ObjectSpec::land(P1, &format!("S8 Source {i}")).with_mana_ability(ManaAbility {
                produces: [(ManaColor::Colorless, 1u32)].into_iter().collect(),
                requires_tap: true,
                ..Default::default()
            }),
        );
    }
    let state = builder.build().expect("X-cost fixture should build");

    let mut game = start_human_game(state);
    let decision = drive_until(&mut game, |a| matches!(a, LegalAction::CastSpell { .. }));
    let idx = index_of(&decision.actions, |a| {
        matches!(a, LegalAction::CastSpell { .. })
    });

    game.submit(
        decision.seq,
        HumanChoice {
            action_index: idx,
            params: ActionParams {
                x_value: 2,
                auto_tap: true,
                ..ActionParams::default()
            },
        },
    )
    .expect("CR 107.3: the auto-tap plan must cover the announced X");

    // The spell is on the stack with the announced X, and exactly three sources were
    // tapped — not one (the printed cost) and not four (everything).
    let tapped = game
        .state()
        .objects()
        .iter()
        .filter(|(_, o)| o.characteristics.name.starts_with("S8 Source") && o.status.tapped)
        .count();
    assert_eq!(
        tapped, 3,
        "X = 2 on a {{X}}{{1}} spell costs three mana, so three sources tap"
    );
    assert_eq!(
        game.state().stack_objects().len(),
        1,
        "the spell reached the stack rather than being refused"
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
    let idx = index_of(&decision.actions, |a| {
        matches!(a, LegalAction::PassPriority)
    });

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

// ── PB-DX21 (OOS-M11-9, §2.7): the offer disappears once CR 508.1 is done ────

/// CR 508.1 / SR-38 — `legal_actions.rs` must not offer `DeclareAttackers` once
/// `CombatState::attackers_declared` is set, because `combat.rs::handle_declare_
/// attackers` now refuses a second declaration with
/// `GameStateError::AlreadyDeclaredAttackers`. Before this offer suppression a
/// vigilant attacker (untapped, still `eligible`, per PB-DX21's plan §2.7) would
/// keep the action on the list forever.
///
/// Discriminates directly against the `!c.attackers_declared` condition added at
/// `legal_actions.rs`'s `DeclareAttackers` arm: three states differing only in
/// `combat`/`attackers_declared`, asserted in order so a revert of exactly that
/// condition (commenting out the guard clause) reddens this test and no other in
/// the file.
#[test]
fn test_dx21_declare_attackers_offer_suppressed_once_the_cr_5081_action_is_done() {
    let state = two_player_state(
        Step::DeclareAttackers,
        vec![ObjectSpec::creature(P1, "DX21 Attacker", 2, 2).in_zone(ZoneId::Battlefield)],
    );

    // (1) No CombatState yet (BeginningOfCombat may not have run) — offered.
    assert!(
        StubProvider
            .legal_actions(&state, P1)
            .iter()
            .any(|a| matches!(a, LegalAction::DeclareAttackers { .. })),
        "with no CombatState the CR 508.1 action has not been performed and must \
         still be offered"
    );

    // (2) A fresh CombatState with the marker clear — still offered.
    let mut not_yet_declared = state.clone();
    *not_yet_declared.combat_mut() = Some(mtg_engine::CombatState::new(P1));
    assert!(
        StubProvider
            .legal_actions(&not_yet_declared, P1)
            .iter()
            .any(|a| matches!(a, LegalAction::DeclareAttackers { .. })),
        "attackers_declared == false must still offer DeclareAttackers"
    );

    // (3) The marker set — SUPPRESSED. This is the discriminating assertion.
    let mut already_declared = state;
    let mut combat = mtg_engine::CombatState::new(P1);
    combat.attackers_declared = true;
    *already_declared.combat_mut() = Some(combat);
    let actions = StubProvider.legal_actions(&already_declared, P1);
    assert!(
        !actions
            .iter()
            .any(|a| matches!(a, LegalAction::DeclareAttackers { .. })),
        "CR 508.1 (PB-DX21): once attackers_declared is set the offer must be \
         suppressed (SR-38: the engine will refuse a second declaration with \
         AlreadyDeclaredAttackers); got {actions:?}"
    );
}
