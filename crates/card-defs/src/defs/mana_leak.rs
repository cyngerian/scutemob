// Mana Leak — {1}{U}, Instant
// Counter target spell unless its controller pays {3}.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("mana-leak"),
        name: "Mana Leak".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            blue: 1,
            ..Default::default()
        }),
        types: types(&[CardType::Instant]),
        oracle_text: "Counter target spell unless its controller pays {3}.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            // PB-AC2 (CR 118.12a): CounterUnlessPays — controller declines -> countered.
            effect: Effect::CounterUnlessPays {
                target: EffectTarget::DeclaredTarget { index: 0 },
                cost: Cost::Mana(ManaCost {
                    generic: 3,
                    ..Default::default()
                }),
            },
            targets: vec![TargetRequirement::TargetSpellWithFilter(
                TargetFilter::default(),
            )],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}
