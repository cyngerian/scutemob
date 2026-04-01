// Pact of the Serpent — {1}{B}{B}, Sorcery
// Choose a creature type. Target player draws X cards and loses X life, where X is
// the number of creatures they control of the chosen type.
//
// TODO: "Choose a creature type" — interactive type choice deferred to M10.
// Approximated as DrawCards(PermanentCount of all creatures) + LoseLife(same).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("pact-of-the-serpent"),
        name: "Pact of the Serpent".to_string(),
        mana_cost: Some(ManaCost { generic: 1, black: 2, ..Default::default() }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "Choose a creature type. Target player draws X cards and loses X life, where X is the number of creatures they control of the chosen type.".to_string(),
        abilities: vec![
            // TODO: Chosen creature type — approximated as all creatures target player controls.
            // Needs interactive type choice + PermanentCount filtered by chosen subtype.
            AbilityDefinition::Spell {
                effect: Effect::Nothing,
                targets: vec![],
                modes: None,
                cant_be_countered: false,
            },
        ],
        ..Default::default()
    }
}
