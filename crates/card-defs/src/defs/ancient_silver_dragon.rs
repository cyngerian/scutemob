// Ancient Silver Dragon — {6}{U}{U}, Creature — Elder Dragon 8/8
// Flying
// Whenever this creature deals combat damage to a player, roll a d20. Draw
// cards equal to the result. You have no maximum hand size for the rest of
// the game.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("ancient-silver-dragon"),
        name: "Ancient Silver Dragon".to_string(),
        mana_cost: Some(ManaCost {
            generic: 6,
            blue: 2,
            ..Default::default()
        }),
        types: creature_types(&["Elder", "Dragon"]),
        oracle_text: "Flying\nWhenever this creature deals combat damage to a player, roll a d20. \
                      Draw cards equal to the result. You have no maximum hand size for the rest \
                      of the game."
            .to_string(),
        power: Some(8),
        toughness: Some(8),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            // CR 706.2 / 706.3b: Roll d20 on combat damage to a player, draw cards equal
            // to result, then set no maximum hand size for the rest of the game. This is
            // all one triggered ability (CR 706.3b) — setting the flag is idempotent
            // across repeated combat damage triggers.
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::WhenDealsCombatDamageToPlayer,
                effect: Effect::Sequence(vec![
                    Effect::RollDice {
                        sides: 20,
                        results: vec![
                            // All results 1-20: draw cards equal to the roll result.
                            (
                                1,
                                20,
                                Effect::DrawCards {
                                    player: PlayerTarget::Controller,
                                    count: EffectAmount::LastDiceRoll,
                                },
                            ),
                        ],
                    },
                    // CR 402.2: no maximum hand size for the rest of the game.
                    Effect::SetNoMaximumHandSize {
                        player: PlayerTarget::Controller,
                    },
                ]),
                intervening_if: None,
                targets: vec![],

                modes: None,
                trigger_zone: None,
            },
        ],
        ..Default::default()
    }
}
