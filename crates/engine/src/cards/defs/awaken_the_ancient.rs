// Awaken the Ancient — {1}{R}{R}{R} Enchantment — Aura
// Enchant Mountain
// Enchanted Mountain is a 7/7 red Giant creature with haste. It's still a land.
//
// CR 702.5a: Enchant Mountain restricts target to lands with Mountain subtype.
// CR 613.1: Layer 4 (type), Layer 5 (color), Layer 7b (P/T set), Layer 6 (ability) effects.
// CR 205.3i: Land subtypes are maintained — Mountain subtype is preserved, land type is preserved.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    let enchant_filter = EnchantFilter {
        has_card_type: Some(CardType::Land),
        has_subtype: Some(SubType("Mountain".to_string())),
        ..Default::default()
    };
    CardDefinition {
        card_id: cid("awaken-the-ancient"),
        name: "Awaken the Ancient".to_string(),
        mana_cost: Some(ManaCost { generic: 1, red: 3, ..Default::default() }),
        types: types_sub(&[CardType::Enchantment], &["Aura"]),
        oracle_text:
            "Enchant Mountain\nEnchanted Mountain is a 7/7 red Giant creature with haste. \
             It's still a land."
                .to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Enchant(EnchantTarget::Filtered(
                enchant_filter,
            ))),
            // Layer 4: Add Creature type while this Aura is on the battlefield.
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::TypeChange,
                    modification: LayerModification::AddCardTypes(
                        [CardType::Creature].into_iter().collect(),
                    ),
                    filter: EffectFilter::AttachedLand,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },
            // Layer 4: Add Giant subtype.
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::TypeChange,
                    modification: LayerModification::AddSubtypes(
                        [SubType("Giant".to_string())].into_iter().collect(),
                    ),
                    filter: EffectFilter::AttachedLand,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },
            // Layer 5: Set color to red.
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::ColorChange,
                    modification: LayerModification::SetColors(
                        [Color::Red].into_iter().collect(),
                    ),
                    filter: EffectFilter::AttachedLand,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },
            // Layer 7b: Set P/T to 7/7.
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::PtSet,
                    modification: LayerModification::SetPowerToughness { power: 7, toughness: 7 },
                    filter: EffectFilter::AttachedLand,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },
            // Layer 6: Add Haste.
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::Ability,
                    modification: LayerModification::AddKeywords(
                        [KeywordAbility::Haste].into_iter().collect(),
                    ),
                    filter: EffectFilter::AttachedLand,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },
        ],
        ..Default::default()
    }
}
