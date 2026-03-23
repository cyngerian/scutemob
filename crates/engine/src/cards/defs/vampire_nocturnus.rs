// Vampire Nocturnus — {1}{B}{B}{B}, Creature — Vampire 3/3
// Play with the top card of your library revealed.
// As long as the top card of your library is black, this creature and other Vampire
// creatures you control get +2/+1 and have flying.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("vampire-nocturnus"),
        name: "Vampire Nocturnus".to_string(),
        mana_cost: Some(ManaCost { generic: 1, black: 3, ..Default::default() }),
        types: creature_types(&["Vampire"]),
        oracle_text: "Play with the top card of your library revealed.\nAs long as the top card of your library is black, this creature and other Vampire creatures you control get +2/+1 and have flying.".to_string(),
        power: Some(3),
        toughness: Some(3),
        abilities: vec![
            // TODO: DSL gap — "Play with the top card of your library revealed."
            // Hidden info reveal not in DSL.
            // TODO: DSL gap — conditional static: "As long as the top card of your library
            // is black, this creature and other Vampire creatures you control get +2/+1 and
            // have flying." Needs Condition::TopCardOfLibraryIsColor + self-inclusive tribal filter.
        ],
        ..Default::default()
    }
}
