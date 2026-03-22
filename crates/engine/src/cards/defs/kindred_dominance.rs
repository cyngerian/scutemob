// Kindred Dominance — {5}{B}{B} Sorcery
// Choose a creature type. Destroy all creatures that aren't of the chosen type.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("kindred-dominance"),
        name: "Kindred Dominance".to_string(),
        mana_cost: Some(ManaCost { generic: 5, black: 2, ..Default::default() }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "Choose a creature type. Destroy all creatures that aren't of the chosen type.".to_string(),
        // TODO: "Choose a creature type" requires runtime creature-type selection, which is not
        // expressible in the current DSL (no ChosenType negation filter for DestroyAll).
        // When dynamic creature-type choice is added (e.g., Effect::ChooseCreatureType +
        // TargetFilter::HasNotChosenType), implement as:
        //   Sequence([ChooseCreatureType, DestroyAll { filter: NotChosenType creature }])
        abilities: vec![],
        ..Default::default()
    }
}
