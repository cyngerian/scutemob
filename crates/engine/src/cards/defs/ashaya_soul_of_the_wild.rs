// Ashaya, Soul of the Wild — {3}{G}{G}, Legendary Creature — Elemental */*
// Ashaya's power and toughness are each equal to the number of lands you control.
// Nontoken creatures you control are Forest lands in addition to their other types.
//
// CDA (*/*): power: None, toughness: None per KI-4.
// TODO: CDA — P/T equals the number of lands you control. DSL has no CountLandsCDA.
// Static: Nontoken creatures you control gain Land + Forest types.
// TODO: "nontoken" filter — EffectFilter::CreaturesYouControl includes tokens.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("ashaya-soul-of-the-wild"),
        name: "Ashaya, Soul of the Wild".to_string(),
        mana_cost: Some(ManaCost { generic: 3, green: 2, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Elemental"],
        ),
        oracle_text: "Ashaya, Soul of the Wild's power and toughness are each equal to the number of lands you control.\nNontoken creatures you control are Forest lands in addition to their other types.".to_string(),
        power: None,
        toughness: None,
        abilities: vec![
            // Layer 4: Add Land + Forest types to creatures you control.
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::TypeChange,
                    modification: LayerModification::AddCardTypes(
                        [CardType::Land].into_iter().collect(),
                    ),
                    filter: EffectFilter::CreaturesYouControl,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::TypeChange,
                    modification: LayerModification::AddSubtypes(
                        [SubType("Forest".to_string())].into_iter().collect(),
                    ),
                    filter: EffectFilter::CreaturesYouControl,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },
        ],
        ..Default::default()
    }
}
