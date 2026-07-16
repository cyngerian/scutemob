pub mod cards;
pub mod effects;
pub mod rules;
pub mod state;
pub mod testing;
pub use cards::defs::all_cards;
pub use cards::{
    army_token_spec, blood_token_spec, clue_token_spec, food_token_spec, treasure_token_spec,
    zombie_decayed_token_spec, AbilityDefinition, AltCastDetails, CardDefinition, CardFace,
    CardRegistry, Completeness, Condition, ContinuousEffectDef as CardContinuousEffectDef, Cost,
    CostModifierScope, CraftMaterials, Effect, EffectAmount, EffectTarget as CardEffectTarget,
    ForEachTarget, LibraryPosition, LoyaltyCost, MeldPair, ModeSelection, PlayerTarget,
    RegistryError, SelfActivatedCostReduction, SelfCostReduction, SoulbondGrant,
    SpellAdditionalCost, SpellCostFilter, SpellCostModifier, TargetController, TargetFilter,
    TargetRequirement, TimingRestriction, TokenSpec, TriggerCondition, TypeLine, ZoneTarget,
};
// Convenience re-exports of primary types
pub use rules::commander::{
    apply_commander_tax, compute_color_identity, validate_deck, validate_partner_commanders,
    DeckValidationResult, DeckViolation,
};
pub use rules::engine::{
    handle_ring_tempts_you, handle_venture_into_dungeon, process_command, start_game,
    start_game_allowing_incomplete,
};
pub use rules::events::{CombatDamageAssignment, CombatDamageTarget};
pub use rules::layers::calculate_characteristics;
pub use rules::protocol::{
    decode, decode_replay_log, encode, encode_replay_log, Envelope, ProtocolEpoch, ProtocolError,
    ReplayLog, PROTOCOL_HISTORY, PROTOCOL_SCHEMA_FINGERPRINT, PROTOCOL_VERSION,
};
pub use rules::sba::check_and_apply_sbas;
pub use rules::{CastSpellData, Command, GameEvent, LossReason};
pub use state::builder::register_commander_zone_replacements;
pub use state::hash::{HashSchemaEpoch, HASH_SCHEMA_HISTORY, HASH_SCHEMA_VERSION};
pub use state::types::ALL_CREATURE_TYPES;
pub use state::{get_dungeon, DungeonDef, DungeonId, DungeonState, RoomDef, RoomIndex};
pub use state::{
    AbilityInstance, AdditionalCost, AffinityTarget, AltCostKind, AttackTarget,
    BlockingExceptionFilter, CardId, CardType, ChampionFilter, Characteristics, Color, CombatState,
    ContinuousEffect, CounterType, CumulativeUpkeepCost, DamageTargetFilter, DayNight,
    DeathTriggerFilter, Designations, ETBSuppressFilter, ETBSuppressor, ETBTriggerFilter,
    EffectDuration, EffectFilter, EffectId, EffectLayer, EnchantControllerConstraint,
    EnchantFilter, EnchantTarget, FaceDownKind, FlashGrant, FlashGrantFilter, GameObject,
    GameRestriction, GameState, GameStateBuilder, GameStateError, HybridMana, HybridManaPayment,
    KeywordAbility, LandwalkType, LayerModification, ManaAbility, ManaColor, ManaCost, ManaPool,
    MergedComponent, ObjectFilter, ObjectId, ObjectSpec, ObjectStatus, PendingZoneChange, Phase,
    PhyrexianMana, PlayFromGraveyardPermission, PlayFromTopFilter, PlayFromTopPermission,
    PlayerBuilder, PlayerFilter, PlayerId, PlayerState, ProtectionQuality, ReplacementEffect,
    ReplacementId, ReplacementModification, ReplacementTrigger, SpellTarget, StackObject,
    StackObjectKind, Step, SubType, SuperType, Target, TriggerData, TriggerDoubler,
    TriggerDoublerFilter, TriggerEvent, TriggeredAbilityDef, TurnFaceUpMethod, TurnState,
    UpkeepCostKind, Zone, ZoneId, ZoneType,
};
pub use testing::replay_harness::{
    build_initial_state, build_initial_state_checked, card_name_to_id, enrich_spec_from_def,
    parse_counter_type, parse_step, translate_player_action,
};

// Test-only re-exports — not part of the runtime API.
#[doc(hidden)]
pub use rules::replacement::bind_object_filter;
