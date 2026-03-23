// Sylvan Library — {1}{G}, Enchantment
// At the beginning of your draw step, you may draw two additional cards. If you do,
// choose two cards in your hand drawn this turn. For each of those cards, pay 4 life
// or put the card on top of your library.
//
// TODO: Complex replacement/draw-step ability — "draw two additional cards" then
//   choose two to pay 4 life or put back. Requires draw-step hook, card tracking
//   ("drawn this turn"), and player choice. Implementing as upkeep draw-2 approximation
//   with life loss (simplified — wrong but captures card advantage aspect).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("sylvan-library"),
        name: "Sylvan Library".to_string(),
        mana_cost: Some(ManaCost { generic: 1, green: 1, ..Default::default() }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "At the beginning of your draw step, you may draw two additional cards. If you do, choose two cards in your hand drawn this turn. For each of those cards, pay 4 life or put the card on top of your library.".to_string(),
        // TODO: Full Sylvan Library requires draw-step trigger (not upkeep),
        //   "drawn this turn" tracking, and player choice to pay/put-back.
        //   No abilities implemented — too complex for current DSL.
        abilities: vec![],
        ..Default::default()
    }
}
