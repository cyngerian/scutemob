// Springleaf Drum — {1} Artifact
// {T}, Tap an untapped creature you control: Add one mana of any color.
//
// DSL gap: "Tap an untapped creature you control" as part of activated ability cost requires
//   Cost::TapAnotherCreature (no such Cost variant; only Cost::Tap taps this permanent).
// W5 policy: cannot faithfully express this — abilities: vec![].
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("springleaf-drum"),
        name: "Springleaf Drum".to_string(),
        mana_cost: Some(ManaCost { generic: 1, ..Default::default() }),
        types: types(&[CardType::Artifact]),
        oracle_text: "{T}, Tap an untapped creature you control: Add one mana of any color.".to_string(),
        abilities: vec![
            // TODO: {T}, Tap an untapped creature you control: Add one mana of any color.
            //   (Cost enum lacks TapAnotherUntappedCreature variant)
        ],
        ..Default::default()
    }
}
