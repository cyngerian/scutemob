// 30. Negate — {1U}, Instant; counter target non-creature spell.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("negate"),
        name: "Negate".to_string(),
        mana_cost: Some(ManaCost { blue: 1, generic: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Counter target noncreature spell.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::CounterSpell {
                target: EffectTarget::DeclaredTarget { index: 0 },
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
