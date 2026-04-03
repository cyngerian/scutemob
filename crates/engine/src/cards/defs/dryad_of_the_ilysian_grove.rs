// Dryad of the Ilysian Grove — {2}{G}, Enchantment Creature — Nymph Dryad 2/4
// You may play an additional land on each of your turns.
// Lands you control are every basic land type in addition to their other types.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("dryad-of-the-ilysian-grove"),
        name: "Dryad of the Ilysian Grove".to_string(),
        mana_cost: Some(ManaCost { generic: 2, green: 1, ..Default::default() }),
        types: full_types(
            &[],
            &[CardType::Enchantment, CardType::Creature],
            &["Nymph", "Dryad"],
        ),
        oracle_text: "You may play an additional land on each of your turns.\nLands you control are every basic land type in addition to their other types.".to_string(),
        power: Some(2),
        toughness: Some(4),
        abilities: vec![
            // CR 305.2: You may play an additional land on each of your turns.
            AbilityDefinition::AdditionalLandPlays { count: 1 },
            // CR 305.7: "Lands you control are every basic land type in addition to their
            // other types." — Layer 4 (type-changing effect). Adds all 5 basic land subtypes
            // to lands controlled by this permanent's controller.
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::TypeChange,
                    modification: LayerModification::AddSubtypes(
                        [
                            SubType("Plains".to_string()),
                            SubType("Island".to_string()),
                            SubType("Swamp".to_string()),
                            SubType("Mountain".to_string()),
                            SubType("Forest".to_string()),
                        ]
                        .into_iter()
                        .collect(),
                    ),
                    filter: EffectFilter::LandsYouControl,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },
        ],
        ..Default::default()
    }
}
