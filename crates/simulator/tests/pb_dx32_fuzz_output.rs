//! PB-DX32 — make the fuzzer's *output* mean something (`OOS-SIM3-2` / `OOS-SIM3-3` /
//! `OOS-SIM3-4` / `OOS-CARDS2-3`).
//!
//! Four measurements become first-class on `GameResult`, across four stages:
//! rejections (SR-38, Stage 2), waste (the SIM-5 test-only tap/pool instrument
//! promoted, Stage 3), a noise floor split (Stage 4), and the fuzz deck pool size
//! (Stage 5). Stage 1 is the plumbing all four ride on — see [`LocalGame::result_snapshot`].
//!
//! **This batch adds no primitive, no `Command`, no `GameEvent`, no card def.** Every
//! probe here gates `crates/simulator` instrumentation and its printed output only.
//!
//! SR-9a does not apply: that gate (`crates/engine/tests/no_stray_test_binaries.rs`) is
//! scoped to `CARGO_MANIFEST_DIR = crates/engine`. `crates/simulator/tests/` is a flat
//! directory of integration targets and adding one is the convention here (see
//! `pb_dx22_fuzz_instrument.rs`).

use std::collections::{BTreeSet, HashMap};

use mtg_engine::{
    all_cards, AttackTarget, Command, GameState, GameStateBuilder, ObjectId, ObjectSpec, PlayerId,
    ZoneId,
};
use mtg_simulator::{
    build_fuzz_state, build_registry, AdvanceOutcome, Bot, GameDriver, GameDriverError,
    LegalAction, LocalGame, LocalGameLimits, RandomBot, StubProvider,
};

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

/// A bot that always attempts to `PlayLand` a nonexistent object — a command the
/// engine rejects unconditionally (the object cannot resolve), so every priority
/// window this bot acts on increments `LocalGame::rejection_count()` by exactly one.
/// Used to drive Stage 2's deterministic non-zero-rejection fixtures (T2.3).
struct AlwaysRejectedBot;

impl Bot for AlwaysRejectedBot {
    fn choose_action(
        &mut self,
        _state: &GameState,
        player: PlayerId,
        _legal: &[LegalAction],
    ) -> Command {
        Command::PlayLand {
            player,
            card: ObjectId(999_999_999),
        }
    }
    fn choose_targets(&mut self, _: &GameState, _: &[ObjectId], _: usize) -> Vec<ObjectId> {
        Vec::new()
    }
    fn choose_attackers(
        &mut self,
        _: &GameState,
        _: &[ObjectId],
        _: &[AttackTarget],
    ) -> Vec<(ObjectId, AttackTarget)> {
        Vec::new()
    }
    fn choose_blockers(
        &mut self,
        _: &GameState,
        _: &[ObjectId],
        _: &[ObjectId],
    ) -> Vec<(ObjectId, ObjectId)> {
        Vec::new()
    }
    fn choose_mulligan_bottom(&mut self, _: &[ObjectId], _: usize) -> Vec<ObjectId> {
        Vec::new()
    }
    fn name(&self) -> &str {
        "always-rejected"
    }
}

/// A bot that concedes (CR 104.3a) the FIRST time it is asked to act, and passes
/// thereafter (unreachable in practice — a concede ends the game the next loop
/// iteration checks `is_game_over`). Used to drive T2.3's deterministic
/// GameOver-with-rejections fixture.
struct ConcedeOnFirstCallBot {
    conceded: bool,
}

impl ConcedeOnFirstCallBot {
    fn new() -> Self {
        Self { conceded: false }
    }
}

impl Bot for ConcedeOnFirstCallBot {
    fn choose_action(
        &mut self,
        _state: &GameState,
        player: PlayerId,
        _legal: &[LegalAction],
    ) -> Command {
        if self.conceded {
            Command::PassPriority { player }
        } else {
            self.conceded = true;
            Command::Concede { player }
        }
    }
    fn choose_targets(&mut self, _: &GameState, _: &[ObjectId], _: usize) -> Vec<ObjectId> {
        Vec::new()
    }
    fn choose_attackers(
        &mut self,
        _: &GameState,
        _: &[ObjectId],
        _: &[AttackTarget],
    ) -> Vec<(ObjectId, AttackTarget)> {
        Vec::new()
    }
    fn choose_blockers(
        &mut self,
        _: &GameState,
        _: &[ObjectId],
        _: &[ObjectId],
    ) -> Vec<(ObjectId, ObjectId)> {
        Vec::new()
    }
    fn choose_mulligan_bottom(&mut self, _: &[ObjectId], _: usize) -> Vec<ObjectId> {
        Vec::new()
    }
    fn name(&self) -> &str {
        "concede-on-first-call"
    }
}

/// Four `RandomBot` seats, seeded exactly as `bin/fuzzer.rs::run_single_game` and
/// `sim5_bot_cast_discipline.rs::bots_for` seed theirs.
fn random_bots(seed: u64, player_count: u32) -> HashMap<PlayerId, Box<dyn Bot>> {
    let mut bots: HashMap<PlayerId, Box<dyn Bot>> = HashMap::new();
    for i in 1..=u64::from(player_count) {
        let bot_seed = seed.wrapping_add(100 + i);
        bots.insert(
            PlayerId(i),
            Box::new(RandomBot::new(bot_seed, format!("Bot-{i}"))),
        );
    }
    bots
}

/// Play a fuzz-shaped game (`build_fuzz_state` + `RandomBot` + `StubProvider`,
/// `record_journal: false`) to conclusion and return the finished `LocalGame` so the
/// caller can read its accessors. Mirrors `bin/fuzzer.rs::run_single_game` and the
/// Stage-0 measurement probe.
fn play_fuzz_shaped(seed: u64, player_count: u32, max_turns: u32) -> LocalGame<StubProvider> {
    let cards = all_cards();
    let registry = build_registry();
    let setup = build_fuzz_state(seed, player_count, &cards, &registry)
        .unwrap_or_else(|e| panic!("fuzz state for seed {seed} must build: {e:?}"));
    let limits = LocalGameLimits {
        max_turns,
        max_commands: max_turns * 200,
        max_consecutive_passes: 500,
        record_journal: false,
    };
    let (mut game, _events) = LocalGame::start(
        setup.state,
        seed,
        StubProvider,
        random_bots(seed, player_count),
        BTreeSet::new(),
        limits,
        true,
    )
    .unwrap_or_else(|e| panic!("fuzz state for seed {seed} must start: {e:?}"));
    // A single `advance()` call runs the whole game to conclusion: with `human_seats`
    // empty it never yields `AwaitingHuman`, so `advance()`'s own internal loop only
    // stops at `GameOver` or `Halted` (mirrors `driver.rs`'s own comment).
    match game.advance() {
        AdvanceOutcome::AwaitingHuman(_) => unreachable!("no human seats in this fixture"),
        AdvanceOutcome::GameOver(_) | AdvanceOutcome::Halted(_) => {}
    }
    game
}

/// A bare, object-free two-player state. `GameStateBuilder::build()` has no minimum
/// object requirement (only "at least one player"); turn_number defaults to 1 and
/// `start_game`/`reset_turn_state` do not touch it, so every test in this file that
/// wants a deterministic `turn_count` uses this fixture rather than a played-out game.
fn bare_two_player_state() -> mtg_engine::GameState {
    GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .build()
        .expect("bare two-player state must build")
}

/// A two-player state with `library_size` unregistered "filler" cards per player's
/// library (no `card_id`, so Architecture Invariant 9's completeness gate never sees
/// them — the same pattern `invariants.rs`'s own test module uses for Stack-zone
/// fixtures). Enough for the game to survive a few CR 504.1 draw steps without anyone
/// hitting CR 104.3c (draw from an empty library, a loss) — which a truly empty-library
/// two-player game reaches almost immediately once turns start advancing, and would
/// turn every "drive this to a Halted outcome" fixture into a GameOver instead.
fn two_player_state_with_libraries(library_size: usize) -> mtg_engine::GameState {
    let mut builder = GameStateBuilder::new().add_player(p(1)).add_player(p(2));
    for player in [p(1), p(2)] {
        for _ in 0..library_size {
            builder =
                builder.object(ObjectSpec::card(player, "Filler").in_zone(ZoneId::Library(player)));
        }
    }
    builder
        .build()
        .expect("two-player state with libraries must build")
}

fn no_bots() -> HashMap<PlayerId, Box<dyn Bot>> {
    HashMap::new()
}

/// **T1.1** (Stage 1) — CR 104.2a (game over) / the `LocalGameLimits::max_turns` safety
/// valve. Both real `GameResult` construction sites — `LocalGame::advance()`'s GameOver
/// return and `GameDriver`'s Halted arm — route through the single
/// [`LocalGame::result_snapshot`] helper as of PB-DX32 Stage 1 (plan §7 R5). Before that,
/// they were two hand-maintained literals that had to agree by inspection alone.
///
/// This test drives one game to `GameOver` (a player already lost, so `is_game_over` is
/// true on the very first `advance()` call, 0 commands applied) and one to
/// `Halted(MaxTurns)` through the ACTUAL `GameDriver` code path (`max_turns: 3`, no bot
/// assigned to either seat so every priority window auto-passes) — the revert-proof
/// target is `driver.rs`'s Halted arm. The Halted `GameResult` is checked against an
/// INDEPENDENT, identically-parameterised `LocalGame` run directly (StubProvider and the
/// no-bot fallback are both deterministic, so the two runs reach the identical halt) —
/// the game's OWN accessors, not a value this test merely asserts is correct. This test
/// is strengthened at each later stage to cover each new instrumentation field (plan §5
/// Stage 1).
#[test]
fn test_dx32_halted_and_game_over_results_carry_the_same_instrumentation() {
    // ---- GameOver half ----
    let mut over_state = bare_two_player_state();
    over_state
        .players_mut()
        .get_mut(&p(2))
        .expect("p2 exists")
        .has_lost = true;

    let limits = LocalGameLimits {
        max_turns: 200,
        max_commands: 400,
        max_consecutive_passes: 500,
        record_journal: false,
    };
    let (mut over_game, _events) = LocalGame::start(
        over_state,
        1,
        StubProvider,
        no_bots(),
        BTreeSet::new(),
        limits,
        true,
    )
    .expect("bare two-player game must start");

    let over_result = match over_game.advance() {
        AdvanceOutcome::GameOver(result) => result,
        other => panic!("expected an immediate GameOver (p2 already lost): {other:?}"),
    };
    assert_eq!(
        over_result.winner,
        Some(p(1)),
        "the sole non-lost player must be reported as the winner"
    );
    assert_eq!(
        over_result.turn_count,
        over_game.state().turn().turn_number,
        "GameResult.turn_count must match the game's own turn accessor"
    );
    assert_eq!(
        over_result.total_commands,
        over_game.command_count() as usize,
        "GameResult.total_commands must match the game's own command_count accessor"
    );
    assert_eq!(
        over_result.total_commands, 0,
        "the loss precedes any command in this fixture"
    );

    // ---- Halted half, via GameDriver -- the PRODUCTION code path, and the revert-proof
    // target (driver.rs's Halted arm) ----
    const HALT_MAX_TURNS: u32 = 3;
    let halted_state = two_player_state_with_libraries(10);
    let driver = GameDriver::new(StubProvider, no_bots(), HALT_MAX_TURNS, 2);
    let (halted_result, _mechanics) = driver.run_game_with_mechanics(halted_state.clone(), 2);

    assert!(
        matches!(
            halted_result.error,
            Some(GameDriverError::MaxTurnsReached(HALT_MAX_TURNS))
        ),
        "expected a MaxTurnsReached({HALT_MAX_TURNS}) halt with no bot assigned to \
         either seat, got {:?}",
        halted_result.error
    );

    // An INDEPENDENT, identically-parameterised `LocalGame` reaches the same halt
    // deterministically (StubProvider offers no randomness and the no-bot fallback is a
    // fixed PassPriority), so its OWN accessors are what `result_snapshot` is checked
    // against here.
    let shadow_limits = LocalGameLimits {
        max_turns: HALT_MAX_TURNS,
        max_commands: HALT_MAX_TURNS * 200,
        max_consecutive_passes: 500,
        record_journal: false,
    };
    let (mut shadow_game, _events2) = LocalGame::start(
        halted_state,
        2,
        StubProvider,
        no_bots(),
        BTreeSet::new(),
        shadow_limits,
        true,
    )
    .expect("two-player state with libraries must start");
    match shadow_game.advance() {
        AdvanceOutcome::Halted(_) => {}
        other => panic!("expected the shadow game to halt identically: {other:?}"),
    }

    assert_eq!(
        halted_result.turn_count,
        shadow_game.state().turn().turn_number,
        "GameResult.turn_count must match the game's own turn accessor"
    );
    assert_eq!(
        halted_result.total_commands,
        shadow_game.command_count() as usize,
        "GameResult.total_commands must match the game's own command_count accessor"
    );
    assert!(
        halted_result.total_commands > 0,
        "non-vacuity: the MaxTurns valve must trip only after real commands were \
         applied, or a revert that hard-codes total_commands: 0 in driver.rs's Halted \
         arm would stay green"
    );
}

/// **T2.1** (Stage 2) — SR-38 (`OOS-SIM3-2`). A fuzz-shaped game with
/// `record_journal: false` (the fuzzer's own configuration) still SAMPLES rejections:
/// non-empty, capped at `MAX_SAMPLED_REJECTIONS`, and `rejection_count() >=
/// rejections().len()` (the count is never truncated; only the record is).
/// Non-vacuity: seed 1 at `max_turns: 25` is the exact seed Stage 0 measured producing
/// 85 rejections over 1,005 commands (`memory/primitive-wip.md`), so this fixture is
/// KNOWN, not hoped, to fire.
#[test]
fn test_dx32_rejections_are_sampled_without_the_journal() {
    let game = play_fuzz_shaped(1, 4, 25);

    assert!(
        game.rejection_count() > 0,
        "seed 1 at max_turns 25 is known to produce rejections (Stage 0 measured 85)"
    );
    assert!(
        !game.rejections().is_empty(),
        "record_journal: false must still sample SOME rejections (SR-38, OOS-SIM3-2)"
    );
    assert!(
        game.rejections().len() <= mtg_simulator::MAX_SAMPLED_REJECTIONS,
        "the sample must be capped at MAX_SAMPLED_REJECTIONS ({}), got {}",
        mtg_simulator::MAX_SAMPLED_REJECTIONS,
        game.rejections().len()
    );
    assert!(
        game.rejection_count() as usize >= game.rejections().len(),
        "the count must never be smaller than the (possibly truncated) sample"
    );
}

/// **T2.2** (Stage 2) — the SR-38 ratchet at the TEST gate's own configuration: 3 seeds
/// ([1, 2, 3]) x 25 turns x `RandomBot` x `build_fuzz_state`, `record_journal: false` —
/// the exact configuration Stage 0 measured (2,767 commands, 86 rejections = 31.081 per
/// mille). Aggregate per-mille must stay at or under
/// `MAX_BOT_REJECTION_PER_MILLE_AT_GATE_CONFIG`. Floors on `total_commands` and
/// `total_rejections` so a game that stops early, or a bot that stops acting, cannot
/// pass trivially.
#[test]
fn test_dx32_sr38_bot_rejection_rate_is_ratcheted() {
    let mut total_commands: u64 = 0;
    let mut total_rejections: u64 = 0;

    for &seed in &[1u64, 2, 3] {
        let game = play_fuzz_shaped(seed, 4, 25);
        eprintln!(
            "T2.2 seed {seed}: commands={} rejections={}",
            game.command_count(),
            game.rejection_count()
        );
        total_commands += u64::from(game.command_count());
        total_rejections += u64::from(game.rejection_count());
    }

    let per_mille = (total_rejections as f64 / total_commands as f64) * 1000.0;
    eprintln!("T2.2 aggregate: {total_rejections} / {total_commands} = {per_mille:.3} per mille");

    assert!(
        per_mille <= f64::from(mtg_simulator::MAX_BOT_REJECTION_PER_MILLE_AT_GATE_CONFIG),
        "aggregate rejection rate {per_mille:.3} per mille exceeds the ratchet \
         MAX_BOT_REJECTION_PER_MILLE_AT_GATE_CONFIG = {}",
        mtg_simulator::MAX_BOT_REJECTION_PER_MILLE_AT_GATE_CONFIG
    );
    // Non-vacuity floors (Stage 0 measured 2,767 commands / 86 rejections at this exact
    // configuration; 80% of the measured command count, per plan §5 Stage 0 step 4).
    assert!(
        total_commands >= 2_200,
        "non-vacuity floor: total_commands {total_commands} is far below the Stage-0 \
         measurement (2,767) — a game that stopped early cannot pass this gate trivially"
    );
    assert!(
        total_rejections > 0,
        "non-vacuity floor: the gate's own seeds are known to produce rejections"
    );
}

/// **T2.3** (Stage 2) — `GameResult.rejection_count` equals `LocalGame::rejection_count()`
/// on BOTH the GameOver and the Halted path, with a NON-ZERO count on both: a
/// zero-on-both fixture cannot discriminate a regression that silently drops the field
/// back to its `Default` value of 0. [`AlwaysRejectedBot`] forces the rejection on both
/// halves; [`ConcedeOnFirstCallBot`] forces the GameOver outcome (p2 concedes at its
/// first priority window, after p1's `AlwaysRejectedBot` has already been rejected
/// once), and the Halted half reuses [`two_player_state_with_libraries`] the same way
/// T1.1's Halted half does.
#[test]
fn test_dx32_game_result_carries_the_rejection_channel() {
    // ---- GameOver half ----
    let over_state = two_player_state_with_libraries(10);
    let mut over_bots: HashMap<PlayerId, Box<dyn Bot>> = HashMap::new();
    over_bots.insert(p(1), Box::new(AlwaysRejectedBot));
    over_bots.insert(p(2), Box::new(ConcedeOnFirstCallBot::new()));
    let limits = LocalGameLimits {
        max_turns: 200,
        max_commands: 400,
        max_consecutive_passes: 500,
        record_journal: false,
    };
    let (mut over_game, _events) = LocalGame::start(
        over_state,
        3,
        StubProvider,
        over_bots,
        BTreeSet::new(),
        limits,
        true,
    )
    .expect("fixture must start");
    let over_result = match over_game.advance() {
        AdvanceOutcome::GameOver(result) => result,
        other => panic!("expected GameOver once p2 concedes: {other:?}"),
    };
    assert!(
        over_result.rejection_count > 0,
        "p1's AlwaysRejectedBot must have produced >=1 rejection before p2 conceded: \
         {over_result:?}"
    );
    assert_eq!(
        over_result.rejection_count,
        over_game.rejection_count(),
        "GameResult.rejection_count must match the game's own accessor (GameOver path)"
    );

    // ---- Halted half ----
    const HALT_MAX_TURNS: u32 = 3;
    let halted_state = two_player_state_with_libraries(10);
    let mut halted_bots: HashMap<PlayerId, Box<dyn Bot>> = HashMap::new();
    halted_bots.insert(p(1), Box::new(AlwaysRejectedBot));
    // p2 gets no bot assigned -- falls to the "no bot assigned" auto-pass branch, same
    // as T1.1's Halted half.
    let halted_limits = LocalGameLimits {
        max_turns: HALT_MAX_TURNS,
        max_commands: HALT_MAX_TURNS * 200,
        max_consecutive_passes: 500,
        record_journal: false,
    };
    let (mut halted_game, _events2) = LocalGame::start(
        halted_state,
        4,
        StubProvider,
        halted_bots,
        BTreeSet::new(),
        halted_limits,
        true,
    )
    .expect("fixture must start");
    let halt_reason = match halted_game.advance() {
        AdvanceOutcome::Halted(reason) => reason,
        other => panic!("expected a MaxTurns halt: {other:?}"),
    };
    let halted_result = halted_game.result_snapshot(None, Some(halt_reason.into()));
    assert!(
        halted_result.rejection_count > 0,
        "p1's AlwaysRejectedBot must have produced >=1 rejection before the turn cap: \
         {halted_result:?}"
    );
    assert_eq!(
        halted_result.rejection_count,
        halted_game.rejection_count(),
        "GameResult.rejection_count must match the game's own accessor (Halted path)"
    );
}
