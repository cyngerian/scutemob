// Untimely Malfunction — {1}{R} Instant; choose one of three modes:
// 0: Destroy target artifact.
// 1: Change the target of target spell or ability with a single target.
// 2: One or two target creatures can't block this turn.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("untimely-malfunction"),
        name: "Untimely Malfunction".to_string(),
        mana_cost: Some(ManaCost { generic: 1, red: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Choose one —\n\
            • Destroy target artifact.\n\
            • Change the target of target spell or ability with a single target.\n\
            • One or two target creatures can't block this turn."
            .to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::Sequence(vec![]),
            // Mode 0: target artifact. Modes 1 and 2 targets declared for completeness.
            targets: vec![
                TargetRequirement::TargetArtifact, // mode 0
                // TODO: mode 1 requires TargetSpellOrAbility (not in DSL)
                // TODO: mode 2 requires CantBlock effect (not in DSL)
            ],
            modes: Some(ModeSelection {
                min_modes: 1,
                max_modes: 1,
                allow_duplicate_modes: false,
                mode_costs: None,
                modes: vec![
                    // Mode 0: Destroy target artifact.
                    Effect::DestroyPermanent {
                        target: EffectTarget::DeclaredTarget { index: 0 },
                    cant_be_regenerated: false,
                    },
                    // Mode 1: Change the target of target spell or ability with a single target.
                    // TODO: ChangeTarget / RetargetSpell effect does not exist in the DSL.
                    // When it is added, implement this mode with the appropriate target selector.
                    Effect::Sequence(vec![]),
                    // Mode 2: One or two target creatures can't block this turn.
                    // TODO: CantBlock-until-EOT effect does not exist in the DSL.
                    // When it is added, implement this mode with EffectFilter::DeclaredTarget
                    // and UntilEndOfTurn duration.
                    Effect::Sequence(vec![]),
                ],
            }),
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}
