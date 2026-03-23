// Smothering Abomination — {2}{B}{B}, Creature — Eldrazi 4/3
// Devoid
// Flying
// At the beginning of your upkeep, sacrifice a creature.
// Whenever you sacrifice a creature, draw a card.
//
// TODO: "Whenever you sacrifice a creature" trigger not in DSL.
// TODO: Forced sacrifice on upkeep not expressible.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("smothering-abomination"),
        name: "Smothering Abomination".to_string(),
        mana_cost: Some(ManaCost { generic: 2, black: 2, ..Default::default() }),
        types: creature_types(&["Eldrazi"]),
        oracle_text: "Devoid\nFlying\nAt the beginning of your upkeep, sacrifice a creature.\nWhenever you sacrifice a creature, draw a card.".to_string(),
        power: Some(4),
        toughness: Some(3),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Devoid),
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            // TODO: Sacrifice triggers not in DSL.
        ],
        ..Default::default()
    }
}
