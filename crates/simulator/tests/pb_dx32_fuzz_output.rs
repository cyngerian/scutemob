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

use mtg_engine::{GameStateBuilder, ObjectSpec, PlayerId, ZoneId};
use mtg_simulator::{
    AdvanceOutcome, Bot, GameDriver, GameDriverError, LocalGame, LocalGameLimits, StubProvider,
};

fn p(n: u64) -> PlayerId {
    PlayerId(n)
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
