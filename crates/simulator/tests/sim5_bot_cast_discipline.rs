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

use mtg_engine::{
    all_cards, enrich_spec_from_def, process_command, CardDefinition, CardId, CardRegistry, Color,
    Command, GameEvent, GameState, GameStateBuilder, ManaAbility, ManaColor, ObjectId, ObjectSpec,
    PlayerId, Step, Target, ZoneId,
};
use mtg_simulator::bot::Bot;
use mtg_simulator::heuristic_bot::HeuristicBot;
use mtg_simulator::legal_actions::{LegalAction, LegalActionProvider, StubProvider};
use mtg_simulator::local_game::{AdvanceOutcome, LocalGame, LocalGameLimits};
use mtg_simulator::params::{action_to_command_with_params, ActionParams, HumanChoice};
use mtg_simulator::random_bot::RandomBot;
use mtg_simulator::setup::{self, BotKind, DeckSource, LocalGameConfig};
use mtg_simulator::targeting::{plan_targets, TargetPlan};

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

/// For every `ManaPoolsEmptied` (CR 500.4), the commands that preceded it.
///
/// This is the residual-explaining instrument: after SIM-5 a destroyed pool should
/// never be a *wasted* one (a tap run followed by its own player's pass), but it can
/// still be greedy-solver slack on a cast that actually happened (`OOS-SIM2-1`).
///
/// The window is 40 rather than a handful because the one residual left on the A/B
/// seeds needed it: seed 7's T14 pool was funded by a four-tap run **twenty-odd
/// commands earlier**, whose cast succeeded, with the remainder partly spent on a
/// second cast and the rest destroyed at the step boundary. A five-command window
/// showed only passes and would have made that residual look unexplained.
pub fn emptied_pool_context(game: &LocalGame<StubProvider>) -> Vec<String> {
    let journal = game.journal();
    journal
        .iter()
        .enumerate()
        .filter(|(_, r)| {
            r.events
                .iter()
                .any(|e| matches!(e, GameEvent::ManaPoolsEmptied))
        })
        .map(|(i, r)| {
            let from = i.saturating_sub(40);
            let preceding: Vec<String> = journal[from..i]
                .iter()
                .map(|p| short_command(&p.command))
                .collect();
            format!(
                "T{} [{}] <- {}",
                r.turn,
                preceding.join(", "),
                short_command(&r.command)
            )
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

    // One call: with no human seat `advance()` only returns on `GameOver` or `Halted`,
    // so the game is played to its conclusion (or to a safety valve) right here.
    match game.advance() {
        AdvanceOutcome::AwaitingHuman(_) => unreachable!("no human seats in this fixture"),
        AdvanceOutcome::GameOver(_) | AdvanceOutcome::Halted(_) => {}
    }
    (game, card_names)
}

/// **The SIM-5 A/B gate.** A bot's `[taps…, cast]` plan is atomic (`advance()` →
/// `apply_sequence`), so a rejected cast can no longer leave taps committed: no tap
/// run is followed by its own player's `PassPriority`.
///
/// Before the fix these seeds produced 30 wasted runs / 45 wasted taps (recorded in
/// the task handoff); the triage measured the same shape live at 18/38 runs and 26/72
/// taps.
///
/// # This is a whole-game measurement, NOT the primary atomicity gate
///
/// Measured during the SIM-5 review: with targeting kept and *only* the
/// `apply_sequence` call reverted, just seed 42 goes red here (1 wasted run) — because
/// fix (2) removed nearly every cast-side refusal, so there is little left for fix (1)
/// to roll back. [`a_rejected_bot_cast_commits_no_taps`] carries the real regression
/// load for atomicity: it freezes a zero-target bot into the fixture so it keeps
/// failing on a reverted `apply_sequence` no matter how good targeting gets. If this
/// seed list is ever re-picked, do not treat this test as the atomicity gate.
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
            eprintln!(
                "  rejections: {} retained/{} total",
                game.rejections().len(),
                game.rejection_count()
            );
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

    // CR 601.2c (SIM-5 fix (2)): bots announce targets now. This was 0 on every seed
    // before the fix -- `random_bot::action_to_command` filled only
    // `attackers`/`blockers`, so no bot had ever cast a targeted spell in the history
    // of this simulator.
    let targeted: usize = measured.iter().map(|(_, m)| m.targeted_casts).sum();
    assert!(
        targeted > 0,
        "bots must be able to cast targeted spells: {measured:?}"
    );
}

/// **T3.2** (PB-DX32 Stage 3) — the anti-drift gate for the `WasteTally` promotion.
/// `LocalGame::waste()` is folded incrementally at the same two sites
/// `MechanicsTally::record` is folded at (plan §3.4, fact F8); this proves the
/// streaming fold and this file's own journal-walk ([`metrics_of`]) are the SAME
/// measurement on a journal-ON game, field for field, across all seven counters. This
/// is what stops the promoted copy from silently drifting from its origin.
///
/// # The trailing open-run close (plan §7 R8) needed a SECOND, purpose-built case
///
/// None of `AB_SEEDS` ever exercises `waste()`'s trailing "close the still-open run"
/// step (`local_game.rs::waste`, mirroring `metrics_of`'s own `:196-198`) — verified
/// by executing the revert (dropping that close) against this test's AB_SEEDS loop
/// alone: it stayed GREEN. The reason is structural, not a missing seed:
/// `HeuristicBot` scores `LegalAction::TapForMana` at 0, "below passing"
/// (`heuristic_bot.rs:271`), so it only ever taps as an auto-tap PREFIX bundled
/// with a cast (`advance()`'s `[taps…, cast]` vector) — and a whole `[taps…, cast]`
/// bundle is folded, and its own run therefore CLOSED, inside the single
/// `apply_sequence` call that commits it, before that call ever returns to the caller.
/// A run can only survive PAST one `advance()` iteration if that iteration's WHOLE
/// decision was a STANDALONE tap with nothing queued after it — which `HeuristicBot`
/// structurally never chooses. So the block below drives a controlled fixture instead:
/// a human seat submits exactly one `TapForMana` command and nothing follows, which
/// is the only way to leave `waste_run` open at the moment `.waste()` is called.
#[test]
fn test_dx32_streaming_waste_tally_equals_the_sim5_journal_walk() {
    for &seed in &AB_SEEDS {
        let (game, _names) = play(seed, BotKind::Heuristic, AB_MAX_TURNS);
        let walked = metrics_of(&game);
        let streamed = game.waste();
        assert_eq!(
            streamed.tap_runs as usize, walked.tap_runs,
            "seed {seed}: tap_runs — streamed {streamed:?} vs walked {walked:?}"
        );
        assert_eq!(
            streamed.wasted_tap_runs as usize, walked.wasted_tap_runs,
            "seed {seed}: wasted_tap_runs — streamed {streamed:?} vs walked {walked:?}"
        );
        assert_eq!(
            streamed.wasted_taps as usize, walked.wasted_taps,
            "seed {seed}: wasted_taps — streamed {streamed:?} vs walked {walked:?}"
        );
        assert_eq!(
            streamed.total_taps as usize, walked.total_taps,
            "seed {seed}: total_taps — streamed {streamed:?} vs walked {walked:?}"
        );
        assert_eq!(
            streamed.mana_pools_emptied as usize, walked.mana_pools_emptied,
            "seed {seed}: mana_pools_emptied — streamed {streamed:?} vs walked {walked:?}"
        );
        assert_eq!(
            streamed.casts as usize, walked.casts,
            "seed {seed}: casts — streamed {streamed:?} vs walked {walked:?}"
        );
        assert_eq!(
            streamed.targeted_casts as usize, walked.targeted_casts,
            "seed {seed}: targeted_casts — streamed {streamed:?} vs walked {walked:?}"
        );
    }

    // The controlled mid-tap-run case (see this test's doc comment for why AB_SEEDS
    // alone cannot reach it): a human seat with one untapped mana-producing land taps
    // it and submit() returns -- nothing else has been applied, so the run is open.
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(
            ObjectSpec::land(p1, "Swamp 1")
                .with_mana_ability(ManaAbility::tap_for(ManaColor::Black)),
        )
        .build()
        .expect("mid-tap-run fixture must build");
    state.turn_mut().priority_holder = Some(p1);

    let human: BTreeSet<PlayerId> = [p1].into_iter().collect();
    let (mut mid_run_game, _events) = LocalGame::start(
        state,
        0,
        StubProvider,
        HashMap::new(),
        human,
        LocalGameLimits {
            max_turns: 3,
            max_commands: 400,
            max_consecutive_passes: 100,
            record_journal: true,
        },
        false,
    )
    .expect("mid-tap-run fixture must start");

    let decision = match mid_run_game.advance() {
        AdvanceOutcome::AwaitingHuman(d) => d,
        other => panic!("expected p1 to hold priority: {other:?}"),
    };
    let index = decision
        .actions
        .iter()
        .position(|a| matches!(a, LegalAction::TapForMana { .. }))
        .expect("the untapped Swamp must offer a TapForMana action");
    mid_run_game
        .submit(
            decision.seq,
            HumanChoice {
                action_index: index,
                params: ActionParams::default(),
            },
        )
        .expect("tapping the fixture land must succeed");

    let mid_run_waste = mid_run_game.waste();
    assert_eq!(
        mid_run_waste.total_taps, 1,
        "the tap itself must always be counted regardless of run-closing: {mid_run_waste:?}"
    );
    assert_eq!(
        mid_run_waste.tap_runs, 1,
        "the still-open run must be closed on the snapshot COPY `waste()` returns \
         (plan §3.4 / T3.2, the R8 case this AB_SEEDS loop above cannot reach): \
         {mid_run_waste:?}"
    );
}

/// **T3.3** (PB-DX32 Stage 3) — criterion (b)'s literal requirement: `OOS-SIM2-1`
/// named IN the assertion message, at the pin. Reuses the same A/B seeds
/// (`AB_SEEDS`, `AB_MAX_TURNS`, `HeuristicBot`) as
/// [`seeded_four_bot_game_wastes_no_taps`] above.
#[test]
fn heuristic_pools_emptied_is_pinned() {
    for &seed in &AB_SEEDS {
        let (game, _names) = play(seed, BotKind::Heuristic, AB_MAX_TURNS);
        let m = metrics_of(&game);
        assert!(
            m.mana_pools_emptied <= mtg_simulator::MAX_HEURISTIC_POOLS_EMPTIED_PER_SEED,
            "seed {seed}: mana_pools_emptied {} exceeds \
             MAX_HEURISTIC_POOLS_EMPTIED_PER_SEED ({}) — a rise past this pin means \
             either a new wasted-tap class or that the greedy-solver slack OOS-SIM2-1 \
             leaves on casts that SUCCEED has widened: {m:?}",
            m.mana_pools_emptied,
            mtg_simulator::MAX_HEURISTIC_POOLS_EMPTIED_PER_SEED
        );
    }
}

// ── Focused gates ────────────────────────────────────────────────────────────────
//
// The A/B test above is a whole-game measurement; these three are fixtures small
// enough that a reviewer can see what is being pinned.

/// Registry + name index holding exactly the one real card def these fixtures need.
///
/// A `TargetRequirement` comes from the card DEFINITION
/// (`casting::card_def_target_requirements`), never from an `ObjectSpec`, so a
/// synthetic object cannot exercise targeting at all — the fixture must register a
/// real def. `Doom Blade` is the minimal choice: one mandatory
/// `TargetCreatureWithFilter { exclude_colors: {Black} }` (CR 601.2c), so a board can
/// be built where exactly one of two creatures is a legal target.
fn doom_blade_registry() -> (
    std::sync::Arc<CardRegistry>,
    HashMap<String, CardDefinition>,
) {
    let def = all_cards()
        .into_iter()
        .find(|c| c.card_id == CardId("doom-blade".to_string()))
        .expect("doom-blade must be in the card pool");
    let defs: HashMap<String, CardDefinition> =
        [(def.name.clone(), def.clone())].into_iter().collect();
    (CardRegistry::new(vec![def]), defs)
}

/// `p1` holds Doom Blade with two black sources untapped; `creatures` are put on the
/// battlefield under `p2`.
fn doom_blade_state(creatures: Vec<ObjectSpec>) -> GameState {
    let (registry, defs) = doom_blade_registry();
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);
    let mut builder = GameStateBuilder::new()
        .with_registry(registry)
        .add_player(p1)
        .add_player(p2)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(enrich_spec_from_def(
            ObjectSpec::card(p1, "Doom Blade")
                .with_card_id(CardId("doom-blade".to_string()))
                .in_zone(ZoneId::Hand(p1)),
            &defs,
        ))
        .object(
            ObjectSpec::land(p1, "Swamp 1")
                .with_mana_ability(ManaAbility::tap_for(ManaColor::Black)),
        )
        .object(
            ObjectSpec::land(p1, "Swamp 2")
                .with_mana_ability(ManaAbility::tap_for(ManaColor::Black)),
        );
    for c in creatures {
        builder = builder.object(c);
    }
    let mut state = builder.build().expect("fixture must build");
    state.turn_mut().priority_holder = Some(p1);
    state
}

fn find_by_name(state: &GameState, name: &str) -> ObjectId {
    state
        .objects()
        .iter()
        .find(|(_, o)| o.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("no object named {name:?} in this fixture"))
}

fn cast_action(state: &GameState, card: ObjectId) -> LegalAction {
    StubProvider
        .legal_actions(state, PlayerId(1))
        .into_iter()
        .find(|a| matches!(a, LegalAction::CastSpell { card: c, .. } if *c == card))
        .expect("the provider must offer the cast for this fixture to mean anything")
}

/// **SIM-5 fix (2), CR 601.2c.** A bot announces a target, and it announces a *legal*
/// one: the board holds a black creature (which Doom Blade may not target) and a
/// colourless one (which it may), and the bot must pick the second — which is only
/// possible by asking the engine, since both are "the first creature" by some
/// ordering. The command is then handed to `process_command`, so the gate is
/// "the engine accepts it", not "it looks right".
///
/// Before SIM-5 this produced `targets: []` and `process_command` answered
/// `InvalidTarget("expected 1..=1 target(s) but got 0")` (`casting.rs:5931`) — the
/// structural G5 mechanism.
#[test]
fn bot_announces_a_legal_target_and_the_engine_accepts_the_cast() {
    let p2 = PlayerId(2);
    let state = doom_blade_state(vec![
        // Lower `ObjectId` than the legal one, and illegal: a "first creature on the
        // battlefield" policy would pick this and be refused.
        ObjectSpec::creature(p2, "Black Bear", 2, 2).with_colors(vec![Color::Black]),
        ObjectSpec::creature(p2, "Grey Ogre", 2, 2),
    ]);
    let card = find_by_name(&state, "Doom Blade");
    let legal_target = find_by_name(&state, "Grey Ogre");
    let action = cast_action(&state, card);

    for (label, mut bot) in [
        (
            "RandomBot",
            Box::new(RandomBot::new(1, "bot".into())) as Box<dyn Bot>,
        ),
        (
            "HeuristicBot",
            Box::new(HeuristicBot::new(1, "bot".into())) as Box<dyn Bot>,
        ),
    ] {
        let cmd = bot.choose_action(&state, PlayerId(1), std::slice::from_ref(&action));
        let Command::CastSpell(cast) = &cmd else {
            panic!("{label}: expected a CastSpell, got {cmd:?}");
        };
        assert_eq!(
            cast.targets,
            vec![Target::Object(legal_target)],
            "{label}: the bot must announce the one legal (non-black) creature"
        );

        // The mana still has to be paid, so tap first — the point of this assertion is
        // that the ANNOUNCEMENT is accepted, not that the cast is free.
        let mut working = state.clone();
        for land in ["Swamp 1", "Swamp 2"] {
            let source = find_by_name(&working, land);
            let (next, _) = process_command(
                working,
                Command::TapForMana {
                    player: PlayerId(1),
                    source,
                    ability_index: 0,
                    chosen_color: None,
                    hybrid_choices: vec![],
                    phyrexian_life_payments: vec![],
                },
            )
            .expect("tapping a fixture land must succeed");
            working = next;
        }
        process_command(working, cmd)
            .unwrap_or_else(|e| panic!("{label}: the engine refused the bot's cast: {e:?}"));
    }
}

/// **SIM-5 fix (2), CR 601.2c.** With no legal target on the board the announcement
/// is impossible however it is parameterised, and `plan_targets` says so rather than
/// inventing one. This is the predicate a future offer gate (G5 fix (4)) would use.
#[test]
fn plan_targets_reports_an_unsatisfiable_requirement() {
    let p2 = PlayerId(2);
    let state = doom_blade_state(vec![
        ObjectSpec::creature(p2, "Black Bear", 2, 2).with_colors(vec![Color::Black])
    ]);
    let card = find_by_name(&state, "Doom Blade");
    let action = cast_action(&state, card);
    assert_eq!(
        plan_targets(&state, PlayerId(1), &action),
        TargetPlan::Unsatisfiable,
        "the only creature is black, and Doom Blade excludes black (CR 601.2c)"
    );

    // ...and an action with no target requirements at all is left alone, so the
    // pre-SIM-5 behaviour is preserved everywhere it was already correct.
    assert_eq!(
        plan_targets(&state, PlayerId(1), &LegalAction::PassPriority),
        TargetPlan::NotTargeted
    );
}

/// A bot that always casts the first spell it is offered, announcing **nothing** —
/// the pre-SIM-5 `ActionParams::default()` behaviour, frozen so
/// [`a_rejected_bot_cast_commits_no_taps`] pins ATOMICITY and nothing else. Without
/// this the test would pass for the wrong reason the moment targeting improved.
struct ZeroTargetCastBot;

impl Bot for ZeroTargetCastBot {
    fn choose_action(
        &mut self,
        state: &GameState,
        player: PlayerId,
        legal: &[LegalAction],
    ) -> Command {
        for action in legal {
            if matches!(action, LegalAction::CastSpell { .. }) {
                if let Ok(cmd) =
                    action_to_command_with_params(state, player, action, &ActionParams::default())
                {
                    return cmd;
                }
            }
        }
        Command::PassPriority { player }
    }
    fn choose_targets(&mut self, _: &GameState, _: &[ObjectId], _: usize) -> Vec<ObjectId> {
        Vec::new()
    }
    fn choose_attackers(
        &mut self,
        _: &GameState,
        _: &[ObjectId],
        _: &[mtg_engine::AttackTarget],
    ) -> Vec<(ObjectId, mtg_engine::AttackTarget)> {
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
        "zero-target"
    }
}

/// **SIM-5 fixes (1) and (3) — the G5 mechanism itself.**
///
/// `advance()` auto-taps for a cast it is about to make (`auto_tap_commands_for`,
/// which prices the mana cost and never looks at targets), and the engine then
/// refuses the cast. Before SIM-5 the `[tap, tap, cast]` vector was applied one
/// command at a time, so both taps were committed, the bot passed, and CR 500.4 threw
/// the floating mana away at the next step boundary — 26 wasted taps in the triage's
/// live game. Now the whole vector goes through `apply_sequence`, so a refused cast
/// leaves **every land untapped and the pool empty**, and the refusal is recorded
/// instead of discarded.
///
/// Reverting `advance()`'s `apply_sequence` call to the old per-command loop makes
/// this fail on the `is_tapped` assertion (executed, not assumed).
#[test]
fn a_rejected_bot_cast_commits_no_taps() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);
    // No creature anywhere: Doom Blade's one mandatory requirement is unsatisfiable,
    // so the cast is refused however it is announced.
    let state = doom_blade_state(Vec::new());

    let mut bots: HashMap<PlayerId, Box<dyn Bot>> = HashMap::new();
    bots.insert(p1, Box::new(ZeroTargetCastBot));
    // `p2` is a human seat purely so `advance()` returns control to this test between
    // priority windows -- `LocalGame::start` resets the turn to Untap, so a bot-only
    // fixture would run the whole game before we could look at it.
    let human: BTreeSet<PlayerId> = [p2].into_iter().collect();
    let (mut game, _events) = LocalGame::start(
        state,
        0,
        StubProvider,
        bots,
        human,
        LocalGameLimits {
            max_turns: 3,
            max_commands: 400,
            max_consecutive_passes: 100,
            record_journal: true,
        },
        false,
    )
    .expect("fixture game must start");

    // Drive until the bot has attempted (and been refused) its cast.
    for _ in 0..200 {
        if game.rejection_count() > 0 {
            break;
        }
        match game.advance() {
            AdvanceOutcome::AwaitingHuman(decision) => {
                let index = decision
                    .actions
                    .iter()
                    .position(|a| matches!(a, LegalAction::PassPriority))
                    .expect("the human seat is always offered a pass");
                game.submit(
                    decision.seq,
                    HumanChoice {
                        action_index: index,
                        params: ActionParams::default(),
                    },
                )
                .expect("passing must be accepted");
            }
            other => panic!("fixture ended early: {other:?}"),
        }
    }

    assert_eq!(
        game.rejection_count(),
        1,
        "the bot's unsatisfiable Doom Blade cast must have been refused exactly once"
    );

    // (3): the error is observable rather than discarded.
    let rejection = &game.rejections()[0];
    assert_eq!(rejection.player, p1);
    assert!(
        matches!(&rejection.command, Command::CastSpell(c) if c.targets.is_empty()),
        "the recorded command must be the bot's cast: {:?}",
        rejection.command
    );
    assert!(
        rejection.error.contains("InvalidTarget"),
        "the engine's own reason must be kept verbatim, got {:?}",
        rejection.error
    );

    // (1): nothing was spent on it.
    for land in ["Swamp 1", "Swamp 2"] {
        let id = find_by_name(game.state(), land);
        assert!(
            !game.state().objects().get(&id).unwrap().status.tapped,
            "{land} was tapped for a cast the engine refused (G5)"
        );
    }
    assert_eq!(
        game.state().player(p1).unwrap().mana_pool.total(),
        0,
        "no mana may be left floating from a refused cast"
    );
    assert!(
        !game
            .journal()
            .iter()
            .any(|r| matches!(r.command, Command::TapForMana { .. })),
        "no tap may reach the journal from a rolled-back sequence"
    );
}
