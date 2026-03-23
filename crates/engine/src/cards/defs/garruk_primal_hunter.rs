// Garruk, Primal Hunter — {2}{G}{G}{G}, Legendary Planeswalker — Garruk
// +1: Create a 3/3 green Beast creature token.
// −3: Draw cards equal to the greatest power among creatures you control.
// −6: Create a 6/6 green Wurm creature token for each land you control.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("garruk-primal-hunter"),
        name: "Garruk, Primal Hunter".to_string(),
        mana_cost: Some(ManaCost { generic: 2, green: 3, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Planeswalker],
            &["Garruk"],
        ),
        oracle_text: "+1: Create a 3/3 green Beast creature token.\n\u{2212}3: Draw cards equal to the greatest power among creatures you control.\n\u{2212}6: Create a 6/6 green Wurm creature token for each land you control.".to_string(),
        starting_loyalty: Some(3),
        abilities: vec![
            // +1: Create 3/3 Beast
            AbilityDefinition::LoyaltyAbility {
                cost: LoyaltyCost::Plus(1),
                effect: Effect::CreateToken {
                    spec: TokenSpec {
                        name: "Beast".to_string(),
                        card_types: [CardType::Creature].into_iter().collect(),
                        subtypes: [SubType("Beast".to_string())].into_iter().collect(),
                        colors: [Color::Green].into_iter().collect(),
                        power: 3,
                        toughness: 3,
                        count: 1,
                        supertypes: im::OrdSet::new(),
                        keywords: im::OrdSet::new(),
                        tapped: false,
                        enters_attacking: false,
                        mana_color: None,
                        mana_abilities: vec![],
                        activated_abilities: vec![],
                    },
                },
                targets: vec![],
            },
            // −3: Draw cards equal to greatest power
            // TODO: EffectAmount lacks "greatest power among creatures you control" variant.
            AbilityDefinition::LoyaltyAbility {
                cost: LoyaltyCost::Minus(3),
                effect: Effect::DrawCards {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(3),
                },
                targets: vec![],
            },
            // −6: Create Wurm tokens equal to lands
            // TODO: Count-based token creation (lands you control) not in DSL.
            AbilityDefinition::LoyaltyAbility {
                cost: LoyaltyCost::Minus(6),
                effect: Effect::CreateToken {
                    spec: TokenSpec {
                        name: "Wurm".to_string(),
                        card_types: [CardType::Creature].into_iter().collect(),
                        subtypes: [SubType("Wurm".to_string())].into_iter().collect(),
                        colors: [Color::Green].into_iter().collect(),
                        power: 6,
                        toughness: 6,
                        count: 5,
                        supertypes: im::OrdSet::new(),
                        keywords: im::OrdSet::new(),
                        tapped: false,
                        enters_attacking: false,
                        mana_color: None,
                        mana_abilities: vec![],
                        activated_abilities: vec![],
                    },
                },
                targets: vec![],
            },
        ],
        ..Default::default()
    }
}
