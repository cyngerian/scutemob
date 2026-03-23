// Luminous Broodmoth — {2}{W}{W}, Creature — Insect 3/4
// Flying
// Whenever a creature you control without flying dies, return it to the battlefield under
// its owner's control with a flying counter on it.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("luminous-broodmoth"),
        name: "Luminous Broodmoth".to_string(),
        mana_cost: Some(ManaCost { generic: 2, white: 2, ..Default::default() }),
        types: creature_types(&["Insect"]),
        oracle_text: "Flying\nWhenever a creature you control without flying dies, return it to the battlefield under its owner's control with a flying counter on it.".to_string(),
        power: Some(3),
        toughness: Some(4),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            // TODO: DSL gap — "Whenever a creature you control without flying dies" needs
            // WheneverCreatureDies with controller filter + keyword exclusion filter +
            // return from GY + flying counter. Multiple DSL gaps.
        ],
        ..Default::default()
    }
}
