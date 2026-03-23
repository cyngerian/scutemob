// Goblin Bushwhacker — {R}, Creature — Goblin Warrior 1/1
// Kicker {R}
// When this creature enters, if it was kicked, creatures you control get +1/+0 and gain
// haste until end of turn.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("goblin-bushwhacker"),
        name: "Goblin Bushwhacker".to_string(),
        mana_cost: Some(ManaCost { red: 1, ..Default::default() }),
        types: creature_types(&["Goblin", "Warrior"]),
        oracle_text: "Kicker {R}\nWhen this creature enters, if it was kicked, creatures you control get +1/+0 and gain haste until end of turn.".to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Kicker),
            AbilityDefinition::Kicker {
                cost: ManaCost { red: 1, ..Default::default() },
                is_multikicker: false,
            },
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                effect: Effect::Conditional {
                    condition: Condition::WasKicked,
                    if_true: Box::new(Effect::Sequence(vec![
                        // CR 613.1c / Layer 7c: creatures you control get +1/+0 until EOT.
                        Effect::ApplyContinuousEffect {
                            effect_def: Box::new(ContinuousEffectDef {
                                layer: EffectLayer::PtModify,
                                modification: LayerModification::ModifyPower(1),
                                filter: EffectFilter::CreaturesYouControl,
                                duration: EffectDuration::UntilEndOfTurn,
                            }),
                        },
                        // CR 613.1f / Layer 6: creatures you control gain haste until EOT.
                        Effect::ApplyContinuousEffect {
                            effect_def: Box::new(ContinuousEffectDef {
                                layer: EffectLayer::Ability,
                                modification: LayerModification::AddKeyword(KeywordAbility::Haste),
                                filter: EffectFilter::CreaturesYouControl,
                                duration: EffectDuration::UntilEndOfTurn,
                            }),
                        },
                    ])),
                    if_false: Box::new(Effect::Nothing),
                },
                intervening_if: None,
                targets: vec![],
            },
        ],
        ..Default::default()
    }
}
