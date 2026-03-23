// Mangara, the Diplomat — {3}{W}, Legendary Creature — Human Cleric 2/4
// Lifelink
// Whenever an opponent attacks with creatures, if two or more of those creatures
// are attacking you and/or planeswalkers you control, draw a card.
// Whenever an opponent casts their second spell each turn, draw a card.
//
// TODO: Both triggers require opponent-action tracking not in DSL.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("mangara-the-diplomat"),
        name: "Mangara, the Diplomat".to_string(),
        mana_cost: Some(ManaCost { generic: 3, white: 1, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Human", "Cleric"],
        ),
        oracle_text: "Lifelink\nWhenever an opponent attacks with creatures, if two or more of those creatures are attacking you and/or planeswalkers you control, draw a card.\nWhenever an opponent casts their second spell each turn, draw a card.".to_string(),
        power: Some(2),
        toughness: Some(4),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Lifelink),
            // TODO: Both draw triggers need opponent-action tracking.
        ],
        ..Default::default()
    }
}
