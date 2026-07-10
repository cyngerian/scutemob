// Magus of the Moon — {2}{R}, Creature — Human Wizard 2/2
// Nonbasic lands are Mountains.
// (Identical effect to Blood Moon: Layer 4 SetTypeLine + Layer 6 RemoveAllAbilities on all nonbasic lands.)
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("magus-of-the-moon"),
        name: "Magus of the Moon".to_string(),
        mana_cost: Some(ManaCost { generic: 2, red: 1, ..Default::default() }),
        types: creature_types(&["Human", "Wizard"]),
        oracle_text: "Nonbasic lands are Mountains.".to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![
            // CR 613.1b/f: "Nonbasic lands are Mountains."
            // Layer 4: SetTypeLine — Land + Mountain subtype (removes all other types and subtypes).
            // CR 305.6: Basic land subtypes (Plains, Island, Swamp, Mountain, Forest) grant
            // the corresponding mana ability via layer 6. Setting type to Mountain removes
            // old abilities; the Mountain mana ability is granted implicitly by the type.
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
            // Layer 6: RemoveAllAbilities — removes any activated/triggered abilities
            // the nonbasic lands had (mana abilities, etc.). The Mountain's basic mana
            // ability ({T}: Add {R}) is re-granted implicitly via the Layer 4 type change.
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
