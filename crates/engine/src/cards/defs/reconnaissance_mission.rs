// Reconnaissance Mission — {2}{U}{U}, Enchantment
// Whenever a creature you control deals combat damage to a player, you may draw a card.
// Cycling {2}
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("reconnaissance-mission"),
        name: "Reconnaissance Mission".to_string(),
        mana_cost: Some(ManaCost { generic: 2, blue: 2, ..Default::default() }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "Whenever a creature you control deals combat damage to a player, you may draw a card.\nCycling {2}".to_string(),
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
            },
            // Cycling {2}
            AbilityDefinition::Keyword(KeywordAbility::Cycling),
            AbilityDefinition::Cycling {
                cost: ManaCost { generic: 2, ..Default::default() },
            },
        ],
        ..Default::default()
    }
}
