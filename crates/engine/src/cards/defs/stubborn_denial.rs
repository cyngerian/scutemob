// Stubborn Denial — {U}, Instant
// Counter target noncreature spell unless its controller pays {1}.
// Ferocious — If you control a creature with power 4 or greater, counter that
// spell instead.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("stubborn-denial"),
        name: "Stubborn Denial".to_string(),
        mana_cost: Some(ManaCost { blue: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Counter target noncreature spell unless its controller pays {1}.\nFerocious — If you control a creature with power 4 or greater, counter that spell instead.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            // TODO: CounterUnlessPay not in DSL. Ferocious conditional counter also missing.
            // Using unconditional counter (stronger than non-Ferocious, correct with Ferocious).
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
