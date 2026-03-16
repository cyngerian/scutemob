// Rule of Law — {2}{W}, Enchantment
// Each player can't cast more than one spell each turn.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("rule-of-law"),
        name: "Rule of Law".to_string(),
        mana_cost: Some(ManaCost { generic: 2, white: 1, ..Default::default() }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "Each player can't cast more than one spell each turn.".to_string(),
        abilities: vec![
            AbilityDefinition::StaticRestriction {
                restriction: GameRestriction::MaxSpellsPerTurn { max: 1 },
            },
        ],
        ..Default::default()
    }
}
