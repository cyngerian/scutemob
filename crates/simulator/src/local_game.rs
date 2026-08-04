//! Steppable local-game core (M11-local Session 1).
//!
//! `LocalGame` extracts the loop body of the old `GameDriver::run_game` (driver.rs)
//! so a caller can advance a game until a designated human seat must act, receive a
//! `PendingDecision` describing the legal actions available, and later hand back a
//! chosen `Command` via `submit`. With `human_seats` empty this behaves identically to
//! the pre-existing bot-only driver — `GameDriver::run_game` is re-expressed on top of
//! it in this same file.
//!
//! CR 117.3 governs priority and CR 903.9a the commander zone-change choice; both are
//! reachable today. `DecisionKind` also declares `Mulligan` (CR 103.5) and the combat
//! declarations (CR 508.1 / 509.1), but **mulligans are not actually reachable yet** —
//! see `decision_kind_for` below. Session 2 owns pregame setup and mulligans.
//! See `memory/m11-session-plan.md` §3-4.

use std::collections::{BTreeSet, HashMap};

use mtg_engine::{
    process_command, start_game, Command, GameEvent, GameState, GameStateError, PlayerId,
};

use crate::bot::Bot;
use crate::invariants::{self, InvariantViolation};
use crate::legal_actions;
use crate::legal_actions::{LegalAction, LegalActionProvider};
use crate::mana_solver;
use crate::params::{action_to_command_with_params, HumanChoice};
use crate::report::{GameDriverError, GameResult};

/// Turn/command/pass safety valves for a `LocalGame`. Mirrors the constants
/// `GameDriver` used to hardcode (driver.rs) as configurable fields so a human game
/// (which legitimately passes a lot) is not forced to share the fuzzer's defaults.
#[derive(Clone, Copy, Debug)]
pub struct LocalGameLimits {
    pub max_turns: u32,
    pub max_commands: u32,
    pub max_consecutive_passes: u32,
    /// Whether to retain a `CommandRecord` per applied command (see `journal()`).
    ///
    /// The play server needs this — it is the event feed and the bug-report export.
    /// The fuzzer does not: `GameDriver::run_game` discards events, and at its
    /// defaults (`max_commands = max_turns * 200`, thousands of games in parallel) an
    /// unconditional journal would retain up to tens of thousands of records per
    /// in-flight game, each holding a cloned `Vec<GameEvent>`, where the pre-M11 driver
    /// retained nothing. `GameDriver` therefore sets this `false`.
    pub record_journal: bool,
}

/// The outcome of a single `LocalGame::advance()` call.
#[derive(Debug)]
pub enum AdvanceOutcome {
    /// A human-occupied seat must act. The game state has not been advanced any
    /// further than the moment this decision became available.
    AwaitingHuman(PendingDecision),
    /// CR 104.2a/720 etc. — the game concluded normally.
    GameOver(GameResult),
    /// A safety valve tripped, or the engine rejected a command issued on a bot
    /// seat's behalf with no viable fallback.
    Halted(HaltReason),
}

/// Why `advance()` stopped without reaching `GameOver` or a human decision.
#[derive(Clone, Debug)]
pub enum HaltReason {
    /// `LocalGameLimits::max_turns` was exceeded without the game concluding.
    MaxTurns { max_turns: u32, turn: u32 },
    /// `LocalGameLimits::max_commands` or `max_consecutive_passes` was exceeded
    /// (stuck-game / infinite-loop protection).
    InfiniteLoop { turn: u32 },
    /// The legal-action provider returned no actions for the acting player and even
    /// `PassPriority` was rejected by the engine.
    NoLegalActions { player: PlayerId, turn: u32 },
    /// A bot-seat command was rejected by the engine and the `PassPriority` fallback
    /// was also rejected.
    EngineError(String),
}

impl From<HaltReason> for GameDriverError {
    fn from(reason: HaltReason) -> Self {
        match reason {
            HaltReason::MaxTurns { max_turns, .. } => GameDriverError::MaxTurnsReached(max_turns),
            HaltReason::InfiniteLoop { turn } => GameDriverError::InfiniteLoop { turn },
            HaltReason::NoLegalActions { player, turn } => {
                GameDriverError::NoLegalActions { player, turn }
            }
            HaltReason::EngineError(msg) => GameDriverError::EngineError(msg),
        }
    }
}

/// CR 117.3 (priority), CR 103.5 (mulligan), CR 903.9a (commander zone-change choice),
/// CR 508.1 / 509.1 (combat declarations), CR 514.1 (cleanup discard, PB-DP7 / DP-3) —
/// what kind of decision a human seat faces.
///
/// This enumerates command-submission-time decisions AND the out-of-band
/// engine-blocking decisions introduced by PB-DP7's `BlockingDecision`
/// mechanism (`rules::engine::blocking_decision`). PB-DP8 added the trigger-time
/// class (CR 603.3d) and PB-DP9 the MID-RESOLUTION class (CR 608.2d --
/// 701.22a/701.23a/701.25a), which the PB-DP7 plan's §1.5/1.6 and audit §9.4
/// rec 1 said this enum did not reach.
///
/// `#[non_exhaustive]`: audit §9.4 rec 1. This enum is no longer "the complete
/// set of decisions reachable by this architecture" (contrast the old claim at
/// audit §9.2), so a downstream exhaustive match must add a wildcard arm.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DecisionKind {
    Priority,
    Mulligan,
    CommanderZoneChoice,
    DeclareAttackers,
    DeclareBlockers,
    /// CR 514.1 (PB-DP7 / DP-3): the active player must discard to hand size.
    /// Unlike every other variant, this is not a priority-window choice --
    /// CR 514.3 grants no priority in cleanup, so it is the engine's first
    /// out-of-band BLOCKING decision (`rules::engine::BlockingDecision`).
    CleanupDiscard,
    /// CR 603.3d (PB-DP8 / DP-6): the controller of a triggered ability must
    /// announce its targets before it goes on the stack. Like `CleanupDiscard`
    /// this is an out-of-band BLOCKING decision, not a priority-window choice --
    /// CR 603.3 grants priority only once the whole CR 603.3b batch is placed.
    TriggerTargets,
    /// CR 608.2d (PB-DP9 / DP-7/8/9): a player must announce a resolution-time
    /// choice -- which card a search finds, or how a scry/surveil splits the
    /// cards looked at. Also an out-of-band BLOCKING decision, and the first one
    /// that arises INSIDE a resolution: the engine has rolled the whole
    /// resolution back to the moment before it began and will re-run it once the
    /// answer arrives.
    EffectChoice,
}

/// A decision a human-occupied seat must make before the game can advance further.
#[derive(Clone, Debug)]
pub struct PendingDecision {
    /// Monotonically increasing per `LocalGame`. `submit()` rejects a mismatched
    /// `seq` so a stale browser tab cannot act on a superseded action list.
    pub seq: u64,
    pub player: PlayerId,
    pub kind: DecisionKind,
    pub actions: Vec<LegalAction>,
}

/// A command that was actually applied to the game state, and the events it produced.
#[derive(Clone, Debug)]
pub struct CommandRecord {
    pub command: Command,
    pub events: Vec<GameEvent>,
    pub turn: u32,
}

/// A command the engine **refused** on a bot seat's behalf, and why (SIM-5 fix (3),
/// G5 of `memory/playtest-triage-2026-08-02b.md`).
///
/// Before SIM-5 the error at `advance()`'s rejection arm was bound and then dropped,
/// so a rejected bot command left no trace anywhere: the journal records applied
/// commands only, which is why the triage could only *infer* why bots were wasting
/// mana ("the journal records applied commands only, so the rejected command and its
/// error string are unrecoverable"). Recording it here is what lets the next triage
/// classify a bot failure instead of inferring it.
///
/// This is not an engine bug by itself — `StubProvider` is a bot move generator, not
/// a rules-complete action enumerator (see the SR-38 discussion in `legal_actions.rs`),
/// so it can legitimately offer an action the engine refuses. A *rising* count, or a
/// new error string, is the signal.
#[derive(Clone, Debug)]
pub struct RejectedCommand {
    /// The seat the command was issued for.
    pub player: PlayerId,
    /// `state.turn().turn_number` at the moment of refusal.
    pub turn: u32,
    /// The action the bot chose — **not** any tapping commands prepended to it. The
    /// taps are not recorded because, post-SIM-5, they were never applied: the whole
    /// `[taps…, cast]` sequence goes through `apply_sequence` and is rolled back
    /// wholesale.
    pub command: Command,
    /// The engine's own rejection reason, stringified.
    pub error: String,
}

/// A constant-size census of the rules mechanics a game actually exercised.
///
/// # Why this exists (PB-DX22 fix cycle, review Findings 1 and 2)
///
/// PB-DX22's headline numbers — `CommanderCastFromCommandZone` 0 → 36, 13 CR 903.9a
/// returns, the first-cast turn band 3-29 — were produced by a scratch
/// `crates/simulator/examples/` binary that was **deleted**, so nothing in the shipped
/// tree could re-derive them. The review called that out as the batch's own defect class:
/// *the batch committed its "before" and discarded its "after"*. This type is the repair.
/// `mtg-fuzzer` now reports these in its own summary, over **every** game in the run, so
/// the numbers are a property of committed code rather than of a lost scratch file.
///
/// # It is a counter set, not a journal
///
/// `GameDriver` deliberately runs with `record_journal: false` (thousands of long games
/// in parallel; a journal retains up to tens of thousands of `CommandRecord`s per game).
/// Everything here is `u32`/`Option<u32>` and is folded from the events of each applied
/// command as they go past, so the fuzzer pays constant memory and no retention. It is
/// therefore available on **every** game, not on the sampled subset the binary prints
/// per-violation detail for.
///
/// CR 601.2 (`SpellCast`), CR 305.1 (`LandPlayed`), CR 903.8
/// (`CommanderCastFromCommandZone`), CR 903.9a (`CommanderReturnedToCommandZone`),
/// CR 903.9b (`CommanderZoneRedirect`), CR 903.10a (`commander_damage_received`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MechanicsTally {
    /// CR 601.2 — every `GameEvent::SpellCast`. A commander cast from the command zone
    /// emits **both** `SpellCast` and `CommanderCastFromCommandZone` (see that event's
    /// doc), so it is counted here too; `commander_casts_from_command_zone` is the
    /// subset.
    pub spell_casts: u32,
    /// Turn number of the first `SpellCast` — the number `OOS-UI2-1` / `OOS-SIM3-1` are
    /// about. `None` if the game cast nothing at all, which is what the pre-PB-DX22
    /// unshuffled instrument produced below `--max-turns` ~140.
    pub first_spell_cast_turn: Option<u32>,
    /// CR 601.2 — the first `SpellCast` that is **not** a command-zone commander cast.
    /// Distinct from `first_spell_cast_turn` because only this one is gated by library
    /// order, i.e. only this one measures the CR 103.3 shuffle (review Finding 4).
    pub first_library_spell_cast_turn: Option<u32>,
    /// CR 305.1 — lands played, and the turn of the first. Records whether land
    /// availability, rather than draw depth, is what gates the first cast.
    pub lands_played: u32,
    pub first_land_played_turn: Option<u32>,
    /// CR 903.8 — commander casts from the command zone. **0 in every fuzz game before
    /// PB-DX22**, because `commander_ids` was never populated (`OOS-SIM1-4`).
    pub commander_casts_from_command_zone: u32,
    pub first_commander_cast_turn: Option<u32>,
    /// CR 903.9a — commanders returned to the command zone from a graveyard or exile.
    pub commander_returns_to_command_zone: u32,
    /// CR 903.9b — commanders redirected to the command zone instead of changing zones.
    pub commander_zone_redirects: u32,
    /// CR 903.10a — seats whose `commander_damage_received` map is non-empty at the end
    /// of the game. Read from the final state by [`LocalGame::mechanics`], not folded
    /// from events (no event carries the running total).
    pub seats_dealt_commander_damage: u32,
    /// CR 903.10a — the largest single (dealt-to, dealt-by) commander-damage total at the
    /// end of the game. The rule's threshold is 21.
    pub max_commander_damage: u32,
}

impl MechanicsTally {
    /// Fold one applied command's events into the census. `turn` is
    /// `state.turn().turn_number` as of *after* the command was applied — the same
    /// number `CommandRecord::turn` carries.
    fn record(&mut self, events: &[GameEvent], turn: u32) {
        for event in events {
            match event {
                GameEvent::SpellCast { .. } => {
                    self.spell_casts = self.spell_casts.saturating_add(1);
                    self.first_spell_cast_turn.get_or_insert(turn);
                }
                GameEvent::LandPlayed { .. } => {
                    self.lands_played = self.lands_played.saturating_add(1);
                    self.first_land_played_turn.get_or_insert(turn);
                }
                GameEvent::CommanderCastFromCommandZone { .. } => {
                    self.commander_casts_from_command_zone =
                        self.commander_casts_from_command_zone.saturating_add(1);
                    self.first_commander_cast_turn.get_or_insert(turn);
                }
                GameEvent::CommanderReturnedToCommandZone { .. } => {
                    self.commander_returns_to_command_zone =
                        self.commander_returns_to_command_zone.saturating_add(1);
                }
                GameEvent::CommanderZoneRedirect { .. } => {
                    self.commander_zone_redirects = self.commander_zone_redirects.saturating_add(1);
                }
                _ => {}
            }
        }
        // A command-zone commander cast emits BOTH `SpellCast` and
        // `CommanderCastFromCommandZone`, so "was any cast in this batch of events a
        // library cast?" is decided per COMMAND, not per event: if the command produced
        // a `SpellCast` and no `CommanderCastFromCommandZone`, the spell came from
        // somewhere that is not the command zone.
        if self.first_library_spell_cast_turn.is_none()
            && events
                .iter()
                .any(|e| matches!(e, GameEvent::SpellCast { .. }))
            && !events
                .iter()
                .any(|e| matches!(e, GameEvent::CommanderCastFromCommandZone { .. }))
        {
            self.first_library_spell_cast_turn = Some(turn);
        }
    }
}

/// Errors `LocalGame::start` / `submit` can return. Distinct from `GameStateError`
/// (the engine's own rejection reason) so a caller can tell "your seq was stale" apart
/// from "the engine said no" apart from "the engine itself is unusable".
#[derive(Clone, Debug)]
pub enum LocalGameError {
    /// `submit`'s `seq` did not match the currently pending decision's `seq`.
    StaleDecision { expected: u64, got: u64 },
    /// `submit` was called with no outstanding `PendingDecision`.
    NoPendingDecision,
    /// `choice.action_index` was out of range for the pending decision's
    /// `actions`. Session 3 (item 7): this is also, structurally, what a
    /// "cross-seat command" attempt collapses into — there is no longer any way
    /// to name a `Command` for a player other than `pending.player`.
    UnknownAction(usize),
    /// `action_to_command_with_params` rejected the supplied `ActionParams`
    /// (`ParamError`, stringified) — Session 3 (item 5/7).
    BadParams(String),
    /// The engine rejected the submitted command. `self.state` is left untouched —
    /// `submit` never falls back to `PassPriority` on a human seat's behalf.
    Rejected(GameStateError),
    /// A failure while starting the game or advancing bot seats.
    Engine(GameStateError),
}

/// A steppable local Commander game: owns the `GameState`, a `LegalActionProvider`,
/// and the bots for every non-human seat. `advance()` runs bot seats autonomously
/// (identically to the old `GameDriver::run_game` loop) and stops the moment a
/// human-occupied seat must act. `human_seats` empty makes this behave exactly like
/// the bot-only driver — see `GameDriver::run_game` below.
pub struct LocalGame<P: LegalActionProvider> {
    state: GameState,
    /// The seed the game was started with, carried only so `GameResult.seed` (a
    /// pre-existing, unchanged field on the fuzzer's report type) can be populated by
    /// `advance()` on `GameOver`; `LocalGame` itself never consumes randomness.
    seed: u64,
    provider: P,
    bots: HashMap<PlayerId, Box<dyn Bot>>,
    /// Seats a human occupies. Empty => pure bot game (the `GameDriver` case).
    human_seats: BTreeSet<PlayerId>,
    limits: LocalGameLimits,
    consecutive_passes: u32,
    command_count: u32,
    /// Turn number as of the last *tracked* command (mirrors the old `run_game`'s
    /// local `prev_turn`) — feeds `check_game_progression`'s stagnation check.
    prev_turn: u32,
    /// Monotonic. Every emitted `PendingDecision` carries the post-increment value.
    decision_seq: u64,
    /// The decision `submit` is currently allowed to answer, if any.
    pending: Option<PendingDecision>,
    journal: Vec<CommandRecord>,
    /// Bot-seat commands the engine refused (SIM-5 fix (3)). Retention is capped at
    /// [`MAX_RETAINED_REJECTIONS`]; `rejection_count` is not, so truncation is
    /// visible rather than silent.
    rejections: Vec<RejectedCommand>,
    rejection_count: u32,
    violations: Vec<InvariantViolation>,
    check_invariants: bool,
    /// Constant-size mechanics census (PB-DX22 fix cycle). Always on: it costs a fold
    /// over events already in hand and no retention, so unlike `journal` it does not
    /// need `LocalGameLimits::record_journal`. See [`MechanicsTally`].
    mechanics: MechanicsTally,
}

/// How many [`RejectedCommand`]s a `LocalGame` retains **when `record_journal` is
/// on** (play-server, the SIM-5/SIM-6 fixtures). Unbounded retention is not safe here:
/// nothing caps how often a bot re-chooses an action the engine refuses (`advance()`'s
/// own comment on the auto-tap notes the identical action is re-offered next priority).
/// The count is kept whole so a caller can always see that dropping happened — see
/// [`LocalGame::rejection_count`].
pub const MAX_RETAINED_REJECTIONS: usize = 256;

/// How many [`RejectedCommand`]s a `LocalGame` retains **when `record_journal` is
/// off** (PB-DX32 Stage 2 — `mtg-fuzzer`'s own case, see
/// [`LocalGame::record_rejection`]).
///
/// A much smaller cap than [`MAX_RETAINED_REJECTIONS`] on purpose: `results` in
/// `bin/fuzzer.rs` retains **every** game's `GameResult` for the whole run, so at
/// `--games 1000` a 256-cap would retain up to 256,000 cloned `Command`s. Eight per
/// game is a diagnosis sample — [`LocalGame::rejection_count`] is uncapped and
/// ungated (see that method), so truncation of the sample stays visible even though
/// the sample itself is small.
pub const MAX_SAMPLED_REJECTIONS: usize = 8;

impl<P: LegalActionProvider> LocalGame<P> {
    /// Starts a game from an assembled (but not yet started) `GameState`. Delegates to
    /// `mtg_engine::start_game`, which enforces Architecture Invariant 9 — a game
    /// whose objects reference a non-`Complete` `CardDefinition` is refused with
    /// `GameStateError::IncompleteCardsInGame`, surfaced here as `LocalGameError::Engine`.
    #[allow(clippy::too_many_arguments)]
    pub fn start(
        state: GameState,
        seed: u64,
        provider: P,
        bots: HashMap<PlayerId, Box<dyn Bot>>,
        human_seats: BTreeSet<PlayerId>,
        limits: LocalGameLimits,
        check_invariants: bool,
    ) -> Result<(Self, Vec<GameEvent>), LocalGameError> {
        let (state, start_events) = start_game(state).map_err(LocalGameError::Engine)?;
        let prev_turn = state.turn().turn_number;
        let game = LocalGame {
            state,
            seed,
            provider,
            bots,
            human_seats,
            limits,
            consecutive_passes: 0,
            command_count: 0,
            prev_turn,
            decision_seq: 0,
            pending: None,
            journal: Vec::new(),
            rejections: Vec::new(),
            rejection_count: 0,
            violations: Vec::new(),
            check_invariants,
            mechanics: MechanicsTally::default(),
        };
        Ok((game, start_events))
    }

    pub fn state(&self) -> &GameState {
        &self.state
    }

    /// Build a [`GameResult`] snapshot from the current game state (PB-DX32 Stage 1).
    ///
    /// The single construction point for both REAL `GameResult` sites — the `GameOver`
    /// return in `advance()` below, and `GameDriver`'s Halted arm. Before this, those
    /// two sites were hand-maintained literals that had to agree, field-for-field, by
    /// inspection alone; PB-DX32 adds instrumentation fields across four stages, and a
    /// second hand-maintained literal is exactly the divergence class that produces a
    /// Halted-game report silently missing its instrumentation (plan §7 R5). `winner`
    /// and `error` are parameters because only the caller knows which of those two
    /// outcomes it is reporting — everything else is read straight off `self`.
    pub fn result_snapshot(
        &self,
        winner: Option<PlayerId>,
        error: Option<GameDriverError>,
    ) -> GameResult {
        GameResult {
            seed: self.seed,
            winner,
            turn_count: self.state.turn().turn_number,
            total_commands: self.command_count as usize,
            violations: self.violations.clone(),
            error,
            // PB-DX32 Stage 2 (SR-38).
            rejection_count: self.rejection_count,
            rejections: self.rejections.clone(),
        }
    }

    pub fn command_count(&self) -> u32 {
        self.command_count
    }

    pub fn violations(&self) -> &[InvariantViolation] {
        &self.violations
    }

    pub fn journal(&self) -> &[CommandRecord] {
        &self.journal
    }

    /// The game's mechanics census (PB-DX22 fix cycle, review Finding 1).
    ///
    /// Everything except the CR 903.10a fields is folded from events as commands are
    /// applied; `seats_dealt_commander_damage` and `max_commander_damage` are read from
    /// the **current** state here, because no event carries the running total —
    /// `rules/combat.rs` accumulates it directly into
    /// `PlayerState::commander_damage_received`, gated on `commander_ids`, which is
    /// exactly the registration PB-DX22 added.
    ///
    /// Available regardless of `LocalGameLimits::record_journal`.
    pub fn mechanics(&self) -> MechanicsTally {
        let mut tally = self.mechanics;
        for player in self.state.players().values() {
            if player.commander_damage_received.is_empty() {
                continue;
            }
            tally.seats_dealt_commander_damage =
                tally.seats_dealt_commander_damage.saturating_add(1);
            for by_card in player.commander_damage_received.values() {
                for dmg in by_card.values() {
                    tally.max_commander_damage = tally.max_commander_damage.max(*dmg);
                }
            }
        }
        tally
    }

    /// Bot-seat commands the engine refused, oldest first (SIM-5 fix (3)).
    ///
    /// **PB-DX32 Stage 2 correction**: before this stage the doc here said "Empty when
    /// `LocalGameLimits::record_journal` is off" — true at the time, false the moment
    /// SR-38 needed a sample from exactly that configuration (`mtg-fuzzer`'s own case).
    /// The record is no longer gated on `record_journal` at all; only its CAP is —
    /// truncated at [`MAX_RETAINED_REJECTIONS`] when the journal is on, or the much
    /// smaller [`MAX_SAMPLED_REJECTIONS`] when it is off (see
    /// [`Self::record_rejection`]). Compare against [`Self::rejection_count`], which is
    /// never gated or truncated, to see whether anything was dropped.
    pub fn rejections(&self) -> &[RejectedCommand] {
        &self.rejections
    }

    /// How many bot-seat commands the engine refused over the whole game. Never
    /// truncated, unlike [`Self::rejections`].
    pub fn rejection_count(&self) -> u32 {
        self.rejection_count
    }

    /// Every `CommandRecord` recorded since `cursor` (an index into `journal()`).
    /// Empty when `LocalGameLimits::record_journal` is `false`.
    pub fn journal_since(&self, cursor: usize) -> &[CommandRecord] {
        if cursor >= self.journal.len() {
            &[]
        } else {
            &self.journal[cursor..]
        }
    }

    /// The decision currently awaiting a human answer, if any.
    pub fn pending_decision(&self) -> Option<&PendingDecision> {
        self.pending.as_ref()
    }

    /// Advance the game: run bot seats autonomously (identically to the old
    /// `GameDriver::run_game` loop body) until the game ends, a human-occupied seat
    /// must act, or a safety valve trips.
    /// `advance()` is **idempotent while a decision is outstanding**: if a previous call
    /// returned `AwaitingHuman` and no `submit()` has answered it, this returns that same
    /// `PendingDecision` — same `seq`, same actions — rather than issuing a new one.
    /// Without that, a poll/keepalive endpoint or a browser refresh would silently
    /// invalidate the `seq` the client is holding, and the client's `submit()` would fail
    /// with `StaleDecision { expected: <a seq it never saw> }`.
    pub fn advance(&mut self) -> AdvanceOutcome {
        loop {
            if is_game_over(&self.state) {
                let winner = find_winner(&self.state);
                // A finished game has no outstanding decision — do not keep reporting one
                // from `pending_decision()`, and do not let `submit()` accept it.
                self.pending = None;
                return AdvanceOutcome::GameOver(self.result_snapshot(winner, None));
            }

            // Re-entrancy guard (see the doc comment above). Placed after the game-over
            // check so a concluded game reports `GameOver`, not a stale decision.
            if let Some(pending) = &self.pending {
                return AdvanceOutcome::AwaitingHuman(pending.clone());
            }

            if self.state.turn().turn_number > self.limits.max_turns {
                return AdvanceOutcome::Halted(HaltReason::MaxTurns {
                    max_turns: self.limits.max_turns,
                    turn: self.state.turn().turn_number,
                });
            }

            if self.command_count >= self.limits.max_commands {
                return AdvanceOutcome::Halted(HaltReason::InfiniteLoop {
                    turn: self.state.turn().turn_number,
                });
            }

            if self.consecutive_passes >= self.limits.max_consecutive_passes {
                return AdvanceOutcome::Halted(HaltReason::InfiniteLoop {
                    turn: self.state.turn().turn_number,
                });
            }

            // Determine acting player (CR 903.9a commander zone choice, then CR 117.3
            // priority holder, then a structural pass to advance between steps).
            //
            // Human seats are NOT branched on here. The seat is resolved first, legal
            // actions are enumerated for it, and only then does a human seat stop the
            // loop — so a human hits the same empty-legal-actions auto-pass a bot does.
            // Stopping earlier would hand a human an `AwaitingHuman` carrying an empty
            // action list, with every later `advance()` re-issuing it and no counter
            // moving: a deadlock with no safety valve. `StubProvider` cannot produce that
            // state today (it always offers the priority holder `PassPriority`), but
            // Session 3 replaces the provider.
            // PB-DP7 / DP-3 (CR 514.1): the outstanding cleanup discard, if any,
            // MUST be resolved before the commander-zone branch below --
            // the engine's admission gate (`rules::engine::process_command`)
            // rejects `ReturnCommanderToCommandZone` while a cleanup discard is
            // blocking, so offering that first would produce a command the
            // engine refuses and `advance()` would return
            // `Halted(EngineError)`.
            // Fix-cycle Finding 4 (MEDIUM): read the liveness-filtered
            // predicate, not the raw `pending_cleanup_discard()` field -- a
            // dead active player's stale entry must not make `LocalGame`
            // resolve `acting_player` to a player who can never answer.
            let (acting_player, forced_kind) = if let Some(decision) =
                self.state.blocking_decision()
            {
                // PB-DP8: an EXHAUSTIVE match, not a hard-coded kind. Before
                // PB-DP8 this branch mapped every `BlockingDecision` to
                // `DecisionKind::CleanupDiscard`, which would have handed a
                // browser client the wrong picker the moment a second variant
                // existed. `BlockingDecision` is deliberately not
                // `#[non_exhaustive]`, so this is now compile-forced for every
                // future variant.
                use mtg_engine::rules::engine::BlockingDecision;
                let kind = match decision {
                    BlockingDecision::CleanupDiscard { .. } => DecisionKind::CleanupDiscard,
                    BlockingDecision::TriggerTargets { .. } => DecisionKind::TriggerTargets,
                    BlockingDecision::EffectChoice { .. } => DecisionKind::EffectChoice,
                };
                (decision.player(), Some(kind))
            } else if let Some(pending) = self.state.pending_commander_zone_choices().iter().next()
            {
                (pending.0, Some(DecisionKind::CommanderZoneChoice))
            } else if let Some(priority) = self.state.turn().priority_holder {
                (priority, None)
            } else {
                // No one has priority and no pending choices — pass to advance. This
                // can happen between steps; issue PassPriority for active player.
                let active = self.state.turn().active_player;
                let cmd = Command::PassPriority { player: active };
                match self.apply_command(cmd, false) {
                    Ok(_events) => {
                        self.consecutive_passes += 1;
                        continue;
                    }
                    Err(e) => {
                        return AdvanceOutcome::Halted(HaltReason::EngineError(format!("{:?}", e)));
                    }
                }
            };

            // Get legal actions for the acting player, human or bot.
            let legal = self.provider.legal_actions(&self.state, acting_player);

            if legal.is_empty() {
                // No legal actions — pass priority to advance. Deliberately ahead of the
                // human-seat check: there is nothing for a human to choose here either.
                let cmd = Command::PassPriority {
                    player: acting_player,
                };
                match self.apply_command(cmd, false) {
                    Ok(_events) => {
                        self.consecutive_passes += 1;
                        continue;
                    }
                    Err(_) => {
                        return AdvanceOutcome::Halted(HaltReason::NoLegalActions {
                            player: acting_player,
                            turn: self.state.turn().turn_number,
                        });
                    }
                }
            }

            // A human-occupied seat must act: stop and hand the decision out.
            if self.human_seats.contains(&acting_player) {
                // Classified from the PROVIDER's list, before augmentation — the
                // human-only extras are never what the decision is *about*.
                let kind = forced_kind.unwrap_or_else(|| decision_kind_for(&self.state, &legal));
                let mut legal = legal;
                legal.extend(human_only_actions(
                    &self.state,
                    acting_player,
                    forced_kind.is_some(),
                ));
                return self.await_human(acting_player, kind, legal);
            }

            // Bot chooses an action.
            let cmd = if let Some(bot) = self.bots.get_mut(&acting_player) {
                bot.choose_action(&self.state, acting_player, &legal)
            } else {
                // No bot assigned — pass priority.
                Command::PassPriority {
                    player: acting_player,
                }
            };

            // Track passes for loop detection.
            if matches!(cmd, Command::PassPriority { .. }) {
                self.consecutive_passes += 1;
            } else {
                self.consecutive_passes = 0;
            }

            // If the command is CastSpell, auto-tap mana sources first.
            //
            // CR 903.8 (SIM-1): same helper as the human path and the offer gate --
            // see `auto_tap_commands_for`. A bot offered a taxed commander cast and
            // handed a printed-cost tap plan gets its cast rejected, falls through to
            // the `PassPriority` fallback below, and is re-offered the identical
            // action next priority: `HeuristicBot` scores `CastSpell` at
            // `50 + 10*mana_value` and `RandomBot` picks uniformly, and nothing caps
            // the retry -- so the taxed cost must be known here too, not just at the
            // offer gate.
            //
            // SIM-2: this is now the SAME function the human path calls, rather than a
            // parallel `solve_mana_payment` on the printed-plus-tax cost. It was a
            // parallel one on the strength of a rationale that only held because the
            // solver was pool-blind: "a bot never has a reason to prefer its existing
            // pool over a fresh tap, so the asymmetry is harmless". With a residual
            // solver the two are not merely symmetric but identical, and the bot gets
            // the announced-`{X}` handling (CR 107.3) for free -- which it never had,
            // so a bot casting an `{X}` spell tapped for the base cost and had the
            // cast refused every time.
            //
            // UI-2's additional-cost pricing (CR 702.157 Squad) lives INSIDE
            // `auto_tap_commands_for` via `effective_cast_cost_with_additional`, so
            // the unified call keeps this site, the offer gate and the human path in
            // agreement about what a Squad-paying cast charges.
            let commands = {
                let mut cmds = self
                    .auto_tap_commands_for(&cmd, acting_player)
                    .unwrap_or_default();
                cmds.push(cmd.clone());
                cmds
            };

            // Execute the whole plan (tap commands + the action) ATOMICALLY.
            //
            // SIM-5 fix (1), G5 of `memory/playtest-triage-2026-08-02b.md`. This used
            // to be a `for c in commands` loop of `apply_command(c, true)` calls, and
            // that is the entire mechanism of G5: the taps were committed one at a
            // time, so a rejected cast left them applied, the bot passed, and CR 500.4
            // destroyed the floating mana at the next step boundary. The human path has
            // never had that failure mode -- `submit` routes the identical
            // `[taps…, cast]` vector through `apply_sequence`, whose doc says in as many
            // words that it exists to prevent "a tap-then-cast sequence where the tap
            // succeeded but the cast was rejected". This is that same call, on the bot
            // path. Measured on seeds 0/7/42 at 25 turns: **45 wasted taps across 30
            // wasted tap runs** before (of 82 tap runs in all), 0 after, with
            // `ManaPoolsEmptied` falling 1:1 with the wasted RUNS -- 30 before, 1 after
            // (`crates/simulator/tests/sim5_bot_cast_discipline.rs`).
            //
            // Two behavioural differences from the old loop, both deliberate:
            //
            // * **Invariants are checked once per sequence, not once per command.** A
            //   sequence is longer than one command only when a cast is being funded, so
            //   the states no longer checked are mid-payment ones (some taps applied,
            //   the cast not yet). `apply_sequence` still checks the post-sequence state
            //   against the same `prev_turn`.
            // * **Recorded fuzz seeds move only where a cast is REJECTED** -- a
            //   succeeding sequence commits exactly what the loop committed, in the same
            //   order, so `journal`/`command_count` are unchanged for it.
            //
            //   **The second half of that argument is now history.** SIM-5 added: "per
            //   `OOS-UI2-1` the fuzzer has never cast a spell at all, so the fuzzer's
            //   seeds cannot reach the changed branch." That premise is CLOSED by
            //   PB-DX22 (`scutemob-196`), which shuffles every fuzz library (CR 103.3)
            //   and registers the commander (CR 903.6): the first `SpellCast` moved from
            //   game turn **143-154** to a **3-29** band over 20 seeds, and 670 spells
            //   were cast across a 20-game run that previously produced ~120. So the
            //   fuzzer's seeds now DO reach this branch.
            //
            //   PB-DX22 did **not** re-run SIM-5's parity argument, and nothing here
            //   should be read as re-validating it. The first bullet (a succeeding
            //   sequence is byte-identical to the old loop) is a structural claim and
            //   stands on its own; the "unreachable in fuzz" claim is retired as
            //   evidence rather than refreshed.
            if let Err(e) = self.apply_sequence(commands) {
                // Command rejected — not necessarily fatal. The provider may produce
                // invalid actions for a bot seat (SR-38 is a goal, not a guarantee, in
                // `StubProvider`). Fall back to passing.
                //
                // SIM-5 fix (3): the error is RECORDED before the fallback rather than
                // dropped on the floor. See `RejectedCommand`.
                self.record_rejection(acting_player, &cmd, &e);
                let fallback = Command::PassPriority {
                    player: acting_player,
                };
                match self.apply_command(fallback, false) {
                    Ok(_events) => {
                        self.consecutive_passes += 1;
                    }
                    Err(e2) => {
                        return AdvanceOutcome::Halted(HaltReason::EngineError(format!(
                            "Both action and fallback failed: {:?}, {:?}",
                            e, e2
                        )));
                    }
                }
            }
        }
    }

    /// Submit a human's chosen action for the currently pending decision. Validates
    /// `seq` against the outstanding `PendingDecision`, resolves `choice.action_index`
    /// against `pending.actions`, and builds the `Command` via
    /// `action_to_command_with_params` — always **for `pending.player`**. Because the
    /// command is always built for the seat that was asked, a command naming a
    /// different seat is structurally unrepresentable (Session 1's separate
    /// `command_player` cross-seat check is gone — there is nothing left for it to
    /// check). On any failure (`UnknownAction`, `BadParams`, or an engine `Rejected`)
    /// `self.state` is left untouched — `submit` never falls back to `PassPriority`.
    ///
    /// `params.auto_tap` (Session 3, item 7 — the pool half of OOS-M11-2): when the
    /// resolved command is a `CastSpell` and the caster's EXISTING mana pool cannot
    /// already cover its cost, tapping commands are prepended. The whole sequence
    /// (taps + the action) is applied atomically to a clone of `self.state` — see
    /// `apply_sequence` — so a tap that succeeds but leaves the main command rejected
    /// never partially mutates `self.state`.
    pub fn submit(
        &mut self,
        seq: u64,
        choice: HumanChoice,
    ) -> Result<Vec<GameEvent>, LocalGameError> {
        let pending = self
            .pending
            .as_ref()
            .ok_or(LocalGameError::NoPendingDecision)?;
        if pending.seq != seq {
            return Err(LocalGameError::StaleDecision {
                expected: pending.seq,
                got: seq,
            });
        }

        let player = pending.player;
        let action = pending
            .actions
            .get(choice.action_index)
            .cloned()
            .ok_or(LocalGameError::UnknownAction(choice.action_index))?;

        let command = action_to_command_with_params(&self.state, player, &action, &choice.params)
            .map_err(|e| LocalGameError::BadParams(e.to_string()))?;

        let mut commands = Vec::new();
        if choice.params.auto_tap {
            if let Some(tap_commands) = self.auto_tap_commands_for(&command, player) {
                commands.extend(tap_commands);
            }
        }
        let is_pass = matches!(command, Command::PassPriority { .. });
        commands.push(command);

        let events = self.apply_sequence(commands)?;

        if is_pass {
            self.consecutive_passes += 1;
        } else {
            self.consecutive_passes = 0;
        }
        self.pending = None;
        Ok(events)
    }

    /// The auto-tapper, shared by the human `submit` path and `advance()`'s bot path
    /// (SIM-2 — they were two code paths with two different notions of the cost until
    /// this batch). Only ever fires for a `Command::CastSpell`; returns `None` when the
    /// command is something else, when the card has no mana cost, or when no plan
    /// exists, in which case the caller applies the main command alone.
    ///
    /// # The pool half of OOS-M11-2 is CLOSED here — as a residual, not a special case
    ///
    /// M11-local S3 opened this function to keep a human from over-tapping when the
    /// pool already covered a cost. It did that with an all-or-nothing check, and the
    /// "nothing" branch was wrong: anything short of FULL coverage handed the solver the
    /// entire printed cost with the pool never subtracted, so two floating mana plus a
    /// `{3}` cast tapped three more sources and CR 500.4 destroyed the float at the step
    /// boundary. A human observed exactly that in the browser client (triage F3).
    /// `mana_solver::solve_mana_payment_with_pool` now does the subtraction itself, and
    /// a fully-covering pool is simply the residual-is-zero case of the general rule.
    ///
    /// # Two `?`s remain here as the only error-discarding constructs on the human
    /// path, and neither of them hides a failure (M11-local S8, item 4; count and
    /// list updated by SIM-1, re-counted by SIM-2)
    ///
    /// The S8 error-surfacing audit swept this file and `tools/play-server/src` for
    /// anything that drops a `Result`. Reachable from `submit`, there were originally
    /// three in this function: `state.object(..).ok()?`, `state.player(..).ok()?`
    /// and `flatten_hybrid_phyrexian(..).ok()?`. SIM-1 replaced the first with a call
    /// to `legal_actions::effective_cast_cost`, which performs the identical
    /// `state.object(..).ok()?` (plus a `mana_cost.clone()?`) INSIDE itself — so that
    /// `?` still exists, just one level down, and the argument below still holds for
    /// it. SIM-2 removed the other two from this function outright: the pool clone and
    /// the flatten both moved inside the solver, where the pool read is an `if let Ok`
    /// (a missing player means "no pool to subtract", not a lost error) and the flatten
    /// is `PipTracker::from_cost`, which is total. **So exactly one `?` of this class is
    /// now reachable from `submit`, and it is inside `effective_cast_cost`.** It returns
    /// `None`, which means only *"prepend no tapping commands"* — the caller then
    /// applies the main `CastSpell` alone, and if it cannot be paid for the **engine**
    /// rejects it and `submit` returns `LocalGameError::Rejected`. So a discarded error
    /// here still surfaces, as the cast's own refusal, rather than as a silently
    /// different game.
    ///
    /// Everything else the sweep found is on the **bot** path inside `advance()`
    /// (the `Err(_)` auto-pass arm, the auto-tap's `unwrap_or_default()`) and is
    /// unreachable from `submit`, which never calls `apply_command` at all — it goes to
    /// `apply_sequence`, whose only failure mode is to return `Rejected` with
    /// `self.state` untouched. The bot-seat `PassPriority` fallback is therefore
    /// structurally out of reach from a human submission, not merely unused:
    /// `advance()` returns at the `human_seats.contains(..)` branch before the bot
    /// branch exists.
    ///
    /// **Known limitation, narrowed by SIM-1 -- the commander-tax half of
    /// OOS-M11-2 is now CLOSED**: the pool used to be checked against
    /// `obj.characteristics.mana_cost`, the *printed* cost, with no commander tax
    /// (CR 903.8) folded in. `legal_actions::effective_cast_cost` now applies that
    /// tax before either the pool check or the solve, so the offer gate, this human
    /// auto-tap and the bot auto-tap in `advance()` cannot disagree about what has to
    /// be paid (T8/T8b pin this). **Still open, and still invisible here**: no
    /// Thalia-style cost INCREASE, no cost REDUCTION, and the pool subtraction is made
    /// with no `SpellContext`, so CR 106.12 restricted mana is invisible. Those three
    /// remain the surviving halves of OOS-M11-2 and are out of SIM-1's and SIM-2's
    /// scope -- fixing them means teaching the solver/helper about layer-resolved cost
    /// MODIFIERS, which neither batch takes.
    ///
    /// **What SIM-2 did close**: the POOL half, at the solver
    /// (`mana_solver::solve_mana_payment_with_pool` — this function no longer compensates
    /// for a pool-blind solver, because the solver is not pool-blind), and the
    /// LAYER-RESOLUTION half, which was recorded as a theoretical gap about *granted*
    /// mana abilities and turned out to be live-wrong about **face-down** ones
    /// (CR 707.2 — see `mana_solver::gather_sources`). So OOS-M11-2 is now exactly its
    /// cost-modifier and CR 106.12 residue, and nothing else.
    ///
    /// # `{X}` is paid for — **OOS-M11-8 CLOSED for the human path** (S8, item 2) —
    /// **and now for the bot path too** (SIM-2)
    ///
    /// It was not, until S8. This function read the *printed* `mana_cost` and knew
    /// nothing about the announced `cast.x_value`, so a human casting `Fireball` with
    /// X = 3 got the base cost tapped for and the engine then refused the whole cast
    /// — observed by S7 as `422 "player does not have enough mana to pay the cost"`.
    /// CR 107.3 / 601.2b: X is announced at cast time and is part of the cost from
    /// that moment, so `x_value × mana_cost.x_count` generic is added to the cost
    /// **before** the solve.
    ///
    /// S8's close-out said "OOS-M11-8 CLOSED" of a fix that lived in this function
    /// only, while `advance()` kept its own `solve_mana_payment` call on the taxed
    /// printed cost — so a *bot* announcing X > 0 still tapped for the base cost and
    /// had its cast refused. SIM-2 makes `advance()` call this function, which closes
    /// the second half by construction rather than by a second copy of the arithmetic.
    /// (`RandomBot`/`HeuristicBot` announce `x_value: 0` today, so the bot half was
    /// latent, not live — recorded because the *claim* was whole and the fix was half.)
    ///
    /// `x_count`, not a bare `+ x_value`: a card printed `{X}{X}{R}` (Fireball is
    /// `{X}{R}`, but e.g. Rolling Thunder is not the only two-X card) has
    /// `x_count: 2` and costs 2X generic. The multiply is saturating so a hostile
    /// `x_value: u32::MAX` cannot overflow into a small cost that then looks payable.
    fn auto_tap_commands_for(&self, command: &Command, player: PlayerId) -> Option<Vec<Command>> {
        let Command::CastSpell(cast) = command else {
            return None;
        };
        // CR 903.8 / CR 601.2f (SIM-1): the PRINTED cost is not what the engine
        // charges. The shared helper applies commander tax when the card is being
        // cast from this player's command zone, so the offer gate
        // (`legal_actions::can_afford`), this human auto-tap and the bot auto-tap in
        // `advance()` cannot disagree about what has to be paid.
        // CR 702.157 (UI-2): the SQUAD payment is a cost INCREASE folded in here too --
        // `effective_cast_cost_with_additional` calls `effective_cast_cost` and adds
        // `cast.additional_costs`'s `AdditionalCost::Squad` count on top, so this site,
        // the offer gate and `advance()`'s bot auto-tap above cannot disagree about
        // what a Squad-paying cast actually charges.
        let mut cost = legal_actions::effective_cast_cost_with_additional(
            &self.state,
            player,
            cast.card,
            &cast.additional_costs,
        )?;
        // CR 107.3 / 601.2b — see the doc block above (OOS-M11-8).
        cost.generic = cost
            .generic
            .saturating_add(cast.x_value.saturating_mul(cost.x_count));
        // SIM-2 (triage F3): solve for the RESIDUAL — the cost that survives the
        // player's existing pool. `solve_mana_payment_with_pool` performs the
        // subtraction in `ManaPool::can_spend`'s own order, which is what the
        // `can_pay_cost` early return this replaces was checking: a pool that covers
        // the whole cost now yields a zero residual and therefore an EMPTY plan, the
        // same "tap nothing" outcome, reached by the general rule instead of a special
        // case. The special case was the bug — everything short of full coverage fell
        // through to a solve for the ENTIRE printed cost, so two floating mana and a
        // `{3}` cast tapped three more sources and CR 500.4 destroyed the float at the
        // step boundary.
        //
        // The flatten that used to be needed here (for `can_spend`'s debug-assert) has
        // moved inside the solver: `PipTracker::from_cost` applies the identical
        // all-default hybrid/Phyrexian plan that `action_to_command_with_params`'s
        // CastSpell arm uses, so there is one flattening on this path, not two.
        mana_solver::solve_mana_payment_with_pool(&self.state, player, &cost)
    }

    /// Apply a sequence of commands ATOMICALLY: every command is run against a clone
    /// of `self.state` in order, and `self.state`/`command_count`/`journal` are only
    /// committed if every command in the sequence succeeds. On the first failure,
    /// returns `LocalGameError::Rejected` and `self.state` is untouched — no partial
    /// application of e.g. a tap-then-cast sequence where the tap succeeded but the
    /// cast was rejected.
    fn apply_sequence(&mut self, commands: Vec<Command>) -> Result<Vec<GameEvent>, LocalGameError> {
        let mut working = self.state.clone();
        let mut all_events = Vec::new();
        let mut records = Vec::new();

        for command in commands {
            match process_command(working, command.clone()) {
                Ok((new_state, events)) => {
                    working = new_state;
                    all_events.extend(events.clone());
                    records.push(CommandRecord {
                        command,
                        events,
                        turn: working.turn().turn_number,
                    });
                }
                Err(e) => return Err(LocalGameError::Rejected(e)),
            }
        }
        // Folded only on the COMMIT path: `apply_sequence` is atomic, so a sequence that
        // failed part-way applied nothing and must census nothing either.
        for record in &records {
            self.mechanics.record(&record.events, record.turn);
        }

        if self.check_invariants {
            let new_violations = invariants::check_all(&working, Some(self.prev_turn));
            self.violations.extend(new_violations);
        }
        self.prev_turn = working.turn().turn_number;
        self.command_count += records.len() as u32;
        self.state = working;
        if self.limits.record_journal {
            self.journal.extend(records);
        }
        Ok(all_events)
    }

    /// Apply a command that is known-legal-to-attempt (bot seat auto-play or a
    /// structural pass). `track` mirrors the old `run_game`'s distinction between
    /// "clean" command execution (invariants checked, `prev_turn` advanced) and the
    /// untracked structural/fallback passes it never ran invariants against.
    fn apply_command(
        &mut self,
        command: Command,
        track: bool,
    ) -> Result<Vec<GameEvent>, GameStateError> {
        match process_command(self.state.clone(), command.clone()) {
            Ok((new_state, events)) => {
                if track {
                    if self.check_invariants {
                        let new_violations =
                            invariants::check_all(&new_state, Some(self.prev_turn));
                        self.violations.extend(new_violations);
                    }
                    self.prev_turn = new_state.turn().turn_number;
                }
                self.state = new_state;
                self.command_count += 1;
                self.mechanics
                    .record(&events, self.state.turn().turn_number);
                if self.limits.record_journal {
                    self.journal.push(CommandRecord {
                        command,
                        events: events.clone(),
                        turn: self.state.turn().turn_number,
                    });
                }
                Ok(events)
            }
            Err(e) => Err(e),
        }
    }

    /// SIM-5 fix (3): keep the engine's refusal instead of discarding it.
    ///
    /// `command` is the bot's chosen action, not the tapping plan around it — see
    /// [`RejectedCommand::command`].
    ///
    /// **The count is always incremented and never gated.** The RECORD is always
    /// retained too, as of PB-DX32 Stage 2 (SR-38, `OOS-SIM3-2`) — only its CAP
    /// depends on `LocalGameLimits::record_journal`: [`MAX_RETAINED_REJECTIONS`] (256)
    /// when the journal is on, or the much smaller [`MAX_SAMPLED_REJECTIONS`] (8) when
    /// it is off.
    ///
    /// **Correction of this doc's pre-Stage-2 account**: it used to say the record
    /// followed `record_journal` as a gate — true before Stage 2, false the moment
    /// SR-38 needed a sample from exactly the configuration that gate excluded:
    /// `mtg-fuzzer` runs with `record_journal: false` (`GameDriver` sets it so
    /// deliberately, to retain nothing per-game across thousands of parallel games —
    /// see that field's doc), and a rejection sample is precisely what
    /// `bin/fuzzer.rs`'s `print_sr38_summary` needs from that exact configuration. A
    /// full [`MAX_RETAINED_REJECTIONS`] (256) cap would be unsafe there for the same
    /// reason the journal itself stays off: `results` in `bin/fuzzer.rs` retains
    /// **every** game's `GameResult` for the whole run, so at `--games 1000` a 256-cap
    /// would retain up to 256,000 cloned `Command`s. Eight per game is a diagnosis
    /// sample, not the journal — [`Self::rejection_count`] stays uncapped and
    /// ungated, so truncation of the sample is always visible.
    fn record_rejection(&mut self, player: PlayerId, command: &Command, error: &LocalGameError) {
        self.rejection_count = self.rejection_count.saturating_add(1);
        let cap = if self.limits.record_journal {
            MAX_RETAINED_REJECTIONS
        } else {
            MAX_SAMPLED_REJECTIONS
        };
        if self.rejections.len() < cap {
            self.rejections.push(RejectedCommand {
                player,
                turn: self.state.turn().turn_number,
                command: command.clone(),
                error: format!("{error:?}"),
            });
        }
    }

    fn await_human(
        &mut self,
        player: PlayerId,
        kind: DecisionKind,
        actions: Vec<LegalAction>,
    ) -> AdvanceOutcome {
        self.decision_seq += 1;
        let decision = PendingDecision {
            seq: self.decision_seq,
            player,
            kind,
            actions,
        };
        self.pending = Some(decision.clone());
        AdvanceOutcome::AwaitingHuman(decision)
    }
}

/// Actions offered to a **human**-occupied seat and to no other, appended to whatever
/// `LegalActionProvider::legal_actions` already enumerated (M11-local S8, plan items 2
/// and 3).
///
/// # Why these live here and not in the provider
///
/// Both would be wrong in `StubProvider`, for two independent reasons:
///
/// 1. **Bots must not take them.** `Concede` (CR 104.3a) is a legal action at literally
///    every moment, so a `RandomBot` that saw it would concede roughly one game in `n`
///    on its first priority window. `legal_actions.rs` already says so in prose
///    ("Concede is intentionally omitted — bots should never auto-concede"); this is
///    that comment made structural rather than merely honoured.
/// 2. **The fuzzer must stay byte-comparable.** `RandomBot` picks an index into the
///    provider's list, so *appending* anything to it re-rolls every subsequent RNG draw
///    and changes what every recorded fuzz seed reproduces (plan §8 R11). Because this
///    augmentation happens strictly on the human branch of `advance()` — after the
///    `legal.is_empty()` auto-pass and after the bot branch was not taken — a game with
///    `human_seats` empty (the `GameDriver` / fuzzer case) never calls it at all.
///
/// # What is offered
///
/// * **`Concede` (CR 104.3a), unconditionally.** The engine's admission gate
///   (`rules::engine::process_command`) exempts `Command::Concede` from the
///   `BlockingDecision` block and validates only that the player exists, so this is
///   never an action the engine would refuse — which is the SR-38 standard the rest of
///   this crate holds itself to. `blocking` is therefore *not* consulted for it.
/// * **`OrderBlockers` (CR 509.2)**, one per attacker this player controls that has two
///   or more blockers, and only when `decision_is_forced` is `false`.
///
/// # What `decision_is_forced` actually means, and the one case where it over-suppresses
///
/// `advance()` passes `forced_kind.is_some()`, which is `true` for **two** situations,
/// not one, and they differ in how load-bearing the suppression is:
///
/// * a `rules::engine::BlockingDecision` is outstanding — here suppression is
///   **required**: the admission gate refuses `Command::OrderBlockers` outright, so
///   offering it would hand the human an action the engine rejects (the SR-38 standard);
/// * a **commander zone-change choice** (CR 903.9a) is pending — here suppression is a
///   **judgement**, not a necessity. That branch is not gated by the engine, so an
///   `OrderBlockers` issued during it would in fact be accepted. It is suppressed anyway
///   because `StubProvider` early-returns with only the two zone choices, and adding an
///   unrelated combat action to a two-option forced choice is noise; nothing is lost,
///   because answering the zone choice returns the human to a priority window where the
///   order is offered again (it is only withheld once the order has actually been set).
///
/// `Concede` is offered in **both**, because the gate exempts it by name and CR 104.3a
/// makes it legal at any time.
///
/// # The one moment a human is *not* offered Concede
///
/// `advance()` auto-passes when the provider returns an empty list, ahead of this
/// branch, and that ordering is deliberate and pre-existing (see its comment: stopping
/// earlier hands a human an empty action list and deadlocks with no safety valve). A
/// human therefore gets `Concede` at every decision they are actually asked to make,
/// which is every priority window they hold, not literally every instant CR 104.3a
/// permits. Widening that would mean giving `advance()` a way to interrupt itself,
/// which no HTTP request/response surface can use.
///
/// `pub` so a test — or a future host that drives `LocalGame` itself — can compose the
/// exact same list `advance()` hands out (`provider.legal_actions(..)` then this),
/// against a hand-built `GameState`. Driving a real game to a declared combat just to
/// see the CR 509.2 offer is not otherwise possible: `start_game` resets the turn
/// (`reset_turn_state`, `step = Untap`), so a fixture cannot simply *begin* in
/// `Step::DeclareBlockers` with a populated `CombatState`.
pub fn human_only_actions(
    state: &GameState,
    player: PlayerId,
    decision_is_forced: bool,
) -> Vec<LegalAction> {
    // CR 104.3a: "A player can concede the game at any time."
    let mut extra = vec![LegalAction::Concede];

    if !decision_is_forced {
        extra.extend(order_blocker_actions(state, player));
    }

    extra
}

/// CR 509.2: one `LegalAction::OrderBlockers` per attacker of `player`'s that is
/// blocked by two or more creatures.
///
/// Mirrors `combat::handle_order_blockers`' own preconditions rather than
/// re-deriving them, so the provider never offers something the engine rejects
/// (SR-38): the step must be `DeclareBlockers`, `player` must be
/// `combat.attacking_player`, and the attacker must be a declared attacker. The
/// single-blocker case is excluded because CR 509.2 only calls for an order when a
/// creature "is blocked by multiple creatures" — with one blocker there is exactly
/// one permutation and the command would be pure ceremony.
///
/// The candidate list is built by filtering `combat.blockers` in its own `OrdMap`
/// order, which is precisely the order `apply_combat_damage` falls back to when no
/// order was set. That is what makes an unanswered (or default-answered)
/// `OrderBlockers` a genuine no-op — see `params.rs`'s arm.
///
/// **An attacker whose order has already been set is not offered again**, and that is
/// a termination property, not a preference. `handle_order_blockers` accepts the same
/// command any number of times (it only `insert`s into
/// `combat.damage_assignment_order`) and answering it does not consume priority, so a
/// client that always takes the first non-pass action would be re-offered the identical
/// action forever. CR 509.2 orders blockers once, as a turn-based action, so there is
/// nothing to re-decide either.
fn order_blocker_actions(state: &GameState, player: PlayerId) -> Vec<LegalAction> {
    if state.turn().step != mtg_engine::Step::DeclareBlockers {
        return Vec::new();
    }
    let Some(combat) = state.combat().as_ref() else {
        return Vec::new();
    };
    if combat.attacking_player != player {
        return Vec::new();
    }
    combat
        .attackers
        .keys()
        .filter(|attacker| !combat.damage_assignment_order.contains_key(attacker))
        .filter_map(|attacker| {
            let blockers: Vec<mtg_engine::ObjectId> = combat
                .blockers
                .iter()
                .filter(|(_, blocked)| *blocked == attacker)
                .map(|(blocker, _)| *blocker)
                .collect();
            if blockers.len() < 2 {
                return None;
            }
            Some(LegalAction::OrderBlockers {
                attacker: *attacker,
                blockers,
            })
        })
        .collect()
}

/// CR 117.3 (priority), CR 508.1 / 509.1 (combat declarations) — classify what a
/// priority-holding human seat is actually being asked to do, from the legal actions
/// the provider already computed for them.
///
/// The `Mulligan` arm (CR 103.5) is **currently unreachable**: it needs
/// `turn_number == 0` (mirroring `StubProvider`'s own mulligan gate), but
/// `GameStateBuilder` defaults `turn_number` to 1 and nothing in the tree sets it to 0.
/// Mulligans also need per-player resolution, whereas `advance()` derives the acting
/// seat from `priority_holder` alone. Session 2 owns pregame setup and will make this
/// reachable; it is kept here so the enum and this classifier stay in step.
fn decision_kind_for(state: &GameState, actions: &[LegalAction]) -> DecisionKind {
    if state.turn().is_first_turn_of_game && state.turn().turn_number == 0 {
        return DecisionKind::Mulligan;
    }
    if actions
        .iter()
        .any(|a| matches!(a, LegalAction::DeclareAttackers { .. }))
    {
        return DecisionKind::DeclareAttackers;
    }
    if actions
        .iter()
        .any(|a| matches!(a, LegalAction::DeclareBlockers { .. }))
    {
        return DecisionKind::DeclareBlockers;
    }
    DecisionKind::Priority
}

/// Check if the game is over (one or zero players remain).
fn is_game_over(state: &GameState) -> bool {
    let alive = state.active_players();
    alive.len() <= 1
}

/// Find the winner (last player standing), if any.
fn find_winner(state: &GameState) -> Option<PlayerId> {
    let alive = state.active_players();
    if alive.len() == 1 {
        Some(alive[0])
    } else {
        None
    }
}

// The `command_player`/`test_command_player_extracts_acting_player` unit that used
// to live here is gone (Session 3, item 7): `submit` no longer accepts a pre-built
// `Command` at all, so there is nothing left to extract a player from. The
// cross-seat guarantee it existed to check is now structural — see `submit`'s doc
// comment above, and `crates/simulator/tests/local_game.rs`'s
// `test_local_game_submit_unknown_action_index_is_rejected`.
