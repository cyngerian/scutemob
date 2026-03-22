// Aetherflux Reservoir — {4} Artifact
// Whenever you cast a spell, you gain 1 life for each spell you've cast this turn.
// Pay 50 life: This artifact deals 50 damage to any target.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("aetherflux-reservoir"),
        name: "Aetherflux Reservoir".to_string(),
        mana_cost: Some(ManaCost { generic: 4, ..Default::default() }),
        types: types(&[CardType::Artifact]),
        oracle_text: "Whenever you cast a spell, you gain 1 life for each spell you've cast this turn.\nPay 50 life: Aetherflux Reservoir deals 50 damage to any target.".to_string(),
        abilities: vec![
            // Whenever you cast a spell, gain 1 life per spell cast this turn.
            // TODO: TriggerCondition::ControllerCastsSpell exists but "gain 1 life for
            // each spell you've cast this turn" needs a spell-count-this-turn tracker
            // (EffectAmount::SpellsCastThisTurn) which is not in DSL.
            // Pay 50 life: deal 50 damage to any target.
            AbilityDefinition::Activated {
                cost: Cost::PayLife(50),
                effect: Effect::DealDamage {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                    amount: EffectAmount::Fixed(50),
                },
                targets: vec![TargetRequirement::TargetAny],
                timing_restriction: None,
                activation_condition: None,
            },
        ],
        ..Default::default()
    }
}
