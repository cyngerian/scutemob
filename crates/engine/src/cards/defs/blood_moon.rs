// Blood Moon — {2}{R}, Enchantment
// Nonbasic lands are Mountains.
// CR 305.7: Blood Moon turns all nonbasic lands into Mountains (type-change Layer 4)
// and removes all abilities (Layer 6). This is the canonical Blood Moon pattern.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("blood-moon"),
        name: "Blood Moon".to_string(),
        mana_cost: Some(ManaCost { generic: 2, red: 1, ..Default::default() }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "Nonbasic lands are Mountains.".to_string(),
        abilities: vec![
            // Layer 4: Nonbasic lands become Mountains (type line set to "Land — Mountain").
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::TypeChange,
                    modification: LayerModification::SetTypeLine {
                        supertypes: im::OrdSet::new(),
                        card_types: [CardType::Land].into_iter().collect(),
                        subtypes: [SubType("Mountain".to_string())].into_iter().collect(),
                    },
                    filter: EffectFilter::AllNonbasicLands,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },
            // Layer 6: Remove all abilities from nonbasic lands.
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::Ability,
                    modification: LayerModification::RemoveAllAbilities,
                    filter: EffectFilter::AllNonbasicLands,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },
        ],
        ..Default::default()
    }
}
