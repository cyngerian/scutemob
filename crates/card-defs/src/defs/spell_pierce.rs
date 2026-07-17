// Spell Pierce — {U}, Instant
// Counter target noncreature spell unless its controller pays {2}.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("spell-pierce"),
        name: "Spell Pierce".to_string(),
        mana_cost: Some(ManaCost {
            blue: 1,
            ..Default::default()
        }),
        types: types(&[CardType::Instant]),
        oracle_text: "Counter target noncreature spell unless its controller pays {2}.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            // PB-AC2 (CR 118.12a): CounterUnlessPays — controller declines -> countered.
            effect: Effect::CounterUnlessPays {
                target: EffectTarget::DeclaredTarget { index: 0 },
                cost: Cost::Mana(ManaCost {
                    generic: 2,
                    ..Default::default()
                }),
            },
            targets: vec![TargetRequirement::TargetSpellWithFilter(TargetFilter {
                non_creature: true,
                ..Default::default()
            })],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}
