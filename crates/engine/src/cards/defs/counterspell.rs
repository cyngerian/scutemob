// 29. Counterspell — {UU}, Instant; counter target spell.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("counterspell"),
        name: "Counterspell".to_string(),
        mana_cost: Some(ManaCost { blue: 2, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Counter target spell.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::CounterSpell {
                target: EffectTarget::DeclaredTarget { index: 0 },
            },
            targets: vec![TargetRequirement::TargetSpell],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}
