// Aqueous Form — {U}, Enchantment — Aura
// Enchant creature
// Enchanted creature can't be blocked.
// Whenever enchanted creature attacks, scry 1.
//
// TODO: "Whenever enchanted creature attacks, scry 1" — needs a trigger condition
//   for "enchanted creature attacks" (TriggerCondition has no WhenEnchantedCreatureAttacks).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("aqueous-form"),
        name: "Aqueous Form".to_string(),
        mana_cost: Some(ManaCost { blue: 1, ..Default::default() }),
        types: types_sub(&[CardType::Enchantment], &["Aura"]),
        oracle_text: "Enchant creature\nEnchanted creature can't be blocked.\nWhenever enchanted creature attacks, scry 1.".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Enchant(EnchantTarget::Creature)),
            // Enchanted creature can't be blocked — static grant
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::Ability,
                    modification: LayerModification::AddKeyword(KeywordAbility::CantBeBlocked),
                    filter: EffectFilter::AttachedCreature,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },
        ],
        ..Default::default()
    }
}
