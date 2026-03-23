// Warleader's Call — {1}{R}{W}, Enchantment
// Creatures you control get +1/+1.
// Whenever a creature you control enters, this enchantment deals 1 damage to each opponent.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("warleaders-call"),
        name: "Warleader's Call".to_string(),
        mana_cost: Some(ManaCost { generic: 1, red: 1, white: 1, ..Default::default() }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "Creatures you control get +1/+1.\nWhenever a creature you control enters, this enchantment deals 1 damage to each opponent.".to_string(),
        abilities: vec![
            // Static: creatures you control get +1/+1 (Layer 7c).
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::PtModify,
                    modification: LayerModification::ModifyBoth(1),
                    filter: EffectFilter::CreaturesYouControl,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                },
            },
            // Whenever a creature you control enters, deal 1 to each opponent.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WheneverCreatureEntersBattlefield {
                    filter: Some(TargetFilter {
                        controller: TargetController::You,
                        ..Default::default()
                    }),
                },
                effect: Effect::ForEach {
                    over: ForEachTarget::EachOpponent,
                    effect: Box::new(Effect::DealDamage {
                        target: EffectTarget::DeclaredTarget { index: 0 },
                        amount: EffectAmount::Fixed(1),
                    }),
                },
                intervening_if: None,
                targets: vec![],
            },
        ],
        ..Default::default()
    }
}
