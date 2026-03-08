//! Card definition types and keyword ability implementations.
//!
//! A `CardDefinition` encodes what a card does in a structured form the engine
//! can execute. It is separate from `Characteristics` (the runtime game-state
//! representation). Card definitions are static data loaded once at startup
//! into a `CardRegistry`, which `GameState` holds as `Arc<CardRegistry>`.
//!
//! See architecture doc Section 3.7 for the full design.

pub mod card_definition;
pub mod defs;
pub mod helpers;
pub mod registry;

pub use card_definition::Effect;
pub use card_definition::{
    army_token_spec, blood_token_spec, clue_token_spec, food_token_spec, treasure_token_spec,
    zombie_decayed_token_spec,
    AbilityDefinition, CardDefinition, Condition, ContinuousEffectDef, Cost, EffectAmount,
    EffectTarget, ForEachTarget, LibraryPosition, ModeSelection, PlayerTarget, SoulbondGrant,
    TargetController, TargetFilter, TargetRequirement, TimingRestriction, TokenSpec,
    TriggerCondition, TypeLine, ZoneTarget,
};
pub use registry::CardRegistry;
