// Broodcaller Scourge — {5}{G}{G}, Creature — Dragon 5/7
// "Flying
// Whenever one or more Dragons you control deal combat damage to a player, you may put a
// permanent card with mana value less than or equal to that damage from your hand onto
// the battlefield."
//
// Flying is implemented.
//
// TODO: DSL gap — the triggered ability requires:
// 1. A trigger that fires when one or more Dragons you control deal combat damage to a player.
// 2. EffectAmount based on the amount of damage dealt (not a fixed number).
// 3. Putting a permanent card from hand onto the battlefield filtered by mana value <= damage.
// No such combat-damage trigger with damage-amount comparison or hand-to-battlefield
// permanent deployment exists in the DSL.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("broodcaller-scourge"),
        name: "Broodcaller Scourge".to_string(),
        mana_cost: Some(ManaCost { generic: 5, green: 2, ..Default::default() }),
        types: creature_types(&["Dragon"]),
        oracle_text: "Flying\nWhenever one or more Dragons you control deal combat damage to a player, you may put a permanent card with mana value less than or equal to that damage from your hand onto the battlefield.".to_string(),
        power: Some(5),
        toughness: Some(7),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
        ],
        ..Default::default()
    }
}
