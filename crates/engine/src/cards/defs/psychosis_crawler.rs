// Psychosis Crawler — {5}, Artifact Creature — Phyrexian Horror */*
// Psychosis Crawler's power and toughness are each equal to the number of cards
// in your hand.
// Whenever you draw a card, each opponent loses 1 life.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("psychosis-crawler"),
        name: "Psychosis Crawler".to_string(),
        mana_cost: Some(ManaCost { generic: 5, ..Default::default() }),
        types: full_types(
            &[],
            &[CardType::Artifact, CardType::Creature],
            &["Phyrexian", "Horror"],
        ),
        oracle_text: "Psychosis Crawler's power and toughness are each equal to the number of cards in your hand.\nWhenever you draw a card, each opponent loses 1 life.".to_string(),
        // */* CDA creature — use None for power/toughness
        power: None,
        toughness: None,
        abilities: vec![
            // TODO: CDA "P/T equal to cards in hand" — needs CharacteristicDefiningAbility
            //   with EffectAmount::CardCount for hand zone. Not in DSL.
            // Whenever you draw a card, each opponent loses 1 life.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WheneverYouDrawACard,
                effect: Effect::LoseLife {
                    player: PlayerTarget::EachOpponent,
                    amount: EffectAmount::Fixed(1),
                },
                intervening_if: None,
                targets: vec![],
            },
        ],
        ..Default::default()
    }
}
