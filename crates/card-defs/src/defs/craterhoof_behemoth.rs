// Craterhoof Behemoth — {5}{G}{G}{G}, Creature — Beast 5/5; Haste.
// ETB trigger: creatures you control gain trample and get +X/+X where X = creature count.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("craterhoof-behemoth"),
        name: "Craterhoof Behemoth".to_string(),
        mana_cost: Some(ManaCost {
            generic: 5,
            green: 3,
            ..Default::default()
        }),
        types: creature_types(&["Beast"]),
        oracle_text: "Haste\nWhen this creature enters, creatures you control gain trample and \
                      get +X/+X until end of turn, where X is the number of creatures you control."
            .to_string(),
        power: Some(5),
        toughness: Some(5),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Haste),
            // CR 603.3: ETB trigger — creatures you control gain trample and get +X/+X
            // until end of turn, X = number of creatures you control. X is locked in at
            // resolution (CR 608.2h) via ModifyBothDynamic substitution.
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                effect: Effect::Sequence(vec![
                    Effect::ApplyContinuousEffect {
                        effect_def: Box::new(ContinuousEffectDef {
                            layer: EffectLayer::PtModify,
                            modification: LayerModification::ModifyBothDynamic {
                                amount: Box::new(EffectAmount::PermanentCount {
                                    filter: TargetFilter {
                                        has_card_type: Some(CardType::Creature),
                                        ..Default::default()
                                    },
                                    controller: PlayerTarget::Controller,
                                }),
                                negate: false,
                            },
                            filter: EffectFilter::CreaturesYouControl,
                            duration: EffectDuration::UntilEndOfTurn,
                            condition: None,
                        }),
                    },
                    Effect::ApplyContinuousEffect {
                        effect_def: Box::new(ContinuousEffectDef {
                            layer: EffectLayer::Ability,
                            modification: LayerModification::AddKeywords(
                                [KeywordAbility::Trample].into_iter().collect(),
                            ),
                            filter: EffectFilter::CreaturesYouControl,
                            duration: EffectDuration::UntilEndOfTurn,
                            condition: None,
                        }),
                    },
                ]),
                intervening_if: None,
                targets: vec![],
                modes: None,
                trigger_zone: None,
            },
        ],
        ..Default::default()
    }
}
