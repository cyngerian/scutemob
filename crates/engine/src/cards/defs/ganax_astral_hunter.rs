// Ganax, Astral Hunter — {4}{R}, Legendary Creature — Dragon 3/4
// Flying
// Whenever Ganax or another Dragon you control enters, create a Treasure token.
// Choose a Background
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("ganax-astral-hunter"),
        name: "Ganax, Astral Hunter".to_string(),
        mana_cost: Some(ManaCost { generic: 4, red: 1, ..Default::default() }),
        types: full_types(&[SuperType::Legendary], &[CardType::Creature], &["Dragon"]),
        oracle_text: "Flying\nWhenever Ganax or another Dragon you control enters, create a Treasure token.\nChoose a Background".to_string(),
        power: Some(3),
        toughness: Some(4),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            // TODO: "Whenever Ganax or another Dragon enters" — subtype-filtered ETB trigger not in DSL
            AbilityDefinition::Keyword(KeywordAbility::ChooseABackground),
        ],
        ..Default::default()
    }
}
