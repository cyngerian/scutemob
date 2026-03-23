// Kishla Skimmer — {G}{U}, Creature — Bird Scout 2/2
// Flying
// Whenever a card leaves your graveyard during your turn, draw a card.
// This ability triggers only once each turn.
//
// TODO: "Card leaves graveyard" trigger + once-per-turn not in DSL.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("kishla-skimmer"),
        name: "Kishla Skimmer".to_string(),
        mana_cost: Some(ManaCost { green: 1, blue: 1, ..Default::default() }),
        types: creature_types(&["Bird", "Scout"]),
        oracle_text: "Flying\nWhenever a card leaves your graveyard during your turn, draw a card. This ability triggers only once each turn.".to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            // TODO: Graveyard-leave trigger + once-per-turn not in DSL.
        ],
        ..Default::default()
    }
}
