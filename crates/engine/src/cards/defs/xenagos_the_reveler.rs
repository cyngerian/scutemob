// Xenagos, the Reveler — {2}{R}{G}, Legendary Planeswalker — Xenagos, Loyalty 3
// +1: Add X mana in any combination of {R} and/or {G}, where X is the number of
// creatures you control.
// 0: Create a 2/2 red and green Satyr creature token with haste.
// −6: Exile the top seven cards of your library. You may put any number of creature
// and/or land cards from among them onto the battlefield.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("xenagos-the-reveler"),
        name: "Xenagos, the Reveler".to_string(),
        mana_cost: Some(ManaCost { generic: 2, red: 1, green: 1, ..Default::default() }),
        types: full_types(&[SuperType::Legendary], &[CardType::Planeswalker], &["Xenagos"]),
        oracle_text: "+1: Add X mana in any combination of {R} and/or {G}, where X is the number of creatures you control.\n0: Create a 2/2 red and green Satyr creature token with haste.\n\u{2212}6: Exile the top seven cards of your library. You may put any number of creature and/or land cards from among them onto the battlefield.".to_string(),
        starting_loyalty: Some(3),
        abilities: vec![
            // +1: TODO — count-based mana production (X = creatures you control) not in DSL
            AbilityDefinition::LoyaltyAbility {
                cost: LoyaltyCost::Plus(1),
                effect: Effect::Nothing,
                targets: vec![],
            },
            // 0: Create a 2/2 red and green Satyr with haste
            AbilityDefinition::LoyaltyAbility {
                cost: LoyaltyCost::Zero,
                effect: Effect::CreateToken {
                    spec: TokenSpec {
                        name: "Satyr".to_string(),
                        card_types: [CardType::Creature].into_iter().collect(),
                        subtypes: [SubType("Satyr".to_string())].into_iter().collect(),
                        colors: [Color::Red, Color::Green].into_iter().collect(),
                        power: 2,
                        toughness: 2,
                        count: 1,
                        supertypes: im::OrdSet::new(),
                        keywords: [KeywordAbility::Haste].into_iter().collect(),
                        tapped: false,
                        enters_attacking: false,
                        mana_color: None,
                        mana_abilities: vec![],
                        activated_abilities: vec![],
                    },
                },
                targets: vec![],
            },
            // -6: TODO — exile top 7, put creatures/lands onto battlefield not in DSL
            AbilityDefinition::LoyaltyAbility {
                cost: LoyaltyCost::Minus(6),
                effect: Effect::Nothing,
                targets: vec![],
            },
        ],
        ..Default::default()
    }
}
