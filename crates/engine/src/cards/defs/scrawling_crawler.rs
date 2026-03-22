// Scrawling Crawler — {3}, Artifact Creature — Phyrexian Construct 3/2
// At the beginning of your upkeep, each player draws a card.
// Whenever an opponent draws a card, that player loses 1 life.
//
// TODO: "Whenever an opponent draws a card" — WheneverPlayerDrawsCard doesn't filter by
//   opponent. Using it as-is (fires on all draws — slightly wrong for self-draws).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("scrawling-crawler"),
        name: "Scrawling Crawler".to_string(),
        mana_cost: Some(ManaCost { generic: 3, ..Default::default() }),
        types: types_sub(&[CardType::Artifact, CardType::Creature], &["Phyrexian", "Construct"]),
        oracle_text: "At the beginning of your upkeep, each player draws a card.\nWhenever an opponent draws a card, that player loses 1 life.".to_string(),
        power: Some(3),
        toughness: Some(2),
        abilities: vec![
            // At the beginning of your upkeep, each player draws a card
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::AtBeginningOfYourUpkeep,
                effect: Effect::DrawCards {
                    player: PlayerTarget::EachPlayer,
                    count: EffectAmount::Fixed(1),
                },
                intervening_if: None,
                targets: vec![],
            },
            // Whenever an opponent draws a card, that player loses 1 life
            // TODO: should only fire on opponent draws
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WheneverPlayerDrawsCard,
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
