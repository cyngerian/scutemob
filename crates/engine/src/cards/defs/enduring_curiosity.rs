// Enduring Curiosity — {2}{U}{U}, Enchantment Creature — Cat Glimmer 4/3
// Flash
// Whenever a creature you control deals combat damage to a player, draw a card.
// When Enduring Curiosity dies, if it was a creature, return it to the battlefield
// under its owner's control. It's an enchantment. (It's not a creature.)
//
// TODO: Death return as enchantment-only — Glimmer mechanic not in DSL.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("enduring-curiosity"),
        name: "Enduring Curiosity".to_string(),
        mana_cost: Some(ManaCost { generic: 2, blue: 2, ..Default::default() }),
        types: full_types(&[], &[CardType::Enchantment, CardType::Creature], &["Cat", "Glimmer"]),
        oracle_text: "Flash\nWhenever a creature you control deals combat damage to a player, draw a card.\nWhen Enduring Curiosity dies, if it was a creature, return it to the battlefield under its owner's control. It's an enchantment.".to_string(),
        power: Some(4),
        toughness: Some(3),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flash),
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
            // TODO: Glimmer death return mechanic (becomes non-creature enchantment) not in DSL.
        ],
        ..Default::default()
    }
}
