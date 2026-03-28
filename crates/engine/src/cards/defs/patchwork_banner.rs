// Patchwork Banner — {3} Artifact; ETB choose creature type, +1/+1 to those creatures;
// {T}: Add one mana of any color.
// TODO: "As this artifact enters, choose a creature type" and "Creatures you control of the chosen
// type get +1/+1" require ETB-choose-type and static-grant-buff-to-chosen-type, both DSL gaps.
// The self tap-for-any-color ability is implemented.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("patchwork-banner"),
        name: "Patchwork Banner".to_string(),
        mana_cost: Some(ManaCost { generic: 3, ..Default::default() }),
        types: types(&[CardType::Artifact]),
        oracle_text: "As this artifact enters, choose a creature type.\nCreatures you control of the chosen type get +1/+1.\n{T}: Add one mana of any color.".to_string(),
        abilities: vec![
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddManaAnyColor { player: PlayerTarget::Controller },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
            },
        ],
        ..Default::default()
    }
}
