// Kemba, Kha Regent — {1}{W}{W}, Legendary Creature — Cat Cleric 2/4
// At the beginning of your upkeep, create a 2/2 white Cat creature token for each Equipment
// attached to Kemba.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("kemba-kha-regent"),
        name: "Kemba, Kha Regent".to_string(),
        mana_cost: Some(ManaCost { generic: 1, white: 2, ..Default::default() }),
        types: full_types(&[SuperType::Legendary], &[CardType::Creature], &["Cat", "Cleric"]),
        oracle_text: "At the beginning of your upkeep, create a 2/2 white Cat creature token for each Equipment attached to Kemba.".to_string(),
        power: Some(2),
        toughness: Some(4),
        abilities: vec![
            // TODO: AtBeginningOfYourUpkeep trigger that creates N tokens where N = count of
            // Equipment attached to self. EffectAmount::CountAttachedEquipment not in DSL.
        ],
        ..Default::default()
    }
}
