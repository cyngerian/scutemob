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
    /// CR 101.6: If true, this spell can't be countered by spells or abilities.
    /// Set from the card definition at cast time.
    #[serde(default)]
    pub cant_be_countered: bool,
    /// CR 707.10: If true, this is a copy of a spell — it has no physical card
    /// object to move when it resolves. Copies execute the spell's effect but do
    /// NOT move a source card to graveyard or battlefield at resolution.
    ///
    /// Copies are NOT cast (no "cast" triggers), are not affected by effects that
    /// care about casting, and do not go to graveyard when they finish resolving.
    #[serde(default)]
    pub is_copy: bool,
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
        /// Effect captured at activation time (CR 602.2). Required when the source
        /// is sacrificed as a cost and will not exist at resolution time (CR 608.3b).
        /// Boxed to keep the enum variant size manageable (clippy::large_enum_variant).
        #[serde(default)]
        embedded_effect: Option<Box<crate::cards::card_definition::Effect>>,
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

    /// CR 702.85a: Cascade triggered ability on the stack.
    ///
    /// Cascade is a triggered ability that triggers when the cascade spell is
    /// cast. When this trigger resolves, the cascade procedure runs: exile
    /// cards until a qualifying nonland card with mana value strictly less than
    /// `spell_mana_value` is found, cast it for free, put the rest on the
    /// bottom of the library.
    ///
    /// `spell_mana_value` is captured at trigger time (when the cascade spell
    /// is cast) because continuous effects could change the mana value later.
    CascadeTrigger {
        source_object: ObjectId,
        spell_mana_value: u32,
    },

    /// CR 702.40a: Storm triggered ability on the stack.
    ///
    /// Storm is a triggered ability that triggers when the storm spell is cast.
    /// When this trigger resolves, `storm_count` copies of the original spell
    /// are created on the stack. Storm copies are NOT cast (CR 702.40c) —
    /// they do not trigger "whenever you cast a spell."
    ///
    /// `storm_count` and `original_stack_id` are captured at trigger time
    /// (when the storm spell is cast).
    StormTrigger {
        source_object: ObjectId,
        original_stack_id: ObjectId,
        storm_count: u32,
    },
}
