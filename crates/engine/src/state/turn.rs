//! Turn structure types: phases, steps, and turn state (CR 500-514).
use super::player::PlayerId;
use im::{OrdSet, Vector};
use serde::{Deserialize, Serialize};
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
impl Step {
    /// Returns the next step in normal turn order.
    ///
    /// Skips FirstStrikeDamage (conditionally inserted in M6).
    /// Returns None after Cleanup (end of turn).
    pub fn next(self) -> Option<Step> {
        match self {
            Step::Untap => Some(Step::Upkeep),
            Step::Upkeep => Some(Step::Draw),
            Step::Draw => Some(Step::PreCombatMain),
            Step::PreCombatMain => Some(Step::BeginningOfCombat),
            Step::BeginningOfCombat => Some(Step::DeclareAttackers),
            Step::DeclareAttackers => Some(Step::DeclareBlockers),
            Step::DeclareBlockers => Some(Step::CombatDamage),
            Step::CombatDamage => Some(Step::EndOfCombat),
            Step::FirstStrikeDamage => Some(Step::CombatDamage),
            Step::EndOfCombat => Some(Step::PostCombatMain),
            Step::PostCombatMain => Some(Step::End),
            Step::End => Some(Step::Cleanup),
            Step::Cleanup => None,
        }
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
    /// Queue of players who will get extra turns (LIFO -- most recently added goes first).
    pub extra_turns: Vector<PlayerId>,
    /// Queue of additional phases to insert into the turn (CR 500.8 LIFO ordering).
    ///
    /// Each entry is Phase::Combat or Phase::PostCombatMain. When EndOfCombat
    /// or PostCombatMain transitions, the engine pops the back entry and redirects
    /// the step accordingly. LIFO: most recently created phase occurs first.
    pub additional_phases: Vector<Phase>,
    /// Whether we are currently in an extra combat phase.
    pub in_extra_combat: bool,
    /// Whether this is the very first turn of the game (first player skips draw).
    pub is_first_turn_of_game: bool,
    /// The active player of the last regular (non-extra) turn.
    /// Used to resume normal turn order after extra turns.
    pub last_regular_active: PlayerId,
    /// CR 514.3a: counts how many SBA-check rounds have occurred during the
    /// current cleanup step.  Resets to 0 at the start of each new turn.
    /// Acts as a safety guard against infinite cleanup loops (max 100).
    #[serde(default)]
    pub cleanup_sba_rounds: u32,
}
