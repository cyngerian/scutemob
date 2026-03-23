// Fathom Mage — {2}{G}{U}, Creature — Human Wizard 1/1
// Evolve
// Whenever a +1/+1 counter is put on Fathom Mage, you may draw a card.
//
// TODO: "Whenever a +1/+1 counter is put on" trigger not in DSL.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("fathom-mage"),
        name: "Fathom Mage".to_string(),
        mana_cost: Some(ManaCost { generic: 2, green: 1, blue: 1, ..Default::default() }),
        types: creature_types(&["Human", "Wizard"]),
        oracle_text: "Evolve\nWhenever a +1/+1 counter is put on Fathom Mage, you may draw a card.".to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Evolve),
            // TODO: Counter-placed trigger not in DSL.
        ],
        ..Default::default()
    }
}
