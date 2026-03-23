// Scourge of the Throne — {4}{R}{R}, Creature — Dragon 5/5
// Flying
// Dethrone
// Whenever this creature attacks for the first time each turn, if it's attacking the player
// with the most life or tied for most life, untap all attacking creatures. After this phase,
// there is an additional combat phase.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("scourge-of-the-throne"),
        name: "Scourge of the Throne".to_string(),
        mana_cost: Some(ManaCost { generic: 4, red: 2, ..Default::default() }),
        types: creature_types(&["Dragon"]),
        oracle_text: "Flying\nDethrone (Whenever this creature attacks the player with the most life or tied for most life, put a +1/+1 counter on it.)\nWhenever Scourge of the Throne attacks for the first time each turn, if it's attacking the player with the most life or tied for most life, untap all attacking creatures. After this phase, there is an additional combat phase.".to_string(),
        power: Some(5),
        toughness: Some(5),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            AbilityDefinition::Keyword(KeywordAbility::Dethrone),
            // TODO: DSL gap — "first time each turn" attack trigger with most-life condition
            // + untap all attacking creatures + additional combat phase.
        ],
        ..Default::default()
    }
}
