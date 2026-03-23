// Aurelia, the Law Above — {3}{R}{W}, Legendary Creature — Angel 4/4
// Flying, vigilance, haste
// Whenever a player attacks with three or more creatures, you draw a card.
// Whenever a player attacks with five or more creatures, Aurelia deals 3 damage to each
// of your opponents and you gain 3 life.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("aurelia-the-law-above"),
        name: "Aurelia, the Law Above".to_string(),
        mana_cost: Some(ManaCost { generic: 3, red: 1, white: 1, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Angel"],
        ),
        oracle_text: "Flying, vigilance, haste\nWhenever a player attacks with three or more creatures, you draw a card.\nWhenever a player attacks with five or more creatures, Aurelia deals 3 damage to each of your opponents and you gain 3 life.".to_string(),
        power: Some(4),
        toughness: Some(4),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            AbilityDefinition::Keyword(KeywordAbility::Vigilance),
            AbilityDefinition::Keyword(KeywordAbility::Haste),
            // TODO: "Whenever a player attacks with 3+ creatures" trigger not in DSL.
            // TODO: "Whenever a player attacks with 5+ creatures" trigger not in DSL.
        ],
        ..Default::default()
    }
}
