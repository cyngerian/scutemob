// Twilight Prophet — {2}{B}{B}, Creature — Vampire Cleric 2/4
// Flying, Ascend; upkeep trigger (with city's blessing): reveal top, draw it, opponents lose X, gain X
// TODO: Upkeep drain trigger requires reveal-top + mana-value-based DrainLife — DSL gap
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("twilight-prophet"),
        name: "Twilight Prophet".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            black: 2,
            ..Default::default()
        }),
        types: creature_types(&["Vampire", "Cleric"]),
        oracle_text: "Flying\nAscend (If you control ten or more permanents, you get the city's blessing for the rest of the game.)\nAt the beginning of your upkeep, if you have the city's blessing, reveal the top card of your library and put it into your hand. Each opponent loses X life and you gain X life, where X is that card's mana value.".to_string(),
        power: Some(2),
        toughness: Some(4),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            // Ascend keyword — triggers city's blessing check
            AbilityDefinition::Keyword(KeywordAbility::Ascend),
            // TODO: Upkeep trigger conditioned on city's blessing, with drain-life based on
            // revealed card's mana value — requires EffectAmount::ManaValueOfRevealed or similar.
        ],
        ..Default::default()
    }
}
