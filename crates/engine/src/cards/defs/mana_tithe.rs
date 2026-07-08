// Mana Tithe — {W}, Instant
// Counter target spell unless its controller pays {1}.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("mana-tithe"),
        name: "Mana Tithe".to_string(),
        mana_cost: Some(ManaCost { white: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Counter target spell unless its controller pays {1}.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            // PB-AC2 (CR 118.12a): CounterUnlessPays — controller declines -> countered.
            effect: Effect::CounterUnlessPays {
                target: EffectTarget::DeclaredTarget { index: 0 },
                cost: Cost::Mana(ManaCost { generic: 1, ..Default::default() }),
            },
            targets: vec![TargetRequirement::TargetSpellWithFilter(TargetFilter::default())],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}
