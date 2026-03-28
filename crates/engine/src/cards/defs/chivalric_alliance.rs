// Chivalric Alliance — {1}{W}, Enchantment
// Whenever you attack with two or more creatures, draw a card.
// {2}, Discard a card: Create a 2/2 white and blue Knight creature token with vigilance.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("chivalric-alliance"),
        name: "Chivalric Alliance".to_string(),
        mana_cost: Some(ManaCost { generic: 1, white: 1, ..Default::default() }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "Whenever you attack with two or more creatures, draw a card.\n{2}, Discard a card: Create a 2/2 white and blue Knight creature token with vigilance.".to_string(),
        abilities: vec![
            // Whenever you attack, draw a card.
            // TODO: "with two or more creatures" condition not in DSL.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WheneverYouAttack,
                effect: Effect::DrawCards {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(1),
                },
                intervening_if: None,
                targets: vec![],

                modes: None,
                trigger_zone: None,
            },
            // {2}, Discard: Create Knight token
            AbilityDefinition::Activated {
                cost: Cost::Sequence(vec![
                    Cost::Mana(ManaCost { generic: 2, ..Default::default() }),
                    Cost::DiscardCard,
                ]),
                effect: Effect::CreateToken {
                    spec: TokenSpec {
                        name: "Knight".to_string(),
                        card_types: [CardType::Creature].into_iter().collect(),
                        subtypes: [SubType("Knight".to_string())].into_iter().collect(),
                        colors: [Color::White, Color::Blue].into_iter().collect(),
                        power: 2,
                        toughness: 2,
                        count: 1,
                        supertypes: im::OrdSet::new(),
                        keywords: [KeywordAbility::Vigilance].into_iter().collect(),
                        tapped: false,
                        enters_attacking: false,
                        mana_color: None,
                        mana_abilities: vec![],
                        activated_abilities: vec![],
                        ..Default::default()
                    },
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
            },
        ],
        ..Default::default()
    }
}
