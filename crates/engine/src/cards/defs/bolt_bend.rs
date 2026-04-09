// Bolt Bend — {3}{R}, Instant
// This spell costs {3} less to cast if you control a creature with power 4 or greater.
// Change the target of target spell or ability with a single target.
//
// CR 115.7a: "Change the target" — must change to a different legal target.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("bolt-bend"),
        name: "Bolt Bend".to_string(),
        mana_cost: Some(ManaCost { generic: 3, red: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "This spell costs {3} less to cast if you control a creature with power 4 or greater.\nChange the target of target spell or ability with a single target.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            // CR 115.7a: Change the target of target spell or ability with a single target.
            // must_change: true — the target MUST be changed to a different legal target.
            // If no other legal target exists, the original target is unchanged.
            effect: Effect::ChangeTargets {
                target: EffectTarget::DeclaredTarget { index: 0 },
                must_change: true,
            },
            targets: vec![TargetRequirement::TargetSpellOrAbilityWithSingleTarget],
            modes: None,
            cant_be_countered: false,
        }],
        self_cost_reduction: Some(SelfCostReduction::ConditionalPowerThreshold {
            threshold: 4,
            reduction: 3,
        }),
        ..Default::default()
    }
}
