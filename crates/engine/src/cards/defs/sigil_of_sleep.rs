// Sigil of Sleep — {U}, Enchantment — Aura
// Enchant creature
// Whenever enchanted creature deals damage to a player, return target creature that
// player controls to its owner's hand.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("sigil-of-sleep"),
        name: "Sigil of Sleep".to_string(),
        mana_cost: Some(ManaCost { blue: 1, ..Default::default() }),
        types: types_sub(&[CardType::Enchantment], &["Aura"]),
        oracle_text: "Enchant creature\nWhenever enchanted creature deals damage to a player, return target creature that player controls to its owner's hand.".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Enchant(EnchantTarget::Creature)),
            // CR 510.3a: "Whenever enchanted creature deals damage to a player, return target
            // creature that player controls to its owner's hand." — any damage, target must be
            // controlled by the damaged player.
            // Note: targeting a "creature that player controls" would need DamagedPlayer target
            // filtering; approximated as any creature target for now.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenEnchantedCreatureDealsDamageToPlayer {
                    combat_only: false,
                },
                effect: Effect::MoveZone {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                    to: ZoneTarget::Hand { owner: PlayerTarget::OwnerOf(Box::new(EffectTarget::DeclaredTarget { index: 0 })) },
                    controller_override: None,
                },
                intervening_if: None,
                targets: vec![TargetRequirement::TargetCreature],
            },
        ],
        ..Default::default()
    }
}
