// Caged Sun — {6}, Artifact
// As Caged Sun enters, choose a color.
// Creatures you control of the chosen color get +1/+1.
// Whenever a land's ability causes you to add one or more mana of the chosen color,
// add an additional one mana of that color.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("caged-sun"),
        name: "Caged Sun".to_string(),
        mana_cost: Some(ManaCost { generic: 6, ..Default::default() }),
        types: types(&[CardType::Artifact]),
        oracle_text: "As Caged Sun enters, choose a color.\nCreatures you control of the chosen color get +1/+1.\nWhenever a land's ability causes you to add one or more mana of the chosen color, add an additional one mana of that color.".to_string(),
        abilities: vec![
            // CR 614.12 / CR 614.12a: "As this enters, choose a color."
            // Replacement effect — NOT a triggered ability (PB-X C1 lesson).
            // Default: White (arbitrary; deterministic fallback overrides at ETB time).
            AbilityDefinition::Replacement {
                trigger: ReplacementTrigger::WouldEnterBattlefield {
                    filter: ObjectFilter::Any,
                },
                modification: ReplacementModification::ChooseColor(Color::White),
                is_self: true,
                unless_condition: None,
            },
            // CR 613.1f / CR 105.1: Static +1/+1 to creatures you control of the chosen color.
            // EffectFilter::CreaturesYouControlOfChosenColor reads chosen_color from
            // this permanent dynamically at layer-application time.
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::PtModify,
                    modification: LayerModification::ModifyBoth(1),
                    filter: EffectFilter::CreaturesYouControlOfChosenColor,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },
            // CR 106.6a / CR 105.1: "Whenever a land's ability causes you to add one or more
            // mana of the chosen color, add an additional one mana of that color."
            // Implemented as a replacement on ManaWouldBeProduced per CR 605.3 (mana abilities
            // are stackless, so the "trigger" is implemented as an additive replacement to
            // maintain engine-wide consistency with PB-E's mana-modification pattern).
            AbilityDefinition::Replacement {
                trigger: ReplacementTrigger::ManaWouldBeProduced {
                    // PlayerId(0) is a placeholder; bound to controller at ETB registration.
                    controller: PlayerId(0),
                    // Only fires when the tapped land produces mana of the chosen color.
                    color_filter: Some(ChosenColorRef::SelfChosen),
                    // Applies to any land (Caged Sun: "a land's ability").
                    source_filter: Some(ReplacementManaSourceFilter::AnyLand),
                },
                modification: ReplacementModification::AddOneManaOfChosenColor,
                is_self: false,
                unless_condition: None,
            },
        ],
        ..Default::default()
    }
}
