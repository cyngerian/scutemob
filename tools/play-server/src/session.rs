//! Play-session lifecycle: build a game, advance it, answer a decision, mulligan.
//!
//! M11-local Session 5, plan item 3 (`memory/m11-session-plan.md` §4).
//!
//! # This module knows nothing about tokio
//!
//! Every function here is **synchronous**. The async boundary is exactly one
//! function deep and lives in `api.rs`: an axum handler acquires the session
//! mutex and runs these calls inside `tokio::task::block_in_place`, so a long
//! resolution (a deep trigger cascade, a 100-card shuffle) cannot stall the
//! reactor. Nothing below `api.rs` may reference tokio — that is what keeps the
//! simulator/engine stack callable from a test, a TUI, or a future non-async
//! host without change (plan §3, "the async boundary sits ...").
//!
//! # `std::sync::Mutex`, deliberately
//!
//! [`AppState::session`] is an `Arc<std::sync::Mutex<Option<PlaySession>>>`, not
//! a `tokio::sync::Mutex`. Every critical section is synchronous — there is no
//! `.await` between lock and unlock anywhere in this crate — so an async-aware
//! mutex would buy nothing and cost a task-yield per request. The plan specifies
//! this shape explicitly.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use mtg_engine::{GameEvent, PlayerId};
use mtg_simulator::{
    setup, AdvanceOutcome, Bot, BotKind, HeuristicBot, HumanChoice, LocalGame, LocalGameConfig,
    LocalGameError, PendingDecision, RandomBot, SetupError, StubProvider,
};

/// A single in-progress local game plus everything the HTTP layer needs to
/// render one seat's view of it.
///
/// Field list is the plan's (§4 Session 5 item 3) plus one addition,
/// `mulligan_count`: `setup::redeal` is keyed by `(seat, mulligan_count)` and
/// nothing else in the process remembers how many times this seat has
/// mulliganed. The engine's own `PlayerState::mulligan_count` is always 0 on
/// this path because a pregame redeal rebuilds the table from scratch rather
/// than issuing `Command::TakeMulligan` — see [`PlaySession::mulligan`].
pub struct PlaySession {
    pub game: LocalGame<StubProvider>,
    pub human: PlayerId,
    pub names: HashMap<PlayerId, String>,
    pub cfg: LocalGameConfig,
    /// Index into `LocalGame::journal()`; everything before it has already been
    /// shipped to the client as `EventView`s.
    pub journal_cursor: usize,
    /// Mirror of `LocalGame::pending_decision()`, refreshed by
    /// [`PlaySession::advance`]. `api.rs` maps `action_index` back through this
    /// so a `LegalAction` never has to be serialized.
    pub pending: Option<PendingDecision>,
    /// How many pregame redeals this seat has taken (CR 103.5). Additive to the
    /// plan's field list — see the struct doc.
    pub mulligan_count: u32,
}

/// CLI-supplied fallbacks for `POST /api/game` when the request body omits a
/// field (or is absent entirely).
#[derive(Clone, Copy, Debug)]
pub struct NewGameDefaults {
    pub players: u32,
    pub bot: BotKind,
    pub seed: u64,
}

/// The axum `State` handle. `Clone` is all axum requires; the session itself
/// lives behind the `Arc<Mutex<..>>` the plan specifies.
#[derive(Clone)]
pub struct AppState {
    pub session: Arc<Mutex<Option<PlaySession>>>,
    pub defaults: NewGameDefaults,
}

impl AppState {
    pub fn new(defaults: NewGameDefaults) -> Self {
        AppState {
            session: Arc::new(Mutex::new(None)),
            defaults,
        }
    }
}

/// Alias kept so handler signatures read the same way the replay viewer's do.
pub type SharedState = AppState;

/// Anything that can go wrong building or rebuilding a session.
#[derive(Debug)]
pub enum SessionError {
    /// Pregame assembly failed — a deck was refused by `validate_deck`
    /// (Architecture Invariant 9), or no deck could be built for a seat.
    Setup(SetupError),
    /// `LocalGame::start` failed — in practice `start_game`'s
    /// `check_all_defs_complete` (Architecture Invariant 9, second line of
    /// defence).
    Start(LocalGameError),
    /// The requested player count is outside the range this server will build.
    BadPlayerCount(u32),
    /// A mulligan was requested after the game had already been played into.
    NotPregame,
}

impl std::fmt::Display for SessionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SessionError::Setup(e) => write!(f, "pregame setup failed: {e}"),
            SessionError::Start(e) => write!(f, "the game could not be started: {e:?}"),
            SessionError::BadPlayerCount(n) => {
                write!(f, "player count {n} is out of range (2..={MAX_PLAYERS})")
            }
            SessionError::NotPregame => write!(
                f,
                "mulligans are pregame only; this game has already had commands applied"
            ),
        }
    }
}

impl std::error::Error for SessionError {}

/// CR 903.1 is a multiplayer format; a one-seat table has no game to play, and
/// the upper bound is the six-player suite the engine's tests already cover.
const MAX_PLAYERS: u32 = 6;

/// The human always sits in seat 1, matching `tools/tui`'s `PlayApp::new` and
/// `setup::seat_name`'s `Human-1` / `Bot-n` convention.
pub const HUMAN_SEAT: PlayerId = PlayerId(1);

/// Build a `LocalGameConfig` for a new game. The single place the play server
/// decides limits, so `new_game` and `mulligan` cannot drift apart.
///
/// The stall guards are the fuzzer's numbers with `max_turns` left generous: a
/// human game legitimately passes a great deal (plan §8 R6). `record_journal`
/// is **on** — the play server *is* the event feed, which is exactly the case
/// `LocalGameLimits::record_journal`'s own doc comment names.
pub fn config_for(defaults: NewGameDefaults) -> Result<LocalGameConfig, SessionError> {
    if !(2..=MAX_PLAYERS).contains(&defaults.players) {
        return Err(SessionError::BadPlayerCount(defaults.players));
    }
    Ok(LocalGameConfig {
        player_count: defaults.players,
        human_seats: [HUMAN_SEAT].into_iter().collect(),
        bot_kind: defaults.bot,
        seed: defaults.seed,
        decks: mtg_simulator::DeckSource::RandomPerSeat,
        limits: mtg_simulator::LocalGameLimits {
            max_turns: 200,
            max_commands: 200 * 200,
            max_consecutive_passes: 500,
            record_journal: true,
        },
    })
}

/// One `Box<dyn Bot>` per non-human seat, built the same way
/// `tools/tui/src/play/app.rs` and `mtg-fuzzer` build theirs: the bot kind is
/// read from `cfg.bot_kind` exactly once, and each seat's seed is derived
/// deterministically from `cfg.seed` so a given seed reproduces a given table.
fn bots_for(cfg: &LocalGameConfig) -> HashMap<PlayerId, Box<dyn Bot>> {
    let mut bots: HashMap<PlayerId, Box<dyn Bot>> = HashMap::new();
    for i in 1..=u64::from(cfg.player_count) {
        let pid = PlayerId(i);
        if cfg.human_seats.contains(&pid) {
            continue;
        }
        // Same derivation `crates/simulator/tests/local_game.rs::bots_for` uses.
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

/// CR 103.5 / 903.6 pregame assembly, then `start_game`.
///
/// Delegates wholesale to `mtg_simulator::setup::build_initial_state` (M11-local
/// Session 2), which admits every deck through the real `validate_deck`
/// (Architecture Invariant 9) before a single object is placed, and to
/// `LocalGame::start`, which runs `check_all_defs_complete` as the independent
/// second check. Neither gate is re-derived here.
pub fn new_game(cfg: LocalGameConfig) -> Result<PlaySession, SessionError> {
    let (state, names) = setup::build_initial_state(&cfg).map_err(SessionError::Setup)?;
    let bots = bots_for(&cfg);
    let (game, _start_events) = LocalGame::start(
        state,
        cfg.seed,
        StubProvider,
        bots,
        cfg.human_seats.clone(),
        cfg.limits,
        // Invariant checking is cheap relative to an HTTP round trip and this is
        // a play-testing surface: a violation should surface here, not in a
        // fuzzer run three weeks later.
        true,
    )
    .map_err(SessionError::Start)?;

    let human = cfg.human_seats.iter().next().copied().unwrap_or(HUMAN_SEAT);

    Ok(PlaySession {
        game,
        human,
        names,
        cfg,
        // `LocalGame::start`'s own `start_events` are deliberately dropped: they
        // are not `CommandRecord`s (no command produced them) and the journal —
        // the thing `journal_cursor` indexes — starts empty.
        journal_cursor: 0,
        pending: None,
        mulligan_count: 0,
    })
}

impl PlaySession {
    /// Run bot seats until the human must act, the game ends, or a safety valve
    /// trips, and refresh [`PlaySession::pending`] from the outcome.
    ///
    /// `LocalGame::advance` is idempotent while a decision is outstanding, so
    /// calling this from a read-only endpoint re-issues the same `seq` rather
    /// than invalidating the one the client holds.
    pub fn advance(&mut self) -> AdvanceOutcome {
        let outcome = self.game.advance();
        self.pending = match &outcome {
            AdvanceOutcome::AwaitingHuman(decision) => Some(decision.clone()),
            // A concluded or halted game has no outstanding decision.
            AdvanceOutcome::GameOver(_) | AdvanceOutcome::Halted(_) => None,
        };
        outcome
    }

    /// Answer the outstanding decision. Thin pass-through: `LocalGame::submit`
    /// owns the `seq` check, the `action_index` resolution and the atomic
    /// application, and leaves the state untouched on any rejection.
    pub fn submit(
        &mut self,
        seq: u64,
        choice: HumanChoice,
    ) -> Result<Vec<GameEvent>, LocalGameError> {
        let events = self.game.submit(seq, choice)?;
        self.pending = None;
        Ok(events)
    }

    /// CR 103.5 / 103.5c — take a pregame mulligan by rebuilding the table from
    /// a perturbed seed (`setup::redeal`).
    ///
    /// **Whole-table rebuild, and its two limitations are real.** `setup::redeal`
    /// documents them: the rebuild is *not* invisible to the other seats (CR
    /// 903.6 puts every commander in the public command zone, and a rebuild
    /// re-rolls them), and it cannot represent a partially-decided table (each
    /// seat has its own CR 103.5c mulligan count; one `(seat, count)` signature
    /// has nowhere to record that seat 2 already kept). `redeal`'s doc says the
    /// per-seat model "belongs with the Session 5 play-server pregame flow" —
    /// this session keeps the whole-table rebuild, because a per-seat model needs
    /// a second decision channel (each bot seat must be *asked*), which is more
    /// than a small addition and would be the first thing in this milestone to
    /// need a decision the engine does not already offer.
    ///
    /// **CR 103.5's bottoming half is not expressible here either.** After the
    /// n-th mulligan a kept hand puts `n - 1` cards on the bottom, and
    /// `handle_keep_hand` enforces exactly that against
    /// `PlayerState::mulligan_count` — which a rebuild always leaves at 0,
    /// because no `Command::TakeMulligan` is ever issued. `api.rs` therefore
    /// *refuses* a non-empty `cards_to_bottom` with 400 rather than accepting
    /// and silently discarding it.
    pub fn mulligan(&mut self) -> Result<(), SessionError> {
        if !self.is_pregame() {
            return Err(SessionError::NotPregame);
        }
        let next_count = self.mulligan_count + 1;
        let (state, names) =
            setup::redeal(&self.cfg, self.human, next_count).map_err(SessionError::Setup)?;
        let bots = bots_for(&self.cfg);
        let (game, _start_events) = LocalGame::start(
            state,
            self.cfg.seed,
            StubProvider,
            bots,
            self.cfg.human_seats.clone(),
            self.cfg.limits,
            true,
        )
        .map_err(SessionError::Start)?;

        self.game = game;
        self.names = names;
        self.journal_cursor = 0;
        self.pending = None;
        self.mulligan_count = next_count;
        Ok(())
    }

    /// True while no command has been applied to this game.
    ///
    /// This is the gate on the mulligan endpoint. "Pregame" cannot be read off
    /// the turn number: `setup::build_initial_state` sets `first_turn_of_game`
    /// and `GameStateBuilder` defaults `turn_number` to 1, so a freshly built
    /// game is already *in* turn 1 (which is also why
    /// `local_game::decision_kind_for`'s `Mulligan` arm, gated on
    /// `turn_number == 0`, is unreachable). A zero command count is the honest
    /// test: nothing has happened that a rebuild would discard.
    pub fn is_pregame(&self) -> bool {
        self.game.command_count() == 0
    }

    /// Drain the journal since `journal_cursor` and advance the cursor.
    ///
    /// Returns the `CommandRecord`s cloned, because the caller renders them into
    /// `EventView`s against `self.game.state()` and cannot hold a borrow of the
    /// journal across that.
    pub fn take_new_records(&mut self) -> Vec<mtg_simulator::CommandRecord> {
        let records = self.game.journal_since(self.journal_cursor).to_vec();
        self.journal_cursor += records.len();
        records
    }
}
