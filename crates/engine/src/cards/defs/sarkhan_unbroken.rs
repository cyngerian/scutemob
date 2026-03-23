// Sarkhan Unbroken — {2}{G}{U}{R}, Legendary Planeswalker — Sarkhan
// +1: Draw a card, then add one mana of any color.
// −2: Create a 4/4 red Dragon creature token with flying.
// −8: Search your library for any number of Dragon creature cards, put them onto the
//     battlefield, then shuffle.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("sarkhan-unbroken"),
        name: "Sarkhan Unbroken".to_string(),
        mana_cost: Some(ManaCost { generic: 2, green: 1, blue: 1, red: 1, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Planeswalker],
            &["Sarkhan"],
        ),
        oracle_text: "+1: Draw a card, then add one mana of any color.\n\u{2212}2: Create a 4/4 red Dragon creature token with flying.\n\u{2212}8: Search your library for any number of Dragon creature cards, put them onto the battlefield, then shuffle.".to_string(),
        starting_loyalty: Some(4),
        abilities: vec![
            // +1: Draw + add mana
            AbilityDefinition::LoyaltyAbility {
                cost: LoyaltyCost::Plus(1),
                effect: Effect::Sequence(vec![
                    Effect::DrawCards {
                        player: PlayerTarget::Controller,
                        count: EffectAmount::Fixed(1),
                    },
                    // TODO: "Add one mana of any color" — player choice not in DSL.
                    //   Defaults to green; actual color should be player's choice.
                    Effect::AddMana {
                        player: PlayerTarget::Controller,
                        mana: mana_pool(0, 0, 0, 0, 0, 1),
                    },
                ]),
                targets: vec![],
            },
            // −2: Create 4/4 Dragon
            AbilityDefinition::LoyaltyAbility {
                cost: LoyaltyCost::Minus(2),
                effect: Effect::CreateToken {
                    spec: TokenSpec {
                        name: "Dragon".to_string(),
                        card_types: [CardType::Creature].into_iter().collect(),
                        subtypes: [SubType("Dragon".to_string())].into_iter().collect(),
                        colors: [Color::Red].into_iter().collect(),
                        power: 4,
                        toughness: 4,
                        count: 1,
                        supertypes: im::OrdSet::new(),
                        keywords: [KeywordAbility::Flying].into_iter().collect(),
                        tapped: false,
                        enters_attacking: false,
                        mana_color: None,
                        mana_abilities: vec![],
                        activated_abilities: vec![],
                    },
                },
                targets: vec![],
            },
            // −8: Search for Dragons — too complex (any number)
            // TODO: "Search for any number of Dragon cards" not expressible.
            AbilityDefinition::LoyaltyAbility {
                cost: LoyaltyCost::Minus(8),
                effect: Effect::Nothing,
                targets: vec![],
            },
        ],
        ..Default::default()
    }
}
