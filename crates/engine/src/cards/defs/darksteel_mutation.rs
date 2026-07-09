// Darksteel Mutation — {1}{W}, Enchantment — Aura
// Enchant creature
// Enchanted creature is an Insect artifact creature with base power and toughness 0/1
// and has indestructible, and it loses all other abilities, card types, and creature types.
//
// PB-AC7: unblocked by LayerModification::SetCardTypes + SetCreatureTypes (Layer 4,
// CR 205.1a). Ordering matters (CR 613.7): the RemoveAllAbilities Static ability is
// listed BEFORE the AddKeyword(Indestructible) Static ability so it gets an earlier
// timestamp (register_static_continuous_effects assigns incrementing timestamps in
// ability-vec order) — Indestructible is granted with a LATER timestamp and survives
// the "loses all OTHER abilities" removal.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("darksteel-mutation"),
        name: "Darksteel Mutation".to_string(),
        mana_cost: Some(ManaCost { generic: 1, white: 1, ..Default::default() }),
        types: types_sub(&[CardType::Enchantment], &["Aura"]),
        oracle_text: "Enchant creature\nEnchanted creature is an Insect artifact creature with base power and toughness 0/1 and has indestructible, and it loses all other abilities, card types, and creature types.".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Enchant(EnchantTarget::Creature)),
            // Loses all OTHER abilities (Layer 6, CR 613.1f) — listed FIRST so it gets
            // an earlier timestamp than the Indestructible grant below.
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::Ability,
                    modification: LayerModification::RemoveAllAbilities,
                    filter: EffectFilter::AttachedCreature,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },
            // ...but HAS indestructible (Layer 6) — listed AFTER the removal so it
            // gets a later timestamp and survives (CR 613.7).
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::Ability,
                    modification: LayerModification::AddKeyword(KeywordAbility::Indestructible),
                    filter: EffectFilter::AttachedCreature,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },
            // Card types become exactly {Artifact, Creature} (Layer 4, CR 205.1a) —
            // preserves supertypes.
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::TypeChange,
                    modification: LayerModification::SetCardTypes(
                        [CardType::Artifact, CardType::Creature].into_iter().collect(),
                    ),
                    filter: EffectFilter::AttachedCreature,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },
            // Creature-type subtypes become exactly {Insect} (Layer 4, CR 205.1a).
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::TypeChange,
                    modification: LayerModification::SetCreatureTypes(
                        [SubType("Insect".to_string())].into_iter().collect(),
                    ),
                    filter: EffectFilter::AttachedCreature,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },
            // Base P/T 0/1 (Layer 7b)
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::PtSet,
                    modification: LayerModification::SetPowerToughness { power: 0, toughness: 1 },
                    filter: EffectFilter::AttachedCreature,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },
        ],
        ..Default::default()
    }
}
