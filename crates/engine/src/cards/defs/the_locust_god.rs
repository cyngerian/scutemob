// The Locust God — {4}{U}{R}, Legendary Creature — God 4/4
// Flying
// Whenever you draw a card, create a 1/1 blue and red Insect creature token with
// flying and haste.
// {2}{U}{R}: Draw a card, then discard a card.
// When The Locust God dies, return it to its owner's hand at the beginning of the
// next end step.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("the-locust-god"),
        name: "The Locust God".to_string(),
        mana_cost: Some(ManaCost { generic: 4, blue: 1, red: 1, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["God"],
        ),
        oracle_text: "Flying\nWhenever you draw a card, create a 1/1 blue and red Insect creature token with flying and haste.\n{2}{U}{R}: Draw a card, then discard a card.\nWhen The Locust God dies, return it to its owner's hand at the beginning of the next end step.".to_string(),
        power: Some(4),
        toughness: Some(4),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            // Whenever you draw a card, create 1/1 U/R Insect with flying + haste.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WheneverYouDrawACard,
                effect: Effect::CreateToken {
                    spec: TokenSpec {
                        name: "Insect".to_string(),
                        card_types: [CardType::Creature].into_iter().collect(),
                        subtypes: [SubType("Insect".to_string())].into_iter().collect(),
                        colors: [Color::Blue, Color::Red].into_iter().collect(),
                        power: 1,
                        toughness: 1,
                        count: 1,
                        supertypes: im::OrdSet::new(),
                        keywords: [KeywordAbility::Flying, KeywordAbility::Haste].into_iter().collect(),
                        tapped: false,
                        enters_attacking: false,
                        mana_color: None,
                        mana_abilities: vec![],
                        activated_abilities: vec![],
                        ..Default::default()
                    },
                },
                intervening_if: None,
                targets: vec![],

                modes: None,
                trigger_zone: None,
            },
            // {2}{U}{R}: Draw a card, then discard a card.
            // TODO: "then discard" — forced discard after draw not expressible.
            AbilityDefinition::Activated {
                cost: Cost::Mana(ManaCost { generic: 2, blue: 1, red: 1, ..Default::default() }),
                effect: Effect::DrawCards {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(1),
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
            once_per_turn: false,
            },
            // When The Locust God dies, return it to its owner's hand at the beginning
            // of the next end step. Sets return_to_hand_at_end_step flag on graveyard object.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenDies,
                effect: Effect::SetReturnToHandAtEndStep,
                intervening_if: None,
                targets: vec![],

                modes: None,
                trigger_zone: None,
            },
        ],
        ..Default::default()
    }
}
