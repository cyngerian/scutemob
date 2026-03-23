// Beastmaster Ascension — {2}{G}, Enchantment
// Whenever a creature you control attacks, you may put a quest counter on this enchantment.
// As long as this enchantment has seven or more quest counters on it, creatures you control
// get +5/+5.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("beastmaster-ascension"),
        name: "Beastmaster Ascension".to_string(),
        mana_cost: Some(ManaCost { generic: 2, green: 1, ..Default::default() }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "Whenever a creature you control attacks, you may put a quest counter on this enchantment.\nAs long as this enchantment has seven or more quest counters on it, creatures you control get +5/+5.".to_string(),
        abilities: vec![
            // TODO: DSL gap — "Whenever a creature you control attacks" trigger condition
            // (WheneverCreatureYouControlAttacks) does not exist.
            // TODO: DSL gap — conditional static: "As long as this has 7+ quest counters,
            // creatures you control get +5/+5." Needs Condition-gated Static ability.
        ],
        ..Default::default()
    }
}
