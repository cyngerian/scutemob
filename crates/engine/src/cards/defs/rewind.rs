// Rewind — {2}{U}{U}, Instant
// Counter target spell. Untap up to four lands.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("rewind"),
        name: "Rewind".to_string(),
        mana_cost: Some(ManaCost { generic: 2, blue: 2, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Counter target spell. Untap up to four lands.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            // Counter the spell. TODO: "Untap up to four lands" — requires untap-N
            // permanents with land filter and "up to" choice.
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
