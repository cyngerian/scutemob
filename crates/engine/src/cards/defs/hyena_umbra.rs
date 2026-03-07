// Hyena Umbra — {W}, Enchantment — Aura.
// "Enchant creature. Enchanted creature gets +1/+1 and has first strike.
// Umbra armor (If enchanted creature would be destroyed, instead remove all
// damage from it and destroy this Aura.)"
//
// CR 702.5a: Enchant creature — restricts casting target and legal attachments.
// CR 613.4c: +1/+1 is a layer-7c P/T-modifying effect (two separate modifications).
// CR 702.7a: First Strike keyword granted in layer 6.
// CR 702.89a: Umbra armor — replacement effect on the Aura itself; engine scans
// battlefield Auras with UmbraArmor keyword when a creature would be destroyed.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("hyena-umbra"),
        name: "Hyena Umbra".to_string(),
        mana_cost: Some(ManaCost { white: 1, ..Default::default() }),
        types: types_sub(&[CardType::Enchantment], &["Aura"]),
        oracle_text: "Enchant creature\nEnchanted creature gets +1/+1 and has first strike.\nUmbra armor (If enchanted creature would be destroyed, instead remove all damage from it and destroy this Aura.)".to_string(),
        abilities: vec![
            // CR 702.5a: Enchant creature — defines legal targets and attachment restriction.
            AbilityDefinition::Keyword(KeywordAbility::Enchant(EnchantTarget::Creature)),
            // CR 613.4c: Enchanted creature gets +1 power (layer 7c, P/T modify).
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::PtModify,
                    modification: LayerModification::ModifyPower(1),
                    filter: EffectFilter::AttachedCreature,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                },
            },
            // CR 613.4c: Enchanted creature gets +1 toughness (layer 7c, P/T modify).
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::PtModify,
                    modification: LayerModification::ModifyToughness(1),
                    filter: EffectFilter::AttachedCreature,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                },
            },
            // CR 702.7a: Enchanted creature has first strike (layer 6, ability grant).
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::Ability,
                    modification: LayerModification::AddKeyword(KeywordAbility::FirstStrike),
                    filter: EffectFilter::AttachedCreature,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                },
            },
            // CR 702.89a: Umbra armor lives on the Aura itself. The engine's
            // check_umbra_armor() scans battlefield Auras with this keyword when
            // their enchanted permanent would be destroyed, replacing destruction
            // with: remove all damage + destroy this Aura instead.
            AbilityDefinition::Keyword(KeywordAbility::UmbraArmor),
        ],
        ..Default::default()
    }
}
