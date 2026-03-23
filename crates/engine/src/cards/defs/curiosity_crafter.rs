// Curiosity Crafter — {3}{U}, Creature — Bird Wizard 3/3
// Flying
// You have no maximum hand size.
// Whenever a creature token you control deals combat damage to a player, draw a card.
//
// TODO: "No maximum hand size" static not in DSL.
// TODO: "Creature token deals combat damage" trigger not in DSL.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("curiosity-crafter"),
        name: "Curiosity Crafter".to_string(),
        mana_cost: Some(ManaCost { generic: 3, blue: 1, ..Default::default() }),
        types: creature_types(&["Bird", "Wizard"]),
        oracle_text: "Flying\nYou have no maximum hand size.\nWhenever a creature token you control deals combat damage to a player, draw a card.".to_string(),
        power: Some(3),
        toughness: Some(3),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            // TODO: No max hand size + token combat damage trigger not in DSL.
        ],
        ..Default::default()
    }
}
