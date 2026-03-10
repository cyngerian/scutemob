// Mana Crypt — {0} Artifact.
// "At the beginning of your upkeep, flip a coin. If you lose the flip,
// Mana Crypt deals 3 damage to you."
// "{T}: Add {C}{C}."
// TODO: DSL gap — coin flip not expressible. Upkeep trigger modeled as
// always dealing 3 damage (worst-case deterministic fallback).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("mana-crypt"),
        name: "Mana Crypt".to_string(),
        mana_cost: Some(ManaCost::default()),
        types: types(&[CardType::Artifact]),
        oracle_text: "At the beginning of your upkeep, flip a coin. If you lose the flip, Mana Crypt deals 3 damage to you.\n{T}: Add {C}{C}.".to_string(),
        abilities: vec![
            // Upkeep trigger: deterministic fallback = always deal 3 damage
            // TODO: coin flip — should only deal damage 50% of the time
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::AtBeginningOfYourUpkeep,
                effect: Effect::DealDamage {
                    target: EffectTarget::Controller,
                    amount: EffectAmount::Fixed(3),
                },
                intervening_if: None,
            },
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(0, 0, 0, 0, 0, 2),
                },
                timing_restriction: None,
            },
        ],
        ..Default::default()
    }
}
