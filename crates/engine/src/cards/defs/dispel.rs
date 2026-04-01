// Dispel — {U}, Instant
// Counter target instant spell.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("dispel"),
        name: "Dispel".to_string(),
        mana_cost: Some(ManaCost { blue: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Counter target instant spell.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::CounterSpell {
                target: EffectTarget::DeclaredTarget { index: 0 },
            },
            targets: vec![TargetRequirement::TargetSpellWithFilter(TargetFilter {
                has_card_type: Some(CardType::Instant),
                ..Default::default()
            })],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}
