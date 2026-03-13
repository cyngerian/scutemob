// Lightning, Army of One — {1}{R}{W}, Legendary Creature — Human Soldier 3/2
// First strike, trample, lifelink
// Stagger — Whenever Lightning deals combat damage to a player, until your next turn,
// if a source would deal damage to that player or a permanent that player controls,
// it deals double that damage instead.
//
// First strike, trample, and lifelink are implemented.
//
// TODO: DSL gap — Stagger triggered ability "until your next turn, if a source would
// deal damage to that player or a permanent that player controls, it deals double that
// damage instead" requires a replacement effect with a duration of "until your next turn"
// and a scope of "all damage to a specific player or their permanents". Neither the
// duration variant nor the damage-doubling replacement effect are expressible in the DSL.
// Omitted.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("lightning-army-of-one"),
        name: "Lightning, Army of One".to_string(),
        mana_cost: Some(ManaCost { generic: 1, red: 1, white: 1, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Human", "Soldier"],
        ),
        oracle_text: "First strike, trample, lifelink\nStagger — Whenever Lightning deals combat damage to a player, until your next turn, if a source would deal damage to that player or a permanent that player controls, it deals double that damage instead.".to_string(),
        power: Some(3),
        toughness: Some(2),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::FirstStrike),
            AbilityDefinition::Keyword(KeywordAbility::Trample),
            AbilityDefinition::Keyword(KeywordAbility::Lifelink),
        ],
        ..Default::default()
    }
}
