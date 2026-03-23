// Kindred Discovery — {3}{U}{U}, Enchantment
// As Kindred Discovery enters, choose a creature type.
// Whenever a creature you control of the chosen type enters or attacks, draw a card.
//
// TODO: "Choose a creature type" — needs ChosenType designation on GameObject.
//   Both trigger conditions require knowing the chosen type at runtime.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("kindred-discovery"),
        name: "Kindred Discovery".to_string(),
        mana_cost: Some(ManaCost { generic: 3, blue: 2, ..Default::default() }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "As Kindred Discovery enters, choose a creature type.\nWhenever a creature you control of the chosen type enters or attacks, draw a card.".to_string(),
        // TODO: ChosenType designation not in DSL.
        abilities: vec![],
        ..Default::default()
    }
}
