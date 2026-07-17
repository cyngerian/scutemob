// Rewind — {2}{U}{U}, Instant
// Counter target spell. Untap up to four lands.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("rewind"),
        name: "Rewind".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            blue: 2,
            ..Default::default()
        }),
        types: types(&[CardType::Instant]),
        oracle_text: "Counter target spell. Untap up to four lands.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            // Counter the spell. TODO: "Untap up to four lands" — requires untap-N
            // permanents with land filter and "up to" choice.
            effect: Effect::CounterSpell {
                target: EffectTarget::DeclaredTarget { index: 0 },
                exile_instead: false,
            },
            targets: vec![TargetRequirement::TargetSpell],
            modes: None,
            cant_be_countered: false,
        }],
        completeness: Completeness::partial(
            "needs-rewiring: TargetRequirement::UpToN (card_definition.rs:2798) supplies the 'up \
             to four' choice the old note called missing. Remaining question is how UpToN target \
             slots map to EffectTarget::DeclaredTarget indices for the untap Sequence — trace \
             that before authoring.",
        ),
        ..Default::default()
    }
}
