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

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Instant;

use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;

use mtg_engine::{
    all_cards, enrich_spec_from_def, CardDefinition, CardId, CardRegistry, GameStateBuilder,
    ObjectSpec, PlayerId, ZoneId,
};
use mtg_simulator::{
    build_registry, random_deck, CrashReport, DeckConfig, GameDriver, GameDriverError, GameResult,
    HeuristicBot, RandomBot, StubProvider,
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

    let base_seed = cli.seed.unwrap_or_else(|| rand::thread_rng().gen());
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
        let result = run_single_game(
            replay_seed,
            cli.players,
            cli.max_turns,
            &cli.bot,
            &cards,
            &registry,
        );
        print_game_result(&result, true);
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

    let results: Vec<GameResult> = (0..cli.games)
        .into_par_iter()
        .filter_map(|i| {
            if cli.stop_on_error && should_stop.load(Ordering::Relaxed) {
                return None;
            }

            let game_seed = base_seed.wrapping_add(i as u64);
            let result = run_single_game(
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

            Some(result)
        })
        .collect();

    pb.finish_with_message("done");
    let elapsed = start.elapsed();

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
    let avg_turns: f64 =
        results.iter().map(|r| r.turn_count as f64).sum::<f64>() / results.len().max(1) as f64;

    println!("Wins: {}  Draws: {}  Errors: {}", wins, draws, errors);
    println!("Total violations: {}", total_violations);
    println!("Avg turns per game: {:.1}", avg_turns);

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
}

fn run_single_game(
    seed: u64,
    player_count: u32,
    max_turns: u32,
    bot_type: &BotType,
    cards: &[CardDefinition],
    registry: &Arc<CardRegistry>,
) -> GameResult {
    let mut rng = StdRng::seed_from_u64(seed);

    // Build random decks for each player
    let player_ids: Vec<PlayerId> = (1..=player_count).map(|i| PlayerId(i as u64)).collect();

    let mut decks: Vec<(PlayerId, DeckConfig)> = Vec::new();
    for &pid in &player_ids {
        if let Some(deck) = random_deck(&mut rng, cards) {
            decks.push((pid, deck));
        } else {
            // Fallback: just basic lands
            let fallback = DeckConfig {
                commander: CardId("teysa-karlov".to_string()),
                main_deck: (0..99).map(|_| CardId("plains".to_string())).collect(),
            };
            decks.push((pid, fallback));
        }
    }

    // Build initial state using GameStateBuilder, populating libraries from decks
    let mut builder = GameStateBuilder::new().with_registry(registry.clone());

    for &pid in &player_ids {
        builder = builder.add_player(pid);
    }

    // Build a name→def lookup for enriching card specs
    let card_defs: HashMap<String, CardDefinition> =
        cards.iter().map(|c| (c.name.clone(), c.clone())).collect();

    // Add library cards from decks
    for (pid, deck) in &decks {
        // Add commander to command zone
        if let Some(def) = cards.iter().find(|c| c.card_id == deck.commander) {
            let spec = ObjectSpec::card(*pid, &def.name)
                .in_zone(ZoneId::Command(*pid))
                .with_card_id(deck.commander.clone());
            let spec = enrich_spec_from_def(spec, &card_defs);
            builder = builder.object(spec);
        }

        // Add main deck cards to library
        for card_id in &deck.main_deck {
            if let Some(def) = cards.iter().find(|c| c.card_id == *card_id) {
                let spec = ObjectSpec::card(*pid, &def.name)
                    .in_zone(ZoneId::Library(*pid))
                    .with_card_id(card_id.clone());
                let spec = enrich_spec_from_def(spec, &card_defs);
                builder = builder.object(spec);
            }
        }
    }

    builder = builder.first_turn_of_game();

    let state = match builder.build() {
        Ok(s) => s,
        Err(e) => {
            return GameResult {
                seed,
                winner: None,
                turn_count: 0,
                total_commands: 0,
                violations: Vec::new(),
                error: Some(GameDriverError::EngineError(format!(
                    "Failed to build state: {:?}",
                    e
                ))),
            };
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
    let mut driver = GameDriver::new(StubProvider, bots, max_turns, seed);
    driver.run_game(state, seed)
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
