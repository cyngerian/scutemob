// Putrefy — {1}{B}{G} Instant; destroy target artifact or creature.
// It can't be regenerated.
// TODO: DSL gap — "can't be regenerated" clause not modeled (regeneration
// prevention is a no-op since regeneration itself is rarely used).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("putrefy"),
        name: "Putrefy".to_string(),
        mana_cost: Some(ManaCost { generic: 1, black: 1, green: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Destroy target artifact or creature. It can't be regenerated.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::DestroyPermanent {
                target: EffectTarget::DeclaredTarget { index: 0 },
                    cant_be_regenerated: true,
            },
            // TODO: no TargetArtifactOrCreature — using TargetPermanent as broader fallback
            targets: vec![TargetRequirement::TargetPermanent],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}
