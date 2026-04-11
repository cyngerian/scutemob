// Obelisk of Urd — {6} Artifact (Convoke)
// As this artifact enters, choose a creature type.
// Creatures you control of the chosen type get +2/+2.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("obelisk-of-urd"),
        name: "Obelisk of Urd".to_string(),
        mana_cost: Some(ManaCost { generic: 6, ..Default::default() }),
        types: types(&[CardType::Artifact]),
        oracle_text: "Convoke (Your creatures can help cast this spell. Each creature you tap while casting this spell pays for {1} or one mana of that creature's color.)\nAs this artifact enters, choose a creature type.\nCreatures you control of the chosen type get +2/+2.".to_string(),
        abilities: vec![
            // CR 702.51: Convoke — creatures you control can help pay the mana cost.
            AbilityDefinition::Keyword(KeywordAbility::Convoke),
            // CR 603.3 / ETB: "As this artifact enters, choose a creature type."
            // Effect::ChooseCreatureType stores the choice on obj.chosen_creature_type.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                effect: Effect::ChooseCreatureType { default: SubType("Human".to_string()) },
                intervening_if: None,
                targets: vec![],
                modes: None,
                trigger_zone: None,
            },
            // CR 613.1c / Layer 7c: Static "+2/+2 to creatures you control of the chosen type."
            // EffectFilter::CreaturesYouControlOfChosenType reads chosen_creature_type from
            // the source object dynamically at layer-application time.
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::PtModify,
                    modification: LayerModification::ModifyBoth(2),
                    filter: EffectFilter::CreaturesYouControlOfChosenType,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },
        ],
        ..Default::default()
    }
}
