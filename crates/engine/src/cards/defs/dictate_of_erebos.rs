// Dictate of Erebos — {3}{B}{B}, Enchantment
// Flash
// Whenever a creature you control dies, each opponent sacrifices a creature of their choice.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("dictate-of-erebos"),
        name: "Dictate of Erebos".to_string(),
        mana_cost: Some(ManaCost { generic: 3, black: 2, ..Default::default() }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "Flash\nWhenever a creature you control dies, each opponent sacrifices a creature of their choice.".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flash),
            // TODO: DSL gap — "Whenever a creature you control dies" trigger with
            // controller filter + ForEach EachOpponent SacrificePermanents.
        ],
        ..Default::default()
    }
}
