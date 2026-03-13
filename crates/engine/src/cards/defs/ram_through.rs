// Ram Through — {1}{G}, Instant
// Target creature you control deals damage equal to its power to target creature you
// don't control. If the creature you control has trample, excess damage is dealt to
// that creature's controller instead.
//
// TODO: DSL gap — this spell's effect uses target[0]'s power to deal damage to target[1],
// where target[0] must be controlled by the caster and target[1] must not be. TargetRequirement
// has no "you control" / "you don't control" variants, so both targets are modeled as
// TargetCreature. The trample excess-damage clause is also not expressible in the DSL.
// The effect is approximated: target[0]'s power damages target[1]; controller restriction
// and trample excess are omitted.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("ram-through"),
        name: "Ram Through".to_string(),
        mana_cost: Some(ManaCost { generic: 1, green: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Target creature you control deals damage equal to its power to target creature you don't control. If the creature you control has trample, excess damage is dealt to that creature's controller instead.".to_string(),
        abilities: vec![
            AbilityDefinition::Spell {
                effect: Effect::DealDamage {
                    target: EffectTarget::DeclaredTarget { index: 1 },
                    amount: EffectAmount::PowerOf(EffectTarget::DeclaredTarget { index: 0 }),
                },
                targets: vec![
                    TargetRequirement::TargetCreature,
                    TargetRequirement::TargetCreature,
                ],
                modes: None,
                cant_be_countered: false,
            },
        ],
        ..Default::default()
    }
}
