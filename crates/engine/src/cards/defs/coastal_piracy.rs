// Coastal Piracy — {2}{U}{U}, Enchantment
// Whenever a creature you control deals combat damage to an opponent, you may draw a card.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("coastal-piracy"),
        name: "Coastal Piracy".to_string(),
        mana_cost: Some(ManaCost { generic: 2, blue: 2, ..Default::default() }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "Whenever a creature you control deals combat damage to an opponent, you may draw a card.".to_string(),
        abilities: vec![
            // CR 510.3a: "Whenever a creature you control deals combat damage to a player,
            // draw a card." PB-23: WheneverCreatureYouControlDealsCombatDamageToPlayer.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WheneverCreatureYouControlDealsCombatDamageToPlayer { filter: None },
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
