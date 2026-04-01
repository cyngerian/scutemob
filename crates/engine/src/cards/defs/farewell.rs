// Farewell — {4}{W}{W}, Sorcery
// Choose one or more —
// • Exile all artifacts.
// • Exile all creatures.
// • Exile all enchantments.
// • Exile all graveyards.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("farewell"),
        name: "Farewell".to_string(),
        mana_cost: Some(ManaCost { generic: 4, white: 2, ..Default::default() }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "Choose one or more —\n• Exile all artifacts.\n• Exile all creatures.\n• Exile all enchantments.\n• Exile all graveyards.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::Sequence(vec![]),
            targets: vec![],
            modes: Some(ModeSelection {
                min_modes: 1,
                max_modes: 4,
                allow_duplicate_modes: false,
                mode_costs: None,
                modes: vec![
                    // Mode 0: Exile all artifacts.
                    Effect::ExileAll {
                        filter: TargetFilter {
                            has_card_type: Some(CardType::Artifact),
                            ..Default::default()
                        },
                    },
                    // Mode 1: Exile all creatures.
                    Effect::ExileAll {
                        filter: TargetFilter {
                            has_card_type: Some(CardType::Creature),
                            ..Default::default()
                        },
                    },
                    // Mode 2: Exile all enchantments.
                    Effect::ExileAll {
                        filter: TargetFilter {
                            has_card_type: Some(CardType::Enchantment),
                            ..Default::default()
                        },
                    },
                    // Mode 3: Exile all graveyards.
                    // TODO: "exile all graveyards" — ExileAll only targets battlefield
                    // permanents. Needs a zone-scoped ExileAll or separate effect.
                    Effect::Nothing,
                ],
            }),
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}
