// Bloodchief Ascension — {B}, Enchantment
// At the beginning of each end step, if an opponent lost 2 or more life this turn,
// you may put a quest counter on this enchantment.
// Whenever a card is put into an opponent's graveyard from anywhere, if this enchantment
// has three or more quest counters on it, you may have that player lose 2 life.
// If you do, you gain 2 life.
//
// TODO: Both abilities are complex — end-step conditional counter placement needs
//   "opponent lost 2+ life this turn" condition (not in DSL), and the graveyard trigger
//   needs "3+ quest counters" intervening-if + "card put into opponent's graveyard"
//   trigger condition. Neither is expressible.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("bloodchief-ascension"),
        name: "Bloodchief Ascension".to_string(),
        mana_cost: Some(ManaCost { black: 1, ..Default::default() }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "At the beginning of each end step, if an opponent lost 2 or more life this turn, you may put a quest counter on Bloodchief Ascension. (Damage causes loss of life.)\nWhenever a card is put into an opponent's graveyard from anywhere, if Bloodchief Ascension has three or more quest counters on it, you may have that player lose 2 life. If you do, you gain 2 life.".to_string(),
        abilities: vec![],
        ..Default::default()
    }
}
