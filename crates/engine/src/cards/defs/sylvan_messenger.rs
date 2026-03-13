// Sylvan Messenger — {3}{G}, Creature — Elf 2/2
// "Trample (This creature can deal excess combat damage to the player or planeswalker it's
// attacking.)
// When this creature enters, reveal the top four cards of your library. Put all Elf cards
// revealed this way into your hand and the rest on the bottom of your library in any order."
//
// Trample is implemented.
//
// TODO: DSL gap — the ETB trigger requires:
// 1. Revealing the top N cards of your library.
// 2. Filtering by creature subtype (Elf) and putting matching cards into hand.
// 3. Putting the rest on the bottom of the library in any order.
// No RevealTopN + subtype filter + split-destination effect exists in the DSL.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("sylvan-messenger"),
        name: "Sylvan Messenger".to_string(),
        mana_cost: Some(ManaCost { generic: 3, green: 1, ..Default::default() }),
        types: creature_types(&["Elf"]),
        oracle_text: "Trample (This creature can deal excess combat damage to the player or planeswalker it's attacking.)\nWhen this creature enters, reveal the top four cards of your library. Put all Elf cards revealed this way into your hand and the rest on the bottom of your library in any order.".to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Trample),
        ],
        ..Default::default()
    }
}
