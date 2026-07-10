// Niv-Mizzet, Visionary — {4}{U}{R}, Legendary Creature — Dragon Wizard 5/5
// Flying
// You have no maximum hand size.
// Whenever a source you control deals noncombat damage to an opponent, you draw that many cards.
//
// Flying is implemented.
//
// "You have no maximum hand size" now expressed via KeywordAbility::NoMaxHandSize (PB-AC8).
//
// TODO: ENGINE-BLOCKED — "whenever a source you control deals noncombat damage to an opponent,
// draw that many cards" requires a TriggerCondition for any-source noncombat damage events
// with a variable draw amount equal to the damage dealt. Neither is expressible in the DSL.
// Omitted.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("niv-mizzet-visionary"),
        name: "Niv-Mizzet, Visionary".to_string(),
        mana_cost: Some(ManaCost { generic: 4, blue: 1, red: 1, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Dragon", "Wizard"],
        ),
        oracle_text: "Flying\nYou have no maximum hand size.\nWhenever a source you control deals noncombat damage to an opponent, you draw that many cards.".to_string(),
        power: Some(5),
        toughness: Some(5),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            AbilityDefinition::Keyword(KeywordAbility::NoMaxHandSize),
            // TODO: ENGINE-BLOCKED — any-source noncombat-damage trigger with variable
            // draw amount gap (see header).
        ],
        ..Default::default()
    }
}
