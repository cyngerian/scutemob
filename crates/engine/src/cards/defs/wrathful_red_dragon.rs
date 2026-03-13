// Wrathful Red Dragon — {3}{R}{R}, Creature — Dragon 5/5
// Flying
// TODO: DSL gap — triggered ability "Whenever a Dragon you control is dealt damage, it deals
//   that much damage to any target that isn't a Dragon."
//   (WhenDealtDamage trigger with subtype filter on trigger source, and variable damage amount
//   equal to damage received, not supported in card DSL)
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("wrathful-red-dragon"),
        name: "Wrathful Red Dragon".to_string(),
        mana_cost: Some(ManaCost {
            generic: 3,
            red: 2,
            ..Default::default()
        }),
        types: creature_types(&["Dragon"]),
        oracle_text: "Flying\nWhenever a Dragon you control is dealt damage, it deals that much damage to any target that isn't a Dragon.".to_string(),
        power: Some(5),
        toughness: Some(5),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
        ],
        ..Default::default()
    }
}
