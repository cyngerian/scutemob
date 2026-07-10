//! The pure-data half of the game state model.
//!
//! Every type here is a value: it describes an object, a filter, a cost, or a
//! characteristic, and it can be constructed by a card definition without a
//! game in progress. `GameState` itself — and everything that reads or mutates
//! a live game — lives in `mtg-engine`'s `state` module, which re-exports these
//! modules so that `crate::state::…` paths resolve identically on both sides of
//! the crate boundary.
//!
//! Nothing in this module may reference `GameState`.
pub mod combat;
pub mod continuous_effect;
pub mod dungeon;
pub mod game_object;
pub mod player;
pub mod replacement_effect;
pub mod stack;
pub mod stubs;
pub mod targeting;
pub mod types;
pub mod zone;
pub use combat::{AttackTarget, CombatState};
pub use continuous_effect::{
    ContinuousEffect, EffectDuration, EffectFilter, EffectId, EffectLayer, LayerModification,
};
pub use dungeon::{get_dungeon, DungeonDef, DungeonId, DungeonState, RoomDef, RoomIndex};
pub use game_object::{
    AbilityInstance, ActivatedAbility, ActivationCost, Characteristics, DeathTriggerFilter,
    Designations, ETBTriggerFilter, GameObject, HybridMana, HybridManaPayment, InterveningIf,
    ManaAbility, ManaCost, MergedComponent, ObjectId, ObjectStatus, PhyrexianMana, SacrificeFilter,
    TriggerEvent, TriggeredAbilityDef,
};
pub use player::{CardId, ManaPool, PlayerId, PlayerState};
pub use replacement_effect::{
    DamageTargetFilter, ObjectFilter, PendingZoneChange, PlayerFilter, ReplacementEffect,
    ReplacementId, ReplacementModification, ReplacementTrigger,
};
pub use stack::{StackObject, StackObjectKind, TriggerData, UpkeepCostKind};
pub use stubs::{
    ActiveRestriction, AdditionalLandPlaySource, DelayedTrigger, ETBSuppressFilter, ETBSuppressor,
    FlashGrant, FlashGrantFilter, GameRestriction, PendingTrigger, PlayFromGraveyardPermission,
    PlayFromTopFilter, PlayFromTopPermission, TriggerDoubler, TriggerDoublerFilter,
};
pub use targeting::{SpellTarget, Target};
pub use types::{
    AdditionalCost, AffinityTarget, AltCostKind, BlockingExceptionFilter, CardType, ChampionFilter,
    Color, CounterType, CumulativeUpkeepCost, DayNight, EnchantControllerConstraint, EnchantFilter,
    EnchantTarget, FaceDownKind, KeywordAbility, LandwalkType, ManaColor, ProtectionQuality,
    SubType, SuperType, TurnFaceUpMethod,
};
pub use zone::{Zone, ZoneId, ZoneType};
