// Teferi, Temporal Pilgrim — {3}{U}{U}, Legendary Planeswalker — Teferi
// Whenever you draw a card, put a loyalty counter on Teferi.
// 0: Draw a card.
// −2: Create a 2/2 blue Spirit creature token with vigilance and "Whenever you
//   draw a card, put a +1/+1 counter on this token."
// −12: Target opponent chooses a permanent they control and returns it to its
//   owner's hand. Then they shuffle each nonland permanent they control into its
//   owner's library.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("teferi-temporal-pilgrim"),
        name: "Teferi, Temporal Pilgrim".to_string(),
        mana_cost: Some(ManaCost { generic: 3, blue: 2, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Planeswalker],
            &["Teferi"],
        ),
        oracle_text: "Whenever you draw a card, put a loyalty counter on Teferi, Temporal Pilgrim.\n0: Draw a card.\n\u{2212}2: Create a 2/2 blue Spirit creature token with vigilance and \"Whenever you draw a card, put a +1/+1 counter on this creature.\"\n\u{2212}12: Target opponent chooses a permanent they control and returns it to its owner's hand. Then they shuffle each nonland permanent they control into its owner's library.".to_string(),
        starting_loyalty: Some(4),
        abilities: vec![
            // Whenever you draw a card, put a loyalty counter on Teferi.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WheneverYouDrawACard,
                effect: Effect::AddCounter {
                    target: EffectTarget::Source,
                    counter: CounterType::Loyalty,
                    count: 1,
                },
                intervening_if: None,
                targets: vec![],
            },
            // 0: Draw a card.
            AbilityDefinition::LoyaltyAbility {
                cost: LoyaltyCost::Zero,
                effect: Effect::DrawCards {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(1),
                },
                targets: vec![],
            },
            // −2: Create a 2/2 blue Spirit with vigilance + draw-counter trigger.
            // TODO: Token with "whenever you draw a card, put +1/+1 counter" —
            //   token triggered abilities not expressible in TokenSpec.
            AbilityDefinition::LoyaltyAbility {
                cost: LoyaltyCost::Minus(2),
                effect: Effect::CreateToken {
                    spec: TokenSpec {
                        name: "Spirit".to_string(),
                        card_types: [CardType::Creature].into_iter().collect(),
                        subtypes: [SubType("Spirit".to_string())].into_iter().collect(),
                        colors: [Color::Blue].into_iter().collect(),
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
                targets: vec![],
            },
            // −12: Complex bounce + shuffle — too complex for DSL.
        ],
        ..Default::default()
    }
}
