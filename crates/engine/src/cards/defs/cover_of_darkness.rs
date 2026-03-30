// Cover of Darkness — {1}{B}, Enchantment
// As Cover of Darkness enters, choose a creature type.
// Creatures of the chosen type have fear.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("cover-of-darkness"),
        name: "Cover of Darkness".to_string(),
        mana_cost: Some(ManaCost { generic: 1, black: 1, ..Default::default() }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "As Cover of Darkness enters, choose a creature type.\nCreatures of the chosen type have fear. (They can't be blocked except by artifact creatures and/or black creatures.)".to_string(),
        abilities: vec![
            // TODO: "As this enters, choose a creature type" — no ChooseCreatureType
            // effect or chosen_subtype field on GameObject. The static grant of Fear
            // depends on the chosen type (Layer 6).
        ],
        ..Default::default()
    }
}
