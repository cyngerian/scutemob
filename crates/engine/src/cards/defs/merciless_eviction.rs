// Merciless Eviction — {4}{W}{B}, Sorcery
// Choose one — Exile all artifacts / creatures / enchantments / planeswalkers.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("merciless-eviction"),
        name: "Merciless Eviction".to_string(),
        mana_cost: Some(ManaCost { generic: 4, white: 1, black: 1, ..Default::default() }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "Choose one —\n• Exile all artifacts.\n• Exile all creatures.\n• Exile all enchantments.\n• Exile all planeswalkers.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::Sequence(vec![]),
            targets: vec![],
            modes: Some(ModeSelection {
                min_modes: 1,
                max_modes: 1,
                allow_duplicate_modes: false,
                mode_costs: None,
                modes: vec![
                    Effect::ExileAll {
                        filter: TargetFilter {
                            has_card_type: Some(CardType::Artifact),
                            ..Default::default()
                        },
                    },
                    Effect::ExileAll {
                        filter: TargetFilter {
                            has_card_type: Some(CardType::Creature),
                            ..Default::default()
                        },
                    },
                    Effect::ExileAll {
                        filter: TargetFilter {
                            has_card_type: Some(CardType::Enchantment),
                            ..Default::default()
                        },
                    },
                    Effect::ExileAll {
                        filter: TargetFilter {
                            has_card_type: Some(CardType::Planeswalker),
                            ..Default::default()
                        },
                    },
                ],
            }),
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}
