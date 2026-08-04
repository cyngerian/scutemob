//! mtg-fuzzer — run thousands of automated Commander games in parallel.
//!
//! Usage:
//!   mtg-fuzzer [OPTIONS]
//!
//! Options:
//!   --games <N>         Number of games (default: 1000)
//!   --players <N>       Players per game, 2-6 (default: 4)
//!   --max-turns <N>     Turn limit per game (default: 200)
//!   --seed <SEED>       Base RNG seed (default: random)
//!   --threads <N>       Parallel threads (default: num_cpus)
//!   --bot <TYPE>        random | heuristic (default: random)
//!   --stop-on-error     Stop after first violation
//!   --replay <SEED>     Replay a specific game by seed
//!   --verbose           Print each game result
//!
//! # Run it with assertions on (SR-32)
//!
//! The SR-4 / SR-14 tripwires that flag IMPOSSIBLE-class state absences are
//! `debug_assert!`s and are compiled out of `--release`. Fuzz through the
//! dedicated `fuzz` profile (release speed, `debug-assertions = true`,
//! `overflow-checks = true`; defined in the workspace `Cargo.toml`) so an
//! anomaly actually trips them instead of silently fizzling:
//!
//! ```text
//! cargo run --profile fuzz --bin mtg-fuzzer -- --games 10000 --stop-on-error
//! ```
//!
//! # Repro seeds are not portable across engine changes
//!
//! A `--replay <SEED>` reproduces a game only against the *same* engine build
//! that produced it. Seeds recorded before 2026-07-10 are dead: SR-10 moved the
//! RNG to `rand` 0.9 (different value streams for a fixed seed) and SR-12 added
//! the `random_deck` Complete-only deck-pool filter, so a given seed now evolves
//! a different game. Treat a crash seed as valid only within the run (and build)
//! that emitted it; capture the crash JSON, not just the seed, for anything that
//! must outlive the build.
//!
//! **Boundary event: the PB-DX22 merge (`scutemob-196`, `95f53b78`).** Every seed recorded
//! before it is dead, and this one moves more than the earlier two did. The deal
//! itself changed: `fuzz_setup::build_fuzz_state` now shuffles each library from
//! the game's own seeded RNG (CR 103.3 / 903.6) and registers each seat's
//! commander (CR 903.6 / 903.8), so a fixed seed deals a different opening
//! library AND offers a command-zone cast that did not exist before. Nothing
//! about a pre-merge seed survives that. Filed as `OOS-DX22-7`.
//!
//! The A/B, with each side attributed to the instrument that produced it — as
//! shipped, PB-DX22 filed all four pairs under this binary's own command line,
//! and this binary could not then print two of them:
//!
//! | metric | before | after | instrument, and its denominator |
//! |---|---|---|---|
//! | avg turns / halt distribution | 191.7; 9 wins + 11 `MaxTurnsReached` | 103.4; 20 wins + 0 errors | **this binary**, `--games 20 --seed 1 --max-turns 200 --threads 1 --profile fuzz` (20 games each side) |
//! | first `SpellCast` game turn | 143-154 | 3-29 (median 12) | **before**: a scratch instrument over **5** games (`memory/primitives/pb-dx22-measurement-head.txt`). **after**: this binary's `print_mechanics_summary`, **20** games |
//! | `CommanderCastFromCommandZone` | 0 in ~56,800 commands | 36, in 16 of 20 games | as above — the "0" is a **5**-game number, the "36" a **20**-game one |
//!
//! Both post-fix rows are printed by this binary as of the PB-DX22 fix cycle
//! (review Finding 3), so the attribution above is now checkable by running the
//! command; the raw run is committed at
//! `memory/primitives/pb-dx22-measurement-after-fixcycle.txt`. The pre-fix rows
//! cannot be: the build path they measured no longer exists.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Instant;

use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;

use mtg_engine::{all_cards, CardDefinition, CardRegistry, PlayerId};
use mtg_simulator::{
    build_fuzz_state, build_registry, CrashReport, FuzzSetupError, GameDriver, GameDriverError,
    GameResult, HeuristicBot, MechanicsTally, RandomBot, StubProvider,
};
use rand::prelude::*;

#[derive(Parser)]
#[command(
    name = "mtg-fuzzer",
    about = "Fuzz-test the MTG Commander engine with automated bot games",
    version
)]
struct Cli {
    /// Number of games to run
    #[arg(long, default_value = "1000")]
    games: u32,

    /// Players per game (2-6)
    #[arg(long, default_value = "4")]
    players: u32,

    /// Maximum turns per game before declaring a draw
    #[arg(long, default_value = "200")]
    max_turns: u32,

    /// Base RNG seed (each game uses base_seed + game_index)
    #[arg(long)]
    seed: Option<u64>,

    /// Number of parallel threads (default: num_cpus)
    #[arg(long)]
    threads: Option<usize>,

    /// Bot type: random or heuristic
    #[arg(long, default_value = "random")]
    bot: BotType,

    /// Stop after first invariant violation
    #[arg(long)]
    stop_on_error: bool,

    /// Replay a specific game by its seed (single-threaded, verbose)
    #[arg(long)]
    replay: Option<u64>,

    /// Print result of each game
    #[arg(long)]
    verbose: bool,
}

#[derive(Clone, Debug, clap::ValueEnum)]
enum BotType {
    Random,
    Heuristic,
}

fn main() {
    let cli = Cli::parse();

    // Set thread pool size
    if let Some(threads) = cli.threads {
        rayon::ThreadPoolBuilder::new()
            .num_threads(threads)
            .build_global()
            .ok();
    }

    let base_seed = cli.seed.unwrap_or_else(|| rand::rng().random());
    let registry = build_registry();
    let cards = all_cards();

    println!("MTG Fuzzer — Commander Rules Engine");
    println!("===================================");
    println!(
        "Games: {}  Players: {}  Max turns: {}  Bot: {:?}  Seed: {}",
        cli.games, cli.players, cli.max_turns, cli.bot, base_seed
    );
    println!("Cards available: {}", cards.len());
    println!();

    // Single-game replay mode
    if let Some(replay_seed) = cli.replay {
        println!("Replaying game with seed {}...", replay_seed);
        let (result, mechanics) = run_single_game(
            replay_seed,
            cli.players,
            cli.max_turns,
            &cli.bot,
            &cards,
            &registry,
        );
        print_game_result(&result, true);
        // One game, so the per-run summaries below are a per-game report here — which is
        // what a replay is for. `print_game_result(.., true)` above already printed ALL
        // of this game's violations, not the first five.
        print_violation_histogram(std::slice::from_ref(&result));
        print_mechanics_summary(std::slice::from_ref(&mechanics));
        print_sr38_summary(std::slice::from_ref(&result));
        print_waste_summary(std::slice::from_ref(&result), &cli.bot);
        return;
    }

    // Parallel fuzzing
    let start = Instant::now();
    let violation_count = AtomicUsize::new(0);
    let completed_count = AtomicUsize::new(0);
    let error_count = AtomicUsize::new(0);
    let should_stop = AtomicBool::new(false);

    let pb = ProgressBar::new(cli.games as u64);
    pb.set_style(
        ProgressStyle::with_template("[{bar:40.cyan/blue}] {pos}/{len} games  {msg}")
            .unwrap()
            .progress_chars("##-"),
    );

    let outcomes: Vec<(GameResult, MechanicsTally)> = (0..cli.games)
        .into_par_iter()
        .filter_map(|i| {
            if cli.stop_on_error && should_stop.load(Ordering::Relaxed) {
                return None;
            }

            let game_seed = base_seed.wrapping_add(i as u64);
            let (result, mechanics) = run_single_game(
                game_seed,
                cli.players,
                cli.max_turns,
                &cli.bot,
                &cards,
                &registry,
            );

            if !result.violations.is_empty() {
                violation_count.fetch_add(result.violations.len(), Ordering::Relaxed);
                if cli.stop_on_error {
                    should_stop.store(true, Ordering::Relaxed);
                }
            }

            if result.error.is_some() {
                error_count.fetch_add(1, Ordering::Relaxed);
            }

            let done = completed_count.fetch_add(1, Ordering::Relaxed) + 1;
            let viols = violation_count.load(Ordering::Relaxed);
            let errs = error_count.load(Ordering::Relaxed);
            pb.set_position(done as u64);
            pb.set_message(format!("{} violations  {} errors", viols, errs));

            Some((result, mechanics))
        })
        .collect();

    pb.finish_with_message("done");
    let elapsed = start.elapsed();

    let results: Vec<GameResult> = outcomes.iter().map(|(r, _)| r.clone()).collect();
    let tallies: Vec<MechanicsTally> = outcomes.iter().map(|(_, m)| *m).collect();

    // Summary
    println!();
    println!("Results");
    println!("-------");
    println!(
        "Games completed: {}  Time: {:.1}s  ({:.0} games/sec)",
        results.len(),
        elapsed.as_secs_f64(),
        results.len() as f64 / elapsed.as_secs_f64()
    );

    let wins: usize = results.iter().filter(|r| r.winner.is_some()).count();
    let draws: usize = results
        .iter()
        .filter(|r| r.winner.is_none() && r.error.is_none())
        .count();
    let errors: usize = results.iter().filter(|r| r.error.is_some()).count();
    let total_violations: usize = results.iter().map(|r| r.violations.len()).sum();
    // PB-DX32 Stage 4: printed as a SEPARATE line, deliberately -- `OOS-DX22-13`'s
    // lesson is that redefining what a headline number means without saying so is how
    // a stale comparison gets made. "Total violations" now means the HARD bucket only
    // (what `--stop-on-error` halts on); the TRANSIENT count is a different number,
    // printed next to it rather than folded in.
    let total_transient: usize = results.iter().map(|r| r.transient_violations.len()).sum();
    let avg_turns: f64 =
        results.iter().map(|r| r.turn_count as f64).sum::<f64>() / results.len().max(1) as f64;

    println!("Wins: {}  Draws: {}  Errors: {}", wins, draws, errors);
    println!("Total violations (HARD): {}", total_violations);
    println!(
        "Total violations (TRANSIENT, reported -- does not halt --stop-on-error): {}",
        total_transient
    );
    println!("Avg turns per game: {:.1}", avg_turns);

    print_violation_histogram(&results);
    print_mechanics_summary(&tallies);
    let sr38_breached = print_sr38_summary(&results);
    print_waste_summary(&results, &cli.bot);

    if cli.verbose {
        for result in &results {
            print_game_result(result, false);
        }
    }

    // Print first few violations for debugging
    let mut violation_seeds: Vec<u64> = Vec::new();
    for result in &results {
        if !result.violations.is_empty() {
            violation_seeds.push(result.seed);
            if violation_seeds.len() <= 5 {
                println!();
                println!("Violation in game seed {}:", result.seed);
                for v in &result.violations {
                    println!("  [{}] {} (turn {})", v.check, v.description, v.turn_number);
                }
            }
        }
    }

    if violation_seeds.len() > 5 {
        println!(
            "... and {} more games with violations",
            violation_seeds.len() - 5
        );
    }

    if !violation_seeds.is_empty() {
        println!();
        println!("Replay violations with: mtg-fuzzer --replay <SEED>");
    }

    // Write crash reports for games with violations
    let crash_dir = std::path::Path::new("crash-reports");
    if !results.iter().all(|r| r.violations.is_empty()) {
        std::fs::create_dir_all(crash_dir).ok();
        for result in &results {
            if let Some(v) = result.violations.first() {
                let report = CrashReport {
                    seed: result.seed,
                    player_count: cli.players as usize,
                    violation: v.clone(),
                    command_history: Vec::new(), // Would need to capture during game
                    turn_number: v.turn_number,
                    total_commands: result.total_commands,
                };
                let path = crash_dir.join(format!("crash_{}.json", result.seed));
                report.write_to_file(&path).ok();
            }
        }
    }

    // SR-38 (PB-DX32 Stage 2): fail the run loudly on a threshold breach. Safe to do
    // unconditionally -- F19, this binary is not run in CI, so a non-zero exit here
    // cannot redden a pipeline.
    if sr38_breached {
        std::process::exit(1);
    }
}

fn run_single_game(
    seed: u64,
    player_count: u32,
    max_turns: u32,
    bot_type: &BotType,
    cards: &[CardDefinition],
    registry: &Arc<CardRegistry>,
) -> (GameResult, MechanicsTally) {
    // PB-DX22 §B3: the state build lives in `mtg_simulator::fuzz_setup` so integration
    // tests can reach it. This function does nothing else to the state, so a probe on
    // `build_fuzz_state` is a probe on this binary.
    let player_ids: Vec<PlayerId> = (1..=player_count).map(|i| PlayerId(i as u64)).collect();

    let state = match build_fuzz_state(seed, player_count, cards, registry) {
        Ok(setup) => setup.state,
        // Byte-identical to the string this arm produced before the extraction — crash
        // reports and `driver.rs`'s error-shape comment depend on it.
        Err(FuzzSetupError::Builder(e)) => {
            return (
                // Error path: state build failed before any `LocalGame` existed.
                // `..Default::default()` picks up every instrumentation field PB-DX32
                // adds (starting with this stage's `rejection_count`/`rejections`)
                // without this site ever needing another edit (plan §5 Stage 1 step 2
                // named this site; the edit landed here, at Stage 2, per plan §7 R7 —
                // see the Stage 1 handoff for why).
                GameResult {
                    seed,
                    winner: None,
                    turn_count: 0,
                    total_commands: 0,
                    violations: Vec::new(),
                    error: Some(GameDriverError::EngineError(format!(
                        "Failed to build state: {:?}",
                        e
                    ))),
                    ..Default::default()
                },
                MechanicsTally::default(),
            );
        }
    };

    // Create bots
    let mut bots: HashMap<PlayerId, Box<dyn mtg_simulator::Bot>> = HashMap::new();
    for (i, &pid) in player_ids.iter().enumerate() {
        let bot_seed = seed.wrapping_add(100 + i as u64);
        let name = format!("Bot-{}", pid.0);
        let bot: Box<dyn mtg_simulator::Bot> = match bot_type {
            BotType::Random => Box::new(RandomBot::new(bot_seed, name)),
            BotType::Heuristic => Box::new(HeuristicBot::new(bot_seed, name)),
        };
        bots.insert(pid, bot);
    }

    // Run game
    let driver = GameDriver::new(StubProvider, bots, max_turns, seed);
    driver.run_game_with_mechanics(state, seed)
}

/// One bucket of [`print_violation_histogram`] — HARD (`result.violations`) or
/// TRANSIENT (`result.transient_violations`), selected by `select`. Prints raw AND
/// distinct counts (`OOS-SIM3-3`'s "report distinct conditions alongside the raw
/// count" prescription — `distinct` is `mtg_simulator::invariants::distinct`, the
/// SAME dedupe the noise-floor split's own gates use, not a second copy of it), plus
/// the by-`check` breakdown with seed lists.
fn print_violation_bucket(
    label: &str,
    results: &[GameResult],
    select: impl Fn(&GameResult) -> &[mtg_simulator::InvariantViolation],
) -> usize {
    let mut by_check: HashMap<&str, (usize, Vec<u64>)> = HashMap::new();
    let mut games_with = 0usize;
    let mut raw_total = 0usize;
    let mut all: Vec<mtg_simulator::InvariantViolation> = Vec::new();
    for result in results {
        let vs = select(result);
        if !vs.is_empty() {
            games_with += 1;
        }
        raw_total += vs.len();
        all.extend(vs.iter().cloned());
        for v in vs {
            let entry = by_check.entry(v.check.as_str()).or_insert((0, Vec::new()));
            entry.0 += 1;
            if entry.1.last() != Some(&result.seed) {
                entry.1.push(result.seed);
            }
        }
    }
    let distinct_total = mtg_simulator::invariants::distinct(&all).len();

    println!();
    println!(
        "  {label} (raw {raw_total} / distinct {distinct_total}), {games_with}/{} game(s)",
        results.len()
    );
    if by_check.is_empty() {
        println!("    (none)");
    } else {
        let mut rows: Vec<(&str, (usize, Vec<u64>))> = by_check.into_iter().collect();
        // Descending count, then name, so the output is stable across runs regardless of
        // HashMap iteration order — this text gets committed as evidence.
        rows.sort_by(|a, b| b.1 .0.cmp(&a.1 .0).then_with(|| a.0.cmp(b.0)));
        for (check, (count, mut seeds)) in rows {
            // Seeds too, not just counts: "which games did check X fire in" is the first
            // question a successor asks, and `--replay <SEED>` prints ALL of that game's
            // violations where this run prints the first five games' worth.
            seeds.sort_unstable();
            println!(
                "    {:<24} {:<6} in {} game(s): {:?}",
                check,
                count,
                seeds.len(),
                seeds
            );
        }
    }
    raw_total
}

/// Every violation in the run, grouped by `check` — **all games, not a sample**, split
/// into the HARD and TRANSIENT buckets `result.violations` /
/// `result.transient_violations` already carry (PB-DX32 Stage 4, `OOS-SIM3-3` /
/// `OOS-SIM3-4`). `--stop-on-error` and the crash-report writer key on the HARD bucket
/// only, so the TRANSIENT block below is diagnostic, not a halting signal.
///
/// # Why this exists (PB-DX22 fix cycle, review Finding 2)
///
/// The per-violation detail loop below prints only the first five offending games
/// (`if violation_seeds.len() <= 5`). PB-DX22 read a by-check breakdown off those printed
/// lines and then stated a universal negative about the whole run — "426 total violations,
/// and not one of them is `stack_consistency`" — from 94 of them. Every `GameResult` in
/// `results` carries its complete `violations`/`transient_violations` vectors, so the real
/// tally costs one fold and there is no reason to sample it. Anything that wants to say
/// "check X did not fire" must read this block, not the detail loop.
fn print_violation_histogram(results: &[GameResult]) {
    println!();
    println!("Violations by check (ALL {} games)", results.len());
    println!("---------------------------------");
    println!(
        "  Raw counts are CHECKPOINT-weighted (OOS-SIM3-3): the same underlying condition \
         can be reported once per checkpoint until the next SBA sweep clears it. Distinct \
         counts (deduped by (check, description), first occurrence wins) are the \
         defect-shaped number."
    );
    let hard_total = print_violation_bucket("HARD", results, |r| r.violations.as_slice());
    let transient_total = print_violation_bucket(
        "TRANSIENT (reported, does NOT halt --stop-on-error and does NOT write a crash \
         report -- known-transient class only, PB-DX32 Stage 4)",
        results,
        |r| r.transient_violations.as_slice(),
    );
    println!();
    println!(
        "  games with >=1 HARD violation: {} / {}",
        results.iter().filter(|r| !r.violations.is_empty()).count(),
        results.len()
    );
    // Printed so the block is self-checking: `hard_total` must equal the "Total
    // violations (HARD)" line in `main`'s summary above. If it does not, the histogram
    // is reading a different population than the summary and neither number should be
    // quoted.
    println!("  histogram total (HARD): {hard_total}  (TRANSIENT): {transient_total}");
}

/// The commander-mechanics and first-cast census over the whole run.
///
/// # Why this exists (PB-DX22 fix cycle, review Finding 1)
///
/// PB-DX22's headline A/B numbers were produced by a scratch `examples/dx22_p10.rs` that
/// was **deleted**, and no committed code could re-derive them: `grep
/// CommanderCastFromCommandZone crates/simulator/src` found three comments and zero code.
/// This block is the answer to that. Every number the batch published about commander
/// mechanics and cast depth is now printed by the instrument the batch is named for, over
/// every game in the run — so the "after" side of an A/B has the same standing as the
/// "before" side, and a successor re-derives it with one command instead of rewriting the
/// instrument.
///
/// CR 601.2, CR 305.1, CR 903.8, CR 903.9a, CR 903.9b, CR 903.10a.
fn print_mechanics_summary(tallies: &[MechanicsTally]) {
    fn band(values: &[u32]) -> String {
        if values.is_empty() {
            return "n/a".to_string();
        }
        let mut sorted = values.to_vec();
        sorted.sort_unstable();
        format!(
            "min {} / median {} / max {}",
            sorted[0],
            sorted[sorted.len() / 2],
            sorted[sorted.len() - 1]
        )
    }

    let games = tallies.len();
    let spell_casts: u64 = tallies.iter().map(|t| u64::from(t.spell_casts)).sum();
    let lands: u64 = tallies.iter().map(|t| u64::from(t.lands_played)).sum();
    let cmdr_casts: u64 = tallies
        .iter()
        .map(|t| u64::from(t.commander_casts_from_command_zone))
        .sum();
    let cmdr_returns: u64 = tallies
        .iter()
        .map(|t| u64::from(t.commander_returns_to_command_zone))
        .sum();
    let cmdr_redirects: u64 = tallies
        .iter()
        .map(|t| u64::from(t.commander_zone_redirects))
        .sum();

    let first_cast: Vec<u32> = tallies
        .iter()
        .filter_map(|t| t.first_spell_cast_turn)
        .collect();
    let first_lib_cast: Vec<u32> = tallies
        .iter()
        .filter_map(|t| t.first_library_spell_cast_turn)
        .collect();
    let first_land: Vec<u32> = tallies
        .iter()
        .filter_map(|t| t.first_land_played_turn)
        .collect();
    let first_cmdr_cast: Vec<u32> = tallies
        .iter()
        .filter_map(|t| t.first_commander_cast_turn)
        .collect();

    let games_with_cmdr_cast = tallies
        .iter()
        .filter(|t| t.commander_casts_from_command_zone > 0)
        .count();
    let games_with_cmdr_damage = tallies
        .iter()
        .filter(|t| t.seats_dealt_commander_damage > 0)
        .count();
    let max_cmdr_damage = tallies
        .iter()
        .map(|t| t.max_commander_damage)
        .max()
        .unwrap_or(0);

    println!();
    println!("Mechanics census (ALL {} games)", games);
    println!("------------------------------");
    println!(
        "  CR 601.2  SpellCast                    {} total; first on game turn {} ({} of {} games cast)",
        spell_casts,
        band(&first_cast),
        first_cast.len(),
        games
    );
    println!(
        "  CR 601.2  first NON-commander cast     game turn {} ({} of {} games) -- this is the one CR 103.3 library order gates",
        band(&first_lib_cast),
        first_lib_cast.len(),
        games
    );
    println!(
        "  CR 305.1  LandPlayed                   {} total; first on game turn {} ({} of {} games)",
        lands,
        band(&first_land),
        first_land.len(),
        games
    );
    println!(
        "  CR 903.8  CommanderCastFromCommandZone {} total, in {} of {} games; first on game turn {}",
        cmdr_casts,
        games_with_cmdr_cast,
        games,
        band(&first_cmdr_cast)
    );
    println!(
        "  CR 903.9a CommanderReturnedToCommandZone {}",
        cmdr_returns
    );
    println!(
        "  CR 903.9b CommanderZoneRedirect        {}",
        cmdr_redirects
    );
    println!(
        "  CR 903.10a commander damage            non-empty in {} of {} games; largest single total {} (rule threshold 21)",
        games_with_cmdr_damage, games, max_cmdr_damage
    );
}

/// SR-38 at run scale (PB-DX32 Stage 2, `OOS-SIM3-2`) — every `GameResult` in `results`
/// carries `rejection_count` (uncapped) and a bounded `rejections` diagnosis sample
/// (`report::MAX_SAMPLED_REJECTIONS` per game with the journal off, which is this
/// binary's own configuration). Prints total rejections, total commands, the aggregate
/// per-mille against [`mtg_simulator::MAX_BOT_REJECTION_PER_MILLE`], the per-seed band
/// (seeds with >=1 rejection only, to keep this readable at `--games 1000`), and the top
/// rejection classes by error-string prefix — truncated at the first `(` so e.g.
/// `InvalidTarget("expected 1..=1 target(s) but got 0")` and `InvalidTarget("modal spell
/// with per-mode targets requires exactly 1 target(s) for ..")` group under one
/// `InvalidTarget` row. The class breakdown is read from the SAMPLE
/// (`GameResult::rejections`), not the full count, and says so.
///
/// Rows sorted descending by count then by name, so the output is stable across runs —
/// this text gets committed as evidence, mirroring `print_violation_histogram`.
///
/// Returns whether the aggregate rate breached [`mtg_simulator::MAX_BOT_REJECTION_PER_MILLE`],
/// so `main` can fail the run loudly (F19: this binary is not run in CI).
fn print_sr38_summary(results: &[GameResult]) -> bool {
    let total_commands: u64 = results.iter().map(|r| r.total_commands as u64).sum();
    let total_rejections: u64 = results.iter().map(|r| u64::from(r.rejection_count)).sum();
    let per_mille = if total_commands == 0 {
        0.0
    } else {
        (total_rejections as f64 / total_commands as f64) * 1000.0
    };

    let mut per_seed: Vec<(u64, u32, usize)> = results
        .iter()
        .map(|r| (r.seed, r.rejection_count, r.total_commands))
        .collect();
    per_seed.sort_unstable_by_key(|(seed, ..)| *seed);

    let mut by_class: HashMap<String, usize> = HashMap::new();
    for result in results {
        for rejection in &result.rejections {
            // Every recorded rejection is `LocalGameError::Rejected(GameStateError)`
            // (the only variant `record_rejection`'s call site ever sees — see
            // `apply_sequence`), so the Debug string is always wrapped as
            // `Rejected(<GameStateError Debug>)`. Strip that wrapper first so the
            // class is the GameStateError variant (`InvalidTarget`, `InsufficientMana`,
            // ...), which is what a reader actually wants grouped, not the constant
            // outer wrapper every rejection shares.
            let inner = rejection
                .error
                .strip_prefix("Rejected(")
                .unwrap_or(&rejection.error);
            let class = inner
                .split_once('(')
                .map(|(prefix, _)| prefix.trim().to_string())
                .unwrap_or_else(|| inner.trim_end_matches(')').trim().to_string());
            *by_class.entry(class).or_insert(0) += 1;
        }
    }

    println!();
    println!(
        "SR-38: bot-seat command rejections (ALL {} games)",
        results.len()
    );
    println!("---------------------------------------------------");
    println!(
        "  {} rejections / {} commands = {:.3} per mille (threshold {})",
        total_rejections,
        total_commands,
        per_mille,
        mtg_simulator::MAX_BOT_REJECTION_PER_MILLE
    );
    let seeds_with_rejections: Vec<&(u64, u32, usize)> =
        per_seed.iter().filter(|(_, n, _)| *n > 0).collect();
    if seeds_with_rejections.is_empty() {
        println!("  per-seed band: (no rejections in any game)");
    } else {
        println!("  per-seed band (seed: rejections/commands):");
        for (seed, rejections, commands) in seeds_with_rejections {
            println!("    seed {seed}: {rejections}/{commands}");
        }
    }
    if by_class.is_empty() {
        println!("  top rejection classes (from the sample): (none sampled)");
    } else {
        let mut rows: Vec<(String, usize)> = by_class.into_iter().collect();
        rows.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        println!("  top rejection classes (from the sample, NOT the full count):");
        for (class, count) in rows {
            println!("    {:<6} {}", count, class);
        }
    }

    let breached = per_mille > f64::from(mtg_simulator::MAX_BOT_REJECTION_PER_MILLE);
    if breached {
        println!(
            "  SR-38 THRESHOLD EXCEEDED: {:.3} per mille > {} (MAX_BOT_REJECTION_PER_MILLE)",
            per_mille,
            mtg_simulator::MAX_BOT_REJECTION_PER_MILLE
        );
    }
    breached
}

/// The promoted SIM-5 tap/pool instrument (PB-DX32 Stage 3) — every `GameResult` in
/// `results` now carries a [`mtg_simulator::WasteTally`]. Prints total taps, wasted
/// taps and their percentage, `ManaPoolsEmptied` (CR 500.4), casts and targeted casts.
///
/// **One sentence naming the bot, deliberately** (plan §5 Stage 3 step 3): a
/// `RandomBot` wasted-tap percentage and a `HeuristicBot` one are NOT comparable —
/// `RandomBot` picks `TapForMana` uniformly with no plan, so a value near
/// [`mtg_simulator::MAX_RANDOM_BOT_WASTED_TAP_PCT`] is ordinary behaviour for it and
/// would be a real regression for `HeuristicBot`.
fn print_waste_summary(results: &[GameResult], bot: &BotType) {
    let mut t = mtg_simulator::WasteTally::default();
    for result in results {
        t.tap_runs = t.tap_runs.saturating_add(result.waste.tap_runs);
        t.wasted_tap_runs = t
            .wasted_tap_runs
            .saturating_add(result.waste.wasted_tap_runs);
        t.wasted_taps = t.wasted_taps.saturating_add(result.waste.wasted_taps);
        t.total_taps = t.total_taps.saturating_add(result.waste.total_taps);
        t.mana_pools_emptied = t
            .mana_pools_emptied
            .saturating_add(result.waste.mana_pools_emptied);
        t.casts = t.casts.saturating_add(result.waste.casts);
        t.targeted_casts = t.targeted_casts.saturating_add(result.waste.targeted_casts);
    }

    let wasted_pct = if t.total_taps == 0 {
        0.0
    } else {
        (f64::from(t.wasted_taps) / f64::from(t.total_taps)) * 100.0
    };

    println!();
    println!(
        "Waste census (ALL {} games) -- bot: {:?}",
        results.len(),
        bot
    );
    println!("-----------------------------------------");
    println!(
        "  {:?} wastes taps BY DESIGN (no plan) -- do not read a high percentage here as an \
         engine defect for this bot.",
        bot
    );
    println!(
        "  tap runs: {} total, {} wasted ({} taps of {} total = {:.1}%, threshold {}%)",
        t.tap_runs,
        t.wasted_tap_runs,
        t.wasted_taps,
        t.total_taps,
        wasted_pct,
        mtg_simulator::MAX_RANDOM_BOT_WASTED_TAP_PCT
    );
    println!("  CR 500.4 ManaPoolsEmptied: {}", t.mana_pools_emptied);
    println!(
        "  casts: {} total, {} with >=1 announced target (CR 601.2c)",
        t.casts, t.targeted_casts
    );
}

fn print_game_result(result: &GameResult, verbose: bool) {
    let status = if let Some(winner) = result.winner {
        format!("Winner: P{}", winner.0)
    } else if let Some(ref err) = result.error {
        format!("Error: {:?}", err)
    } else {
        "Draw".to_string()
    };

    println!(
        "  Seed: {}  Turns: {}  Commands: {}  Violations: {}  {}",
        result.seed,
        result.turn_count,
        result.total_commands,
        result.violations.len(),
        status,
    );

    if verbose {
        for v in &result.violations {
            println!(
                "    [{}] {} (turn {})",
                v.check, v.description, v.turn_number
            );
        }
    }
}
