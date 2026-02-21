//! Player commands — the only way to change game state.
//!
//! All player actions are Commands. There is no way to change game state
//! except through the Command enum. This enables networking, replay, and
//! deterministic testing.

use serde::{Deserialize, Serialize};

use crate::state::player::PlayerId;

/// A player action submitted to the engine.
///
/// In M2, only `PassPriority` and `Concede` are functional.
/// Future milestones add the remaining variants.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Command {
    /// The player passes priority (CR 117.3d).
    PassPriority { player: PlayerId },

    /// The player concedes the game (CR 104.3a).
    Concede { player: PlayerId },
    // --- Future milestones ---
    // CastSpell { player, card, targets, modes, costs }       // M3
    // ActivateAbility { player, source, ability_index, ... }   // M3
    // PlayLand { player, card }                                // M3
    // DeclareAttackers { player, attackers }                   // M6
    // DeclareBlockers { player, blockers }                     // M6
    // OrderBlockers { player, ordering }                       // M6
    // MakeChoice { player, choice }                            // M3
    // PayCost { player, cost }                                 // M3
}
