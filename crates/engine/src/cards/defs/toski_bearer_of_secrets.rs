// Toski, Bearer of Secrets — {3}{G}, Legendary Creature — Squirrel 1/1
// This spell can't be countered.
// Indestructible
// Toski attacks each combat if able.
// Whenever a creature you control deals combat damage to a player, draw a card.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("toski-bearer-of-secrets"),
        name: "Toski, Bearer of Secrets".to_string(),
        mana_cost: Some(ManaCost { generic: 3, green: 1, ..Default::default() }),
        types: full_types(&[SuperType::Legendary], &[CardType::Creature], &["Squirrel"]),
        oracle_text: "This spell can't be countered.\nIndestructible\nToski attacks each combat if able.\nWhenever a creature you control deals combat damage to a player, draw a card.".to_string(),
        power: Some(1),
        toughness: Some(1),
        cant_be_countered: true,
        abilities: vec![
            // Indestructible — correct standalone.
            AbilityDefinition::Keyword(KeywordAbility::Indestructible),
            // TODO: "Toski attacks each combat if able." — MustAttack restriction not in DSL.
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
