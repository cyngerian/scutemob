// Flare of Denial — {1}{U}{U}, Instant
// You may sacrifice a nontoken blue creature rather than pay this spell's mana cost.
// Counter target spell.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("flare-of-denial"),
        name: "Flare of Denial".to_string(),
        mana_cost: Some(ManaCost { generic: 1, blue: 2, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "You may sacrifice a nontoken blue creature rather than pay this spell's mana cost.\nCounter target spell.".to_string(),
        abilities: vec![
            // TODO: Alt cost — "sacrifice a nontoken blue creature" as alternative to mana cost.
            // Requires pitch-sacrifice alt cost (color-filtered nontoken creature). Not in DSL.
            AbilityDefinition::Spell {
                effect: Effect::CounterSpell {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                },
                targets: vec![TargetRequirement::TargetSpell],
                modes: None,
                cant_be_countered: false,
            },
        ],
        ..Default::default()
    }
}
