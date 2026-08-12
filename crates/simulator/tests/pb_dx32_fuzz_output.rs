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

use mtg_engine::rules::engine::BlockingDecision;
use mtg_engine::{
    all_cards, AttackTarget, CardDefinition, CardType, Command, EffectChoiceQuestion, GameState,
    GameStateBuilder, ObjectId, ObjectSpec, PendingEffectChoice, PlayerId, SuperType, ZoneId,
};
use mtg_simulator::{
    build_fuzz_state, build_registry, invariants, random_deck, row_id_for, AdvanceOutcome, Bot,
    GameDriver, GameDriverError, InvariantViolation, LegalAction, LocalGame, LocalGameLimits,
    RandomBot, StubProvider, OBSERVABLE_ROW_IDS,
};
use rand::{rngs::StdRng, SeedableRng};

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

/// **T3.1** (Stage 3) — the waste ratio at the TEST gate's own configuration: 3 seeds
/// x 25 turns x `RandomBot` x `build_fuzz_state`, `record_journal: false` (the same
/// configuration T2.2 uses). `wasted_taps * 100 / total_taps` must stay at or under
/// `MAX_RANDOM_BOT_WASTED_TAP_PCT_AT_GATE_CONFIG` — a SEPARATE pin from
/// `MAX_RANDOM_BOT_WASTED_TAP_PCT` (the fuzz BINARY's 200-turn threshold), for the
/// same reason T2.2 needed its own SR-38 pin distinct from the binary's: measured
/// live, this exact 3-seed/25-turn configuration produces 89% wasted taps, ABOVE the
/// 200-turn population's 85% ceiling — reusing that ceiling here would be red on
/// arrival (see the constant's own doc for why the two populations genuinely differ).
/// Floor: `total_taps > 0`, so a game with no taps at all cannot pass trivially.
#[test]
fn test_dx32_random_bot_waste_ratio_is_bounded() {
    let mut total_taps: u64 = 0;
    let mut wasted_taps: u64 = 0;

    for &seed in &[1u64, 2, 3] {
        let game = play_fuzz_shaped(seed, 4, 25);
        let waste = game.waste();
        eprintln!("T3.1 seed {seed}: {waste:?}");
        total_taps += u64::from(waste.total_taps);
        wasted_taps += u64::from(waste.wasted_taps);
    }

    // Review finding M7: a bare `> 0` floor is a token gesture next to T2.2's own
    // measured 80%-of-baseline floor twelve lines away in that test. Stage 0 measured
    // 97 taps at this exact configuration; 77 is 80% of that, same rule T2.2 uses, so
    // a change that collapsed the tap population (a bot scoring change, an offer-gate
    // change, a `build_fuzz_state` change) cannot pass this gate at a single unwasted
    // tap.
    assert!(
        total_taps >= 77,
        "non-vacuity floor: total_taps {total_taps} is far below the Stage-0 \
         measurement (97) at this configuration — a run that stopped tapping cannot \
         pass this gate trivially"
    );
    let pct = wasted_taps * 100 / total_taps;
    eprintln!("T3.1 aggregate: {wasted_taps} / {total_taps} = {pct}%");
    assert!(
        pct <= u64::from(mtg_simulator::MAX_RANDOM_BOT_WASTED_TAP_PCT_AT_GATE_CONFIG),
        "RandomBot wasted-tap ratio {pct}% exceeds \
         MAX_RANDOM_BOT_WASTED_TAP_PCT_AT_GATE_CONFIG = {} -- RandomBot wastes taps BY \
         DESIGN (no plan), so this is a ceiling on ordinary behaviour, not a zero target",
        mtg_simulator::MAX_RANDOM_BOT_WASTED_TAP_PCT_AT_GATE_CONFIG
    );
}

/// **T4.1** (Stage 4) — CR 704.3 / `OOS-M11-7`: `no_orphaned_tokens` reports are
/// transient by construction, and the strictly stronger end-state property holds.
/// Seed 2 at `max_turns: 25` (the exact `play_fuzz_shaped` configuration T2.x/T3.x
/// already use) is KNOWN, not hoped, to produce them: measured at implementation time,
/// 4 raw `no_orphaned_tokens` reports (all the same Treasure token, turn 24), 0 hard
/// violations, 0 leaked tokens in the final state.
///
/// **Re-measured after PB-DX21** (2026-08-04, `scutemob-200`, review finding
/// M7): still 4 raw reports, UNMOVED, and this is PROVEN, not just observed:
/// disabling PB-DX21's `legal_actions.rs` offer-suppression clause entirely
/// and re-running this exact seed produces a byte-identical
/// `command_count`/`rejection_count`/`transient_violations().len()` triple —
/// the suppression window (a same-active-player re-priority within one
/// `DeclareAttackers` step, which needs a mid-step instant response) is
/// simply never reached by this specific low-turn, low-complexity trajectory,
/// unlike the T2.2/T3.1 gate-config aggregate above, which spans a wider
/// 3-seed sample and does move.
#[test]
fn test_dx32_orphaned_tokens_are_transient_and_the_end_state_is_clean() {
    let game = play_fuzz_shaped(2, 4, 25);

    assert!(
        !game.transient_violations().is_empty(),
        "seed 2 at max_turns 25 is known to produce no_orphaned_tokens transient reports \
         (measured at implementation time: 4 raw reports)"
    );
    assert!(
        game.transient_violations()
            .iter()
            .all(|v| v.check == "no_orphaned_tokens"),
        "transient_violations() must contain ONLY no_orphaned_tokens: {:?}",
        game.transient_violations()
    );
    assert!(
        game.violations()
            .iter()
            .all(|v| v.check != "no_orphaned_tokens"),
        "violations() (the hard bucket) must contain NO no_orphaned_tokens -- the split \
         must be exhaustive in both directions: {:?}",
        game.violations()
    );
    let leaked = invariants::check_no_leaked_tokens(game.state());
    assert!(
        leaked.is_empty(),
        "CR 704.3 / OOS-M11-7: the transient reports must actually BE transient -- no \
         token may remain outside the battlefield in the FINAL state, or the split would \
         be hiding a real defect: {leaked:?}"
    );
}

/// **T4.2** (Stage 4) — `check_no_leaked_tokens`, both directions (the paired-probe
/// convention `invariants.rs`'s own test module already uses). A state with no token
/// at all is silent; a hand-built terminal state with one token stranded in a
/// graveyard produces exactly one `leaked_tokens` violation.
#[test]
fn test_dx32_leaked_token_at_game_end_is_a_hard_violation() {
    let clean = bare_two_player_state();
    assert!(
        invariants::check_no_leaked_tokens(&clean).is_empty(),
        "a state with no token at all must be silent"
    );

    let leaked_state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .object(
            ObjectSpec::card(p(1), "Spirit")
                .token()
                .in_zone(ZoneId::Graveyard(p(1))),
        )
        .build()
        .expect("leaked-token fixture must build");

    let violations = invariants::check_no_leaked_tokens(&leaked_state);
    assert_eq!(
        violations.len(),
        1,
        "exactly one token, exactly one violation: {violations:?}"
    );
    assert_eq!(violations[0].check, "leaked_tokens");
}

/// **T4.3** (Stage 4) — `distinct` collapses checkpoint weighting (`OOS-SIM3-3`):
/// first occurrence per `(check, description)` wins, order preserved. The hand-built
/// half proves the ORDER guarantee (three identical `(check, description)` pairs at
/// three different turn numbers — neither field carries the turn, so all three
/// collapse, and the FIRST turn number survives); the real-seeded half (seed 2, the
/// same fixture T4.1 uses) proves the collapse on genuine engine output, matching
/// Stage 0's own 94 -> 20 collapse at full scale (§0.3).
///
/// **Re-measured after PB-DX21** (review finding M7): the real-seeded half
/// still measures raw=4/distinct=1, UNMOVED -- same fixture as T4.1, same
/// ablation-proven reason (see T4.1's doc).
#[test]
fn test_dx32_distinct_collapses_checkpoint_weighting() {
    let hand_built = vec![
        InvariantViolation {
            check: "no_orphaned_tokens".into(),
            description: "Token ObjectId(1) 'Spirit' found in zone Graveyard(PlayerId(1))".into(),
            turn_number: 3,
        },
        InvariantViolation {
            check: "no_orphaned_tokens".into(),
            description: "Token ObjectId(1) 'Spirit' found in zone Graveyard(PlayerId(1))".into(),
            turn_number: 4,
        },
        InvariantViolation {
            check: "no_orphaned_tokens".into(),
            description: "Token ObjectId(1) 'Spirit' found in zone Graveyard(PlayerId(1))".into(),
            turn_number: 5,
        },
    ];
    let deduped = invariants::distinct(&hand_built);
    assert_eq!(deduped.len(), 1, "{deduped:?}");
    assert_eq!(
        deduped[0].turn_number, 3,
        "the FIRST occurrence must be preserved, not the last"
    );

    let game = play_fuzz_shaped(2, 4, 25);
    let raw = game.transient_violations();
    let distinct = invariants::distinct(raw);
    assert!(
        distinct.len() < raw.len(),
        "seed 2 at max_turns 25 is known to repeat a violation (Stage 0's own 94 -> 20 \
         collapse at full scale, §0.3): raw {} distinct {}",
        raw.len(),
        distinct.len()
    );
}

// ── Stage 5 -- (d) the corpus→seed gate (`OOS-CARDS2-3`) ───────────────────────────

/// Measured on this branch, 2026-08-03. An exact pin, both directions (the
/// `MAX_AUTO_CHOSEN_COMPLETE_UNION` idiom, F18): a completeness flip that changes any
/// of these three numbers silently re-rolls every recorded fuzz seed, because
/// `random_deck` (`deck.rs:30-157`) draws its commander and its color-identity pool
/// straight from `all_cards()`.
const CORPUS_DEFS: usize = 1803;
// PB-DX26 (2026-08-11, `scutemob-206`): UNCHANGED at 1133, and that is a trap worth
// naming. Two markers moved in opposite directions and the COUNT cancelled while the
// SET did not: `sword_of_body_and_mind` `partial` -> `Complete` (its only blocker was
// the missing Equip {2} that `OOS-CARDS1-3` authored) and `the_reaver_cleaver`
// derive-`Complete` -> `partial` (an honest demotion, review Finding 7). The fuzz deck
// pool therefore holds a DIFFERENT card than it did, so every seeded fixture still
// deals a different game -- exactly what MOVED_MSG warns about -- while this constant
// stays put and cannot warn about it. `UI3_SPLIT_COMBAT_SEED` in
// `tools/play-server/src/main.rs` was re-observed for precisely this reason.
// Re-measured by executing this gate, not predicted.
const CORPUS_COMPLETE: usize = 1133;
const COMMANDER_POOL: usize = 90;

/// Mirrors `crates/simulator/src/deck.rs:40-47`'s three-clause commander filter
/// EXACTLY -- not re-derived from memory, so this pin cannot stay green while
/// `random_deck`'s own filter changes underneath it (plan §3.6). T5.2 is the proof
/// that this mirror has NOT diverged from the real filter.
fn commander_pool() -> Vec<CardDefinition> {
    all_cards()
        .into_iter()
        .filter(|c| {
            c.completeness.is_complete()
                && c.types.supertypes.contains(&SuperType::Legendary)
                && c.types.card_types.contains(&CardType::Creature)
        })
        .collect()
}

/// **T5.1** (Stage 5) — `OOS-CARDS2-3`: the fuzz deck pool size is pinned, exactly, in
/// both directions. Before this gate, a completeness flip that changed the pool size
/// was discovered only by watching eight seeded fixtures across the workspace go red
/// one at a time; this gate makes the change announce itself in one place, with a
/// message that says what to do.
#[test]
fn test_dx32_fuzz_deck_pool_size_is_pinned() {
    let defs = all_cards();
    let complete = defs.iter().filter(|c| c.completeness.is_complete()).count();
    let commanders = commander_pool().len();

    const MOVED_MSG: &str = "the fuzz deck pool changed. Every seeded fixture in the \
         workspace now deals a different game (OOS-CARDS2-3). Update these three \
         constants in the SAME commit as the card-def change, and expect the seeded \
         pins listed in memory/workstream-state.md (CARDS-2 handoff, item 1) to move \
         -- including this file's OWN other seeded gates (T2.2, T3.1, T4.1, T4.3, \
         T6.3), which deal from the same corpus and will redden alongside this one.";

    assert_eq!(
        defs.len(),
        CORPUS_DEFS,
        "all_cards().len() moved from the pinned CORPUS_DEFS ({CORPUS_DEFS}) to {} -- {MOVED_MSG}",
        defs.len()
    );
    assert_eq!(
        complete, CORPUS_COMPLETE,
        "the Complete-def count moved from the pinned CORPUS_COMPLETE \
         ({CORPUS_COMPLETE}) to {complete} -- {MOVED_MSG}"
    );
    assert_eq!(
        commanders, COMMANDER_POOL,
        "the commander pool (Complete + Legendary + Creature, deck.rs:40-47) moved \
         from the pinned COMMANDER_POOL ({COMMANDER_POOL}) to {commanders} -- {MOVED_MSG}"
    );
}

/// **T5.2** (Stage 5) — non-vacuity + anti-drift for T5.1's mirrored filter. Without
/// this, `commander_pool`'s mirror could diverge from `deck.rs`'s own filter while
/// T5.1's exact pin stayed green (a filter that always returned the empty set would
/// still satisfy an exact-count pin of 0 just as well as one of 90). Two independent
/// checks: the pool sits strictly between empty and the full Complete corpus
/// (non-vacuity), and `random_deck` on a fixed seed actually picks a commander that
/// IS a member of the recomputed pool (the real anti-drift proof — if
/// `commander_pool`'s filter ever diverges from `deck.rs`'s own, `random_deck`'s pick
/// would fall outside it).
#[test]
fn test_dx32_commander_pool_filter_mirrors_deck_rs() {
    let defs = all_cards();
    let pool = commander_pool();
    let complete_count = defs.iter().filter(|c| c.completeness.is_complete()).count();
    assert!(
        !pool.is_empty() && pool.len() < complete_count,
        "COMMANDER_POOL must be a non-empty STRICT subset of the Complete corpus: \
         pool={} complete={complete_count}",
        pool.len()
    );

    let mut rng = StdRng::seed_from_u64(1);
    let deck =
        random_deck(&mut rng, &defs).expect("random_deck must succeed against the real corpus");
    assert!(
        pool.iter().any(|c| c.card_id == deck.commander),
        "random_deck's own commander pick ({:?}) must be a member of the mirrored \
         commander_pool -- if this fails, commander_pool's filter has diverged from \
         deck.rs's own (plan §3.6)",
        deck.commander
    );
}

// ── Stage 6 -- (e) decision-point runtime coverage ──────────────────────────────────

/// A `GameState` carrying a hand-built `PendingEffectChoice` (mirrors
/// `crates/engine/tests/core/hash_schema.rs:807`'s own fixture pattern) — the only
/// public way to seed `state.pending_effect_choice()` from outside the engine crate
/// (`GameStateBuilder::pending_effect_choice`).
fn effect_choice_state(question: EffectChoiceQuestion) -> GameState {
    GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .pending_effect_choice(PendingEffectChoice {
            choice_id: 1,
            player: p(1),
            source: ObjectId(1),
            question,
            index: 0,
        })
        .build()
        .expect("effect-choice fixture must build")
}

/// **T6.2** (Stage 6) — the non-vacuity partner of T6.1: `decision_coverage::row_id_for`
/// is exercised from REAL code (a hand-built `BlockingDecision` +, for the
/// `EffectChoice` rows, a matching `PendingEffectChoice`), not merely declared in a
/// constant list. One fixture per observable row; asserts the returned id AND that the
/// full set of ids `row_id_for` can ever return equals `OBSERVABLE_ROW_IDS` exactly.
/// Also proves `CleanupDiscard` (CR 514.1) maps to `None` — a real decision with no
/// `ROWS` row, not a silently-skipped one.
#[test]
fn test_dx32_row_id_for_covers_every_observable_row() {
    let cases: Vec<(&str, BlockingDecision, GameState)> = vec![
        (
            "triggered_targets",
            BlockingDecision::TriggerTargets {
                player: p(1),
                choice_id: 1,
                source: ObjectId(1),
            },
            bare_two_player_state(),
        ),
        (
            "search_library",
            BlockingDecision::EffectChoice {
                player: p(1),
                choice_id: 1,
                source: ObjectId(1),
            },
            effect_choice_state(EffectChoiceQuestion::SearchLibrary {
                candidates: vec![],
                may_fail_to_find: false,
            }),
        ),
        (
            "scry",
            BlockingDecision::EffectChoice {
                player: p(1),
                choice_id: 1,
                source: ObjectId(1),
            },
            effect_choice_state(EffectChoiceQuestion::Scry { looked_at: vec![] }),
        ),
        (
            "surveil",
            BlockingDecision::EffectChoice {
                player: p(1),
                choice_id: 1,
                source: ObjectId(1),
            },
            effect_choice_state(EffectChoiceQuestion::Surveil { looked_at: vec![] }),
        ),
        (
            "discard_cards",
            BlockingDecision::EffectChoice {
                player: p(1),
                choice_id: 1,
                source: ObjectId(1),
            },
            effect_choice_state(EffectChoiceQuestion::Discard {
                hand: vec![],
                count: 0,
            }),
        ),
    ];

    let mut reachable: BTreeSet<&'static str> = BTreeSet::new();
    for (expected_id, decision, state) in &cases {
        let got = row_id_for(state, decision);
        assert_eq!(
            got,
            Some(*expected_id),
            "row_id_for must return {expected_id:?} for this fixture, got {got:?}"
        );
        if let Some(id) = got {
            reachable.insert(id);
        }
    }

    // Review finding L9: this test observes only the five fixtures constructed above
    // -- it does not itself prove `row_id_for` can NEVER return anything else. That
    // bound comes from `row_id_for`'s own match being EXHAUSTIVE with no wildcard on
    // both `BlockingDecision` and `EffectChoiceQuestion` (a compile-time property, not
    // a runtime one this test can observe). What this assertion DOES prove: every id
    // in OBSERVABLE_ROW_IDS is reachable from a real fixture, and every fixture above
    // maps to a row in OBSERVABLE_ROW_IDS -- non-vacuity in both directions.
    let observable: BTreeSet<&'static str> = OBSERVABLE_ROW_IDS.iter().copied().collect();
    assert_eq!(
        reachable, observable,
        "these five fixtures must reach exactly OBSERVABLE_ROW_IDS -- every id \
         reachable from a real fixture, and no fixture mapping outside the list"
    );

    // CR 514.1: CleanupDiscard is a real decision with NO ROWS row -- proven here as
    // an explicit None, not merely never exercised.
    let cleanup = BlockingDecision::CleanupDiscard {
        player: p(1),
        count: 1,
    };
    assert_eq!(
        row_id_for(&bare_two_player_state(), &cleanup),
        None,
        "CleanupDiscard (CR 514.1) is a real decision with no ROWS row"
    );
}

/// **T6.3** (Stage 6, plan §7 R9) — does a fuzz-shaped run reach at least one served
/// decision row? R9 was explicitly unmeasured in the plan, and the honest answer,
/// widening seeds/turns until measured rather than guessed, is BETTER than the
/// plan's own worst-case hypothesis ("possibly 0 of 5"): at 10 fuzz-shaped games x
/// 60 turns (`RandomBot`, `build_fuzz_state`, `record_journal: false`), **4 of the 5
/// served rows are reached** — `triggered_targets`, `search_library`, `scry`,
/// `discard_cards` — and exactly one, `surveil`, is never reached at this budget.
/// Deterministic (re-run twice at implementation time, identical partition both
/// times), so this is asserted EXACTLY, not as a floor, and the message on failure
/// tells the reader this is a finding to report, not a knob to retune blindly.
///
/// **Re-measured after PB-DX21** (review finding M7): identical partition,
/// UNMOVED -- `{"discard_cards", "scry", "search_library", "triggered_targets"}`
/// reached, `{"surveil"}` never reached, across 10 seeds x 60 turns. Less
/// surprising than T4.1's byte-exact match: this is a coarse binary
/// reached/never-reached membership test over a much larger aggregate (10
/// seeds), so a trajectory perturbation would have to flip EVERY seed's
/// outcome for `surveil` specifically to move this partition, which none did.
#[test]
fn test_dx32_a_fuzz_run_reaches_at_least_one_served_row() {
    let mut combined = mtg_simulator::DecisionCoverage::default();
    for seed in 1u64..=10 {
        let game = play_fuzz_shaped(seed, 4, 60);
        let coverage = game.decision_coverage();
        for id in OBSERVABLE_ROW_IDS {
            for _ in 0..coverage.observations(id) {
                combined.observe(id);
            }
        }
    }
    let reached: BTreeSet<&str> = combined.reached().into_iter().collect();
    let never_reached: BTreeSet<&str> = combined.never_reached().into_iter().collect();
    eprintln!("T6.3 reached: {reached:?}");
    eprintln!("T6.3 never reached: {never_reached:?}");

    let expected_reached: BTreeSet<&str> = [
        "triggered_targets",
        "search_library",
        "scry",
        "discard_cards",
    ]
    .into_iter()
    .collect();
    assert_eq!(
        reached, expected_reached,
        "the reached/never-reached partition of a 10-seed x 60-turn fuzz-shaped run \
         changed from the measured baseline (4 of 5 served rows: triggered_targets, \
         search_library, scry, discard_cards; surveil never reached at this budget). \
         Report this as a finding (does the engine now serve fewer/more decisions, or \
         did an unrelated change move which cards get drawn/cast) rather than \
         silently re-tuning the seed range to make it pass: reached {reached:?}, \
         never reached {never_reached:?}"
    );
}
