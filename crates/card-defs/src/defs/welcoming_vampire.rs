// Welcoming Vampire — {2}{W}, Creature — Vampire 2/3
// Flying
// Whenever one or more other creatures you control with power 2 or less enter,
// draw a card. This ability triggers only once each turn.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("welcoming-vampire"),
        name: "Welcoming Vampire".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            white: 1,
            ..Default::default()
        }),
        types: creature_types(&["Vampire"]),
        oracle_text: "Flying\nWhenever one or more other creatures you control with power 2 or \
                      less enter, draw a card. This ability triggers only once each turn."
            .to_string(),
        power: Some(2),
        toughness: Some(3),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            // CR 603.2h: "Whenever one or more other creatures you control with power 2 or
            // less enter, draw a card. This ability triggers only once each turn."
            AbilityDefinition::Triggered {
                once_per_turn: true,
                trigger_condition: TriggerCondition::WheneverCreatureEntersBattlefield {
                    filter: Some(TargetFilter {
                        controller: TargetController::You,
                        max_power: Some(2),
                        ..Default::default()
                    }),
                    exclude_self: true,
                },
                effect: Effect::DrawCards {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(1),
                },
                intervening_if: None,
                targets: vec![],
                modes: None,
                trigger_zone: None,
            },
        ],
        ..Default::default()
    }
}
