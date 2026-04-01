// Retraced Image — {U}, Sorcery
// Reveal a card in your hand, then put that card onto the battlefield if it has the
// same name as a permanent.
//
// TODO: Interactive — requires revealing a hand card and checking if any permanent on
// the battlefield shares its name. No "reveal from hand and conditional put onto
// battlefield" Effect exists. Leaving as Effect::Nothing.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("retraced-image"),
        name: "Retraced Image".to_string(),
        mana_cost: Some(ManaCost { blue: 1, ..Default::default() }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "Reveal a card in your hand, then put that card onto the battlefield if it has the same name as a permanent.".to_string(),
        abilities: vec![
            // TODO: Reveal-from-hand + conditional-on-permanent-name-match + put-onto-battlefield.
            // Multiple DSL gaps: reveal from hand, name matching, conditional zone change.
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
