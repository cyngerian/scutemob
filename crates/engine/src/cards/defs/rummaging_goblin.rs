// Rummaging Goblin — {2}{R}, Creature — Goblin Rogue 1/1
// {T}, Discard a card: Draw a card.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("rummaging-goblin"),
        name: "Rummaging Goblin".to_string(),
        mana_cost: Some(ManaCost { generic: 2, red: 1, ..Default::default() }),
        types: creature_types(&["Goblin", "Rogue"]),
        oracle_text: "{T}, Discard a card: Draw a card.".to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![
            AbilityDefinition::Activated {
                cost: Cost::Sequence(vec![Cost::Tap, Cost::DiscardCard]),
                effect: Effect::DrawCards {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(1),
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
            once_per_turn: false,
            },
        ],
        ..Default::default()
    }
}
