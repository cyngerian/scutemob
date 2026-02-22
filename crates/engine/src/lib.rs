pub mod cards;
pub mod effects;
pub mod rules;
pub mod state;
pub mod testing;

// Convenience re-exports of primary types
pub use state::{
    AbilityInstance, CardId, CardType, Characteristics, Color, CounterType, GameObject, GameState,
    GameStateBuilder, GameStateError, KeywordAbility, ManaAbility, ManaColor, ManaCost, ManaPool,
    ObjectId, ObjectSpec, ObjectStatus, Phase, PlayerBuilder, PlayerId, PlayerState, SpellTarget,
    StackObject, StackObjectKind, Step, SubType, SuperType, Target, TurnState, Zone, ZoneId,
    ZoneType,
};

pub use rules::engine::{process_command, start_game};
pub use rules::{Command, GameEvent, LossReason};
