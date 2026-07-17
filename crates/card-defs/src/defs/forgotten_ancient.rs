// Forgotten Ancient — {3}{G}, Creature — Elemental 0/3
// Whenever a player casts a spell, you may put a +1/+1 counter on this creature.
// At the beginning of your upkeep, you may move any number of +1/+1 counters from this
// creature onto other creatures.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("forgotten-ancient"),
        name: "Forgotten Ancient".to_string(),
        mana_cost: Some(ManaCost { generic: 3, green: 1, ..Default::default() }),
        types: creature_types(&["Elemental"]),
        oracle_text: "Whenever a player casts a spell, you may put a +1/+1 counter on this creature.\nAt the beginning of your upkeep, you may move any number of +1/+1 counters from this creature onto other creatures.".to_string(),
        power: Some(0),
        toughness: Some(3),
        abilities: vec![
            // TODO: DSL gap — "Whenever a player casts a spell" trigger condition.
            // WheneverASpellIsCast exists but may not match "a player" scope.
            // TODO: DSL gap — "move any number of +1/+1 counters" requires counter
            // movement with player choice (M10 player choice).
        ],
        completeness: Completeness::partial("Blocked on the upkeep ability: 'move any number of +1/+1 counters from this creature onto other creatures' has no Effect expression (no counter-move-with-player-choice primitive; AddCounter/RemoveCounter/Proliferate do not compose to it), and both abilities are 'you may' (no optional-effect wrapper; Effect::Choose is non-interactive at effects/mod.rs:3190). NOT blocked on the trigger: 'whenever a player casts a spell' is expressible as WheneverYouCastSpell (card_definition.rs:3117) + WheneverOpponentCastsSpell together."),
        ..Default::default()
    }
}
