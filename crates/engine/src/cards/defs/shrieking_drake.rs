// Shrieking Drake — {U}, Creature — Drake 1/1
// "Flying
// When this creature enters, return a creature you control to its owner's hand."
//
// Flying is implemented.
//
// TODO: DSL gap — "return a creature you control to its owner's hand" is an ETB triggered
// ability that bounces a creature you control. No Effect::ReturnToHand targeting a creature
// you control exists in the current DSL.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("shrieking-drake"),
        name: "Shrieking Drake".to_string(),
        mana_cost: Some(ManaCost { blue: 1, ..Default::default() }),
        types: creature_types(&["Drake"]),
        oracle_text: "Flying\nWhen this creature enters, return a creature you control to its owner's hand.".to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
        ],
        ..Default::default()
    }
}
