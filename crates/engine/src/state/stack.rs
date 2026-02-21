//! Stack object types: spells and abilities on the stack (CR 405).
//!
//! The stack is an ordered zone (LIFO). When a spell is cast or an ability
//! is activated/triggered, a StackObject is pushed onto the stack.
//! Resolution pops the top object off the stack (CR 608.1).
//!
//! For spells, the corresponding card has moved to `ZoneId::Stack` and appears
//! as a `GameObject` there. For abilities, no corresponding `GameObject` exists
//! in the Stack zone — the `StackObject` alone represents the ability on the stack.

use serde::{Deserialize, Serialize};

use super::game_object::ObjectId;
use super::player::PlayerId;
use super::targeting::SpellTarget;

/// An object on the stack: a spell, activated ability, or triggered ability
/// (CR 405.1).
///
/// Stack objects are ordered LIFO — the last one pushed is the first to resolve.
/// Use `GameState.stack_objects` to access the stack (index 0 = bottom, last = top).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StackObject {
    /// Unique identifier for this stack item.
    pub id: ObjectId,
    /// The player who controls this spell or ability (CR 108.4).
    pub controller: PlayerId,
    /// What kind of object this is (spell, activated ability, or triggered ability).
    pub kind: StackObjectKind,
    /// Targets announced at cast time (CR 601.2c). Empty for non-targeting spells.
    /// Validated again at resolution for the fizzle rule (CR 608.2b).
    pub targets: Vec<SpellTarget>,
}

/// The kind of object on the stack.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum StackObjectKind {
    /// A spell being cast (CR 601).
    ///
    /// The `source_object` is the `ObjectId` of the card now in `ZoneId::Stack`.
    /// When the spell resolves, the card moves to the graveyard (or stays in play
    /// for permanents).
    Spell { source_object: ObjectId },

    /// An activated ability (CR 602).
    ///
    /// The `source_object` remains in whatever zone it is in — the source does
    /// NOT move to the stack when an activated ability is put on the stack.
    /// `ability_index` identifies which ability on the source object this is.
    ActivatedAbility {
        source_object: ObjectId,
        ability_index: usize,
    },

    /// A triggered ability (CR 603).
    ///
    /// The `source_object` may be in any zone (it triggered from wherever it
    /// was when the trigger condition was met). `ability_index` identifies
    /// which ability triggered.
    TriggeredAbility {
        source_object: ObjectId,
        ability_index: usize,
    },
}
