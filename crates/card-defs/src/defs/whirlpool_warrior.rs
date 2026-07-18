// Whirlpool Warrior — {2}{U}, Creature — Merfolk Warrior 2/2
// When this creature enters, shuffle the cards from your hand into your
// library, then draw that many cards.
// {R}, Sacrifice this creature: Each player shuffles the cards from their
// hand into their library, then draws that many cards.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("whirlpool-warrior"),
        name: "Whirlpool Warrior".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            blue: 1,
            ..Default::default()
        }),
        types: creature_types(&["Merfolk", "Warrior"]),
        oracle_text: "When this creature enters, shuffle the cards from your hand into your \
                      library, then draw that many cards.\n{R}, Sacrifice this creature: Each \
                      player shuffles the cards from their hand into their library, then draws \
                      that many cards."
            .to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![
            // ETB: only the controller wheels their hand.
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                effect: Effect::WheelHand {
                    player: PlayerTarget::Controller,
                    disposal: WheelDisposal::ShuffleHandIntoLibrary,
                    draw: WheelDraw::ThatMany,
                },
                intervening_if: None,
                targets: vec![],
                modes: None,
                trigger_zone: None,
            },
            // {R}, Sacrifice this creature: EACH player wheels their hand.
            AbilityDefinition::Activated {
                cost: Cost::Sequence(vec![
                    Cost::Mana(ManaCost {
                        red: 1,
                        ..Default::default()
                    }),
                    Cost::SacrificeSelf,
                ]),
                effect: Effect::WheelHand {
                    player: PlayerTarget::EachPlayer,
                    disposal: WheelDisposal::ShuffleHandIntoLibrary,
                    draw: WheelDraw::ThatMany,
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
                once_per_turn: false,
            },
        ],
        ..Default::default()
    }
}
