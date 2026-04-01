// Exploration — {G}, Enchantment
// You may play an additional land on each of your turns.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("exploration"),
        name: "Exploration".to_string(),
        mana_cost: Some(ManaCost { green: 1, ..Default::default() }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "You may play an additional land on each of your turns.".to_string(),
        abilities: vec![
            // CR 305.2: Static ability granting one additional land play per turn.
            AbilityDefinition::AdditionalLandPlays { count: 1 },
        ],
        ..Default::default()
    }
}
