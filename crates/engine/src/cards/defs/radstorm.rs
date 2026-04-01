// Radstorm — {2}{U}, Instant
// Storm (When you cast this spell, copy it for each spell cast before it this turn.)
// Proliferate. (Choose any number of permanents and/or players, then give each another
// counter of each kind already there.)
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("radstorm"),
        name: "Radstorm".to_string(),
        mana_cost: Some(ManaCost { generic: 2, blue: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Storm (When you cast this spell, copy it for each spell cast before it this turn. You may choose new targets for the copies.)\nProliferate. (Choose any number of permanents and/or players, then give each another counter of each kind already there.)".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Storm),
            AbilityDefinition::Spell {
                effect: Effect::Proliferate,
                targets: vec![],
                modes: None,
                cant_be_countered: false,
            },
        ],
        ..Default::default()
    }
}
