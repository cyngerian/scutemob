// Maze of Ith — Land
// {T}: Untap target attacking creature. Prevent all combat damage that would be dealt to and dealt by that creature this turn.
//
// Note: "target attacking creature" approximated as TargetCreature (no TargetAttackingCreature
// variant exists). In practice, the ability is only meaningfully used during combat.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("maze-of-ith"),
        name: "Maze of Ith".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "{T}: Untap target attacking creature. Prevent all combat damage that would be dealt to and dealt by that creature this turn.".to_string(),
        abilities: vec![
            // CR 615.1: {T}: Untap target creature. Prevent all combat damage dealt to and by
            // that creature this turn. (Approximation: "attacking creature" → TargetCreature)
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::Sequence(vec![
                    Effect::UntapPermanent {
                        target: EffectTarget::DeclaredTarget { index: 0 },
                    },
                    Effect::PreventCombatDamageFromOrTo {
                        target: EffectTarget::DeclaredTarget { index: 0 },
                        prevent_from: true,
                        prevent_to: true,
                    },
                ]),
                timing_restriction: None,
                targets: vec![TargetRequirement::TargetCreature],
                activation_condition: None,
                activation_zone: None,
            },
        ],
        ..Default::default()
    }
}
