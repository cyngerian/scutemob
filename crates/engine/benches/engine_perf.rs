//! Performance benchmarks for the MTG Commander rules engine.
//!
//! Measures priority cycle time, SBA check time, and full turn processing time
//! for 4-player and 6-player Commander games.
//!
//! Baseline targets (red flag thresholds):
//! - Priority cycle: >10ms per complete priority round is a red flag
//! - SBA check:      >1ms per SBA check pass is a red flag
//! - Full turn:      informational only (no hard target at this stage)

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use mtg_engine::{
    check_and_apply_sbas, process_command, start_game, Command, GameState, GameStateBuilder,
    ObjectSpec, PlayerId, Step, ZoneId,
};

// ── State Factories ───────────────────────────────────────────────────────────

/// Build an N-player state at PreCombatMain with library cards for each player.
///
/// Each player gets 83 library cards so nobody decks out during full-turn benches.
fn build_np_state(n: u64, step: Step) -> GameState {
    let mut builder = GameStateBuilder::new();
    for pid in 1..=n {
        builder = builder.add_player(PlayerId(pid));
    }
    builder = builder.at_step(step);
    for pid in 1..=n {
        let player = PlayerId(pid);
        for i in 0..83 {
            builder = builder.object(
                ObjectSpec::card(player, &format!("Lib {}:{}", pid, i))
                    .in_zone(ZoneId::Library(player)),
            );
        }
    }
    builder.build().unwrap()
}

/// Build a state with 20 permanents on the battlefield for SBA benchmarking.
///
/// All creatures are healthy (2/2 with no damage) so no SBAs fire — this
/// measures the cost of the scan + fixed-point loop, not the cost of applying
/// SBA consequences.
fn build_sba_state() -> GameState {
    let mut builder = GameStateBuilder::four_player().at_step(Step::PreCombatMain);
    // 5 creatures per player × 4 players = 20 permanents total
    for pid in 1u64..=4 {
        let player = PlayerId(pid);
        for i in 0..5 {
            builder = builder.object(ObjectSpec::creature(
                player,
                &format!("Creature {}:{}", pid, i),
                2,
                2,
            ));
        }
    }
    builder.build().unwrap()
}

// ── Helper: advance priority until the step changes ──────────────────────────

/// Drive priority until all active players have passed once.
///
/// Returns the new GameState after the step (or turn) advances.
fn pass_until_advance(mut state: GameState) -> GameState {
    loop {
        let holder = match state.turn.priority_holder {
            Some(h) => h,
            None => break,
        };
        let (new_state, events) = process_command(state, Command::PassPriority { player: holder })
            .expect("PassPriority failed");
        let advanced = events.iter().any(|e| {
            matches!(
                e,
                mtg_engine::GameEvent::StepChanged { .. }
                    | mtg_engine::GameEvent::TurnStarted { .. }
            )
        });
        state = new_state;
        if advanced {
            break;
        }
    }
    state
}

// ── Benchmarks ────────────────────────────────────────────────────────────────

/// Benchmark: 4 players each pass priority once.
///
/// Measures a single complete priority round in a 4-player game at
/// PreCombatMain (empty stack). This is the inner loop of every interactive
/// game moment. Target: well under 10ms.
fn bench_priority_cycle_4p(c: &mut Criterion) {
    c.bench_function("priority_cycle_4p", |b| {
        b.iter_with_setup(
            || build_np_state(4, Step::PreCombatMain),
            |state| {
                // Pass priority until the step advances (all 4 players pass).
                black_box(pass_until_advance(black_box(state)))
            },
        )
    });
}

/// Benchmark: 6 players each pass priority once.
///
/// Same as 4p but with 6 players. Shows O(N) scaling of the priority loop.
/// Target: well under 10ms.
fn bench_priority_cycle_6p(c: &mut Criterion) {
    c.bench_function("priority_cycle_6p", |b| {
        b.iter_with_setup(
            || build_np_state(6, Step::PreCombatMain),
            |state| black_box(pass_until_advance(black_box(state))),
        )
    });
}

/// Benchmark: SBA check on a board with 20 permanents.
///
/// Calls `check_and_apply_sbas` directly on a state with 20 healthy creatures.
/// No SBAs should fire — measures scan + fixed-point termination cost.
/// Target: well under 1ms.
fn bench_sba_check(c: &mut Criterion) {
    c.bench_function("sba_check", |b| {
        b.iter_with_setup(
            || build_sba_state(),
            |mut state| {
                let events = check_and_apply_sbas(black_box(&mut state));
                black_box(events)
            },
        )
    });
}

/// Benchmark: complete a full turn for a 4-player game.
///
/// Starts at Upkeep and drives through every step of P1's turn
/// (Upkeep → Draw → PreCombatMain → BeginningOfCombat → DeclareAttackers →
/// DeclareBlockers → CombatDamage → EndOfCombat → PostCombatMain → End →
/// Cleanup → next player's Upkeep). No attackers are declared.
fn bench_full_turn_4p(c: &mut Criterion) {
    c.bench_function("full_turn_4p", |b| {
        b.iter_with_setup(
            || {
                let raw = build_np_state(4, Step::Untap);
                // start_game handles the Untap step and puts us at Upkeep.
                start_game(raw).expect("start_game failed").0
            },
            |state| {
                // Drive through all remaining steps of P1's turn until P2 becomes active.
                let mut s = state;
                loop {
                    let active_before = s.turn.active_player;
                    s = pass_until_advance(s);
                    // When the active player changes, P1's turn is over.
                    if s.turn.active_player != active_before {
                        break;
                    }
                }
                black_box(s)
            },
        )
    });
}

/// Benchmark: complete a full turn for a 6-player game.
///
/// Same as 4p but with 6 players. Each priority round is longer; measures
/// the combined cost of turn management × priority × 6-player overhead.
fn bench_full_turn_6p(c: &mut Criterion) {
    c.bench_function("full_turn_6p", |b| {
        b.iter_with_setup(
            || {
                let raw = build_np_state(6, Step::Untap);
                start_game(raw).expect("start_game failed").0
            },
            |state| {
                let mut s = state;
                loop {
                    let active_before = s.turn.active_player;
                    s = pass_until_advance(s);
                    if s.turn.active_player != active_before {
                        break;
                    }
                }
                black_box(s)
            },
        )
    });
}

criterion_group!(
    benches,
    bench_priority_cycle_4p,
    bench_priority_cycle_6p,
    bench_sba_check,
    bench_full_turn_4p,
    bench_full_turn_6p,
);
criterion_main!(benches);
