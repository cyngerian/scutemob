//! SIM-5 (G5 of `memory/playtest-triage-2026-08-02b.md`) — bot cast discipline.
//!
//! The triage measured, from a live 4-bot browser game, that **26 of 72 bot taps
//! (36%) were wasted**: 18 of 38 consecutive tap runs were followed immediately by
//! that same player's `PassPriority` with no cast in between, and the engine emitted
//! exactly 18 `ManaPoolsEmptied` events (CR 500.4) on exactly those turns. The
//! mechanism: `LocalGame::advance()` applied the bot's `[taps…, cast]` plan **one
//! command at a time**, so a rejected cast left the taps committed; and the cast was
//! rejected because bots announced **zero targets, always**.
//!
//! This file is both the A/B measurement instrument for that finding and the
//! regression gate for its fix. `measure()` walks the journal of a seeded bot-only
//! game exactly the way the triage walked `GET /api/game/report`.

use std::collections::{BTreeSet, HashMap};

use mtg_engine::{Command, GameEvent, ObjectId, PlayerId, Target};
use mtg_simulator::bot::Bot;
use mtg_simulator::heuristic_bot::HeuristicBot;
use mtg_simulator::legal_actions::StubProvider;
use mtg_simulator::local_game::{AdvanceOutcome, LocalGame, LocalGameLimits};
use mtg_simulator::random_bot::RandomBot;
use mtg_simulator::setup::{self, BotKind, DeckSource, LocalGameConfig};

/// The seeds the A/B measurement is reported on, and how far each is played.
///
/// 25 turns because the triage's live game was 29 turns deep when it was read; the
/// three seeds are arbitrary but fixed, so the before/after tables in the handoff
/// compare the same games.
const AB_SEEDS: [u64; 3] = [0, 7, 42];
const AB_MAX_TURNS: u32 = 25;

/// The counts the triage reported, recomputed from a journal.
#[derive(Debug, Default, Clone)]
pub struct Metrics {
    /// Consecutive-`TapForMana`-by-one-player runs in the journal.
    pub tap_runs: usize,
    /// Runs followed immediately by that same player's `PassPriority` — the triage's
    /// "wasted" classification.
    pub wasted_tap_runs: usize,
    /// Taps inside those wasted runs.
    pub wasted_taps: usize,
    pub total_taps: usize,
    /// CR 500.4 — emitted only when at least one pool was actually non-empty
    /// (`turn_actions.rs:1388`), so this is a count of destroyed floating mana.
    pub mana_pools_emptied: usize,
    pub casts: usize,
    /// Casts that announced at least one target (CR 601.2c).
    pub targeted_casts: usize,
    pub turns: u32,
    pub commands: usize,
}

/// Names of the cards a bot cast with at least one announced target, in order.
///
/// `names` is built from the **pregame** state, not the final one: a spell that has
/// resolved is a new object (CR 400.7) and its cast-time `ObjectId` is gone from
/// `state.objects()` by the end of the game. Every card that can be cast exists at
/// setup (in a hand or a library), so the pregame index covers all of them.
pub fn targeted_cast_names(
    game: &LocalGame<StubProvider>,
    names: &HashMap<ObjectId, String>,
) -> Vec<String> {
    let mut out = Vec::new();
    for record in game.journal() {
        if let Command::CastSpell(cast) = &record.command {
            if !cast.targets.is_empty() {
                let label = names
                    .get(&cast.card)
                    .cloned()
                    .unwrap_or_else(|| format!("{:?}", cast.card));
                out.push(format!(
                    "T{} {label} -> {:?}",
                    record.turn,
                    cast.targets
                        .iter()
                        .map(|t| match t {
                            Target::Object(id) =>
                                names.get(id).cloned().unwrap_or_else(|| format!("{id:?}")),
                            Target::Player(p) => format!("player {}", p.0),
                        })
                        .collect::<Vec<_>>()
                ));
            }
        }
    }
    out
}

/// For every `ManaPoolsEmptied` (CR 500.4), the handful of commands that preceded it.
///
/// This is the residual-explaining instrument: after SIM-5 a destroyed pool should
/// never be a *wasted* one (a tap run followed by its own player's pass), but it can
/// still be greedy-solver slack on a cast that actually happened (`OOS-SIM2-1`).
pub fn emptied_pool_context(game: &LocalGame<StubProvider>) -> Vec<String> {
    let journal = game.journal();
    journal
        .iter()
        .enumerate()
        .filter(|(_, r)| r.events.iter().any(|e| matches!(e, GameEvent::ManaPoolsEmptied)))
        .map(|(i, r)| {
            let from = i.saturating_sub(12);
            let preceding: Vec<String> = journal[from..i]
                .iter()
                .map(|p| short_command(&p.command))
                .collect();
            format!("T{} [{}] <- {}", r.turn, preceding.join(", "), short_command(&r.command))
        })
        .collect()
}

fn short_command(command: &Command) -> String {
    match command {
        Command::TapForMana { player, .. } => format!("tap(p{})", player.0),
        Command::PassPriority { player } => format!("pass(p{})", player.0),
        Command::CastSpell(c) => format!("cast(p{}, {} targets)", c.player.0, c.targets.len()),
        Command::PlayLand { player, .. } => format!("land(p{})", player.0),
        Command::ActivateAbility { player, .. } => format!("activate(p{})", player.0),
        other => format!("{:?}", std::mem::discriminant(other)),
    }
}

pub fn metrics_of(game: &LocalGame<StubProvider>) -> Metrics {
    let mut m = Metrics {
        turns: game.state().turn().turn_number,
        commands: game.journal().len(),
        ..Metrics::default()
    };

    // One pass, grouping maximal consecutive `TapForMana` runs by one player and
    // classifying a run by whatever command follows it.
    let mut run: Option<(PlayerId, usize)> = None;
    for record in game.journal() {
        m.mana_pools_emptied += record
            .events
            .iter()
            .filter(|e| matches!(e, GameEvent::ManaPoolsEmptied))
            .count();

        match &record.command {
            Command::TapForMana { player, .. } => {
                m.total_taps += 1;
                match &mut run {
                    Some((p, n)) if *p == *player => *n += 1,
                    Some(_) => {
                        // A different player interleaved: close the old run unclassified.
                        m.tap_runs += 1;
                        run = Some((*player, 1));
                    }
                    None => run = Some((*player, 1)),
                }
            }
            Command::PassPriority { player } => {
                if let Some((p, n)) = run.take() {
                    m.tap_runs += 1;
                    if p == *player {
                        m.wasted_tap_runs += 1;
                        m.wasted_taps += n;
                    }
                }
            }
            other => {
                if let Command::CastSpell(cast) = other {
                    m.casts += 1;
                    if !cast.targets.is_empty() {
                        m.targeted_casts += 1;
                    }
                }
                if run.take().is_some() {
                    m.tap_runs += 1;
                }
            }
        }
    }
    if run.take().is_some() {
        m.tap_runs += 1;
    }
    m
}

fn bots_for(cfg: &LocalGameConfig) -> HashMap<PlayerId, Box<dyn Bot>> {
    // Seeded exactly as `session.rs::bots_for` and `mtg-fuzzer` seed theirs.
    let mut bots: HashMap<PlayerId, Box<dyn Bot>> = HashMap::new();
    for i in 1..=u64::from(cfg.player_count) {
        let pid = PlayerId(i);
        let bot_seed = cfg.seed.wrapping_add(100 + i);
        let name = format!("Bot-{i}");
        let bot: Box<dyn Bot> = match cfg.bot_kind {
            BotKind::Heuristic => Box::new(HeuristicBot::new(bot_seed, name)),
            BotKind::Random => Box::new(RandomBot::new(bot_seed, name)),
        };
        bots.insert(pid, bot);
    }
    bots
}

/// Play a seeded four-bot game (no human seat) for at most `max_turns` and return
/// the finished game (so the caller can read `journal()` / `state()`) together with a
/// pregame `ObjectId -> name` index — see [`targeted_cast_names`] for why the index
/// must be taken before play, not after.
pub fn play(
    seed: u64,
    bot_kind: BotKind,
    max_turns: u32,
) -> (LocalGame<StubProvider>, HashMap<ObjectId, String>) {
    let cfg = LocalGameConfig {
        player_count: 4,
        human_seats: BTreeSet::new(),
        bot_kind,
        seed,
        decks: DeckSource::RandomPerSeat,
        limits: LocalGameLimits {
            max_turns,
            max_commands: max_turns * 800,
            max_consecutive_passes: 500,
            // The whole instrument: the journal IS the measurement.
            record_journal: true,
        },
    };
    let (state, _seat_names) = setup::build_initial_state(&cfg).expect("seeded setup must build");
    let card_names: HashMap<ObjectId, String> = state
        .objects()
        .iter()
        .map(|(id, o)| (*id, o.characteristics.name.clone()))
        .collect();
    let (mut game, _events) = LocalGame::start(
        state,
        cfg.seed,
        StubProvider,
        bots_for(&cfg),
        cfg.human_seats.clone(),
        cfg.limits,
        false,
    )
    .expect("seeded game must start");

    loop {
        match game.advance() {
            AdvanceOutcome::AwaitingHuman(_) => unreachable!("no human seats in this fixture"),
            AdvanceOutcome::GameOver(_) | AdvanceOutcome::Halted(_) => break,
        }
    }
    (game, card_names)
}

/// **The SIM-5 A/B gate.** A bot's `[taps…, cast]` plan is atomic (`advance()` →
/// `apply_sequence`), so a rejected cast can no longer leave taps committed: no tap
/// run is followed by its own player's `PassPriority`.
///
/// Before the fix this seed produced wasted runs (recorded in the task handoff);
/// the triage measured the same shape live at 18/38 runs and 26/72 taps.
#[test]
fn seeded_four_bot_game_wastes_no_taps() {
    // Every seed is measured and printed before anything is asserted, so a failure
    // report carries the whole A/B table rather than only the first bad seed.
    let measured: Vec<(u64, Metrics)> = AB_SEEDS
        .iter()
        .map(|&seed| {
            let (game, names) = play(seed, BotKind::Heuristic, AB_MAX_TURNS);
            let m = metrics_of(&game);
            eprintln!("SIM-5 A/B seed {seed}: {m:?}");
            eprintln!("  rejections: {} retained/{} total", game.rejections().len(), game.rejection_count());
            let mut classes: std::collections::BTreeMap<String, usize> =
                std::collections::BTreeMap::new();
            for r in game.rejections() {
                let card = match &r.command {
                    Command::CastSpell(c) => names
                        .get(&c.card)
                        .cloned()
                        .unwrap_or_else(|| format!("{:?}", c.card)),
                    other => short_command(other),
                };
                *classes.entry(format!("{card}: {}", r.error)).or_default() += 1;
            }
            for (class, n) in &classes {
                eprintln!("    x{n} {class}");
            }
            for line in targeted_cast_names(&game, &names) {
                eprintln!("  targeted cast: {line}");
            }
            for line in emptied_pool_context(&game) {
                eprintln!("  pools emptied: {line}");
            }
            (seed, m)
        })
        .collect();

    for (seed, m) in &measured {
        assert_eq!(
            m.wasted_tap_runs, 0,
            "seed {seed}: a tap run was followed by its own player's PassPriority — \
             the bot committed taps for a cast that never happened (G5): {m:?}"
        );
        assert_eq!(m.wasted_taps, 0, "seed {seed}: {m:?}");
    }

    // Non-vacuity floor: a run in which no bot ever tapped would satisfy the two
    // assertions above trivially.
    let total_taps: usize = measured.iter().map(|(_, m)| m.total_taps).sum();
    assert!(
        total_taps > 0,
        "non-vacuity floor: the seeds must actually produce bot taps"
    );
}
