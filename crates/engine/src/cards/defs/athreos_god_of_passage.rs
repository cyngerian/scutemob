// Athreos, God of Passage — {1}{W}{B}, Legendary Enchantment Creature — God 5/4
// Indestructible
// As long as your devotion to white and black is less than seven, Athreos isn't a creature.
// Whenever another creature you own dies, return it to your hand unless target opponent
// pays 3 life.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("athreos-god-of-passage"),
        name: "Athreos, God of Passage".to_string(),
        mana_cost: Some(ManaCost { generic: 1, white: 1, black: 1, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Enchantment, CardType::Creature],
            &["God"],
        ),
        oracle_text: "Indestructible\nAs long as your devotion to white and black is less than seven, Athreos isn't a creature.\nWhenever another creature you own dies, return it to your hand unless target opponent pays 3 life.".to_string(),
        power: Some(5),
        toughness: Some(4),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Indestructible),
            // TODO: DSL gap — Devotion-based creature type loss (Theros Gods).
            // TODO: DSL gap — "Whenever another creature you own dies" with opponent
            // choice to pay life or return. Multiple DSL gaps.
        ],
        ..Default::default()
    }
}
