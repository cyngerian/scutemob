//! Acceptance tests for `LocalGame` (M11-local Session 1, `crates/simulator/src/local_game.rs`).
//!
//! CR 117.3 / 117.3a — priority; CR 903.9a — commander zone-change choice.
//!
//! These tests deliberately build games from a fixed, low-complexity 100-card deck
//! (99 Plains + a simple `Complete` legendary creature commander) rather than the
//! full 1,804-card random pool `mtg-fuzzer` draws from. Some interactions in that
//! full pool resolve through
//! recursion deep enough to need a very large thread stack — a pre-existing,
//! out-of-scope engine characteristic observed independently of this session's
//! changes (see the Session 1 completion report), not something these tests should
//! risk tripping over. A land-heavy deck lets many real turns run with none of that
//! risk, while still exercising `advance()`/`submit()` against a real `GameState`
//! built the same way the fuzzer and TUI build one.

use std::collections::{BTreeSet, HashMap};
use std::sync::Arc;

use mtg_engine::{
    all_cards, enrich_spec_from_def, legal_targets_per_slot, spell_target_requirements,
    AttackTarget, CardDefinition, CardId, CardRegistry, Command, GameState, GameStateBuilder,
    ManaAbility, ManaColor, ObjectId, ObjectSpec, PlayerId, Target, TargetRequirement, ZoneId,
};
use mtg_simulator::{
    action_to_command_with_params, build_registry, ActionParams, AdvanceOutcome, Bot, DecisionKind,
    DeckConfig, GameDriver, HaltReason, HumanChoice, LegalAction, LocalGame, LocalGameError,
    LocalGameLimits, ParamError, RandomBot, StubProvider,
};

/// A fixed, low-complexity deck: 99 Plains plus the first `Complete` legendary
/// creature in `cards` as commander (deterministic — `all_cards()` is a plain,
/// statically-ordered `Vec`, not a hash-ordered collection). Plains carries no color
/// identity (CR 903.4), so it is a legal companion to any commander regardless of
/// color — Architecture Invariant 9 just needs both the commander and the main deck
/// to be `Complete` (`GameStateBuilder`/`start_game` do not additionally enforce
/// `validate_deck`'s color-identity or 100-card checks; that gate is Session 2's
/// `setup.rs`, not this session's `LocalGame`).
fn fixed_deck(cards: &[CardDefinition]) -> DeckConfig {
    use mtg_engine::{CardType, SuperType};

    let commander = cards
        .iter()
        .find(|c| {
            c.completeness.is_complete()
                && c.types.supertypes.contains(&SuperType::Legendary)
                && c.types.card_types.contains(&CardType::Creature)
        })
        .expect("at least one Complete legendary creature must exist in the card pool");

    DeckConfig {
        commander: commander.card_id.clone(),
        main_deck: (0..99).map(|_| CardId("plains".to_string())).collect(),
    }
}

/// Build an un-started `GameState` for `player_count` players, each with the fixed
/// deck: commander in the command zone, main deck in the library, `first_turn_of_game`
/// set. Mirrors `mtg-fuzzer::run_single_game`'s builder logic, minus the RNG-driven
/// deck selection.
fn build_state(
    player_count: u32,
    registry: &Arc<CardRegistry>,
    cards: &[CardDefinition],
) -> GameState {
    let player_ids: Vec<PlayerId> = (1..=player_count).map(|i| PlayerId(i as u64)).collect();
    let card_defs: HashMap<String, CardDefinition> =
        cards.iter().map(|c| (c.name.clone(), c.clone())).collect();

    let mut builder = GameStateBuilder::new().with_registry(registry.clone());
    for &pid in &player_ids {
        builder = builder.add_player(pid);
    }

    for &pid in &player_ids {
        let deck = fixed_deck(cards);
        if let Some(def) = cards.iter().find(|c| c.card_id == deck.commander) {
            let spec = ObjectSpec::card(pid, &def.name)
                .in_zone(ZoneId::Command(pid))
                .with_card_id(deck.commander.clone());
            builder = builder.object(enrich_spec_from_def(spec, &card_defs));
        }
        for card_id in &deck.main_deck {
            if let Some(def) = cards.iter().find(|c| c.card_id == *card_id) {
                let spec = ObjectSpec::card(pid, &def.name)
                    .in_zone(ZoneId::Library(pid))
                    .with_card_id(card_id.clone());
                builder = builder.object(enrich_spec_from_def(spec, &card_defs));
            }
        }
    }

    builder
        .first_turn_of_game()
        .build()
        .expect("fixed-deck state should build")
}

/// A fresh `RandomBot` per seat, deterministically seeded from `seed` the same way
/// `mtg-fuzzer::run_single_game` seeds its bots.
fn bots_for(player_count: u32, seed: u64) -> HashMap<PlayerId, Box<dyn Bot>> {
    let mut bots: HashMap<PlayerId, Box<dyn Bot>> = HashMap::new();
    for i in 1..=player_count as u64 {
        let bot_seed = seed.wrapping_add(100 + i);
        bots.insert(
            PlayerId(i),
            Box::new(RandomBot::new(bot_seed, format!("Bot-{i}"))),
        );
    }
    bots
}

fn small_limits(max_turns: u32) -> LocalGameLimits {
    LocalGameLimits {
        max_turns,
        max_commands: max_turns * 200,
        max_consecutive_passes: 500,
        // On for tests: the journal assertions below depend on it, and this is the
        // configuration the play server will use. `GameDriver` sets it `false`.
        record_journal: true,
    }
}

/// Find the index of the first action in `actions` matching `pred`. M11-local
/// Session 3 (item 7) made `HumanChoice` carry an `action_index` into the pending
/// decision's `actions` rather than a pre-built `Command`, so every test that used
/// to construct a `Command` directly now has to locate the equivalent
/// `LegalAction`'s position first.
fn find_action_index(actions: &[LegalAction], pred: impl Fn(&LegalAction) -> bool) -> usize {
    actions
        .iter()
        .position(pred)
        .unwrap_or_else(|| panic!("no matching action found in {:?}", actions))
}

/// The most common case in this file: locate `LegalAction::PassPriority` and
/// submit it with default params.
fn submit_pass<P: mtg_simulator::LegalActionProvider>(
    game: &mut LocalGame<P>,
    decision: &mtg_simulator::PendingDecision,
) -> Result<Vec<mtg_engine::GameEvent>, LocalGameError> {
    let idx = find_action_index(&decision.actions, |a| {
        matches!(a, LegalAction::PassPriority)
    });
    game.submit(
        decision.seq,
        HumanChoice {
            action_index: idx,
            params: ActionParams::default(),
        },
    )
}

/// A bot that always passes priority, regardless of what else is legal. Used to
/// deterministically and quickly trip `LocalGameLimits::max_consecutive_passes`.
struct AlwaysPassBot;

impl Bot for AlwaysPassBot {
    fn choose_action(
        &mut self,
        _state: &GameState,
        player: PlayerId,
        _legal: &[LegalAction],
    ) -> Command {
        Command::PassPriority { player }
    }

    fn choose_targets(
        &mut self,
        _state: &GameState,
        _valid: &[ObjectId],
        _count: usize,
    ) -> Vec<ObjectId> {
        Vec::new()
    }

    fn choose_attackers(
        &mut self,
        _state: &GameState,
        _eligible: &[ObjectId],
        _targets: &[AttackTarget],
    ) -> Vec<(ObjectId, AttackTarget)> {
        Vec::new()
    }

    fn choose_blockers(
        &mut self,
        _state: &GameState,
        _eligible: &[ObjectId],
        _attackers: &[ObjectId],
    ) -> Vec<(ObjectId, ObjectId)> {
        Vec::new()
    }

    fn choose_mulligan_bottom(&mut self, _hand: &[ObjectId], _count: usize) -> Vec<ObjectId> {
        Vec::new()
    }

    fn name(&self) -> &str {
        "AlwaysPassBot"
    }
}

/// Session 1 acceptance: `GameDriver::run_game` (the pre-existing bot-only path) and
/// a directly-driven `LocalGame` with zero human seats must reach the identical
/// winner / turn count / command count for the same seed and initial state — proving
/// `GameDriver` is now a thin wrapper over `LocalGame` rather than a second
/// implementation of the loop.
#[test]
fn test_local_game_bot_only_matches_game_driver_for_fixed_seeds() {
    let registry = build_registry();
    let cards = all_cards();
    let max_turns = 15;

    for seed in [1u64, 2, 3, 4, 5] {
        let state_a = build_state(4, &registry, &cards);
        let state_b = build_state(4, &registry, &cards);

        let driver = GameDriver::new(StubProvider, bots_for(4, seed), max_turns, seed);
        let result_a = driver.run_game(state_a, seed);

        let (mut game, _start_events) = LocalGame::start(
            state_b,
            seed,
            StubProvider,
            bots_for(4, seed),
            BTreeSet::new(),
            small_limits(max_turns),
            true,
        )
        .expect("game should start");

        let (winner_b, turn_count_b, total_commands_b) = match game.advance() {
            AdvanceOutcome::GameOver(r) => (r.winner, r.turn_count, r.total_commands),
            AdvanceOutcome::Halted(_) => (
                None,
                game.state().turn().turn_number,
                game.command_count() as usize,
            ),
            AdvanceOutcome::AwaitingHuman(_) => panic!("no human seats configured"),
        };

        assert_eq!(
            result_a.winner, winner_b,
            "seed {seed}: winner mismatch between GameDriver and LocalGame"
        );
        assert_eq!(
            result_a.turn_count, turn_count_b,
            "seed {seed}: turn_count mismatch between GameDriver and LocalGame"
        );
        assert_eq!(
            result_a.total_commands, total_commands_b,
            "seed {seed}: total_commands mismatch between GameDriver and LocalGame"
        );
    }
}

/// CR 117.3a — the moment a human-occupied seat holds priority, `advance()` must
/// stop and hand back a `PendingDecision` rather than acting on the human's behalf.
#[test]
fn test_local_game_halts_awaiting_human_at_first_priority() {
    let registry = build_registry();
    let cards = all_cards();
    let state = build_state(2, &registry, &cards);

    let human_seats: BTreeSet<PlayerId> = [PlayerId(1)].into_iter().collect();
    let (mut game, _start_events) = LocalGame::start(
        state,
        99,
        StubProvider,
        bots_for(2, 99),
        human_seats,
        small_limits(10),
        true,
    )
    .expect("game should start");

    match game.advance() {
        AdvanceOutcome::AwaitingHuman(decision) => {
            assert_eq!(decision.player, PlayerId(1));
            assert_eq!(decision.seq, 1);
            assert!(!decision.actions.is_empty());
        }
        other => panic!("expected AwaitingHuman, got {:?}", other),
    }
}

/// `advance()` is idempotent while a decision is outstanding. A play server will call
/// it from a poll or keepalive endpoint, and a browser refresh will call it again; if
/// each call minted a fresh `seq`, the `seq` the client is holding would be silently
/// invalidated and its `submit()` would fail against a `seq` it never saw.
#[test]
fn test_local_game_repeated_advance_preserves_pending_decision() {
    let registry = build_registry();
    let cards = all_cards();
    let state = build_state(2, &registry, &cards);

    let human_seats: BTreeSet<PlayerId> = [PlayerId(1)].into_iter().collect();
    let (mut game, _start_events) = LocalGame::start(
        state,
        99,
        StubProvider,
        bots_for(2, 99),
        human_seats,
        small_limits(10),
        true,
    )
    .expect("game should start");

    let first = match game.advance() {
        AdvanceOutcome::AwaitingHuman(d) => d,
        other => panic!("expected AwaitingHuman, got {:?}", other),
    };
    let commands_after_first = game.command_count();

    for call in 0..3 {
        match game.advance() {
            AdvanceOutcome::AwaitingHuman(again) => {
                assert_eq!(
                    again.seq, first.seq,
                    "advance() call {} minted a new seq for an unanswered decision",
                    call
                );
                assert_eq!(again.player, first.player);
                assert_eq!(again.actions.len(), first.actions.len());
            }
            other => panic!("expected AwaitingHuman, got {:?}", other),
        }
        assert_eq!(
            game.command_count(),
            commands_after_first,
            "a re-entrant advance() must not apply any command"
        );
    }

    // The seq handed out first is still the one that works.
    submit_pass(&mut game, &first).expect("the originally-issued seq must still be valid");
}

/// Session 1's cross-seat guarantee ("the seat that was asked is the only seat that
/// may answer") is now STRUCTURAL rather than checked (M11-local Session 3, item 7):
/// `submit` resolves `action_index` against `pending.actions` and always builds the
/// `Command` for `pending.player` — there is no field anywhere in `HumanChoice` that
/// could name a different player, so a cross-seat submission is unrepresentable at
/// the type level and `LocalGameError::BadParams` (Session 1's runtime check) no
/// longer has anything to report. What remains checkable at runtime is an
/// out-of-range `action_index`, which is exactly what a client attempting to act
/// on a decision it does not hold degenerates into.
#[test]
fn test_local_game_submit_unknown_action_index_is_rejected() {
    let registry = build_registry();
    let cards = all_cards();
    let state = build_state(2, &registry, &cards);

    let human_seats: BTreeSet<PlayerId> = [PlayerId(1)].into_iter().collect();
    let (mut game, _start_events) = LocalGame::start(
        state,
        11,
        StubProvider,
        bots_for(2, 11),
        human_seats,
        small_limits(10),
        true,
    )
    .expect("game should start");

    let decision = match game.advance() {
        AdvanceOutcome::AwaitingHuman(d) => d,
        other => panic!("expected AwaitingHuman, got {:?}", other),
    };
    assert_eq!(decision.player, PlayerId(1));

    let commands_before = game.command_count();
    let out_of_range = decision.actions.len(); // one past the last valid index

    let result = game.submit(
        decision.seq,
        HumanChoice {
            action_index: out_of_range,
            params: ActionParams::default(),
        },
    );

    assert!(
        matches!(result, Err(LocalGameError::UnknownAction(i)) if i == out_of_range),
        "expected UnknownAction({out_of_range}), got {:?}",
        result
    );
    assert_eq!(
        game.command_count(),
        commands_before,
        "an unknown action_index must not apply anything"
    );
    assert_eq!(
        game.pending_decision().map(|d| d.seq),
        Some(decision.seq),
        "a rejected submit must not consume the decision"
    );
}

/// `LocalGameLimits::record_journal` gates the journal. The play server needs it; the
/// fuzzer must not pay for it (`GameDriver` runs thousands of long games in parallel
/// and discards events, where the pre-M11 driver retained nothing).
#[test]
fn test_local_game_journal_can_be_disabled() {
    let registry = build_registry();
    let cards = all_cards();
    let state = build_state(2, &registry, &cards);

    let mut limits = small_limits(3);
    limits.record_journal = false;

    let (mut game, _start_events) = LocalGame::start(
        state,
        21,
        StubProvider,
        bots_for(2, 21),
        BTreeSet::new(),
        limits,
        true,
    )
    .expect("game should start");

    let _ = game.advance();

    assert!(
        game.command_count() > 0,
        "the game must actually have run commands for this test to mean anything"
    );
    assert!(
        game.journal().is_empty(),
        "journal must stay empty when record_journal is false"
    );
    assert!(game.journal_since(0).is_empty());
}

// `test_local_game_submit_illegal_command_returns_err_and_preserves_state` used to
// live here (Session 1): it submitted `Command::PlayLand { card: ObjectId(999_999),
// .. }` — a bogus, unrepresentable-by-any-`LegalAction` command — and asserted
// `LocalGameError::Rejected` with the game state untouched. M11-local Session 3
// (item 7) removed the ability to submit an arbitrary `Command` at all: `submit`
// now only ever builds a `Command` from an `action_index` into `pending.actions`
// plus `ActionParams`, so there is no longer a way to construct that scenario.
// DELETED rather than repurposed: the invariant it protected ("an engine
// rejection at submit time leaves `self.state` untouched") is not lost — it is
// exercised, with a real offered action and a real engine-level rejection, by
// `test_human_illegal_target_is_rejected_without_state_change` below (item 8),
// which is a strictly stronger test of the same property because the rejected
// command is one `action_to_command_with_params` actually produced, not one
// hand-built to be nonsensical.

/// A stale `seq` (one not matching the current `PendingDecision`) is rejected without
/// touching game state — protects against a stale browser tab acting on a superseded
/// action list.
#[test]
fn test_local_game_submit_stale_seq_rejected() {
    let registry = build_registry();
    let cards = all_cards();
    let state = build_state(2, &registry, &cards);

    let human_seats: BTreeSet<PlayerId> = [PlayerId(1)].into_iter().collect();
    let (mut game, _start_events) = LocalGame::start(
        state,
        8,
        StubProvider,
        bots_for(2, 8),
        human_seats,
        small_limits(10),
        true,
    )
    .expect("game should start");

    let decision = match game.advance() {
        AdvanceOutcome::AwaitingHuman(d) => d,
        other => panic!("expected AwaitingHuman, got {:?}", other),
    };
    assert_eq!(decision.seq, 1);

    let stale_seq = decision.seq + 1; // Never issued.
    let idx = find_action_index(&decision.actions, |a| {
        matches!(a, LegalAction::PassPriority)
    });
    let result = game.submit(
        stale_seq,
        HumanChoice {
            action_index: idx,
            params: ActionParams::default(),
        },
    );

    match result {
        Err(LocalGameError::StaleDecision { expected, got }) => {
            assert_eq!(expected, decision.seq);
            assert_eq!(got, stale_seq);
        }
        other => panic!("expected StaleDecision, got {:?}", other),
    }
}

/// Every applied command (human `submit` and autonomous bot seats alike) is recorded
/// in the journal exactly once; `journal().len()` always matches `command_count()`.
#[test]
fn test_local_game_journal_length_matches_commands() {
    let registry = build_registry();
    let cards = all_cards();
    let state = build_state(3, &registry, &cards);

    let human_seats: BTreeSet<PlayerId> = [PlayerId(1)].into_iter().collect();
    let (mut game, _start_events) = LocalGame::start(
        state,
        11,
        StubProvider,
        bots_for(3, 11),
        human_seats,
        small_limits(5),
        true,
    )
    .expect("game should start");

    // Drive the human seat with an always-pass policy until the game concludes, is
    // halted, or a generous step cap is hit (defends against a hang if this policy is
    // ever wrong for some future default state).
    for _ in 0..2000 {
        match game.advance() {
            AdvanceOutcome::AwaitingHuman(decision) => {
                submit_pass(&mut game, &decision)
                    .expect("PassPriority is always legal when it is this player's turn to act");
            }
            AdvanceOutcome::GameOver(_) | AdvanceOutcome::Halted(_) => break,
        }
    }

    assert!(!game.journal().is_empty());
    assert_eq!(game.journal().len() as u32, game.command_count());

    // journal_since is consistent with the full journal.
    let half = game.journal().len() / 2;
    assert_eq!(game.journal_since(half).len(), game.journal().len() - half);
    assert!(game.journal_since(game.journal().len()).is_empty());
}

/// `LocalGameLimits::max_consecutive_passes` is a real safety valve: an all-pass bot
/// game halts with `HaltReason::InfiniteLoop` rather than running forever.
#[test]
fn test_local_game_max_consecutive_passes_halts() {
    let registry = build_registry();
    let cards = all_cards();
    let state = build_state(2, &registry, &cards);

    let mut bots: HashMap<PlayerId, Box<dyn Bot>> = HashMap::new();
    bots.insert(PlayerId(1), Box::new(AlwaysPassBot));
    bots.insert(PlayerId(2), Box::new(AlwaysPassBot));

    let limits = LocalGameLimits {
        max_turns: 200,
        max_commands: 100_000,
        max_consecutive_passes: 5,
        record_journal: true,
    };
    let (mut game, _start_events) =
        LocalGame::start(state, 42, StubProvider, bots, BTreeSet::new(), limits, true)
            .expect("game should start");

    match game.advance() {
        AdvanceOutcome::Halted(HaltReason::InfiniteLoop { .. }) => {}
        other => panic!("expected Halted(InfiniteLoop), got {:?}", other),
    }
}

// ── PB-DP7 / DP-3 (CR 514.1): T14/T15, the cleanup discard reaches LocalGame ─

/// A 2-player un-started `GameState` with P1 active and 8 filler cards
/// already in P1's hand (over the default max hand size of 7). No library
/// cards are needed: CR 103.8a's 2-player first-turn draw-skip
/// (`state.turn.is_first_turn_of_game && state.players.len() <= 2`) means P1
/// never draws before reaching Cleanup, so the hand stays at 8 and Cleanup
/// pauses on turn 1 -- the fastest deterministic route to the pending
/// decision, without threading a full deck/library setup through it.
fn state_with_oversized_hand_for_p1() -> GameState {
    let mut builder = GameStateBuilder::new()
        .add_player(PlayerId(1))
        .add_player(PlayerId(2))
        .active_player(PlayerId(1));
    for i in 0..8u32 {
        builder = builder.object(
            ObjectSpec::card(PlayerId(1), &format!("Filler {i}"))
                .in_zone(ZoneId::Hand(PlayerId(1))),
        );
    }
    builder.build().expect("oversized-hand state should build")
}

/// Drive `game` forward, answering every ordinary `Priority` decision with
/// `PassPriority`, until the `CleanupDiscard` decision (PB-DP7 / DP-3)
/// appears. Panics if the game ends or halts first -- that would mean the
/// fixture stopped producing the pause this test exists to observe.
fn drive_to_cleanup_discard<P: mtg_simulator::LegalActionProvider>(
    game: &mut LocalGame<P>,
) -> mtg_simulator::PendingDecision {
    loop {
        match game.advance() {
            AdvanceOutcome::AwaitingHuman(d) if d.kind == DecisionKind::CleanupDiscard => {
                return d;
            }
            AdvanceOutcome::AwaitingHuman(d) => {
                submit_pass(game, &d)
                    .unwrap_or_else(|e| panic!("PassPriority submit failed: {:?}", e));
            }
            other => panic!(
                "expected to reach CleanupDiscard, got a terminal outcome instead: {:?}",
                other
            ),
        }
    }
}

/// T14: CR 117.3a-style stop, but for the engine's first BLOCKING decision
/// (CR 514.1, PB-DP7 / DP-3). A human seat is offered exactly one
/// `DiscardToHandSize` action; a second `advance()` returns the SAME `seq`
/// (S1's idempotence guard); `submit` rejects a command naming another seat
/// (`BadParams`) and a stale `seq` (`StaleDecision`); the correct `submit`
/// lets the game proceed.
#[test]
fn test_dp7_local_game_awaits_human_on_cleanup_discard() {
    let state = state_with_oversized_hand_for_p1();
    let human_seats: BTreeSet<PlayerId> = [PlayerId(1)].into_iter().collect();
    let mut bots: HashMap<PlayerId, Box<dyn Bot>> = HashMap::new();
    bots.insert(
        PlayerId(2),
        Box::new(RandomBot::new(1, "Bot-2".to_string())),
    );
    let (mut game, _start_events) = LocalGame::start(
        state,
        1,
        StubProvider,
        bots,
        human_seats,
        small_limits(5),
        true,
    )
    .expect("game should start");

    let decision = drive_to_cleanup_discard(&mut game);
    assert_eq!(decision.player, PlayerId(1));
    // 2, not 1: the PROVIDER offers exactly the one answer (which
    // `test_dp8_stub_provider_offers_only_the_answer` pins directly against
    // `StubProvider`), and M11-local S8 appends `Concede` (CR 104.3a) for a human
    // seat in `LocalGame::human_only_actions`. Concede is legal here — the engine's
    // `BlockingDecision` admission gate exempts it explicitly — so offering it is
    // not a widening of what the engine would accept.
    assert_eq!(decision.actions.len(), 2);
    assert_eq!(
        decision
            .actions
            .iter()
            .filter(|a| matches!(a, LegalAction::Concede))
            .count(),
        1,
        "exactly one Concede, appended by the human-only augmentation"
    );
    match &decision.actions[0] {
        LegalAction::DiscardToHandSize { count, hand, cards } => {
            assert_eq!(*count, 1);
            assert_eq!(hand.len(), 8);
            assert_eq!(cards.len(), 1);
        }
        other => panic!("expected DiscardToHandSize, got {:?}", other),
    };

    // Idempotence (S1's guard): a second advance() with the decision still
    // outstanding returns the SAME seq, not a freshly issued one.
    match game.advance() {
        AdvanceOutcome::AwaitingHuman(d2) => assert_eq!(d2.seq, decision.seq),
        other => panic!("expected the same AwaitingHuman again, got {:?}", other),
    }

    // The Session 1 "submit() naming another seat -> BadParams" sub-check that used
    // to live here is gone: `HumanChoice` (Session 3) has no field that can name a
    // player at all, so there is nothing left to submit "for player 2" with — see
    // `test_local_game_submit_unknown_action_index_is_rejected`, which covers what
    // that scenario degenerates into (an out-of-range `action_index`).

    // submit() with a stale seq -> StaleDecision.
    let stale = game.submit(
        decision.seq + 100,
        HumanChoice {
            action_index: 0,
            params: ActionParams::default(),
        },
    );
    assert!(matches!(
        stale,
        Err(LocalGameError::StaleDecision { expected, got })
            if expected == decision.seq && got == decision.seq + 100
    ));

    // Correct submit -> the game proceeds (no error, and the decision clears).
    // `DiscardToHandSize` ignores `params` and forwards the engine's own default
    // `cards` verbatim (SR-38), so `ActionParams::default()` is sufficient.
    let ok = game.submit(
        decision.seq,
        HumanChoice {
            action_index: 0,
            params: ActionParams::default(),
        },
    );
    assert!(
        ok.is_ok(),
        "the correct answer must be accepted: {:?}",
        ok.err()
    );
}

/// T15: a bot-only game does not halt at a cleanup discard -- `RandomBot`
/// (via its new `LegalAction::DiscardToHandSize` arm) submits the provider's
/// deterministic default subset, exactly as SR-38 requires (the provider
/// never offers an action the engine rejects), and the game keeps running.
#[test]
fn test_dp7_local_game_bot_seat_auto_answers() {
    let state = state_with_oversized_hand_for_p1();
    let mut bots: HashMap<PlayerId, Box<dyn Bot>> = HashMap::new();
    bots.insert(
        PlayerId(1),
        Box::new(RandomBot::new(1, "Bot-1".to_string())),
    );
    bots.insert(
        PlayerId(2),
        Box::new(RandomBot::new(2, "Bot-2".to_string())),
    );
    let (mut game, _start_events) = LocalGame::start(
        state,
        7,
        StubProvider,
        bots,
        BTreeSet::new(),
        small_limits(3),
        true,
    )
    .expect("game should start");

    let outcome = game.advance();
    // The regression this guards is a bot game halting (or looping) because
    // a rejected/impossible cleanup-discard command left both the action and
    // its PassPriority fallback failing. `EngineError`/`NoLegalActions` are
    // exactly that failure mode; `MaxTurns`/`InfiniteLoop` or a natural
    // `GameOver` (P2's empty library, CR 104.3b) are unrelated, benign stops.
    assert!(
        !matches!(
            outcome,
            AdvanceOutcome::Halted(HaltReason::EngineError(_))
                | AdvanceOutcome::Halted(HaltReason::NoLegalActions { .. })
        ),
        "a bot-only game must not halt because of a rejected cleanup discard: {:?}",
        outcome
    );
    // At least one DiscardToHandSize command must have been applied and
    // journalled -- the regression this test guards is a bot game silently
    // halting (or looping) at cleanup instead of answering.
    assert!(
        game.journal()
            .iter()
            .any(|r| matches!(r.command, Command::DiscardToHandSize { .. })),
        "the journal must record at least one DiscardToHandSize command"
    );
}

// ── PB-DP8 / DP-6 (CR 603.3d): T16/T17/T18 — trigger targets reach LocalGame ──

/// A 2-player un-started `GameState` where P1 controls an enchantment whose
/// triggered ability declares one `TargetCreature` slot, plus TWO legal creature
/// targets, and one `PendingTrigger` already queued for it.
///
/// Queuing the `PendingTrigger` directly (rather than staging a real ETB) is the
/// deterministic route to the CR 603.3d pause: `LocalGame` drives whole turns, and
/// the first thing that flushes pending triggers puts the announcement in front of
/// the acting seat. Two creatures means two legal choices, so CR 601.2c's
/// forced-choice narrowing does NOT apply and the engine must ask.
fn state_with_pending_targeted_trigger() -> (GameState, ObjectId) {
    use mtg_engine::cards::card_definition::TargetRequirement;
    use mtg_engine::state::stubs::{PendingTrigger, PendingTriggerKind};
    use mtg_engine::{CardEffectTarget, Effect, EffectAmount, TriggerEvent, TriggeredAbilityDef};

    let zapper = ObjectSpec::enchantment(PlayerId(1), "DP8 Zapper").with_triggered_ability(
        TriggeredAbilityDef {
            trigger_on: TriggerEvent::AnyPermanentEntersBattlefield,
            intervening_if: None,
            description: "PB-DP8 fixture: deal 2 damage to the declared target".to_string(),
            effect: Some(Effect::DealDamage {
                source: None,
                target: CardEffectTarget::DeclaredTarget { index: 0 },
                amount: EffectAmount::Fixed(2),
            }),
            etb_filter: None,
            death_filter: None,
            combat_damage_filter: None,
            triggering_creature_filter: None,
            targets: vec![TargetRequirement::TargetCreature],
            counter_filter: None,
            counter_on_self: false,
            once_per_turn: false,
        },
    );

    let mut state = GameStateBuilder::new()
        .add_player(PlayerId(1))
        .add_player(PlayerId(2))
        .active_player(PlayerId(1))
        .object(zapper)
        .object(ObjectSpec::creature(PlayerId(1), "Target A", 2, 2))
        .object(ObjectSpec::creature(PlayerId(1), "Target B", 2, 2))
        .build()
        .expect("PB-DP8 fixture should build");

    let zapper_id = state
        .objects()
        .iter()
        .find(|(_, o)| o.characteristics.name == "DP8 Zapper")
        .map(|(id, _)| *id)
        .expect("zapper must exist");

    state
        .pending_triggers_mut()
        .push_back(PendingTrigger::blank(
            zapper_id,
            PlayerId(1),
            PendingTriggerKind::Normal,
        ));
    (state, zapper_id)
}

/// T16: CR 603.3d reaches `LocalGame` as its OWN `DecisionKind`.
///
/// The `kind == TriggerTargets` assertion is the point: before PB-DP8,
/// `advance()`'s acting-player chain hard-coded `DecisionKind::CleanupDiscard`
/// for every `BlockingDecision`, so a browser client would have been handed the
/// wrong picker. Also re-pins S1's idempotence guard, the foreign-seat rejection
/// and the stale-`seq` rejection against the new decision class.
#[test]
fn test_dp8_local_game_awaits_human_on_trigger_targets() {
    let (state, _zapper) = state_with_pending_targeted_trigger();
    let human_seats: BTreeSet<PlayerId> = [PlayerId(1)].into_iter().collect();
    let mut bots: HashMap<PlayerId, Box<dyn Bot>> = HashMap::new();
    bots.insert(
        PlayerId(2),
        Box::new(RandomBot::new(1, "Bot-2".to_string())),
    );
    let (mut game, _start_events) = LocalGame::start(
        state,
        1,
        StubProvider,
        bots,
        human_seats,
        small_limits(20),
        true,
    )
    .expect("game should start");

    let decision = loop {
        match game.advance() {
            AdvanceOutcome::AwaitingHuman(d) if d.kind == DecisionKind::TriggerTargets => break d,
            AdvanceOutcome::AwaitingHuman(d) => {
                submit_pass(&mut game, &d)
                    .unwrap_or_else(|e| panic!("PassPriority submit failed: {:?}", e));
            }
            other => panic!("expected to reach TriggerTargets, got {:?}", other),
        }
    };

    assert_eq!(decision.player, PlayerId(1), "CR 603.3a: the controller");
    assert_eq!(
        decision.actions.len(),
        2,
        "exactly one action is legal while the CR 603.3b batch is suspended, plus \
         S8's human-only Concede (CR 104.3a), which the engine's admission gate \
         exempts from the BlockingDecision block"
    );
    assert!(matches!(decision.actions[1], LegalAction::Concede));
    match &decision.actions[0] {
        LegalAction::ChooseTriggerTargets { slots, targets, .. } => {
            assert_eq!(slots.len(), 1, "one TargetCreature slot");
            assert_eq!(
                slots[0].candidates.len(),
                2,
                "CR 601.2c: both creatures are legal choices"
            );
            assert_eq!(targets.len(), slots.len());
        }
        other => panic!("expected ChooseTriggerTargets, got {:?}", other),
    };

    // S1 idempotence: a second advance() returns the SAME seq.
    match game.advance() {
        AdvanceOutcome::AwaitingHuman(d2) => assert_eq!(d2.seq, decision.seq),
        other => panic!("expected the same AwaitingHuman again, got {:?}", other),
    }

    // The Session 1 "a command naming another seat -> BadParams" sub-check that used
    // to live here is gone for the same structural reason as
    // `test_local_game_submit_unknown_action_index_is_rejected`: `HumanChoice` has
    // no field that can name a player.

    // A stale seq -> StaleDecision.
    let stale = game.submit(
        decision.seq + 100,
        HumanChoice {
            action_index: 0,
            params: ActionParams::default(),
        },
    );
    assert!(matches!(stale, Err(LocalGameError::StaleDecision { .. })));

    // The correct answer is accepted. `ChooseTriggerTargets` ignores `params` and
    // forwards the engine's own default `targets` verbatim (SR-38).
    let ok = game.submit(
        decision.seq,
        HumanChoice {
            action_index: 0,
            params: ActionParams::default(),
        },
    );
    assert!(ok.is_ok(), "SR-38: {:?}", ok.err());
}

/// T17: a bot-only game never halts on a CR 603.3d announcement.
///
/// This is the guard against `driver.rs`'s `unreachable!()`: a provider gap on a
/// blocking decision turns a recoverable state into a dead game
/// (`Halted(EngineError)` via the `PassPriority` fallback, OOS-DP7-12).
#[test]
fn test_dp8_bot_game_never_halts_on_a_trigger_target() {
    let (state, _zapper) = state_with_pending_targeted_trigger();
    let mut bots: HashMap<PlayerId, Box<dyn Bot>> = HashMap::new();
    bots.insert(
        PlayerId(1),
        Box::new(RandomBot::new(1, "Bot-1".to_string())),
    );
    bots.insert(
        PlayerId(2),
        Box::new(RandomBot::new(2, "Bot-2".to_string())),
    );
    let (mut game, _start_events) = LocalGame::start(
        state,
        7,
        StubProvider,
        bots,
        BTreeSet::new(),
        small_limits(60),
        true,
    )
    .expect("game should start");

    // `advance()` runs bot seats until a terminal outcome, so one call suffices;
    // the assertion is that the outcome is never `AwaitingHuman` and never an
    // engine error.
    match game.advance() {
        AdvanceOutcome::AwaitingHuman(d) => {
            panic!("a bot-only game must never await a human: {:?}", d)
        }
        AdvanceOutcome::Halted(HaltReason::EngineError(e)) => {
            panic!("bot game halted on an engine error: {e}")
        }
        AdvanceOutcome::Halted(_) | AdvanceOutcome::GameOver { .. } => {}
    }

    let answered = game
        .journal()
        .iter()
        .filter(|r| matches!(r.command, Command::ChooseTriggerTargets { .. }))
        .count();
    assert!(
        answered >= 1,
        "the bot must have answered the CR 603.3d announcement at least once"
    );
}

/// T18: SR-38 — while a CR 603.3d announcement is outstanding, `StubProvider`
/// offers the blocked player exactly one action and every other player none, and
/// the offered default is ACCEPTED by `process_command`.
#[test]
fn test_dp8_stub_provider_offers_only_the_answer() {
    use mtg_engine::process_command;
    use mtg_simulator::LegalActionProvider;

    let (state, _zapper) = state_with_pending_targeted_trigger();
    // Flush the trigger through the engine so the announcement is outstanding.
    // `pending_triggers` are flushed on step entry (CR 603.3), so pass priority
    // around until the step advances and the flush suspends.
    let mut state = state;
    for _ in 0..8 {
        if state.blocking_decision().is_some() {
            break;
        }
        let holder = state
            .turn()
            .priority_holder
            .unwrap_or(state.turn().active_player);
        let (s, _events) =
            process_command(state, Command::PassPriority { player: holder }).unwrap();
        state = s;
    }
    assert!(
        state.blocking_decision().is_some(),
        "the fixture must reach a CR 603.3d announcement"
    );

    let p1_actions = StubProvider.legal_actions(&state, PlayerId(1));
    let p2_actions = StubProvider.legal_actions(&state, PlayerId(2));
    assert_eq!(p1_actions.len(), 1);
    assert!(
        p2_actions.is_empty(),
        "no other seat may act while the batch is suspended"
    );

    let (choice_id, targets, slots) = match &p1_actions[0] {
        LegalAction::ChooseTriggerTargets {
            choice_id,
            targets,
            slots,
            ..
        } => (*choice_id, targets.clone(), slots.clone()),
        other => panic!("expected ChooseTriggerTargets, got {:?}", other),
    };
    assert_eq!(targets.len(), slots.len());

    let accepted = process_command(
        state,
        Command::ChooseTriggerTargets {
            player: PlayerId(1),
            choice_id,
            targets,
        },
    );
    assert!(
        accepted.is_ok(),
        "SR-38: the engine must accept the action its own provider offered: {:?}",
        accepted.err()
    );
}

// ── PB-DP9 (DP-7 / DP-8 / DP-9): CR 608.2d resolution-time choices ───────────

/// A two-player state with a "Scry 2" sorcery already **on the stack** for
/// `PlayerId(1)`, plus a library to scry into.
///
/// The spell is put on the stack by casting it through `process_command`, so the
/// stack object is built exactly the way a real cast builds it. `LocalGame` then
/// starts from a state whose next event is the resolution — and therefore the
/// CR 608.2d announcement.
fn state_with_pending_scry() -> GameState {
    use mtg_engine::cards::card_definition::PlayerTarget;
    use mtg_engine::rules::command::CastSpellData;
    use mtg_engine::state::turn::Step;
    use mtg_engine::{
        process_command, AbilityDefinition, CardType, Effect, EffectAmount, ManaColor, ManaCost,
        TypeLine,
    };

    let def = CardDefinition {
        name: "DP9 Scry Spell".to_string(),
        card_id: CardId("dp9-sim-scry".to_string()),
        mana_cost: Some(ManaCost {
            generic: 1,
            ..ManaCost::default()
        }),
        types: TypeLine {
            card_types: [CardType::Sorcery].into_iter().collect(),
            ..Default::default()
        },
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::Scry {
                player: PlayerTarget::Controller,
                count: EffectAmount::Fixed(2),
            },
            targets: vec![],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    };

    let mut builder = GameStateBuilder::new()
        .add_player(PlayerId(1))
        .add_player(PlayerId(2))
        .with_registry(CardRegistry::new(vec![def.clone()]))
        .active_player(PlayerId(1))
        .at_step(Step::PreCombatMain)
        .object(
            ObjectSpec::card(PlayerId(1), &def.name)
                .with_card_id(def.card_id.clone())
                .with_types(vec![CardType::Sorcery])
                .with_mana_cost(ManaCost {
                    generic: 1,
                    ..ManaCost::default()
                })
                .in_zone(ZoneId::Hand(PlayerId(1))),
        );
    for name in ["Bottom", "Middle", "Top"] {
        builder = builder.object(
            ObjectSpec::creature(PlayerId(1), name, 1, 1).in_zone(ZoneId::Library(PlayerId(1))),
        );
    }
    let mut state = builder.build().expect("PB-DP9 fixture should build");
    state
        .players_mut()
        .get_mut(&PlayerId(1))
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 5);
    state.turn_mut().priority_holder = Some(PlayerId(1));

    let spell_id = state
        .objects()
        .iter()
        .find(|(_, o)| o.characteristics.name == "DP9 Scry Spell")
        .map(|(id, _)| *id)
        .expect("the spell must exist");
    let (state, _) = process_command(
        state,
        Command::CastSpell(Box::new(CastSpellData {
            player: PlayerId(1),
            card: spell_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: None,
            prototype: false,
            modes_chosen: vec![],
            x_value: 0,
            face_down_kind: None,
            additional_costs: vec![],
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        })),
    )
    .expect("cast should succeed");
    state
}

/// T-sim-1: CR 608.2d reaches `LocalGame` as its OWN `DecisionKind`.
///
/// The `kind == EffectChoice` assertion is the point: `advance()`'s acting-player
/// chain matches `BlockingDecision` **exhaustively**, so a new variant is
/// compile-forced rather than silently mapped to the wrong picker (the bug
/// PB-DP8 had to fix). Also re-pins S1's idempotence guard, the foreign-seat
/// rejection and the stale-`seq` rejection against the new decision class.
#[test]
fn test_dp9_local_game_awaits_human() {
    let state = state_with_pending_scry();
    let human_seats: BTreeSet<PlayerId> = [PlayerId(1)].into_iter().collect();
    let mut bots: HashMap<PlayerId, Box<dyn Bot>> = HashMap::new();
    bots.insert(
        PlayerId(2),
        Box::new(RandomBot::new(1, "Bot-2".to_string())),
    );
    let (mut game, _start_events) = LocalGame::start(
        state,
        1,
        StubProvider,
        bots,
        human_seats,
        small_limits(20),
        true,
    )
    .expect("game should start");

    let decision = loop {
        match game.advance() {
            AdvanceOutcome::AwaitingHuman(d) if d.kind == DecisionKind::EffectChoice => break d,
            AdvanceOutcome::AwaitingHuman(d) => {
                submit_pass(&mut game, &d)
                    .unwrap_or_else(|e| panic!("PassPriority submit failed: {:?}", e));
            }
            other => panic!("expected to reach EffectChoice, got {:?}", other),
        }
    };

    assert_eq!(
        decision.player,
        PlayerId(1),
        "CR 701.22a: the scrying player answers"
    );
    assert_eq!(
        decision.actions.len(),
        2,
        "exactly one action is legal while the resolution is rolled back, plus S8's \
         human-only Concede (CR 104.3a)"
    );
    assert!(matches!(decision.actions[1], LegalAction::Concede));
    match &decision.actions[0] {
        LegalAction::AnswerEffectChoice { question, .. } => match question {
            mtg_engine::EffectChoiceQuestion::Scry { looked_at } => {
                assert_eq!(looked_at.len(), 2, "CR 701.22a: the top 2 were looked at");
            }
            other => panic!("expected a Scry question, got {:?}", other),
        },
        other => panic!("expected AnswerEffectChoice, got {:?}", other),
    };

    // S1 idempotence: a second advance() returns the SAME seq.
    match game.advance() {
        AdvanceOutcome::AwaitingHuman(d2) => assert_eq!(d2.seq, decision.seq),
        other => panic!("expected the same AwaitingHuman again, got {:?}", other),
    }

    // The Session 1 "a command naming another seat -> BadParams" sub-check that used
    // to live here is gone for the same structural reason as
    // `test_local_game_submit_unknown_action_index_is_rejected`: `HumanChoice` has
    // no field that can name a player.

    // A stale seq -> StaleDecision.
    let stale = game.submit(
        decision.seq + 100,
        HumanChoice {
            action_index: 0,
            params: ActionParams::default(),
        },
    );
    assert!(matches!(stale, Err(LocalGameError::StaleDecision { .. })));

    // The correct answer is accepted. `AnswerEffectChoice` ignores `params` and
    // forwards the engine's own default `answer` verbatim (SR-38).
    let ok = game.submit(
        decision.seq,
        HumanChoice {
            action_index: 0,
            params: ActionParams::default(),
        },
    );
    assert!(ok.is_ok(), "SR-38: {:?}", ok.err());
}

/// T-sim-2: a bot-only game never halts on a CR 608.2d announcement.
///
/// This is the guard against `driver.rs`'s `unreachable!()`: a provider gap on a
/// blocking decision turns a recoverable state into a dead game
/// (`Halted(EngineError)` via the `PassPriority` fallback, OOS-DP7-12).
#[test]
fn test_dp9_bot_game_never_halts_on_an_effect_choice() {
    let state = state_with_pending_scry();
    let mut bots: HashMap<PlayerId, Box<dyn Bot>> = HashMap::new();
    bots.insert(
        PlayerId(1),
        Box::new(RandomBot::new(1, "Bot-1".to_string())),
    );
    bots.insert(
        PlayerId(2),
        Box::new(RandomBot::new(2, "Bot-2".to_string())),
    );
    let (mut game, _start_events) = LocalGame::start(
        state,
        7,
        StubProvider,
        bots,
        BTreeSet::new(),
        small_limits(30),
        true,
    )
    .expect("game should start");

    match game.advance() {
        AdvanceOutcome::AwaitingHuman(d) => {
            panic!("a bot-only game must never await a human: {:?}", d)
        }
        AdvanceOutcome::Halted(HaltReason::EngineError(e)) => {
            panic!("bot game halted on an engine error: {e}")
        }
        AdvanceOutcome::Halted(_) | AdvanceOutcome::GameOver { .. } => {}
    }

    let answered = game
        .journal()
        .iter()
        .filter(|r| matches!(r.command, Command::AnswerEffectChoice { .. }))
        .count();
    assert!(
        answered >= 1,
        "the bot must have answered the CR 608.2d choice at least once"
    );
}

/// T-sim-3: SR-38 — while a CR 608.2d choice is outstanding, `StubProvider`
/// offers the blocked player exactly one action and every other player none, and
/// the offered default is ACCEPTED by `process_command`.
#[test]
fn test_dp9_stub_provider_offers_only_the_answer() {
    use mtg_engine::process_command;
    use mtg_simulator::LegalActionProvider;

    let mut state = state_with_pending_scry();
    for _ in 0..8 {
        if state.blocking_decision().is_some() {
            break;
        }
        let holder = state
            .turn()
            .priority_holder
            .unwrap_or(state.turn().active_player);
        let (s, _events) =
            process_command(state, Command::PassPriority { player: holder }).unwrap();
        state = s;
    }
    assert!(
        state.blocking_decision().is_some(),
        "the fixture must reach a CR 608.2d announcement"
    );

    let p1_actions = StubProvider.legal_actions(&state, PlayerId(1));
    let p2_actions = StubProvider.legal_actions(&state, PlayerId(2));
    assert_eq!(p1_actions.len(), 1);
    assert!(
        p2_actions.is_empty(),
        "no other seat may act while the resolution is rolled back"
    );

    let (choice_id, answer) = match &p1_actions[0] {
        LegalAction::AnswerEffectChoice {
            choice_id, answer, ..
        } => (*choice_id, answer.clone()),
        other => panic!("expected AnswerEffectChoice, got {:?}", other),
    };

    let accepted = process_command(
        state,
        Command::AnswerEffectChoice {
            player: PlayerId(1),
            choice_id,
            answer,
        },
    );
    assert!(
        accepted.is_ok(),
        "SR-38: the engine must accept the action its own provider offered: {:?}",
        accepted.err()
    );
}

/// T-det-1: PB-DP9's abort-and-replay makes execution determinism a **runtime**
/// requirement, not only a test one (SR-9b). Two bot-only games from the same
/// seed must be byte-identical.
///
/// If this ever reddens, the mechanism itself is unsound: a banked answer could
/// be applied to a question the engine never asked.
#[test]
fn test_dp9_same_seed_twice_is_byte_identical() {
    fn run(seed: u64) -> (Vec<String>, [u8; 32]) {
        let mut bots: HashMap<PlayerId, Box<dyn Bot>> = HashMap::new();
        bots.insert(
            PlayerId(1),
            Box::new(RandomBot::new(seed, "Bot-1".to_string())),
        );
        bots.insert(
            PlayerId(2),
            Box::new(RandomBot::new(seed + 1, "Bot-2".to_string())),
        );
        let (mut game, _) = LocalGame::start(
            state_with_pending_scry(),
            seed,
            StubProvider,
            bots,
            BTreeSet::new(),
            small_limits(30),
            true,
        )
        .expect("game should start");
        let _ = game.advance();
        let journal: Vec<String> = game
            .journal()
            .iter()
            .map(|r| format!("{:?}", r.command))
            .collect();
        (journal, game.state().public_state_hash())
    }

    let (j1, h1) = run(4242);
    let (j2, h2) = run(4242);
    assert_eq!(j1, j2, "the same seed must produce the same command trace");
    assert_eq!(h1, h2, "...and the same final public state hash");
    assert!(
        j1.iter().any(|c| c.contains("AnswerEffectChoice")),
        "the run must actually have exercised a CR 608.2d choice; trace: {j1:?}"
    );
}

// ── M11-local Session 3 (item 8): action parameterization + engine target
// queries reach LocalGame ──────────────────────────────────────────────────────

/// A 2-player un-started `GameState`: `PlayerId(1)` holds an Instant in hand
/// ("Session3 Bolt") that deals 3 damage to a declared `TargetCreature`, has
/// an UNTAPPED mana source that can pay its cost (CR 500.4 empties the mana pool
/// between steps, so a PRE-FILLED pool set before `LocalGame::start` -- which
/// runs the game through Untap and Upkeep before the human ever sees a decision
/// -- would be emptied before this fixture's caller gets to use it; a battlefield
/// mana source survives that transition and is tapped, via `ActionParams::auto_tap`,
/// atomically with the cast itself), and `PlayerId(2)` controls a creature on the
/// battlefield as the only legal target. `PlayerId(1)` holds priority.
fn state_with_targeted_spell_and_target() -> (GameState, ObjectId, ObjectId) {
    use mtg_engine::state::turn::Step;
    use mtg_engine::{
        AbilityDefinition, CardEffectTarget, CardType, Effect, EffectAmount, ManaCost, TypeLine,
    };

    let def = CardDefinition {
        name: "Session3 Bolt".to_string(),
        card_id: CardId("session3-bolt".to_string()),
        mana_cost: Some(ManaCost {
            generic: 1,
            ..ManaCost::default()
        }),
        types: TypeLine {
            card_types: [CardType::Instant].into_iter().collect(),
            ..Default::default()
        },
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::DealDamage {
                source: None,
                target: CardEffectTarget::DeclaredTarget { index: 0 },
                amount: EffectAmount::Fixed(3),
            },
            targets: vec![TargetRequirement::TargetCreature],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    };

    let mut state = GameStateBuilder::new()
        .add_player(PlayerId(1))
        .add_player(PlayerId(2))
        .with_registry(CardRegistry::new(vec![def.clone()]))
        .active_player(PlayerId(1))
        .at_step(Step::PreCombatMain)
        .object(
            ObjectSpec::card(PlayerId(1), &def.name)
                .with_card_id(def.card_id.clone())
                .with_types(vec![CardType::Instant])
                .with_mana_cost(ManaCost {
                    generic: 1,
                    ..ManaCost::default()
                })
                .in_zone(ZoneId::Hand(PlayerId(1))),
        )
        .object(
            ObjectSpec::land(PlayerId(1), "Session3 Source").with_mana_ability(ManaAbility {
                produces: [(ManaColor::Colorless, 1u32)].into_iter().collect(),
                requires_tap: true,
                ..Default::default()
            }),
        )
        // 5 toughness so the spell's 3 damage is NOT lethal (CR 704.5g) -- the
        // creature must still exist afterward so the test can read
        // `damage_marked` off the SAME object rather than chasing a CR 400.7
        // zone-change identity change into the graveyard.
        .object(ObjectSpec::creature(PlayerId(2), "Target Creature", 3, 5))
        .build()
        .expect("targeted-spell fixture should build");

    state.turn_mut().priority_holder = Some(PlayerId(1));

    let spell_id = state
        .objects()
        .iter()
        .find(|(_, o)| o.characteristics.name == "Session3 Bolt")
        .map(|(id, _)| *id)
        .expect("the spell must exist");
    let creature_id = state
        .objects()
        .iter()
        .find(|(_, o)| o.characteristics.name == "Target Creature")
        .map(|(id, _)| *id)
        .expect("the target creature must exist");

    (state, spell_id, creature_id)
}

/// M11-local Session 3's acceptance criterion: a human can cast a spell with a
/// `TargetRequirement` at a legal target through `LocalGame::submit`. Uses the
/// newly re-exported engine query fns (`spell_target_requirements` +
/// `legal_targets_per_slot`, `crates/engine/src/rules/queries.rs`) to pick the
/// target, exercising the engine half end-to-end — not just the simulator half.
/// CR 601.2c (targeting) / CR 608.2h (damage resolves).
#[test]
fn test_human_casts_targeted_spell_through_local_game() {
    let (state, spell_id, creature_id) = state_with_targeted_spell_and_target();

    let human_seats: BTreeSet<PlayerId> = [PlayerId(1)].into_iter().collect();
    let mut bots: HashMap<PlayerId, Box<dyn Bot>> = HashMap::new();
    bots.insert(
        PlayerId(2),
        Box::new(RandomBot::new(1, "Bot-2".to_string())),
    );

    let (mut game, _start_events) = LocalGame::start(
        state,
        1,
        StubProvider,
        bots,
        human_seats,
        small_limits(5),
        true,
    )
    .expect("game should start");

    let decision = match game.advance() {
        AdvanceOutcome::AwaitingHuman(d) => d,
        other => panic!("expected AwaitingHuman, got {:?}", other),
    };

    let action_index = find_action_index(
        &decision.actions,
        |a| matches!(a, LegalAction::CastSpell { card, .. } if *card == spell_id),
    );

    // CR 601.2c: use the engine's own query surface to pick a legal target.
    let reqs = spell_target_requirements(game.state(), spell_id, &[], None);
    assert_eq!(reqs, vec![TargetRequirement::TargetCreature]);
    let candidates = legal_targets_per_slot(game.state(), PlayerId(1), spell_id, &reqs);
    assert_eq!(candidates.len(), 1, "one slot in, one slot out");
    assert!(
        candidates[0].contains(&Target::Object(creature_id)),
        "the opponent's creature must be a legal target for TargetCreature"
    );
    let target = Target::Object(creature_id);

    let result = game.submit(
        decision.seq,
        HumanChoice {
            action_index,
            params: ActionParams {
                targets: vec![target],
                // The fixture's mana source has an empty pool going in (CR 500.4
                // empties it between steps) -- tap it as part of this submit.
                auto_tap: true,
                ..Default::default()
            },
        },
    );
    assert!(
        result.is_ok(),
        "the targeted cast must be accepted: {:?}",
        result.err()
    );

    // The spell is on the stack after `submit`, not yet resolved (CR 608.1 --
    // priority must pass around before it does). CR 117.3c gives priority back to
    // the human caster first, so `advance()` returns another `AwaitingHuman`
    // rather than running the bot seat by itself; pass, then `advance()` again to
    // let the bot pass too and resolve the stack.
    let post_cast = match game.advance() {
        AdvanceOutcome::AwaitingHuman(d) => d,
        other => panic!(
            "expected AwaitingHuman (priority back to the caster), got {:?}",
            other
        ),
    };
    submit_pass(&mut game, &post_cast).expect("passing priority after casting must succeed");
    let _ = game.advance();

    let damage = game
        .state()
        .object(creature_id)
        .expect("the creature must still exist")
        .damage_marked;
    assert_eq!(damage, 3, "the targeted spell's damage must have resolved");
}

/// A human submitting a target the engine does not consider legal (here: an
/// `ObjectId` that does not exist) is rejected — `LocalGameError::Rejected` — and
/// the game state is left byte-for-byte untouched: same `public_state_hash`, same
/// `command_count`, same journal length. CR 608.2b's "the target no longer exists"
/// case fizzles at RESOLUTION, not at cast time; this test is about CAST-time
/// target legality (CR 601.2c/601.3), which `handle_cast_spell` rejects up front.
#[test]
fn test_human_illegal_target_is_rejected_without_state_change() {
    let (state, spell_id, _creature_id) = state_with_targeted_spell_and_target();

    let human_seats: BTreeSet<PlayerId> = [PlayerId(1)].into_iter().collect();
    let mut bots: HashMap<PlayerId, Box<dyn Bot>> = HashMap::new();
    bots.insert(
        PlayerId(2),
        Box::new(RandomBot::new(1, "Bot-2".to_string())),
    );

    let (mut game, _start_events) = LocalGame::start(
        state,
        1,
        StubProvider,
        bots,
        human_seats,
        small_limits(5),
        true,
    )
    .expect("game should start");

    let decision = match game.advance() {
        AdvanceOutcome::AwaitingHuman(d) => d,
        other => panic!("expected AwaitingHuman, got {:?}", other),
    };
    let action_index = find_action_index(
        &decision.actions,
        |a| matches!(a, LegalAction::CastSpell { card, .. } if *card == spell_id),
    );

    let commands_before = game.command_count();
    let journal_len_before = game.journal().len();
    let hash_before = game.state().public_state_hash();

    // A target that does not exist -- never legal. `auto_tap: true` is
    // deliberate here, not incidental: it means the rejected `CastSpell` is
    // preceded by a `TapForMana` that WOULD have succeeded on its own, so this
    // test also exercises `submit`'s atomicity guarantee (item 7) -- a
    // successful tap followed by a rejected cast must roll back as one unit,
    // leaving the mana source untapped and the pool empty.
    let bogus_target = Target::Object(ObjectId(999_999));
    let result = game.submit(
        decision.seq,
        HumanChoice {
            action_index,
            params: ActionParams {
                targets: vec![bogus_target],
                auto_tap: true,
                ..Default::default()
            },
        },
    );

    assert!(
        matches!(result, Err(LocalGameError::Rejected(_))),
        "expected Rejected, got {:?}",
        result
    );
    assert_eq!(
        game.command_count(),
        commands_before,
        "command_count must be unchanged on rejection"
    );
    assert_eq!(
        game.journal().len(),
        journal_len_before,
        "journal must be unchanged on rejection"
    );
    assert_eq!(
        game.state().public_state_hash(),
        hash_before,
        "the state must be byte-for-byte unchanged on rejection"
    );
    let still_pending = game
        .pending_decision()
        .expect("a rejected submit must not consume the pending decision");
    assert_eq!(still_pending.seq, decision.seq);
}

/// PB-RS2 precedent (OOS-RS-2): `action_to_command_with_params` must forward a
/// `LegalAction::TapForMana`'s `hybrid_choices`/`phyrexian_life_payments`
/// VERBATIM into the `Command`, element for element -- never re-derive them.
/// Constructs the `LegalAction` directly (a fixed-colour ability, so no
/// `chosen_color` validation applies) rather than going through the provider,
/// since the provider only offers a hybrid/Phyrexian plan when one happens to be
/// payable in a real game state.
#[test]
fn test_hybrid_payment_plan_is_forwarded_verbatim() {
    use mtg_engine::state::turn::Step;
    use mtg_engine::HybridManaPayment;

    let state = GameStateBuilder::new()
        .add_player(PlayerId(1))
        .add_player(PlayerId(2))
        .active_player(PlayerId(1))
        .at_step(Step::PreCombatMain)
        .object(
            ObjectSpec::land(PlayerId(1), "Filter Source").with_mana_ability(ManaAbility {
                requires_tap: true,
                ..Default::default()
            }),
        )
        .build()
        .expect("fixture should build");

    let source_id = state
        .objects()
        .iter()
        .find(|(_, o)| o.characteristics.name == "Filter Source")
        .map(|(id, _)| *id)
        .expect("the source must exist");

    let hybrid_choices = vec![
        HybridManaPayment::Color(ManaColor::Red),
        HybridManaPayment::Generic,
    ];
    let phyrexian_life_payments = vec![true, false];

    let action = LegalAction::TapForMana {
        source: source_id,
        ability_index: 0,
        chosen_color: None,
        hybrid_choices: hybrid_choices.clone(),
        phyrexian_life_payments: phyrexian_life_payments.clone(),
    };

    let command =
        action_to_command_with_params(&state, PlayerId(1), &action, &ActionParams::default())
            .expect("a fixed-colour ability needs no chosen_color");

    match command {
        Command::TapForMana {
            hybrid_choices: got_hybrid,
            phyrexian_life_payments: got_phyrexian,
            ..
        } => {
            assert_eq!(
                got_hybrid, hybrid_choices,
                "hybrid_choices must be forwarded verbatim"
            );
            assert_eq!(
                got_phyrexian, phyrexian_life_payments,
                "phyrexian_life_payments must be forwarded verbatim"
            );
        }
        other => panic!("expected TapForMana, got {:?}", other),
    }
}

/// CR 605.3b / CR 106.1b: `action_to_command_with_params` rejects a `TapForMana`
/// on an `any_color` mana ability with no `chosen_color` (`ParamError::MissingChosenColor`)
/// rather than silently defaulting to `Colorless`, and rejects `chosen_color:
/// Some(Colorless)` outright (`ParamError::InvalidChosenColor`) -- Colorless is never
/// a legal "any color" choice. A concrete colour is accepted.
#[test]
fn test_param_error_any_color_without_chosen_color_is_rejected() {
    use mtg_engine::state::turn::Step;

    let state = GameStateBuilder::new()
        .add_player(PlayerId(1))
        .add_player(PlayerId(2))
        .active_player(PlayerId(1))
        .at_step(Step::PreCombatMain)
        .object(
            ObjectSpec::land(PlayerId(1), "Any Color Source").with_mana_ability(ManaAbility {
                requires_tap: true,
                any_color: true,
                ..Default::default()
            }),
        )
        .build()
        .expect("fixture should build");

    let source_id = state
        .objects()
        .iter()
        .find(|(_, o)| o.characteristics.name == "Any Color Source")
        .map(|(id, _)| *id)
        .expect("the source must exist");

    let missing = LegalAction::TapForMana {
        source: source_id,
        ability_index: 0,
        chosen_color: None,
        hybrid_choices: vec![],
        phyrexian_life_payments: vec![],
    };
    let err =
        action_to_command_with_params(&state, PlayerId(1), &missing, &ActionParams::default())
            .expect_err("an any_color ability with no chosen_color must be rejected");
    assert_eq!(err, ParamError::MissingChosenColor);

    let colorless = LegalAction::TapForMana {
        source: source_id,
        ability_index: 0,
        chosen_color: Some(ManaColor::Colorless),
        hybrid_choices: vec![],
        phyrexian_life_payments: vec![],
    };
    let err2 =
        action_to_command_with_params(&state, PlayerId(1), &colorless, &ActionParams::default())
            .expect_err("Colorless is never legal for an any_color ability (CR 106.1b)");
    assert_eq!(err2, ParamError::InvalidChosenColor);

    let ok = LegalAction::TapForMana {
        source: source_id,
        ability_index: 0,
        chosen_color: Some(ManaColor::Blue),
        hybrid_choices: vec![],
        phyrexian_life_payments: vec![],
    };
    let command = action_to_command_with_params(&state, PlayerId(1), &ok, &ActionParams::default())
        .expect("a concrete colour must be accepted");
    assert!(matches!(
        command,
        Command::TapForMana {
            chosen_color: Some(ManaColor::Blue),
            ..
        }
    ));
}

/// A 2-player un-started `GameState`: `PlayerId(1)` holds an Instant in hand
/// ("Session3 Cantrip", no targets) costing `{1}`, and controls **two** untapped
/// mana sources producing `{C}`. The pool starts EMPTY -- CR 500.4 empties it
/// between steps, so pre-filling it here (before `LocalGame::start` runs the game
/// through Untap and Upkeep) would just be discarded before the caller ever gets a
/// decision; the "pool already covers the cost" scenario is instead produced by
/// manually tapping source A with a separate `submit` call first, within the same
/// step as the cast.
///
/// **The SECOND source is what makes `test_auto_tap_skipped_when_pool_already_covers_cost`
/// half 1 non-vacuous, and it is load-bearing.** With only one source, the manual
/// pre-tap exhausts the board: `mana_solver::solve_mana_payment` skips
/// `status.tapped` sources (`mana_solver.rs:31-33`), so it returns `None` and no
/// `TapForMana` is issued *whether or not the pool check exists* — the assertion
/// would be satisfied by source exhaustion rather than by the feature, and the test
/// stayed green with the guard neutered to `if false && …`. With source B left
/// untapped, deleting the pool check makes the solver find it and emit a spurious
/// tap, so half 1 reddens on regression. Do not remove source B.
fn state_for_auto_tap_test() -> (GameState, ObjectId, ObjectId, ObjectId) {
    use mtg_engine::state::turn::Step;
    use mtg_engine::{
        AbilityDefinition, CardType, Effect, EffectAmount, ManaCost, PlayerTarget, TypeLine,
    };

    let def = CardDefinition {
        name: "Session3 Cantrip".to_string(),
        card_id: CardId("session3-cantrip".to_string()),
        mana_cost: Some(ManaCost {
            generic: 1,
            ..ManaCost::default()
        }),
        types: TypeLine {
            card_types: [CardType::Instant].into_iter().collect(),
            ..Default::default()
        },
        abilities: vec![AbilityDefinition::Spell {
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

    let mut state = GameStateBuilder::new()
        .add_player(PlayerId(1))
        .add_player(PlayerId(2))
        .with_registry(CardRegistry::new(vec![def.clone()]))
        .active_player(PlayerId(1))
        .at_step(Step::PreCombatMain)
        .object(
            ObjectSpec::card(PlayerId(1), &def.name)
                .with_card_id(def.card_id.clone())
                .with_types(vec![CardType::Instant])
                .with_mana_cost(ManaCost {
                    generic: 1,
                    ..ManaCost::default()
                })
                .in_zone(ZoneId::Hand(PlayerId(1))),
        )
        .object(
            ObjectSpec::land(PlayerId(1), "Session3 Source A").with_mana_ability(ManaAbility {
                produces: [(ManaColor::Colorless, 1u32)].into_iter().collect(),
                requires_tap: true,
                ..Default::default()
            }),
        )
        .object(
            ObjectSpec::land(PlayerId(1), "Session3 Source B").with_mana_ability(ManaAbility {
                produces: [(ManaColor::Colorless, 1u32)].into_iter().collect(),
                requires_tap: true,
                ..Default::default()
            }),
        )
        .build()
        .expect("auto-tap fixture should build");

    state.turn_mut().priority_holder = Some(PlayerId(1));

    let spell_id = state
        .objects()
        .iter()
        .find(|(_, o)| o.characteristics.name == "Session3 Cantrip")
        .map(|(id, _)| *id)
        .expect("the spell must exist");
    let find_source = |name: &str| {
        state
            .objects()
            .iter()
            .find(|(_, o)| o.characteristics.name == name)
            .map(|(id, _)| *id)
            .unwrap_or_else(|| panic!("{name} must exist"))
    };
    let source_a = find_source("Session3 Source A");
    let source_b = find_source("Session3 Source B");

    (state, spell_id, source_a, source_b)
}

/// Item 7's conditional auto-tap (the pool half of OOS-M11-2). BOTH halves in one
/// test, or it proves nothing.
///
/// **Still true after SIM-2, by a different mechanism** (`scutemob-176`): the explicit
/// `can_pay_cost` early return this test was written against is gone, and half 1 now
/// passes because `solve_mana_payment_with_pool` subtracts the pool and finds a residual
/// of zero, which is an EMPTY plan. Same observable outcome, reached by the general rule
/// instead of a special case — and the non-vacuity argument below is unchanged, because a
/// solver that ignored the pool would still find source B and tap it.
///
/// Half 1 (`pre_tap: true`): source A is tapped MANUALLY first (a separate `submit`
/// of the `TapForMana` action, filling the pool within the same step), then the cast
/// is submitted with `auto_tap: true` -- no ADDITIONAL `TapForMana` is issued by the
/// auto-tap machinery, because the pool already covers the cost. **Source B is left
/// untapped precisely so this half is not vacuous**: without the pool check,
/// `solve_mana_payment` would find B and emit a spurious tap. See
/// `state_for_auto_tap_test`'s doc comment — a one-source fixture passed this
/// assertion by source exhaustion even with the guard neutered.
///
/// Half 2 (`pre_tap: false`): the cast is submitted directly with `auto_tap: true`
/// against an empty pool -- a `TapForMana` IS issued and a source ends up tapped.
#[test]
fn test_auto_tap_skipped_when_pool_already_covers_cost() {
    /// `(a tap was issued by the cast's own submit, source A tapped, source B tapped)`.
    fn cast_with_auto_tap(pre_tap: bool) -> (bool, bool, bool) {
        let (state, spell_id, source_id, source_b) = state_for_auto_tap_test();
        let human_seats: BTreeSet<PlayerId> = [PlayerId(1)].into_iter().collect();
        let mut bots: HashMap<PlayerId, Box<dyn Bot>> = HashMap::new();
        bots.insert(
            PlayerId(2),
            Box::new(RandomBot::new(1, "Bot-2".to_string())),
        );
        let (mut game, _start_events) = LocalGame::start(
            state,
            1,
            StubProvider,
            bots,
            human_seats,
            small_limits(5),
            true,
        )
        .expect("game should start");

        let mut decision = match game.advance() {
            AdvanceOutcome::AwaitingHuman(d) => d,
            other => panic!("expected AwaitingHuman, got {:?}", other),
        };

        if pre_tap {
            let tap_index = find_action_index(
                &decision.actions,
                |a| matches!(a, LegalAction::TapForMana { source, .. } if *source == source_id),
            );
            game.submit(
                decision.seq,
                HumanChoice {
                    action_index: tap_index,
                    params: ActionParams::default(),
                },
            )
            .expect("the manual tap must succeed");
            decision = match game.advance() {
                AdvanceOutcome::AwaitingHuman(d) => d,
                other => panic!(
                    "expected AwaitingHuman after the manual tap, got {:?}",
                    other
                ),
            };
        }

        let journal_len_before_cast = game.journal().len();

        let action_index = find_action_index(
            &decision.actions,
            |a| matches!(a, LegalAction::CastSpell { card, .. } if *card == spell_id),
        );
        let result = game.submit(
            decision.seq,
            HumanChoice {
                action_index,
                params: ActionParams {
                    auto_tap: true,
                    ..Default::default()
                },
            },
        );
        assert!(
            result.is_ok(),
            "the cast must be accepted: {:?}",
            result.err()
        );

        let tap_issued_by_cast_submit = game.journal()[journal_len_before_cast..]
            .iter()
            .any(|r| matches!(r.command, Command::TapForMana { .. }));
        let tapped = |id| {
            game.state()
                .object(id)
                .expect("the mana source must still exist")
                .status
                .tapped
        };
        (
            tap_issued_by_cast_submit,
            tapped(source_id),
            tapped(source_b),
        )
    }

    let (tap_issued, a_tapped, b_tapped) = cast_with_auto_tap(true);
    assert!(
        !tap_issued,
        "no ADDITIONAL TapForMana should be issued by the cast's own submit when the \
         pool already covers the cost"
    );
    assert!(
        a_tapped,
        "source A is tapped -- by the manual pre-tap, not by auto-tap"
    );
    assert!(
        !b_tapped,
        "source B must be UNTOUCHED: this is the assertion that fails if the pool \
         check is removed, because `solve_mana_payment` would then find B and tap it"
    );

    let (tap_issued, a_tapped, b_tapped) = cast_with_auto_tap(false);
    assert!(
        tap_issued,
        "a TapForMana must be issued when the pool cannot cover the cost"
    );
    assert!(
        a_tapped || b_tapped,
        "auto-tap must have tapped a source to pay the {{1}} cost"
    );
}

// ── UI-2 (criterion 6000): a BOT pays a mandatory additional cost ─────────────

/// **CR 118.8 — the bot command path builds a cast the engine accepts.**
///
/// This is criterion 6000's "bots handle both cost kinds without new engine
/// rejections", asserted at the level where it is decidable: the provider's own
/// offer, through `RandomBot`'s own `choose_action`, into `process_command`. Before
/// UI-2 that chain produced a `CastSpell` with an EMPTY `additional_costs`,
/// `casting.rs:3311` refused it, and `advance()` fell back to `PassPriority` — the
/// spell sat in hand being re-offered forever.
///
/// # Why this is not driven through a whole `advance()` game, and what was found
/// trying
///
/// Two pre-existing defects make a full bot game a bad witness for this property,
/// both OBSERVED here rather than reasoned about, and both filed rather than worked
/// around:
///
/// * **`OOS-UI2-2`** — `HeuristicBot` scores `TapForMana` at 5 against
///   `PassPriority`'s 1, and during the UPKEEP those are the only two actions that
///   exist. So it taps its lands in a step where it cannot spend the mana, the pool
///   empties at end of step (CR 500.4), and by its own main phase `can_afford` sees
///   no untapped source and the cast is never OFFERED. The journal of such a game is
///   two `TapForMana` and then nothing but passes, with **zero** engine rejections.
/// * **`OOS-UI2-1`** — `bin/fuzzer.rs` builds its libraries through
///   `GameStateBuilder` **without shuffling them**, while `random_deck` appends its
///   ~34 basics LAST and `Zone::Ordered`'s top is the last index. Every fuzzer game
///   therefore deals basics off the top for its whole length. Instrumenting the
///   provider over 5 games × 80 turns produced **25,964 hand-card observations and
///   not one non-land**. The 360-game A/B this batch ran came back byte-identical
///   because the fuzzer never casts a spell at all — not because the change is
///   neutral.
///
/// The bot is handed a one-element action list so the choice is forced and the test
/// is deterministic. That is not a weaker test than "let it choose": what is under
/// test is the MAPPING from a chosen action to an accepted command, which is exactly
/// what `RandomBot::choose_action` does and exactly where UI-2 changed anything.
#[test]
fn test_ui2_a_bot_pays_a_mandatory_sacrifice_cost_without_an_engine_rejection() {
    use mtg_engine::{process_command, AdditionalCost};
    use mtg_simulator::LegalActionProvider;

    let cards = all_cards();
    let registry = build_registry();
    let defs: HashMap<String, CardDefinition> =
        cards.iter().map(|c| (c.name.clone(), c.clone())).collect();

    let p1 = PlayerId(1);
    let p2 = PlayerId(2);
    let mut state = GameStateBuilder::new()
        .with_registry(Arc::clone(&registry))
        .add_player(p1)
        .add_player(p2)
        .active_player(p1)
        .object(enrich_spec_from_def(
            ObjectSpec::card(p1, "Life's Legacy")
                .with_card_id(CardId("lifes-legacy".to_string()))
                .in_zone(ZoneId::Hand(p1)),
            &defs,
        ))
        // The only eligible sacrifice, so the plan's `eligible[0]` default is
        // unambiguous and the assertion below cannot be satisfied by accident.
        .object(ObjectSpec::creature(p1, "Sacrificial Bear", 2, 2).in_zone(ZoneId::Battlefield));
    // Two untapped Forests cover Life's Legacy's {1}{G}.
    for _ in 0..2 {
        state = state.object(enrich_spec_from_def(
            ObjectSpec::card(p1, "Forest")
                .with_card_id(CardId("forest".to_string()))
                .in_zone(ZoneId::Battlefield),
            &defs,
        ));
    }
    let mut state = state.build().expect("UI-2 bot fixture should build");
    state.turn_mut().priority_holder = Some(p1);

    let bear_id = state
        .objects()
        .iter()
        .find(|(_, o)| o.characteristics.name == "Sacrificial Bear")
        .map(|(id, _)| *id)
        .expect("the bear must exist");

    // The provider's own offer, with the descriptor UI-2 added.
    let offer = StubProvider
        .legal_actions(&state, p1)
        .into_iter()
        .find(|a| matches!(a, LegalAction::CastSpell { .. }))
        .expect("Life's Legacy must be offered: it is affordable and a creature is eligible");
    let LegalAction::CastSpell {
        additional_costs, ..
    } = &offer
    else {
        unreachable!("matched by discriminant above");
    };
    assert_eq!(
        additional_costs
            .sacrifice
            .as_ref()
            .map(|s| (s.eligible.clone(), s.default)),
        Some((vec![bear_id], bear_id)),
        "the offer must name the bear as the one eligible sacrifice"
    );

    // The bot's OWN mapping — `RandomBot::choose_action`, not a hand-built command.
    let mut bot = RandomBot::new(7, "Bot-1".to_string());
    let cmd = bot.choose_action(&state, p1, std::slice::from_ref(&offer));
    let Command::CastSpell(cast) = &cmd else {
        panic!("a one-element list of CastSpell must yield a CastSpell, got {cmd:?}");
    };
    assert_eq!(
        cast.additional_costs,
        vec![AdditionalCost::Sacrifice {
            ids: vec![bear_id],
            lki: vec![],
        }],
        "an all-default `ActionParams` must still carry the plan's required sacrifice \
         (CR 118.8) -- this is the whole reason `merge_required_additional_costs` exists"
    );

    // Pay for it the way `advance()` does, then cast. The taps are the engine's own
    // commands, so nothing here fakes a mana pool.
    let mut working = state;
    for tap in mtg_simulator::solve_mana_payment(
        &working,
        p1,
        &mtg_simulator::effective_cast_cost_with_additional(
            &working,
            p1,
            cast.card,
            &cast.additional_costs,
        )
        .expect("Life's Legacy has a mana cost"),
    )
    .expect("two Forests must be a solvable payment plan")
    {
        working = process_command(working, tap)
            .expect("the provider only offers payable taps")
            .0;
    }

    let (after, _events) =
        process_command(working, cmd).expect("the engine must accept the bot's own cast");
    // CR 400.7: the sacrificed permanent became a NEW object in the graveyard, so
    // the old `bear_id` is gone from the object table entirely. Checked by NAME in
    // the graveyard rather than by following the dead id -- the same reason UI-1's
    // scry probe asserts over the library instead of chasing ids into the hand.
    assert!(
        !after.objects().contains_key(&bear_id),
        "CR 400.7: the pre-sacrifice ObjectId must be dead, not merely moved"
    );
    assert!(
        after
            .objects_in_zone(&ZoneId::Graveyard(p1))
            .iter()
            .any(|o| o.characteristics.name == "Sacrificial Bear"),
        "the bear must have been sacrificed to pay CR 118.8's additional cost"
    );
    assert!(
        !after
            .objects_in_zone(&ZoneId::Hand(p1))
            .iter()
            .any(|o| o.characteristics.name == "Life's Legacy"),
        "the spell must have left hand for the stack"
    );
}
