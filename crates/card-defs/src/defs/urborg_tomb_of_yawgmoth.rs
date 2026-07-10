// Urborg, Tomb of Yawgmoth — Legendary Land
// Each land is a Swamp in addition to its other land types.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("urborg-tomb-of-yawgmoth"),
        name: "Urborg, Tomb of Yawgmoth".to_string(),
        mana_cost: None,
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Land],
            &[],
        ),
        oracle_text: "Each land is a Swamp in addition to its other land types.".to_string(),
        abilities: vec![
            // Layer 4: Each land gains Swamp subtype.
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::TypeChange,
                    modification: LayerModification::AddSubtypes(
                        [SubType("Swamp".to_string())].into_iter().collect(),
                    ),
                    filter: EffectFilter::AllLands,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },
        ],
        ..Default::default()
    }
}
