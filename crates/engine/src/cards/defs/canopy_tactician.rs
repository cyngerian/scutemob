// Canopy Tactician — {3}{G}, Creature — Elf Warrior 3/3
// Other Elves you control get +1/+1.
// {T}: Add {G}{G}{G}.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("canopy-tactician"),
        name: "Canopy Tactician".to_string(),
        mana_cost: Some(ManaCost { generic: 3, green: 1, ..Default::default() }),
        types: creature_types(&["Elf", "Warrior"]),
        oracle_text: "Other Elves you control get +1/+1.\n{T}: Add {G}{G}{G}.".to_string(),
        power: Some(3),
        toughness: Some(3),
        abilities: vec![
            // CR 613.1c / Layer 7c: "Other Elves you control get +1/+1."
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::PtModify,
                    modification: LayerModification::ModifyBoth(1),
                    filter: EffectFilter::OtherCreaturesYouControlWithSubtype(SubType("Elf".to_string())),
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },
            // {T}: Add {G}{G}{G}.
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(0, 0, 0, 0, 3, 0),
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
