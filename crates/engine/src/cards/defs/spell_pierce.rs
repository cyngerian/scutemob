// Spell Pierce — {U}, Instant
// Counter target noncreature spell unless its controller pays {2}.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("spell-pierce"),
        name: "Spell Pierce".to_string(),
        mana_cost: Some(ManaCost { blue: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Counter target noncreature spell unless its controller pays {2}.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            // TODO: "unless controller pays {2}" — CounterUnlessPay not in DSL.
            // Using unconditional counter (stronger than intended).
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
