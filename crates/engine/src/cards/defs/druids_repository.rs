// Druids' Repository — {1}{G}{G}, Enchantment
// Whenever a creature you control attacks, put a charge counter on this enchantment.
// Remove a charge counter from this enchantment: Add one mana of any color.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("druids-repository"),
        name: "Druids' Repository".to_string(),
        mana_cost: Some(ManaCost { generic: 1, green: 2, ..Default::default() }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "Whenever a creature you control attacks, put a charge counter on this enchantment.\nRemove a charge counter from this enchantment: Add one mana of any color.".to_string(),
        abilities: vec![
            // TODO: DSL gap — "Whenever a creature you control attacks" trigger condition
            // (WheneverCreatureYouControlAttacks) does not exist.
            // TODO: Cost::RemoveCounter does not exist — "Remove a charge counter: Add one
            // mana of any color" cannot be expressed as an activated ability.
        ],
        ..Default::default()
    }
}
