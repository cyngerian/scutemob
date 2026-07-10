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
            // "As this artifact enters, choose a creature type" — self-replacement (CR 614.1c).
            // Must be a Replacement (not Triggered) so chosen_creature_type is set on the
            // permanent before it fully enters the battlefield. This ensures the static anthem
            // below is active immediately — with a Triggered form, the choice would be deferred
            // to trigger-resolution, leaving chosen_creature_type=None during the ETB window.
            // Pattern mirrors Urza's Incubator, Vanquisher's Banner, Morophon (all use Replacement).
            AbilityDefinition::Replacement {
                trigger: ReplacementTrigger::WouldEnterBattlefield {
                    filter: ObjectFilter::Any,
                },
                modification: ReplacementModification::ChooseCreatureType(SubType("Human".to_string())),
                is_self: true,
                unless_condition: None,
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
