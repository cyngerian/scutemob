// Abundance — {2}{G}{G}, Enchantment
// If you would draw a card, you may instead choose land or nonland and reveal cards
// from the top of your library until you reveal a card of the chosen kind. Put that
// card into your hand and put all other cards revealed this way on the bottom of
// your library in any order.
//
// TODO: Draw replacement effect with player choice (land/nonland), reveal-until-found,
//   and bottom-of-library reorder. Far too complex for current DSL.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("abundance"),
        name: "Abundance".to_string(),
        mana_cost: Some(ManaCost { generic: 2, green: 2, ..Default::default() }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "If you would draw a card, you may instead choose land or nonland and reveal cards from the top of your library until you reveal a card of the chosen kind. Put that card into your hand and put all other cards revealed this way on the bottom of your library in any order.".to_string(),
        // TODO: Draw replacement effect too complex for DSL.
        abilities: vec![],
        ..Default::default()
    }
}
