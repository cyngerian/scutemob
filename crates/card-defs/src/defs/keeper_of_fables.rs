// Keeper of Fables — {3}{G}{G}, Creature — Cat 4/5
// Whenever one or more non-Human creatures you control deal combat damage to a player,
// draw a card.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("keeper-of-fables"),
        name: "Keeper of Fables".to_string(),
        mana_cost: Some(ManaCost {
            generic: 3,
            green: 2,
            ..Default::default()
        }),
        types: creature_types(&["Cat"]),
        oracle_text: "Whenever one or more non-Human creatures you control deal combat damage to \
                      a player, draw a card."
            .to_string(),
        power: Some(4),
        toughness: Some(5),
        abilities: vec![
            // CR 510.3a/603.2c: "Whenever one or more non-Human creatures you control deal
            // combat damage to a player, draw a card." Fires once per damaged player per
            // combat damage step (batch semantics), restricted to non-Human via filter.
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition:
                    TriggerCondition::WhenOneOrMoreCreaturesYouControlDealCombatDamageToPlayer {
                        filter: Some(TargetFilter {
                            has_card_type: Some(CardType::Creature),
                            exclude_subtypes: vec![SubType("Human".to_string())],
                            ..Default::default()
                        }),
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
