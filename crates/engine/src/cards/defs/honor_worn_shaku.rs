// Honor-Worn Shaku — {3} Artifact
// {T}: Add {C}.
// Tap an untapped legendary permanent you control: Untap this artifact.
//
// {T}: Add {C} is faithfully implemented.
// DSL gap: "Tap an untapped legendary permanent you control" as activated ability cost
//   requires Cost::TapAnotherLegendary (no such Cost variant).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("honor-worn-shaku"),
        name: "Honor-Worn Shaku".to_string(),
        mana_cost: Some(ManaCost { generic: 3, ..Default::default() }),
        types: types(&[CardType::Artifact]),
        oracle_text: "{T}: Add {C}.\nTap an untapped legendary permanent you control: Untap this artifact.".to_string(),
        abilities: vec![
            // {T}: Add {C}.
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(0, 0, 0, 0, 0, 1),
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
            },
            // TODO: Tap an untapped legendary permanent you control: Untap this artifact.
            //   (Cost enum lacks TapAnotherPermanentWithSupertype(SuperType::Legendary) variant)
        ],
        ..Default::default()
    }
}
