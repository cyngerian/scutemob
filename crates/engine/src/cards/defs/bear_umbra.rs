// Bear Umbra — {2}{G}{G}, Enchantment — Aura
// Enchant creature
// Enchanted creature gets +2/+2 and has "Whenever this creature attacks, untap all lands you control."
// Umbra armor
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("bear-umbra"),
        name: "Bear Umbra".to_string(),
        mana_cost: Some(ManaCost { generic: 2, green: 2, ..Default::default() }),
        types: types_sub(&[CardType::Enchantment], &["Aura"]),
        oracle_text: "Enchant creature\nEnchanted creature gets +2/+2 and has \"Whenever this creature attacks, untap all lands you control.\"\nUmbra armor (If enchanted creature would be destroyed, instead remove all damage from it and destroy this Aura.)".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Enchant(EnchantTarget::Creature)),
            // Enchanted creature gets +2/+2 (layer 7c).
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::PtModify,
                    modification: LayerModification::ModifyPower(2),
                    filter: EffectFilter::AttachedCreature,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                },
            },
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::PtModify,
                    modification: LayerModification::ModifyToughness(2),
                    filter: EffectFilter::AttachedCreature,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                },
            },
            // TODO: DSL gap — "Whenever this creature attacks, untap all lands you control"
            // granted to enchanted creature. Requires grant-triggered-ability-to-attached-creature
            // pattern not in DSL.
            AbilityDefinition::Keyword(KeywordAbility::UmbraArmor),
        ],
        ..Default::default()
    }
}
