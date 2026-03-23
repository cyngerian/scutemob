// Cavern-Hoard Dragon — {7}{R}{R}, Creature — Dragon 6/6
// Flying, trample, haste
// This spell costs {X} less to cast, where X is the greatest number of artifacts an
// opponent controls.
// Whenever this creature deals combat damage to a player, you create a Treasure token for
// each artifact that player controls.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("cavern-hoard-dragon"),
        name: "Cavern-Hoard Dragon".to_string(),
        mana_cost: Some(ManaCost { generic: 7, red: 2, ..Default::default() }),
        types: creature_types(&["Dragon"]),
        oracle_text: "This spell costs {X} less to cast, where X is the greatest number of artifacts an opponent controls.\nFlying, trample, haste\nWhenever this creature deals combat damage to a player, you create a Treasure token for each artifact that player controls.".to_string(),
        power: Some(6),
        toughness: Some(6),
        abilities: vec![
            // TODO: "This spell costs {X} less to cast, where X is the greatest number of
            // artifacts an opponent controls." — dynamic cost reduction based on opponent
            // artifact count not expressible in DSL (only static ReduceGenericCost(N) exists).
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            AbilityDefinition::Keyword(KeywordAbility::Trample),
            AbilityDefinition::Keyword(KeywordAbility::Haste),
            // TODO: "Create a Treasure token for each artifact that player controls."
            // DSL gap: EffectAmount/ForEach variant for artifacts controlled by the damaged
            // player does not exist. Implementing partial (1 token) would be wrong game state.
        ],
        ..Default::default()
    }
}
