// Fierce Guardianship — {2}{U}, Instant
// If you control a commander, you may cast this spell without paying its mana cost.
// Counter target noncreature spell.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("fierce-guardianship"),
        name: "Fierce Guardianship".to_string(),
        mana_cost: Some(ManaCost { generic: 2, blue: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "If you control a commander, you may cast this spell without paying its mana cost.\nCounter target noncreature spell.".to_string(),
        abilities: vec![
            // TODO: Conditional free-cast (control a commander) not in DSL.
            AbilityDefinition::Spell {
                effect: Effect::CounterSpell {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                },
                targets: vec![TargetRequirement::TargetSpellWithFilter(TargetFilter {
                    non_creature: true,
                    ..Default::default()
                })],
                modes: None,
                cant_be_countered: false,
            },
        ],
        ..Default::default()
    }
}
