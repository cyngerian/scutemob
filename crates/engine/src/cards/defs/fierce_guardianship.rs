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
            // CR 118.9 / 2020-04-17 ruling: cast without paying mana cost if you control
            // any commander on the battlefield (any player's commander qualifies).
            AbilityDefinition::AltCastAbility {
                kind: AltCostKind::CommanderFreeCast,
                cost: ManaCost::default(),
                details: None,
            },
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
