//! DSL prelude for card definition files.
//!
//! Each per-card file imports `use crate::cards::helpers::*;` to get all
//! types and helper functions needed to build a `CardDefinition`.

pub use im::OrdSet;

pub use crate::state::player::PlayerId;
pub use crate::state::{
    AffinityTarget, CardId, CardType, Color, CounterType, EnchantTarget, KeywordAbility,
    LandwalkType, ManaCost, ManaPool, SubType, SuperType,
};

pub use super::card_definition::{
    food_token_spec, treasure_token_spec, AbilityDefinition, CardDefinition, Condition,
    ContinuousEffectDef, Cost, Effect, EffectAmount, EffectTarget, ForEachTarget,
    LibraryPosition, PlayerTarget, TargetController, TargetFilter, TargetRequirement,
    TimingRestriction, TokenSpec, TriggerCondition, TypeLine, ZoneTarget,
};
pub use crate::state::continuous_effect::{
    EffectDuration, EffectFilter, EffectLayer, LayerModification,
};
pub use crate::state::replacement_effect::{
    ObjectFilter, ReplacementModification, ReplacementTrigger,
};
pub use crate::state::zone::ZoneType;

// ── Helper functions ─────────────────────────────────────────────────────────

pub fn cid(s: &str) -> CardId {
    CardId(s.to_string())
}

pub fn types(card_types: &[CardType]) -> TypeLine {
    TypeLine {
        card_types: card_types.iter().copied().collect(),
        ..Default::default()
    }
}

pub fn supertypes(supers: &[SuperType], card_types: &[CardType]) -> TypeLine {
    TypeLine {
        supertypes: supers.iter().copied().collect(),
        card_types: card_types.iter().copied().collect(),
        ..Default::default()
    }
}

pub fn types_sub(card_types: &[CardType], subtypes: &[&str]) -> TypeLine {
    TypeLine {
        card_types: card_types.iter().copied().collect(),
        subtypes: subtypes.iter().map(|s| SubType(s.to_string())).collect(),
        ..Default::default()
    }
}

pub fn full_types(supers: &[SuperType], card_types: &[CardType], subtypes: &[&str]) -> TypeLine {
    TypeLine {
        supertypes: supers.iter().copied().collect(),
        card_types: card_types.iter().copied().collect(),
        subtypes: subtypes.iter().map(|s| SubType(s.to_string())).collect(),
    }
}

pub fn creature_types(subtypes: &[&str]) -> TypeLine {
    types_sub(&[CardType::Creature], subtypes)
}

pub fn mana_pool(c: u32, u: u32, b: u32, r: u32, g: u32, colorless: u32) -> ManaPool {
    ManaPool {
        white: c,
        blue: u,
        black: b,
        red: r,
        green: g,
        colorless,
    }
}

pub fn basic_land_filter() -> TargetFilter {
    TargetFilter {
        basic: true,
        has_card_type: Some(CardType::Land),
        ..Default::default()
    }
}
