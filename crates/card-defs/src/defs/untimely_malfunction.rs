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
        mana_cost: Some(ManaCost {
            generic: 1,
            red: 1,
            ..Default::default()
        }),
        types: types(&[CardType::Instant]),
        oracle_text: "Choose one —\n• Destroy target artifact.\n• Change the target of target \
                      spell or ability with a single target.\n• One or two target creatures can't \
                      block this turn."
            .to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::Sequence(vec![]),
            // Pooled targets across all modes:
            //   index 0: mode 0 — artifact
            //   index 1: mode 1 — spell or ability with a single target (CR 115.7a)
            //   index 2: mode 2 — creature that can't block
            targets: vec![
                TargetRequirement::TargetArtifact,                       // mode 0
                TargetRequirement::TargetSpellOrAbilityWithSingleTarget, // mode 1
                TargetRequirement::TargetCreature,                       // mode 2
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
                    // TODO: "one or two target creatures" requires variable target count (1-2 targets),
                    // which is not expressible in the current DSL. Currently only supports one target.
                    // Same limitation as Abzan Charm mode 2.
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
                mode_targets: None,
            }),
            cant_be_countered: false,
        }],
        // PB-DX25b review Finding C2: "Modes 0 and 1 are complete" (below) was
        // UNVERIFIED at the note's original authoring, and false at PB-DX25b's
        // own HEAD (mode 1's TargetSpellOrAbilityWithSingleTarget slot could
        // never resolve a legal target before the `stack_index_for_
        // announced_target` fix -- casting.rs C1). Now VERIFIED BY PROBE:
        // `crates/engine/tests/primitives/pb_dx25b_announced_stack_target_
        // space.rs::t10_untimely_malfunction_mode1_target_index` casts with
        // mode 1 chosen and all three pooled targets declared in slot order
        // (this card uses `mode_targets: None`, the flat/pooled scheme, so
        // `casting.rs::target_count_range` requires a target for EVERY
        // pooled slot regardless of which single mode is chosen, and
        // `validate_mapped_targets` preserves DECLARATION order rather than
        // reordering to slot order -- see the probe's own doc for both
        // mechanisms) and confirms the redirect actually lands on and
        // changes the correct victim's target. The note's claim now rests on
        // executed evidence, not just prose.
        completeness: Completeness::partial(
            "mode 2 ('One or two target creatures can't block this turn') applies CantBlock to a \
             single target only. TargetRequirement::UpToN exists but has no minimum — \
             UpToN{count:2} would allow 0 targets, violating oracle's 'one or two'. Needs a \
             min/max target-count requirement (NToM). Modes 0 and 1 are complete. Same gap as \
             Abzan Charm mode 2.",
        ),
        ..Default::default()
    }
}
