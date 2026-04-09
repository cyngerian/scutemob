// Untimely Malfunction — {1}{R} Instant; choose one of three modes:
// 0: Destroy target artifact.
// 1: Change the target of target spell or ability with a single target.
// 2: One or two target creatures can't block this turn.
//
// CR 115.7a: Mode 1 uses "change the target" — must_change: true.
// Target index convention (pooled across modes):
//   index 0: mode 0 — target artifact
//   index 1: mode 1 — target spell or ability with a single target
//   index 2: mode 2 — target creature (can't block)
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
            // Pooled targets across all modes:
            //   index 0: mode 0 — artifact
            //   index 1: mode 1 — spell or ability with a single target (CR 115.7a)
            //   index 2: mode 2 — creature that can't block
            targets: vec![
                TargetRequirement::TargetArtifact, // mode 0
                TargetRequirement::TargetSpellOrAbilityWithSingleTarget, // mode 1
                TargetRequirement::TargetCreature, // mode 2
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
                    // CR 115.7a: must_change: true — MUST change to a different legal target.
                    Effect::ChangeTargets {
                        target: EffectTarget::DeclaredTarget { index: 1 },
                        must_change: true,
                    },
                    // Mode 2: One or two target creatures can't block this turn.
                    // CR 509.1b: Grant CantBlock to target creature(s) until end of turn.
                    Effect::ApplyContinuousEffect {
                        effect_def: Box::new(ContinuousEffectDef {
                            layer: EffectLayer::Ability,
                            modification: LayerModification::AddKeyword(KeywordAbility::CantBlock),
                            filter: EffectFilter::DeclaredTarget { index: 2 },
                            duration: EffectDuration::UntilEndOfTurn,
                            condition: None,
                        }),
                    },
                ],
            }),
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}
