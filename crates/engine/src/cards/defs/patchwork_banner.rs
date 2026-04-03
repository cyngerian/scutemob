// Patchwork Banner — {3} Artifact; ETB choose creature type, +1/+1 to those creatures;
// {T}: Add one mana of any color.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("patchwork-banner"),
        name: "Patchwork Banner".to_string(),
        mana_cost: Some(ManaCost { generic: 3, ..Default::default() }),
        types: types(&[CardType::Artifact]),
        oracle_text: "As this artifact enters, choose a creature type.\nCreatures you control of the chosen type get +1/+1.\n{T}: Add one mana of any color.".to_string(),
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
            // {T}: Add one mana of any color.
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddManaAnyColor { player: PlayerTarget::Controller },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
                once_per_turn: false,
            },
        ],
        ..Default::default()
    }
}
