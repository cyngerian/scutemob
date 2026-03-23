// Ohran Frostfang — {3}{G}{G}, Snow Creature — Snake 2/6
// Attacking creatures you control have deathtouch.
// Whenever a creature you control deals combat damage to a player, draw a card.
//
// TODO: Per-creature combat damage trigger not in DSL.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("ohran-frostfang"),
        name: "Ohran Frostfang".to_string(),
        mana_cost: Some(ManaCost { generic: 3, green: 2, ..Default::default() }),
        types: full_types(&[SuperType::Snow], &[CardType::Creature], &["Snake"]),
        oracle_text: "Attacking creatures you control have deathtouch.\nWhenever a creature you control deals combat damage to a player, draw a card.".to_string(),
        power: Some(2),
        toughness: Some(6),
        abilities: vec![
            // TODO: "Attacking creatures you control have deathtouch" — conditional static grant
            // (filtering by "attacking" status) not expressible in current DSL.
            // CR 510.3a: "Whenever a creature you control deals combat damage to a player,
            // draw a card." PB-23: WheneverCreatureYouControlDealsCombatDamageToPlayer.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WheneverCreatureYouControlDealsCombatDamageToPlayer,
                effect: Effect::DrawCards {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(1),
                },
                intervening_if: None,
                targets: vec![],
            },
        ],
        ..Default::default()
    }
}
