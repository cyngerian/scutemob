// The Meathook Massacre
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("the-meathook-massacre"),
        name: "The Meathook Massacre".to_string(),
        mana_cost: Some(ManaCost {
            black: 2,
            x_count: 1,
            ..Default::default()
        }),
        types: full_types(&[SuperType::Legendary], &[CardType::Enchantment], &[]),
        oracle_text: "When The Meathook Massacre enters, each creature gets -X/-X until end of \
                      turn.
Whenever a creature you control dies, each opponent loses 1 life.
Whenever a creature an opponent controls dies, you gain 1 life."
            .to_string(),
        abilities: vec![
            // CR 603.3: ETB "each creature gets -X/-X until end of turn". X is the spell's
            // cast X, propagated from the resolved permanent's x_value into the ETB
            // trigger's EffectContext (resolution.rs: "CR 107.3m: Propagate x_value from
            // the permanent so ETB effects using EffectAmount::XValue resolve correctly"),
            // then locked in at ApplyContinuousEffect resolution (CR 608.2h). Verified
            // empirically against resolution.rs — this is not the doc-warned "unsubstituted
            // XValue reads 0" case, since ETB triggers go through the same
            // Effect::ApplyContinuousEffect substitution path as instants/sorceries.
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                effect: Effect::ApplyContinuousEffect {
                    effect_def: Box::new(ContinuousEffectDef {
                        layer: EffectLayer::PtModify,
                        modification: LayerModification::ModifyBothDynamic {
                            amount: Box::new(EffectAmount::XValue),
                            negate: true,
                        },
                        filter: EffectFilter::AllCreatures,
                        duration: EffectDuration::UntilEndOfTurn,
                        condition: None,
                    }),
                },
                intervening_if: None,
                targets: vec![],
                modes: None,
                trigger_zone: None,
            },
            // CR 603.10a: "Whenever a creature you control dies, each opponent loses 1 life."
            // PB-23: controller_you filter applied via DeathTriggerFilter.
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::WheneverCreatureDies {
                    controller: Some(TargetController::You),
                    exclude_self: false,
                    nontoken_only: false,
                    filter: None,
                },
                effect: Effect::LoseLife {
                    player: PlayerTarget::EachOpponent,
                    amount: EffectAmount::Fixed(1),
                },
                intervening_if: None,
                targets: vec![],

                modes: None,
                trigger_zone: None,
            },
            // CR 603.10a: "Whenever a creature an opponent controls dies, you gain 1 life."
            // PB-23: controller_opponent filter applied via DeathTriggerFilter.
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::WheneverCreatureDies {
                    controller: Some(TargetController::Opponent),
                    exclude_self: false,
                    nontoken_only: false,
                    filter: None,
                },
                effect: Effect::GainLife {
                    player: PlayerTarget::Controller,
                    amount: EffectAmount::Fixed(1),
                },
                intervening_if: None,
                targets: vec![],

                modes: None,
                trigger_zone: None,
            },
        ],
        ..Default::default()
    }
}
