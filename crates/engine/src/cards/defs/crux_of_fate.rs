// Crux of Fate — {3}{B}{B}, Sorcery
// Choose one — Destroy all Dragon creatures. / Destroy all non-Dragon creatures.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("crux-of-fate"),
        name: "Crux of Fate".to_string(),
        mana_cost: Some(ManaCost { generic: 3, black: 2, ..Default::default() }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "Choose one —\n• Destroy all Dragon creatures.\n• Destroy all non-Dragon creatures.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::Sequence(vec![]),
            targets: vec![],
            modes: Some(ModeSelection {
                min_modes: 1,
                max_modes: 1,
                allow_duplicate_modes: false,
                mode_costs: None,
                modes: vec![
                    // Mode 0: Destroy all Dragon creatures.
                    Effect::DestroyAll {
                        filter: TargetFilter {
                            has_card_type: Some(CardType::Creature),
                            has_subtype: Some(SubType("Dragon".to_string())),
                            ..Default::default()
                        },
                        cant_be_regenerated: false,
                    },
                    // Mode 1: Destroy all non-Dragon creatures.
                    Effect::DestroyAll {
                        filter: TargetFilter {
                            has_card_type: Some(CardType::Creature),
                            exclude_subtypes: vec![SubType("Dragon".to_string())],
                            ..Default::default()
                        },
                        cant_be_regenerated: false,
                    },
                ],
            }),
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}
