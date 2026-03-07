// Blessed Alliance — {1}{W} Instant, Escalate {2}, choose one or more
// Mode 0: Target player gains 4 life.
// Mode 1: Untap up to two target creatures. (approximated: untap DeclaredTarget 0 and 1)
// Mode 2: Target opponent sacrifices an attacking creature of their choice.
// (approximated: SacrificePermanents — no filter for attacking-only in current DSL)
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("blessed-alliance"),
        name: "Blessed Alliance".to_string(),
        mana_cost: Some(ManaCost { generic: 1, white: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Escalate {2} (Pay this cost for each mode chosen beyond the first.)\n\
            Choose one or more —\n\
            • Target player gains 4 life.\n\
            • Untap up to two target creatures.\n\
            • Target opponent sacrifices an attacking creature of their choice."
            .to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Escalate),
            AbilityDefinition::Escalate { cost: ManaCost { generic: 2, ..Default::default() } },
            AbilityDefinition::Spell {
                effect: Effect::Sequence(vec![]),
                // Targets are shared across all modes. Declared targets:
                //   index 0: TargetPlayer (mode 0 — gains 4 life)
                //   index 1: TargetCreature (mode 1 — first creature to untap)
                //   index 2: TargetCreature (mode 1 — second creature, "up to two")
                //   index 3: TargetPlayer (mode 2 — opponent who sacrifices)
                // TODO: per-mode target lists are not supported; all targets are declared
                // up front. When mode-scoped targeting is added, refactor so each mode
                // only demands its own targets.
                targets: vec![
                    TargetRequirement::TargetPlayer,   // mode 0
                    TargetRequirement::TargetCreature, // mode 1 creature 1
                    TargetRequirement::TargetCreature, // mode 1 creature 2
                    TargetRequirement::TargetPlayer,   // mode 2 opponent
                ],
                modes: Some(ModeSelection {
                    min_modes: 1,
                    max_modes: 3,
                    allow_duplicate_modes: false,
                    mode_costs: None,
                    modes: vec![
                        // Mode 0: Target player gains 4 life.
                        Effect::GainLife {
                            player: PlayerTarget::DeclaredTarget { index: 0 },
                            amount: EffectAmount::Fixed(4),
                        },
                        // Mode 1: Untap up to two target creatures.
                        // TODO: "up to two" means the second target is optional; the DSL
                        // currently requires all declared targets to be legal. A future
                        // OptionalTarget or UpTo variant would handle this correctly.
                        Effect::Sequence(vec![
                            Effect::UntapPermanent {
                                target: EffectTarget::DeclaredTarget { index: 1 },
                            },
                            Effect::UntapPermanent {
                                target: EffectTarget::DeclaredTarget { index: 2 },
                            },
                        ]),
                        // Mode 2: Target opponent sacrifices an attacking creature of their choice.
                        // TODO: SacrificePermanents has no filter for attacking creatures only.
                        // When an attacking-creature filter is added to SacrificePermanents,
                        // replace this with the filtered variant.
                        Effect::SacrificePermanents {
                            player: PlayerTarget::DeclaredTarget { index: 3 },
                            count: EffectAmount::Fixed(1),
                        },
                    ],
                }),
                cant_be_countered: false,
            },
        ],
        ..Default::default()
    }
}
