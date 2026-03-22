// Eaten by Piranhas — {1}{U}, Enchantment — Aura
// Flash
// Enchant creature
// Enchanted creature loses all abilities and is a black Skeleton creature with base
// power and toughness 1/1. (It loses all other colors, card types, and creature types.)
//
// TODO: Color override (becomes black only) — needs LayerModification::SetColors.
// TODO: Type override (becomes Skeleton only, loses other creature types) — needs
//   LayerModification::SetSubtypes. The "loses all other card types" part means it
//   becomes only a Creature, losing any other types.
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
                },
            },
            // Base P/T 1/1 (Layer 7b)
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::PtSet,
                    modification: LayerModification::SetPowerToughness { power: 1, toughness: 1 },
                    filter: EffectFilter::AttachedCreature,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                },
            },
        ],
        ..Default::default()
    }
}
