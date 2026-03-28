// Kenrith's Transformation — {1}{G}, Enchantment — Aura
// Enchant creature
// When Kenrith's Transformation enters, draw a card.
// Enchanted creature loses all abilities and is a green Elk creature with base
// power and toughness 3/3.
//
// TODO: "Loses all abilities and is a green Elk 3/3" — continuous effect that
//   overrides types/P&T/abilities. Needs Layer 4 type change + Layer 6 ability
//   removal + Layer 7b P/T set. Too complex for DSL.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("kenriths-transformation"),
        name: "Kenrith's Transformation".to_string(),
        mana_cost: Some(ManaCost { generic: 1, green: 1, ..Default::default() }),
        types: full_types(&[], &[CardType::Enchantment], &["Aura"]),
        oracle_text: "Enchant creature\nWhen Kenrith's Transformation enters, draw a card.\nEnchanted creature loses all abilities and is a green Elk creature with base power and toughness 3/3.".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Enchant(EnchantTarget::Creature)),
            // When ETB, draw a card.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                effect: Effect::DrawCards {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(1),
                },
                intervening_if: None,
                targets: vec![],

                modes: None,
                trigger_zone: None,
            },
            // TODO: Enchanted creature becomes green Elk 3/3 with no abilities.
        ],
        ..Default::default()
    }
}
