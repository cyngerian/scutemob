// Impact Tremors — {1}{R}, Enchantment
// Whenever a creature you control enters, this enchantment deals 1 damage to each opponent.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("impact-tremors"),
        name: "Impact Tremors".to_string(),
        mana_cost: Some(ManaCost { generic: 1, red: 1, ..Default::default() }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "Whenever a creature you control enters, this enchantment deals 1 damage to each opponent.".to_string(),
        abilities: vec![AbilityDefinition::Triggered {
            trigger_condition: TriggerCondition::WheneverCreatureEntersBattlefield {
                filter: Some(TargetFilter {
                    controller: TargetController::You,
                    ..Default::default()
                }),
            },
            // ForEach over EachOpponent: engine creates an inner context with each
            // opponent as Target::Player at index 0 (see effects/mod.rs ForEach handler).
            effect: Effect::ForEach {
                over: ForEachTarget::EachOpponent,
                effect: Box::new(Effect::DealDamage {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                    amount: EffectAmount::Fixed(1),
                }),
            },
            intervening_if: None,
        }],
        ..Default::default()
    }
}
