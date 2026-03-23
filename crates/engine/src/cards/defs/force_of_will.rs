// Force of Will — {3}{U}{U}, Instant
// You may pay 1 life and exile a blue card from your hand rather than pay this spell's mana cost.
// Counter target spell.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("force-of-will"),
        name: "Force of Will".to_string(),
        mana_cost: Some(ManaCost { generic: 3, blue: 2, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "You may pay 1 life and exile a blue card from your hand rather than pay this spell's mana cost.\nCounter target spell.".to_string(),
        abilities: vec![
            // TODO: Alternative cost — "pay 1 life and exile a blue card from your hand."
            // Requires pitch-cost alt cost (exile card from hand by color + life payment).
            // No AltCostKind variant for this pattern.
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
