//! Turn structure types: phases, steps, and turn state (CR 500-514).

use im::{OrdSet, Vector};
use serde::{Deserialize, Serialize};

use super::player::PlayerId;

/// Game phases (CR 500.1).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Phase {
    Beginning,
    PreCombatMain,
    Combat,
    PostCombatMain,
    Ending,
}

/// Steps within phases (CR 501-514).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Step {
    /// CR 502: Untap step (no priority granted)
    Untap,
    /// CR 503: Upkeep step
    Upkeep,
    /// CR 504: Draw step
    Draw,
    /// CR 505: Main phase (pre-combat)
    PreCombatMain,
    /// CR 507: Beginning of combat step
    BeginningOfCombat,
    /// CR 508: Declare attackers step
    DeclareAttackers,
    /// CR 509: Declare blockers step
    DeclareBlockers,
    /// CR 510: Combat damage step
    CombatDamage,
    /// CR 510: First/double strike combat damage step
    FirstStrikeDamage,
    /// CR 511: End of combat step
    EndOfCombat,
    /// CR 505: Main phase (post-combat)
    PostCombatMain,
    /// CR 512: End step
    End,
    /// CR 514: Cleanup step (normally no priority)
    Cleanup,
}

impl Step {
    /// Returns the phase this step belongs to.
    pub fn phase(&self) -> Phase {
        match self {
            Step::Untap | Step::Upkeep | Step::Draw => Phase::Beginning,
            Step::PreCombatMain => Phase::PreCombatMain,
            Step::BeginningOfCombat
            | Step::DeclareAttackers
            | Step::DeclareBlockers
            | Step::CombatDamage
            | Step::FirstStrikeDamage
            | Step::EndOfCombat => Phase::Combat,
            Step::PostCombatMain => Phase::PostCombatMain,
            Step::End | Step::Cleanup => Phase::Ending,
        }
    }

    /// Whether players normally receive priority in this step.
    /// CR 502.3: No player receives priority during the untap step.
    /// CR 514.3: Normally no priority during cleanup.
    pub fn has_priority(&self) -> bool {
        !matches!(self, Step::Untap | Step::Cleanup)
    }
}

/// State of the current turn (CR 500).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TurnState {
    pub phase: Phase,
    pub step: Step,
    pub active_player: PlayerId,
    pub priority_holder: Option<PlayerId>,
    /// Players who have passed priority in succession since the last action.
    pub players_passed: OrdSet<PlayerId>,
    pub turn_number: u32,
    /// Player order for the current game (clockwise from starting player).
    pub turn_order: Vector<PlayerId>,
    /// Queue of players who will get extra turns (LIFO — most recently added goes first).
    pub extra_turns: Vector<PlayerId>,
}
