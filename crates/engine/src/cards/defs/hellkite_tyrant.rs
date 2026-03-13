// Hellkite Tyrant — {4}{R}{R}, Creature — Dragon 6/5
// Flying, trample
// Whenever this creature deals combat damage to a player, gain control of all artifacts that player controls.
// At the beginning of your upkeep, if you control twenty or more artifacts, you win the game.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("hellkite-tyrant"),
        name: "Hellkite Tyrant".to_string(),
        mana_cost: Some(ManaCost { generic: 4, red: 2, ..Default::default() }),
        types: creature_types(&["Dragon"]),
        oracle_text: "Flying, trample\nWhenever this creature deals combat damage to a player, gain control of all artifacts that player controls.\nAt the beginning of your upkeep, if you control twenty or more artifacts, you win the game.".to_string(),
        power: Some(6),
        toughness: Some(5),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            AbilityDefinition::Keyword(KeywordAbility::Trample),
            // TODO: Triggered ability — whenever this deals combat damage to a player, gain control
            // of all artifacts that player controls.
            // DSL gap: no "gain control of all permanents of type" effect targeting a damaged player.
            // TODO: Triggered ability — at the beginning of your upkeep, if you control 20+ artifacts,
            // you win the game. DSL gap: no count-threshold winning condition.
        ],
        ..Default::default()
    }
}
