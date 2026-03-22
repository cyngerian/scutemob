// Kyren Negotiations — {2}{R}{R} Enchantment
// Tap an untapped creature you control: This enchantment deals 1 damage to target
// player or planeswalker.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("kyren-negotiations"),
        name: "Kyren Negotiations".to_string(),
        mana_cost: Some(ManaCost { generic: 2, red: 2, ..Default::default() }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "Tap an untapped creature you control: Kyren Negotiations deals 1 damage to target player or planeswalker.".to_string(),
        abilities: vec![
            // TODO: Cost::TapCreatureYouControl — tapping another creature as a cost
            // is not in the DSL (Cost enum lacks TapCreature variant).
        ],
        ..Default::default()
    }
}
