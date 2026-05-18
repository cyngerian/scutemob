// Morbid Opportunist — {2}{B}, Creature — Human Rogue 1/3
// Whenever one or more other creatures die, draw a card. This ability triggers
// only once each turn.
//
// BLOCKED: The once-per-turn limiter on triggered abilities ("This ability triggers
// only once each turn") is not expressible in the DSL. `once_per_turn: bool` exists
// only on AbilityDefinition::Activated, not on Triggered. Without the limiter the
// trigger would fire on every creature death, producing wrong game state (draws
// multiple cards per turn). Abilities omitted per W5 policy.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("morbid-opportunist"),
        name: "Morbid Opportunist".to_string(),
        mana_cost: Some(ManaCost { generic: 2, black: 1, ..Default::default() }),
        types: creature_types(&["Human", "Rogue"]),
        oracle_text: "Whenever one or more other creatures die, draw a card. This ability triggers only once each turn.".to_string(),
        power: Some(1),
        toughness: Some(3),
        abilities: vec![
            // ENGINE-BLOCKED: once-per-turn limiter for triggered abilities is not in the DSL.
            // Without it, the trigger fires per-death instead of at most once per turn.
        ],
        ..Default::default()
    }
}
