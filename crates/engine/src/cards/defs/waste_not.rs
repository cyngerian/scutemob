// Waste Not — {1}{B}, Enchantment
// Whenever an opponent discards a creature card, create a 2/2 black Zombie creature token.
// Whenever an opponent discards a land card, add {B}{B}.
// Whenever an opponent discards a noncreature, nonland card, draw a card.
//
// TODO: All three abilities require TriggerCondition::WheneverOpponentDiscards which does not
// exist in the DSL. Additionally, each trigger requires conditional logic based on the
// discarded card's type (creature / land / noncreature-nonland), which further requires
// a card-type filter on the discard trigger. Both primitives are missing. Omitted per W5 policy.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("waste-not"),
        name: "Waste Not".to_string(),
        mana_cost: Some(ManaCost { generic: 1, black: 1, ..Default::default() }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "Whenever an opponent discards a creature card, create a 2/2 black Zombie creature token.\nWhenever an opponent discards a land card, add {B}{B}.\nWhenever an opponent discards a noncreature, nonland card, draw a card.".to_string(),
        abilities: vec![],
        ..Default::default()
    }
}
