// Eaten by Piranhas — {1}{U}, Enchantment — Aura
// Flash
// Enchant creature
// Enchanted creature loses all abilities and is a black Skeleton creature with base
// power and toughness 1/1. (It loses all other colors, card types, and creature types.)
//
// PB-AC7: unblocked by LayerModification::SetCardTypes + SetCreatureTypes (Layer 4,
// CR 205.1a) — preserves supertypes, unlike SetTypeLine. Colors set directly via
// SetColors (Layer 5).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("eaten-by-piranhas"),
        name: "Eaten by Piranhas".to_string(),
        mana_cost: Some(ManaCost { generic: 1, blue: 1, ..Default::default() }),
        types: types_sub(&[CardType::Enchantment], &["Aura"]),
        oracle_text: "Flash\nEnchant creature\nEnchanted creature loses all abilities and is a black Skeleton creature with base power and toughness 1/1. (It loses all other colors, card types, and creature types.)".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flash),
            AbilityDefinition::Keyword(KeywordAbility::Enchant(EnchantTarget::Creature)),
            // Loses all abilities (Layer 6)
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::Ability,
                    modification: LayerModification::RemoveAllAbilities,
                    filter: EffectFilter::AttachedCreature,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },
            // Card types become exactly {Creature} (Layer 4, CR 205.1a) — preserves
            // supertypes (e.g. Legendary).
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::TypeChange,
                    modification: LayerModification::SetCardTypes(
                        [CardType::Creature].into_iter().collect(),
                    ),
                    filter: EffectFilter::AttachedCreature,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },
            // Creature-type subtypes become exactly {Skeleton} (Layer 4, CR 205.1a).
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::TypeChange,
                    modification: LayerModification::SetCreatureTypes(
                        [SubType("Skeleton".to_string())].into_iter().collect(),
                    ),
                    filter: EffectFilter::AttachedCreature,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },
            // Colors become exactly {Black} (Layer 5).
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::ColorChange,
                    modification: LayerModification::SetColors(
                        [Color::Black].into_iter().collect(),
                    ),
                    filter: EffectFilter::AttachedCreature,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },
            // Base P/T 1/1 (Layer 7b)
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::PtSet,
                    modification: LayerModification::SetPowerToughness { power: 1, toughness: 1 },
                    filter: EffectFilter::AttachedCreature,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },
        ],
        ..Default::default()
    }
}
