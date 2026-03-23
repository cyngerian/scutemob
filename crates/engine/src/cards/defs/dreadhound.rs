// Dreadhound — {4}{B}{B}, Creature — Demon Dog 6/6
// When this creature enters, mill three cards.
// Whenever a creature dies or a creature card is put into a graveyard from a library,
// each opponent loses 1 life.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("dreadhound"),
        name: "Dreadhound".to_string(),
        mana_cost: Some(ManaCost { generic: 4, black: 2, ..Default::default() }),
        types: creature_types(&["Demon", "Dog"]),
        oracle_text: "When Dreadhound enters, mill three cards.\nWhenever a creature dies or a creature card is put into a graveyard from a library, each opponent loses 1 life.".to_string(),
        power: Some(6),
        toughness: Some(6),
        abilities: vec![
            // TODO: DSL gap — ETB mill 3 + death/mill trigger. WheneverCreatureDies exists
            // but "creature card put into GY from library" trigger does not.
        ],
        ..Default::default()
    }
}
