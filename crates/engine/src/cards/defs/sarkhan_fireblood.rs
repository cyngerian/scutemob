// Sarkhan, Fireblood — {1}{R}{R}, Legendary Planeswalker — Sarkhan
// +1: You may discard a card. If you do, draw a card.
// +1: Add two mana in any combination of colors. Spend this mana only to cast Dragon spells.
// −7: Create four 5/5 red Dragon creature tokens with flying.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("sarkhan-fireblood"),
        name: "Sarkhan, Fireblood".to_string(),
        mana_cost: Some(ManaCost { generic: 1, red: 2, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Planeswalker],
            &["Sarkhan"],
        ),
        oracle_text: "+1: You may discard a card. If you do, draw a card.\n+1: Add two mana in any combination of colors. Spend this mana only to cast Dragon spells.\n\u{2212}7: Create four 5/5 red Dragon creature tokens with flying.".to_string(),
        starting_loyalty: Some(3),
        abilities: vec![
            // +1: Rummage (may discard, then draw)
            // TODO: Optional discard-then-draw not in DSL. Using Nothing to avoid free draw.
            AbilityDefinition::LoyaltyAbility {
                cost: LoyaltyCost::Plus(1),
                effect: Effect::Nothing,
                targets: vec![],
            },
            // +1: Add 2 mana (Dragon-restricted)
            // TODO: "Any combination of colors" + Dragon-only restriction not in DSL.
            AbilityDefinition::LoyaltyAbility {
                cost: LoyaltyCost::Plus(1),
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(0, 0, 0, 2, 0, 0),
                },
                targets: vec![],
            },
            // −7: Create four 5/5 red Dragon tokens with flying
            AbilityDefinition::LoyaltyAbility {
                cost: LoyaltyCost::Minus(7),
                effect: Effect::CreateToken {
                    spec: TokenSpec {
                        name: "Dragon".to_string(),
                        card_types: [CardType::Creature].into_iter().collect(),
                        subtypes: [SubType("Dragon".to_string())].into_iter().collect(),
                        colors: [Color::Red].into_iter().collect(),
                        power: 5,
                        toughness: 5,
                        count: 4,
                        supertypes: im::OrdSet::new(),
                        keywords: [KeywordAbility::Flying].into_iter().collect(),
                        tapped: false,
                        enters_attacking: false,
                        mana_color: None,
                        mana_abilities: vec![],
                        activated_abilities: vec![],
                        ..Default::default()
                    },
                },
                targets: vec![],
            },
        ],
        ..Default::default()
    }
}
