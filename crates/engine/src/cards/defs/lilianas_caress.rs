// Liliana's Caress — {1}{B}, Enchantment
// Whenever an opponent discards a card, that player loses 2 life.
//
// TODO: Requires TriggerCondition::WheneverOpponentDiscards which does not exist in the DSL.
// The life-loss target "that player" also requires the trigger to pass the discarding player
// as a target reference, which is not supported. Omitted per W5 policy.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("lilianas-caress"),
        name: "Liliana's Caress".to_string(),
        mana_cost: Some(ManaCost { generic: 1, black: 1, ..Default::default() }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "Whenever an opponent discards a card, that player loses 2 life.".to_string(),
        abilities: vec![
            // Whenever an opponent discards a card, that player loses 2 life.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WheneverOpponentDiscards,
                effect: Effect::LoseLife {
                    player: PlayerTarget::TriggeringPlayer,
                    amount: EffectAmount::Fixed(2),
                },
                intervening_if: None,
                targets: vec![],
            },
        ],
        ..Default::default()
    }
}
