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

    #[error("object {0:?} is not on the battlefield")]
    ObjectNotOnBattlefield(ObjectId),

    #[error("player {player:?} does not control object {object_id:?}")]
    NotController {
        player: PlayerId,
        object_id: ObjectId,
    },

    #[error("permanent {0:?} is already tapped")]
    PermanentAlreadyTapped(ObjectId),

    #[error("player {0:?} has no land plays remaining this turn")]
    NoLandPlaysRemaining(PlayerId),

    #[error("object {object_id:?} has no mana ability at index {index}")]
    InvalidAbilityIndex { object_id: ObjectId, index: usize },

    #[error("this action requires a main phase")]
    NotMainPhase,

    #[error("this action requires an empty stack")]
    StackNotEmpty,

    #[error("player does not have enough mana to pay the cost")]
    InsufficientMana,

    #[error("invalid target: {0}")]
    InvalidTarget(String),

    #[error("invalid attack target: {0}")]
    InvalidAttackTarget(String),

    #[error("creature {0:?} is already blocking an attacker")]
    DuplicateBlocker(ObjectId),

    #[error("blocker order is incomplete: {provided} entries provided, {required} blockers blocking this attacker")]
    IncompleteBlockerOrder { provided: usize, required: usize },

    #[error("creature {blocker:?} cannot block attacker {attacker:?}: attacker is not targeting this player")]
    CrossPlayerBlock {
        blocker: ObjectId,
        attacker: ObjectId,
    },

    #[error("player {0:?} has already declared blockers this combat phase")]
    AlreadyDeclaredBlockers(PlayerId),
}
