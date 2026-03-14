// Brutal Cathar // Moonrage Brute — DFC with Daybound/Nightbound (CR 702.146)
// Front: {2}{W} Human Soldier Werewolf 2/2, Daybound,
//        when ETB exile target creature opponent controls until this leaves battlefield
// Back:  Moonrage Brute, Werewolf 3/3, Nightbound, first strike, ward {pay 3 life}
//
// DSL gap: ETB exile-until-leaves (ExileUntilLeaves effect not yet in DSL).
// Daybound/Nightbound keywords and back_face are faithfully represented.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("brutal-cathar-moonrage-brute"),
        name: "Brutal Cathar".to_string(),
        mana_cost: Some(ManaCost { generic: 2, white: 1, ..Default::default() }),
        types: creature_types(&["Human", "Soldier", "Werewolf"]),
        oracle_text: "When this creature enters the battlefield, exile target creature an opponent controls until this creature leaves the battlefield.\nDaybound".to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Daybound),
            // DSL gap: ETB exile-until-leaves requires ExileUntilLeaves effect
        ],
        color_indicator: None,
        back_face: Some(CardFace {
            name: "Moonrage Brute".to_string(),
            mana_cost: None,
            types: creature_types(&["Werewolf"]),
            oracle_text: "First strike\nWard—Pay 3 life.\nNightbound".to_string(),
            power: Some(3),
            toughness: Some(3),
            abilities: vec![
                AbilityDefinition::Keyword(KeywordAbility::Nightbound),
                AbilityDefinition::Keyword(KeywordAbility::FirstStrike),
                // DSL gap: Ward—Pay 3 life (Ward(u32) only supports mana costs)
            ],
            color_indicator: Some(vec![Color::White]),
        }),
    }
}
