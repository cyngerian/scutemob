// Grave Pact
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("grave-pact"),
        name: "Grave Pact".to_string(),
        mana_cost: Some(ManaCost { generic: 1, black: 3, ..Default::default() }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "Whenever a creature you control dies, each other player sacrifices a creature of their choice.".to_string(),
        abilities: vec![
            // TODO: DSL gap — "Whenever a creature you control dies" trigger (needs
            // controller filter on WheneverCreatureDies). Effect also needs ForEach
            // EachOpponent + SacrificePermanents with player choice.
        ],
        ..Default::default()
    }
}
