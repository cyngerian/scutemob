// Vanquisher's Banner — {5} Artifact
// As this artifact enters, choose a creature type.
// Creatures you control of the chosen type get +1/+1.
// Whenever you cast a creature spell of the chosen type, draw a card.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("vanquishers-banner"),
        name: "Vanquisher's Banner".to_string(),
        mana_cost: Some(ManaCost { generic: 5, ..Default::default() }),
        types: types(&[CardType::Artifact]),
        oracle_text: "As this artifact enters, choose a creature type.\nCreatures you control of the chosen type get +1/+1.\nWhenever you cast a creature spell of the chosen type, draw a card.".to_string(),
        abilities: vec![
            // TODO: "As this enters, choose a creature type" — no chosen-type tracking
            // in DSL. Both the static +1/+1 buff and the cast trigger require knowing
            // the chosen type at runtime. Needs a ChosenType designation on GameObject
            // and EffectFilter::CreaturesYouControlWithChosenType variant.
        ],
        ..Default::default()
    }
}
