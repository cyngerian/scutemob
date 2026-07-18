// Raiders' Wake — {3}{B}, Enchantment
// Whenever an opponent discards a card, that player loses 2 life.
// Raid — At the beginning of your end step, if you attacked this turn, target opponent
// discards a card.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("raiders-wake"),
        name: "Raiders' Wake".to_string(),
        mana_cost: Some(ManaCost {
            generic: 3,
            black: 1,
            ..Default::default()
        }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "Whenever an opponent discards a card, that player loses 2 life.\nRaid — At \
                      the beginning of your end step, if you attacked this turn, target opponent \
                      discards a card."
            .to_string(),
        abilities: vec![
            // Whenever an opponent discards a card, that player loses 2 life.
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::WheneverOpponentDiscards,
                effect: Effect::LoseLife {
                    player: PlayerTarget::TriggeringPlayer,
                    amount: EffectAmount::Fixed(2),
                },
                intervening_if: None,
                targets: vec![],

                modes: None,
                trigger_zone: None,
            },
            // Raid — At the beginning of your end step, if you attacked this turn, target
            // opponent discards a card. PB-AC6 supplies Condition::YouAttackedThisTurn;
            // PB-EF6 supplies TargetRequirement::TargetOpponent.
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::AtBeginningOfYourEndStep,
                effect: Effect::DiscardCards {
                    player: PlayerTarget::DeclaredTarget { index: 0 },
                    count: EffectAmount::Fixed(1),
                },
                intervening_if: Some(Condition::YouAttackedThisTurn),
                targets: vec![TargetRequirement::TargetOpponent],

                modes: None,
                trigger_zone: None,
            },
        ],
        ..Default::default()
    }
}
