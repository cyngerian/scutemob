// Mirkwood Bats — {3}{B}, Creature — Bat 2/3
// Flying
// Whenever you create or sacrifice a token, each opponent loses 1 life.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("mirkwood-bats"),
        name: "Mirkwood Bats".to_string(),
        mana_cost: Some(ManaCost { generic: 3, black: 1, ..Default::default() }),
        types: creature_types(&["Bat"]),
        oracle_text: "Flying\nWhenever you create or sacrifice a token, each opponent loses 1 life.".to_string(),
        power: Some(2),
        toughness: Some(3),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            // TODO: "Whenever you create or sacrifice a token" trigger not in DSL.
        ],
        ..Default::default()
    }
}
