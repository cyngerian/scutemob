// Collective Resistance — {1}{G} Instant
// Escalate {G} (Pay this cost for each mode chosen beyond the first.)
// Choose one or more —
// • Destroy target artifact.
// • Destroy target enchantment.
// • Target creature gains hexproof and indestructible until end of turn.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("collective-resistance"),
        name: "Collective Resistance".to_string(),
        mana_cost: Some(ManaCost { generic: 1, green: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Escalate {G} (Pay this cost for each mode chosen beyond the first.)\n\
            Choose one or more —\n\
            • Destroy target artifact.\n\
            • Destroy target enchantment.\n\
            • Target creature gains hexproof and indestructible until end of turn."
            .to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Escalate),
            AbilityDefinition::Escalate { cost: ManaCost { green: 1, ..Default::default() } },
            AbilityDefinition::Spell {
                effect: Effect::Sequence(vec![]),
                // Targets across all modes (flat, NOT migrated to `mode_targets` — PB-AC4):
                //   index 0: TargetArtifact (mode 0)
                //   index 1: TargetEnchantment (mode 1)
                //   index 2: TargetCreature (mode 2)
                //
                // ENGINE-BLOCKED: this card has Escalate, and the engine HARD-REJECTS
                // casting a spell that pays Escalate for 2+ modes when its
                // `ModeSelection.mode_targets` is `Some` (`casting.rs`, PB-AC4 fix-phase
                // Finding 1 — "Escalate combined with ModeSelection.mode_targets is not
                // supported"). Migrating this card would make choosing more than one mode
                // via Escalate uncastable, which is worse than the current flat-target
                // approximation. Escalate's "choose one or more" + per-mode-targets
                // combination is unsupported by design (see pb-plan-AC4.md scope
                // boundary) — do not migrate. Same reasoning as `blessed_alliance`.
                targets: vec![
                    TargetRequirement::TargetArtifact,
                    TargetRequirement::TargetEnchantment,
                    TargetRequirement::TargetCreature,
                ],
                modes: Some(ModeSelection {
                    min_modes: 1,
                    max_modes: 3,
                    allow_duplicate_modes: false,
                    mode_costs: None,
                    modes: vec![
                        // Mode 0: Destroy target artifact.
                        Effect::DestroyPermanent {
                            target: EffectTarget::DeclaredTarget { index: 0 },
                    cant_be_regenerated: false,
                        },
                        // Mode 1: Destroy target enchantment.
                        Effect::DestroyPermanent {
                            target: EffectTarget::DeclaredTarget { index: 1 },
                    cant_be_regenerated: false,
                        },
                        // Mode 2: Target creature gains hexproof and indestructible until end of turn.
                        Effect::Sequence(vec![
                            Effect::ApplyContinuousEffect {
                                effect_def: Box::new(ContinuousEffectDef {
                                    layer: EffectLayer::Ability,
                                    modification: LayerModification::AddKeyword(
                                        KeywordAbility::Hexproof,
                                    ),
                                    filter: EffectFilter::DeclaredTarget { index: 2 },
                                    duration: EffectDuration::UntilEndOfTurn,
                                    condition: None,
                                }),
                            },
                            Effect::ApplyContinuousEffect {
                                effect_def: Box::new(ContinuousEffectDef {
                                    layer: EffectLayer::Ability,
                                    modification: LayerModification::AddKeyword(
                                        KeywordAbility::Indestructible,
                                    ),
                                    filter: EffectFilter::DeclaredTarget { index: 2 },
                                    duration: EffectDuration::UntilEndOfTurn,
                                    condition: None,
                                }),
                            },
                        ]),
                    ],
                    mode_targets: None,
                }),
                cant_be_countered: false,
            },
        ],
        ..Default::default()
    }
}
