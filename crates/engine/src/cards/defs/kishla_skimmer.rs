// Kishla Skimmer — {G}{U}, Creature — Bird Scout 2/2
// Flying
// Whenever a card leaves your graveyard during your turn, draw a card.
// This ability triggers only once each turn.
//
// ENGINE-BLOCKED: "Whenever a card leaves your graveyard during your turn, draw a card."
// PB-AC1 shipped the `once_per_turn` limiter, but there is no `TriggerCondition` for a card
// leaving the graveyard (mill/exile-from-graveyard/cast-from-graveyard/etc. as a single
// unified "leaves graveyard" event) anywhere in the DSL. Stays fully blocked.
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
            // ENGINE-BLOCKED: see header — no "leaves graveyard" TriggerCondition in the DSL.
        ],
        completeness: Completeness::partial("'Whenever a card leaves your graveyard during your turn, draw a card.' PB-AC1 shipped the `once_per_turn` limiter, but..."),
        ..Default::default()
    }
}
