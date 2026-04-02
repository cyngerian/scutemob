// Scourge of Fleets — {5}{U}{U}, Creature — Kraken 6/6
// When this creature enters, return each creature your opponents control with toughness X
// or less to its owner's hand, where X is the number of Islands you control.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("scourge-of-fleets"),
        name: "Scourge of Fleets".to_string(),
        mana_cost: Some(ManaCost { generic: 5, blue: 2, ..Default::default() }),
        types: creature_types(&["Kraken"]),
        oracle_text: "When this creature enters, return each creature your opponents control with toughness X or less to its owner's hand, where X is the number of Islands you control.".to_string(),
        power: Some(6),
        toughness: Some(6),
        abilities: vec![
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                effect: Effect::BounceAll {
                    filter: TargetFilter {
                        has_card_type: Some(CardType::Creature),
                        controller: TargetController::Opponent,
                        ..Default::default()
                    },
                    max_toughness_amount: Some(EffectAmount::PermanentCount {
                        filter: TargetFilter {
                            has_subtype: Some(SubType("Island".to_string())),
                            ..Default::default()
                        },
                        controller: PlayerTarget::Controller,
                    }),
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
