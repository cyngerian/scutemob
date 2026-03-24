// Triumph of the Hordes — {2}{G}{G}, Sorcery
// Until end of turn, creatures you control get +1/+1 and gain trample and infect.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("triumph-of-the-hordes"),
        name: "Triumph of the Hordes".to_string(),
        mana_cost: Some(ManaCost { generic: 2, green: 2, ..Default::default() }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "Until end of turn, creatures you control get +1/+1 and gain trample and infect.".to_string(),
        abilities: vec![
            AbilityDefinition::Spell {
                effect: Effect::Sequence(vec![
                    // CR 613.1c / Layer 7c: creatures you control get +1/+1 until EOT.
                    Effect::ApplyContinuousEffect {
                        effect_def: Box::new(ContinuousEffectDef {
                            layer: EffectLayer::PtModify,
                            modification: LayerModification::ModifyBoth(1),
                            filter: EffectFilter::CreaturesYouControl,
                            duration: EffectDuration::UntilEndOfTurn,
                            condition: None,
                        }),
                    },
                    // CR 613.1f / Layer 6: creatures you control gain trample until EOT.
                    Effect::ApplyContinuousEffect {
                        effect_def: Box::new(ContinuousEffectDef {
                            layer: EffectLayer::Ability,
                            modification: LayerModification::AddKeyword(KeywordAbility::Trample),
                            filter: EffectFilter::CreaturesYouControl,
                            duration: EffectDuration::UntilEndOfTurn,
                            condition: None,
                        }),
                    },
                    // CR 613.1f / Layer 6: creatures you control gain infect until EOT.
                    Effect::ApplyContinuousEffect {
                        effect_def: Box::new(ContinuousEffectDef {
                            layer: EffectLayer::Ability,
                            modification: LayerModification::AddKeyword(KeywordAbility::Infect),
                            filter: EffectFilter::CreaturesYouControl,
                            duration: EffectDuration::UntilEndOfTurn,
                            condition: None,
                        }),
                    },
                ]),
                targets: vec![],
                modes: None,
                cant_be_countered: false,
            },
        ],
        ..Default::default()
    }
}
