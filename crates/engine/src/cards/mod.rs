//! Card definition types and the card corpus, re-exported from their own crates.
//!
//! A `CardDefinition` encodes what a card does in a structured form the engine
//! can execute. It is separate from `Characteristics` (the runtime game-state
//! representation). Card definitions are static data loaded once at startup
//! into a `CardRegistry`, which `GameState` holds as `Arc<CardRegistry>`.
//!
//! Neither the DSL types nor the 1,749 definitions live in this crate any more:
//! the types are in `mtg-card-types` and the defs in `mtg-card-defs`, both
//! *below* the engine in the dependency graph. Editing a rule or an effect
//! therefore cannot force a recompile of the card corpus. This module keeps the
//! `crate::cards::…` paths the rest of the engine is written against.
//!
//! See architecture doc Section 3.7 for the full design.
pub use mtg_card_types::cards::{card_definition, helpers, registry};

/// The card corpus: one `CardDefinition` per file in `mtg-card-defs`, plus the
/// generated `all_cards()` collector. Tests reach individual cards through it as
/// `cards::defs::<card_name>::card()`, so the whole module is re-exported, not
/// just the collector.
pub use mtg_card_defs::defs;

pub use card_definition::Effect;
pub use card_definition::{
    army_token_spec, blood_token_spec, clue_token_spec, food_token_spec, treasure_token_spec,
    zombie_decayed_token_spec, AbilityDefinition, AltCastDetails, CardDefinition, CardFace,
    Completeness, Condition, ContinuousEffectDef, Cost, CostModifierScope, CraftMaterials,
    EffectAmount, EffectTarget, ForEachTarget, LibraryPosition, LoyaltyCost, MeldPair,
    ModeSelection, PlayerTarget, SelfActivatedCostReduction, SelfCostReduction, SoulbondGrant,
    SpellAdditionalCost, SpellCostFilter, SpellCostModifier, TargetController, TargetFilter,
    TargetOwner, TargetRequirement, TimingRestriction, TokenSpec, TriggerCondition, TypeLine,
    ZoneTarget,
};
pub use registry::{CardRegistry, RegistryError};
