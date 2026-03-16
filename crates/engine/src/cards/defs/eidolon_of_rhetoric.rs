// Eidolon of Rhetoric — {2}{W}, Enchantment Creature — Spirit 1/4
// Each player can't cast more than one spell each turn.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("eidolon-of-rhetoric"),
        name: "Eidolon of Rhetoric".to_string(),
        mana_cost: Some(ManaCost { generic: 2, white: 1, ..Default::default() }),
        types: full_types(&[], &[CardType::Enchantment, CardType::Creature], &["Spirit"]),
        oracle_text: "Each player can't cast more than one spell each turn.".to_string(),
        power: Some(1),
        toughness: Some(4),
        abilities: vec![
            AbilityDefinition::StaticRestriction {
                restriction: GameRestriction::MaxSpellsPerTurn { max: 1 },
            },
        ],
        ..Default::default()
    }
}
