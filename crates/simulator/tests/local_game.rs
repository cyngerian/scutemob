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
    all_cards, enrich_spec_from_def, AttackTarget, CardDefinition, CardId, CardRegistry, Command,
    GameState, GameStateBuilder, ObjectId, ObjectSpec, PlayerId, ZoneId,
};
use mtg_simulator::{
    build_registry, AdvanceOutcome, Bot, DeckConfig, GameDriver, HaltReason, HumanChoice,
    LegalAction, LocalGame, LocalGameError, LocalGameLimits, RandomBot, StubProvider,
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
    let pass = Command::PassPriority {
        player: PlayerId(1),
    };
    game.submit(first.seq, HumanChoice::Command(pass))
        .expect("the originally-issued seq must still be valid");
}

/// The seat that was asked is the only seat that may answer. A client holding a valid
/// `seq` for its own decision must not be able to submit a command naming a different
/// player — Architecture Invariant 7 once this sits behind HTTP.
#[test]
fn test_local_game_submit_rejects_command_for_another_seat() {
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

    // A legal-looking command, but for the *other* seat.
    let cross_seat = Command::PassPriority {
        player: PlayerId(2),
    };
    let result = game.submit(decision.seq, HumanChoice::Command(cross_seat));

    assert!(
        matches!(result, Err(LocalGameError::BadParams(_))),
        "expected BadParams for a cross-seat command, got {:?}",
        result
    );
    assert_eq!(
        game.command_count(),
        commands_before,
        "a cross-seat submit must not apply anything"
    );
    assert_eq!(
        game.pending_decision().map(|d| d.seq),
        Some(decision.seq),
        "a rejected cross-seat submit must not consume the decision"
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

/// `submit` never falls back to `PassPriority`: an illegal command is reported as
/// `LocalGameError::Rejected` and the game state (and pending decision) survive
/// untouched.
#[test]
fn test_local_game_submit_illegal_command_returns_err_and_preserves_state() {
    let registry = build_registry();
    let cards = all_cards();
    let state = build_state(2, &registry, &cards);

    let human_seats: BTreeSet<PlayerId> = [PlayerId(1)].into_iter().collect();
    let (mut game, _start_events) = LocalGame::start(
        state,
        7,
        StubProvider,
        bots_for(2, 7),
        human_seats,
        small_limits(10),
        true,
    )
    .expect("game should start");

    let decision = match game.advance() {
        AdvanceOutcome::AwaitingHuman(d) => d,
        other => panic!("expected AwaitingHuman, got {:?}", other),
    };

    let commands_before = game.command_count();
    let turn_before = game.state().turn().turn_number;
    let journal_len_before = game.journal().len();

    // References an object that does not exist — always illegal.
    let bogus = Command::PlayLand {
        player: PlayerId(1),
        card: ObjectId(999_999),
    };
    let result = game.submit(decision.seq, HumanChoice::Command(bogus));

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
        game.state().turn().turn_number,
        turn_before,
        "turn must be unchanged on rejection"
    );
    assert_eq!(
        game.journal().len(),
        journal_len_before,
        "journal must be unchanged on rejection"
    );
    let still_pending = game
        .pending_decision()
        .expect("a rejected submit must not consume the pending decision");
    assert_eq!(still_pending.seq, decision.seq);
}

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
    let cmd = Command::PassPriority {
        player: PlayerId(1),
    };
    let result = game.submit(stale_seq, HumanChoice::Command(cmd));

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
                let cmd = Command::PassPriority {
                    player: decision.player,
                };
                game.submit(decision.seq, HumanChoice::Command(cmd))
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
