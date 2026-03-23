// Black Market Connections — {2}{B} Enchantment
// At the beginning of your first main phase, choose one or more —
// • Sell Contraband — Create a Treasure token. You lose 1 life.
// • Buy Information — Draw a card. You lose 2 life.
// • Hire a Mercenary — Create a 3/2 colorless Shapeshifter creature token with changeling.
//   You lose 3 life.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("black-market-connections"),
        name: "Black Market Connections".to_string(),
        mana_cost: Some(ManaCost { generic: 2, black: 1, ..Default::default() }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "At the beginning of your first main phase, choose one or more —\n• Sell Contraband — Create a Treasure token. You lose 1 life.\n• Buy Information — Draw a card. You lose 2 life.\n• Hire a Mercenary — Create a 3/2 colorless Shapeshifter creature token with changeling. You lose 3 life.".to_string(),
        abilities: vec![
            // TODO: "At the beginning of your first main phase" trigger not in DSL —
            // TriggerCondition has no AtBeginningOfFirstMainPhase variant.
            // Additionally, "choose one or more" modal on a triggered ability is not
            // expressible in DSL. Needs both a new trigger condition and modal trigger support.
        ],
        ..Default::default()
    }
}
