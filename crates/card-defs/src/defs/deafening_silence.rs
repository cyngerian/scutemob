// Deafening Silence — {W}, Enchantment
// Each player can't cast more than one noncreature spell each turn.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("deafening-silence"),
        name: "Deafening Silence".to_string(),
        mana_cost: Some(ManaCost { white: 1, ..Default::default() }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "Each player can't cast more than one noncreature spell each turn.".to_string(),
        abilities: vec![
            AbilityDefinition::StaticRestriction {
                restriction: GameRestriction::MaxNoncreatureSpellsPerTurn { max: 1 },
            },
        ],
        ..Default::default()
    }
}
