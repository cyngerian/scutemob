// Mana Drain — {U}{U}, Instant
// Counter target spell. At the beginning of your next main phase, add an amount of {C}
// equal to that spell's mana value.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("mana-drain"),
        name: "Mana Drain".to_string(),
        mana_cost: Some(ManaCost { blue: 2, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Counter target spell. At the beginning of your next main phase, add an amount of {C} equal to that spell's mana value.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            // Counter the spell. TODO: delayed trigger adding {C} equal to mana value
            // at next main phase — requires delayed triggers + mana-value tracking.
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
