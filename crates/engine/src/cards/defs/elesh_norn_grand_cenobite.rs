// Elesh Norn, Grand Cenobite — {5}{W}{W}, Legendary Creature — Phyrexian Praetor 4/7
// Vigilance
// Other creatures you control get +2/+2.
// Creatures your opponents control get -2/-2.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("elesh-norn-grand-cenobite"),
        name: "Elesh Norn, Grand Cenobite".to_string(),
        mana_cost: Some(ManaCost { generic: 5, white: 2, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Phyrexian", "Praetor"],
        ),
        oracle_text: "Vigilance\nOther creatures you control get +2/+2.\nCreatures your opponents control get -2/-2.".to_string(),
        power: Some(4),
        toughness: Some(7),
        abilities: vec![
            // CR 702.20a: Vigilance.
            AbilityDefinition::Keyword(KeywordAbility::Vigilance),
            // CR 613.4c (Layer 7c): "Other creatures you control get +2/+2."
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::PtModify,
                    modification: LayerModification::ModifyBoth(2),
                    filter: EffectFilter::OtherCreaturesYouControl,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },
            // CR 613.4c (Layer 7c): "Creatures your opponents control get -2/-2."
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::PtModify,
                    modification: LayerModification::ModifyBoth(-2),
                    filter: EffectFilter::CreaturesOpponentsControl,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },
        ],
        ..Default::default()
    }
}
