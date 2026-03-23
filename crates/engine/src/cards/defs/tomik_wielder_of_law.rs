// Tomik, Wielder of Law — {1}{W}{B}, Legendary Creature — Human Advisor 2/4
// Affinity for planeswalkers
// Flying, vigilance
// Whenever an opponent attacks with creatures, if two or more of those creatures are
// attacking you and/or planeswalkers you control, that opponent loses 3 life and you
// draw a card.
//
// TODO: "Affinity for planeswalkers" — AffinityTarget::Planeswalkers not in enum.
// TODO: Attack-count trigger with threshold — not expressible.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("tomik-wielder-of-law"),
        name: "Tomik, Wielder of Law".to_string(),
        mana_cost: Some(ManaCost { generic: 1, white: 1, black: 1, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Human", "Advisor"],
        ),
        oracle_text: "Affinity for planeswalkers\nFlying, vigilance\nWhenever an opponent attacks with creatures, if two or more of those creatures are attacking you and/or planeswalkers you control, that opponent loses 3 life and you draw a card.".to_string(),
        power: Some(2),
        toughness: Some(4),
        abilities: vec![
            // TODO: Affinity for planeswalkers — AffinityTarget lacks Planeswalkers.
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            AbilityDefinition::Keyword(KeywordAbility::Vigilance),
            // TODO: Attack-count trigger with 2+ threshold not expressible.
        ],
        ..Default::default()
    }
}
