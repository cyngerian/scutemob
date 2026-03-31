// Incarnation Technique — {4}{B} Sorcery
// Demonstrate (When you cast this spell, you may copy it. If you do, choose an opponent
// to also copy it.)
// Mill five cards, then return a creature card from your graveyard to the battlefield.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("incarnation-technique"),
        name: "Incarnation Technique".to_string(),
        mana_cost: Some(ManaCost { generic: 4, black: 1, ..Default::default() }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "Demonstrate (When you cast this spell, you may copy it. If you do, choose an opponent to also copy it.)\nMill five cards, then return a creature card from your graveyard to the battlefield.".to_string(),
        abilities: vec![
            // TODO: Demonstrate is a keyword that triggers when the spell is cast and lets
            // you copy it (with an opponent also getting a copy). No DSL support for Demonstrate.
            // The main effect (Mill 5, then return a creature from graveyard) is partially
            // expressible, but "return a creature card from your graveyard" with no target
            // declaration (implicit — choose at resolution) is also not expressible.
            // Empty per W5 policy — wrong implementation would miss demonstrate trigger and
            // the non-targeted graveyard choice.
        ],
        ..Default::default()
    }
}
