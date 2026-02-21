pub mod cards;
pub mod effects;
pub mod rules;
pub mod state;

// Convenience re-exports of primary types
pub use state::{
    AbilityInstance, CardId, CardType, Characteristics, Color, CounterType, GameObject, GameState,
    GameStateBuilder, GameStateError, KeywordAbility, ManaAbility, ManaColor, ManaCost, ManaPool,
    ObjectId, ObjectSpec, ObjectStatus, Phase, PlayerBuilder, PlayerId, PlayerState, StackObject,
    StackObjectKind, Step, SubType, SuperType, TurnState, Zone, ZoneId, ZoneType,
};

pub use rules::engine::process_command;
pub use rules::{Command, GameEvent, LossReason};
