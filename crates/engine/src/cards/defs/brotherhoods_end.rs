// Brotherhood's End — {1}{R}{R} Sorcery
// Choose one —
// • Brotherhood's End deals 3 damage to each creature and each planeswalker.
// • Destroy all artifacts with mana value 3 or less.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("brotherhoods-end"),
        name: "Brotherhood's End".to_string(),
        mana_cost: Some(ManaCost { generic: 1, red: 2, ..Default::default() }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "Choose one —\n\u{2022} Brotherhood's End deals 3 damage to each creature and each planeswalker.\n\u{2022} Destroy all artifacts with mana value 3 or less.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::Sequence(vec![]),
            targets: vec![],
            modes: Some(ModeSelection {
                min_modes: 1,
                max_modes: 1,
                allow_duplicate_modes: false,
                mode_costs: None,
                modes: vec![
                    // Mode 0: 3 damage to each creature and each planeswalker.
                    // CR 120.3: damage is distributed to each creature and planeswalker.
                    // The DSL DealDamage with AllPermanentsMatching covers both card types.
                    Effect::Sequence(vec![
                        Effect::DealDamage {
                            target: EffectTarget::AllPermanentsMatching(Box::new(TargetFilter {
                                has_card_type: Some(CardType::Creature),
                                ..Default::default()
                            })),
                            amount: EffectAmount::Fixed(3),
                        },
                        Effect::DealDamage {
                            target: EffectTarget::AllPermanentsMatching(Box::new(TargetFilter {
                                has_card_type: Some(CardType::Planeswalker),
                                ..Default::default()
                            })),
                            amount: EffectAmount::Fixed(3),
                        },
                    ]),
                    // Mode 1: Destroy all artifacts with mana value 3 or less.
                    Effect::DestroyAll {
                        filter: TargetFilter {
                            has_card_type: Some(CardType::Artifact),
                            max_cmc: Some(3),
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
