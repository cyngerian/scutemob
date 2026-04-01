// Herald's Horn — {3}, Artifact
// As this enters, choose a creature type.
// Creature spells of the chosen type cost {1} less to cast.
// At the beginning of your upkeep, look at the top card of your library. If it's
// a creature card of the chosen type, you may reveal it and put it into your hand.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("heralds-horn"),
        name: "Herald's Horn".to_string(),
        mana_cost: Some(ManaCost { generic: 3, ..Default::default() }),
        types: types(&[CardType::Artifact]),
        oracle_text: "As Herald's Horn enters, choose a creature type.\nCreature spells you cast of the chosen type cost {1} less to cast.\nAt the beginning of your upkeep, look at the top card of your library. If it's a creature card of the chosen type, you may reveal it and put it into your hand.".to_string(),
        abilities: vec![
            // TODO: "Choose a creature type" on ETB + type-specific cost reduction.
            // TODO: Upkeep look-at-top + conditional draw for chosen type.
            // Cost reduction for a chosen creature type needs ChosenType infrastructure.
        ],
        ..Default::default()
    }
}
