// Kenrith's Transformation — {1}{G}, Enchantment — Aura
// Enchant creature
// When this Aura enters, draw a card.
// Enchanted creature loses all abilities and is a green Elk creature with base
// power and toughness 3/3. (It loses all other card types and creature types.)
//
// PB-AC7: unblocked by LayerModification::SetCardTypes + SetCreatureTypes (Layer 4,
// CR 205.1a), which preserve supertypes — no supertype-wipe concern here since the
// oracle text only mentions losing card types and creature types (not colors, unlike
// Eaten by Piranhas). Color is set directly via SetColors (Layer 5) since the effect
// says the creature "is a green Elk creature."
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("kenriths-transformation"),
        name: "Kenrith's Transformation".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            green: 1,
            ..Default::default()
        }),
        types: full_types(&[], &[CardType::Enchantment], &["Aura"]),
        oracle_text: "Enchant creature\nWhen this Aura enters, draw a card.\nEnchanted creature \
                      loses all abilities and is a green Elk creature with base power and \
                      toughness 3/3. (It loses all other card types and creature types.)"
            .to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Enchant(EnchantTarget::Creature)),
            // When ETB, draw a card.
            AbilityDefinition::Triggered {
                once_per_turn: false,
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
            // Enchanted creature loses all abilities (Layer 6, CR 613.1f).
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::Ability,
                    modification: LayerModification::RemoveAllAbilities,
                    filter: EffectFilter::AttachedCreature,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },
            // Enchanted creature's card types become exactly {Creature} (Layer 4,
            // CR 205.1a) — preserves supertypes (e.g. Legendary).
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
            // Enchanted creature's creature-type subtypes become exactly {Elk}
            // (Layer 4, CR 205.1a).
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::TypeChange,
                    modification: LayerModification::SetCreatureTypes(
                        [SubType("Elk".to_string())].into_iter().collect(),
                    ),
                    filter: EffectFilter::AttachedCreature,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },
            // Enchanted creature is green (Layer 5).
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::ColorChange,
                    modification: LayerModification::SetColors(
                        [Color::Green].into_iter().collect(),
                    ),
                    filter: EffectFilter::AttachedCreature,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },
            // Enchanted creature's base power/toughness becomes 3/3 (Layer 7b).
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::PtSet,
                    modification: LayerModification::SetPowerToughness {
                        power: 3,
                        toughness: 3,
                    },
                    filter: EffectFilter::AttachedCreature,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },
        ],
        ..Default::default()
    }
}
