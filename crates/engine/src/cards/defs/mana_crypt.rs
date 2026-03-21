// Mana Crypt — {0} Artifact.
// "At the beginning of your upkeep, flip a coin. If you lose the flip,
// Mana Crypt deals 3 damage to you."
// "{T}: Add {C}{C}."
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("mana-crypt"),
        name: "Mana Crypt".to_string(),
        mana_cost: Some(ManaCost::default()),
        types: types(&[CardType::Artifact]),
        oracle_text: "At the beginning of your upkeep, flip a coin. If you lose the flip, Mana Crypt deals 3 damage to you.\n{T}: Add {C}{C}.".to_string(),
        abilities: vec![
            // CR 705.1: Upkeep trigger — flip a coin; lose = 3 damage to controller.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::AtBeginningOfYourUpkeep,
                effect: Effect::CoinFlip {
                    on_win: Box::new(Effect::Nothing),
                    on_lose: Box::new(Effect::DealDamage {
                        target: EffectTarget::Controller,
                        amount: EffectAmount::Fixed(3),
                    }),
                },
                intervening_if: None,
                targets: vec![],
            },
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(0, 0, 0, 0, 0, 2),
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
            },
        ],
        ..Default::default()
    }
}
