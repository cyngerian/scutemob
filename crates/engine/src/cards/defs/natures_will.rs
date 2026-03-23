// Nature's Will — {2}{G}{G}, Enchantment
// Whenever one or more creatures you control deal combat damage to a player,
// tap all lands that player controls and untap all lands you control.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("natures-will"),
        name: "Nature's Will".to_string(),
        mana_cost: Some(ManaCost { generic: 2, green: 2, ..Default::default() }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "Whenever one or more creatures you control deal combat damage to a player, tap all lands that player controls and untap all lands you control.".to_string(),
        abilities: vec![
            // TODO: DSL gap — "Whenever one or more creatures you control deal combat damage
            // to a player" requires a per-combat-damage-step trigger that fires once per
            // damaged player. No such trigger condition exists.
            // Also needs: tap all lands target player controls + untap all lands you control.
        ],
        ..Default::default()
    }
}
