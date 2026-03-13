// Molimo, Maro-Sorcerer — {4}{G}{G}{G}, Legendary Creature — Elemental Sorcerer */*
// Trample
// Molimo's power and toughness are each equal to the number of lands you control.
//
// Trample is implemented.
// TODO: DSL gap — dynamic P/T equal to the number of lands you control requires a
// Layer 7b continuous effect with a CountLands(controller) modifier. Not in DSL.
// Power/toughness set to 0/0 as placeholder.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("molimo-maro-sorcerer"),
        name: "Molimo, Maro-Sorcerer".to_string(),
        mana_cost: Some(ManaCost { generic: 4, green: 3, ..Default::default() }),
        types: full_types(&[SuperType::Legendary], &[CardType::Creature], &["Elemental", "Sorcerer"]),
        oracle_text: "Trample (This creature can deal excess combat damage to the player or planeswalker it's attacking.)\nMolimo's power and toughness are each equal to the number of lands you control.".to_string(),
        power: Some(0),
        toughness: Some(0),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Trample),
            // TODO: P/T = number of lands you control (dynamic Layer 7b, CountLands not in DSL)
        ],
        ..Default::default()
    }
}
