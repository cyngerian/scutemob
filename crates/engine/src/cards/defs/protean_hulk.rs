// Protean Hulk — {5}{G}{G}, Creature — Beast 6/6
// When this creature dies, search your library for any number of creature cards with total
// mana value 6 or less, put them onto the battlefield, then shuffle.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("protean-hulk"),
        name: "Protean Hulk".to_string(),
        mana_cost: Some(ManaCost { generic: 5, green: 2, ..Default::default() }),
        types: creature_types(&["Beast"]),
        oracle_text: "When Protean Hulk dies, search your library for any number of creature cards with total mana value 6 or less, put them onto the battlefield, then shuffle.".to_string(),
        power: Some(6),
        toughness: Some(6),
        abilities: vec![
            // TODO: DSL gap — death trigger + multi-card search with total MV constraint.
            // SearchLibrary finds one card; "any number with total MV 6 or less" needs
            // multi-card search with cumulative cost tracking (M10 player choice).
        ],
        ..Default::default()
    }
}
