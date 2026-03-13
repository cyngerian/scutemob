// Ancient Brass Dragon — {5}{B}{B}, Creature — Elder Dragon 7/6
// Flying
// Whenever this creature deals combat damage to a player, roll a d20. When
// you do, put any number of target creature cards with total mana value X or
// less from graveyards onto the battlefield under your control, where X is
// the result.
//
// Flying is implemented.
// TODO: DSL gap — the combat damage trigger involves a d20 roll and variable
// reanimation from graveyards based on the roll result. No dice-roll mechanic
// or variable-count reanimation exists in the DSL.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("ancient-brass-dragon"),
        name: "Ancient Brass Dragon".to_string(),
        mana_cost: Some(ManaCost { generic: 5, black: 2, ..Default::default() }),
        types: creature_types(&["Elder", "Dragon"]),
        oracle_text: "Flying\nWhenever this creature deals combat damage to a player, roll a d20. When you do, put any number of target creature cards with total mana value X or less from graveyards onto the battlefield under your control, where X is the result.".to_string(),
        power: Some(7),
        toughness: Some(6),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            // TODO: DSL gap — d20 roll + variable reanimation not expressible.
        ],
        ..Default::default()
    }
}
