// Vanquisher's Banner — {5} Artifact
// As this artifact enters, choose a creature type.
// Creatures you control of the chosen type get +1/+1.
// Whenever you cast a creature spell of the chosen type, draw a card.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("vanquishers-banner"),
        name: "Vanquisher's Banner".to_string(),
        mana_cost: Some(ManaCost { generic: 5, ..Default::default() }),
        types: types(&[CardType::Artifact]),
        oracle_text: "As this artifact enters, choose a creature type.\nCreatures you control of the chosen type get +1/+1.\nWhenever you cast a creature spell of the chosen type, draw a card.".to_string(),
        abilities: vec![
            // "As this enters, choose a creature type" — self-replacement (CR 614.1c)
            AbilityDefinition::Replacement {
                trigger: ReplacementTrigger::WouldEnterBattlefield {
                    filter: ObjectFilter::Any,
                },
                modification: ReplacementModification::ChooseCreatureType(SubType("Human".to_string())),
                is_self: true,
                unless_condition: None,
            },
            // "Creatures you control of the chosen type get +1/+1" (CR 205.3m)
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::PtModify,
                    modification: LayerModification::ModifyBoth(1),
                    filter: EffectFilter::CreaturesYouControlOfChosenType,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },
            // "Whenever you cast a creature spell of the chosen type, draw a card."
            // CR 603.1: chosen_subtype_filter fires only for spells matching source's chosen type.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WheneverYouCastSpell {
                    during_opponent_turn: false,
                    spell_type_filter: Some(vec![CardType::Creature]),
                    noncreature_only: false,
                    chosen_subtype_filter: true,
                },
                effect: Effect::DrawCards {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(1),
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
