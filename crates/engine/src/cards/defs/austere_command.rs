// Austere Command — {4}{W}{W} Sorcery
// Choose two —
// • Destroy all artifacts.
// • Destroy all enchantments.
// • Destroy all creatures with mana value 3 or less.
// • Destroy all creatures with mana value 4 or greater.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("austere-command"),
        name: "Austere Command".to_string(),
        mana_cost: Some(ManaCost { generic: 4, white: 2, ..Default::default() }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "Choose two —\n\u{2022} Destroy all artifacts.\n\u{2022} Destroy all enchantments.\n\u{2022} Destroy all creatures with mana value 3 or less.\n\u{2022} Destroy all creatures with mana value 4 or greater.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::Sequence(vec![]),
            targets: vec![],
            modes: Some(ModeSelection {
                min_modes: 2,
                max_modes: 2,
                allow_duplicate_modes: false,
                mode_costs: None,
                modes: vec![
                    // Mode 0: Destroy all artifacts.
                    Effect::DestroyAll {
                        filter: TargetFilter {
                            has_card_type: Some(CardType::Artifact),
                            ..Default::default()
                        },
                        cant_be_regenerated: false,
                    },
                    // Mode 1: Destroy all enchantments.
                    Effect::DestroyAll {
                        filter: TargetFilter {
                            has_card_type: Some(CardType::Enchantment),
                            ..Default::default()
                        },
                        cant_be_regenerated: false,
                    },
                    // Mode 2: Destroy all creatures with mana value 3 or less.
                    Effect::DestroyAll {
                        filter: TargetFilter {
                            has_card_type: Some(CardType::Creature),
                            max_cmc: Some(3),
                            ..Default::default()
                        },
                        cant_be_regenerated: false,
                    },
                    // Mode 3: Destroy all creatures with mana value 4 or greater.
                    Effect::DestroyAll {
                        filter: TargetFilter {
                            has_card_type: Some(CardType::Creature),
                            min_cmc: Some(4),
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
