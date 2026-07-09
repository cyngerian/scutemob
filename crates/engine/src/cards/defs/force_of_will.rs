// Force of Will — {3}{U}{U}, Instant
// You may pay 1 life and exile a blue card from your hand rather than pay this spell's mana cost.
// Counter target spell.
// PB-AC5: Pitch alt cost implemented via AltCostKind::Pitch.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("force-of-will"),
        name: "Force of Will".to_string(),
        mana_cost: Some(ManaCost { generic: 3, blue: 2, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "You may pay 1 life and exile a blue card from your hand rather than pay this spell's mana cost.\nCounter target spell.".to_string(),
        abilities: vec![
            // CR 118.9: Pitch — pay 1 life and exile a blue card from hand instead of the
            // mana cost. `opponents_turn_only: false` — Force of Will's pitch cost has no
            // "not your turn" restriction (unlike Force of Vigor/Negation/Despair).
            AbilityDefinition::AltCastAbility {
                kind: AltCostKind::Pitch,
                cost: ManaCost::default(),
                details: Some(AltCastDetails::Pitch {
                    costs: vec![Cost::PayLife(1), Cost::ExileFromHand { color: Color::Blue }],
                    opponents_turn_only: false,
                }),
            },
            AbilityDefinition::Spell {
                effect: Effect::CounterSpell {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                    exile_instead: false,
                },
                targets: vec![TargetRequirement::TargetSpell],
                modes: None,
                cant_be_countered: false,
            },
        ],
        ..Default::default()
    }
}
