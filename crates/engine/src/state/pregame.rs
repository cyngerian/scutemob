//! CR 103.4-103.6: the pregame procedure, as a piece of state the engine can consult.
//!
//! # Why this exists (`OOS-DX2-4`, PB-DX18)
//!
//! `Command::TakeMulligan` and `Command::KeepHand` are the engine's only pregame
//! commands, and until PB-DX18 `rules::engine::process_command` gated them on
//! `validate_player_exists` **and nothing else**. There was no pregame state anywhere
//! to consult: `rg -i mulligan crates/card-types/src/state` found only
//! `PlayerState::mulligan_count`, which is a *counter*, not a *phase* — it never
//! resets, so it cannot distinguish "before the game began" from "turn 14".
//!
//! The consequence was not cosmetic. A mid-game `TakeMulligan` runs
//! `commander::handle_take_mulligan` verbatim: it shuffles the sender's whole hand
//! into their library, really permutes the library, and draws seven.
//!
//! # Why ONE field carries BOTH CR 103.5 properties
//!
//! CR 103.5 states two separate restrictions, and a bare `game_started: bool` would
//! close only the first:
//!
//! 1. The mulligan procedure happens *before* the game begins — so once
//!    [`crate::rules::engine::start_game`] has run, neither command is legal.
//! 2. *"Once a player chooses not to take a mulligan, the remaining cards become that
//!    player's opening hand, and **that player may not take any further mulligans**."*
//!    — a **per-player** termination, explicit CR text rather than a nicety.
//!
//! [`PregamePhase::Mulligans`] carries the set of players who have already kept, so
//! both questions are answered by reading one field. That keeps this batch at **two**
//! new stored fields (this one and `PlayerState::miracle_pending`) and therefore at
//! **one** `HASH_SCHEMA_VERSION` bump.
//!
//! # Not on the wire
//!
//! This type is reachable only from `GameState`, which
//! `crates/engine/tests/core/protocol_schema.rs`'s `CLOSURE_MUST_NOT_CONTAIN` keeps
//! out of the `Command` / `GameEvent` closure. It moves `HASH_SCHEMA_VERSION` and not
//! `PROTOCOL_VERSION` — the PB-DX21 precedent (`CombatState::attackers_declared`).

use imbl::OrdSet;
use serde::{Deserialize, Serialize};

use mtg_card_types::state::PlayerId;

/// CR 103.4-103.6: where a game is in the pregame procedure.
///
/// [`Default`] is [`PregamePhase::Mulligans`] with nobody having kept — the state a
/// freshly built, not-yet-started game is in. `GameStateBuilder` sets it explicitly;
/// the `Default` matters for `#[serde(default)]` on a state serialized before this
/// field existed. Such a state cannot be loaded across a `HASH_SCHEMA_VERSION` bump
/// anyway (SR-8's strict lockstep), so the choice is documented rather than defended:
/// it matches the builder, which is the documented constructor.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PregamePhase {
    /// The game has not begun. `Command::TakeMulligan` / `Command::KeepHand` are legal
    /// for every player **not** in `kept`.
    Mulligans {
        /// CR 103.5: players who have already declared a keep. Each one is finished
        /// with the mulligan procedure and may take no further mulligans.
        kept: OrdSet<PlayerId>,
    },
    /// [`crate::rules::engine::start_game`] has run. Neither pregame command is legal.
    GameStarted,
}

impl PregamePhase {
    /// CR 103.5 — may `player` still send `TakeMulligan` / `KeepHand`?
    ///
    /// False once the game has started, and false for a player who has already kept.
    pub fn may_mulligan(&self, player: PlayerId) -> bool {
        match self {
            PregamePhase::Mulligans { kept } => !kept.contains(&player),
            PregamePhase::GameStarted => false,
        }
    }

    /// True while the game has not begun, regardless of who has kept.
    pub fn is_pregame(&self) -> bool {
        matches!(self, PregamePhase::Mulligans { .. })
    }
}

impl Default for PregamePhase {
    fn default() -> Self {
        PregamePhase::Mulligans {
            kept: OrdSet::new(),
        }
    }
}
