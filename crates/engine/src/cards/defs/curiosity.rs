// Curiosity — {U}, Enchantment — Aura
// Enchant creature
// Whenever enchanted creature deals damage to an opponent, you may draw a card.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("curiosity"),
        name: "Curiosity".to_string(),
        mana_cost: Some(ManaCost { blue: 1, ..Default::default() }),
        types: types_sub(&[CardType::Enchantment], &["Aura"]),
        oracle_text: "Enchant creature\nWhenever enchanted creature deals damage to an opponent, you may draw a card.".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Enchant(EnchantTarget::Creature)),
            // CR 510.3a: "Whenever enchanted creature deals damage to an opponent, you may
            // draw a card." — enchanted creature trigger (any damage, not combat-only).
            // Note: combat_only: false covers both combat and noncombat damage.
            // TODO(PB-37): approximation — oracle says "an opponent" but
            // WhenEnchantedCreatureDealsDamageToPlayer fires on damage to ANY player (including
            // self if damage is redirected). In multiplayer Commander this can matter.
            // Also, the noncombat damage path (combat_only: false) is not yet dispatched from
            // GameEvent::DamageDealt — deferred to PB-37.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenEnchantedCreatureDealsDamageToPlayer {
                    combat_only: false,
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
