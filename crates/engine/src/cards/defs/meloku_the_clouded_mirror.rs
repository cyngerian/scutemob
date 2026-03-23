// Meloku the Clouded Mirror — {4}{U}, Legendary Creature — Moonfolk Wizard 2/4
// Flying
// {1}, Return a land you control to its owner's hand: Create a 1/1 blue Illusion creature
// token with flying.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("meloku-the-clouded-mirror"),
        name: "Meloku the Clouded Mirror".to_string(),
        mana_cost: Some(ManaCost { generic: 4, blue: 1, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Moonfolk", "Wizard"],
        ),
        oracle_text: "Flying\n{1}, Return a land you control to its owner's hand: Create a 1/1 blue Illusion creature token with flying.".to_string(),
        power: Some(2),
        toughness: Some(4),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            // TODO: "{1}, Return a land you control to its owner's hand: Create a 1/1 blue
            // Illusion creature token with flying."
            // DSL gap: Cost::ReturnPermanentToHand (with filter for lands you control)
            // does not exist. A combined mana + return-land cost is not expressible.
        ],
        ..Default::default()
    }
}
