pub mod cards;
pub mod effects;
pub mod rules;
pub mod state;
pub mod testing;

pub use cards::definitions::all_cards;
pub use cards::{
    AbilityDefinition, CardDefinition, CardRegistry, Condition,
    ContinuousEffectDef as CardContinuousEffectDef, Cost, Effect, EffectAmount,
    EffectTarget as CardEffectTarget, ForEachTarget, LibraryPosition, ModeSelection, PlayerTarget,
    TargetController, TargetFilter, TargetRequirement, TimingRestriction, TokenSpec,
    TriggerCondition, TypeLine, ZoneTarget,
};

// Convenience re-exports of primary types
pub use state::{
    AbilityInstance, AttackTarget, CardId, CardType, Characteristics, Color, CombatState,
    ContinuousEffect, CounterType, EffectDuration, EffectFilter, EffectId, EffectLayer, GameObject,
    GameState, GameStateBuilder, GameStateError, KeywordAbility, LayerModification, ManaAbility,
    ManaColor, ManaCost, ManaPool, ObjectId, ObjectSpec, ObjectStatus, Phase, PlayerBuilder,
    PlayerId, PlayerState, SpellTarget, StackObject, StackObjectKind, Step, SubType, SuperType,
    Target, TriggerEvent, TriggeredAbilityDef, TurnState, Zone, ZoneId, ZoneType,
};

pub use rules::engine::{process_command, start_game};
pub use rules::events::{CombatDamageAssignment, CombatDamageTarget};
pub use rules::layers::calculate_characteristics;
pub use rules::sba::check_and_apply_sbas;
pub use rules::{Command, GameEvent, LossReason};
