// Land Tax — {W} Enchantment.
// "At the beginning of your upkeep, if an opponent controls more lands than you,
// you may search your library for up to three basic land cards, reveal them,
// put them into your hand, then shuffle."
//
// TODO: Condition::OpponentControlsMoreLandsThanYou does not exist in the DSL.
// The intervening-if condition requires comparing two players' land counts at
// trigger time and resolution. No Condition variant captures this comparison.
// W5: wrong implementation omitted — abilities: vec![].
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("land-tax"),
        name: "Land Tax".to_string(),
        mana_cost: Some(ManaCost { white: 1, ..Default::default() }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "At the beginning of your upkeep, if an opponent controls more lands than you, you may search your library for up to three basic land cards, reveal them, put them into your hand, then shuffle.".to_string(),
        abilities: vec![
            // TODO: AtBeginningOfYourUpkeep trigger with intervening-if
            // Condition::OpponentControlsMoreLandsThanYou not in DSL.
            // Needs: compare count of lands controlled by any opponent vs. controller.
        ],
        ..Default::default()
    }
}
