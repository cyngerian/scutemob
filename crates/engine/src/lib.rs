pub mod cards;
pub mod effects;
pub mod rules;
pub mod state;
pub mod testing;

pub use cards::defs::all_cards;
pub use cards::{
    army_token_spec, clue_token_spec, food_token_spec, treasure_token_spec, AbilityDefinition,
    CardDefinition, CardRegistry, Condition, ContinuousEffectDef as CardContinuousEffectDef, Cost,
    Effect, EffectAmount, EffectTarget as CardEffectTarget, ForEachTarget, LibraryPosition,
    ModeSelection, PlayerTarget, SoulbondGrant, TargetController, TargetFilter, TargetRequirement,
    TimingRestriction, TokenSpec, TriggerCondition, TypeLine, ZoneTarget,
};

// Convenience re-exports of primary types
pub use state::types::ALL_CREATURE_TYPES;
pub use state::{
    AbilityInstance, AffinityTarget, AttackTarget, CardId, CardType, ChampionFilter,
    Characteristics, Color, CombatState, ContinuousEffect, CounterType, CumulativeUpkeepCost,
    DamageTargetFilter, ETBTriggerFilter, EffectDuration, EffectFilter, EffectId, EffectLayer,
    EnchantTarget, GameObject, GameState, GameStateBuilder, GameStateError, KeywordAbility,
    LandwalkType, LayerModification, ManaAbility, ManaColor, ManaCost, ManaPool, ObjectFilter,
    ObjectId, ObjectSpec, ObjectStatus, PendingZoneChange, Phase, PlayerBuilder, PlayerFilter,
    PlayerId, PlayerState, ProtectionQuality, ReplacementEffect, ReplacementId,
    ReplacementModification, ReplacementTrigger, SpellTarget, StackObject, StackObjectKind, Step,
    SubType, SuperType, Target, TriggerDoubler, TriggerDoublerFilter, TriggerEvent,
    TriggeredAbilityDef, TurnState, Zone, ZoneId, ZoneType,
};

pub use testing::replay_harness::{
    build_initial_state, card_name_to_id, enrich_spec_from_def, parse_counter_type, parse_step,
    translate_player_action,
};

pub use rules::commander::{
    apply_commander_tax, compute_color_identity, validate_deck, validate_partner_commanders,
    DeckValidationResult, DeckViolation,
};
pub use rules::engine::{process_command, start_game};
pub use rules::events::{CombatDamageAssignment, CombatDamageTarget};
pub use rules::layers::calculate_characteristics;
pub use rules::sba::check_and_apply_sbas;
pub use rules::{Command, GameEvent, LossReason};
pub use state::builder::register_commander_zone_replacements;
