// Ancient Silver Dragon — {6}{U}{U}, Creature — Elder Dragon 8/8
// Flying
// Whenever this creature deals combat damage to a player, roll a d20. Draw
// cards equal to the result. You have no maximum hand size for the rest of
// the game.
//
// Flying is implemented. D20 roll + variable card draw implemented.
// TODO: DSL gap — "no maximum hand size for the rest of the game" requires
// a permanent player designation (not a continuous effect from a permanent).
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
            // CR 706.2: Roll d20 on combat damage to a player, draw cards equal to result.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenDealsCombatDamageToPlayer,
                effect: Effect::RollDice {
                    sides: 20,
                    results: vec![
                        // All results 1-20: draw cards equal to the roll result.
                        (1, 20, Effect::DrawCards {
                            player: PlayerTarget::Controller,
                            count: EffectAmount::LastDiceRoll,
                        }),
                    ],
                },
                intervening_if: None,
                targets: vec![],
            },
            // TODO: "no maximum hand size for the rest of the game" — needs permanent player designation
        ],
        ..Default::default()
    }
}
