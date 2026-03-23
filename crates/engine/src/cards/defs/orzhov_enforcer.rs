// Orzhov Enforcer — {1}{B}, Creature — Human Rogue 1/2
// Deathtouch
// Afterlife 1 (When this creature dies, create a 1/1 white and black Spirit creature token
// with flying.)
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("orzhov-enforcer"),
        name: "Orzhov Enforcer".to_string(),
        mana_cost: Some(ManaCost { generic: 1, black: 1, ..Default::default() }),
        types: creature_types(&["Human", "Rogue"]),
        oracle_text: "Deathtouch\nAfterlife 1 (When this creature dies, create a 1/1 white and black Spirit creature token with flying.)".to_string(),
        power: Some(1),
        toughness: Some(2),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Deathtouch),
            AbilityDefinition::Keyword(KeywordAbility::Afterlife(1)),
        ],
        ..Default::default()
    }
}
