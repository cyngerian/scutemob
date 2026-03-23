// Butcher of Malakir — {5}{B}{B}, Creature — Vampire Warrior 5/4
// Flying
// Whenever this creature or another creature you control dies, each opponent sacrifices
// a creature of their choice.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("butcher-of-malakir"),
        name: "Butcher of Malakir".to_string(),
        mana_cost: Some(ManaCost { generic: 5, black: 2, ..Default::default() }),
        types: creature_types(&["Vampire", "Warrior"]),
        oracle_text: "Flying\nWhenever Butcher of Malakir or another creature you control dies, each opponent sacrifices a creature of their choice.".to_string(),
        power: Some(5),
        toughness: Some(4),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            // TODO: DSL gap — "this creature or another creature you control dies" trigger
            // with controller filter + ForEach EachOpponent sacrifice.
        ],
        ..Default::default()
    }
}
