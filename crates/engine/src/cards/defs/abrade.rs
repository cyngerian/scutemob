// Abrade — {1}{R} Instant; modal: deal 3 damage to target creature OR destroy target artifact.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("abrade"),
        name: "Abrade".to_string(),
        mana_cost: Some(ManaCost { generic: 1, red: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Choose one —\n• Abrade deals 3 damage to target creature.\n• Destroy target artifact.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::Sequence(vec![]),
            targets: vec![
                TargetRequirement::TargetCreature,
                TargetRequirement::TargetArtifact,
            ],
            modes: Some(ModeSelection {
                min_modes: 1,
                max_modes: 1,
                allow_duplicate_modes: false,
                mode_costs: None,
                modes: vec![
                    // Mode 0: Deal 3 damage to target creature.
                    Effect::DealDamage {
                        target: EffectTarget::DeclaredTarget { index: 0 },
                        amount: EffectAmount::Fixed(3),
                    },
                    // Mode 1: Destroy target artifact.
                    Effect::DestroyPermanent {
                        target: EffectTarget::DeclaredTarget { index: 1 },
                    cant_be_regenerated: false,
                    },
                ],
            }),
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}
