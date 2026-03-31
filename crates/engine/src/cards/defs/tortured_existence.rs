// Tortured Existence — {B} Enchantment
// {B}, Discard a creature card: Return target creature card from your graveyard to your hand.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("tortured-existence"),
        name: "Tortured Existence".to_string(),
        mana_cost: Some(ManaCost { black: 1, ..Default::default() }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "{B}, Discard a creature card: Return target creature card from your graveyard to your hand.".to_string(),
        abilities: vec![
            // TODO: "Discard a creature card" — Cost::DiscardCard discards any card (no filter).
            // A filtered discard cost (Cost::DiscardCardWithFilter(TargetFilter)) does not exist.
            // Implementing with Cost::DiscardCard produces wrong game state (allows discarding
            // non-creature cards). Per W5 policy, the ability is omitted.
        ],
        ..Default::default()
    }
}
