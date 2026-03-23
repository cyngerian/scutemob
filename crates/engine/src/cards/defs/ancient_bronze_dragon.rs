// Ancient Bronze Dragon — {5}{G}{G}, Creature — Elder Dragon 7/7
// Flying
// Whenever this creature deals combat damage to a player, roll a d20. When you do, put X
// +1/+1 counters on each of up to two target creatures, where X is the result.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("ancient-bronze-dragon"),
        name: "Ancient Bronze Dragon".to_string(),
        mana_cost: Some(ManaCost { generic: 5, green: 2, ..Default::default() }),
        types: creature_types(&["Elder", "Dragon"]),
        oracle_text: "Flying\nWhenever this creature deals combat damage to a player, roll a d20. When you do, put X +1/+1 counters on each of up to two target creatures, where X is the result.".to_string(),
        power: Some(7),
        toughness: Some(7),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            // TODO: DSL gap — combat damage trigger with d20 roll + reflexive trigger
            // putting X counters on up to 2 targets.
        ],
        ..Default::default()
    }
}
