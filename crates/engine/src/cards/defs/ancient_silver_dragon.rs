// Ancient Silver Dragon — {6}{U}{U}, Creature — Elder Dragon 8/8
// Flying
// Whenever this creature deals combat damage to a player, roll a d20. Draw
// cards equal to the result. You have no maximum hand size for the rest of
// the game.
//
// Flying is implemented.
// TODO: DSL gap — d20 roll with variable card draw and "no maximum hand size"
// continuous effect not expressible. No dice-roll mechanic exists in the DSL.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("ancient-silver-dragon"),
        name: "Ancient Silver Dragon".to_string(),
        mana_cost: Some(ManaCost { generic: 6, blue: 2, ..Default::default() }),
        types: creature_types(&["Elder", "Dragon"]),
        oracle_text: "Flying\nWhenever this creature deals combat damage to a player, roll a d20. Draw cards equal to the result. You have no maximum hand size for the rest of the game.".to_string(),
        power: Some(8),
        toughness: Some(8),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            // TODO: DSL gap — d20 roll + variable card draw + no hand size limit not expressible.
        ],
        ..Default::default()
    }
}
