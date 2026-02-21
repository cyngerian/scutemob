//! Error types for game state operations.

use super::game_object::ObjectId;
use super::player::PlayerId;
use super::zone::ZoneId;

/// Errors that can occur during game state operations.
#[derive(Debug, Clone, thiserror::Error)]
pub enum GameStateError {
    #[error("object not found: {0:?}")]
    ObjectNotFound(ObjectId),

    #[error("player not found: {0:?}")]
    PlayerNotFound(PlayerId),

    #[error("zone not found: {0:?}")]
    ZoneNotFound(ZoneId),

    #[error("object {0:?} is not in zone {1:?}")]
    ObjectNotInZone(ObjectId, ZoneId),

    #[error("invalid zone transition from {from:?} to {to:?}")]
    InvalidZoneTransition { from: ZoneId, to: ZoneId },

    #[error("not the priority holder: expected {expected:?}, got {actual:?}")]
    NotPriorityHolder {
        expected: Option<PlayerId>,
        actual: PlayerId,
    },

    #[error("game is already over")]
    GameAlreadyOver,

    #[error("player {0:?} has been eliminated")]
    PlayerEliminated(PlayerId),

    #[error("no active players remaining")]
    NoActivePlayers,

    #[error("player {0:?} tried to draw from empty library")]
    LibraryEmpty(PlayerId),

    #[error("invalid command: {0}")]
    InvalidCommand(String),
}
