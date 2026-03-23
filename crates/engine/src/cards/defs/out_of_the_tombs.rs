// Out of the Tombs — {2}{B}, Enchantment
// At the beginning of your upkeep, put two eon counters on this enchantment,
// then mill cards equal to the number of eon counters on it.
// If you would draw a card while your library has no cards in it, instead return
// a creature card from your graveyard to the battlefield. If you can't, you lose
// the game.
//
// TODO: Upkeep counter + mill scaling with counter count not expressible.
// TODO: Draw replacement effect when library empty not expressible.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("out-of-the-tombs"),
        name: "Out of the Tombs".to_string(),
        mana_cost: Some(ManaCost { generic: 2, black: 1, ..Default::default() }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "At the beginning of your upkeep, put two eon counters on this enchantment, then mill cards equal to the number of eon counters on it.\nIf you would draw a card while your library has no cards in it, instead return a creature card from your graveyard to the battlefield. If you can't, you lose the game.".to_string(),
        // TODO: Both abilities too complex for DSL.
        abilities: vec![],
        ..Default::default()
    }
}
