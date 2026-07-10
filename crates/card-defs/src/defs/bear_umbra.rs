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
                    condition: None,
                },
            },
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::PtModify,
                    modification: LayerModification::ModifyToughness(2),
                    filter: EffectFilter::AttachedCreature,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },
            // ENGINE-BLOCKED: "Whenever this creature attacks, untap all lands you control"
            // is a triggered ability GRANTED to the enchanted creature (not to Bear Umbra
            // itself). PB-AC1 shipped `Effect::UntapAll` (usable for the untap-lands effect
            // once the trigger fires) but there is still no grant-triggered-ability-to-
            // attached-object primitive in the DSL — the same gap blocks Diamond Pick-Axe's
            // "Whenever this creature attacks, create a Treasure token." Left blocked; when a
            // grant-trigger primitive ships, wire the trigger to
            // `Effect::UntapAll { filter: TargetFilter { has_card_type: Some(CardType::Land),
            // controller: You, .. } }`.
            AbilityDefinition::Keyword(KeywordAbility::UmbraArmor),
        ],
        completeness: Completeness::partial("'Whenever this creature attacks, untap all lands you control' is a triggered ability GRANTED to the enchanted creature..."),
        ..Default::default()
    }
}
