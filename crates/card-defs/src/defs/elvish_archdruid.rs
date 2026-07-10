// Elvish Archdruid — {1}{G}{G}, Creature — Elf Druid 2/2
// Other Elf creatures you control get +1/+1.
// {T}: Add {G} for each Elf you control.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("elvish-archdruid"),
        name: "Elvish Archdruid".to_string(),
        mana_cost: Some(ManaCost { generic: 1, green: 2, ..Default::default() }),
        types: creature_types(&["Elf", "Druid"]),
        oracle_text: "Other Elf creatures you control get +1/+1.\n{T}: Add {G} for each Elf you control.".to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![
            // CR 613.1c / Layer 7c: "Other Elf creatures you control get +1/+1."
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::PtModify,
                    modification: LayerModification::ModifyBoth(1),
                    filter: EffectFilter::OtherCreaturesYouControlWithSubtype(SubType("Elf".to_string())),
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },
            // {T}: Add {G} for each Elf you control.
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddManaScaled {
                    player: PlayerTarget::Controller,
                    color: ManaColor::Green,
                    count: EffectAmount::PermanentCount {
                        filter: TargetFilter {
                            has_card_type: Some(CardType::Creature),
                            has_subtype: Some(SubType("Elf".to_string())),
                            controller: TargetController::You,
                            ..Default::default()
                        },
                        controller: PlayerTarget::Controller,
                    },
                },
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
