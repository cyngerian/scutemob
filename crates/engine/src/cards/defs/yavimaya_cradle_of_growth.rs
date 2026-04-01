// Yavimaya, Cradle of Growth — Legendary Land
// Each land is a Forest in addition to its other land types.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("yavimaya-cradle-of-growth"),
        name: "Yavimaya, Cradle of Growth".to_string(),
        mana_cost: None,
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Land],
            &[],
        ),
        oracle_text: "Each land is a Forest in addition to its other land types.".to_string(),
        abilities: vec![
            // Layer 4: Each land gains Forest subtype.
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::TypeChange,
                    modification: LayerModification::AddSubtypes(
                        [SubType("Forest".to_string())].into_iter().collect(),
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
