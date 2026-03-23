// Windcrag Siege — {1}{R}{W}, Enchantment
// As this enchantment enters, choose Mardu or Jeskai.
// Mardu — If a creature attacking causes a triggered ability of a permanent you control
// to trigger, that ability triggers an additional time.
// Jeskai — At the beginning of your upkeep, create a 1/1 red Goblin creature token.
// It gains lifelink and haste until end of turn.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("windcrag-siege"),
        name: "Windcrag Siege".to_string(),
        mana_cost: Some(ManaCost { generic: 1, red: 1, white: 1, ..Default::default() }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "As this enchantment enters, choose Mardu or Jeskai.\n\u{2022} Mardu \u{2014} If a creature attacking causes a triggered ability of a permanent you control to trigger, that ability triggers an additional time.\n\u{2022} Jeskai \u{2014} At the beginning of your upkeep, create a 1/1 red Goblin creature token. It gains lifelink and haste until end of turn.".to_string(),
        abilities: vec![
            // TODO: ETB mode choice (Mardu/Jeskai) not in DSL.
            // TODO: Mardu — attack trigger doubling not expressible.
            // TODO: Jeskai — upkeep Goblin token with lifelink+haste.
        ],
        ..Default::default()
    }
}
