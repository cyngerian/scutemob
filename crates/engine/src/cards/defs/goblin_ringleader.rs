// Goblin Ringleader — {3}{R}, Creature — Goblin 2/2
// Haste
// TODO: DSL gap — ETB triggered ability "reveal top four cards of your library, put all Goblin
//   cards into your hand and the rest on the bottom in any order."
//   (reveal with subtype filter into hand not supported; SearchLibrary only supports basic lands
//   to battlefield)
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("goblin-ringleader"),
        name: "Goblin Ringleader".to_string(),
        mana_cost: Some(ManaCost {
            generic: 3,
            red: 1,
            ..Default::default()
        }),
        types: creature_types(&["Goblin"]),
        oracle_text: "Haste (This creature can attack and {T} as soon as it comes under your control.)\nWhen this creature enters, reveal the top four cards of your library. Put all Goblin cards revealed this way into your hand and the rest on the bottom of your library in any order.".to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Haste),
        ],
        ..Default::default()
    }
}
