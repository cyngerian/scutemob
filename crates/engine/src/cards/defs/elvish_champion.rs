// Elvish Champion — {1}{G}{G}, Creature — Elf 2/2
// Other Elf creatures get +1/+1 and have forestwalk.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("elvish-champion"),
        name: "Elvish Champion".to_string(),
        mana_cost: Some(ManaCost { generic: 1, green: 2, ..Default::default() }),
        types: creature_types(&["Elf"]),
        oracle_text: "Other Elf creatures get +1/+1 and have forestwalk. (They can't be blocked as long as defending player controls a Forest.)".to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![
            // Other Elf creatures get +1/+1.
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::PtModify,
                    modification: LayerModification::ModifyBoth(1),
                    filter: EffectFilter::OtherCreaturesYouControlWithSubtype(
                        SubType("Elf".to_string()),
                    ),
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },
            // Other Elf creatures have forestwalk.
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::Ability,
                    modification: LayerModification::AddKeyword(KeywordAbility::Landwalk(
                        LandwalkType::BasicType(SubType("Forest".to_string())),
                    )),
                    filter: EffectFilter::OtherCreaturesYouControlWithSubtype(
                        SubType("Elf".to_string()),
                    ),
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },
        ],
        ..Default::default()
    }
}
