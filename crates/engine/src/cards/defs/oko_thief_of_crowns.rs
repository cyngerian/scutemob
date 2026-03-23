// Oko, Thief of Crowns — {1}{G}{U}, Legendary Planeswalker — Oko, Loyalty 4
// +2: Create a Food token.
// +1: Target artifact or creature loses all abilities and becomes a green Elk creature
//     with base power and toughness 3/3.
// −5: Exchange control of target artifact or creature you control and target creature an
//     opponent controls with power 3 or less.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("oko-thief-of-crowns"),
        name: "Oko, Thief of Crowns".to_string(),
        mana_cost: Some(ManaCost { generic: 1, green: 1, blue: 1, ..Default::default() }),
        types: full_types(&[SuperType::Legendary], &[CardType::Planeswalker], &["Oko"]),
        oracle_text: "+2: Create a Food token. (It's an artifact with \"{2}, {T}, Sacrifice this token: You gain 3 life.\")\n+1: Target artifact or creature loses all abilities and becomes a green Elk creature with base power and toughness 3/3.\n\u{2212}5: Exchange control of target artifact or creature you control and target creature an opponent controls with power 3 or less.".to_string(),
        starting_loyalty: Some(4),
        abilities: vec![
            // +2: Create a Food token
            AbilityDefinition::LoyaltyAbility {
                cost: LoyaltyCost::Plus(2),
                effect: Effect::CreateToken {
                    spec: food_token_spec(1),
                },
                targets: vec![],
            },
            // +1: TODO — Elkify (lose abilities, become 3/3 green Elk) not in DSL
            AbilityDefinition::LoyaltyAbility {
                cost: LoyaltyCost::Plus(1),
                effect: Effect::Nothing,
                targets: vec![],
            },
            // -5: TODO — exchange control not in DSL
            AbilityDefinition::LoyaltyAbility {
                cost: LoyaltyCost::Minus(5),
                effect: Effect::Nothing,
                targets: vec![],
            },
        ],
        ..Default::default()
    }
}
