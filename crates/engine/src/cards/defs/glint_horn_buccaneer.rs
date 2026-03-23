// Glint-Horn Buccaneer — {1}{R}{R}, Creature — Minotaur Pirate 2/4
// Haste
// Whenever you discard a card, this creature deals 1 damage to each opponent.
// {1}{R}, Discard a card: Draw a card. Activate only if this creature is attacking.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("glint-horn-buccaneer"),
        name: "Glint-Horn Buccaneer".to_string(),
        mana_cost: Some(ManaCost { generic: 1, red: 2, ..Default::default() }),
        types: creature_types(&["Minotaur", "Pirate"]),
        oracle_text: "Haste\nWhenever you discard a card, this creature deals 1 damage to each opponent.\n{1}{R}, Discard a card: Draw a card. Activate only if this creature is attacking.".to_string(),
        power: Some(2),
        toughness: Some(4),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Haste),
            // TODO: "Whenever you discard a card" — no WheneverYouDiscard trigger in DSL.
            // TODO: "{1}{R}, Discard a card: Draw a card. Activate only if attacking."
            // Requires activation condition (is_attacking) + discard as cost.
        ],
        ..Default::default()
    }
}
